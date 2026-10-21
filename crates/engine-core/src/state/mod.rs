//! Canonical game state and entity stores.

mod cards;
mod players;
mod skills;
mod turn;
mod zones;

pub use cards::{Card, CardKind, CardStore, EquipSlot};
pub use players::{Identity, Player, PlayerStore};
// re-export Identity at crate root via lib.rs
pub use skills::{SkillInstance, SkillRuntimeStore, GENERAL_SLOT_NONE};
pub use turn::{turn_flag, ExtraPhaseTurn, Phase, TurnState};
pub use zones::{Zone, ZoneKind, ZoneStore};

use crate::content::ContentRegistry;
use crate::query::ActiveModifierIndex;
use crate::rng::RuleRngState;
use crate::trigger::ActiveTriggerIndex;
use engine_api::{CardId, MatchOutcome, PlayerId, RulesetId, SkillId};
use rule_api::EffectRequest;
use serde::{Deserialize, Serialize};

/// Default per-segment effect budget (replay header / ruleset config).
pub const DEFAULT_EFFECT_BUDGET: u32 = 10_000;
/// Default trigger chain budget.
pub const DEFAULT_TRIGGER_BUDGET: u32 = 1_000;
/// Default draw count in phaseDraw (noname default `event.num = 2`).
pub const DEFAULT_DRAW_COUNT: u32 = 2;
pub(crate) const POTDIANYI_DEF: u32 = 4_200_000_573;
pub(crate) const POTSHEQUAN_DEF: u32 = 4_200_000_574;
pub(crate) const POTJIANZHUAN_DEF: u32 = 4_200_000_575;
pub(crate) const POTNIZUN_DEF: u32 = 4_200_000_576;
pub(crate) const SHEQUAN_CARD_TAG: u32 = POTSHEQUAN_DEF;
pub(crate) const POTFUJI_DEF: u32 = 4_200_000_578;
pub(crate) const POTFUJI_EFFECT_DEF: u32 = 4_200_000_579;
pub(crate) const POTFUJI_SHA_DEF: u32 = 4_200_000_580;
pub(crate) const POTFUJI_SHAN_DEF: u32 = 4_200_000_581;
pub(crate) const POTFUJI_CARD_TAG: u32 = POTFUJI_DEF;

/// Stable lookup entry for one public, skill-owned expansion zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillPileRef {
    /// Pile owner.
    pub owner: PlayerId,
    /// Skill definition that owns the pile.
    pub skill_def: u32,
    /// Canonical expansion zone.
    pub zone: engine_api::ZoneId,
    /// Whether cards in this pile may be used or responded with as if they were hand cards.
    #[serde(default)]
    pub usable_as_hand: bool,
    /// Opt-in: count cards lost by this pile's owner each turn (沙城 draw-X).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub wants_lost_count: bool,
}

#[cfg(test)]
mod resolution_tests {
    use super::ResolutionState;
    use engine_api::{CardId, PlayerId};

    #[test]
    fn converted_card_resolution_keeps_every_physical_material() {
        let mut resolution = ResolutionState::default();
        let player = PlayerId::new(1);
        let carrier = CardId::new(2);
        let extra = CardId::new(3);
        resolution.plan_card_source_cards(player, carrier, vec![carrier, extra], true);
        resolution.register_card_resolution(player, carrier, 1, Vec::new(), true, 0);
        assert_eq!(
            resolution.card_source_cards(player, carrier),
            [carrier, extra]
        );
    }
}

/// 完整对局状态（Arc 化内容目录：克隆时目录字节不复制）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    /// Monotonic revision; increments on each successful commit.
    pub revision: u64,
    /// Ruleset reference.
    pub ruleset: RulesetRef,
    /// Turn / phase.
    pub turn: TurnState,
    /// Persistent per-player hand-limit bonuses (穆荫 clanmuyin and friends).
    /// Keyed by raw PlayerId; serde-defaulted for replay compatibility.
    #[serde(default)]
    pub hand_limit_bonus: std::collections::BTreeMap<u32, i32>,
    /// Pending extra trick-effect passes queued by 额外结算 (effectCount++) skills
    /// (clanfenshi family), keyed by source card raw id. serde-defaulted.
    #[serde(default)]
    pub extra_effect_count: std::collections::BTreeMap<u32, u32>,
    /// Players.
    pub players: PlayerStore,
    /// Match-time faction changes keyed by player; absent means the character's base faction.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub group_overrides: std::collections::BTreeMap<PlayerId, String>,
    /// §7 技能夺取设施 S1 (ABI 676): skill defs each player gained via
    /// `choose_gain_other_skill`/`lose_gained_skill`, keyed by owner raw id.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub facility_gained_skills: std::collections::BTreeMap<u32, Vec<u32>>,
    /// Deferred per-target exclusions for multi-target trick resolutions whose wuxie
    /// payload is not yet suspended when the triggering skill's effect runs (啖酪
    /// danlao 免除臂, 批 155): `(source card raw, target raw)` pairs consumed by
    /// `advance_aoe_queue` per target. serde-defaulted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deferred_card_exclusions: Vec<(u32, u32)>,
    /// Guozhan ambition-lord (野心家) alliances: each entry records a bidirectional
    /// alliance between two players (`拉拢人心`). Empty in identity mode. See ADR 0598.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub ambition_alliances: std::collections::BTreeSet<[u32; 2]>,
    /// Hand cards deliberately turned face-down by content rules.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub face_down_hand_cards: std::collections::BTreeSet<CardId>,
    /// ADR 0644 P3: debate opinions pinned per player (player.raw() -> card ids).
    /// Multi-card pins count one vote each (jsrgxiongbao double opinion).
    #[serde(default)]
    pub debate_pinned_opinions: std::collections::BTreeMap<u32, Vec<CardId>>,
    /// ADR 0644 P3: participants of the debate currently opening (DebateStarted
    /// dispatch window), consumed by finish_debate.
    #[serde(default)]
    pub debate_pending_participants: Vec<u32>,
    /// ADR 0644 P3: initiator of the pending debate.
    #[serde(default)]
    pub debate_pending_initiator: u32,
    /// ADR 0644 P3 延迟议事族：发起者 -> 回合末议事参与者名单。
    #[serde(default)]
    pub pending_turn_end_debates: std::collections::BTreeMap<u32, Vec<u32>>,
    /// Persistent content-defined tags attached to physical card entities.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub content_card_tags: std::collections::BTreeMap<CardId, std::collections::BTreeSet<u32>>,
    /// Cards.
    pub cards: CardStore,
    /// Zones.
    pub zones: ZoneStore,
    /// Skill runtime instances.
    pub skills: SkillRuntimeStore,
    /// Active trigger index (derived from skills + catalog; rebuilt on restore).
    #[serde(default)]
    pub triggers: ActiveTriggerIndex,
    /// Active modifier index (derived; rebuilt on restore).
    #[serde(default)]
    pub modifiers: ActiveModifierIndex,
    /// Loaded content packages (modules + trigger overlay). Affects ruleset.logic_hash.
    #[serde(default)]
    /// 只读规则目录（内容注册表）：Arc 共享，克隆 O(1)。
    /// 唯一可变路径为包安装（Arc::make_mut 写时复制）。
    /// serde 透明编码为 ContentRegistry（快照格式不变）。
    #[serde(
        serialize_with = "serialize_arc_content",
        deserialize_with = "deserialize_arc_content"
    )]
    pub content: std::sync::Arc<ContentRegistry>,
    /// Match-lifetime directed pairs `(damage source, damaged player)`.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub damage_history: std::collections::BTreeSet<(PlayerId, PlayerId)>,
    /// Distinct players who have entered dying at least once during this match.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub lifetime_dying_players: std::collections::BTreeSet<PlayerId>,
    /// Match-lifetime damage points dealt by source player.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub damage_dealt_totals: std::collections::BTreeMap<PlayerId, u32>,
    /// Match-lifetime number of cards used or played as a response by each player.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub card_action_totals: std::collections::BTreeMap<PlayerId, u32>,
    /// Match-lifetime number of discard-pile reshuffles into the draw pile.
    #[serde(default, skip_serializing_if = "u32_is_zero")]
    pub draw_reshuffle_count: u32,
    /// Match-lifetime count of dying-window entries (every EnteredDying event).
    /// Drives second-dying awakening gates (spolaige 觉醒族, 批 293).
    #[serde(default, skip_serializing_if = "u32_is_zero")]
    pub lifetime_dying_events: u32,
    /// Match-lifetime count of basic-card uses (every basic CardUsed). Drives
    /// dynamic view_as source-card counts (spolxixiang 西向族, 批 297).
    #[serde(default, skip_serializing_if = "u32_is_zero")]
    pub lifetime_basic_card_uses: u32,
    /// Match-lifetime bitmask of used card type-groups (bit0=basic, bit1=trick,
    /// bit2=equip; sanshou 三首, 批 307). Unconditionally maintained; serde
    /// skips zero so packages that never use cards keep stable hashes.
    #[serde(default, skip_serializing_if = "u8_is_zero")]
    pub lifetime_used_type_groups: u8,
    /// Expansion-zone lookup keyed by `(owner, skill definition)`.
    ///
    /// The cards themselves remain canonical zone members; this index only gives each skill a
    /// stable, reusable public pile without adding per-skill state fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_piles: Vec<SkillPileRef>,
    /// Players whose next judgment phase is skipped by a persistent content effect.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pending_judge_skips: Vec<PlayerId>,
    /// Players whose judgment zones have been permanently abolished.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub disabled_judge_zones: std::collections::BTreeSet<PlayerId>,
    /// Resolution / choice barrier state.
    pub resolution: ResolutionState,
    /// Rule RNG state.
    pub rng: RuleRngState,
    /// Match outcome.
    pub outcome: MatchOutcome,
}

/// Ruleset pointer (logical package).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesetRef {
    /// Internal id.
    pub id: RulesetId,
    /// Package logical name.
    pub name: String,
    /// Logic hash of loaded content (zeros if empty bootstrap).
    pub logic_hash: [u8; 32],
}

