<script setup lang="ts">
import { injectStrict } from "@/lib/utils";
import { V1Deployment } from "@kubernetes/client-node";
import { Kubernetes } from "@/services/Kubernetes";
import { ref, h } from "vue";
import { useToast, ToastAction } from "@/components/ui/toast";
import { useRoute, useRouter } from "vue-router";

import { KubeContextStateKey } from "@/providers/KubeContextProvider";
const { context, namespace, kubeConfig } = injectStrict(KubeContextStateKey);

import { SettingsContextStateKey } from "@/providers/SettingsContextProvider";
const { settings } = injectStrict(SettingsContextStateKey);

import { PanelProviderAddTabKey } from "@/providers/PanelProvider";
const addTab = injectStrict(PanelProviderAddTabKey);

import {
  BaseDialogInterface,
  DialogProviderSpawnDialogKey,
} from "@/providers/DialogProvider";
const spawnDialog = injectStrict(DialogProviderSpawnDialogKey);

import DataTable from "@/components/ui/VirtualDataTable.vue";
import { RowAction, getDefaultActions } from "@/components/tables/types";
import { actions as scalableActions } from "@/actions/scalables";
import { columns } from "@/components/tables/deployments";
import { useDataRefresher } from "@/composables/refresher";

const router = useRouter();

const { toast } = useToast();
const deployments = ref<V1Deployment[]>([]);

const rowActions: RowAction<V1Deployment>[] = [
  ...getDefaultActions<V1Deployment>(
    addTab,
    spawnDialog,
    context.value,
    kubeConfig.value
  ),
  {
    label: "Logs",
    handler: (row) => {
      addTab(
        `logs_${row.metadata?.name}`,
        `${row.metadata?.name}`,
        defineAsyncComponent(() => import("@/views/StructuredLogViewer.vue")),
        {
          context: context.value,
          namespace: row.metadata?.namespace ?? namespace.value,
          kubeConfig: kubeConfig.value,
          object: `deployment/${row.metadata?.name}`,
        },
        "logs"
      );
    },
  },
  {
    label: "Port Forward",
    handler: (row) => {
      spawnDialog({
        title: "Port Forward",
        message: "Forward ports from the pod to your local machine",
        component: defineAsyncComponent(
          () => import("@/views/dialogs/PortForward.vue")
        ),
        props: {
          context: context.value,
          namespace: row.metadata?.namespace ?? namespace.value,
          kubeConfig: kubeConfig.value,
          object: row,
        },
        buttons: [],
      });
    },
  },
  {
    label: "Restart",
    massAction: true,
    handler: (rows: V1Deployment[]) => {
      const dialog: BaseDialogInterface = {
        title: "Restart deployment",
        message: `Are you sure you want to restart ${rows.length} deployment(s)?`,
        buttons: [
          {
            label: "Cancel",
            variant: "ghost",
            handler: (dialog) => {
              dialog.close();
            },
          },
          {
            label: "Restart",
            handler: (dialog) => {
              rows.forEach((row) => {
                Kubernetes.restartDeployment(
                  context.value,
                  row.metadata?.namespace || namespace.value,
                  row.metadata?.name || ""
                )
                  .then(() => {
                    dialog.close();
                  })
                  .catch((error) => {
                    dialog.close();
                    toast({
                      title: "An error occured",
                      description: error.message,
                      variant: "destructive",
                    });
                  });
              });
            },
          },
        ],
      };
      spawnDialog(dialog);
    },
  },
  ...scalableActions(
    addTab,
    spawnDialog,
    router,
    context.value,
    kubeConfig.value
  ),
];

const route = useRoute();
const rowClasses = (row: any) => {
  if (route.query.uid) {
    return row.metadata.uid === route.query.uid
      ? "animate-pulse-highlight-once"
      : "";
  }

  return "";
};

async function getDeployments(refresh: boolean = false) {
  if (!refresh) {
    deployments.value = [];
  }

  Kubernetes.getDeployments(
    context.value,
    namespace.value === "all" ? "" : namespace.value
  )
    .then((results: V1Deployment[]) => {
      deployments.value = results;
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
  getDeployments,
  1000,
  [context, namespace]
);

const create = () => {
  addTab(
    `create_` + Math.random().toString(36).substring(7),
    `New Deployment`,
    defineAsyncComponent(() => import("@/views/ObjectEditor.vue")),
    {
      context: context,
      namespace: namespace.value === "all" ? "" : namespace,
      kubeConfig: kubeConfig,
      create: true,
      type: "deployment",
      useKubeCtl: false,
    },
    "edit"
  );
};
</script>
<template>
  <DataTable
    :data="deployments"
    :columns="columns"
    :allow-filter="true"
    :sticky-headers="true"
    :row-actions="rowActions"
    :row-classes="rowClasses"
  >
    <template #action-buttons>
      <button
        class="transition-all ml-2 hover:opacity-100 opacity-50 z-50 rounded-full w-9 h-9 flex items-center justify-center bg-primary text-white text-lg"
        @click="create"
      >
        +
      </button>
    </template>
  </DataTable>
</template>
