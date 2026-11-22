<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { ObserverSnapshot } from "../../../types/engine";
import {
  createDefaultDecadeConfig,
  type DecadeUiConfig,
} from "../../../engine/surfaces/decadeConfigSchema";
import {
  createDefaultShoushaConfig,
  type ShoushaUiConfig,
} from "../../../engine/surfaces/shoushaConfigSchema";
import {
  createInitialBatch7UiSuiteState,
  triggerQyChongzhuHook,
  updateExtensionDecadeConfig,
  updateExtensionShoushaConfig,
  type Batch7UiSuiteState,
  type ExtensionSurfacesSnapshot,
} from "../../../engine/extensionSurfaceRuntime";
import type { BattleModalId } from "../BattleShoushaHud.vue";
import QhlyDetailModal from "./QhlyDetailModal.vue";
import SkinSwitchModal from "./SkinSwitchModal.vue";
import MvpSettlementModal from "./MvpSettlementModal.vue";
import OmnisearchModal from "./OmnisearchModal.vue";
import TaixuCampaignModal from "./TaixuCampaignModal.vue";
import DecadeConfigModal from "../surfaces/DecadeConfigModal.vue";
import ShoushaConfigModal from "../surfaces/ShoushaConfigModal.vue";
import ZuanConfigPanel from "../surfaces/ZuanConfigPanel.vue";
import Batch7WorkbenchModal from "../surfaces/Batch7WorkbenchModal.vue";

const props = defineProps<{
  activeModal: BattleModalId | null;
  surfaces: ExtensionSurfacesSnapshot;
  view: ObserverSnapshot | null;
  finished?: boolean;
}>();

const emit = defineEmits<{
  close: [];
  "update:surfaces": [next: ExtensionSurfacesSnapshot];
  restartMatch: [];
}>();

const selfEquipsCount = computed(() => {
  if (!props.view) return 0;
  return props.view.known_cards.filter(
    (c) => c.owner === props.view?.observer && c.zone === "Equip",
  ).length;
});

function updateBatch7(nextBatch7: Batch7UiSuiteState) {
  const next: ExtensionSurfacesSnapshot = {
    ...latestSurfaces.value,
    uiSuites: { ...latestSurfaces.value.uiSuites, batch7: nextBatch7 },
  };
  latestSurfaces.value = next;
  emit("update:surfaces", next);
}

const batch7OrDefault = computed(
  () => props.surfaces.uiSuites?.batch7 ?? createInitialBatch7UiSuiteState(),
);

const latestSurfaces = ref<ExtensionSurfacesSnapshot>(props.surfaces);
watch(
  () => props.surfaces,
  (next) => {
    latestSurfaces.value = next;
  },
);

function onUpdateDecadeConfig(patch: Partial<DecadeUiConfig>) {
  const next = updateExtensionDecadeConfig(latestSurfaces.value, patch);
  latestSurfaces.value = next;
  emit("update:surfaces", next);
}

function onResetDecadeConfig() {
  onUpdateDecadeConfig(createDefaultDecadeConfig());
}

function onUpdateShoushaConfig(patch: Partial<ShoushaUiConfig>) {
  const next = updateExtensionShoushaConfig(latestSurfaces.value, patch);
  latestSurfaces.value = next;
  emit("update:surfaces", next);
}

function onResetShoushaConfig() {
  onUpdateShoushaConfig(createDefaultShoushaConfig());
}

function onUpdateQhlyState(qhly: NonNullable<NonNullable<ExtensionSurfacesSnapshot["uiSuites"]>["qhly"]>) {
  const next: ExtensionSurfacesSnapshot = {
    ...latestSurfaces.value,
    uiSuites: { ...latestSurfaces.value.uiSuites, qhly },
  };
  latestSurfaces.value = next;
  emit("update:surfaces", next);
}
</script>