impl Default for RulesetRef {
    fn default() -> Self {
        Self {
            id: RulesetId::new(1),
            name: "bootstrap".into(),
            logic_hash: [0u8; 32],
        }
    }
}

/// What the open choice will resume (serializable continuation, not a Promise).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u16)]
pub enum ContinuationKind {
    /// No pending continuation.
    #[default]
    None = 0,
    /// Current responder chooses a deterministic card/source-free save (1/token) or passes (0).
    DyingSave = 1,
    /// Discard phase: select cards down to hand limit.
    DiscardToLimit = 2,
    /// After 过河/顺手: pick a card from target's hand (token = CardId).
    /// payload: `[target, actor, mode]` mode 0=discard 1=take.
    PickTargetCard = 3,
    /// Target responds to 杀 with 闪 (or view_as 闪 / 八卦). Cancel = take damage.
    /// payload: `[source, target, damage_amount, need_count, have_count, nature]`.
    RespondToSha = 4,
    /// Declarative active-skill card selection.
    /// payload: `[skill_id, target_count, target0, ...]`.
    ActiveSkillSelection = 5,
    /// Ordered multi-target response queue (南蛮/万箭/多目标杀).
    /// AOE payload: `[kind, source, amount, idx, len, t0, t1, ...]`.
    /// Multi-target 杀 payload: `[3, source, amount, idx, len, nature, need, have, t0, ...]`.
    /// kind: 1=need_sha(南蛮) 2=need_shan(万箭) 3=multi-target_sha
    RespondAoeQueue = 6,
    /// 决斗: next_responder must play 杀 or take damage from other.
    /// payload: `[source, target, next_responder, exchange_count]`
    JuedouDuel = 7,
    /// Package-defined judgment rewrite with a hand card.
    /// payload: `[owner, delayed, suit, rank, kind_tag, rewriter, rest_len, r0...]`
    RewriteJudgment = 8,
    /// 借刀: holder chooses give weapon (0) or sha victim (1).
    /// payload: `[actor, holder, victim]`
    JiedaoChoice = 9,
    /// 火攻 step1: target shows a hand card.
    /// payload: `[actor, target]`
    HuogongShow = 10,
    /// 火攻 step2: actor discards same-suit card or cancel.
    /// payload: `[actor, target, suit_tag]`
    HuogongDiscard = 11,
    /// 无懈可击窗口：对锦囊生效前 / 对无懈的无懈。
    /// payload: `[nullified, ask_idx, ask_len, p0..p{n-1}, effect_tag, actor, ...args]`
    /// `nullified` 奇偶：1=当前被无懈抵消；选手牌无懈则翻转并重启询问。
    WuxieWindow = 12,
    /// DSL `choose{}`: a Rule VM module paused at HOST_CHOICE. The serialized VM
    /// continuation is held in `ResolutionState.vm_continuation`; the offered
    /// option tokens are the choice options. On ResolveChoice the selected token
    /// is pushed back into the VM and the module resumes.
    VmResume = 13,
    /// Prepare-phase scry: player viewed top N draw-pile cards; selected tokens go to the
    /// bottom (in selection order), unselected stay on top (original order).
    /// payload: `[player, n, c0, c1, ...]` where c0.. are the peeked top cards
    /// listed bottom→top (i.e. draw order last).
    ReorderDrawPile = 14,
    /// Content-defined draw replacement.
    /// Selection token 1 = replace draw; 0 / cancel = draw normally.
    /// payload: `[player, draw_count]`.
    DrawReplacement = 15,
    /// Owner confirms an optional trigger before its fixed task executes.
    OptionalTrigger = 16,
    /// Owner confirms each optional draw bonus before the normal draw resolves.
    /// payload: `[player, accumulated_draw, advance_after, bonus0, ...]`.
    OptionalDrawBonus = 17,
    /// Forced self-discard choice from hand or equipment.
    /// payload: `[player]`.
    DiscardOneHandOrEquip = 18,
    /// A declared single-target Sha paused in a CardUsed trigger.
    /// payload: `[source, target, base_amount, nature, respondable, card, excluded_targets...]`.
    ResumePlayedSha = 19,
    /// A declared multi-target Sha paused in a CardUsed trigger.
    /// payload: `[source, base_amount, nature, respondable, card, len, targets..., excluded_targets...]`.
    ResumePlayedShaQueue = 20,
    /// Reveal/discard-by-suit/use option state machine.
    /// payload: `[stage, player, mark, base_count, max_mark, completed, len, cards..., extra...]`.
    RevealDiscardSuitGainOrUse = 21,
    /// Discard-target and/or fire-damage option state machine.
    /// payload: `[stage, player, max_discard, damage, selected, target]`.
    DiscardOrFireDamageSkipPhases = 22,
    /// Optional draw-phase reorder of up to four cards. Selection order is the desired draw order
    /// followed by exactly two cards in bottom-to-top order.
    /// payload: `[player, n, c0, c1, ...]`, with peeked cards listed bottom-to-top.
    ReorderDrawPileFourSplitTwoThenDraw = 23,
    /// Forced transfer of one selected hand/equipment card.
    /// payload: `[from, to, drawn_only, len, card0, ...]`.
    GiveOneHandOrEquip = 24,
    /// Optional hand discard that grants one of two skills by effective card color.
    /// payload: `[player, red_skill_def, black_skill_def]`.
    DiscardHandAddSkillByColor = 25,
    /// Optional hand-card swap after revealing a basic draw-pile top card.
    /// payload: `[player, revealed_card]`.
    RevealTopGainNonBasicOrSwapBasic = 26,
    /// Draw-or-recover choice for the specified player.
    /// payload: `[player, draw_count, recover_count, allow_cancel]`.
    ChooseDrawOrRecover = 27,
    /// Select targets for a materialized virtual Sha.
    /// payload: `[player, card, ignore_distance, ignore_use_count, draw_on_no_use, skill]`.
    VirtualShaTargets = 28,
    /// Forced exact hand-card discard for `adjust_hand_to`.
    /// payload: `[player, need_count]`.
    AdjustHandTo = 29,
    /// Forced other-player target after a matching standalone judgment.
    /// payload: `[player, amount, nature]`.
    JudgeDamageOtherOnSuit = 30,
    /// Source-free conversion cost paused before proactive virtual-card use.
    SourceFreeViewAsPlay = 31,
    /// Select one enabled equipment slot for a source-free response conversion cost.
    SourceFreeViewAsEquipCost = 32,
    /// Source-free conversion cost paused before completing a response/save action.
    SourceFreeViewAsResponse = 33,
    /// Continue an independently resolved multi-target ordinary trick.
    /// payload: `[card, actor, kind, suit, effect_tag, next_index, len, targets...]`.
    IndependentTrickQueue = 34,
    /// Move-equipment state machine.
    /// Stage 0 payload: `[0, chooser, skill, option_count, card0, ...]`.
    /// Stage 1 payload: `[1, chooser, skill, source, card, option_count, player0, ...]`.
    MoveEquipmentThenDraw = 35,
    /// Choose one equipment card from each of two targets, with at least one armor.
    /// Stage 0 payload: `[0, chooser, skill, target0, target1]`.
    /// Stage 1 payload: `[1, chooser, skill, target0, target1, first_card]`.
    DiscardEquipmentPairWithArmor = 36,
    /// Forced target-equipment discard; a discarded mount is then gained by the chooser.
    /// payload: `[chooser, target]`.
    DiscardTargetEquipmentGainMount = 37,
    /// Forced scoped target-card discard; a discarded mount is then gained by the chooser.
    /// payload: `[chooser, target, scope]`.
    DiscardTargetCardGainMount = 38,
    /// Forced scoped target-card discard without any follow-up gain.
    /// payload: `[chooser, target, scope]`.
    ChooseDiscardTargetCard = 39,
    /// Privately view draw-pile top cards and optionally gain a matching-suit subset.
    /// payload: `[player, suit, n, c0, c1, ...]`, with cards listed bottom-to-top.
    ChooseGainTopCardsBySuit = 40,
    /// Repeated optional other-player choice with empty-hand-sensitive draw count.
    ChooseOtherDrawByHandCount = 41,
    /// Optional one-card self-discard followed by gaining a target equipment slot.
    /// payload: `[player, target, slot]`.
    ChooseDiscardOneThenGainTargetEquipment = 42,
    /// Forced target choice from a typed living-player scope, followed by drawing cards.
    /// payload: `[player, count, scope, skill]`.
    ChooseTargetDraw = 43,
    /// Forced one-card self-discard before refunding the current Sha use and conditionally drawing.
    /// payload: `[player, sha_user, card, skill]`.
    ChooseDiscardOneForShaRefundAndDrawIfRed = 44,
    /// Optional extra target for the currently saved proactive Sha.
    /// payload: `[player, card, skill]`.
    ChooseAddCurrentShaTarget = 45,
    /// Engine-owned post-use continuation for one single-target proactive Sha.
    /// payload: `[player, card, packed_damage_context]`.
    FinishPlayedSha = 46,
    /// Choose one or more hand Sha to store in the current skill's public pile.
    /// payload: `[player, skill]`.
    ChooseStoreHandShaInSkillPile = 47,
    /// Spend one current-skill pile card to make the current dying player use virtual Jiu.
    /// payload: `[player, dying, skill]`.
    ChooseSpendSkillPileForDyingJiu = 48,
    /// Target chooses one hand card against the active invocation's already selected hand card.
    /// payload: `[player, target, player_card, win_skill_def, lose_skill_def, skill]`.
    ChooseCompareTargetHandCard = 49,
    /// Optional confirmation/commit of the current skill instance's bound extra card target.
    /// payload: `[player, card, bound_target, skill]`.
    ChooseAddSkillBoundCardTarget = 50,
    /// Optional matching-color non-basic discard before granting a temporary color-lock helper.
    /// payload: `[player, target, skill_def, skill]`.
    ChooseDiscardMatchingEquipColorAddSkill = 51,
    /// Forced selection of one or two enhancements for a suspended exact card use.
    /// payload: `[player, card, helper_skill_def, skill]`.
    ChooseCurrentCardUseEnhancements = 52,
    /// Forced legal extra-target selection for a suspended Sha or ordinary trick.
    /// payload: `[player, card, skill]`.
    ChooseAddCurrentCardTarget = 53,
    /// Two-stage forced draw/discard branch and exact one-card discard cost.
    /// payload: `[stage, player, draw_skill_def, discard_skill_def, skill]`.
    ChooseDrawOrDiscardAddSkill = 54,
    /// Repeated optional use of cards from one discard batch as Bingliang.
    /// Stage 0: `[0, player, skill, len, cards...]`.
    /// Stage 1: `[1, player, skill, selected_card, len, remaining_cards...]`.
    ChooseDiscardedBlackBasicAsBingliang = 55,
    /// Optional judgment-card discard before a fixed virtual Sha with a no-damage draw.
    /// payload: `[player, target, skill]`.
    ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage = 56,
    /// Owner branch choice followed by an optional forced target discard selection.
    /// payload: `[stage, player, target, skill, count]`.
    ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded = 57,
    /// Two-stage target/card choice for revealing another player's hand card and remembering
    /// its logical category for the current Play phase.
    /// Stage 0: `[0, player, skill]`; stage 1: `[1, player, skill, target]`.
    ChooseRevealOtherHandCardRememberCategory = 58,
    /// Forced exact-card gift after a draw, followed by recording the current Play-phase target.
    /// payload: `[player, target, skill, count]`.
    GiveExactRecordPhaseTarget = 59,
    /// Multi-stage Xinzhongyong recipient/group/follow-up-Sha choice.
    /// Stage 0 stores the resolved card groups; stage 1 adds the chosen recipient; stage 2
    /// selects a physical Sha source; stage 3 selects its targets.
    ChooseGainResolvedShaOrShanThenOptionalUseSha = 60,
    /// Resume the automatic work of an entered Draw phase after a phase-entry trigger choice.
    /// payload: `[player]`.
    ContinueEnteredDrawPhase = 61,
    /// Forced ordered target selection for damage capped by this turn's skipped-phase count.
    /// payload: `[player, amount, skill]`.
    ChooseOtherTargetsBySkippedPhaseCountDamage = 62,
    /// Optional other-player selection after publicly revealing a uniform-color hand.
    /// payload: `[player, max_others, skill, current, phase]`.
    ChooseOthersAfterUniformHandReveal = 63,
    /// Choose another legal target for a suspended single-target card use.
    /// payload: `[player, card, skill]`.
    ChooseCardUserOrAddTarget = 64,
    /// The selected player gives one hand card and becomes the card user, or cancels and becomes
    /// an extra target. payload: `[player, card, skill, target, current, phase]`.
    ChooseGiveCardOrAddTarget = 65,
    /// Choose one effective equipment card to discard and reduce bound damage.
    /// payload: `[player, damage_id, skill]`.
    ChooseDiscardEquipmentReduceCurrentDamage = 66,
    /// Qieting branch/card state machine.
    /// Stage 0 payload: `[0, player, target, skill]`.
    /// Stage 1 payload: `[1, player, target, skill, option_count, card0, ...]`.
    ChooseMoveTargetEquipmentOrDraw = 67,
    /// Xianzhou recipient chooses recovery by canceling or one to the captured equipment count
    /// of attack-range damage targets.
    /// payload: `[owner, recipient, count, skill]`.
    ChooseRecoverOwnerOrDamageByGivenEquipment = 68,
    /// Qingxi-style bound Sha damage choice: the target discards exactly the source weapon's
    /// physical range from hand or the damage increases by one.
    /// payload: `[player, target, damage_id, skill, required, option_count, card0, ...]`.
    ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage = 69,
    /// Choose a different-hand-count target; lower draws, higher loses one deterministic-random
    /// hand card. payload: `[player, skill, option_count, player0, ...]`.
    ChooseHandCountDifferentTargetDrawOrRandomDiscard = 70,
    /// Rezhenjun state machine. Legacy stages 0/1/2 remain frozen; Decade Zhenjun appends
    /// stages 3/4/5 for target, target-card, and optional one-card owner choices.
    /// payload starts with the stage tag.
    ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw = 71,
    /// Eu Tuxi sequential gain choice. Payload snapshots the owner, skill, ordered target list,
    /// current index, and the current card options.
    ChooseGainOneFromDamageTargetAndNeighbors = 72,
    /// Forced damage-source branch: prevent damage, or lose max HP and increase it.
    /// payload: `[player, damage_id, amount, skill]`.
    ChoosePreventDamageOrLoseMaxHpIncreaseDamage = 73,
    /// Qiaoshui one-shot target adjustment. Stage 0 chooses add/remove; stage 1 chooses target.
    /// payload starts with the stage tag.
    ChooseAddOrRemoveCurrentCardTarget = 74,
    /// First participant chooses another living hand-comparison participant.
    /// payload: `[player, win_skill_def, lose_skill_def, skill]`.
    ChooseHandCompareTarget = 75,
    /// First participant chooses their own comparison card.
    /// payload: `[player, target, win_skill_def, lose_skill_def, skill]`.
    ChooseHandComparePlayerCard = 76,
    /// Choose one other living player and deal fixed normal damage.
    /// payload: `[player, amount, skill]`.
    ChooseOtherDamage = 77,
    /// Choose another living player with a discardable card, then reuse the typed target-card
    /// discard choice. payload: `[player, scope, skill]`.
    ChooseOtherDiscardTargetCard = 78,
    /// Choose one matching physical card from the discard pile and gain it.
    /// payload: `[player, card_kind, skill]`.
    ChooseGainDiscardCardByKind = 79,
    /// Choose one candidate skill definition not already owned and gain it.
    /// payload: `[player, skill, first, second, third]`.
    ChooseAddMissingSkill = 80,
    /// Choose another living player, draw by the owner's current lost HP, then turn that target
    /// over. payload: `[player, skill]`.
    ChooseOtherDrawOwnerLostHpTurnOver = 81,
    /// Resume a delayed trick after completed-judgment triggers finish.
    /// payload: `[owner, delayed, kind, suit, rank, rest_len, rest...]`.
    FinishDelayedJudgment = 82,
    /// 乱武 source-card / nearest-target selection.
    /// payload stage 0: `[0, owner, current, skill, rest_len, rest...]`;
    /// stage 1 additionally stores the selected source card before `rest_len`.
    ForceOthersUseShaOnNearestOrLoseHp = 83,
    /// Jianchu target-card choice bound to one saved physical Sha.
    /// payload: `[source, target, card]`.
    ChooseDiscardTargetCardResolveCurrentSha = 84,
    /// Choose one target equipment card to return to that target's hand, then compare range.
    /// payload: `[player, target, skill, option_len, options...]`.
    ChooseTargetReclaimEquipmentDrawIfRangeReduced = 85,
    /// Mandatory target choice for a materialized source-free virtual card.
    /// payload: `[player, card, kind, skill, option_count, option0, ...]`.
    ChooseUseVirtualCard = 86,
    /// Optional other-player target choice before that target must draw or recover.
    /// payload stage 0: `[0, player, draw_count, recover_count, times, skill]`;
    /// stage 1: `[1, player, target, draw_count, recover_count, times, skill]`.
    ChooseOtherDrawOrRecover = 87,
    /// Forced hand/equipment discard by a dynamic count, followed by removal of the emitting
    /// one-shot skill. payload: `[player, skill, required, option_count, option0, ...]`.
    ChooseDiscardHandEquipThenRemoveSelfSkill = 88,
    /// Zishu repeated larger-hand take state machine.
    /// Stage 0 chooses an optional larger-hand target: `[0, player, skill]`.
    /// Stage 1 chooses a mandatory card from that target: `[1, player, target, skill]`.
    ChooseTakeLargerHandUntilMax = 89,
    /// Three-way current-card branch: draw, increase base damage, or perform both and add a mark.
    /// payload: `[player, card, draw_count, damage_amount, mark_def, mark_count, skill]`.
    ChooseDrawOrIncreaseCurrentCardDamageOrBothMark = 90,
    /// Draw-complete branch between giving hand cards to another living player and discarding
    /// hand cards.
    ///
    /// payload stage 0: `[0, player, payment_count, skill]`;
    /// stage 1: `[1, player, payment_count, skill]`;
    /// stage 2: `[2, player, target, payment_count, skill]`;
    /// stage 3: `[3, player, payment_count, skill]`.
    DrawThenChooseGiveOrDiscardHand = 91,
    /// Choose any living target for a draw followed by an owner hand-vs-max-HP check.
    /// payload: `[player, count, skill]`.
    ChooseTargetDrawThenOwnerIfBelowMaxHp = 92,
    /// Choose one living player in a skill-bound source's attack range, then damage that player
    /// from the source. payload: `[player, source, amount, skill]`.
    ChooseBoundSourceAttackRangeDamage = 93,
    /// Choose a living target for a capped-max-HP draw followed by a post-draw excess discard.
    /// payload: `[player, cap, skill]`.
    ChooseTargetDrawMaxHpCapThenDiscardExcess = 94,
    /// Refanjian recipient chooses to reveal and discard every matching-suit hand/equipment card,
    /// or lose 1 HP. payload: `[target, suit_tag]`.
    GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp = 95,
    /// Choose a target able to self-discard one hand/equipment card, then let that target select
    /// the card. payload stage 0: `[0, player, skill]`; stage 1:
    /// `[1, player, target, skill]`.
    ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp = 96,
    /// A third-party giver chooses one current hand/equipment card for the target or declines,
    /// causing the target to recover one HP. payload:
    /// `[owner, giver, target, skill, option_count, options...]`.
    ChooseGiveOneHandEquipOrRecover = 97,
    /// Forced hand/equipment-zone branch for a full-zone discard, delayed draw, and HP loss.
    /// payload: `[player, skill, hand_available, equip_available]`.
    ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp = 98,
    /// Choose a bounded number of distinct living targets, then damage them in seat order.
    /// payload: `[player, min_count, max_count, scope, amount, skill]`.
    ChooseTargetsDamage = 99,
    /// Choose and discard target equipment; a weapon makes that target draw its attack range.
    /// payload: `[player, target, skill, option_count, option0, ...]`.
    DiscardTargetEquipmentDrawWeaponRange = 100,
    /// Choose a living target with hand cards, reveal that hand, and damage it if it contains Sha.
    /// payload: `[player, amount, skill]`.
    ChooseTargetRevealHandDamageIfSha = 101,
    /// Choose another living target before reusing the established one-card gift continuation.
    /// payload: `[player, skill]`.
    ChooseOtherGiveOneHandEquip = 102,
    /// After a limited skill's full recovery resolves, optionally sacrifice another owned skill
    /// to restore a fresh instance of the limited skill.
    /// payload: `[removable_skill_def, restore_skill_def]`.
    RecoverToMaxThenOptionalExchangeSkill = 103,
    /// OL-Qingxi-style bound direct-Sha damage choice.
    ///
    /// Stage 0 chooses discard or damage without a weapon. Stages 1/2 select the target's
    /// hand/equipment cards without/with a source weapon.
    /// payload starts with the stage tag.
    ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth = 104,
    /// Choose OL-Jiangchi's draw branch or a one-card recast branch.
    /// payload: `[stage, player, draw_skill_def, recast_skill_def, skill]`.
    ChooseDrawOrRecastAddSkill = 105,
    /// Confirm gaining the first Sha from draw/discard for one originating opening Play phase.
    /// payload: `[player, card, from_zone, skill, damage_total_baseline]`.
    ChooseGainShaForOpeningExtraPlay = 106,
    /// Choose the Moukui-style branch and record this skill/target on the exact resolving Sha.
    /// payload: `[player, card, target, skill]`.
    ChooseDrawOrDiscardTargetCardRecordSha = 107,
    /// After publicly revealing a hand containing both colors, that player chooses red or black
    /// and discards every discardable current hand card of the selected effective color.
    /// payload: `[player, offered_color_mask]`.
    RevealHandChooseDiscardColor = 108,
    /// Choose target hand/equipment cards; recover after movement triggers when the requested
    /// count was met and every discarded card had the same target-visible category.
    /// payload: `[player, target, count, amount, skill, option_count, option0, ...]`.
    ChooseDiscardTargetCardsRecoverSameCategory = 109,
    /// Target chooses the second comparison card for a repeated-failure seat-damage active.
    /// payload:
    /// `[player, target, player_card, draw_count, failure_count, lose_hp, damage, skill]`.
    ChooseCompareTargetHandCardSecondFailureSeatDamage = 110,
    /// Choose draw or discard after a strict comparison win.
    /// payload: `[player, target, draw_count, skill]`.
    ChooseDrawOrDiscardTargetCard = 111,
    /// Choose one of two equal-length seat paths.
    /// payload: `[player, amount, skill, first_path_count, first_path..., second_path...]`.
    ChooseSeatPathDamage = 112,
    /// Standard Fanjian two-stage target choice.
    /// payload stage 0: `[0, player, target, skill]`;
    /// stage 1: `[1, player, target, skill, suit_tag]`.
    FanjianChooseSuitOrHand = 113,
    /// Standard Ganglie source discard-or-damage choice.
    /// payload: `[player, source]`.
    GanglieSourceDiscardOrDamage = 114,
    /// Standard Tuxi target/card selection.
    /// Stage 0 payload: `[0, player, skill, option_count, targets...]`.
    /// Stage 1 payload:
    /// `[1, player, skill, index, target_count, targets..., option_count, cards...]`.
    TuxiGainOtherHandsReplaceDraw = 115,
    /// Standard Yiji subset/recipient choice.
    /// Stage 0 payload: `[0, player, skill, remaining_count, cards...]`.
    /// Stage 1 payload:
    /// `[1, player, skill, remaining_count, remaining..., selected_count, selected...]`.
    YijiDistributeTopCards = 116,
    /// Standard Xiaoguo two-stage discard choice.
    /// Stage 0 payload: `[0, player, target, skill]`.
    /// Stage 1 is engine-only after cost triggers: `[1, player, target, skill]`.
    /// Stage 2 payload: `[2, player, target, skill]`.
    XiaoguoDiscardBasicThenEquipOrDamage = 117,
    /// Standard Tongji current-target discard and Sha redirection.
    /// payload: `[owner, source, card, target, skill]`.
    TongjiDiscardRedirectSha = 118,
    /// Standard Jijie recipient choice for the revealed bottom card.
    /// payload: `[player, card, skill]`.
    JijieChooseBottomCardRecipient = 119,
    /// Standard Luoshen optional repeat after a black judgment card is gained.
    /// payload: `[player, skill]`.
    LuoshenRepeatBlackJudgment = 120,
    /// Another group member may provide a response card for the skill owner.
    /// payload: `[owner, skill, kind, action, provider_index, provider_count, providers..., original...]`.
    DelegatedGroupResponse = 121,
    /// Standard Liuli discard cost plus replacement-target selection.
    /// Stage 0: `[0, player, source, card, skill]`.
    /// Stage 1: `[1, player, source, card, skill, cost_card]`.
    LiuliDiscardRedirectSha = 122,
    /// Target self-discard followed by a conditional virtual Duel.
    /// Choice payload: `[player, target, skill]`.
    /// Resume payload: `[player, target, skill, discarded_was_sha]`.
    TargetDiscardThenDuelIfNonShaAndHpGteOwner = 123,
    /// SIXIANG 豹变: choose target, then that target discards after owner loses HP.
    /// Choice payload stage 0: `[0, player, skill]`.
    /// Choice payload stage 1: `[1, player, target, skill]`.
    /// Resume payload: `[player, target, skill, discarded_basic]`.
    ChooseTargetLoseHpDiscardHandShaIfBasic = 124,
    /// Confirm the optional HP-loss/card-gain branch of an originating opening extra Play phase.
    /// payload: `[player, card_kind, mandatory_skill_def, skill]`.
    LoseHpGainDiscardCardForOpeningExtraPlay = 125,
    /// Choose one card the player lost from hand/equipment this turn that remains in discard.
    /// payload: `[player, skill]`.
    ChooseGainOwnLostDiscardCardThisTurn = 126,
    /// Choose one field card and move it to a fixed living destination.
    /// payload: `[player, target, skill]`.
    ChooseMoveFieldCardTo = 127,
    /// New Yiji hand-distribution state machine.
    ///
    /// Stage 0 chooses an optional nonempty hand-card subset; stage 1 chooses its mandatory
    /// other-player recipient. Both stages preserve positional card/target plans in the payload.
    DrawThenOptionallyDistributeHandCards = 128,
    /// DIY old Rende optional card/recipient continuation.
    ///
    /// Stage 0 chooses a nonempty hand-card batch; stage 1 chooses one previously unused other
    /// living recipient for that batch.
    GiveSelectedThenContinueDistinctRecoverAtTwo = 129,
    /// New Wuhun: select one highest-marked living other player, then judge and conditionally kill.
    /// payload: `[player, mark_def, skill]`.
    ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan = 130,
    /// New Guixin: choose one H/E/J priority before the engine-owned sequential gain state machine.
    /// payload: `[player, skill]`.
    ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver = 131,
    /// Forced hand discard inside the ordered all-other damage/discard state machine.
    /// payload: `[player, required, option_count, options...]`, or with `1` before options for
    /// combined hand/equipment scope.
    DiscardHandForMassDamage = 132,
    /// Mark-parity control chooses any living target.
    /// payload: `[player, mark, mass_threshold, damage_amount, skill]`.
    ChooseMarkParityControlTarget = 133,
    /// Mark-parity control discards one chosen target H/E/J card.
    /// payload: `[player, target, mark, mass_threshold, damage_amount, skill]`.
    DiscardMarkParityControlTargetCard = 134,
    /// Mark-parity control confirms clearing all marks for mass damage.
    /// payload: `[player, mark, mass_threshold, damage_amount, skill]`.
    ConfirmMarkParityMassDamage = 135,
    /// Choose one surviving selected target for final sourced damage.
    /// payload: `[player, damage_amount, nature, skill, target_count, targets...]`.
    ChooseSelectedTargetDamage = 136,
    /// Choose exactly one revealed processing-zone card for each distinct suit.
    /// payload: `[player, skill, card_count, cards...]`.
    ChooseDistinctSuitCards = 137,
    /// Choose one heart card from the inspected target hand.
    /// payload: `[player, target, skill]`.
    ChooseInspectedHeartCard = 138,
    /// Choose whether the inspected heart card is discarded or put on the draw-pile top.
    /// payload: `[player, target, card, skill]`.
    ChooseInspectedHeartDestination = 139,
    /// Choose small or four-suit big Yeyan.
    /// payload: `[player, skill]`.
    ChooseYeyanMode = 140,
    /// Choose exactly four current hand cards of distinct effective suits.
    /// payload: `[player, skill, option_count, options...]`.
    ChooseYeyanBigCards = 141,
    /// Choose Yeyan targets.
    /// payload: `[player, mode, skill]`.
    ChooseYeyanTargets = 142,
    /// Choose two or three fire damage for one big-Yeyan target.
    /// payload: `[player, skill, target]`.
    ChooseYeyanSingleDamage = 143,
    /// Choose which of two big-Yeyan targets takes two fire damage.
    /// payload: `[player, skill, target0, target1]`.
    ChooseYeyanDoubleTarget = 144,
    /// Choose Qinyin lose-HP, recover, or cancel.
    /// payload: `[player, skill]`.
    ChooseAllLoseHpOrRecover = 145,
    /// Choose physical sources for a multi-card response conversion.
    /// payload: `[player, skill, kind, action, original...]`.
    MultiCardViewAsResponse = 146,
    /// Choose hand cards to exchange with the current skill's Qixing pile.
    /// payload: `[player, skill]`.
    ChooseQixingHandCards = 147,
    /// Choose equal-count Qixing pile cards to exchange with saved hand cards.
    /// payload: `[player, skill, hand_count, hand_cards...]`.
    ChooseQixingPileCards = 148,
    /// Choose Qixing pile cards to spend on a weather helper.
    /// payload: `[player, skill, qixing_def, helper_def, max_targets]`.
    ChooseQixingWeatherCards = 149,
    /// Choose equal-count living weather targets.
    /// payload: `[player, skill, qixing_def, helper_def, max_targets, card_count, cards...]`.
    ChooseQixingWeatherTargets = 150,
    /// Choose another living unlinked player to become linked.
    /// payload: `[player, skill]`.
    ChooseLinkOther = 151,
    /// Choose four distinct-suit cards across the Poxi owner and inspected target hands.
    /// payload: `[player, target, helper_skill_def, skill, len, cards...]`.
    ChooseDrltPoxiCards = 152,
    /// Choose another living unmarked player to receive the Jieying mark.
    /// payload: `[player, mark, skill]`.
    ChooseDrltJieyingTransfer = 153,
    /// Choose one enabled equipment slot to disable for Duorui.
    /// payload: `[player, target, helper_skill_def, skill]`.
    ChooseDrltDuoruiDisableSlot = 154,
    /// Choose one eligible target skill definition to borrow for Duorui.
    /// payload: `[player, target, helper_skill_def, skill]`.
    ChooseDrltDuoruiSkill = 155,
    /// Choose one disabled equipment slot to restore for Zhiti.
    /// payload: `[player, skill]`.
    ChooseDrltZhitiEnableSlot = 156,
    /// Choose whether to discard Jilue Jizhi's just-drawn basic card.
    /// payload: `[player, card, skill]`.
    ChooseJilueJizhiDiscard = 157,
    /// Choose one optional card to rewrite a Beige judgment.
    /// payload: `[owner, target, source, result_card, suit, rank, skill, rewriter_len, ...]`.
    RewriteBeigeJudgment = 158,
    /// Choose the lose-HP or discard-weapon cost, then optionally the weapon.
    /// payload: `[player, skill, stage]`.
    ChooseDiscardWeaponOrLoseHp = 159,
    /// Choose one current target hand/equipment card from a zone not yet used this Play phase.
    /// payload: `[player, target, skill, option_count, options...]`.
    ChooseGainTargetCardAlternateZoneThenTargetDraw = 160,
    /// Recipient may use the exact equipment card just given to them.
    /// payload: `[recipient, card]`.
    UseGivenEquipment = 161,
    /// Choose one hand/equipment card from either endpoint of bound Sha damage.
    /// payload: `[player, target, damage_id, suit, amount, skill, option_count, options...]`.
    ChooseDiscardDamageEndpointCardIncreaseIfSuit = 162,
    /// Choose an extra Draw or Play phase after the current End phase.
    /// payload: `[player, skill]`.
    ChooseEndExtraDrawOrPlay = 163,
    /// Choose one current H/E/J card from the next Guixin target.
    /// payload: `[player, target, skill, option_count, options...]`.
    ChooseGainCardFromEachOtherThenTurnOver = 164,
    /// Choose one or more hand cards for a hidden skill-pile exchange.
    /// payload: `[player, pile_skill_def, action, card, skill]`.
    ChooseHiddenSkillPileHandCards = 165,
    /// Choose hidden skill-pile cards for reveal or exchange.
    /// reveal payload: `[player, pile_skill_def, action, card, skill]`;
    /// exchange payload appends `[count, hand_cards...]`.
    ChooseHiddenSkillPileCards = 166,
    /// Choose a living player with an equipment/judgment card for 长安大舰.
    /// payload: `[player, remaining, skill]`.
    ChooseChangandajianTarget = 167,
    /// Choose one equipment/judgment card from the selected 长安大舰 target.
    /// payload: `[player, target, remaining, skill, option_count, options...]`.
    ChooseChangandajianCard = 168,
    /// Higher-hand target discards one H/E card for 调剂盐梅.
    /// payload: `[actor, left, right, higher]`.
    DiscardTiaojiyanmeiCard = 169,
    /// Card user optionally chooses who gains the discarded 调剂盐梅 card.
    /// payload: `[actor, card]`.
    ChooseTiaojiyanmeiGainTarget = 170,
    /// Choose 国战英魂's target, branch, or forced discard cards.
    ///
    /// payload begins with stage 0, 1, or 2 followed by the captured owner/target/count/skill.
    ChooseOtherYinghun = 171,
    /// Choose 弼政's draw target or one participant's forced discard cards.
    ChooseOtherBizheng = 172,
    /// Choose 霞姝's recipient, revealed hand subset, or gained partition.
    ChooseOtherXiashu = 173,
    /// Choose 观微's discarded card.
    ResolveGuanwei = 174,
    /// Resolve 揽江's per-target confirmations and target choices.
    ResolveLanjiang = 175,
    /// Choose 秉壹's draw recipients after the full-hand reveal.
    ResolveRebingyi = 176,
    /// Choose 旧缔盟's two targets or forced discard cards.
    ResolveOldimeng = 177,
    /// Choose 界结姻's male target or equipment/discard branch.
    ResolveRejieyin = 178,
    /// Choose 仁恤's targets, transferred card, or optional draw.
    ResolveReanxu = 179,
    /// Choose 决堰's equipment-slot group.
    ResolveDrltJueyan = 180,
    /// Confirm 界集智's basic-card discard.
    ResolveRejizhi = 181,
    /// Choose 诫训's target or forced discard cards.
    ResolveJiexun = 182,
    /// Choose 问卦's recipient or top/bottom branch.
    ResolveWengua = 183,
    /// Choose 国战三版刚烈's enemy target.
    ResolveGanglieThree = 184,
    /// Choose old 安恤's giver, receiver, and transferred hand card.
    ResolveOldAnxu = 185,
    /// Choose old 寝情's target.
    ResolveOldQinqing = 186,
    /// Choose old 明鉴's recipient.
    ResolveOldMingjian = 187,
    /// Resolve old 旋风's target and branch.
    ResolveOldXuanfeng = 188,
    /// Choose 剑柔's two discarded cards.
    ResolveJianrou = 189,
    /// Resolve old 贿生's owner/source choices.
    ResolveOldHuisheng = 190,
    /// Choose old 解烦's physical or view-as Sha source.
    ResolveOldJiefan = 191,
    /// Resolve 彻骨's branch, targets, and ordered discarded cards.
    ResolveChegu = 192,
    /// Resolve 归心's branch, lord skill, target, and faction choices.
    ResolveJunkguixin = 193,
    /// Resolve 蛊惑's hidden source and sequential challenge choices.
    ResolveGuhuo = 194,
    /// Choose a tagged face-down hand card for a dynamic view-as conversion.
    ResolveFaceDownViewAs = 195,
    /// Resolve 谶纬's alternating target/card choices.
    ResolveYaoChenwei = 196,
    /// Resolve 游敌's target, forced owner-card discard, and conditional counter-gain.
    ResolveYoudi = 197,
    /// Resolve 狂斧's equipment choice and move/discard branch.
    ResolveKuangfu = 198,
    /// Resolve 缮甲's forced discard after drawing.
    ResolveShanjia = 199,
    /// Resolve 新涯角's recipient and mismatch discard choices.
    ResolveNewYajiao = 200,
    /// Choose one public current-skill pile card to discard.
    ChooseDiscardSkillPileCardAddMarkedSkill = 201,
    /// Choose one current hand card from each submitted active target.
    GainOneHandFromSelectedTargets = 202,
    /// Choose 破虏 draw recipients; the chooser may already be dead.
    ChooseRepoluTargetsDraw = 203,
    /// Resolve 过论's target-hand and optional owner-card choices.
    ResolveXinfuGuolun = 204,
    /// Let a dead 追忆 owner optionally choose one living non-killer recipient.
    /// payload: `[owner, killer, skill]`.
    ResolveZhuiyi = 205,
    /// Resolve SP 骁果's owner hand-card cost and target equipment choice.
    /// payload: `[stage, owner, target, skill]`.
    ResolveSpXiaoguo = 206,
    /// Let 定叛's selected target choose its equipment branch.
    /// payload: `[owner, target, skill]`.
    ChooseDingpanBranch = 207,
    /// Resolve 贪赂's target-owned branch/card choices.
    /// payload: `[stage, owner, target, frozen_hp_gap, skill]`.
    ResolveTanlu = 208,
    /// OFFLINE 洛神 optional repeat after a same-color judgment card is gained.
    /// payload: `[player, skill, first_color]`.
    JudgeSameColorGainRepeat = 209,
    /// Resolve 趫猛's target and target-card choices.
    /// payload: `[stage, player, card, target, helper_skill_def, skill, target_count, targets...]`.
    ResolveDcqiaomeng = 210,
    /// Resolve 界铁骑's same-suit gift or current-Sha penalty.
    /// payload: `[source, card, target, suit_tag]`.
    ResolveRetieji = 211,
    /// Resolve 界许褚「裸衣」's replace-draw confirmation.
    /// payload: `[player, buff_skill_def, skill, card_count, cards...]`.
    ResolveNewReluoyi = 212,
    /// 界邀名 target, action, and target-discard choice stages.
    ResolveXinyaoming = 213,
    /// 界怀异 color, target, and per-target card choices.
    ResolveRehuaiyi = 214,
    /// 界安国 optional arbitrary recast choice.
    ResolveReanguo = 215,
    /// 备诛 optional draw-pile Sha grant.
    ResolveBeizhu = 216,
    /// mobile 缮甲 forced discard after drawing.
    ResolveMbshanjia = 217,
    /// mobile 盗戟 target-equipment choice.
    ResolveDaoji = 218,
    /// mobile 教化 category and recipient choices.
    ResolveJiaohua = 219,
    /// mobile 乱群 private hand selections and optional matching-card gain.
    ResolveLuanqun = 220,
    /// A third-party giver selects exactly two differently colored hand/equipment cards.
    ChooseGiveTwoDifferentColorsOrRecover = 221,
    /// mobile 助国 discard, target, physical-Sha, and Sha-target choices.
    ResolveMbzhuguo = 222,
    /// mobile 暗弦 discarded-Sha and target choices.
    ResolveMbanxian = 223,
    /// mobile 劲镞 target hand-card reveal choice.
    ResolveMbjinzu = 224,
    /// mobile 试探 cross-hand distinct-suit multi-discard continuation.
    ChooseScskuijiDiscard = 225,
    /// offline 破试探 cross-hand distinct-suit multi-discard with asymmetric HP-loss continuation.
    ChoosePskuijiDiscard = 226,
    /// 峻攻：弃置至多 N 张牌或失去 N 点体力，然后使用虚拟【杀】。
    /// payload: `[player, skill, target, n]`。
    ChooseSpjungongDiscardOrLose = 227,
    /// Give-all-hand recipient chooses one exact gifted card and its legal targets.
    /// payload: `[stage, player, target, skill, card, card_count, cards...]`.
    GiveAllHandThenChooseUseOneOrDamage = 228,
    /// Source chooses a card category; target gives one matching card or cannot respond to Sha.
    /// payload: `[stage, source, sha, target, skill, category?]`.
    ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable = 229,
    /// 嬉战：弃置 0/1 张牌或失去体力，再按花色视为用牌。
    /// payload: `[player, phase_player, fangzong_skill_def]`.
    ChooseSpxizhanDiscardOrLose = 230,
    /// 复叛的选牌、目标与可选伤害阶段。
    /// payload: `[stage, player, damage_amount, card, target, skill]`.
    ResolveTwfupan = 231,
    /// 送枢：摸牌角色从仁库技能牌堆选牌获得。
    /// payload: `[player, phase_player, renku_skill_def, block_skill_def]`.
    ChooseSpsongshuCards = 232,
    /// 睿敛的每轮目标及回合结束类别选择。
    /// payload: `[stage, player, target, skill, category_mask]`.
    ResolveTwruilian = 233,
    /// 剑威的发起目标、拼点牌和按区域获牌选择。
    ResolveTwjianwei = 234,
    /// 探锋的目标、弃牌、转化【杀】与阶段选择。
    ResolveTwtanfeng = 235,
    /// 直辩的拼点、选项、背水与区域移动。
    ResolveTwzhibian = 236,
    /// 御严的交牌或取消当前目标选择。
    ResolveTwyuyan = 237,
    /// 果毅的二选一与弃牌续体。
    ResolveTwguoyi = 238,
    /// 除害使命收牌或新获牌弃置。
    ResolveTwchuhai = 239,
    /// 遗珠的埋置牌选择与当前用牌目标修改。
    ResolveTwyizhu = 240,
    /// 狭威的妄行选项与回合结束代价。
    ResolveTwxiawei = 241,
    /// 挽危/约俭的选牌与牌堆分区。
    ResolveTwbianfuren = 242,
    /// 生息智囊选择与宽济选牌/目标。
    ResolveTwfeiyi = 243,
    /// 睦约牌名/弃牌/目标与察异分支选择。
    ResolveTwchenzhen = 244,
    /// 前驱乐/倾袭的目标选择与装备弃置串联。
    ResolveTwqingxi = 245,
    /// 躁厉的非装备手牌选择。
    ResolveTwzaoli = 246,
    /// 征建的开局、交牌与切换选择。
    ResolveTwzhengjian = 247,
    /// 震袭的分支、弃牌、场上牌和目标选择。
    ResolveTwtianyu = 248,
    /// Source-free conversion cost: choose a lower-hand player and swap whole hands.
    ResolveSwapLowerHandViewAs = 249,
    /// 从击的事件红牌选择与接收者选择。
    ResolveTwwujing = 250,
    /// 商和逐人交牌。
    ResolveTwshanghe = 251,
    /// 王昶开济/慑叛选择。
    ResolveTwwangchang = 252,
    /// Continue TW 曹肇 conversion target/skill, optional grant, gift, or equipment use.
    ResolveTwcaozhao = 253,
    /// Continue TW 郭淮 branch or discard choice.
    ResolveTwguohuai = 254,
    /// Content / VM defined.
    Custom = 255,
    /// 神君 phase-end use: stage 0 selects a gaintagged hand card, stage 1 a
    /// target, then the card is played as its own kind.
    /// payload: `[stage, player, skill]` (stage 1 appends the selected card).
    ClanshenjunUseCard = 256,
    /// 迂志 round-start: stage 0 selects one hand card to reveal; the resolver then
    /// reveals + gaintags it, draws its name length (tagging the draws) and records
    /// the round draw counts in the skill's keyed storage.
    /// payload: `[stage, player, skill]`.
    ClanyuzhiRevealChoice = 257,
    /// 迂志 round-end: stage 0 is a yes/no over losing 保族 (selection 0) or taking
    /// 1 thunder damage (selection 1).
    /// payload: `[stage, player, skill]`.
    ClanyuzhiEndChoice = 258,
    /// choose_target_draw: the chooser selected one other player to draw `count` cards.
    /// payload: `[chooser, count]`.
    ChooseTargetDrawChoice = 259,
    /// 惬诣 non-matching reveal: stage 0 selects one hand/equip card; the resolver
    /// puts it on top of the draw pile.
    /// payload: `[stage, player, skill]`.
    QieyiPutTopChoice = 260,
    /// 谏直 phase-end: stage 0 selects the number of judgments (1..惬诣 fires this
    /// turn); the resolver runs the judgments, then opens the assign-card loop.
    /// payload: `[stage, player, skill]`.
    JianzhiCountChoice = 261,
    /// 谏直 assignment loop: stage 0 selects one matching-suit judge card, stage 1 a
    /// recipient; the card is given, the loop re-opens while cards remain, then any
    /// non-matching judge card deals 1 thunder damage.
    /// payload: `[stage, player, skill, has_non_matching, cards...]`.
    JianzhiAssignCard = 262,
    /// Choose to discard one hand/equipment card, or lose the requested HP.
    /// payload: `[player, skill, lose_amount]`.
    /// Appended to keep existing serialized enum ordinals stable. ABI 601.
    ChooseDiscardOneHeOrLoseHp = 263,
    /// The death source confirms whether the owner loses the requested HP.
    /// payload: `[owner, source, skill, amount]`.
    /// Appended to keep existing serialized enum ordinals stable. ABI 603.
    ChooseSourceConfirmOwnerLoseHp = 264,
    /// 临界 (sp/ollinjie): the damage source discards one hand card; the owner draws one if
    /// the discarded card is effectively a Sha.
    /// payload: `[player, source, skill]`.
    /// Appended to keep existing serialized enum ordinals stable. ABI 608.
    ResolveOllinjie = 265,
    /// 寿春 (sp/pxshouchun): the owner selects any living player; that player gains
    /// 1 max hp. payload: `[player, skill]`.
    /// Appended to keep existing serialized enum ordinals stable. ABI 646.
    ChooseTargetGainMaxHp = 266,
    /// 肃州 (sp/pxsizhou): the owner selects another player, then gains one of
    /// their hand cards. payload: stage 0 `[0, player, skill]`; stage 1
    /// `[1, player, target, skill]`.
    /// Appended to keep existing serialized enum ordinals stable. ABI 647.
    GainTargetHandCard = 267,
    /// 穆荫 (clan/clanmuyin): owner selects a same-clan-or-self player whose hand
    /// limit is not the field max; that player gains +1 hand limit (persistent).
    /// payload: `[player, skill]`.
    /// Appended to keep existing serialized enum ordinals stable. ABI 648.
    ChooseClanNotMaxHandLimitGain = 268,
    /// 置入 (expansion P2, ADR 0641): owner selects one hand card to place onto
    /// their skill pile. payload: `[player, skill]`. ABI 650.
    PlaceHandCardToExpansion = 269,
    /// 取用 (expansion P1, ADR 0641): owner selects one card from their own
    /// skill pile to gain into hand. payload: `[player, skill]`. ABI 651.
    GainFromSkillPile = 270,
    /// ABI 652 choose_other_add_mark: chooser picks one other player for a mark adjust.
    ChooseOtherAddMark = 271,
    /// ABI 656 start_debate_targets: chooser picks debate participants.
    StartDebateTargets = 272,
    /// ABI 657 discard_basic_draw_equal: chooser picks basic cards to swap for draws.
    DiscardBasicDrawEqual = 273,
    /// ABI 658 pin_debate_opinions: chooser picks two hand cards as pinned
    /// opinions; other participants are pinned to random hand cards.
    PinDebateOpinions = 274,
    /// ABI 662 distribute_shadow_cards: chooser picks 1..N players to receive
    /// the total shadow-card count split evenly.
    DistributeShadowCards = 275,
    /// ABI 663 register_turn_end_debate: chooser picks turn-end debate
    /// participants (yaoyan).
    RegisterTurnEndDebate = 276,
    /// ABI 664 choose_group: chooser picks a faction to change to (chijie).
    ChooseGroup = 277,
    /// ABI 675 choose_gain_other_skill: pick another player, then one of their
    /// gainable skills to add (§7 技能夺取设施 S0).
    ChooseGainOtherSkill = 283,
    /// ABI 676 lose_gained_skill: pick one facility-gained skill to remove
    /// (§7 技能夺取设施 S1).
    LoseGainedSkill = 284,
    /// ABI 677 resolve_potheyun: pick a player sharing a skill, lose it, they
    /// draw 2 (§7 技能夺取设施 S2·合韵延伸).
    ResolvePotheyun = 285,
    /// ABI 680 choose_hand_compare_between stage 1: `first` picks the opponent.
    /// payload: `[player, first, skill]`.
    BetweenOpponent = 286,
    /// ABI 680 choose_hand_compare_between stage 2: `first` picks their own card.
    /// payload: `[player, first, second, skill]`.
    BetweenFirstCard = 287,
    /// ABI 680 choose_hand_compare_between stage 3: `second` picks their own card,
    /// then commit with winner-vs-loser virtual Sha on non-tie.
    /// payload: `[player, first, second, first_card, skill]`.
    BetweenSecondCard = 288,
    /// ABI 681 guanxing_top: one round of the top/bottom pick loop.
    /// payload: `[player, count, round, flipped_len, flipped..., top_len, top...,
    /// bottom_len, bottom..., skill]`.
    GuanxingPick = 289,
    /// ABI 682 choose_other_give_then_recover: pick the giver.
    /// payload: `[player, skill]`.
    ChooseOtherGiveThenRecover = 290,
    /// ABI 686 choose_one_from_skill_pile_discard: pick one card from the own
    /// (player, pile_def) skill pile and discard it (expansion S1-B, ADR 0641
    /// §8.7 paiyi family).
    /// payload: `[player, skill]`.
    ChooseOneFromSkillPileDiscard = 291,
    /// ABI 688 choose_target_hand_card_to_pile: pick another player with hand
    /// cards, then one of their hand cards, into the (player, pile_def) skill
    /// pile (expansion A1, ADR 0641 §8.7).
    /// payload: `[stage, player, skill, pile_def]` (two-stage).
    ChooseTargetHandCardToPile = 294,
    /// 批 235：resolve_dcbingji 专属状态机各阶段续体（Custom 载荷分派）。
    /// payload: `[magic, player, skill, stage, kind]`。
    ResolveDcbingjiStage = 300,
    /// 批 236：resolve_pxqixian 专属状态机各阶段续体。
    /// payload: `[magic, player, skill, stage, kind, material, target]`。
    ResolvePxqixianStage = 301,
    /// 批 240：resolve_jsrgdaoren 专属状态机各阶段续体。
    /// payload: `[magic, player, skill, stage, card, target]`。
    ResolveJsrgdaorenStage = 302,
    /// 批 241：resolve_qianlong 专属状态机续体。
    /// payload: `[magic, player, skill, c1, c2, c3]`。
    ResolveQianlongStage = 303,
    /// 批 242：resolve_jsrgzuozhan 专属状态机续体。
    /// payload: `[magic, player, skill]`。
    ResolveJsrgzuozhanStage = 304,
    /// 批 243：resolve_jsrgzuozhan_gain 组技补牌状态机续体。
    /// payload: `[magic, player, skill, event_player]`。
    ResolveJsrgzuozhanGainStage = 305,
    /// 批 247：resolve_twdaigui 待归多段状态机续体（slice 1 仅 stage 2 使用，
    /// slice 2 将扩展逐目标选获阶段）。
    /// payload: `[magic, player, skill, count]`。
    ResolveTwdaiguiStage = 306,
    /// 批 249：resolve_twmitu 迷途多段拼点链状态机续体（slice 1 仅 stage 2
    /// 使用；slice 2 将扩展拼点轮次）。
    /// payload: `[magic, player, skill, stage]`。
    ResolveTwmituStage = 307,
    /// 批 252：resolve_qiantun 谦吞拼点族状态机续体。
    /// payload: `[magic, player, skill, stage, target, k, shown(k), ...]`。
    ResolveQiantunStage = 308,
    /// 批 253：resolve_yangming 养名循环拼点状态机续体。
    /// payload: `[magic, player, skill, stage, target, losses, pend]`。
    ResolveYangmingStage = 309,
    /// 批 255：resolve_canyun 残殒装备比较状态机续体。
    /// payload: `[magic, player, skill, stage, discard_card, t2]`。
    ResolveCanyunStage = 310,
    /// 批 257：resolve_biaozhao3 表召·归状态机续体。
    /// payload: `[magic, player, skill, stage, num]`。
    ResolveBiaozhao3Stage = 311,
    /// 批 261：resolve_dcmoukui 谋溃状态机续体。
    /// payload: `[magic, player, skill, stage, target]`。
    ResolveDcmoukuiStage = 312,
    /// 批 262：resolve_zhongyan 忠言状态机续体。
    /// payload: `[magic, player, skill, stage, target, k, revealed(k)]`。
    ResolveZhongyanStage = 313,
    /// 批 263：resolve_chumei 狐魅状态机续体。
    /// payload: `[magic, player, skill, stage, target]`。
    ResolveChumeiStage = 314,
    /// 批 266：resolve_shuaiyan 率言状态机续体。
    /// payload: `[magic, player, skill, stage]`。
    ResolveShuaiyanStage = 315,
    /// 批 267：resolve_zhengjian 诤荐状态机续体。
    /// payload: `[magic, player, skill, stage]`。
    ResolveZhengjianStage = 316,
    /// 批 269：resolve_mbbeiming 孛明状态机续体。
    /// payload: `[magic, player, skill, stage]`。
    ResolveMbbeimingStage = 318,
    /// 批 271：resolve_gaoyuan 告援状态机续体。
    /// payload: `[magic, player, source, card, stage]`。
    ResolveGaoyuanStage = 319,
    /// 批 272：resolve_duoduan 度断状态机续体。
    /// payload: `[magic, player, source, card, stage]`（stage 3 追加被弃牌 id）。
    ResolveDuoduanStage = 320,
    /// 批 273：resolve_dcanxu 暗蓄状态机续体。
    /// payload: `[magic, owner, gainner, giver, skill, stage]`。
    ResolveDcanxuStage = 321,
    /// 批 274：resolve_chulao 黜老状态机续体。
    /// payload: `[magic, stage, owner, cost, idx, t0, t1, …]`。
    ResolveChulaoStage = 322,
    /// 批 276：resolve_mbzhenann 镇南状态机续体。
    /// payload: `[magic, stage, owner]` / `[magic, stage, owner, card]`。
    ResolveMbzhenannStage = 323,
    /// 批 277：resolve_meiyong 姝勇状态机续体。
    /// payload: `[magic, stage, owner]` / `[magic, stage, owner, target]`。
    ResolveMeiyongStage = 324,
    /// 批 278：resolve_qiangyong 羌勇状态机续体。
    /// payload: `[magic, stage, owner, target, x]`。
    ResolveQiangyongStage = 325,
    /// 批 281：resolve_dingce 定策状态机续体。
    /// payload: `[magic, stage, owner, source]`。
    ResolveDingceStage = 326,

