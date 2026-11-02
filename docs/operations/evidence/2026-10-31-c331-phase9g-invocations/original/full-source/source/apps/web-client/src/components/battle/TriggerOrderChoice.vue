<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import type { ChoiceRequest } from "../../types/engine";
import { skillDisplayName } from "../../engine/appearanceStore";

const props = defineProps<{ choice: ChoiceRequest; busy: boolean; enabled: boolean }>();
const emit = defineEmits<{ select: [token: number] }>();
const root = ref<HTMLDivElement>();
const label = (key: string) => skillDisplayName(key.startsWith("skill:") ? key.slice(6) : key);
watch(() => [props.choice.choice_id, props.choice.revision, props.enabled, props.busy], async () => {
  await nextTick();
  if (props.enabled && !props.busy) root.value?.querySelector<HTMLButtonElement>("button")?.focus();
}, { immediate: true });
function navigate(event: KeyboardEvent) {
  const buttons = Array.from(root.value?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? []);
  const index = buttons.indexOf(event.target as HTMLButtonElement);
  if (index < 0 || !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) return;
  event.preventDefault();
  const next = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1
    : (index + (["ArrowLeft", "ArrowUp"].includes(event.key) ? -1 : 1) + buttons.length) % buttons.length;
  buttons[next]?.focus();
}
</script>

<template>
  <div ref="root" class="row" role="group" aria-label="选择下一个技能" @keydown="navigate">
    <button v-for="option in choice.options" :key="`${choice.choice_id}:${option.token}`"
      type="button" :disabled="busy || !enabled" @click="emit('select', option.token)">
      {{ label(option.label_key) }}
    </button>
  </div>
</template>
