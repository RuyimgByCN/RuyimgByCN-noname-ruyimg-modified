<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  COIN_HAVE_FUN_SURFACE_IDS,
  L01_CONFIG_SPECS,
  L02_CONFIG_SPECS,
  L12_SURFACE_IDS,
  L15_CONFIG_SPECS,
  L16_CONFIG_SPECS,
  L17_CONFIG_SPECS,
  L18_CONFIG_SPECS,
  L18_EXPLAIN_MAP,
  L21_CONFIG_SPECS,
  L26_CONFIG_SPECS,
  U05_CONFIG_SPECS,
  applyL01JsonImportInState,
  applyL01WeightMapJson,
  applyL15InfoFix,
  applyL15NamePrefix,
  computeL21DoubleHp,
  createInitialBatch7UiSuiteState,
  dispatchBanCharFHook,
  dispatchCoinArenaReady,
  dispatchFixCaiLanZiTap,
  dispatchHaveFunEffect,
  exportL01ConfigJson,
  importL02ConfigJson,
  importL15ConfigJson,
  importL16ConfigJson,
  importL17ConfigJson,
  importL18ConfigJson,
  importL21ConfigJson,
  importL26ConfigJson,
  importU05ConfigJson,
  normalizeL21Config,
  placeCoinBet,
  rewriteDouzhuanSkillText,
  runL01ConfigAction,
  runL16ManifestDryRun,
  runL18ProgressCheck,
  toggleL01RoleBanGeneral,
  updateL01ConfigInState,
  type Batch7UiSuiteState,
  type CoinHaveFunEffect,
  type CoinSurfaceState,
  type L01AiOptimizeConfig,
  type L01RoleBanKey,
} from "../../../engine/extensionSurfaceRuntime";

const props = defineProps<{
  batch7?: Batch7UiSuiteState;
  coin: CoinSurfaceState;
  selfEquipsCount: number;
}>();

const emit = defineEmits<{
  close: [];
  "update:batch7": [next: Batch7UiSuiteState];
  "update:coin": [next: CoinSurfaceState];
}>();

const state = ref<Batch7UiSuiteState>(props.batch7 ?? createInitialBatch7UiSuiteState());
watch(
  () => props.batch7,
  (v) => {
    if (v) state.value = v;
  },
);

type TabId = "l01" | "pool" | "tools" | "fx";
const activeTab = ref<TabId>("l01");
const l01JsonText = ref<string>(exportL01ConfigJson(state.value.l01));
const l01ImportMsg = ref<string>("L01 JSON 校验就绪（零 eval）");
const l01WeightJson = ref<string>('{"shen_zhugeliang": 1.8, "caocao": 1.3}');
const l01ThreatJson = ref<string>('{"gjcxs_zhugeliang3": 2.1}');
const l01BanInput = ref<string>("caocao");
const activePoolSource = ref<"l17" | "l15" | "l21" | "u05">("l17");
const poolJsonText = ref<string>('{"shoushanameX":"all"}');
const poolJsonMsg = ref<string>("");
const l16ArchiveInput = ref<string>("match_bundle_b7.zip");
const coinDisplayMode = ref<"icon" | "text">("icon");

function pushState(next: Batch7UiSuiteState) {
  state.value = next;
  emit("update:batch7", next);
}

function updateL01Patch(key: keyof L01AiOptimizeConfig, value: unknown) {
  const nextL01 = updateL01ConfigInState(state.value.l01, { [key]: value } as Partial<L01AiOptimizeConfig>);
  pushState({ ...state.value, l01: nextL01 });
}

function onL01Action(key: string) {
  const nextL01 = runL01ConfigAction(state.value.l01, key as Parameters<typeof runL01ConfigAction>[1], {
    generalId: l01BanInput.value,
  });
  pushState({ ...state.value, l01: nextL01 });
  l01ImportMsg.value = nextL01.lastActionNotice;
}

function onAddRoleBan(role: L01RoleBanKey | "editWj") {
  const nextL01 = toggleL01RoleBanGeneral(state.value.l01, role, l01BanInput.value);
  pushState({ ...state.value, l01: nextL01 });
  l01ImportMsg.value = nextL01.lastActionNotice;
}

