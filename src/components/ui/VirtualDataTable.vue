<script setup lang="ts" generic="TData, TValue">
import type { ColumnDef } from "@tanstack/vue-table";
import { UnwrapRef } from "vue";
import { FlexRender, getCoreRowModel, useVueTable } from "@tanstack/vue-table";
import { useVirtualizer } from "@tanstack/vue-virtual";
import {
  ContextMenu,
  ContextMenuTrigger,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuCheckboxItem,
  ContextMenuSub,
  ContextMenuSubTrigger,
  ContextMenuSubContent,
} from "@/components/ui/context-menu";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { RowAction } from "../tables/types";

interface DataTableState<T> {
  contextMenuSubject: T | null;
}

const state = reactive<DataTableState<TData>>({
  contextMenuSubject: null,
});

const setContextMenuSubject = (subject: TData | null) => {
  state.contextMenuSubject = subject as UnwrapRef<TData>;
};

const props = defineProps<{
  stickyHeaders?: boolean;
  columns: ColumnDef<TData, TValue>[];
  rowActions?: RowAction<TData>[];
  rowClasses?: (row: TData) => string | string;
  estimatedRowHeight?: number;
  data: TData[];
  visibleColumns?: {
    [key: string]: boolean;
  };
}>();

const table = useVueTable({
  get data() {
    return props.data;
  },
  get columns() {
    return props.columns;
  },
  initialState: {
    columnVisibility: props.visibleColumns,
  },
  getCoreRowModel: getCoreRowModel(),
});

const tableContainer = ref<HTMLDivElement | null>(null);

const virtualizerOptions = computed(() => {
  return {
    count: props.data.length,
    getScrollElement: () => tableContainer.value,
    estimateSize: () => props.estimatedRowHeight || 37,
    overscan: 50,
  };
});

const virtualizer = useVirtualizer(virtualizerOptions);

const totalSize = computed(() => {
  return virtualizer.value.getTotalSize();
});
</script>

<template>
  <div ref="tableContainer">
    <div :style="{ height: `${totalSize}px` }">
      <Table class="w-full">
        <ContextMenu>
          <ContextMenuTrigger as-child>
            <TableHeader>
              <TableRow
                v-for="headerGroup in table.getHeaderGroups()"
                :key="headerGroup.id"
              >
                <TableHead
                  v-bind:enable-header-drag-region="true"
                  v-for="header in headerGroup.headers"
                  :key="header.id"
                  :style="{ width: `${header.getSize()}px` }"
                  :sticky="stickyHeaders === true"
                >
                  <FlexRender
                    v-if="!header.isPlaceholder"
                    :render="header.column.columnDef.header"
                    :props="header.getContext()"
                  />
                </TableHead>
              </TableRow>
            </TableHeader>
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuSub>
              <ContextMenuSubTrigger>Columns</ContextMenuSubTrigger>
              <ContextMenuSubContent>
                <ContextMenuCheckboxItem
                  :checked="column.getIsVisible()"
                  v-for="column in table.getAllColumns()"
                  :key="column.id"
                  @select="
                    table.setColumnVisibility({
                      [column.id]: !column.getIsVisible(),
                    })
                  "
                >
                  {{ column.columnDef.header }}
                </ContextMenuCheckboxItem>
              </ContextMenuSubContent>
            </ContextMenuSub>
          </ContextMenuContent>
        </ContextMenu>
        <ContextMenu>
          <ContextMenuTrigger as-child>
            <TableBody>
              <template v-if="table.getRowModel().rows?.length">
                <TableRow
                  v-for="row in virtualizer.getVirtualItems()"
                  :key="row.key"
                  :data-state="
                    table.getRowModel().rows[row.index].getIsSelected()
                      ? 'selected'
                      : undefined
                  "
                  :class="
                    typeof rowClasses === 'function'
                      ? rowClasses(table.getRowModel().rows[row.index].original)
                      : rowClasses || ''
                  "
                  @click.right="
                    setContextMenuSubject(
                      table.getRowModel().rows[row.index].original
                    )
                  "
                >
                  <TableCell
                    v-for="cell in table
                      .getRowModel()
                      .rows[row.index].getVisibleCells()"
                    :key="cell.id"
                    :class="
                      cell.column.columnDef.meta?.class?.(
                        table.getRowModel().rows[row.index].original
                      )
                    "
                    class="truncate overflow-hidden"
                  >
                    <FlexRender
                      :render="cell.column.columnDef.cell"
                      :props="cell.getContext()"
                    />
                  </TableCell>
                </TableRow>
              </template>
              <template v-else>
                <TableRow>
                  <TableCell :colSpan="columns.length" class="h-24 text-center">
                    No results.
                  </TableCell>
                </TableRow>
              </template>
            </TableBody>
          </ContextMenuTrigger>
          <ContextMenuContent v-if="rowActions && rowActions?.length > 0">
            <template v-for="(rowAction, index) in rowActions" :key="index">
              <ContextMenuItem
                v-if="!rowAction.options"
                @select="rowAction.handler(state.contextMenuSubject as TData)"
                >{{ rowAction.label }}</ContextMenuItem
              >
              <ContextMenuSub v-else>
                <ContextMenuSubTrigger>
                  {{ rowAction.label }}
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                  <ContextMenuItem
                    v-for="(option, optionIndex) in rowAction.options(state.contextMenuSubject as TData)"
                    :key="optionIndex"
                    @select="option.handler(state.contextMenuSubject as TData)"
                    >{{ option.label }}</ContextMenuItem
                  >
                </ContextMenuSubContent>
              </ContextMenuSub>
            </template>
          </ContextMenuContent>
        </ContextMenu>
      </Table>
    </div>
  </div>
</template>
