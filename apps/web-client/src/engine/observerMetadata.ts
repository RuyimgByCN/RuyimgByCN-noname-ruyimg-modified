import type { ChoiceContext, ChoiceRequest, GeneralSlotView, NativeChoiceKind, ObserverSnapshot, QualifiedSkillRef, R05EffectView } from "../types/engine";

const CHOICE_KINDS: Record<NativeChoiceKind, string> = {
  confirm: "Confirm", select_cards: "SelectCards", select_players: "SelectTargets",
  respond_card: "Custom", pick_tag: "Custom",
};

function isQualifiedSkillRef(value: unknown): value is QualifiedSkillRef {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const ref = value as Record<string, unknown>;
  return typeof ref.namespace === "string" && ref.namespace.length > 0 &&
    typeof ref.package_id === "string" && ref.package_id.length > 0 &&
    Number.isInteger(ref.logical_skill_def_id) && (ref.logical_skill_def_id as number) >= 0 && (ref.logical_skill_def_id as number) <= 0xffffffff;
}

function isSignedSlot(value: unknown): value is number | null {
  return value === null || (Number.isInteger(value) && (value as number) >= -0x80000000 && (value as number) <= 0x7fffffff);
}

export function r05Effects(value: unknown): R05EffectView[] {
  if (value === undefined) return [];
  if (!Array.isArray(value) || !value.every((item) => {
    if (!item || typeof item !== "object" || Array.isArray(item)) return false;
    const row = item as Record<string, unknown>;
    const slots = [row.tiqi_limit, row.baoshu_combs, row.dcbihuo_next, row.dcbihuo_current];
    return isQualifiedSkillRef(row.skill) && slots.every(isSignedSlot) && slots.some((slot) => slot !== null);
  })) throw new Error("native: r05_effects metadata invalid");
  return value as R05EffectView[];
}

export function choiceContext(value: unknown): ChoiceContext | undefined {
  if (value == null) return undefined;
  if (typeof value !== "object" || Array.isArray(value)) throw new Error("native: choice context metadata invalid");
  const ctx = value as Record<string, unknown>;
  if (ctx.kind !== "tiqi_adjustment" || !isQualifiedSkillRef(ctx.skill) ||
    !Number.isInteger(ctx.target) || (ctx.target as number) < 0 || (ctx.target as number) > 256 ||
    !Number.isInteger(ctx.delta) || (ctx.delta as number) <= 0 || (ctx.delta as number) > 0xffffffff) {
    throw new Error("native: choice context metadata invalid");
  }
  return value as ChoiceContext;
}

/** JSON metadata is optional for old producers; malformed present values are errors. */
export function generalSlots(value: unknown): [GeneralSlotView | null, GeneralSlotView | null] {
  if (value === undefined) return [null, null];
  if (!Array.isArray(value) || value.length !== 2 || !value.every((slot) => slot === null || (
    typeof slot === "object" && !Array.isArray(slot) && Number.isInteger(slot.def) && slot.def >= 0 && slot.def <= 0xffffffff &&
    typeof slot.name === "string" && typeof slot.faction === "string" && typeof slot.revealed === "boolean"
  ))) throw new Error("native: generals metadata invalid");
  return value as [GeneralSlotView | null, GeneralSlotView | null];
}

export function choiceMetadata(value: { prompt?: unknown; native_kind?: unknown; source_skill?: unknown; context?: unknown; kind?: string }): Pick<ChoiceRequest, "prompt" | "native_kind" | "source_skill" | "context" | "kind"> {
  if (value.prompt != null && typeof value.prompt !== "string") throw new Error("native: prompt metadata invalid");
  if (value.native_kind != null && (typeof value.native_kind !== "string" || !Object.hasOwn(CHOICE_KINDS, value.native_kind))) throw new Error("native: native_kind metadata invalid");
  if (value.source_skill != null && (typeof value.source_skill !== "string" || value.source_skill.length === 0)) throw new Error("native: source_skill metadata invalid");
  const context = choiceContext(value.context);
  const native_kind = value.native_kind as NativeChoiceKind | undefined;
  return { ...(value.source_skill == null ? {} : {source_skill: value.source_skill as string}), ...(context === undefined ? {} : {context}), ...(value.prompt == null ? {} : {prompt: value.prompt as string}), ...(native_kind == null ? {} : {native_kind}), kind: native_kind === "pick_tag" && value.prompt === "trigger_order" ? "OrderTriggers" : native_kind ? CHOICE_KINDS[native_kind] : value.kind ?? "Custom" };
}