function onApplyWeight(target: "editQz" | "editCf") {
  try {
    const raw = target === "editQz" ? l01WeightJson.value : l01ThreatJson.value;
    const nextL01 = applyL01WeightMapJson(state.value.l01, target, raw);
    pushState({ ...state.value, l01: nextL01 });
    l01ImportMsg.value = nextL01.lastActionNotice;
  } catch (err) {
    l01ImportMsg.value = `拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onImportL01Json() {
  try {
    const nextL01 = applyL01JsonImportInState(state.value.l01, l01JsonText.value);
    pushState({ ...state.value, l01: nextL01 });
    l01ImportMsg.value = nextL01.lastActionNotice;
  } catch (err) {
    l01ImportMsg.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onImportPoolJson() {
  try {
    if (activePoolSource.value === "l15") {
      const cfg = importL15ConfigJson(poolJsonText.value);
      pushState({ ...state.value, l15: { ...state.value.l15, config: cfg } });
    } else if (activePoolSource.value === "l17") {
      const cfg = importL17ConfigJson(poolJsonText.value);
      pushState({ ...state.value, l17: { ...state.value.l17, config: cfg } });
    } else if (activePoolSource.value === "l21") {
      const cfg = importL21ConfigJson(poolJsonText.value);
      pushState({ ...state.value, l21: { ...state.value.l21, config: cfg } });
    } else {
      const cfg = importU05ConfigJson(poolJsonText.value);
      pushState({ ...state.value, u05: { ...state.value.u05, config: cfg } });
    }
    poolJsonMsg.value = `已导入并校验 ${activePoolSource.value.toUpperCase()} 配置`;
  } catch (err) {
    poolJsonMsg.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onImportToolsJson(src: "l02" | "l16" | "l18") {
  try {
    if (src === "l02") {
      const cfg = importL02ConfigJson(poolJsonText.value);
      pushState({ ...state.value, l02: { ...state.value.l02, config: cfg } });
    } else if (src === "l16") {
      const cfg = importL16ConfigJson(poolJsonText.value);
      pushState({ ...state.value, l16: { ...state.value.l16, config: cfg } });
    } else {
      const cfg = importL18ConfigJson(poolJsonText.value);
      pushState({ ...state.value, l18: { ...state.value.l18, config: cfg } });
    }
    poolJsonMsg.value = `已校验 ${src.toUpperCase()}`;
  } catch (err) {
    poolJsonMsg.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onImportL26Json() {
  try {
    const cfg = importL26ConfigJson(poolJsonText.value);
    pushState({ ...state.value, l26: { ...state.value.l26, config: cfg } });
    poolJsonMsg.value = "已校验 L26 配置";
  } catch (err) {
    poolJsonMsg.value = `导入被拒绝：${err instanceof Error ? err.message : String(err)}`;
  }
}

function onToggleCoinHaveFun(eff: CoinHaveFunEffect) {
  emit("update:coin", dispatchHaveFunEffect(props.coin, eff, "toggle"));
}

function onToggleSnowSize() {
  emit("update:coin", dispatchHaveFunEffect(props.coin, "snow", "toggleSnowSize"));
}

function onPlaceCoinBet() {
  try {
    emit("update:coin", placeCoinBet({ ...props.coin, phaseStarted: false }, 20));
  } catch {
    /* already bet or insufficient */
  }
}

const coinHudSummary = computed(() =>
  dispatchCoinArenaReady(props.coin, {
    displayMode: coinDisplayMode.value,
    canvasTop: true,
    snowFall: props.coin.snowFall,
  }),
);
const l15SampleName = computed(() => applyL15NamePrefix("手杀孙权", state.value.l15.config.shoushanameX));
const l15SampleInfo = computed(() => applyL15InfoFix("摸一张牌", state.value.l15.config.infofix));
const l17SampleText = computed(() =>
  state.value.l17.config.miaoshuXiuGai
    ? rewriteDouzhuanSkillText("体力不大于2且不小于1，阴阳转换")
    : "体力不大于2且不小于1，阴阳转换",
);
const l21DoubleHpPreview = computed(() => computeL21DoubleHp(4, 3, state.value.l21.config.double_hp));
const l17RosterMeta: ReadonlyArray<{ id: string; label: string; faction: "wei" | "shu" | "wu" | "qun" }> = [
  { id: "caocao", label: "曹操", faction: "wei" },
  { id: "liubei", label: "刘备", faction: "shu" },
  { id: "sunquan", label: "孙权", faction: "wu" },
  { id: "lvbu", label: "吕布", faction: "qun" },
];
</script>

<template>
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="扩展配置闭环工作台" data-testid="modal-batch7-workbench">
    <div class="modal-card batch7-card">
      <header class="modal-head">
        <div>
          <h3>Batch UI-7 · 剩余 16 来源全量配置与渲染面 ({{ L01_CONFIG_SPECS.length }}+108 项)</h3>
          <p class="tiny muted">覆盖 L01 / L02 / L05 / L07 / L11 / L12 / L13 / L15 / L16 / L17 / L18 / L21 / L23 / L26 / U04 / U05（规则联动已诚实标注）</p>
        </div>
        <button type="button" data-testid="btn-close-batch7-panel" @click="emit('close')">关闭 ✕</button>
      </header>

      <div class="tab-bar">
        <button type="button" :class="{ active: activeTab === 'l01' }" data-testid="b7-tab-l01" @click="activeTab = 'l01'">
          L01 AI优化 ({{ L01_CONFIG_SPECS.length }})
        </button>
        <button type="button" :class="{ active: activeTab === 'pool' }" data-testid="b7-tab-pool" @click="activeTab = 'pool'">
          将池与规则UI (L17/L15/L21/U05)
        </button>
        <button type="button" :class="{ active: activeTab === 'tools' }" data-testid="b7-tab-tools" @click="activeTab = 'tools'">
          工具与更新 (L02/L16/L18)
        </button>
        <button type="button" :class="{ active: activeTab === 'fx' }" data-testid="b7-tab-fx" @click="activeTab = 'fx'">
          视效与适配 (U04/L05/L12/L23/L26)
        </button>
      </div>

      <!-- TAB 1: L01 (63 configs) -->
      <section v-if="activeTab === 'l01'" class="panel-scroll" data-testid="b7-panel-l01">
        <div class="cfg-grid">
          <div v-for="spec in L01_CONFIG_SPECS" :key="spec.key" class="cfg-row" :data-testid="`l01-cfg-row-${spec.key}`">
            <span class="cfg-label">
              {{ spec.label }}
              <em v-if="spec.ruleCoupled" class="rule-tag" title="该配置同时关联原生规则/AI内核">规则联动</em>
            </span>
            <input
              v-if="spec.control === 'boolean'"
              type="checkbox"
              :checked="Boolean(state.l01.config[spec.key])"
              :data-testid="`l01-cfg-input-${spec.key}`"
              @change="(e) => updateL01Patch(spec.key, (e.target as HTMLInputElement).checked)"
            />
            <select
              v-else-if="spec.control === 'select'"
              :value="String(state.l01.config[spec.key])"
              :data-testid="`l01-cfg-input-${spec.key}`"
              @change="(e) => updateL01Patch(spec.key, (e.target as HTMLSelectElement).value)"
            >
              <option v-for="op in spec.options" :key="op.value" :value="op.value">{{ op.label }}</option>
            </select>
            <button
              v-else-if="spec.control === 'action'"
              type="button"
              :data-testid="`l01-cfg-input-${spec.key}`"
              @click="onL01Action(spec.key)"
            >
              执行
            </button>
            <span v-else class="tiny muted" :data-testid="`l01-cfg-input-${spec.key}`">说明项</span>
          </div>
        </div>

        <div class="sub-box">
          <div class="inline-row">
            <input v-model="l01BanInput" type="text" placeholder="输入武将ID" data-testid="l01-ban-input" />
            <button type="button" data-testid="btn-l01-add-banzhu" @click="onAddRoleBan('banzhu')">切换主公禁将</button>
            <button type="button" data-testid="btn-l01-add-wj" @click="onAddRoleBan('editWj')">切换伪禁</button>
            <span v-if="state.l01.inspectedBanRole" class="pill" data-testid="l01-ban-table">
              当前表 {{ state.l01.inspectedBanRole }}：{{ state.l01.config[state.l01.inspectedBanRole].join(",") || "空" }}
            </span>
            <span v-if="state.l01.config.viewAtt" class="pill" data-testid="l01-view-att-badge">
              👁 火眼金睛态度监视已开启（策略={{ state.l01.config.btAi }}）
            </span>
          </div>
          <div class="inline-row">
            <input v-model="l01WeightJson" type="text" data-testid="l01-qz-json" />
            <button type="button" data-testid="btn-l01-apply-qz" @click="onApplyWeight('editQz')">应用权重 JSON</button>
            <input v-model="l01ThreatJson" type="text" data-testid="l01-cf-json" />
            <button type="button" data-testid="btn-l01-apply-cf" @click="onApplyWeight('editCf')">应用威胁度 (>0)</button>
          </div>
          <textarea v-model="l01JsonText" rows="2" data-testid="l01-cfg-json-textarea"></textarea>
          <div class="inline-row">
            <button type="button" data-testid="btn-l01-import-json" @click="onImportL01Json">导入校验 L01 (裁决38)</button>
            <span class="tiny" data-testid="l01-cfg-msg">{{ l01ImportMsg }}</span>
          </div>
        </div>
      </section>

      <!-- TAB 2: L17 + L15 + L21 + U05 -->
      <section v-else-if="activeTab === 'pool'" class="panel-scroll" data-testid="b7-panel-pool">
        <div class="inline-row">
          <button type="button" :class="{ active: activePoolSource === 'l17' }" @click="activePoolSource = 'l17'">L17 斗转星移 (11)</button>
          <button type="button" :class="{ active: activePoolSource === 'l15' }" @click="activePoolSource = 'l15'">L15 手杀全扩 (11)</button>
          <button type="button" :class="{ active: activePoolSource === 'l21' }" @click="activePoolSource = 'l21'">L21 武皇争霸 (8)</button>
          <button type="button" :class="{ active: activePoolSource === 'u05' }" @click="activePoolSource = 'u05'">U05 杀海拾遗 (7)</button>
        </div>

        <div class="dz-bigBg2" :data-qnss-bg="state.l17.config.qnssBackground ? 'on' : 'off'" :data-jj-enabled="state.l17.config.JinJiang ? 'on' : 'off'" data-testid="l17-bancharf-panel">
          <div class="inline-row">
            <strong>L17 斗转星移 (`base.css` + `banCharF`) · 菜单={{ state.l17.config.JinJiang ? "开" : "关" }}</strong>
            <button
              v-for="p in [1, 2, 3] as const"
              :key="p"
              type="button"
              class="banPlanBtn fy-btn"
              :class="{ active: state.l17.activePlanIndex === p }"
              :data-testid="`btn-l17-plan-${p}`"
              @click="pushState({ ...state, l17: dispatchBanCharFHook(state.l17, 'selectPlan', { plan: p }) })"
            >
              {{ state.l17.planNames[p - 1] }} ({{ state.l17.plans[p].length }})
            </button>
            <button
              type="button"
              data-testid="btn-l17-toggle-caocao"
              @click="pushState({ ...state, l17: dispatchBanCharFHook(state.l17, 'toggleGeneral', { generalId: 'caocao' }) })"
            >
              禁/解 曹操
            </button>
          </div>
          <div class="inline-row" data-testid="l17-banned-grid">
            <span
              v-for="g in l17RosterMeta"
              :key="g.id"
              class="dz-ban-chip"
              :class="state.l17.config.jjGroupColor ? `dz-group-${g.faction}` : ''"
              :data-testid="`l17-roster-${g.id}`"
            >
              {{ g.label }}
              <em>{{ state.l17.plans[state.l17.activePlanIndex].includes(g.id) ? "禁" : "可" }}</em>
              <button
                type="button"
                class="tiny"
                :data-testid="`btn-l17-star-${g.id}`"
                @click="pushState({ ...state, l17: dispatchBanCharFHook(state.l17, 'setStars', { generalId: g.id, stars: ((state.l17.charStars[g.id] ?? 3) % 5) + 1 }) })"
              >
                ★{{ state.l17.charStars[g.id] ?? 3 }}
              </button>
            </span>
          </div>
          <p class="tiny" data-testid="l17-rewrite-preview">描述改写：`{{ l17SampleText }}`</p>
          <div class="cfg-grid">
            <div v-for="s in L17_CONFIG_SPECS" :key="s.key" class="cfg-row" :data-testid="`l17-cfg-row-${s.key}`">
              <span class="cfg-label">{{ s.label }} <em v-if="s.ruleCoupled" class="rule-tag">规则联动</em></span>
              <input
                v-if="s.control === 'boolean'"
                type="checkbox"
                :checked="Boolean(state.l17.config[s.key])"
                :data-testid="`l17-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l17: { ...state.l17, config: { ...state.l17.config, [s.key]: (e.target as HTMLInputElement).checked } } })"
              />
              <select
                v-else
                :value="String(state.l17.config[s.key])"
                :data-testid="`l17-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l17: { ...state.l17, config: { ...state.l17.config, [s.key]: (e.target as HTMLSelectElement).value } } })"
              >
                <option v-for="op in s.options" :key="op.value" :value="op.value">{{ op.label }}</option>
              </select>
            </div>
          </div>
        </div>

        <div class="sub-box">
          <strong>L15 手杀全扩 (11 项) · 改名预览=`{{ l15SampleName }}` · 句号补全=`{{ l15SampleInfo }}`</strong>
          <div class="cfg-grid">
            <div v-for="s in L15_CONFIG_SPECS" :key="s.key" class="cfg-row" :data-testid="`l15-cfg-row-${s.key}`">
              <span class="cfg-label">{{ s.label }} <em v-if="s.ruleCoupled" class="rule-tag">规则联动</em></span>
              <input
                v-if="s.control === 'boolean'"
                type="checkbox"
                :checked="Boolean(state.l15.config[s.key])"
                :data-testid="`l15-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l15: { ...state.l15, config: { ...state.l15.config, [s.key]: (e.target as HTMLInputElement).checked } } })"
              />
              <select
                v-else
                :value="String(state.l15.config[s.key])"
                :data-testid="`l15-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l15: { ...state.l15, config: { ...state.l15.config, [s.key]: (e.target as HTMLSelectElement).value } } })"
              >
                <option v-for="op in s.options" :key="op.value" :value="op.value">{{ op.label }}</option>
              </select>
            </div>
          </div>
        </div>

        <div class="sub-box">
          <div class="inline-row">
            <strong>L21 武皇争霸 (7 cfg + `hiddenAvatar.css`) · 双将(4+3)体力=`{{ l21DoubleHpPreview }}`</strong>
            <label class="tiny">
              <input
                type="checkbox"
                :checked="Boolean(state.l21.hiddenPreviewActive)"
                data-testid="l21-toggle-hidden-preview"
                @change="(e) => pushState({ ...state, l21: { ...state.l21, hiddenPreviewActive: (e.target as HTMLInputElement).checked } })"
              />
              启用桌面暗置头像蒙版 (`hiddenAvatar.css`)
            </label>
          </div>
          <div class="cfg-grid">
            <div v-for="s in L21_CONFIG_SPECS" :key="s.key" class="cfg-row" :data-testid="`l21-cfg-row-${s.key}`">
              <span class="cfg-label">{{ s.label }} <em v-if="s.ruleCoupled" class="rule-tag">规则联动</em></span>
              <input
                v-if="s.control === 'boolean'"
                type="checkbox"
                :checked="Boolean(state.l21.config[s.key])"
                :data-testid="`l21-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l21: { ...state.l21, config: normalizeL21Config({ ...state.l21.config, [s.key]: (e.target as HTMLInputElement).checked }) } })"
              />
              <select
                v-else
                :value="String(state.l21.config[s.key])"
                :data-testid="`l21-cfg-input-${s.key}`"
                @change="(e) => {
                  const cfg = normalizeL21Config({ ...state.l21.config, [s.key]: (e.target as HTMLSelectElement).value });
                  const hideCount = Number(cfg.hide_number);
                  const hiddenIndices = Array.from({ length: hideCount }, (_, i) => Number(cfg.show_number) - 1 - i);
                  pushState({ ...state, l21: { config: cfg, hiddenIndices, hiddenPreviewActive: true } });
                }"
              >
                <option v-for="op in s.options" :key="op.value" :value="op.value">{{ op.label }}</option>
              </select>
            </div>
          </div>

          <strong>U05 杀海拾遗 (7 cfg)</strong>
          <div class="cfg-grid">
            <div v-for="s in U05_CONFIG_SPECS" :key="s.key" class="cfg-row" :data-testid="`u05-cfg-row-${s.key}`">
              <span class="cfg-label">{{ s.label }} <em v-if="s.ruleCoupled" class="rule-tag">规则联动</em></span>
              <input
                v-if="s.control === 'boolean'"
                type="checkbox"
                :checked="Boolean(state.u05.config[s.key])"
                :data-testid="`u05-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, u05: { config: { ...state.u05.config, [s.key]: (e.target as HTMLInputElement).checked } } })"
              />
              <select
                v-else-if="s.control === 'select'"
                :value="String(state.u05.config[s.key])"
                :data-testid="`u05-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, u05: { config: { ...state.u05.config, [s.key]: (e.target as HTMLSelectElement).value } } })"
              >
                <option v-for="op in s.options" :key="op.value" :value="op.value">{{ op.label }}</option>
              </select>
              <span v-else class="tiny muted" :data-testid="`u05-cfg-input-${s.key}`">长按提示</span>
            </div>
          </div>

          <textarea v-model="poolJsonText" rows="2" data-testid="pool-cfg-json-textarea"></textarea>
          <div class="inline-row">
            <button type="button" data-testid="btn-pool-import-json" @click="onImportPoolJson">导入校验 ({{ activePoolSource.toUpperCase() }})</button>
            <button type="button" data-testid="btn-tools-import-l02" @click="onImportToolsJson('l02')">校验 L02</button>
            <button type="button" data-testid="btn-tools-import-l16" @click="onImportToolsJson('l16')">校验 L16</button>
            <button type="button" data-testid="btn-tools-import-l18" @click="onImportToolsJson('l18')">校验 L18</button>
            <button type="button" data-testid="btn-tools-import-l26" @click="onImportL26Json">校验 L26</button>
            <span class="tiny" data-testid="pool-cfg-msg">{{ poolJsonMsg }}</span>
          </div>
        </div>
      </section>

      <!-- TAB 3: L02 + L16 + L18 -->
      <section v-else-if="activeTab === 'tools'" class="panel-scroll" data-testid="b7-panel-tools">
        <div class="sub-box">
          <strong>L02 SJ Settings (4 cfg + `extension.css`)</strong>
          <div class="cfg-grid">
            <div v-for="s in L02_CONFIG_SPECS" :key="s.key" class="cfg-row" :data-testid="`l02-cfg-row-${s.key}`">
              <span class="cfg-label">{{ s.label }}</span>
              <input
                v-if="s.control === 'boolean'"
                type="checkbox"
                class="sj-checkbox"
                :checked="state.l02.config.openTools"
                :data-testid="`l02-cfg-input-${s.key}`"
                @change="(e) => {
                  const on = (e.target as HTMLInputElement).checked;
                  pushState({ ...state, l02: { ...state.l02, config: { ...state.l02.config, openTools: on }, consoleLogsOpen: on } });
                }"
              />
              <button
                v-else
                type="button"
                :data-testid="`l02-cfg-input-${s.key}`"
                @click="
                  pushState({
                    ...state,
                    l02: {
                      ...state.l02,
                      tutorialDocOpen: s.key === 'tutorial' || s.key === 'tutorialapk' ? !state.l02.tutorialDocOpen : state.l02.tutorialDocOpen,
                      offlineCatalogOpen: s.key === 'getExtensions' ? !state.l02.offlineCatalogOpen : state.l02.offlineCatalogOpen,
                    },
                  })
                "
              >
                打开
              </button>
            </div>
          </div>
          <div v-if="state.l02.consoleLogsOpen" class="sj-console-drawer" data-testid="l02-console-drawer">
            [Eruda 净室控制台] {{ state.l02.offlineExtensionSummary }} · 零远端 eval
          </div>
          <div v-if="state.l02.tutorialDocOpen" class="sj-console-drawer" data-testid="l02-tutorial-drawer">
            [SJ Settings 离线打包与导入指南] 支持 JSON 结构化配置校验与 APK 离线扩展路径检查（禁用远端动态 eval 注入）。
          </div>
          <div v-if="state.l02.offlineCatalogOpen" class="sj-console-drawer" data-testid="l02-extensions-drawer">
            [离线扩展目录清单] {{ state.l02.offlineExtensionSummary }} · 原生扩展包 46 个 / 源 37 个。
          </div>
        </div>

        <div class="sub-box">
          <strong>L16 拖拽读取 (2 cfg + `extension.css`)</strong>
          <div class="inline-row">
            <button
              v-for="s in L16_CONFIG_SPECS"
              :key="s.key"
              type="button"
              :data-testid="`l16-cfg-input-${s.key}`"
              @click="() => {
                if (s.key === 'loadUpdateContent') {
                  pushState({ ...state, l16: { ...state.l16, changelogOpen: !state.l16.changelogOpen } });
                } else {
                  const msg = runL16ManifestDryRun(l16ArchiveInput);
                  pushState({ ...state, l16: { ...state.l16, lastZipManifest: msg } });
                }
              }"
            >
              {{ s.label }}
            </button>
            <input v-model="l16ArchiveInput" type="text" data-testid="l16-zip-name" />
            <span v-if="state.l16.lastZipManifest" class="drag-read-importZip" data-testid="l16-zip-manifest">{{ state.l16.lastZipManifest }}</span>
          </div>
          <div v-if="state.l16.changelogOpen" class="drag-read-updateContent" data-testid="l16-changelog-box">
            拖拽读取扩展历史：支持安全的包清单核验与受托导出文件名检查。
          </div>
        </div>

        <div class="sub-box">
          <strong>L18 更新保护 (12 cfg + `extension.css` `.zxgxProgress`)</strong>
          <div class="cfg-grid">
            <div v-for="s in L18_CONFIG_SPECS" :key="s.key" class="cfg-row" :data-testid="`l18-cfg-row-${s.key}`">
              <span class="cfg-label">{{ s.label }}</span>
              <input
                v-if="s.control === 'boolean'"
                type="checkbox"
                :checked="Boolean(state.l18.config[s.key])"
                :data-testid="`l18-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l18: { ...state.l18, config: { ...state.l18.config, [s.key]: (e.target as HTMLInputElement).checked } } })"
              />
              <select
                v-else-if="s.control === 'select'"
                :value="String(state.l18.config[s.key])"
                :data-testid="`l18-cfg-input-${s.key}`"
                @change="(e) => {
                  const val = (e.target as HTMLSelectElement).value;
                  const nextCfg = { ...state.l18.config, [s.key]: val };
                  pushState({
                    ...state,
                    l18: {
                      ...state.l18,
                      config: nextCfg,
                      linkExplainText: L18_EXPLAIN_MAP[nextCfg.update_link_explain],
                    },
                  });
                }"
              >
                <option v-for="op in s.options" :key="op.value" :value="op.value">{{ op.label }}</option>
              </select>
              <button
                v-else-if="s.control === 'action'"
                type="button"
                :data-testid="`l18-cfg-input-${s.key}`"
                @click="pushState({ ...state, l18: runL18ProgressCheck(state.l18, s.key === 'checkForUpdate' ? 'game' : 'asset') })"
              >
                检测
              </button>
              <span v-else class="tiny muted" :data-testid="`l18-cfg-input-${s.key}`">琉璃专属</span>
            </div>
          </div>
          <div class="inline-row">
            <progress class="zxgxProgress" max="100" :value="state.l18.progressPct" data-testid="l18-progress-bar"></progress>
            <span class="tiny" data-testid="l18-status-msg">{{ state.l18.statusMessage }} · {{ state.l18.linkExplainText }}</span>
          </div>
        </div>
      </section>

      <!-- TAB 4: U04/L05 + L12 + L23 + L26 -->
      <section v-else class="panel-scroll" data-testid="b7-panel-fx">
        <div class="sub-box">
          <strong>U04 / L05 金币与桌面特效 (`coin_arenaReady` + `{{ COIN_HAVE_FUN_SURFACE_IDS.join(', ') }}`)</strong>
          <div class="inline-row">
            <span class="pill" data-testid="u04-coin-hud-preview">{{ coinHudSummary.hudText || `${coin.balance}金` }}</span>
            <button type="button" data-testid="btn-u04-havefun-firework" @click="onToggleCoinHaveFun('firework')">烟花 (haveFun.firework)</button>
            <button type="button" data-testid="btn-u04-havefun-snow" @click="onToggleCoinHaveFun('snow')">下雪 (haveFun.snow)</button>
            <button type="button" data-testid="btn-u04-snow-size" @click="onToggleSnowSize">雪量={{ coin.snowSize }}</button>
            <button type="button" data-testid="btn-u04-havefun-star" @click="onToggleCoinHaveFun('star')">星云 (haveFun.star)</button>
            <button type="button" data-testid="btn-u04-havefun-blink" @click="onToggleCoinHaveFun('blink')">闪烁 (haveFun.blink)</button>
            <button type="button" data-testid="btn-u04-bet" @click="onPlaceCoinBet">获胜下注 (20金)</button>
          </div>
        </div>

        <div class="sub-box">
          <strong>L12 如真似幻 (6 surfaces，净室回退=`{{ state.l12.assetFallbackStatus }}`)</strong>
          <div class="inline-row">
            <button
              v-for="sid in L12_SURFACE_IDS"
              :key="sid"
              type="button"
              :class="{ active: state.l12.activeSurface === sid }"
              :data-testid="`btn-l12-surface-${sid}`"
              @click="pushState({ ...state, l12: { ...state.l12, activeSurface: sid } })"
            >
              {{ sid }}
            </button>
          </div>
          <div class="rzsh-preview-stage" :data-l12-surface="state.l12.activeSurface" data-testid="l12-rzsh-stage">
            <span class="splashtexthela">如真似幻 · 阶位 {{ state.l12.tierRoman }} · {{ state.l12.activeSurface }}</span>
            <span class="tiny muted"> 缩放={{ state.l12.standeeScale }} · X={{ state.l12.backgroundX }}/Y={{ state.l12.backgroundY }}</span>
          </div>
        </div>

        <div class="sub-box">
          <strong>L23 电脑适配 (`_fixCAILANZI`) · 点击阈值 &lt;10px</strong>
          <div class="inline-row">
            <button
              type="button"
              data-testid="btn-l23-sim-tap"
              @click="pushState({ ...state, l23: dispatchFixCaiLanZiTap(state.l23, { seat: 1, dx: 2, dy: 3, isSelf: false, isDead: false, isOver: false }) })"
            >
              模拟 P1 轻触砸蛋
            </button>
            <span class="tiny" data-testid="l23-tap-status">状态={{ state.l23.lastTapClassification }} · 目标=P{{ state.l23.emojiPickerSeat ?? "-" }}</span>
          </div>
        </div>

        <div class="sub-box">
          <strong>L26 笨蛋插件 (5 cfg + `extension.css` + `_Fool_chooseCard`)</strong>
          <div class="cfg-grid">
            <div v-for="s in L26_CONFIG_SPECS" :key="s.key" class="cfg-row" :data-testid="`l26-cfg-row-${s.key}`">
              <span class="cfg-label">{{ s.label }}</span>
              <input
                v-if="s.control === 'boolean'"
                type="checkbox"
                :checked="Boolean(state.l26.config[s.key])"
                :data-testid="`l26-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l26: { ...state.l26, config: { ...state.l26.config, [s.key]: (e.target as HTMLInputElement).checked } } })"
              />
              <select
                v-else
                :value="String(state.l26.config[s.key])"
                :data-testid="`l26-cfg-input-${s.key}`"
                @change="(e) => pushState({ ...state, l26: { ...state.l26, config: { ...state.l26.config, [s.key]: (e.target as HTMLSelectElement).value } } })"
              >
                <option v-for="op in s.options" :key="op.value" :value="op.value">{{ op.label }}</option>
              </select>
            </div>
          </div>
          <div class="inline-row">
            <label class="tiny">
              <input
                type="checkbox"
                :checked="state.l26.chooseCardEquipCopyEnabled"
                data-testid="l26-chk-equip-mirror"
                @change="(e) => pushState({ ...state, l26: { ...state.l26, chooseCardEquipCopyEnabled: (e.target as HTMLInputElement).checked } })"
              />
              启用 `_Fool_chooseCard` 装备区辅助映射（当前己方装备数={{ selfEquipsCount }}）
            </label>
            <span v-if="state.l26.config.Fool_skill" class="Fool-skilltip" data-testid="l26-skilltip">资援/敏思 ≤13 · 方统 =36</span>
            <span v-else class="tiny muted" data-testid="l26-skilltip-off">技能算牌辅助：已关闭</span>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop { position: fixed; inset: 0; z-index: 70; background: rgba(8, 10, 14, 0.8); display: flex; align-items: center; justify-content: center; padding: 0.75rem; }
