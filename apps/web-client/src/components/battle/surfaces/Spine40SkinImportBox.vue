<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import {
  CleanRoomAnimationState,
  HANDCRAFTED_SPINE40_ATLAS_TEXT,
  HANDCRAFTED_SPINE40_SKELETON_JSON,
  buildBatchGeometry,
  createSkeletonInstance,
  parseSpine40Json,
  parseTextureAtlas,
  renderSpineBatchesToCanvas2D,
  renderSpineBatchesToWebGL,
  resolveDynamicSkinImport,
  type MinimalWebGLContext,
  type SkeletonData,
  type TextureAtlas,
} from "../../../engine/spine40/index";

const props = defineProps<{
  activeClip: string;
  standX: number;
  standY: number;
  scalePct: number;
}>();

const emit = defineEmits<{
  (e: "trigger-clip"): void;
  (e: "import-status", msg: string): void;
}>();

const canvas2dRef = ref<HTMLCanvasElement | null>(null);
const webglCanvasRef = ref<HTMLCanvasElement | null>(null);

const cleanroomSkeleton = parseSpine40Json(HANDCRAFTED_SPINE40_SKELETON_JSON);
const cleanroomAtlas = parseTextureAtlas(HANDCRAFTED_SPINE40_ATLAS_TEXT);

const showImportForm = ref(false);
const paramGeneralId = ref("caocao");
const paramSkinName = ref("玩家自备动皮");
const paramVersion = ref("4.0");
const paramAction = ref("DaiJi");
const paramScale = ref(1.0);
const importedRawAsset = ref<unknown>(null);
const importedAtlasText = ref<string>("");
const lastGlStats = ref<{ drawCallCount: number; triangleCount: number; shaderProgramActive: boolean }>({
  drawCallCount: 0,
  triangleCount: 0,
  shaderProgramActive: false,
});

const skinResolution = computed(() =>
  resolveDynamicSkinImport(
    {
      generalId: paramGeneralId.value,
      skinName: paramSkinName.value,
      x: [0, props.standX],
      y: [0, props.standY],
      scale: paramScale.value * (props.scalePct / 100),
      action: paramAction.value,
      version: paramVersion.value,
    },
    importedRawAsset.value,
  ),
);

const activeSkeleton = computed<SkeletonData>(() => skinResolution.value.skeletonData ?? cleanroomSkeleton);
const activeAtlas = computed<TextureAtlas>(() =>
  importedAtlasText.value.trim() ? parseTextureAtlas(importedAtlasText.value) : cleanroomAtlas,
);

function renderCurrentPose() {
  const skelData = activeSkeleton.value;
  const atlas = activeAtlas.value;
  const clipLower = props.activeClip || "daiji";
  const animNames = Object.keys(skelData.animations);
  const chosenAnim = skelData.animations[clipLower]
    ? clipLower
    : (animNames.find((k) => k.toLowerCase() === paramAction.value.toLowerCase()) ?? animNames[0] ?? "daiji");

  const instance = createSkeletonInstance(skelData, skelData.skins[0]?.name ?? "default");
  const state = new CleanRoomAnimationState();
  if (skelData.animations[chosenAnim]) {
    state.setAnimation(0, chosenAnim, true, 1, "replace");
    state.update(0.3);
    state.apply(instance);
  }
  const hidden = new Set(skinResolution.value.normalizedParams.hideSlots);
  const batches = buildBatchGeometry(instance, atlas, true, hidden);

  const c2d = canvas2dRef.value?.getContext("2d");
  if (c2d) {
    renderSpineBatchesToCanvas2D(c2d, instance, batches);
  }

  const glCanvas = webglCanvasRef.value;
  if (glCanvas) {
    const gl = (glCanvas.getContext("webgl", { premultipliedAlpha: true }) ??
      glCanvas.getContext("experimental-webgl")) as unknown as MinimalWebGLContext | null;
    if (gl) {
      lastGlStats.value = renderSpineBatchesToWebGL(gl, batches, glCanvas.width, glCanvas.height);
    }
  }
}

async function onSkeletonFileChange(ev: Event) {
  const input = ev.target as HTMLInputElement | null;
  const file = input?.files?.[0];
  if (!file) return;
  const buf = await file.arrayBuffer();
  if (file.name.toLowerCase().endsWith(".skel")) {
    importedRawAsset.value = new Uint8Array(buf);
  } else {
    importedRawAsset.value = new TextDecoder("utf-8").decode(buf);
  }
  emit("import-status", `动皮导入 (${file.name}) → ${skinResolution.value.status}: ${skinResolution.value.reason}`);
  renderCurrentPose();
}

