<script setup lang="ts">
import { computed, onMounted, ref } from "vue";

interface Character {
  id: string;
  migration_status: "ported" | "source_synced";
  pack: string;
  name: string;
  display_name: string;
  sex: string | null;
  group: string | null;
  hp: number | string | null;
  skills: string[];
  resolved_skills: string[];
  portrait: string | null;
}

interface Skill {
  id: string;
  name: string;
  display_name: string;
  description: string | null;
  migration_status: "ported" | "source_synced";
}

interface Skin {
  id: string;
  character: string;
  name: string;
  asset?: string;
}

interface Catalog {
  counts: Record<string, number>;
  packages: string[];
  characters: Character[];
  skills: Skill[];
  skins: { static: Skin[]; dynamic: Skin[] };
}

const catalog = ref<Catalog | null>(null);
const error = ref("");
const loading = ref(false);
const query = ref("");
const pack = ref("");
const selected = ref<Character | null>(null);
const contentBase = defaultContentBase();

const skillById = computed(() => new Map(catalog.value?.skills.map((skill) => [skill.id, skill])));
const visibleCharacters = computed(() => {
  const needle = query.value.trim().toLowerCase();
  return (catalog.value?.characters ?? [])
    .filter((character) => !pack.value || character.pack === pack.value)
    .filter((character) =>
      !needle ||
      character.name.toLowerCase().includes(needle) ||
      character.display_name.toLowerCase().includes(needle),
    )
    .slice(0, 80);
});
const selectedSkills = computed(() =>
  selected.value?.resolved_skills.map((id) => skillById.value.get(id)).filter(Boolean) as Skill[] ?? [],
);
const selectedStaticSkins = computed(() =>
  (catalog.value?.skins.static ?? []).filter((skin) => skin.character === selected.value?.name),
);
const selectedDynamicSkins = computed(() =>
  (catalog.value?.skins.dynamic ?? []).filter((skin) => skin.character === selected.value?.name),
);

onMounted(loadCatalog);

async function loadCatalog() {
  loading.value = true;
  error.value = "";
  try {
    const response = await fetch(`${contentBase}/catalog.json`);
    if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
    catalog.value = await response.json() as Catalog;
    selected.value = catalog.value.characters[0] ?? null;
  } catch (cause) {
    error.value = `无法读取原仓库内容目录（${String(cause)}）。请先启动 game-server。`;
  } finally {
    loading.value = false;
  }
}

function assetUrl(asset: string | null | undefined): string {
  return asset ? encodeURI(`${contentBase}/${asset}`) : "";
}

function defaultContentBase(): string {
  const local = !window.location.hostname || window.location.hostname.endsWith("tauri.localhost");
  const host = local ? "127.0.0.1" : window.location.hostname;
  const protocol = window.location.protocol === "https:" ? "https:" : "http:";
  return `${protocol}//${host}:7700/original-content`;
}
</script>