.batch7-card { width: min(940px, 96vw); max-height: 90vh; display: flex; flex-direction: column; gap: 0.5rem; background: var(--bg-panel, #171412); border: 1px solid #e8a54b; border-radius: 12px; padding: 0.9rem 1.1rem; overflow: hidden; }
.modal-head { display: flex; justify-content: space-between; align-items: flex-start; gap: 0.6rem; }
.modal-head h3 { margin: 0; font-size: 0.96rem; }
.tab-bar { display: flex; flex-wrap: wrap; gap: 0.35rem; }
.tab-bar button.active, button.active { border-color: #e8a54b; color: #fde68a; background: rgba(232, 165, 75, 0.16); }
.panel-scroll { overflow-y: auto; flex: 1 1 auto; min-height: 0; display: flex; flex-direction: column; gap: 0.55rem; padding-right: 0.2rem; }
.cfg-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 0.32rem; }
.cfg-row { display: flex; justify-content: space-between; align-items: center; gap: 0.35rem; padding: 0.25rem 0.45rem; border-radius: 6px; border: 1px solid var(--border); background: rgba(24, 28, 36, 0.75); font-size: 0.74rem; }
.rule-tag { font-style: normal; font-size: 0.62rem; color: #fbbf24; border: 1px solid rgba(251, 191, 36, 0.45); border-radius: 4px; padding: 0 0.2rem; margin-left: 0.2rem; }
.sub-box { display: flex; flex-direction: column; gap: 0.35rem; padding: 0.45rem 0.6rem; border-radius: 8px; border: 1px solid var(--border); background: rgba(15, 20, 28, 0.65); }
.inline-row { display: flex; flex-wrap: wrap; align-items: center; gap: 0.35rem; }
.pill { padding: 0.1rem 0.4rem; border-radius: 999px; border: 1px solid var(--border); font-size: 0.7rem; }
.tiny { font-size: 0.72rem; margin: 0; }
textarea { width: 100%; font-family: var(--mono); font-size: 0.72rem; background: var(--bg); color: var(--text); border: 1px solid var(--border); border-radius: 6px; padding: 0.35rem; }
</style>