async function onAtlasFileChange(ev: Event) {
  const input = ev.target as HTMLInputElement | null;
  const file = input?.files?.[0];
  if (!file) return;
  importedAtlasText.value = await file.text();
  renderCurrentPose();
}

function onLoadCleanroomDemoAsset() {
  paramVersion.value = "4.0";
  importedRawAsset.value = HANDCRAFTED_SPINE40_SKELETON_JSON;
  importedAtlasText.value = HANDCRAFTED_SPINE40_ATLAS_TEXT;
  emit("import-status", `Spine 4.0 导入校验：${skinResolution.value.status} (${skinResolution.value.reason})`);
  renderCurrentPose();
}

function onClearImportedAsset() {
  importedRawAsset.value = null;
  importedAtlasText.value = "";
  emit("import-status", `已清空导入动皮：回退静态立绘 (${skinResolution.value.status})`);
  renderCurrentPose();
}

onMounted(() => {
  renderCurrentPose();
});

watch(
  () => [props.activeClip, props.standX, props.standY, props.scalePct, paramVersion.value, paramScale.value],
  () => {
    renderCurrentPose();
  },
);
</script>

<template>
  <div class="sub-surface-box spine40-import-box" data-testid="l24-skin-transform-panel">
    <canvas
      ref="canvas2dRef"
      width="160"
      height="52"
      class="spine-canvas"
      data-testid="spine-skeletal-canvas"
    ></canvas>
    <canvas
      ref="webglCanvasRef"
      width="120"
      height="52"
      class="spine-canvas"
      data-testid="spine-webgl-canvas"
    ></canvas>
    <span class="tiny" data-testid="l24-skin-transform-meta">
      Spine 4.0 净室 · 动作:{{ activeClip }} · 偏移({{ standX }},{{ standY }}) · 缩放 {{ scalePct }}% · 资产状态:{{ skinResolution.status }} · WebGL批次:{{ lastGlStats.drawCallCount }}/{{ lastGlStats.triangleCount }}△
    </span>
    <button type="button" data-testid="btn-l24-trigger-clip" @click="emit('trigger-clip')">
      触发 Spine 骨骼动作 (L08/L24)
    </button>
    <button
      type="button"
      data-testid="btn-spine40-toggle-import"
      @click="showImportForm = !showImportForm"
    >
      {{ showImportForm ? "收起动皮导入" : "玩家动皮导入 (saveSkinParams)" }}
    </button>

    <div v-if="showImportForm" class="spine-import-drawer" data-testid="spine40-import-drawer">
      <div class="import-row">
        <label class="tiny">
          武将ID
          <input v-model="paramGeneralId" type="text" class="mini-input" data-testid="input-spine-general" />
        </label>
        <label class="tiny">
          皮肤名
          <input v-model="paramSkinName" type="text" class="mini-input" data-testid="input-spine-skin-name" />
        </label>
        <label class="tiny">
          版本
          <input v-model="paramVersion" type="text" class="mini-input" data-testid="input-spine-version" />
        </label>
        <label class="tiny">
          动作
          <input v-model="paramAction" type="text" class="mini-input" data-testid="input-spine-action" />
        </label>
      </div>
      <div class="import-row">
        <label class="tiny">
          骨骼 (.json/.skel)
          <input
            type="file"
            accept=".json,.skel"
            data-testid="input-spine-skel-file"
            @change="onSkeletonFileChange"
          />
        </label>
        <label class="tiny">
          图集 (.atlas)
          <input type="file" accept=".atlas,.txt" data-testid="input-spine-atlas-file" @change="onAtlasFileChange" />
        </label>
        <button type="button" data-testid="btn-spine40-load-handcrafted" @click="onLoadCleanroomDemoAsset">
          载入净室4.0校核骨骼
        </button>
        <button type="button" data-testid="btn-spine40-clear-asset" @click="onClearImportedAsset">
          重置回退静态立绘
        </button>
      </div>
      <p class="tiny fallback-reason" data-testid="spine40-resolution-reason">
        解析结果：{{ skinResolution.status }} — {{ skinResolution.reason }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.spine-canvas {
  border-radius: 4px;
  border: 1px solid var(--border);
  background: #121a26;
}
.spine-import-drawer {
  width: 100%;
  margin-top: 0.35rem;
  padding-top: 0.35rem;
  border-top: 1px dashed var(--border);
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.import-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.4rem;
}
.mini-input {
  width: 5.5rem;
  margin-left: 0.2rem;
}
.fallback-reason {
  margin: 0;
  font-family: var(--mono);
}
</style>