    /// 批 286：resolve_dcqiaolve 巧略状态机续体。
    /// payload: `[magic, stage, owner]`。
    ResolveDcqiaolveStage = 327,
    /// 批 291：resolve_xinfu_qiai 求援状态机续体。
    /// payload: `[magic, stage, owner, giver]`。
    ResolveXinfuQiaiStage = 328,
    /// 批 294：resolve_olzhijie 直谏状态机续体。
    /// payload: `[magic, 1, owner]`。
    ResolveOlzhijieStage = 329,
    /// 批 296：resolve_olqiaozhi 巧织状态机续体。
    /// payload: `[magic, stage, owner(, a, b)]`。
    ResolveOlqiaozhiStage = 330,
    /// 批 304：resolve_zhongjie 张介状态机续体。
    /// payload: `[magic, 1, owner]`。
    ResolveZhongjieStage = 332,
    /// 批 308：resolve_nhlundao 论道状态机续体。
    /// payload: `[magic, 1, owner, source]`。
    ResolveNhlundaoStage = 333,
    /// 批 268：resolve_reliangyin 良姻状态机续体。
    /// payload: `[magic, player, skill, mode]`。
    ResolveReliangyinStage = 317,
    /// ABI 691 choose_hand_cards_to_skill_pile: select [min,max] hand cards,
    /// place into the own originating skill pile (cancel = place none).
    /// payload: `[player, skill, min_count, max_count]`.
    ChooseHandCardsToSkillPile = 298,
    /// 批 215：store_selected_in_skill_pile 死腿修复——无前置选择时自开
    /// SelectCards 1/1（从手牌，必选），续体落堆。payload: `[player, skill]`。
    StoreSelectedInSkillPile = 297,
    /// ABI 690 rearrange_pile_to_deck_top: select count cards from the own
    /// (player, pile_def) skill pile and place them on top of the draw pile
    /// (chooseToMove family, ADR 0641 §8.8).
    /// payload: `[player, skill, pile_def, count]`.
    RearrangePileToDeckTop = 296,
    /// ABI 689 place_hand_cards_to_deck_top: select `count` hand cards, place
    /// them on top of the draw pile (chooseToMove family, ADR 0641 §8.8).
    /// payload: `[player, skill, count]`.
    PlaceHandCardsToDeckTop = 295,
    /// ABI 666 choose_place_hand_xumou: chooser picks one hand card and moves
    /// it into their judge zone as a xumou (scheme) card.
    /// payload: `[player, skill]`.
    ChoosePlaceHandXumou = 278,
    /// ABI 668 city_consume: chooser picks a target that lost cards this turn;
    /// one City card is removed and the target draws min(5, lost count).
    /// payload: `[player, skill]`.
    CityConsume = 279,
    /// ABI 670 choose_others_add_mark: chooser picks 1..N other living players
    /// who each gain `delta` stacks of a mark.
    /// payload: `[player, mark, delta]`.
    ChooseOthersAddMark = 280,
    /// ABI 671 choose_others_insert_extra_turn: chooser picks one other living
    /// player whose extra turn is enqueued.
    /// payload: `[player, check, skill]`.
    ChooseOthersInsertExtraTurn = 281,
    /// ABI 673 resolve_twjieqiu sub 0: chooser picks one other living player
    /// whose equipment slots will all be disabled (k = equipment count drawn).
    /// payload: `[player, sub, skill]`.
    TwjieqiuSelect = 282,
}

