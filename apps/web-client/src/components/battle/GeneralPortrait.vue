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

const props = defineProps<{ general: GeneralSlotView | null; legacyDef?: number; alive: boolean; slot: number; self?: boolean }>();
const characterDef = computed(() => props.general?.def ?? props.legacyDef ?? 0);
const visible = computed(() => props.general !== null || props.legacyDef !== undefined);
const displayName = computed(() => visible.value
  ? props.general?.name || characterDisplayName(characterDef.value) || `#${characterDef.value}`
  : "暗将／未配置");
const factions: Record<string, string> = { wei: "魏", shu: "蜀", wu: "吴", qun: "群", jin: "晋", shen: "神" };
const faction = computed(() => {
  const key = props.general?.faction ?? "";
  return Object.hasOwn(factions, key) ? factions[key] : "未知势力";
});
const factionTone = computed(() => props.general?.faction ?? "unknown");
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
  <div
    class="general-slot"
    :class="{ 'is-unassigned': !visible }"
    :data-slot="slot"
    :data-faction="factionTone"
    :aria-label="`${slot === 0 ? '主将' : '副将'} ${displayName}`"
  >
    <img v-if="visible && portraitSrc" class="portrait" :src="portraitSrc" :alt="`${displayName}武将头像`" @error="onPortraitError" />
    <div v-else class="portrait no-image" role="img" :aria-label="`${displayName}暂无头像`">
      <span class="no-image-glyph">{{ visible ? "无图" : "未知" }}</span>
    </div>
    <div class="portrait-caption">
      <span class="name">{{ displayName }}</span>
      <span
        v-if="general"
        class="muted reveal-pill"
        :class="{ 'clear-card-count': !self }"
      >{{ faction }} · {{ general.revealed ? "明置" : "暗置" }}</span>
    </div>
    <select
      v-if="visible && skins.length"
      class="skin-select"
      :value="selectedSkin ?? ''"
      :aria-label="`${displayName}皮肤`"
      @click.stop
      @keydown.stop
      @change="onSkinChange"
    >
      <option value="">默认皮肤</option>
      <option v-for="skin in skins" :key="skin.id" :value="skin.id">{{ skinDisplayName(skin) }}{{ skin.presentation.kind === "spine" ? "（静态回退）" : "" }}</option>
    </select>
  </div>
</template>

<style scoped>
.general-slot {
  position: relative;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 92px;
  overflow: hidden;
  border-radius: 6px;
  background: linear-gradient(175deg, #2b231c 0%, #171412 100%);
  font-size: 0.78rem;
}
.portrait {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: top center;
  border-radius: 6px;
}
.no-image {
  display: grid;
  place-items: center;
  background: radial-gradient(circle at 50% 35%, rgba(232, 165, 75, 0.14), rgba(16, 14, 12, 0.92));
  color: rgba(232, 224, 206, 0.52);
  font-size: 0.74rem;
  letter-spacing: 0.08em;
}
.no-image-glyph {
  padding: 0.2rem 0.45rem;
  border: 1px solid rgba(232, 165, 75, 0.22);
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.35);
}
.portrait-caption {
  position: absolute;
  top: 5px;
  left: 5px;
  right: 26px;
  bottom: 4px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: space-between;
  pointer-events: none;
  z-index: 2;
}
.name {
  display: inline-block;
  padding: 0.16rem 0.2rem;
  border-radius: 4px;
  background: linear-gradient(180deg, rgba(12, 10, 9, 0.72), rgba(12, 10, 9, 0.35));
  color: #f8efd4;
  font-weight: 700;
  font-size: 0.82rem;
  line-height: 1.05;
  writing-mode: vertical-rl;
  letter-spacing: 0.06em;
  text-shadow: 0 1px 3px #000, 0 0 6px #000;
  max-height: 82%;
  overflow: hidden;
  overflow-wrap: anywhere;
}
.general-slot.is-unassigned .name {
  writing-mode: horizontal-tb;
  font-size: 0.7rem;
  color: rgba(220, 212, 196, 0.82);
  background: rgba(0, 0, 0, 0.48);
  padding: 0.12rem 0.28rem;
}
.reveal-pill {
  margin-left: 0;
  font-size: 0.64rem;
  padding: 0.06rem 0.28rem;
  border-radius: 3px;
  background: rgba(12, 10, 9, 0.78);
  border: 1px solid rgba(232, 165, 75, 0.28);
  color: #dfd4b8;
  white-space: nowrap;
}
.reveal-pill.clear-card-count {
  margin-left: 20px;
}
.skin-select {
  position: absolute;
  left: 4px;
  right: 4px;
  bottom: 4px;
  z-index: 4;
  font-size: 0.66rem;
  color: #eee4cc;
  background: rgba(14, 14, 18, 0.88);
  border: 1px solid rgba(232, 165, 75, 0.4);
  border-radius: 4px;
  padding: 0.08rem 0.2rem;
}
@media (max-width: 767px) {
  .general-slot {
    min-height: 70px;
  }
  .no-image {
    align-items: center;
    padding-top: 14px;
  }
  .no-image-glyph {
    padding: 0.1rem 0.36rem;
    font-size: 0.64rem;
  }
  .portrait-caption {
    top: 3px;
    left: 4px;
    right: 22px;
    bottom: 3px;
  }
  .name {
    writing-mode: horizontal-tb;
    font-size: 0.68rem;
    line-height: 1.1;
    max-height: none;
    max-width: 100%;
    white-space: nowrap;
    text-overflow: ellipsis;
    padding: 0.08rem 0.22rem;
  }
  .reveal-pill {
    font-size: 0.56rem;
    padding: 0.03rem 0.2rem;
  }
  .reveal-pill.clear-card-count {
    margin-left: 18px;
  }
}
</style>
