<script setup lang="ts">
import { BaseDirectory, readTextFile } from "@tauri-apps/api/fs";
import { invoke } from "@tauri-apps/api/tauri";
import DataTable from "@/components/ui/UnmanagedVirtualDataTable.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import FiltersIcon from "@/assets/icons/filters.svg";
import AutoColumnsIcon from "@/assets/icons/auto_columns.svg";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Checkbox } from "@/components/ui/checkbox";
import { ColumnDef } from "@tanstack/vue-table";
import { useDebounceFn, useScroll } from "@vueuse/core";

const sessionId = ref<string>("");
const columns = ref<string[]>([]);
const facets = ref<any>([]);
const sortingState = ref<any[]>([]);
const searchQuery = ref<string>("");
const tableScrollContainer = ref<HTMLDivElement | null>(null);
const scrollState = useScroll(tableScrollContainer);

const dataLength = ref(0);
const logData = ref<any[]>([]);

const generateColumns = () => {
  if (typeof logData.value[0] === "object") {
    const keys = new Set<string>();

    Object.keys(logData.value[0]).forEach((key) => {
      keys.add(key);
    });

    columns.value = Array.from(keys);

    facets.value = facets.value.filter((facet) =>
      columns.value.includes(facet)
    );
  }
};

const datatableColumns = computed(() => {
  if (columns.value.length === 0) {
    return [
      {
        accessorKey: "message",
        header: "Message",
      },
    ];
  }

  return columns.value.map((column) => {
    return {
      accessorKey: column,
      header: column,
      cell: ({ cell }) => {
        if (typeof cell.getValue() === "object") {
          return JSON.stringify(cell.getValue());
        }

        return cell.getValue();
      },
    } as ColumnDef<any>;
  });
});

const addFacet = async (facet: string, matchType: "AND" | "OR") => {
  await invoke("add_facet_to_structured_logging_session", {
    sessionId: sessionId.value,
    matchType: matchType,
    property: facet,
  });

  updateFacetValues();
};

const setFacetMatchType = async (facet: string, matchType: "AND" | "OR") => {
  await invoke("set_facet_match_type_for_structured_logging_session", {
    sessionId: sessionId.value,
    property: facet,
    matchType: matchType,
  });

  await updateFacetValues();
  fetchVirtualPage();
};

const removeFacet = (facet: string) => {
  facets.value = facets.value.filter((f) => f !== facet);
};

const setFilteredForFacetValue = async (
  facet: string,
  value: string,
  filtered: boolean
) => {
  await invoke("set_filtered_for_facet_value", {
    sessionId: sessionId.value,
    property: facet,
    value: value,
    filtered: filtered,
  });

  scrollState.y.value = 0;

  await updateFacetValues();
  fetchVirtualPage();
};

const updateFacetValues = async () => {
  facets.value = await invoke("get_facets_for_structured_logging_session", {
    sessionId: sessionId.value,
  });
};

const handleScroll = useDebounceFn(async () => {
  if (scrollState.y.value >= dataLength.value * 31) {
    return;
  }

  await fetchVirtualPage();
}, 25);

watch(
  () => searchQuery.value,
  useDebounceFn(async () => {
    scrollState.y.value = 0;
    await fetchVirtualPage();
  }, 100)
);

const updateSorting = async (sorting: []) => {
  sortingState.value = sorting;

  fetchVirtualPage();
};

const fetchVirtualPage = async () => {
  if (!sessionId.value || !tableScrollContainer.value) {
    return;
  }

  const rowHeight = 31;

  const offset = Math.max(Math.floor(scrollState.y.value / rowHeight) - 250, 0);
  const limit = 200;

  const results = await invoke(
    "get_filtered_data_for_structured_logging_session",
    {
      sessionId: sessionId.value,
      searchQuery: searchQuery.value,
      offset,
      limit,
      sorting: sortingState.value,
    }
  );

  logData.value = results.data;
  dataLength.value = results.total;
};

onMounted(async () => {
  const data = (
    await readTextFile("raw-logs.txt", {
      dir: BaseDirectory.AppConfig,
    })
  ).split("\n");

  sessionId.value = await invoke("start_structured_logging_session", {
    initialData: data,
  });

  await fetchVirtualPage();
});
</script>
<template>
  <div class="absolute left-0 top-0 flex flex-row w-full h-full border-t">
    <div
      class="overflow-y-auto h-full min-w-[250px] max-w-[250px] border-r p-2 space-y-4"
      v-if="columns.length > 0"
    >
      <div class="flex justify-between items-center">
        <span class="font-bold">Filters</span>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="outline" size="icon">
              <FiltersIcon class="h-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuCheckboxItem
              v-for="column in columns"
              :key="column"
              :checked="facets.includes(column)"
              @click="
                !facets.includes(column)
                  ? addFacet(column, 'OR')
                  : removeFacet(column)
              "
              >{{ column }}</DropdownMenuCheckboxItem
            >
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      <div v-for="facet in facets" :key="facet.property">
        <div class="flex justify-between items-center mb-4">
          <div class="font-bold">{{ facet.property }}</div>
          <div
            class="flex items-center text-xxs space-x-1 bg-secondary p-1 rounded-full"
          >
            <span
              class="rounded-full px-1 py-0.5 cursor-pointer leading-none"
              :class="{
                'text-background bg-foreground': facet.matchType == 'OR',
              }"
              @click="setFacetMatchType(facet.property, 'OR')"
              >OR</span
            >
            <span
              class="rounded-full px-1 py-0.5 cursor-pointer leading-none"
              :class="{
                'text-background bg-foreground': facet.matchType == 'AND',
              }"
              @click="setFacetMatchType(facet.property, 'AND')"
              >AND</span
            >
          </div>
        </div>
        <ul class="border rounded-lg">
          <li
            v-for="value in facet.values.sort((a, b) => b.total - a.total)"
            :key="value.value"
            class="flex items-center justify-between p-3 border-b last:border-b-0"
            :title="value.value"
          >
            <label
              :for="`facet-${facet.property}-${value.value}`"
              class="flex truncate space-x-2 cursor-pointer"
            >
              <Checkbox
                :id="`facet-${facet.property}-${value.value}`"
                class="border-secondary"
                :checked="value.filtered"
                @update:checked="
                  setFilteredForFacetValue(
                    facet.property,
                    value.value,
                    !value.filtered
                  )
                "
              />
              <span class="text-xs truncate">{{ value.value }}</span>
            </label>
            <span class="bg-secondary text-foreground rounded-full px-2">{{
              value.total
            }}</span>
          </li>
        </ul>
      </div>
    </div>
    <div class="relative flex flex-col flex-grow h-full overflow-auto">
      <div class="flex p-2 space-x-2">
        <Button
          variant="outline"
          @click="generateColumns"
          class="flex-shrink-0"
        >
          <AutoColumnsIcon class="h-4 mr-2" /> Auto Columns
        </Button>
        <Input v-model="searchQuery" type="text" placeholder="Search..." />
      </div>
      <!-- Hack to fix sticky header table data to shine through -->
      <div class="h-[1px] w-full bg-background z-50"></div>
      <div
        class="flex-grow w-full overflow-auto"
        ref="tableScrollContainer"
        @scroll="handleScroll"
      >
        <DataTable
          :columns="datatableColumns"
          :data="logData"
          :row-classes="() => 'font-mono text-xs'"
          :data-length="dataLength"
          :estimated-row-height="31"
          :scroll-offset="tableScrollContainer?.scrollTop || 0"
          @sorting-change="updateSorting"
          sticky-headers
        />
      </div>
    </div>
  </div>
</template>
