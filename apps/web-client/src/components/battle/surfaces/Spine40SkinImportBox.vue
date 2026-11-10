<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import {
  CleanRoomAnimationState,
  HANDCRAFTED_SPINE40_ATLAS_TEXT,
  HANDCRAFTED_SPINE40_SKELETON_JSON,
  buildBatchGeometry,
  createSkeletonInstance,
  parseTextureAtlas,
  renderSpineBatchesToCanvas2D,
  renderSpineBatchesToWebGL,
  resolveDynamicSkinImport,
  uploadRgbaTextureToWebGL,
  type MinimalWebGLContext,
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

const cleanroomAtlas = parseTextureAtlas(HANDCRAFTED_SPINE40_ATLAS_TEXT);

const showImportForm = ref(false);
const paramGeneralId = ref("caocao");
const paramSkinName = ref("玩家自备动皮");
const paramVersion = ref("4.0");
const paramAction = ref("DaiJi");
const paramScale = ref(1.0);
const importedRawAsset = ref<unknown>(null);
const importedAtlasText = ref<string>("");
const importedTexturePixels = ref<{ name: string; width: number; height: number; pixels: Uint8Array } | null>(null);
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

const activeAtlas = computed<TextureAtlas>(() =>
  importedAtlasText.value.trim() ? parseTextureAtlas(importedAtlasText.value) : cleanroomAtlas,
);

function renderStaticPortraitFallback() {
  const c2d = canvas2dRef.value?.getContext("2d");
  if (c2d && canvas2dRef.value) {
    const w = canvas2dRef.value.width;
    const h = canvas2dRef.value.height;
    c2d.clearRect(0, 0, w, h);
    c2d.fillStyle = "#172033";
    c2d.fillRect(0, 0, w, h);
    c2d.strokeStyle = "#475569";
    c2d.strokeRect(2, 2, w - 4, h - 4);
    c2d.fillStyle = "#cbd5e1";
    c2d.font = "10px sans-serif";
    c2d.fillText(`静态立绘: ${paramGeneralId.value}`, 8, 22);
    c2d.fillStyle = "#94a3b8";
    c2d.fillText(paramSkinName.value.slice(0, 12), 8, 38);
  }
  const glCanvas = webglCanvasRef.value;
  if (glCanvas) {
    const gl = (glCanvas.getContext("webgl", { premultipliedAlpha: true }) ??
      glCanvas.getContext("experimental-webgl")) as unknown as MinimalWebGLContext | null;
    if (gl) {
      gl.viewport(0, 0, glCanvas.width, glCanvas.height);
      gl.clearColor(0.07, 0.1, 0.15, 1.0);
      gl.clear(gl.COLOR_BUFFER_BIT);
    }
  }
  lastGlStats.value = { drawCallCount: 0, triangleCount: 0, shaderProgramActive: false };
}

function renderCurrentPose() {
  const skelData = skinResolution.value.skeletonData;
  if (!skelData || skinResolution.value.status !== "ready-spine40") {
    renderStaticPortraitFallback();
    return;
  }
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
      let pageTextures: Record<string, unknown> | undefined;
      if (importedTexturePixels.value) {
        const tex = uploadRgbaTextureToWebGL(
          gl,
          importedTexturePixels.value.width,
          importedTexturePixels.value.height,
          importedTexturePixels.value.pixels,
        );
        if (tex) {
          pageTextures = { [importedTexturePixels.value.name]: tex };
          const firstPage = atlas.pages[0]?.name;
          if (firstPage) pageTextures[firstPage] = tex;
        }
      }
      lastGlStats.value = renderSpineBatchesToWebGL(gl, batches, glCanvas.width, glCanvas.height, pageTextures);
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

async function onTextureFileChange(ev: Event) {
  const input = ev.target as HTMLInputElement | null;
  const file = input?.files?.[0];
  if (!file || typeof createImageBitmap !== "function") return;
  const bmp = await createImageBitmap(file);
  const offscreen = document.createElement("canvas");
  offscreen.width = bmp.width;
  offscreen.height = bmp.height;
  const ctx = offscreen.getContext("2d");
  if (!ctx) return;
  ctx.drawImage(bmp, 0, 0);
  const imgData = ctx.getImageData(0, 0, bmp.width, bmp.height);
  importedTexturePixels.value = {
    name: file.name,
    width: bmp.width,
    height: bmp.height,
    pixels: new Uint8Array(imgData.data.buffer, imgData.data.byteOffset, imgData.data.byteLength),
  };
  emit("import-status", `动皮贴图导入 (${file.name}, ${bmp.width}x${bmp.height})`);
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
  importedTexturePixels.value = null;
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
        <label class="tiny">
          贴图 (.png)
          <input
            type="file"
            accept=".png,image/png,.jpg,.jpeg,.webp"
            data-testid="input-spine-png-file"
            @change="onTextureFileChange"
          />
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
