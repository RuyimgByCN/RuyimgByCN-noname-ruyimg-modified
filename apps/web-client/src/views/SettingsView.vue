<script setup lang="ts">
import { ref, watch, onMounted } from "vue";

// §30 Accessibility: full profile controls

const reduceMotion = ref(
  typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
);
const highContrast = ref(false);
const skipAnimations = ref(false);
const fontScale = ref<"small" | "normal" | "large" | "xlarge">("normal");
const colorBlind = ref<"none" | "protanopia" | "deuteranopia" | "tritanopia" | "achromatopsia">("none");
const screenReader = ref<"off" | "default" | "verbose">("default");

// Persist to localStorage
const STORAGE_KEY = "noname-next-a11y";

interface A11yState {
  highContrast: boolean;
  skipAnimations: boolean;
  fontScale: string;
  colorBlind: string;
  screenReader: string;
}

function loadState(): A11yState | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

function saveState() {
  if (typeof window === "undefined") return;
  const state: A11yState = {
    highContrast: highContrast.value,
    skipAnimations: skipAnimations.value,
    fontScale: fontScale.value,
    colorBlind: colorBlind.value,
    screenReader: screenReader.value,
  };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

function applyToDom() {
  const el = document.documentElement;
  el.dataset.contrast = highContrast.value ? "high" : "";
  el.dataset.skipAnimations = skipAnimations.value ? "true" : "false";
  el.dataset.fontScale = fontScale.value;
  el.dataset.colorBlind = colorBlind.value !== "none" ? colorBlind.value : "";
}

onMounted(() => {
  const saved = loadState();
  if (saved) {
    highContrast.value = saved.highContrast;
    skipAnimations.value = saved.skipAnimations;
    fontScale.value = (saved.fontScale as any) || "normal";
    colorBlind.value = (saved.colorBlind as any) || "none";
    screenReader.value = (saved.screenReader as any) || "default";
  }
  applyToDom();
});

watch([highContrast, skipAnimations, fontScale, colorBlind, screenReader], () => {
  applyToDom();
  saveState();
});
</script>

<template>
  <section class="panel stack" aria-labelledby="settings-title">
    <h1 id="settings-title">设置</h1>
    <p class="muted" role="note">
      表现层设置不影响规则核心（§30 可访问性）。所有设置即时生效并保存到本地。
    </p>

    <fieldset class="a11y-group">
      <legend>可访问性（§30）</legend>

      <!-- §30: 减少动态效果 -->
      <label class="check">
        <input
          v-model="reduceMotion"
          type="checkbox"
          disabled
          aria-describedby="reduce-motion-desc"
        />
        <span>减少动态效果（系统偏好，只读）</span>
      </label>
      <p id="reduce-motion-desc" class="sr-only">
        遵循系统 prefers-reduced-motion 设置。
      </p>

      <!-- §30: 动画跳过 -->
      <label class="check">
        <input
          v-model="skipAnimations"
          type="checkbox"
          aria-describedby="skip-anim-desc"
        />
        <span>跳过动画（跳过后仍可理解规则结果）</span>
      </label>
      <p id="skip-anim-desc" class="sr-only">
        跳过所有动画；规则结果仍通过文字日志和图标完整传达。
      </p>

      <!-- §30: 高对比度 -->
      <label class="check">
        <input v-model="highContrast" type="checkbox" />
        <span>高对比度模式</span>
      </label>

      <!-- §30: 字号缩放 -->
      <label class="select-row">
        <span>字号缩放</span>
        <select v-model="fontScale" aria-label="字号缩放">
          <option value="small">小 (85%)</option>
          <option value="normal">标准 (100%)</option>
          <option value="large">大 (125%)</option>
          <option value="xlarge">特大 (150%)</option>
        </select>
      </label>

      <!-- §30: 色觉友好 -->
      <label class="select-row">
        <span>色觉友好模式</span>
        <select v-model="colorBlind" aria-label="色觉友好模式">
          <option value="none">关闭</option>
          <option value="protanopia">红色盲（Protanopia）</option>
          <option value="deuteranopia">绿色盲（Deuteranopia）</option>
          <option value="tritanopia">蓝色盲（Tritanopia）</option>
          <option value="achromatopsia">全色盲（Achromatopsia）</option>
        </select>
      </label>

      <!-- §30: 屏幕阅读器 -->
      <label class="select-row">
        <span>屏幕阅读器详细度</span>
        <select v-model="screenReader" aria-label="屏幕阅读器详细度">
          <option value="off">关闭</option>
          <option value="default">标准</option>
          <option value="verbose">详细（播报所有状态变化）</option>
        </select>
      </label>
    </fieldset>

    <div class="state-legend" role="region" aria-label="状态信号图例">
      <h2>状态信号说明</h2>
      <p class="muted">
        所有游戏状态通过图标 + 文字传达，不只依赖颜色（§30: 不只依赖颜色表达状态）。
      </p>
      <ul class="legend-list">
        <li><span class="state-icon" style="color: var(--color-positive)">✓</span> 存活</li>
        <li><span class="state-icon" style="color: var(--color-warning)">⚠</span> 濒死</li>
        <li><span class="state-icon" style="color: var(--color-negative)">✗</span> 阵亡</li>
        <li><span class="state-icon" style="color: var(--color-positive)">▸</span> 可出牌</li>
        <li><span class="state-icon" style="color: var(--color-negative)">✕</span> 不可用</li>
        <li><span class="state-icon" style="color: var(--color-accent)">►</span> 当前回合</li>
        <li><span class="state-icon" style="color: var(--color-warning)">?</span> 请选择</li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
.a11y-group {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}
.a11y-group legend {
  font-weight: 600;
  padding: 0 0.5rem;
}
.check {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  padding: 0.2rem 0;
}
.select-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  padding: 0.2rem 0;
}
.select-row span {
  min-width: 8rem;
}
.select-row select {
  background: var(--bg-elevated);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.3rem 0.5rem;
}
.state-legend {
  margin-top: 0.5rem;
}
.state-legend h2 {
  font-size: 1.05rem;
}
.legend-list {
  list-style: none;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
  gap: 0.3rem;
}
.legend-list li {
  display: flex;
  gap: 0.4rem;
  align-items: center;
}
code {
  font-family: var(--mono);
  font-size: 0.9em;
}
</style>