/// In-flight resolution / choice barrier (canonical).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionState {
    /// Pending choice id (0 = none).
    pub pending_choice_id: u32,
    /// Monotonic allocator for choice ids (never reused silently).
    pub next_choice_seq: u32,
    /// Monotonic allocator for pre-resolution damage ids.
    #[serde(default)]
    pub next_damage_seq: u32,
    /// Damage id bound while a `damage_before` trigger queue is running.
    #[serde(default)]
    pub current_damage_id: u32,
    /// 旧不屈（张绣）标记：本次体力流失因「创」牌点数均不同而免于进入濒死。
    #[serde(default)]
    pub gzbuqu_nodying: bool,
    /// True only while planning a damage segment already redirected by a replacement effect.
    #[serde(default)]
    pub damage_redirect_in_progress: bool,
    /// Damage ids prevented by content before their commit requests execute.
    #[serde(default)]
    pub prevented_damage_ids: Vec<u32>,
    /// Damage-bound deltas queued by `damage_before` trigger effects that ran
    /// before the prepared CommitDamage was deferred (批 282): applied when the
    /// commit enters `deferred_tail_effects`.
    #[serde(default)]
    pub pending_bound_damage_deltas: Vec<(u32, i32)>,
    /// Monotonic allocator for pre-resolution recovery ids. ABI 635.
    #[serde(default)]
    pub next_recovery_seq: u32,
    /// Recovery id bound while a `recover_before` queue is running.
    #[serde(default)]
    pub current_recovery_id: u32,
    /// Recovery ids cancelled by replacement effects before commit.
    #[serde(default)]
    pub prevented_recovery_ids: Vec<u32>,
    /// First participant of the hand comparison whose start triggers are currently resolving.
    #[serde(default)]
    pub current_hand_compare_player: u32,
    /// Second participant of the hand comparison whose start triggers are currently resolving.
    #[serde(default)]
    pub current_hand_compare_target: u32,
    /// Bit 0 randomizes the first participant's card; bit 1 randomizes the second participant.
    #[serde(default)]
    pub current_hand_compare_random_mask: u8,
    /// Continuation id for VM / effect scheduler.
    pub continuation_id: u64,
    /// Open choice observer (0 = none).
    pub pending_observer: u32,
    /// Revision at which the pending choice was published (post-commit stamped).
    pub choice_revision: u64,
    /// Continuation kind for ResolveChoice resume.
    pub continuation_kind: ContinuationKind,
    /// Continuation payload (stable u32 tokens; meaning depends on kind).
    ///
    /// DyingSave: `[dying_player, killer, responder, rest_len, rest...]`
    /// DiscardToLimit: `[player, need_count]`
    pub continuation_payload: Vec<u32>,
    /// Saved continuation kind when dying or a nested choice interrupts resolution.
    pub resume_kind: ContinuationKind,
    /// Saved continuation payload for resume after the nested choice.
    pub resume_payload: Vec<u32>,
    /// Exact Sha cards whose post-use no-damage action still needs resolution.
    /// A zero skill id means refund the ordinary use count; non-zero means skill-attributed draw.
    #[serde(default)]
    pub sha_no_damage_draw_skills: Vec<(CardId, SkillId)>,
    /// In-flight logical Sha uses whose complete targets and physical Shan sources must survive
    /// nested choices until the post-use boundary.
    #[serde(default)]
    pub sha_resolution_records: Vec<ShaResolutionRecord>,
    /// Duel cards that committed damage and have not emitted their completion event yet.
    #[serde(default)]
    pub duel_damage_cards: Vec<CardId>,
    /// Virtual card replays armed by current-card skills and waiting for a true card-use boundary.
    #[serde(default)]
    pub pending_card_replays: Vec<PendingCardReplay>,
    /// In-flight complete card uses waiting for the shared post-resolution event boundary.
    #[serde(default)]
    pub pending_card_resolutions: Vec<PendingCardResolution>,
    /// Multi-card view-as sources queued before their `UseCard` effect executes.
    #[serde(default)]
    pub planned_card_source_cards: Vec<(PlayerId, CardId, Vec<CardId>, bool)>,
    /// Configured max effect budget (segment resets to this).
    pub effect_budget_max: u32,
    /// Effect budget remaining in current segment.
    pub effect_budget: u32,
    /// Trigger depth budget max.
    pub trigger_budget_max: u32,
    /// Trigger budget remaining.
    pub trigger_budget: u32,
    /// Skill instance that originated the effect currently being dispatched.
    ///
    /// Scheduler-local provenance; stable states and snapshots always observe zero.
    #[serde(skip)]
    pub effect_source_skill: u32,
    /// For `VmResume`: serialized VM continuation (frame pack). Empty otherwise.
    #[serde(default)]
    pub vm_cont_frames: Vec<u8>,
    /// For `VmResume`: module name whose bytecode to resume.
    #[serde(default)]
    pub vm_cont_module: String,
    /// For `VmResume`: program counter to resume at.
    #[serde(default)]
    pub vm_cont_pc: u32,
    /// For `VmResume`: fuel remaining for the resumed segment.
    #[serde(default)]
    pub vm_cont_fuel: u32,
    /// For `VmResume`: host context (self, event_player, event_source, event_card).
    #[serde(default)]
    pub vm_cont_ctx: [u32; 4],
    /// For `VmResume`: current skill instance id.
    #[serde(default)]
    pub vm_cont_skill: u32,
    /// Trigger marker to emit after a paused trigger VM completes.
    #[serde(default)]
    pub vm_cont_trigger_marker: [u32; 3],
    /// Effect queue suspended behind the current choice barrier.
    #[serde(default)]
    pub deferred_effects: Vec<EffectRequest>,
    /// Effect continuations that must run after all triggers produced by the current effect.
    #[serde(default)]
    pub deferred_tail_effects: Vec<EffectRequest>,
    /// Fixed trigger queue suspended behind the current choice barrier.
    #[serde(default)]
    pub deferred_triggers: Vec<crate::trigger::PendingTrigger>,
    /// Optional trigger currently awaiting owner confirmation.
    #[serde(default)]
    pub optional_trigger: Option<crate::trigger::PendingTrigger>,
}

