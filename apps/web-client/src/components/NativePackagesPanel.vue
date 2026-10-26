<script setup lang="ts">
import { onMounted, ref } from "vue";

type NativePackageInfo = { namespace: string; package_id: string; version: string };

const STORAGE_KEY = "noname-next.native-server";
const serverUrl = ref(
  (typeof localStorage !== "undefined" && localStorage.getItem(STORAGE_KEY)) ||
    "http://127.0.0.1:7700",
);
const packages = ref<NativePackageInfo[]>([]);
const busy = ref(false);
const message = ref("");

function persistServer(): void {
  try {
    localStorage.setItem(STORAGE_KEY, serverUrl.value);
  } catch {
    /* 隐私模式忽略 */
  }
}

async function call(path: string, init?: RequestInit): Promise<Response> {
  const response = await fetch(new URL(path, serverUrl.value), {
    ...init,
    headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
  });
  if (!response.ok) throw new Error((await response.text()) || `HTTP ${response.status}`);
  return response;
}

async function refresh(): Promise<void> {
  busy.value = true;
  try {
    const response = await call("/native/packages");
    packages.value = (await response.json()) as NativePackageInfo[];
    message.value = `已载入 ${packages.value.length} 个原生包`;
  } catch (error) {
    message.value = `读取失败：${error instanceof Error ? error.message : String(error)}`;
  } finally {
    busy.value = false;
  }
}

async function importPackages(): Promise<void> {
  busy.value = true;
  try {
    await call("/native/packages/import", { method: "POST", body: "{}" });
    await refresh();
    message.value = "安装扫描完成";
  } catch (error) {
    message.value = `安装失败：${error instanceof Error ? error.message : String(error)}`;
  } finally {
    busy.value = false;
  }
}

async function lifecycle(
  action: "activate" | "disable" | "uninstall",
  pkg: NativePackageInfo,
): Promise<void> {
  if (action === "uninstall" && !window.confirm(`卸载 ${pkg.package_id} 的所有版本？`)) return;
  busy.value = true;
  try {
    await call(`/native/packages/${action}`, {
      method: "POST",
      body: JSON.stringify({
        namespace: pkg.namespace,
        package_id: pkg.package_id,
        version: pkg.version,
      }),
    });
    await refresh();
    message.value = `${action} 完成：${pkg.package_id}`;
  } catch (error) {
    message.value = `${action} 失败：${error instanceof Error ? error.message : String(error)}`;
  } finally {
    busy.value = false;
  }
}

onMounted(refresh);
</script>

<template>
  <fieldset class="a11y-group">
    <legend>原生扩展包</legend>
    <p class="muted" role="note">
      通过服务端正式入口安装、启停、升级或卸载原生包；客户端无需重新构建。
    </p>
    <div class="row">
      <label>
        服务端
        <input v-model="serverUrl" type="url" aria-label="原生服务端地址" @change="persistServer" />
      </label>
      <button type="button" :disabled="busy" @click="refresh">刷新</button>
      <button type="button" :disabled="busy" @click="importPackages">扫描安装</button>
    </div>
    <ul class="legend-list" aria-live="polite">
      <li v-for="pkg in packages" :key="`${pkg.namespace}/${pkg.package_id}@${pkg.version}`">
        <code>{{ pkg.namespace }}/{{ pkg.package_id }}@{{ pkg.version }}</code>
        <button type="button" :disabled="busy" @click="lifecycle('activate', pkg)">激活</button>
        <button type="button" :disabled="busy" @click="lifecycle('disable', pkg)">停用</button>
        <button type="button" :disabled="busy" @click="lifecycle('uninstall', pkg)">卸载</button>
      </li>
    </ul>
    <p v-if="packages.length === 0" class="muted">尚未读取到已安装包。</p>
    <p class="status" role="status">{{ message }}</p>
  </fieldset>
</template>
