<script setup lang="ts">
import { computed, ref } from "vue";
import {
  EXTENSION_CATALOG,
  extensionKey,
} from "../engine/extensionCatalog";
import {
  activateExtension,
  contentLockSummary,
  deactivateExtension,
  importExtensionPackage,
  installAndActivateAll,
  installExtension,
  readExtensionStore,
  revokeExtension,
  rollbackExtension,
  uninstallExtension,
  upgradeExtension,
  type ExtensionStoreSnapshot,
} from "../engine/extensionStore";

const snap = ref<ExtensionStoreSnapshot>(readExtensionStore());
const error = ref("");
const notice = ref("");
const importText = ref("");

function refresh() {
  snap.value = readExtensionStore();
  error.value = "";
}

function run(fn: () => void) {
  error.value = "";
  notice.value = "";
  try {
    fn();
    refresh();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

/** 正式导入入口：粘贴或选择作者产物（完整 npack JSON，可含 VM 模块）。 */
function importFromText() {
  run(() => {
    const text = importText.value;
    importExtensionPackage(text);
    const manifest = (JSON.parse(text) as { manifest: { namespace: string; package_id: string; version: string } }).manifest;
    notice.value = `已导入 ${manifest.namespace}/${manifest.package_id} @ ${manifest.version}`;
    importText.value = "";
  });
}

async function importFromFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  try {
    importText.value = await file.text();
    importFromText();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    input.value = "";
  }
}

function record(key: string) {
  return snap.value.installed.find((r) => r.key === key);
}

function statusLabel(key: string): string {
  const r = record(key);
  if (!r) return "未安装";
  if (r.lifecycle === "active") return "已启用";
  if (r.lifecycle === "disabled") return "已停用";
  if (r.lifecycle === "revoked") return "已撤销";
  return "已安装";
}

const lock = computed(() => contentLockSummary(snap.value));

const rows = computed(() => {
  const catalogRows = EXTENSION_CATALOG.map((e) => {
    const key = extensionKey(e);
    const r = record(key);
    return { title: e.title, version: e.version, key, record: r, status: statusLabel(key), imported: false };
  });
  const catalogKeys = new Set(catalogRows.map((row) => row.key));
  const importedRows = snap.value.installed
    .filter((r) => !catalogKeys.has(r.key))
    .map((r) => ({ title: `${r.packageId}（导入）`, version: r.version, key: r.key, record: r, status: statusLabel(r.key), imported: true }));
  return [...catalogRows, ...importedRows];
});
</script>

<template>
  <div class="stack">
    <section class="panel">
      <h1>扩展包管理</h1>
      <p class="muted">
        对齐宿主契约：安装 / 启用 / 停用 / 升级 / 回退 / 卸载 / 撤销。当前目录
        {{ EXTENSION_CATALOG.length }} 个 G1 转换包。开局内容锁由启用集构成；对局中冻结变更。
        安装时保存实际包内容，本地原生对局直接装载；存档保留原局包副本。
      </p>
      <div class="row">
        <button class="primary" type="button" @click="run(() => { installAndActivateAll(); notice = `已安装并启用全部 ${EXTENSION_CATALOG.length} 包`; })">
          全部安装并启用
        </button>
        <button type="button" @click="refresh">刷新</button>
      </div>
      <p v-if="notice" class="ok" role="status">{{ notice }}</p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <p v-if="snap.matchFrozen" class="warn" role="status">对局进行中 · 内容锁已冻结</p>
    </section>

    <section class="panel">
      <h2>导入扩展包</h2>
      <p class="muted">
        正式导入入口：粘贴或选择作者产物（完整 npack JSON，可含 VM 模块与卡牌/武将/牌堆资产）。
        同包新版本按升级处理并保留旧版本副本；同版本内容冲突、已撤销、对局冻结或结构校验失败时拒绝且不改写已安装内容。
      </p>
      <textarea
        v-model="importText"
        class="import-box"
        rows="4"
        spellcheck="false"
        aria-label="扩展包 JSON"
        placeholder='{"manifest": {...}, "modules": [...], ...}'
      ></textarea>
      <div class="row">
        <button class="primary" type="button" :disabled="!importText.trim() || snap.matchFrozen" @click="importFromText">导入</button>
        <label>
          选择文件
          <input type="file" accept=".json,application/json" aria-label="选择扩展包文件" :disabled="snap.matchFrozen" @change="importFromFile" />
        </label>
      </div>
    </section>

    <section class="panel">
      <h2>内容锁</h2>
      <p class="muted mono">{{ lock.hashSeed || "（空）" }}</p>
      <ul v-if="lock.packages.length" class="lock-list">
        <li v-for="p in lock.packages" :key="p.key">
          {{ p.key }} @ {{ p.version }} · {{ p.fingerprint }}
        </li>
      </ul>
      <p v-else class="muted">尚未启用任何扩展。</p>
    </section>

    <section class="panel">
      <h2>包列表</h2>
      <div class="table" role="table">
        <div class="thead" role="row">
          <span>名称</span>
          <span>标识</span>
          <span>版本</span>
          <span>状态</span>
          <span>操作</span>
        </div>
        <div v-for="row in rows" :key="row.key" class="trow" role="row">
          <span>{{ row.title }}</span>
          <span class="mono">{{ row.key }}</span>
          <span class="mono">{{ row.record?.version ?? row.version }}</span>
          <span>{{ row.status }}</span>
          <span class="actions">
            <button type="button" :disabled="row.imported || (!!row.record && row.record.lifecycle !== 'revoked')" @click="run(() => installExtension(row.key))">
              安装
            </button>
            <button type="button" :disabled="!row.record || row.record.lifecycle === 'active' || row.record.lifecycle === 'revoked'" @click="run(() => activateExtension(row.key))">
              启用
            </button>
            <button type="button" :disabled="!row.record || row.record.lifecycle !== 'active'" @click="run(() => deactivateExtension(row.key))">
              停用
            </button>
            <button type="button" :disabled="row.imported || !row.record || row.record.lifecycle === 'revoked'" :title="row.imported ? '导入包请导入新版本包以升级' : ''" @click="run(() => upgradeExtension(row.key))">
              升级
            </button>
            <button
              type="button"
              :disabled="!row.record || (row.record.history?.length ?? 0) < 2"
              @click="run(() => {
                const hist = row.record!.history.filter((v) => v !== row.record!.version);
                if (hist.length) rollbackExtension(row.key, hist[hist.length - 1]!);
              })"
            >
              回退
            </button>
            <button type="button" :disabled="!row.record" @click="run(() => uninstallExtension(row.key))">
              卸载
            </button>
            <button type="button" :disabled="!row.record || row.record.lifecycle === 'revoked'" @click="run(() => revokeExtension(row.key))">
              撤销
            </button>
          </span>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.ok { color: var(--color-positive, #3d8); }
.warn { color: var(--color-warning, #c90); }
.error { color: var(--danger, #e55); }
.mono { font-family: var(--mono, ui-monospace, monospace); font-size: 0.85em; word-break: break-all; }
.import-box { width: 100%; box-sizing: border-box; font-family: var(--mono, ui-monospace, monospace); font-size: 0.8em; margin-bottom: 0.5rem; }
.lock-list { margin: 0.4rem 0 0; padding-left: 1.2rem; line-height: 1.6; }
.table { display: flex; flex-direction: column; gap: 0.35rem; }
.thead, .trow {
  display: grid;
  grid-template-columns: 7rem 1fr 4.5rem 4.5rem 1fr;
  gap: 0.5rem;
  align-items: center;
  font-size: 0.88rem;
}
.thead { color: var(--muted); font-weight: 600; border-bottom: 1px solid var(--border); padding-bottom: 0.35rem; }
.trow { padding: 0.35rem 0; border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent); }
.actions { display: flex; flex-wrap: wrap; gap: 0.25rem; }
.actions button { font-size: 0.75rem; padding: 0.2rem 0.4rem; }
@media (max-width: 900px) {
  .thead { display: none; }
  .trow {
    grid-template-columns: 1fr;
    gap: 0.2rem;
    padding: 0.6rem 0;
  }
}
</style>
