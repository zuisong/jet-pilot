pub mod structured_logging {
    use std::collections::{HashMap, HashSet};
    use std::
        sync::Mutex
    ;
    use uuid::Uuid;

    static STRUCTURED_LOGGING_SESSIONS: Mutex<Option<HashMap<String, StructuredLoggingSession>>> = Mutex::new(None);

    #[derive(Debug)]
    pub struct StructuredLoggingSession {
        data: Vec<serde_json::Value>,
        facets: Vec<Facet>,
    }

    #[derive(Clone, Debug, serde::Serialize)]
    pub enum MatchType {
        AND,
        OR,
    }

    #[derive(Clone, Debug, serde::Serialize)]
    pub struct Facet {
        property: String,
        // matchType should be enum for AND or OR
        matchType: MatchType,
        values: Vec<FacetValue>,
    }

    #[derive(Clone, Debug, serde::Serialize)]
    pub struct FacetValue {
        value: String,
        filtered: bool,
        total: u32,
    }

    #[derive(Clone, Debug, serde::Serialize)]
    pub struct FilteredLogResult {
        data: Vec<serde_json::Value>,
        filtered_total: u32,
        total: u32,
    }

    #[tauri::command]
    pub async fn start_structured_logging_session(initial_data: Vec<String>) -> String {
        // generate a random session id
        let session_id = Uuid::new_v4().to_string();
        

        if STRUCTURED_LOGGING_SESSIONS.lock().unwrap().is_none() {
            *STRUCTURED_LOGGING_SESSIONS.lock().unwrap() = Some(HashMap::new());
        }

        STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().insert(
            session_id.clone(),
            StructuredLoggingSession {
                data: initial_data.into_iter().map(|d|
                    serde_json::from_str(&d).unwrap_or_else(|_| serde_json::Value::String(d))
                ).collect(),
                facets: Vec::new(),
            },
        );

        return session_id;
    }

    #[tauri::command]
    pub async fn add_data_to_structured_logging_session(session_id: String, data: Vec<String>) {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            session.data.extend(data.into_iter().map(|d| serde_json::Value::String(d)));
        }

        update_unique_facet_values_for_logging_session(session_id);
    }

    #[tauri::command]
    pub async fn add_facet_to_structured_logging_session(session_id: String, property: String, match_type: String) {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            let match_type = match match_type.as_str() {
                "AND" => MatchType::AND,
                "OR" => MatchType::OR,
                _ => MatchType::OR,
            };
            session.facets.push(Facet {
                property,
                matchType: match_type,
                values: Vec::new(),
            });
        }

        update_unique_facet_values_for_logging_session(session_id);
    }

    #[tauri::command]
    pub async fn set_facet_match_type_for_structured_logging_session(session_id: String, property: String, match_type: String) {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            for facet in session.facets.iter_mut() {
                if facet.property == property {
                    facet.matchType = match match_type.as_str() {
                        "AND" => MatchType::AND,
                        "OR" => MatchType::OR,
                        _ => MatchType::OR,
                    };
                    break;
                }
            }
        }
    }

    #[tauri::command]
    pub async fn remove_facet_from_structured_logging_session(session_id: String, property: String) {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            session.facets.retain(|f| f.property != property);
        }

        update_unique_facet_values_for_logging_session(session_id);
    }

    #[tauri::command]
    pub async fn set_filtered_for_facet_value(session_id: String, property: String, value: String, filtered: bool) {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            for facet in session.facets.iter_mut() {
                if facet.property == property {
                    for facet_value in facet.values.iter_mut() {
                        if facet_value.value == value {
                            facet_value.filtered = filtered;
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }

    #[tauri::command]
    pub async fn get_facets_for_structured_logging_session(session_id: String) -> Vec<Facet> {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            return session.facets.clone();
        }

        return Vec::new();
    }

    #[tauri::command]
    pub async fn get_filtered_data_for_structured_logging_session(session_id: String, search_query: String, offset: u32, limit: u32) -> FilteredLogResult {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            let mut filtered_data = Vec::new();
            let mut filtered_data_count = 0;
            let mut any_filtered = false;
            let mut facet_matches: Vec<HashMap<String, HashSet<usize>>> = Vec::new();
        
            // Collect all matching rows for each filtered facet value
            for facet in session.facets.iter() {
                let mut facet_matched_indices: HashSet<usize> = HashSet::new();

                for facet_value in facet.values.iter() {
                    if facet_value.filtered {
                        any_filtered = true;
                        for (index, data) in session.data.iter().enumerate() {
                            if let Some(value) = data.get(&facet.property).and_then(|v| v.as_str()) {
                                if value == &facet_value.value {
                                    facet_matched_indices.insert(index);
                                }
                            }
                        }
                    }
                }

                if !facet_matched_indices.is_empty() {
                    facet_matches.push(HashMap::from([(facet.property.clone(), facet_matched_indices)]));
                }
            }

            let mut matched_indices = HashSet::new();

            if !facet_matches.is_empty() {
                // mathced indices based on AND or OR
                matched_indices = facet_matches[0].iter().map(|(_, indices)| indices.clone()).flatten().collect();
                for (property, indices) in facet_matches.iter().skip(1).flat_map(|m| m.iter()) {
                    // get matchtype for property
                    let facet = session.facets.iter().find(|f| f.property == *property).unwrap();
                    match facet.matchType {
                        MatchType::OR => {
                            matched_indices = matched_indices.union(indices).cloned().collect();
                        }
                        MatchType::AND => {
                            matched_indices = matched_indices.intersection(indices).cloned().collect();
                        }
                    }
                }
            }
        
            // Collect the filtered data based on matched indices if any filters are applied
            if any_filtered {
                // Apply search query if any and filter the matched indices
                if !search_query.is_empty() {
                    let search_query = search_query.to_lowercase();
                    matched_indices = matched_indices.into_iter().filter(|index| {
                        let data = &session.data[*index];
                        for value in data.as_object().unwrap().values() {
                            if value.is_string() {
                                if value.as_str().unwrap().to_lowercase().contains(&search_query) {
                                    return true;
                                }
                            }

                            if value.is_object() {
                                let serialized_value = serde_json::to_string(value).unwrap();
                                if serialized_value.to_lowercase().contains(&search_query) {
                                    return true;
                                }
                            }
                        }
                        return false;
                    }).collect();
                }

                for index in matched_indices.iter() {
                    if filtered_data_count >= offset {
                        filtered_data.push(session.data[*index].clone());
                    }
                    filtered_data_count += 1;
                    if filtered_data_count >= offset + limit {
                        break;
                    }
                }
            } else {
                // if any search query, apply query and then return data within offset and limit, otherwise return all data within offset and limit
                if !search_query.is_empty() {
                    let search_query = search_query.to_lowercase();
                    for (_index, data) in session.data.iter().enumerate() {
                        if data.as_object().is_none() {
                            continue;
                        }

                        for value in data.as_object().unwrap().values() {
                            if value.is_string() {
                                if value.as_str().unwrap().to_lowercase().contains(&search_query) {
                                    if filtered_data_count >= offset {
                                        filtered_data.push(data.clone());
                                    }
                                    filtered_data_count += 1;
                                    break;
                                }
                            }

                            if value.is_object() {
                                let serialized_value = serde_json::to_string(value).unwrap();
                                if serialized_value.to_lowercase().contains(&search_query) {
                                    if filtered_data_count >= offset {
                                        filtered_data.push(data.clone());
                                    }
                                    filtered_data_count += 1;
                                    break;
                                }
                            }
                        }
                        if filtered_data_count >= offset + limit {
                            break;
                        }
                    }
                } else {
                    for (_index, data) in session.data.iter().enumerate() {
                        if filtered_data_count >= offset {
                            filtered_data.push(data.clone());
                        }
                        filtered_data_count += 1;
                        if filtered_data_count >= offset + limit {
                            break;
                        }
                    }
                }
            }

            return FilteredLogResult {
                data: filtered_data,
                filtered_total: filtered_data_count,
                total: session.data.len() as u32,
            };
        }

        return FilteredLogResult {
            data: Vec::new(),
            filtered_total: 0,
            total: 0,
        };
    }

    fn update_unique_facet_values_for_logging_session(session_id: String) {
        if let Some(session) = STRUCTURED_LOGGING_SESSIONS.lock().unwrap().as_mut().unwrap().get_mut(&session_id) {
            for facet in session.facets.iter_mut() {
                
                let mut values = HashMap::new();
                for data in session.data.iter() {
                    if let Some(value) = data.get(&facet.property).and_then(|v| v.as_str()) {
                        *values.entry(value).or_insert(0) += 1;
                    }
                }

                for (value, total) in values.iter() {
                    if let Some(facet_value) = facet.values.iter_mut().find(|v| v.value == *value) {
                        facet_value.total = *total;
                    } else {
                        facet.values.push(FacetValue {
                            value: value.to_string(),
                            filtered: false,
                            total: *total,
                        });
                    }
                }
            }
        }
    }
}