impl Default for ResolutionState {
    fn default() -> Self {
        Self {
            pending_choice_id: 0,
            next_choice_seq: 0,
            next_damage_seq: 0,
            current_damage_id: 0,
            gzbuqu_nodying: false,
            damage_redirect_in_progress: false,
            prevented_damage_ids: Vec::new(),
            pending_bound_damage_deltas: Vec::new(),
            next_recovery_seq: 0,
            current_recovery_id: 0,
            prevented_recovery_ids: Vec::new(),
            current_hand_compare_player: 0,
            current_hand_compare_target: 0,
            current_hand_compare_random_mask: 0,
            continuation_id: 0,
            pending_observer: 0,
            choice_revision: 0,
            continuation_kind: ContinuationKind::None,
            continuation_payload: Vec::new(),
            resume_kind: ContinuationKind::None,
            resume_payload: Vec::new(),
            sha_no_damage_draw_skills: Vec::new(),
            sha_resolution_records: Vec::new(),
            duel_damage_cards: Vec::new(),
            pending_card_replays: Vec::new(),
            pending_card_resolutions: Vec::new(),
            planned_card_source_cards: Vec::new(),
            effect_budget_max: DEFAULT_EFFECT_BUDGET,
            effect_budget: DEFAULT_EFFECT_BUDGET,
            trigger_budget_max: DEFAULT_TRIGGER_BUDGET,
            trigger_budget: DEFAULT_TRIGGER_BUDGET,
            effect_source_skill: 0,
            vm_cont_frames: Vec::new(),
            vm_cont_module: String::new(),
            vm_cont_pc: 0,
            vm_cont_fuel: 0,
            vm_cont_ctx: [0; 4],
            vm_cont_skill: 0,
            vm_cont_trigger_marker: [0; 3],
            deferred_effects: Vec::new(),
            deferred_tail_effects: Vec::new(),
            deferred_triggers: Vec::new(),
            optional_trigger: None,
        }
    }
}

