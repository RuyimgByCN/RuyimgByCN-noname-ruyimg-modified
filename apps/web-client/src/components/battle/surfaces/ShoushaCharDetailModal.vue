<script setup lang="ts">
import { computed } from "vue";
import GeneralPortrait from "../GeneralPortrait.vue";
import { resolveShoushaGuanjie } from "../../../engine/surfaces/shoushaSurfacesRuntime";
import {
  dispatchShoushaUiHook,
  updateShoushaConfigInState,
  type ShoushaUiRuntimeState,
} from "../../../engine/surfaces/shoushaUiSurface";
import type { SeatView } from "../../../types/engine";

const props = defineProps<{
  seat: SeatView;
  shoushaUi: ShoushaUiRuntimeState;
}>();

const emit = defineEmits<{
  close: [];
  updateShoushaUi: [next: ShoushaUiRuntimeState];
}>();

const outcropOn = computed(() =>
  Boolean(props.shoushaUi.zlltOutcropEnabled || props.shoushaUi.config.ZLLT),
);

const legacyDef = computed(() =>
  props.seat.generals?.[0] ? undefined : (props.seat.character_def ?? 0),
);

const guanjie = computed(() => resolveShoushaGuanjie(props.seat, 5));

function toggleZlltOutcrop() {
  let next = dispatchShoushaUiHook(props.shoushaUi, "ZLLT");
  next = updateShoushaConfigInState(next, { ZLLT: !props.shoushaUi.config.ZLLT });
  emit("updateShoushaUi", next);
}
</script>

<template>
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="手杀武将详情页"
    data-testid="shousha-char-detail-modal"
    @click.self="emit('close')"
  >
    <div
      class="shousha-char-detail-card"
      :data-charactermenu="shoushaUi.config.charactermenu"
      :data-zllt-outcrop="outcropOn ? 'on' : 'off'"
      data-testid="shousha-char-detail-card"
    >
      <header class="detail-head">
        <div>
          <h3>
            P{{ seat.player }} · {{ seat.generals?.[0]?.name ?? "未点将" }}
            <span class="guanjie-pill">{{ guanjie.label }}</span>
          </h3>
          <span class="tiny muted">
            样式：{{ shoushaUi.config.charactermenu }} · 布局：{{ shoushaUi.config.yangshi === "on" ? "character/main1*" : "character/main2*" }}
          </span>
        </div>
        <button type="button" class="btn-close" data-testid="btn-close-shousha-detail" @click="emit('close')">✕</button>
      </header>

      <div class="detail-body">
        <div class="portrait-crop-frame">
          <div class="shousha-detail-portrait-img" data-testid="shousha-detail-portrait-img">
            <GeneralPortrait
              :general="seat.generals?.[0] ?? null"
              :legacy-def="legacyDef"
              :alive="seat.alive"
              :slot="0"
              :dynamic-skin-enabled="true"
              :dynamic-skin-outcrop="outcropOn"
            />
          </div>
        </div>
        <div class="detail-meta">
          <p class="tiny">
            资料页露头 (ZLLT)：<strong>{{ outcropOn ? "开启 (头肩裁剪)" : "关闭 (标准居中)" }}</strong>
          </p>
          <button
            type="button"
            class="btn-toggle-zllt"
            data-testid="btn-shousha-detail-toggle-zllt"
            @click="toggleZlltOutcrop"
          >
            切换 ZLLT 露头裁剪
          </button>
          <div class="skills-list">
            <span v-for="sk in seat.skills" :key="sk.id" class="skill-chip">
              {{ sk.name ?? `技能#${sk.def}` }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 62;
  background: rgba(6, 8, 12, 0.72);
  display: grid;
  place-items: center;
  padding: 1rem;
}
.shousha-char-detail-card {
  width: min(520px, 100%);
  border-radius: 12px;
  border: 1.5px solid #e8a54b;
  background: #141922;
  color: #f4ebd4;
  padding: 0.9rem 1.05rem;
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
}
.detail-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}
.detail-head h3 {
  margin: 0;
  font-size: 0.98rem;
}
.guanjie-pill {
  font-size: 0.72rem;
  padding: 0.08rem 0.35rem;
  border-radius: 4px;
  background: rgba(180, 83, 9, 0.55);
  border: 1px solid #fbbf24;
  margin-left: 0.35rem;
}
.detail-body {
  display: flex;
  gap: 0.85rem;
  align-items: center;
}
.portrait-crop-frame {
  width: 132px;
  height: 156px;
  overflow: hidden;
  border-radius: 8px;
  border: 1px solid rgba(251, 191, 36, 0.5);
  background: #0b0f16;
  flex-shrink: 0;
}
.shousha-detail-portrait-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.2s ease, object-position 0.2s ease;
}
.portrait-placeholder {
  width: 100%;
  height: 100%;
  display: grid;
  place-items: center;
  font-size: 0.78rem;
  color: #94a3b8;
}
.detail-meta {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}
.btn-toggle-zllt,
.btn-close {
  padding: 0.24rem 0.6rem;
  border-radius: 6px;
  border: 1px solid rgba(232, 165, 75, 0.5);
  background: rgba(232, 165, 75, 0.16);
  color: #fef3c7;
  font-size: 0.76rem;
  cursor: pointer;
}
.skills-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.skill-chip {
  font-size: 0.72rem;
  padding: 0.12rem 0.4rem;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.07);
  border: 1px solid rgba(255, 255, 255, 0.14);
}
.tiny {
  font-size: 0.72rem;
}
</style>
