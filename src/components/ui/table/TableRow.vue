<script setup lang="ts">
import { cn } from "@/lib/utils";

const props = defineProps<{ class?: string }>();
</script>

<template>
  <tr
    class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted"
  >
    <slot />
  </tr>
</template>