const KIND_LABELS: Record<string, string> = {
  confirm: "确认", select_cards: "选择卡牌", select_players: "选择角色", respond_card: "响应出牌", pick_tag: "选择选项",
  Confirm: "确认", SelectCards: "选择卡牌", SelectTargets: "选择角色", Custom: "选择选项", OrderTriggers: "选择下一个技能", DiscardToLimit: "弃牌至手牌上限",
};
const PROMPTS: Record<string, string> = {
  trigger_order: "请选择下一个发动的技能",
  trigger_confirm: "是否发动技能？", respond_shan: "请打出闪", respond_sha: "请打出杀", aoe_response: "请响应群体锦囊",
  dying_save: "请选择救援牌", discard_to_limit: "请弃牌至手牌上限",
  tuxi_target: "突袭：选择获得手牌的角色", guanxing_arrange: "观星：排列牌堆顶的卡牌",
  xunxun_arrange: "恂恂：选择获得的卡牌",
  jijie_give_target: "机捷：选择获得此牌的角色",
  gaiming_pick: "改命：选择判定牌", haotianta_pick: "昊天塔：选择判定牌", liaoyuan_discard: "燎原：选择弃置的卡牌",
  fanjian_target: "反间：选择目标角色", fanjian_suit: "反间：选择花色", fanjian_pick: "反间：选择获得的手牌",
  rvm_tiqi_confirm: "涕泣：是否发动？", rvm_tiqi_sign: "涕泣：选择手牌上限增减",
  rvm_baoshu_cost: "宝梳：选择目标角色", rvm_dcbihuo_target: "避祸：选择目标角色",
  cangchizhibi_target: "苍螭之璧：选择至多三名角色横置或重置",
};

export function choiceKindLabel(choice: ChoiceRequest): string {
  const kind = choice.native_kind ?? choice.kind ?? "Custom";
  return Object.hasOwn(KIND_LABELS, kind) ? KIND_LABELS[kind] : "选择选项";
}

export function choicePromptLabel(choice: ChoiceRequest): string {
  if (choice.prompt === "trigger_confirm" && choice.source_skill) return `是否发动【${choice.source_skill}】？`;
  if (!choice.prompt) return choiceKindLabel(choice);
  return Object.hasOwn(PROMPTS, choice.prompt) ? PROMPTS[choice.prompt] : choiceKindLabel(choice);
}

export function validateChoiceCounts(choice: Pick<ChoiceRequest, "min_count" | "max_count">): void {
  for (const value of [choice.min_count, choice.max_count]) {
    if (value !== undefined && (!Number.isInteger(value) || value < 0 || value > 0xffffffff)) throw new Error("native: choice counts invalid");
  }
  if (choice.min_count !== undefined && choice.max_count !== undefined && choice.min_count > choice.max_count) throw new Error("native: choice counts invalid");
}

/** Check optional metadata before an IPC/network payload enters the view store. */
export function validateObserverMetadata(view: ObserverSnapshot, choice: ChoiceRequest | null): void {
  for (const seat of view.seats ?? []) {
    generalSlots(seat.generals);
    r05Effects(seat.r05_effects);
    if (seat.linked !== undefined && typeof seat.linked !== "boolean") throw new Error("native: linked metadata invalid");
  }
  if (choice) {
    validateChoiceCounts(choice);
    choiceMetadata(choice);
    if (choice.deadline_unix_ms != null && (!Number.isSafeInteger(choice.deadline_unix_ms) || choice.deadline_unix_ms <= 0)) throw new Error("native: choice deadline invalid");
  }
}