<template>
  <div class="stack">
    <section class="panel">
      <h1>原仓库武将库</h1>
      <p class="muted">
        武将、技能源码、静态皮肤和动态皮肤均从本地原仓库目录读取；旧 JavaScript 不会在客户端执行。
      </p>
      <p v-if="catalog" class="counts">
        {{ catalog.counts.packs }} 包 · {{ catalog.counts.characters }} 武将 ·
        {{ catalog.counts.skills }} 技能 · {{ catalog.counts.static_skins }} 静态皮肤 ·
        {{ catalog.counts.dynamic_skins }} 动态皮肤
      </p>
      <p v-if="loading" class="muted" role="status">正在加载目录…</p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>

    <section v-if="catalog" class="browser">
      <aside class="panel roster">
        <div class="filters">
          <input v-model="query" type="search" placeholder="搜索武将" aria-label="搜索武将" />
          <select v-model="pack" aria-label="武将包">
            <option value="">全部武将包</option>
            <option v-for="name in catalog.packages" :key="name" :value="name">{{ name }}</option>
          </select>
        </div>
        <button
          v-for="character in visibleCharacters"
          :key="character.id"
          type="button"
          class="character-row"
          :class="{ selected: selected?.id === character.id }"
          @click="selected = character"
        >
          <img v-if="character.portrait" :src="assetUrl(character.portrait)" alt="" loading="lazy" />
          <span><strong>{{ character.display_name }}</strong><small>{{ character.pack }} · {{ character.name }}</small></span>
        </button>
        <p v-if="visibleCharacters.length === 80" class="muted limit">仅显示前 80 项，请继续筛选。</p>
      </aside>

      <article v-if="selected" class="panel detail">
        <header>
          <img v-if="selected.portrait" :src="assetUrl(selected.portrait)" :alt="`${selected.display_name}武将头像`" />
          <div>
            <h2>{{ selected.display_name }}</h2>
            <span
              class="character-status"
              :class="selected.migration_status"
              :title="selected.migration_status === 'ported' ? '完整武将定义已接入确定性内容包' : '仅同步目录、源码与资源，尚不能进入确定性对局'"
            >{{ selected.migration_status === 'ported' ? '武将包已接入' : '待完整移植' }}</span>
            <p class="muted">{{ selected.pack }} · {{ selected.name }} · {{ selected.group ?? '未知势力' }} · {{ selected.hp }} 体力</p>
          </div>
        </header>

        <h3>技能</h3>
        <ul v-if="selectedSkills.length" class="skills">
          <li v-for="skill in selectedSkills" :key="skill.id">
            <strong>{{ skill.display_name }}</strong>
            <span
              :class="skill.migration_status"
              :title="skill.migration_status === 'ported' ? '已接入确定性内容包；是否与原技能完全等价仍以 Golden 为准' : '仅保留原源码与目录，不能进入确定性对局'"
            >{{ skill.migration_status === 'ported' ? 'VM 已接入' : '待 DSL 重写' }}</span>
            <p>{{ skill.description ?? skill.name }}</p>
          </li>
        </ul>
        <p v-else class="muted">没有可解析的武将包技能。</p>

        <h3>静态皮肤（{{ selectedStaticSkins.length }}）</h3>
        <div v-if="selectedStaticSkins.length" class="skins">
          <figure v-for="skin in selectedStaticSkins" :key="skin.id">
            <img :src="assetUrl(skin.asset)" :alt="`${selected.display_name} · ${skin.name}`" loading="lazy" />
            <figcaption>{{ skin.name }}</figcaption>
          </figure>
        </div>
        <p v-else class="muted">原仓库未提供静态皮肤。</p>

        <h3>动态皮肤（{{ selectedDynamicSkins.length }}）</h3>
        <p class="muted">
          {{ selectedDynamicSkins.map((skin) => skin.name).join('、') || '原仓库未提供动态皮肤。' }}
        </p>
      </article>
    </section>
  </div>
</template>

<style scoped>
.counts { color: var(--accent); margin-bottom: 0; }
.error { color: var(--danger); }
.browser { display: grid; grid-template-columns: minmax(260px, 340px) 1fr; gap: 0.85rem; align-items: start; }
.roster { max-height: calc(100vh - 8rem); overflow: auto; }
.filters { display: grid; gap: 0.5rem; position: sticky; top: 0; padding-bottom: 0.6rem; background: var(--bg-panel); z-index: 1; }
.filters input, .filters select { width: 100%; background: var(--bg); color: var(--text); border: 1px solid var(--border); border-radius: 8px; padding: 0.5rem; }
.character-row { display: flex; align-items: center; gap: 0.55rem; width: 100%; margin-top: 0.35rem; padding: 0.35rem; text-align: left; }
.character-row.selected { border-color: var(--accent); }
.character-row img { width: 42px; height: 54px; object-fit: cover; object-position: top; border-radius: 5px; }
.character-row span { display: grid; min-width: 0; }
.character-row small { color: var(--muted); overflow: hidden; text-overflow: ellipsis; }
.limit { font-size: 0.8rem; }
.detail > header { display: flex; gap: 1rem; align-items: flex-start; }
.detail > header img { width: 150px; height: 200px; object-fit: cover; object-position: top; border-radius: 9px; }
.detail h3 { margin-top: 1.2rem; }
.character-status { display: inline-block; padding: 0.1rem 0.4rem; border-radius: 999px; font-size: 0.72rem; }
.character-status.ported { color: var(--ok); border: 1px solid var(--ok); }
.character-status.source_synced { color: var(--muted); border: 1px solid var(--border); }
.skills { padding: 0; list-style: none; }
.skills li { border-top: 1px solid var(--border); padding: 0.65rem 0; }
.skills li > span { margin-left: 0.5rem; padding: 0.1rem 0.4rem; border-radius: 999px; font-size: 0.72rem; }
.skills .ported { color: var(--ok); border: 1px solid var(--ok); }
.skills .source_synced { color: var(--muted); border: 1px solid var(--border); }
.skills p { margin: 0.35rem 0 0; color: var(--muted); }
.skins { display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: 0.65rem; }
.skins figure { margin: 0; }
.skins img { width: 100%; aspect-ratio: 3 / 4; object-fit: cover; object-position: top; border-radius: 7px; background: var(--bg); }
.skins figcaption { margin-top: 0.25rem; font-size: 0.8rem; text-align: center; }
@media (max-width: 760px) {
  .browser { grid-template-columns: 1fr; }
  .roster { max-height: 45vh; }
  .detail > header img { width: 110px; height: 150px; }
}
</style>