/// Canonical replay armed by a skill until the exact source card use reaches its end boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingCardReplay {
    /// Exact source card whose use must finish first.
    pub source_card: CardId,
    /// Replay user.
    pub player: PlayerId,
    /// Logical card kind, including Sha nature.
    pub kind: u8,
    /// Original declared targets.
    pub targets: Vec<PlayerId>,
    /// Originating skill instance.
    pub skill: SkillId,
}

/// Canonical context retained until one logical card use reaches its shared end boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingCardResolution {
    /// Card user.
    pub source: PlayerId,
    /// Physical carrier or engine materialization.
    pub card: CardId,
    /// Effective card-kind tag captured at use time.
    pub kind: u8,
    /// Declared targets.
    pub targets: Vec<PlayerId>,
    /// Targets damaged by this exact card use.
    pub damaged_targets: Vec<PlayerId>,
    /// Whether the physical carrier originated in the user's hand.
    pub source_was_hand: bool,
    /// Whether every physical material originated in the user's hand.
    #[serde(default)]
    pub all_sources_were_hand: bool,
    /// Red/black mask forbidden for other players' responses to this exact card.
    pub response_color_lock_mask: u8,
    /// Every physical material consumed by this logical use.
    #[serde(default)]
    pub source_cards: Vec<CardId>,
    /// Skill definition that produced this view-as use, or zero for ordinary uses.
    #[serde(default)]
    pub originating_view_as_skill_def: u32,
    /// Targets excluded by the originating skill before resolution.
    #[serde(default)]
    pub skill_excluded_targets: Vec<PlayerId>,
}

