<script setup lang="ts">
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectGroup,
  SelectLabel,
  SelectItem,
} from "@/components/ui/select";

import { AlertDialogFooter } from "@/components/ui/alert-dialog";
import { Command } from "@tauri-apps/plugin-shell";

import { useToast } from "@/components/ui/toast";
const { toast } = useToast();

import { parseJSON } from "date-fns";
import { formatDateTime } from "@/lib/utils";
import { error } from "@/lib/logger";

const rollbackRevision = ref<string>("");
const revisions = ref<any[]>([]);

const sortedRevisions = computed(() => {
  // Sort the revisions in descending order
  return revisions.value.slice().sort((a, b) => b.revision - a.revision);
});

const props = defineProps<{
  context: string;
  namespace: string;
  kubeConfig: string;
  release: any;
}>();

const emit = defineEmits(["closeDialog"]);

const rollback = () => {
  const args = [
    "rollback",
    props.release.name,
    rollbackRevision.value.toString(),
    "--kubeconfig",
    props.kubeConfig,
    "--kube-context",
    props.context,
    "--namespace",
    props.release.namespace,
  ];

  const command = Command.create("helm", args);
  command.stdout.on("data", (data) => {
    toast({
      title: "Rollback successful",
      description: `Rollback of ${props.release.name} to revision ${rollbackRevision.value} was successful`,
      autoDismiss: true,
    });
    emit("closeDialog");
  });

  command.stderr.on("data", (data) => {
    error(`Error rolling back release ${props.release.name} to revision ${rollbackRevision.value}: ${data}`);
    toast({
      title: "Error",
      description: data,
      variant: "destructive",
    });
  });

  command.spawn();
};

const fetchRevisions = async () => {
  const args = [
    "history",
    props.release.name,
    "--kubeconfig",
    props.kubeConfig,
    "--output",
    "json",
    "--kube-context",
    props.context,
    "--namespace",
    props.release.namespace,
  ];

  const command = Command.create("helm", args);
  command.stdout.on("data", (data) => {
    const parsedData = JSON.parse(data);

    revisions.value = parsedData;
  });

  command.stderr.on("data", (data) => {
    error(`Error fetching Helm release history: ${data}`);
  });

  command.spawn();
};

onMounted(() => {
  fetchRevisions();
});
</script>
<template>
  <Select v-model="rollbackRevision">
    <SelectTrigger>
      <SelectValue />
    </SelectTrigger>
    <SelectContent>
      <SelectGroup>
        <SelectLabel>Revisions</SelectLabel>
        <template v-for="revision in sortedRevisions" :key="revision.revision">
          <SelectItem :value="revision.revision">
            {{ revision.revision }} - {{ revision.chart }} -
            {{ revision.app_version }} -
            {{ formatDateTime(parseJSON(revision.updated)) }}
          </SelectItem>
        </template>
      </SelectGroup>
    </SelectContent>
  </Select>
  <AlertDialogFooter>
    <Button variant="default" @click="emit('closeDialog')">Cancel</Button>
    <Button variant="default" @click="rollback">Rollback</Button>
  </AlertDialogFooter>
</template>