<template>
  <QhlyDetailModal
    v-if="activeModal === 'qhly'"
    :state="surfaces.uiSuites?.qhly"
    @close="emit('close')"
    @update="onUpdateQhlyState"
  />
  <SkinSwitchModal
    v-else-if="activeModal === 'skinswitch'"
    :state="surfaces.uiSuites?.skinSwitch"
    @close="emit('close')"
    @update="
      (skinSwitch) => {
        const next: ExtensionSurfacesSnapshot = {
          ...latestSurfaces,
          uiSuites: { ...latestSurfaces.uiSuites, skinSwitch },
        };
        latestSurfaces = next;
        emit('update:surfaces', next);
      }
    "
  />
  <MvpSettlementModal
    v-else-if="activeModal === 'mvp'"
    :mvp="surfaces.mvp"
    :l13="batch7OrDefault.l13"
    :view="view"
    @close="emit('close')"
    @update:l13="(l13) => updateBatch7({ ...batch7OrDefault, l13 })"
    @recast-cue="
      (seat) => {
        const res = triggerQyChongzhuHook(batch7OrDefault.l13, latestSurfaces.mvp, seat);
        const next: ExtensionSurfacesSnapshot = {
          ...latestSurfaces,
          mvp: res.mvp,
          uiSuites: {
            ...latestSurfaces.uiSuites,
            batch7: { ...batch7OrDefault, l13: res.l13 },
          },
        };
        latestSurfaces = next;
        emit('update:surfaces', next);
      }
    "
    @one-more="() => { if (finished) emit('restartMatch'); }"
  />
  <OmnisearchModal
    v-else-if="activeModal === 'omnisearch'"
    :state="surfaces.omnisearch"
    :l07="batch7OrDefault.l07"
    :view="view"
    @close="emit('close')"
    @update="(omnisearch) => emit('update:surfaces', { ...props.surfaces, omnisearch })"
    @update:l07="(l07) => updateBatch7({ ...batch7OrDefault, l07 })"
  />
  <TaixuCampaignModal
    v-else-if="activeModal === 'taixu'"
    :taixu="surfaces.taixu"
    :taixu-ui="surfaces.uiSuites?.taixuUi"
    @close="emit('close')"
    @update="
      ({ taixu, taixuUi }) =>
        emit('update:surfaces', {
          ...props.surfaces,
          taixu,
          uiSuites: { ...props.surfaces.uiSuites, taixuUi },
        })
    "
  />
  <DecadeConfigModal
    v-else-if="activeModal === 'decade-menu'"
    :state="surfaces.uiSuites?.decadeUi"
    @close="emit('close')"
    @update-config="onUpdateDecadeConfig"
    @reset-config="onResetDecadeConfig"
    @update-decade-ui="
      (decadeUi) =>
        emit('update:surfaces', {
          ...props.surfaces,
          uiSuites: { ...props.surfaces.uiSuites, decadeUi },
        })
    "
  />
  <ShoushaConfigModal
    v-else-if="activeModal === 'shousha-menu'"
    :state="surfaces.uiSuites?.shoushaUi"
    @close="emit('close')"
    @update-config="onUpdateShoushaConfig"
    @reset-config="onResetShoushaConfig"
  />
  <ZuanConfigPanel
    v-else-if="activeModal === 'zuan-panel'"
    :zuan-ui="surfaces.uiSuites?.zuanUi"
    :batch6="surfaces.uiSuites?.batch6Patch"
    @close="emit('close')"
    @update-zuan="
      (zuanUi) => {
        const next: ExtensionSurfacesSnapshot = {
          ...latestSurfaces,
          uiSuites: { ...latestSurfaces.uiSuites, zuanUi },
        };
        latestSurfaces = next;
        emit('update:surfaces', next);
      }
    "
    @update-batch6="
      (batch6Patch) => {
        const next: ExtensionSurfacesSnapshot = {
          ...latestSurfaces,
          uiSuites: { ...latestSurfaces.uiSuites, batch6Patch },
        };
        latestSurfaces = next;
        emit('update:surfaces', next);
      }
    "
  />
  <Batch7WorkbenchModal
    v-else-if="activeModal === 'batch7-workbench'"
    :batch7="batch7OrDefault"
    :coin="surfaces.coin"
    :self-equips-count="selfEquipsCount"
    @close="emit('close')"
    @update:batch7="updateBatch7"
    @update:coin="
      (coin) => {
        const next: ExtensionSurfacesSnapshot = { ...latestSurfaces, coin };
        latestSurfaces = next;
        emit('update:surfaces', next);
      }
    "
  />
</template>
