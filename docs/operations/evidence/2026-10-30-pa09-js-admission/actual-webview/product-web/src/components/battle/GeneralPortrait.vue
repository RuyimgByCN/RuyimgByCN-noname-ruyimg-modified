<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  characterDisplayName,
  characterImageUrl,
  characterPortraitUrl,
  ensureAppearanceCharacter,
  fallbackPortraitUrl,
  selectedSkinId,
  selectSkin,
  skinDisplayName,
  skinsForCharacter,
  substituteImageUrl,
} from "../../engine/appearanceStore";
import type { GeneralSlotView } from "../../types/engine";

const props = defineProps<{ general: GeneralSlotView | null; legacyDef?: number; alive: boolean; slot: number }>();
const characterDef = computed(() => props.general?.def ?? props.legacyDef ?? 0);
const visible = computed(() => props.general !== null || props.legacyDef !== undefined);
const displayName = computed(() => visible.value
  ? props.general?.name || characterDisplayName(characterDef.value) || `#${characterDef.value}`
  : "暗将／未配置");
const factions: Record<string, string> = { wei: "魏", shu: "蜀", wu: "吴", qun: "群" };
const faction = computed(() => {
  const key = props.general?.faction ?? "";
  return Object.hasOwn(factions, key) ? factions[key] : "未知势力";
});
const skins = computed(() => skinsForCharacter(characterDef.value));
const selectedSkin = computed(() => selectedSkinId(characterDef.value));
const selectedPortrait = computed(() =>
  characterImageUrl(characterDef.value)
);
const basePortrait = computed(() =>
  characterPortraitUrl(characterDef.value)
);
const originalPortrait = computed(() =>
  fallbackPortraitUrl(characterDef.value)
);
const portraitCandidates = computed(() =>
  Array.from(
    new Set([
      ...(props.alive
        ? []
        : [substituteImageUrl(characterDef.value, "dead")]),
      selectedPortrait.value,
      basePortrait.value,
      originalPortrait.value,
    ])
  ).filter((url): url is string => !!url)
);
const failedAssets = ref<string[]>([]);
const portraitSrc = computed(
  () =>
    portraitCandidates.value.find(
      (url) => !failedAssets.value.includes(url)
    ) ?? ""
);

watch(
  [characterDef, visible],
  ([characterId, isVisible]) => {
    if (isVisible) void ensureAppearanceCharacter(characterId).catch(() => undefined);
  },
  { immediate: true }
);

watch(portraitCandidates, () => {
  failedAssets.value = [];
});

function onPortraitError(): void {
  if (portraitSrc.value) {
    failedAssets.value = [...failedAssets.value, portraitSrc.value];
  }
}

function onSkinChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value;
  selectSkin(characterDef.value, value === "" ? null : Number(value));
}

</script>

<template>
  <div class="general-slot" :data-slot="slot" :aria-label="`${slot === 0 ? '主将' : '副将'} ${displayName}`">
    <img v-if="visible && portraitSrc" class="portrait" :src="portraitSrc" :alt="`${displayName}武将头像`" @error="onPortraitError" />
    <div v-else class="portrait no-image" role="img" :aria-label="`${displayName}暂无头像`">{{ visible ? "无图" : "未知" }}</div>
    <span class="name">{{ displayName }}</span>
    <span v-if="general" class="muted">{{ faction }} · {{ general.revealed ? "明置" : "暗置" }}</span>
    <select v-if="visible && skins.length" class="skin-select" :value="selectedSkin ?? ''" :aria-label="`${displayName}皮肤`" @click.stop @keydown.stop @change="onSkinChange">
      <option value="">默认皮肤</option>
      <option v-for="skin in skins" :key="skin.id" :value="skin.id">{{ skinDisplayName(skin) }}{{ skin.presentation.kind === "spine" ? "（静态回退）" : "" }}</option>
    </select>
  </div>
</template>

<style scoped>
.general-slot { display: grid; gap: 0.15rem; min-width: 0; font-size: 0.8rem; }
.portrait { width: 100%; height: 5rem; object-fit: cover; object-position: top center; border-radius: 5px; }
.no-image { display: grid; place-items: center; background: var(--bg-elevated); color: var(--muted); }
.name { font-weight: 650; overflow-wrap: anywhere; }
.skin-select { width: 100%; color: var(--muted); background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 4px; }
</style>
