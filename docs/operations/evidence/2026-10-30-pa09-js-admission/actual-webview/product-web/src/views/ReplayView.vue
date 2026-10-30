<script setup lang="ts">
import { onUnmounted, ref, watch } from "vue";
import { EngineClient } from "../engine/client";
import type { ReplayFrame, ReplayMetadata, ReplayPrivacy } from "../types/engine";

const client = new EngineClient();
const replayData = ref<{ header?: { engine_build_id?: string } } | null>(null);
const metadata = ref<ReplayMetadata | null>(null);
const frame = ref<ReplayFrame | null>(null);
const errorMsg = ref<string | null>(null);
const busy = ref(false);
const privacyLevel = ref<ReplayPrivacy>("full");

async function loadText(text: string) {
  busy.value = true;
  errorMsg.value = null;
  try {
    const parsed = JSON.parse(text);
    await client.connect();
    metadata.value = await client.loadReplay(text);
    replayData.value = parsed;
    frame.value = await client.seekReplay(0, privacyLevel.value);
  } catch (error) {
    replayData.value = null;
    metadata.value = null;
    frame.value = null;
    errorMsg.value = error instanceof Error ? error.message : String(error);
  } finally {
    busy.value = false;
  }
}

function loadFile(file: File) {
  const reader = new FileReader();
  reader.onload = () => void loadText(reader.result as string);
  reader.onerror = () => { errorMsg.value = "文件读取失败"; };
  reader.readAsText(file);
}

function onFileChange(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (file) loadFile(file);
}

function onDrop(event: DragEvent) {
  event.preventDefault();
  const file = event.dataTransfer?.files?.[0];
  if (file) loadFile(file);
}

async function seek(step: number) {
  if (!metadata.value || busy.value) return;
  busy.value = true;
  errorMsg.value = null;
  try {
    frame.value = await client.seekReplay(
      Math.max(0, Math.min(step, metadata.value.total)),
      privacyLevel.value
    );
  } catch (error) {
    errorMsg.value = error instanceof Error ? error.message : String(error);
  } finally {
    busy.value = false;
  }
}

watch(privacyLevel, () => {
  if (frame.value) void seek(frame.value.step);
});

onUnmounted(() => client.dispose());
</script>

<template>
  <div class="replay stack">
    <section class="panel">
      <h1>录像回放</h1>
      <p class="muted">导入 Command Log 后，规则引擎会真实重演并逐步核对状态哈希。</p>
    </section>

    <section class="panel drop-zone" @drop="onDrop" @dragover.prevent aria-label="导入录像文件">
      <h2>导入录像</h2>
      <p class="muted">拖拽 .json 录像文件到此处，或点击选择</p>
      <input type="file" accept=".json,application/json" @change="onFileChange" aria-label="选择录像文件" />
      <p v-if="busy" class="muted">正在验证并重演…</p>
      <p v-if="errorMsg" class="error" role="alert">{{ errorMsg }}</p>
    </section>

    <section v-if="replayData && metadata" class="panel">
      <h2>录像信息</h2>
      <dl class="info-grid">
        <dt>Manifest Hash</dt><dd class="mono">{{ metadata.manifest_hash }}</dd>
        <dt>引擎构建</dt><dd class="mono">{{ replayData.header?.engine_build_id ?? "—" }}</dd>
        <dt>总步数</dt><dd>{{ metadata.total }}</dd>
        <dt>完整性</dt><dd class="verified">✓ 哈希链与全部状态检查点已验证</dd>
        <dt>隐私等级</dt>
        <dd><select v-model="privacyLevel" :disabled="busy" aria-label="隐私等级">
          <option value="full">完整（裁判视角）</option>
          <option value="player">玩家 1 视角</option>
          <option value="public">公开视角</option>
        </select></dd>
      </dl>
    </section>

    <section v-if="frame && metadata" class="panel">
      <h2>回放控制</h2>
      <div class="row playback-controls">
        <button type="button" @click="seek(0)" :disabled="busy || frame.step === 0">⏮ 开头</button>
        <button type="button" @click="seek(frame.step - 1)" :disabled="busy || frame.step === 0">◀ 后退</button>
        <span class="step-display mono">{{ frame.step }} / {{ frame.total }}</span>
        <button type="button" @click="seek(frame.step + 1)" :disabled="busy || frame.step >= frame.total">前进 ▶</button>
        <button type="button" @click="seek(frame.total)" :disabled="busy || frame.step >= frame.total">结尾 ⏭</button>
      </div>
      <div class="progress-bar" role="progressbar" :aria-valuenow="frame.step" :aria-valuemax="frame.total">
        <div class="progress-fill" :style="{ width: (frame.step / Math.max(frame.total, 1) * 100) + '%' }"></div>
      </div>
      <dl class="info-grid frame-info">
        <dt>Revision</dt><dd>{{ frame.revision }}</dd>
        <dt>State Hash</dt><dd class="mono">{{ frame.hash_hex }}</dd>
        <dt>比赛状态</dt><dd>{{ frame.finished ? "已结束" : "进行中" }}</dd>
      </dl>
    </section>

    <section v-if="frame" class="panel">
      <h2>当前局面</h2>
      <div v-if="frame.view.seats.length" class="seat-grid">
        <article v-for="seat in frame.view.seats" :key="seat.player" class="seat">
          <strong>座位 {{ seat.seat }} · 玩家 {{ seat.player }}</strong>
          <span>体力 {{ seat.hp }} / {{ seat.max_hp }}</span>
          <span>手牌 {{ seat.hand_count }} · {{ seat.alive ? "存活" : "阵亡" }}{{ seat.turned_over ? " · 翻面" : "" }}</span>
          <span>身份 {{ seat.identity ?? "未知" }}</span>
        </article>
      </div>
      <p v-else class="muted">比赛尚未开始。</p>
    </section>

    <section v-if="replayData" class="panel privacy-notice" role="note">
      <h2>隐私分级（§22.5）</h2>
      <p class="muted tiny">每个画面直接从对应的裁判、玩家或公开 Observer 投影构建，不会先生成完整视图再删字段。</p>
    </section>
  </div>
</template>

<style scoped>
.drop-zone { border: 2px dashed var(--border); text-align: center; padding: 1.5rem; transition: border-color 0.2s; }
.drop-zone:hover { border-color: var(--accent-2); }
.info-grid { display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 0.4rem 1rem; margin: 0.5rem 0; }
.info-grid dt { color: var(--muted); font-size: 0.85rem; }
.info-grid dd { margin: 0; overflow-wrap: anywhere; }
.mono { font-family: var(--mono); font-size: 0.85rem; }
.verified { color: var(--success, #63c174); }
.playback-controls { align-items: center; gap: 0.6rem; }
.step-display { font-weight: 600; min-width: 5rem; text-align: center; }
.progress-bar { margin-top: 0.6rem; height: 6px; background: var(--bg-elevated); border-radius: 3px; overflow: hidden; }
.progress-fill { height: 100%; background: var(--accent); transition: width 0.2s; }
.frame-info { margin-top: 1rem; }
.seat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(190px, 1fr)); gap: 0.6rem; }
.seat { display: grid; gap: 0.25rem; padding: 0.7rem; border: 1px solid var(--border); border-radius: 8px; }
.seat span { color: var(--muted); font-size: 0.85rem; }
.tiny { font-size: 0.78rem; }
.error { color: var(--danger); margin-top: 0.5rem; }
</style>