/// Canonical context retained for one in-flight logical Sha use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaResolutionRecord {
    /// Sha user.
    pub source: PlayerId,
    /// Physical Sha carrier or engine materialization.
    pub card: CardId,
    /// Complete declared targets.
    pub targets: Vec<PlayerId>,
    /// Whether this proactive Sha incremented the per-turn use count.
    #[serde(default)]
    pub use_counted: bool,
    /// Whether that use count has already been refunded.
    #[serde(default)]
    pub use_refunded: bool,
    /// Whether damage from this exact Sha has no player source.
    #[serde(default)]
    pub damage_source_none: bool,
    /// Every physical card consumed by this logical Sha, in conversion order.
    #[serde(default)]
    pub source_cards: Vec<CardId>,
    /// Physical Shan source cards in response order.
    pub response_cards: Vec<CardId>,
    /// Originating skill/target bindings armed for this exact Sha.
    #[serde(default)]
    pub skill_targets: Vec<(SkillId, PlayerId)>,
    /// Skill/target pairs that chose a post-resolution backwater retaliation.
    #[serde(default)]
    pub backwater_targets: Vec<(SkillId, PlayerId)>,
    /// Targets that entered dying from this exact Sha.
    #[serde(default)]
    pub dying_targets: Vec<PlayerId>,
}

mod u8_is_zero;
pub(crate) use u8_is_zero::*;

mod tag;
