<script setup lang="ts">
import type { ObserverSnapshot } from "../../../types/engine";
import type { ExtensionSurfacesSnapshot } from "../../../engine/extensionSurfaceRuntime";
import type { BattleModalId } from "../BattleShoushaHud.vue";
import QhlyDetailModal from "./QhlyDetailModal.vue";
import SkinSwitchModal from "./SkinSwitchModal.vue";
import MvpSettlementModal from "./MvpSettlementModal.vue";
import OmnisearchModal from "./OmnisearchModal.vue";
import TaixuCampaignModal from "./TaixuCampaignModal.vue";

const props = defineProps<{
  activeModal: BattleModalId | null;
  surfaces: ExtensionSurfacesSnapshot;
  view: ObserverSnapshot | null;
}>();

const emit = defineEmits<{
  close: [];
  "update:surfaces": [next: ExtensionSurfacesSnapshot];
}>();
</script>

<template>
  <QhlyDetailModal
    v-if="activeModal === 'qhly'"
    :state="surfaces.uiSuites?.qhly"
    @close="emit('close')"
    @update="
      (qhly) =>
        emit('update:surfaces', {
          ...props.surfaces,
          uiSuites: { ...props.surfaces.uiSuites, qhly },
        })
    "
  />
  <SkinSwitchModal
    v-else-if="activeModal === 'skinswitch'"
    :state="surfaces.uiSuites?.skinSwitch"
    @close="emit('close')"
    @update="
      (skinSwitch) =>
        emit('update:surfaces', {
          ...props.surfaces,
          uiSuites: { ...props.surfaces.uiSuites, skinSwitch },
        })
    "
  />
  <MvpSettlementModal
    v-else-if="activeModal === 'mvp'"
    :mvp="surfaces.mvp"
    :view="view"
    @close="emit('close')"
  />
  <OmnisearchModal
    v-else-if="activeModal === 'omnisearch'"
    :state="surfaces.omnisearch"
    :view="view"
    @close="emit('close')"
    @update="(omnisearch) => emit('update:surfaces', { ...props.surfaces, omnisearch })"
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
</template>
