<script setup lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { Child, Command } from "@tauri-apps/api/shell";
import DataTable from "@/components/ui/VirtualDataTable.vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import PlusIcon from "@/assets/icons/plus.svg";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuCheckboxItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Checkbox } from "@/components/ui/checkbox";
import { useDebounceFn } from "@vueuse/core";
import { formatSnakeCaseToHumanReadable } from "@/lib/utils";

const sessionId = ref<string>("");
const columns = ref<string[]>([]);
const facets = ref<any>([]);
const sortingState = ref<any[]>([]);
const searchQuery = ref<string>("");
let logProcess: Child | null = null;

const autoScroll = ref(true);
const liveTail = ref(true);
const currentSince = ref<string>("5m");

const logsSinceOptions = ["1m", "5m", "15m", "30m", "1h"];

const logData = ref<any[]>([]);

const props = defineProps<{
  context: string;
  namespace: string;
  object: string;
}>();

const initCommand = computed(() => {
  const initCommandArgs = [
    "logs",
    "--context",
    props.context,
    "--namespace",
    props.namespace,
    "--timestamps",
  ];

  if (liveTail.value) {
    initCommandArgs.push("--follow");
  }

  initCommandArgs.push("--since=" + currentSince.value);

  initCommandArgs.push(props.object);

  return initCommandArgs;
});

const initLogOutput = async () => {
  killProcess();

  const command = new Command("kubectl", initCommand.value);

  command.stdout.on("data", async (data: string) => {
    if (data === "") {
      return;
    }

    await invoke("add_data_to_structured_logging_session", {
      sessionId: sessionId.value,
      data: data,
    });

    updateColumns();
    updateFacetValues();

    fetchDataDebounced();
  });

  command.stderr.on("data", (data: string) => {
    console.log("stderr", data);
  });

  const child = await command.spawn();
  logProcess = child;
};

const repurposeLoggingSession = async () => {
  await invoke("repurpose_structured_logging_session", {
    sessionId: sessionId.value,
  });
};

const killProcess = async () => {
  if (logProcess) {
    logProcess.kill();
  }
};

const datatableColumns = computed(() => {
  return [
    {
      accessorKey: "timestamp",
      header: "Timestamp",
    },
    {
      accessorKey: "content",
      header: "Content",
    },
  ];
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
  fetchData();
};

const removeFacet = async (facet: string) => {
  await invoke("remove_facet_from_structured_logging_session", {
    sessionId: sessionId.value,
    property: facet,
  });

  await updateFacetValues();
  fetchData();
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

  await updateFacetValues();
  fetchData();
};

const updateFacetValues = async () => {
  facets.value = await invoke("get_facets_for_structured_logging_session", {
    sessionId: sessionId.value,
  });
};

const updateColumns = async () => {
  columns.value = await invoke("get_columns_for_structured_logging_session", {
    sessionId: sessionId.value,
  });
};

watch(
  () => searchQuery.value,
  useDebounceFn(async () => {
    await fetchData();
  }, 250)
);

const updateSorting = async (sorting: []) => {
  sortingState.value = sorting;

  fetchData();
};

const fetchDataDebounced = useDebounceFn(() => {
  fetchData();
}, 25);

const fetchData = async () => {
  if (!sessionId.value) {
    return;
  }

  const results = await invoke(
    "get_filtered_data_for_structured_logging_session",
    {
      sessionId: sessionId.value,
      searchQuery: searchQuery.value,
      sorting: sortingState.value,
    }
  );

  logData.value = results.entries;
};

const setLogsSince = async (value: string) => {
  currentSince.value = value;
  await killProcess();
  await repurposeLoggingSession();
  await initLogOutput();
};

const setLiveTail = async (value: boolean) => {
  liveTail.value = value;
  await killProcess();
  await repurposeLoggingSession();
  await initLogOutput();
};

onMounted(async () => {
  sessionId.value = await invoke("start_structured_logging_session", {
    initialData: [],
  });

  initLogOutput();
});

onUnmounted(() => {
  killProcess();
});
</script>
<template>
  <div class="absolute left-0 top-0 flex flex-row w-full h-full border-t">
    <div
      v-if="columns.length > 0"
      class="overflow-y-auto h-full min-w-[250px] max-w-[250px] border-r p-2 space-y-4"
    >
      <div class="flex justify-between items-center">
        <span class="font-bold">Filters</span>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="outline" size="icon">
              <PlusIcon class="h-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuCheckboxItem
              v-for="column in columns"
              :key="column"
              :checked="facets.find((f) => f.property === column) !== undefined"
              @click="
                !facets.find((f) => f.property === column)
                  ? addFacet(column, 'OR')
                  : removeFacet(column)
              "
              >{{
                formatSnakeCaseToHumanReadable(column)
              }}</DropdownMenuCheckboxItem
            >
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      <div v-for="facet in facets" :key="facet.property">
        <div class="flex justify-between items-center mb-4">
          <div class="font-bold">
            {{ formatSnakeCaseToHumanReadable(facet.property) }}
          </div>
          <div
            v-if="facets.length > 1"
            class="flex items-center text-xxs space-x-1 bg-secondary p-1 rounded-full"
          >
            <span
              class="rounded-full px-1 py-0.5 cursor-pointer leading-none"
              :class="{
                'text-background bg-foreground': facet.match_type == 'OR',
              }"
              @click="setFacetMatchType(facet.property, 'OR')"
              >OR</span
            >
            <span
              class="rounded-full px-1 py-0.5 cursor-pointer leading-none"
              :class="{
                'text-background bg-foreground': facet.match_type == 'AND',
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
    <div class="relative flex flex-col w-full h-full overflow-auto">
      <div class="flex p-2 space-x-2">
        <Input v-model="searchQuery" type="text" placeholder="Search..." />
        <Button variant="outline" @click="autoScroll = !autoScroll">
          <div
            class="w-2 h-2 rounded-full mr-2 bg-green-500"
            :class="{ 'bg-red-600': !autoScroll }"
          ></div>
          Autoscroll
        </Button>
        <Button
          class="flex-shrink-0"
          variant="outline"
          @click="setLiveTail(!liveTail)"
        >
          <div
            class="w-2 h-2 rounded-full mr-2 bg-green-500"
            :class="{ 'bg-red-600': !liveTail }"
          ></div>
          Live Tail
        </Button>
        <Button
          v-for="since in logsSinceOptions"
          :key="since"
          :variant="currentSince == since ? 'outline' : 'ghost'"
          @click="setLogsSince(since)"
        >
          {{ since }}
        </Button>
      </div>
      <!-- Hack to fix sticky header table data to shine through -->
      <div class="absolute h-[5px] w-full bg-background z-[9999]"></div>
      <DataTable
        :columns="datatableColumns"
        :data="logData"
        :row-classes="() => 'font-mono text-xs'"
        :estimated-row-height="37"
        :auto-scroll="autoScroll"
        sticky-headers
        @sorting-change="updateSorting"
      />
    </div>
  </div>
</template>
