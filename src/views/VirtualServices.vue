<script setup lang="ts">
import { injectStrict } from "@/lib/utils";
import { VirtualService } from "@kubernetes-models/istio/networking.istio.io/v1beta1";
import { Kubernetes } from "@/services/Kubernetes";
import { ref, h } from "vue";
import { useToast, ToastAction } from "@/components/ui/toast";

import { KubeContextStateKey } from "@/providers/KubeContextProvider";
const { context, namespace, kubeConfig } = injectStrict(KubeContextStateKey);

import DataTable from "@/components/ui/VirtualDataTable.vue";
import { columns } from "@/components/tables/virtualservices";
import { useDataRefresher } from "@/composables/refresher";

const { toast } = useToast();
const virtualServices = ref<VirtualService[]>([]);

import { RowAction, getDefaultActions } from "@/components/tables/types";
import { PanelProviderAddTabKey } from "@/providers/PanelProvider";
const addTab = injectStrict(PanelProviderAddTabKey);

import { DialogProviderSpawnDialogKey } from "@/providers/DialogProvider";
const spawnDialog = injectStrict(DialogProviderSpawnDialogKey);

const rowActions: RowAction<VirtualService>[] = [
  ...getDefaultActions<VirtualService>(
    addTab,
    spawnDialog,
    context.value,
    kubeConfig.value
  ),
];

import { useRoute } from "vue-router";
const route = useRoute();
const rowClasses = (row: any) => {
  if (route.query.uid) {
    return row.metadata.uid === route.query.uid
      ? "animate-pulse-highlight-once"
      : "";
  }

  return "";
};

async function getVirtualServices(refresh: boolean = false) {
  if (!refresh) {
    virtualServices.value = [];
  }

  Kubernetes.getVirtualServices(
    context.value,
    namespace.value === "all" ? "" : namespace.value
  )
    .then((results: VirtualService[]) => {
      virtualServices.value = results;
    })
    .catch((error) => {
      toast({
        title: "An error occured",
        description: error.message,
        variant: "destructive",
        action: h(
          ToastAction,
          { altText: "Retry", onClick: () => startRefreshing() },
          { default: () => "Retry" }
        ),
      });
      stopRefreshing();
    });
}

const { startRefreshing, stopRefreshing } = useDataRefresher(
  getVirtualServices,
  1000,
  [context, namespace]
);
</script>
<template>
  <DataTable
    :data="virtualServices"
    :columns="columns"
    :allow-filter="true"
    :sticky-headers="true"
    :row-actions="rowActions"
    :row-classes="rowClasses"
  />
</template>
