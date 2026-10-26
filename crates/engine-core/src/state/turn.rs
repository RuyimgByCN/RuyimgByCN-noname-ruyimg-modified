//! Turn and phase state.

use engine_api::{CardId, PlayerId, SkillId, TurnPhaseTag};
use serde::{Deserialize, Serialize};

/// Phase within a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Phase {
    /// Before anyone's turn.
    #[default]
    Idle,
    /// Prepare.
    Prepare,
    /// Judge.
    Judge,
    /// Draw.
    Draw,
    /// Play.
    Play,
    /// Discard.
    Discard,
    /// End.
    End,
}

impl Phase {
    /// Map to public API tag.
    pub fn to_tag(self) -> TurnPhaseTag {
        match self {
            Phase::Idle => TurnPhaseTag::Idle,
            Phase::Prepare => TurnPhaseTag::Prepare,
            Phase::Judge => TurnPhaseTag::Judge,
            Phase::Draw => TurnPhaseTag::Draw,
            Phase::Play => TurnPhaseTag::Play,
            Phase::Discard => TurnPhaseTag::Discard,
            Phase::End => TurnPhaseTag::End,
        }
    }

    /// From raw u8 (effect path).
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Phase::Prepare,
            2 => Phase::Judge,
            3 => Phase::Draw,
            4 => Phase::Play,
            5 => Phase::Discard,
            6 => Phase::End,
            _ => Phase::Idle,
        }
    }

    /// To raw u8.
    pub fn to_u8(self) -> u8 {
        match self {
            Phase::Idle => 0,
            Phase::Prepare => 1,
            Phase::Judge => 2,
            Phase::Draw => 3,
            Phase::Play => 4,
            Phase::Discard => 5,
            Phase::End => 6,
        }
    }
}

/// Turn state.
/// A cropped extra turn: only the listed phases run for `player`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExtraPhaseTurn {
    /// Player who gains the cropped turn.
    pub player: PlayerId,
    /// Allowed phases, encoded as `TurnPhaseTag` discriminants.
    pub phases: Vec<u8>,
    /// When set, the cropped turn does not count as a turn-over for round logic.
    pub no_turn_over: bool,
}

/// Per-match turn sequencing state (round counter, phase flow, queues).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnState {
    /// Round number (starts at 0 pre-game, 1 on first turn).
    pub round: u32,
    /// Current player (NONE if idle).
    pub current: PlayerId,
    /// Phase.
    pub phase: Phase,
    /// Extra turn flags (content); sorted key-value for hash stability.
    pub flags: Vec<(u32, i32)>,
    /// Marks removed from a player during this turn, as `(mark_def_raw, player)`.
    ///
    /// Used by rules that need "a mark was removed earlier this turn" after the
    /// stack is gone (涤仇 R01). Cleared at the turn boundary together with the
    /// per-player counters; empty states serialize/hash exactly as before.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub removed_marks: std::collections::BTreeSet<(u32, PlayerId)>,
    /// First-loss order of physical cards each player lost from hand/equipment this turn.
    #[serde(default)]
    pub lost_cards: Vec<(PlayerId, CardId)>,
    /// Physical cards discarded during this turn. Consumers must still check that a card remains
    /// in the discard pile because later effects may move it elsewhere.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub discarded_cards: std::collections::BTreeSet<CardId>,
    /// Physical cards that entered the discard pile during this turn, regardless of move reason.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub entered_discard_cards: std::collections::BTreeSet<CardId>,
    /// Ordered physical cards used this turn that can become 管宁's「经」.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ddd_chengjing_eligible_uses: Vec<CardId>,
    /// Physical materials of logical cards that dealt damage during this turn.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub damage_card_materials: std::collections::BTreeSet<CardId>,
    /// Turn-scoped content tags carried by cards that left a player's hand/equipment this turn.
    /// Minimal gaintag_map foundation: records which gaintags were "lost" so a later trigger can
    /// detect "a tagged card was consumed". Hash-neutral when empty.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub lost_card_tags: std::collections::BTreeSet<u32>,
    /// Turn-scoped card-use bans: (banned user, banned target) pairs.
    #[serde(default)]
    pub use_ban_pairs: Vec<(PlayerId, PlayerId)>,
    /// Physical cards that entered each player's hand during this turn.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub gained_hand_cards: std::collections::BTreeSet<(PlayerId, CardId)>,
    /// Players who gained a physical hand card in the current Play phase.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub play_phase_card_gain_players: std::collections::BTreeSet<PlayerId>,
    /// Players who completed a draw effect in the current Play phase.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub play_phase_draw_players: std::collections::BTreeSet<PlayerId>,
    /// Physical hand cards excluded from one player's ordinary discard-to-limit accounting
    /// for this turn.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub hand_limit_exempt_cards: std::collections::BTreeSet<(PlayerId, CardId)>,
    /// Applied damage-event counts by source and target for this turn.
    #[serde(
        default,
        with = "pair_map_u32",
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub damage_pair_counts: std::collections::BTreeMap<(PlayerId, PlayerId), u32>,
    /// Card-use counts by user and declared target for this turn.
    #[serde(
        default,
        with = "pair_map_u32",
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub card_target_counts: std::collections::BTreeMap<(PlayerId, PlayerId), u32>,
    /// Distinct players whose dying state was entered because of damage from each source this turn.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub dying_caused_targets: std::collections::BTreeSet<(PlayerId, PlayerId)>,
    /// Players that entered dying for any reason during this turn.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub entered_dying_players: std::collections::BTreeSet<PlayerId>,
    /// Physical cards used or responded with from a skill pile exposed as hand cards this turn.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub used_playable_skill_pile_cards: std::collections::BTreeSet<(PlayerId, CardId)>,
    /// Skill piles from which a card was used or responded with this turn.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub used_playable_skill_piles: std::collections::BTreeSet<(PlayerId, u32)>,
    /// Originating skills for queued turn-opening Play phases, in resolution order.
    #[serde(default)]
    pub opening_extra_play_queue: Vec<SkillId>,
    /// Originating skill for the active turn-opening Play phase.
    #[serde(default = "none_skill_id")]
    pub opening_extra_play_active_skill: SkillId,
    /// Owner damage total captured when the active turn-opening Play phase began.
    #[serde(default)]
    pub opening_extra_play_damage_baseline: u32,
    /// Full extra turns queued by turn-end skills, in resolution order.
    #[serde(default)]
    pub extra_turn_queue: Vec<PlayerId>,
    /// Cropped extra turns: run only the listed phases (phaseList semantics,
    /// offline zj_fuli / extra_offline sm_kulian).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_phase_queue: Vec<ExtraPhaseTurn>,
    /// Play-only extra phases queued after the current turn, in resolution order.
    #[serde(default)]
    pub extra_play_phase_queue: Vec<PlayerId>,
    /// Whether the current Play phase is an inserted play-only phase.
    #[serde(default)]
    pub extra_play_phase_active: bool,
    /// Draw-only extra phases queued after the current turn, in resolution order.
    #[serde(default)]
    pub extra_draw_phase_queue: Vec<PlayerId>,
    /// Whether the current Draw phase is an inserted draw-only phase.
    #[serde(default)]
    pub extra_draw_phase_active: bool,
    /// Normal-turn anchor retained while queued extra turns or phases are being inserted.
    #[serde(default = "no_player")]
    pub extra_turn_anchor: PlayerId,
    /// Skill temporarily suppressing other players' non-locked skills during this turn.
    #[serde(default = "none_skill_id")]
    pub non_locked_skill_suppression: SkillId,
    /// Players whose non-locked skills are suppressed until this turn ends.
    #[serde(default)]
    pub non_locked_skill_suppressed_players: Vec<PlayerId>,
}

fn none_skill_id() -> SkillId {
    SkillId::NONE
}

fn no_player() -> PlayerId {
    PlayerId::NONE
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            round: 0,
            use_ban_pairs: Vec::new(),
            current: PlayerId::NONE,
            phase: Phase::Idle,
            flags: Vec::new(),
            removed_marks: std::collections::BTreeSet::new(),
            lost_cards: Vec::new(),
            discarded_cards: std::collections::BTreeSet::new(),
            entered_discard_cards: std::collections::BTreeSet::new(),
            ddd_chengjing_eligible_uses: Vec::new(),
            damage_card_materials: std::collections::BTreeSet::new(),
            lost_card_tags: std::collections::BTreeSet::new(),
            gained_hand_cards: std::collections::BTreeSet::new(),
            play_phase_card_gain_players: std::collections::BTreeSet::new(),
            play_phase_draw_players: std::collections::BTreeSet::new(),
            hand_limit_exempt_cards: std::collections::BTreeSet::new(),
            damage_pair_counts: std::collections::BTreeMap::new(),
            card_target_counts: std::collections::BTreeMap::new(),
            dying_caused_targets: std::collections::BTreeSet::new(),
            entered_dying_players: std::collections::BTreeSet::new(),
            used_playable_skill_pile_cards: std::collections::BTreeSet::new(),
            used_playable_skill_piles: std::collections::BTreeSet::new(),
            opening_extra_play_queue: Vec::new(),
            opening_extra_play_active_skill: SkillId::NONE,
            opening_extra_play_damage_baseline: 0,
            extra_turn_queue: Vec::new(),
            extra_phase_queue: Vec::new(),
            extra_play_phase_queue: Vec::new(),
            extra_play_phase_active: false,
            extra_draw_phase_queue: Vec::new(),
            extra_draw_phase_active: false,
            extra_turn_anchor: PlayerId::NONE,
            non_locked_skill_suppression: SkillId::NONE,
            non_locked_skill_suppressed_players: Vec::new(),
        }
    }
}

/// Turn flag keys (content / core).
pub mod turn_flag {
    use engine_api::PlayerId;

    /// Skip draw phase this turn (兵粮).
    pub const SKIP_DRAW: u32 = 1;
    /// Skip play phase this turn (乐).
    pub const SKIP_PLAY: u32 = 2;
    /// Number of proactive 杀 used by the current player this turn (克己 checks non-zero).
    pub const SHA_PLAYED: u32 = 3;
    /// Set when a content rule replaces the current player's draw phase.
    pub const DRAW_REPLACED: u32 = 4;
    /// Total damage dealt by the current player during this turn.
    pub const DAMAGE_DEALT: u32 = 5;
    /// Current player dealt damage to another Qun character this turn.
    pub const DAMAGE_DEALT_OTHER_QUN: u32 = 6;
    /// Total cards proactively used by the current player this turn.
    pub const CARD_USED: u32 = 7;
    /// Bitset of players that received damage during this turn (players are capped at eight).
    pub const DAMAGED_PLAYERS: u32 = 8;
    /// Previous card used by the current player: suit, rank, and phase packed into one value.
    pub const LAST_CARD_META: u32 = 9;
    /// Previous card used during the current play phase: suit and rank packed into one value.
    pub const LAST_PLAY_CARD_META: u32 = 10;
    /// Effective kind of the previous card used by the current player.
    pub const LAST_CARD_KIND: u32 = 11;
    /// Physical definition id of the previous card used by the current player.
    pub const LAST_CARD_DEF: u32 = 12;
    /// Physical card currently being used through a view-as rule (raw CardId).
    /// Cleared immediately after the nested use pipeline returns.
    pub const VIEW_AS_CARD: u32 = 13;
    /// Bitset of players that dealt damage during this turn (players are capped at eight).
    pub const DAMAGE_SOURCES: u32 = 14;
    /// Physical cards discarded because any player used or responded with them this turn.
    pub const USE_RESPONSE_DISCARDED: u32 = 15;
    /// Number of extra Play phases queued by turn-begin skills before Prepare.
    pub const OPENING_EXTRA_PLAY_PENDING: u32 = 16;
    /// Non-zero while the current Play phase belongs to the turn-opening queue.
    pub const OPENING_EXTRA_PLAY_ACTIVE: u32 = 17;
    /// Number of Sha uses charged against the current Play phase's allowance.
    pub const SHA_PLAYED_IN_PLAY_PHASE: u32 = 18;
    /// Non-zero when the current turn emitted an explicit `TurnBegan` event.
    pub const EXPLICIT_TURN_BEGIN_EVENT: u32 = 19;
    /// Number of phases skipped during the current turn, including delayed-card, modifier, and
    /// content-effect skips.
    pub const SKIPPED_PHASE_COUNT: u32 = 20;
    /// Non-zero once the current turn player uses a card that names another player as a target.
    pub const CARD_TARGETED_OTHER: u32 = 21;
    /// Additional cards accumulated for the pending draw phase and consumed exactly once by the
    /// authoritative draw entry.
    pub const CURRENT_DRAW_COUNT_BONUS: u32 = 22;
    /// Phase-scoped bitset of players named as targets by a used card.
    pub const PHASE_TARGETED_PLAYERS: u32 = 23;
    /// Phase-scoped bitset of players whose actual HP value changed.
    pub const PHASE_HP_CHANGED_PLAYERS: u32 = 24;
    /// Non-zero while a card created by an ending-phase trigger resolves; its nested target and
    /// HP events must not leak into the following phase.
    pub const PHASE_HISTORY_SUPPRESSED: u32 = 25;
    /// Non-zero once the current turn player deals damage to another player this turn.
    pub const DAMAGE_DEALT_OTHER: u32 = 26;
    /// Non-zero when a Draw-phase trigger has replaced the pending normal draw with its own
    /// resolution. Kept separate from Luoyi's damage-enabling `DRAW_REPLACED` flag.
    pub const DRAW_PHASE_DRAW_ZEROED: u32 = 27;
    /// Physical hand/equipment cards discarded by the current player during the current
    /// Discard phase.
    pub const DISCARD_PHASE_DISCARDED: u32 = 28;
    /// One plus the `lost_cards` length when the current Discard phase was entered; zero when
    /// the phase was skipped or no Discard phase has been entered this turn. Slice base for
    /// 固政-style discard-phase card tracking (ABI 513).
    pub const DISCARD_PHASE_LOST_BASE: u32 = 41;
    /// 飞军（ABI 526）：本回合拥有者首次选择了此前未选择过的新目标。
    const FEIJUN_NEW_TARGET_BASE: u32 = 1900;

    /// Turn flag key recording that the owner first chose a new 飞军 target this turn.
    pub fn feijun_new_target_by_player(player: PlayerId) -> u32 {
        FEIJUN_NEW_TARGET_BASE + player.raw()
    }

    /// Whether the key belongs to the per-player 飞军 new-target family.
    pub(super) fn is_feijun_new_target_by_player(key: u32) -> bool {
        (FEIJUN_NEW_TARGET_BASE + 1..=FEIJUN_NEW_TARGET_BASE + 8).contains(&key)
    }
    /// The End-phase audit event has run and normal turn advancement may proceed.
    pub const TURN_END_DISPATCHED: u32 = 29;
    /// Skill instance currently producing a view-as card.
    /// Cleared immediately after the nested use/response pipeline returns.
    pub const VIEW_AS_SKILL: u32 = 30;
    /// Bitmask of extra phases inserted after End: bit 0 = Draw, bit 1 = Play.
    pub const END_EXTRA_PHASES: u32 = 31;
    /// One extra Draw phase to finish before entering or skipping the normal Draw phase.
    pub const PRE_NORMAL_EXTRA_DRAW: u32 = 32;
    /// Turn-scoped bitset of physical suits that entered the discard pile.
    pub const DISCARD_PILE_SUIT_MASK: u32 = 33;
    /// Non-zero while the current Draw-phase event has fixed its pending normal draw count.
    ///
    /// This is distinct from the count itself because `TurnState::set_flag` removes zero values,
    /// while a fixed count of zero is the most common draw-replacement case.
    pub const DRAW_PHASE_NUM_FIXED: u32 = 34;
    /// Pending normal Draw-phase count when `DRAW_PHASE_NUM_FIXED` is set.
    pub const DRAW_PHASE_FIXED_COUNT: u32 = 35;
    /// Logical card kind currently produced by a view-as rule; cleared with `VIEW_AS_SKILL`.
    pub const VIEW_AS_LOGICAL_KIND: u32 = 36;
    /// Skip the current turn's future Discard phase once.
    pub const SKIP_DISCARD: u32 = 37;
    /// Bitmask (PlayerId - 1 bits) of the previous card's targets, recorded on CardUsed
    /// when the turn card-name history is tracked (xinfu_lianpian 连篇). ABI 472.
    pub const LAST_CARD_TARGETS: u32 = 38;
    /// Bitmask (PlayerId - 1 bits) of the previous Sha's targets, recorded on Sha use
    /// (gongjian 拱坚). ABI 473.
    pub const LAST_SHA_TARGETS: u32 = 39;
    /// Non-zero after the first discard move outside the ordinary Discard phase this turn.
    pub const NON_DISCARD_PHASE_DISCARD_SEEN: u32 = 40;
    const CARD_USED_BY_PLAYER_BASE: u32 = 100;
    const LAST_GAME_CARD_META_BY_PLAYER_BASE: u32 = 200;
    const CARD_ACTION_COUNT_BY_PLAYER_BASE: u32 = 300;
    const LOST_CARD_COUNT_BY_PLAYER_BASE: u32 = 3000;
    const EXTRA_TURN_CHECK_PENDING_BY_PLAYER_BASE: u32 = 3100;
    const EXTRA_TURN_ACTIVE_BY_PLAYER_BASE: u32 = 3200;
    const CARD_TARGETED_COUNT_BY_PLAYER_BASE: u32 = 3300;
    const SHA_DAMAGE_DEALT_BY_PLAYER_BASE: u32 = 400;
    const RED_CARD_TARGET_STATE_BY_PLAYER_BASE: u32 = 500;
    const RED_CARD_TARGET_CARD_BY_PLAYER_BASE: u32 = 600;
    const SHA_USED_BY_PLAYER_BASE: u32 = 700;
    const HAND_CARD_SUIT_HISTORY_BY_PLAYER_BASE: u32 = 800;
    const PLAY_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE: u32 = 900;
    const PLAY_PHASE_CARD_USED_BY_PLAYER_BASE: u32 = 1000;
    const CARD_GAINED_BY_PLAYER_BASE: u32 = 1100;
    const CARD_LOST_BY_PLAYER_BASE: u32 = 1200;
    const KILL_COUNT_BY_PLAYER_BASE: u32 = 1300;
    const JUEDOU_USED_BY_PLAYER_BASE: u32 = 1400;
    const USED_CARD_SUIT_HISTORY_BY_PLAYER_BASE: u32 = 1500;
    const PLAY_PHASE_CARD_SUIT_HISTORY_BY_PLAYER_BASE: u32 = 1600;
    const OWNER_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE: u32 = 1700;
    const HAND_DISCARD_EVENT_BY_PLAYER_BASE: u32 = 1800;
    const HP_CHANGE_COUNT_BY_PLAYER_BASE: u32 = 1900;
    /// Turn-scoped count of damage events actually applied to one player (only recorded when the
    /// loaded content asks for it via the ABI 347 opt-in, so unrelated package sets keep their
    /// canonical state hashes).
    const DAMAGE_TAKEN_COUNT_BY_PLAYER_BASE: u32 = 2000;
    /// Turn-scoped bitset of logical basic-card name/nature variants used by one player.
    const BASIC_CARD_VARIANT_HISTORY_BY_PLAYER_BASE: u32 = 2100;
    /// Turn-scoped basic/trick/equipment mask gained through TW 教化 by one player.
    const TWJIAOHUA_CATEGORY_BY_PLAYER_BASE: u32 = 2200;

    /// Fixed flag: 成略（许攸）本回合无限制花色的位掩码（位 1..=4 对应黑桃/红桃/梅花/方块）。
    pub const CHENGLVE_UNLIMITED_SUITS: u32 = 41;
    /// Fixed flag: 寸目（许攸，主公技）当前摸牌阶段改为从牌堆底摸牌。
    pub const DRAW_PHASE_FROM_BOTTOM: u32 = 42;
    /// Turn-scoped bitset of logical basic/trick/equipment categories one player already used
    /// through 恃才（许攸）首用判定。
    const SHICAI_USED_CATEGORY_BY_PLAYER_BASE: u32 = 2300;
    /// Turn-scoped 雄乱（张绣）限定目标（0 = 无限制）。
    const XIONGLUAN_TARGET_BY_PLAYER_BASE: u32 = 2400;
    /// Turn-scoped bitset (seat bits) of players damaged BY one source player
    /// this turn (ADR 0644 P3 霸政：受我伤者意见对齐).
    const DAMAGED_BY_PLAYER_BASE: u32 = 2500;
    /// Turn-scoped 踞襄（张绣）触发角色本回合已用。
    const TWJUXIANG_USED_BY_EVENT_PLAYER_BASE: u32 = 2500;
    const XIACE_USED_BY_PLAYER_BASE: u32 = 2600;
    const DCSHAOWEI_USED_BY_PLAYER_BASE: u32 = 2700;
    const DCSBMENGMOU_USED_BY_PLAYER_BASE: u32 = 2800;
    /// Round-scoped proactive card-use counter key for one player (resets at round rollover).
    /// Foundation for upstream `player.getRoundHistory("useCard").length`.
    const ROUND_CARD_USED_BY_PLAYER_BASE: u32 = 2900;
    /// Turn-scoped all-phase card-use counter key for one player (resets when current changes).
    const TURN_CARD_USED_BY_PLAYER_BASE: u32 = 3000;

    /// Per-turn branch mask for XIANDING 柏灵筠「黠策」.
    pub fn xiace_used_by_player(player: PlayerId) -> u32 {
        XIACE_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Whether a flag key stores a 柏灵筠「黠策」per-turn branch mask.
    pub fn is_xiace_used_by_player(key: u32) -> bool {
        (XIACE_USED_BY_PLAYER_BASE..XIACE_USED_BY_PLAYER_BASE + 100).contains(&key)
    }

    /// Per-turn branch mask for XIANDING 威关银屏「绍威」.
    pub fn dcshaowei_used_by_player(player: PlayerId) -> u32 {
        DCSHAOWEI_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Whether a flag key stores a「绍威」per-turn branch mask.
    pub fn is_dcshaowei_used_by_player(key: u32) -> bool {
        (DCSHAOWEI_USED_BY_PLAYER_BASE..DCSHAOWEI_USED_BY_PLAYER_BASE + 100).contains(&key)
    }

    /// Per-turn branch mask for XIANDING 新杀谋鲁肃「盟谋」.
    pub fn dcsbmengmou_used_by_player(player: PlayerId) -> u32 {
        DCSBMENGMOU_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Whether a flag key stores「盟谋」per-turn branch usage.
    pub fn is_dcsbmengmou_used_by_player(key: u32) -> bool {
        (DCSBMENGMOU_USED_BY_PLAYER_BASE..DCSBMENGMOU_USED_BY_PLAYER_BASE + 100).contains(&key)
    }

    /// Turn-scoped 恃才（许攸）已判定为“本回合该类牌首用”的类别位掩码键。
    pub fn shicai_used_category_by_player(player: PlayerId) -> u32 {
        SHICAI_USED_CATEGORY_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped 雄乱（张绣）限定目标键（值为目标 PlayerId，0 = 无限制）。
    pub fn xiongluan_target_by_player(player: PlayerId) -> u32 {
        XIONGLUAN_TARGET_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped 踞襄（张绣）触发角色本回合是否已用键。
    pub fn twjuxiang_used_by_event_player(event_player: PlayerId) -> u32 {
        TWJUXIANG_USED_BY_EVENT_PLAYER_BASE + event_player.raw()
    }

    /// Turn-scoped used-card counter key for one player.
    pub fn card_used_by_player(player: PlayerId) -> u32 {
        CARD_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Game-scoped previous used-card metadata key for one player.
    pub fn last_game_card_meta_by_player(player: PlayerId) -> u32 {
        LAST_GAME_CARD_META_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped used-or-responded card counter key for one player.
    pub fn card_action_count_by_player(player: PlayerId) -> u32 {
        CARD_ACTION_COUNT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped card-targeted counter key for one player (how many times
    /// they were named a target of a card use this turn).
    pub fn card_targeted_count_by_player(player: PlayerId) -> u32 {
        CARD_TARGETED_COUNT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped logical-Sha damage-event counter key for one source player.
    /// Queue-time marker: `player`'s upcoming extra turn owes a post-turn
    /// lose-1-HP check (jsrgpingrong 平戎尾随失去体力).
    pub fn extra_turn_check_pending_by_player(player: PlayerId) -> u32 {
        EXTRA_TURN_CHECK_PENDING_BY_PLAYER_BASE + player.raw()
    }

    /// Marks that `player` is currently inside their extra turn awaiting the
    /// post-turn check above.
    pub fn extra_turn_active_by_player(player: PlayerId) -> u32 {
        EXTRA_TURN_ACTIVE_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped counter of cards LOST by one player (left the player's
    /// zones to another owner or a shared pile), gated on piles that opted in
    /// via `SkillPileRef.wants_lost_count` (jsrgshacheng 沙城 draw-X).
    pub fn lost_card_count_by_player(player: PlayerId) -> u32 {
        LOST_CARD_COUNT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped logical-Sha damage-event counter key for one source player.
    pub fn sha_damage_dealt_by_player(player: PlayerId) -> u32 {
        SHA_DAMAGE_DEALT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped logical-Sha use counter key for one player.
    pub fn sha_used_by_player(player: PlayerId) -> u32 {
        SHA_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped logical-Juedou use counter key for one player.
    pub fn juedou_used_by_player(player: PlayerId) -> u32 {
        JUEDOU_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped bitset of effective suits used from one player's hand.
    pub fn hand_card_suit_history_by_player(player: PlayerId) -> u32 {
        HAND_CARD_SUIT_HISTORY_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped bitset of effective suits among all cards used by one player.
    pub fn used_card_suit_history_by_player(player: PlayerId) -> u32 {
        USED_CARD_SUIT_HISTORY_BY_PLAYER_BASE + player.raw()
    }

    /// Play-phase-scoped bitset of logical card categories used by one player.
    pub fn play_card_category_history_by_player(player: PlayerId) -> u32 {
        PLAY_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE + player.raw()
    }

    /// Play-phase-scoped proactive card-use counter key for one player.
    pub fn play_phase_card_used_by_player(player: PlayerId) -> u32 {
        PLAY_PHASE_CARD_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Round-scoped proactive card-use counter key for one player.
    pub fn round_card_used_by_player(player: PlayerId) -> u32 {
        ROUND_CARD_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped all-phase card-use counter key for one player.
    pub fn turn_card_used_by_player(player: PlayerId) -> u32 {
        TURN_CARD_USED_BY_PLAYER_BASE + player.raw()
    }

    /// Play-phase-scoped bitset of effective suits among all cards used by one player.
    pub fn play_phase_card_suit_history_by_player(player: PlayerId) -> u32 {
        PLAY_PHASE_CARD_SUIT_HISTORY_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped bitset of logical categories used or discarded by one player.
    pub fn owner_card_category_history_by_player(player: PlayerId) -> u32 {
        OWNER_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped number of physical cards gained by one player.
    pub fn card_gained_by_player(player: PlayerId) -> u32 {
        CARD_GAINED_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped number of physical cards lost from hand/equipment by one player.
    pub fn card_lost_by_player(player: PlayerId) -> u32 {
        CARD_LOST_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped number of hand-card discard events for one player.
    pub fn hand_discard_event_by_player(player: PlayerId) -> u32 {
        HAND_DISCARD_EVENT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped number of actual HP changes for one player.
    pub fn hp_change_count_by_player(player: PlayerId) -> u32 {
        HP_CHANGE_COUNT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped damage-event counter key for one player.
    pub fn damage_taken_count_by_player(player: PlayerId) -> u32 {
        DAMAGE_TAKEN_COUNT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped logical basic-card variant history key for one player.
    /// Flag key holding the seat bitset of players damaged by `player` this turn.
    pub fn damaged_by_player(source: PlayerId) -> u32 {
        DAMAGED_BY_PLAYER_BASE + source.raw()
    }

    /// Whether `key` is a damaged-by-player bitset flag.
    pub(super) fn is_damaged_by_player(key: u32) -> bool {
        (DAMAGED_BY_PLAYER_BASE + 1..=DAMAGED_BY_PLAYER_BASE + 8).contains(&key)
    }

    /// Turn-scoped bitset of logical basic-card variants used by one player.
    pub fn basic_card_variant_history_by_player(player: PlayerId) -> u32 {
        BASIC_CARD_VARIANT_HISTORY_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped TW 教化 category mask key for one player.
    pub fn twjiaohua_category_by_player(player: PlayerId) -> u32 {
        TWJIAOHUA_CATEGORY_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped number of characters killed by one player.
    pub fn kill_count_by_player(player: PlayerId) -> u32 {
        KILL_COUNT_BY_PLAYER_BASE + player.raw()
    }

    /// Turn-scoped distinct-red-card target state for one player: 0=none, 1=one, 2=at least two.
    pub fn red_card_target_state_by_player(player: PlayerId) -> u32 {
        RED_CARD_TARGET_STATE_BY_PLAYER_BASE + player.raw()
    }

    /// Physical card id when exactly one distinct red card targeted this player this turn.
    pub fn red_card_target_card_by_player(player: PlayerId) -> u32 {
        RED_CARD_TARGET_CARD_BY_PLAYER_BASE + player.raw()
    }

    pub(super) fn is_card_used_by_player(key: u32) -> bool {
        (CARD_USED_BY_PLAYER_BASE + 1..=CARD_USED_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_card_action_count_by_player(key: u32) -> bool {
        (CARD_ACTION_COUNT_BY_PLAYER_BASE + 1..=CARD_ACTION_COUNT_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_sha_damage_dealt_by_player(key: u32) -> bool {
        (SHA_DAMAGE_DEALT_BY_PLAYER_BASE + 1..=SHA_DAMAGE_DEALT_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_sha_used_by_player(key: u32) -> bool {
        (SHA_USED_BY_PLAYER_BASE + 1..=SHA_USED_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_juedou_used_by_player(key: u32) -> bool {
        (JUEDOU_USED_BY_PLAYER_BASE + 1..=JUEDOU_USED_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_red_card_target_history(key: u32) -> bool {
        (RED_CARD_TARGET_STATE_BY_PLAYER_BASE + 1..=RED_CARD_TARGET_STATE_BY_PLAYER_BASE + 8)
            .contains(&key)
            || (RED_CARD_TARGET_CARD_BY_PLAYER_BASE + 1..=RED_CARD_TARGET_CARD_BY_PLAYER_BASE + 8)
                .contains(&key)
    }

    pub(super) fn is_hand_card_suit_history(key: u32) -> bool {
        (HAND_CARD_SUIT_HISTORY_BY_PLAYER_BASE + 1..=HAND_CARD_SUIT_HISTORY_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_used_card_suit_history(key: u32) -> bool {
        (USED_CARD_SUIT_HISTORY_BY_PLAYER_BASE + 1..=USED_CARD_SUIT_HISTORY_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_play_card_category_history(key: u32) -> bool {
        (PLAY_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE + 1
            ..=PLAY_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_play_phase_card_used_by_player(key: u32) -> bool {
        (PLAY_PHASE_CARD_USED_BY_PLAYER_BASE + 1..=PLAY_PHASE_CARD_USED_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_round_card_used_by_player(key: u32) -> bool {
        (ROUND_CARD_USED_BY_PLAYER_BASE + 1..=ROUND_CARD_USED_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_turn_card_used_by_player(key: u32) -> bool {
        (TURN_CARD_USED_BY_PLAYER_BASE + 1..=TURN_CARD_USED_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_play_phase_card_suit_history(key: u32) -> bool {
        (PLAY_PHASE_CARD_SUIT_HISTORY_BY_PLAYER_BASE + 1
            ..=PLAY_PHASE_CARD_SUIT_HISTORY_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_hp_change_count_by_player(key: u32) -> bool {
        (HP_CHANGE_COUNT_BY_PLAYER_BASE + 1..=HP_CHANGE_COUNT_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_damage_taken_count_by_player(key: u32) -> bool {
        (DAMAGE_TAKEN_COUNT_BY_PLAYER_BASE + 1..=DAMAGE_TAKEN_COUNT_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_basic_card_variant_history(key: u32) -> bool {
        (BASIC_CARD_VARIANT_HISTORY_BY_PLAYER_BASE + 1
            ..=BASIC_CARD_VARIANT_HISTORY_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_twjiaohua_category_history(key: u32) -> bool {
        (TWJIAOHUA_CATEGORY_BY_PLAYER_BASE + 1..=TWJIAOHUA_CATEGORY_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_owner_card_category_history(key: u32) -> bool {
        (OWNER_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE + 1
            ..=OWNER_CARD_CATEGORY_HISTORY_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_card_gained_by_player(key: u32) -> bool {
        (CARD_GAINED_BY_PLAYER_BASE + 1..=CARD_GAINED_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_card_lost_by_player(key: u32) -> bool {
        (CARD_LOST_BY_PLAYER_BASE + 1..=CARD_LOST_BY_PLAYER_BASE + 8).contains(&key)
    }

    pub(super) fn is_hand_discard_event_by_player(key: u32) -> bool {
        (HAND_DISCARD_EVENT_BY_PLAYER_BASE + 1..=HAND_DISCARD_EVENT_BY_PLAYER_BASE + 8)
            .contains(&key)
    }

    pub(super) fn is_kill_count_by_player(key: u32) -> bool {
        (KILL_COUNT_BY_PLAYER_BASE + 1..=KILL_COUNT_BY_PLAYER_BASE + 8).contains(&key)
    }
}

impl TurnState {
    /// Read flag value (0 if absent).
    pub fn flag(&self, key: u32) -> i32 {
        self.flags
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    }

    /// Set flag (0 removes).
    pub fn set_flag(&mut self, key: u32, value: i32) {
        self.flags.retain(|(k, _)| *k != key);
        if value != 0 {
            self.flags.push((key, value));
            self.flags.sort_by_key(|(k, _)| *k);
        }
    }

    /// Whether the current Draw-phase event has an immutable normal draw count.
    ///
    /// `DRAW_PHASE_DRAW_ZEROED` remains a read-compatible alias for snapshots produced before
    /// fixed non-zero draw counts were modeled independently.
    pub fn draw_phase_is_fixed(&self) -> bool {
        self.flag(turn_flag::DRAW_PHASE_NUM_FIXED) != 0
            || self.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED) != 0
    }

    /// Immutable normal draw count for the current Draw-phase event, when present.
    pub fn draw_phase_fixed_count(&self) -> Option<u32> {
        if self.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED) != 0 {
            Some(0)
        } else if self.flag(turn_flag::DRAW_PHASE_NUM_FIXED) != 0 {
            Some(
                self.flag(turn_flag::DRAW_PHASE_FIXED_COUNT)
                    .max(0)
                    .try_into()
                    .unwrap_or(u32::MAX),
            )
        } else {
            None
        }
    }

    /// Fix the current Draw-phase event's normal draw count.
    pub fn set_draw_phase_fixed_count(&mut self, count: u32) {
        let stored = count.min(i32::MAX as u32) as i32;
        self.set_flag(turn_flag::DRAW_PHASE_NUM_FIXED, 1);
        self.set_flag(turn_flag::DRAW_PHASE_FIXED_COUNT, stored);
        self.set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, i32::from(count == 0));
        self.set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, 0);
    }

    /// Clear Draw-phase fixed-count state at a new draw-event boundary.
    pub fn clear_draw_phase_fixed_count(&mut self) {
        self.set_flag(turn_flag::DRAW_PHASE_NUM_FIXED, 0);
        self.set_flag(turn_flag::DRAW_PHASE_FIXED_COUNT, 0);
        self.set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, 0);
    }

    /// Clear all per-player card and damage counters at the turn boundary.
    pub fn clear_per_player_card_counts(&mut self) {
        self.removed_marks.clear();
        self.flags.retain(|(key, _)| {
            !turn_flag::is_card_used_by_player(*key)
                && !turn_flag::is_card_action_count_by_player(*key)
                && !turn_flag::is_sha_damage_dealt_by_player(*key)
                && !turn_flag::is_sha_used_by_player(*key)
                && !turn_flag::is_juedou_used_by_player(*key)
                && !turn_flag::is_red_card_target_history(*key)
                && !turn_flag::is_hand_card_suit_history(*key)
                && !turn_flag::is_used_card_suit_history(*key)
                && !turn_flag::is_play_card_category_history(*key)
                && !turn_flag::is_play_phase_card_used_by_player(*key)
                && !turn_flag::is_play_phase_card_suit_history(*key)
                && !turn_flag::is_owner_card_category_history(*key)
                && !turn_flag::is_card_gained_by_player(*key)
                && !turn_flag::is_card_lost_by_player(*key)
                && !turn_flag::is_hand_discard_event_by_player(*key)
                && !turn_flag::is_kill_count_by_player(*key)
                && !turn_flag::is_hp_change_count_by_player(*key)
                && !turn_flag::is_damage_taken_count_by_player(*key)
                && !turn_flag::is_basic_card_variant_history(*key)
                && !turn_flag::is_twjiaohua_category_history(*key)
                && !turn_flag::is_feijun_new_target_by_player(*key)
                && !turn_flag::is_xiace_used_by_player(*key)
                && !turn_flag::is_dcshaowei_used_by_player(*key)
                && !turn_flag::is_dcsbmengmou_used_by_player(*key)
                && !turn_flag::is_damaged_by_player(*key)
        });
        self.lost_cards.clear();
        self.discarded_cards.clear();
        self.entered_discard_cards.clear();
        self.ddd_chengjing_eligible_uses.clear();
        self.damage_card_materials.clear();
        self.lost_card_tags.clear();
        self.gained_hand_cards.clear();
        self.damage_pair_counts.clear();
        self.card_target_counts.clear();
        self.dying_caused_targets.clear();
        self.entered_dying_players.clear();
        self.used_playable_skill_pile_cards.clear();
        self.used_playable_skill_piles.clear();
    }

    /// Remember one physical card consumed from a playable skill pile.
    pub fn record_playable_skill_pile_card(&mut self, player: PlayerId, card: CardId) {
        if player.is_some() && card.is_some() {
            self.used_playable_skill_pile_cards.insert((player, card));
        }
    }

    /// Remember that this player consumed a card from one playable skill pile this turn.
    pub fn record_playable_skill_pile(&mut self, player: PlayerId, skill_def: u32) {
        if player.is_some() && skill_def != 0 {
            self.used_playable_skill_piles.insert((player, skill_def));
        }
    }

    /// Whether this player already consumed a card from this playable skill pile this turn.
    pub fn used_playable_skill_pile(&self, player: PlayerId, skill_def: u32) -> bool {
        self.used_playable_skill_piles
            .contains(&(player, skill_def))
    }

    /// Whether this card was consumed from a playable skill pile during the current turn.
    pub fn used_playable_skill_pile_card(&self, player: PlayerId, card: CardId) -> bool {
        self.used_playable_skill_pile_cards
            .contains(&(player, card))
    }

    /// Record physical cards entering one player's hand during the current turn.
    pub fn record_card_gain(&mut self, player: PlayerId, card: CardId) {
        if player.is_none() || card.is_none() {
            return;
        }
        self.gained_hand_cards.insert((player, card));
        let key = turn_flag::card_gained_by_player(player);
        self.set_flag(key, self.flag(key).saturating_add(1));
    }

    /// Physical cards gained by one player during the current turn.
    pub fn card_gain_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::card_gained_by_player(player)).max(0) as u32
    }

    /// Whether a physical card entered this player's hand during the current turn.
    pub fn gained_hand_card(&self, player: PlayerId, card: CardId) -> bool {
        self.gained_hand_cards.contains(&(player, card))
    }

    /// Record that one player gained a hand card in this Play phase.
    pub fn record_play_phase_card_gain(&mut self, player: PlayerId) {
        if player.is_some() {
            self.play_phase_card_gain_players.insert(player);
        }
    }

    /// Whether one player gained a hand card in this Play phase.
    pub fn gained_card_in_play_phase(&self, player: PlayerId) -> bool {
        self.play_phase_card_gain_players.contains(&player)
    }

    /// Record that one player completed a draw effect in this Play phase.
    pub fn record_play_phase_draw(&mut self, player: PlayerId) {
        if player.is_some() {
            self.play_phase_draw_players.insert(player);
        }
    }

    /// Whether one player previously completed a draw effect in this Play phase.
    pub fn drew_in_play_phase(&self, player: PlayerId) -> bool {
        self.play_phase_draw_players.contains(&player)
    }

    /// Record one physical card leaving a player's hand/equipment region this turn.
    pub fn record_card_loss(&mut self, player: PlayerId, card: CardId) {
        let key = turn_flag::card_lost_by_player(player);
        self.set_flag(key, self.flag(key).saturating_add(1));
        if !self.lost_cards.contains(&(player, card)) {
            self.lost_cards.push((player, card));
        }
    }

    /// Record that a card carrying `tag` left a player this turn (gaintag_map foundation).
    /// Called alongside `record_card_loss` for cards that carry content tags.
    pub fn record_lost_card_tag(&mut self, tag: u32) {
        if tag != 0 {
            self.lost_card_tags.insert(tag);
        }
    }

    /// Whether any card carrying `tag` left a player this turn.
    pub fn lost_card_tag(&self, tag: u32) -> bool {
        self.lost_card_tags.contains(&tag)
    }

    /// Clear turn-scoped lost-tag tracking (call at turn/round rollover with the other clears).
    pub fn clear_lost_card_tags(&mut self) {
        self.lost_card_tags.clear();
    }

    /// Record physical cards moved by a discard action during the current turn.
    pub fn record_discarded_cards(&mut self, cards: impl IntoIterator<Item = CardId>) {
        self.discarded_cards
            .extend(cards.into_iter().filter(|card| card.is_some()));
    }

    /// Record physical cards entering the discard pile this turn.
    pub fn record_entered_discard_cards(&mut self, cards: impl IntoIterator<Item = CardId>) {
        self.entered_discard_cards
            .extend(cards.into_iter().filter(|card| card.is_some()));
    }

    /// Record physical materials belonging to a card use that dealt damage this turn.
    pub fn record_damage_card_materials(&mut self, cards: impl IntoIterator<Item = CardId>) {
        self.damage_card_materials
            .extend(cards.into_iter().filter(|card| card.is_some()));
    }

    /// Number of hand/equipment cards lost by one player this turn, counting repeat losses.
    pub fn card_loss_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::card_lost_by_player(player)).max(0) as u32
    }

    /// Record one actual HP-value change during the current turn.
    pub fn record_hp_change(&mut self, player: PlayerId) {
        let key = turn_flag::hp_change_count_by_player(player);
        self.set_flag(key, self.flag(key).saturating_add(1));
    }

    /// Number of actual HP-value changes for one player during the current turn.
    pub fn hp_change_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::hp_change_count_by_player(player))
            .max(0) as u32
    }

    /// Record one applied damage event for one player during the current turn. Only written when
    /// the loaded content opted in via the ABI 347 gate, so unrelated packages keep their hashes.
    pub fn record_damage_taken(&mut self, player: PlayerId) {
        let key = turn_flag::damage_taken_count_by_player(player);
        self.set_flag(key, self.flag(key).saturating_add(1));
    }

    /// Record one card-targeting of `player` this turn. Only written when the loaded content
    /// opted in via the card-target-history track, so unrelated packages keep their hashes.
    pub fn record_card_targeted(&mut self, player: PlayerId) {
        let key = turn_flag::card_targeted_count_by_player(player);
        self.set_flag(key, self.flag(key).saturating_add(1));
    }

    /// Number of times one player was named a card target this turn.
    pub fn card_targeted_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::card_targeted_count_by_player(player)).max(0) as u32
    }

    /// Number of applied damage events for one player during the current turn.
    pub fn damage_taken_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::damage_taken_count_by_player(player))
            .max(0) as u32
    }

    /// Remember one of the six supported logical basic-card name/nature variants this turn.
    pub fn remember_basic_card_variant(&mut self, player: PlayerId, variant: u8) {
        if variant >= 6 {
            return;
        }
        let key = turn_flag::basic_card_variant_history_by_player(player);
        self.set_flag(key, self.flag(key) | (1_i32 << variant));
    }

    /// Number of distinct logical basic-card name/nature variants used this turn.
    pub fn distinct_basic_card_variants(&self, player: PlayerId) -> u32 {
        (self.flag(turn_flag::basic_card_variant_history_by_player(player)) as u32).count_ones()
    }

    /// Record one applied damage event from `source` to `target` during this turn.
    pub fn record_damage_pair(&mut self, source: PlayerId, target: PlayerId) {
        let count = self.damage_pair_counts.entry((source, target)).or_default();
        *count = count.saturating_add(1);
    }

    /// Applied damage-event count for one source-target pair during this turn.
    pub fn damage_pair_count(&self, source: PlayerId, target: PlayerId) -> u32 {
        self.damage_pair_counts
            .get(&(source, target))
            .copied()
            .unwrap_or(0)
    }

    /// Record one card use that declared `target`.
    pub fn record_card_target(&mut self, user: PlayerId, target: PlayerId) {
        let count = self.card_target_counts.entry((user, target)).or_default();
        *count = count.saturating_add(1);
    }

    /// Number of cards one player used targeting another during this turn.
    pub fn card_target_count(&self, user: PlayerId, target: PlayerId) -> u32 {
        self.card_target_counts
            .get(&(user, target))
            .copied()
            .unwrap_or(0)
    }

    /// Applied damage-event count attributed to one source during this turn.
    pub fn damage_dealt_count(&self, source: PlayerId) -> u32 {
        self.damage_pair_counts
            .iter()
            .filter_map(|((found, _), count)| (*found == source).then_some(*count))
            .sum()
    }

    /// Record that this exact damage source made a distinct other player enter dying this turn.
    pub fn record_dying_caused(&mut self, source: PlayerId, target: PlayerId) {
        if source.is_some() && source != target {
            self.dying_caused_targets.insert((source, target));
        }
    }

    /// Record any player entering dying during the current turn.
    pub fn record_entered_dying(&mut self, player: PlayerId) {
        if player.is_some() {
            self.entered_dying_players.insert(player);
        }
    }

    /// Number of distinct other players made dying by this source during the current turn.
    pub fn dying_caused_count(&self, source: PlayerId) -> usize {
        self.dying_caused_targets
            .iter()
            .filter(|(found, _)| *found == source)
            .count()
    }

    /// Unique physical cards lost by one player this turn, in first-loss order.
    pub fn lost_cards_for(&self, player: PlayerId) -> impl Iterator<Item = CardId> + '_ {
        self.lost_cards
            .iter()
            .filter_map(move |(owner, card)| (*owner == player).then_some(*card))
    }

    /// Clear card-category history when a new play phase begins.
    pub fn clear_play_card_category_history(&mut self) {
        self.flags
            .retain(|(key, _)| !turn_flag::is_play_card_category_history(*key));
    }

    /// Clear proactive card-use counters when a new play phase begins.
    pub fn clear_play_phase_card_use_counts(&mut self) {
        self.flags.retain(|(key, _)| {
            !turn_flag::is_play_phase_card_used_by_player(*key)
                && !turn_flag::is_play_phase_card_suit_history(*key)
        });
        self.play_phase_card_gain_players.clear();
        self.play_phase_draw_players.clear();
    }

    /// Number of proactive cards used by one player in the current Play phase.
    pub fn play_phase_card_use_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::play_phase_card_used_by_player(player))
            .max(0) as u32
    }

    /// Record one proactive card use by `player` for the current round.
    pub fn record_round_card_use(&mut self, player: PlayerId) {
        let key = turn_flag::round_card_used_by_player(player);
        self.set_flag(key, self.flag(key).saturating_add(1));
    }

    /// Number of proactive cards used by one player in the current round.
    pub fn round_card_use_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::round_card_used_by_player(player))
            .max(0) as u32
    }

    /// Number of cards used by one player during the current turn (all phases).
    pub fn turn_card_use_count(&self, player: PlayerId) -> u32 {
        self.flag(turn_flag::turn_card_used_by_player(player))
            .max(0) as u32
    }

    /// Record one card use by `player` for the current turn.
    pub fn record_turn_card_use(&mut self, player: PlayerId) {
        let key = turn_flag::turn_card_used_by_player(player);
        self.set_flag(key, self.flag(key).saturating_add(1));
    }

    /// Clear turn-scoped card-use counters (current player changed).
    pub fn clear_turn_card_use_counts(&mut self) {
        self.flags.retain(|(key, _)| !turn_flag::is_turn_card_used_by_player(*key));
    }

    /// Clear all round-scoped proactive card-use counters (call at round rollover).
    pub fn clear_round_card_use_counts(&mut self) {
        self.flags
            .retain(|(key, _)| !turn_flag::is_round_card_used_by_player(*key));
    }

    /// Remember one effective suit used by one player in the current Play phase.
    pub fn remember_play_phase_card_suit_used(&mut self, player: PlayerId, suit: engine_api::Suit) {
        let bit = match suit {
            engine_api::Suit::Spade => 1,
            engine_api::Suit::Heart => 1 << 1,
            engine_api::Suit::Club => 1 << 2,
            engine_api::Suit::Diamond => 1 << 3,
            engine_api::Suit::None => return,
        };
        let key = turn_flag::play_phase_card_suit_history_by_player(player);
        self.set_flag(key, self.flag(key) | bit);
    }

    /// Number of distinct effective card suits used by one player this Play phase.
    pub fn distinct_play_phase_card_suits(&self, player: PlayerId) -> u32 {
        (self.flag(turn_flag::play_phase_card_suit_history_by_player(player)) as u32).count_ones()
    }

    /// Bitmask of distinct effective card suits used by one player this Play phase
    /// (Spade=1, Heart=2, Club=4, Diamond=8).
    pub fn play_phase_card_suit_mask(&self, player: PlayerId) -> u8 {
        self.flag(turn_flag::play_phase_card_suit_history_by_player(player)) as u8
    }

    /// Remember physical card suits entering the discard pile this turn.
    pub fn remember_discard_pile_suits(
        &mut self,
        suits: impl IntoIterator<Item = engine_api::Suit>,
    ) {
        let mut mask = self.flag(turn_flag::DISCARD_PILE_SUIT_MASK);
        for suit in suits {
            mask |= match suit {
                engine_api::Suit::Spade => 1,
                engine_api::Suit::Heart => 1 << 1,
                engine_api::Suit::Club => 1 << 2,
                engine_api::Suit::Diamond => 1 << 3,
                engine_api::Suit::None => 0,
            };
        }
        self.set_flag(turn_flag::DISCARD_PILE_SUIT_MASK, mask);
    }

    /// Remember one of the four effective suits used from a physical hand card this turn.
    pub fn remember_hand_card_suit_used(&mut self, player: PlayerId, suit: engine_api::Suit) {
        let bit = match suit {
            engine_api::Suit::Spade => 1,
            engine_api::Suit::Heart => 1 << 1,
            engine_api::Suit::Club => 1 << 2,
            engine_api::Suit::Diamond => 1 << 3,
            engine_api::Suit::None => return,
        };
        let key = turn_flag::hand_card_suit_history_by_player(player);
        self.set_flag(key, self.flag(key) | bit);
    }

    /// Number of distinct effective suits used from a physical hand card this turn.
    pub fn distinct_hand_card_suits_used(&self, player: PlayerId) -> u32 {
        (self.flag(turn_flag::hand_card_suit_history_by_player(player)) as u32).count_ones()
    }

    /// Remember one of the four effective suits among all cards used this turn.
    pub fn remember_used_card_suit(&mut self, player: PlayerId, suit: engine_api::Suit) {
        let bit = match suit {
            engine_api::Suit::Spade => 1,
            engine_api::Suit::Heart => 1 << 1,
            engine_api::Suit::Club => 1 << 2,
            engine_api::Suit::Diamond => 1 << 3,
            engine_api::Suit::None => return,
        };
        let key = turn_flag::used_card_suit_history_by_player(player);
        self.set_flag(key, self.flag(key) | bit);
    }

    /// Number of distinct effective suits among all cards used by one player this turn.
    pub fn distinct_used_card_suits(&self, player: PlayerId) -> u32 {
        (self.flag(turn_flag::used_card_suit_history_by_player(player)) as u32).count_ones()
    }

    /// Remember a logical basic/trick/equipment category used in the current play phase.
    ///
    /// Bit 3 is a bounded duplicate-category summary, written only when loaded content asks for
    /// it so unrelated package sets keep their existing canonical state hashes.
    pub fn remember_play_card_category_used(
        &mut self,
        player: PlayerId,
        category: u8,
        track_duplicate: bool,
    ) {
        if category >= 3 {
            return;
        }
        let key = turn_flag::play_card_category_history_by_player(player);
        let current = self.flag(key);
        let category_bit = 1_i32 << category;
        let duplicate_bit = if track_duplicate && current & category_bit != 0 {
            1_i32 << 3
        } else {
            0
        };
        self.set_flag(key, current | category_bit | duplicate_bit);
    }

    /// Number of distinct logical card categories used in the current play phase.
    pub fn play_card_category_count(&self, player: PlayerId) -> u32 {
        ((self.flag(turn_flag::play_card_category_history_by_player(player)) as u32) & 0b111)
            .count_ones()
    }

    /// Remember a logical basic/trick/equipment category used or discarded this turn.
    pub fn remember_owner_card_category(&mut self, player: PlayerId, category: u8) {
        if category >= 3 {
            return;
        }
        let key = turn_flag::owner_card_category_history_by_player(player);
        self.set_flag(key, self.flag(key) | (1_i32 << category));
    }

    /// Bitset of logical basic/trick/equipment categories used or discarded this turn.
    pub fn owner_card_category_mask(&self, player: PlayerId) -> u8 {
        (self.flag(turn_flag::owner_card_category_history_by_player(player)) & 0b111) as u8
    }

    /// Whether one logical basic/trick/equipment category was used at least twice this phase.
    pub fn play_card_category_repeated(&self, player: PlayerId) -> bool {
        self.flag(turn_flag::play_card_category_history_by_player(player)) & (1_i32 << 3) != 0
    }

    /// Remember that `player` was targeted by this physical red card during the current turn.
    ///
    /// The saturated 0/1/2 representation is sufficient to answer whether another distinct
    /// card exists while keeping turn state bounded by the player cap instead of card-use count.
    pub fn remember_red_card_target(&mut self, player: PlayerId, card: CardId) {
        let state_key = turn_flag::red_card_target_state_by_player(player);
        let card_key = turn_flag::red_card_target_card_by_player(player);
        match self.flag(state_key) {
            0 => {
                self.set_flag(state_key, 1);
                self.set_flag(card_key, card.raw() as i32);
            }
            1 if self.flag(card_key) as u32 != card.raw() => {
                self.set_flag(state_key, 2);
                self.set_flag(card_key, 0);
            }
            _ => {}
        }
    }

    /// Whether `player` was targeted by a distinct red card other than `current_card` this turn.
    pub fn was_targeted_by_other_red_card(&self, player: PlayerId, current_card: CardId) -> bool {
        let state = self.flag(turn_flag::red_card_target_state_by_player(player));
        state >= 2
            || (state == 1
                && self.flag(turn_flag::red_card_target_card_by_player(player)) as u32
                    != current_card.raw())
    }

    /// Record one player as having received damage this turn.
    pub fn mark_player_damaged(&mut self, player: PlayerId) {
        if let Some(bit) = player
            .raw()
            .checked_sub(1)
            .and_then(|shift| 1_i32.checked_shl(shift))
        {
            self.set_flag(
                turn_flag::DAMAGED_PLAYERS,
                self.flag(turn_flag::DAMAGED_PLAYERS) | bit,
            );
        }
    }

    /// Record one player as having dealt damage this turn.
    pub fn mark_player_dealt_damage(&mut self, player: PlayerId) {
        if let Some(bit) = player
            .raw()
            .checked_sub(1)
            .and_then(|shift| 1_i32.checked_shl(shift))
        {
            self.set_flag(
                turn_flag::DAMAGE_SOURCES,
                self.flag(turn_flag::DAMAGE_SOURCES) | bit,
            );
        }
    }

    /// Record one player as having been named by a card use in the current phase.
    pub fn mark_phase_card_target(&mut self, player: PlayerId) {
        self.mark_phase_player_bit(turn_flag::PHASE_TARGETED_PLAYERS, player);
    }

    /// Record one player as having changed actual HP in the current phase.
    pub fn mark_phase_hp_changed(&mut self, player: PlayerId) {
        self.mark_phase_player_bit(turn_flag::PHASE_HP_CHANGED_PLAYERS, player);
    }

    fn mark_phase_player_bit(&mut self, key: u32, player: PlayerId) {
        if let Some(bit) = player
            .raw()
            .checked_sub(1)
            .and_then(|shift| 1_i32.checked_shl(shift))
        {
            self.set_flag(key, self.flag(key) | bit);
        }
    }

    /// Clear both bounded phase histories after their ending snapshot is captured.
    pub fn clear_phase_histories(&mut self) {
        self.set_flag(turn_flag::PHASE_TARGETED_PLAYERS, 0);
        self.set_flag(turn_flag::PHASE_HP_CHANGED_PLAYERS, 0);
    }

    /// Whether one player dealt damage during this turn.
    pub fn player_dealt_damage(&self, player: PlayerId) -> bool {
        player
            .raw()
            .checked_sub(1)
            .and_then(|shift| 1_i32.checked_shl(shift))
            .is_some_and(|bit| self.flag(turn_flag::DAMAGE_SOURCES) & bit != 0)
    }

    /// Whether one player received damage during this turn.
    pub fn player_received_damage(&self, player: PlayerId) -> bool {
        player
            .raw()
            .checked_sub(1)
            .and_then(|shift| 1_i32.checked_shl(shift))
            .is_some_and(|bit| self.flag(turn_flag::DAMAGED_PLAYERS) & bit != 0)
    }

    /// Number of distinct players that received damage this turn.
    pub fn damaged_player_count(&self) -> u32 {
        (self.flag(turn_flag::DAMAGED_PLAYERS) as u32).count_ones()
    }

    /// Whether draw is skipped.
    pub fn skip_draw(&self) -> bool {
        self.flag(turn_flag::SKIP_DRAW) != 0
    }

    /// Whether play is skipped.
    pub fn skip_play(&self) -> bool {
        self.flag(turn_flag::SKIP_PLAY) != 0
    }

    /// Whether discard is skipped.
    pub fn skip_discard(&self) -> bool {
        self.flag(turn_flag::SKIP_DISCARD) != 0
    }

    /// Number of phases skipped in this turn.
    pub fn skipped_phase_count(&self) -> u32 {
        self.flag(turn_flag::SKIPPED_PHASE_COUNT).max(0) as u32
    }

    /// Record one skipped phase, saturating at the canonical signed flag limit.
    pub fn record_skipped_phase(&mut self) {
        let next = self
            .flag(turn_flag::SKIPPED_PHASE_COUNT)
            .saturating_add(1)
            .max(1);
        self.set_flag(turn_flag::SKIPPED_PHASE_COUNT, next);
    }
}


/// JSON-friendly encoding for `(u8,u8) -> u32` maps: keys become `"a:b"`.
/// `serde_json` cannot serialize tuple keys directly, which previously made
/// non-empty damage/card-target history snapshots fail to encode.
mod pair_map_u32 {
    use super::PlayerId;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeMap;

    pub fn serialize<S: Serializer>(
        map: &BTreeMap<(PlayerId, PlayerId), u32>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut out = serializer.serialize_map(Some(map.len()))?;
        for ((a, b), value) in map {
            serde::ser::SerializeMap::serialize_entry(
                &mut out,
                &format!("{}:{}", a.raw(), b.raw()),
                value,
            )?;
        }
        serde::ser::SerializeMap::end(out)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<(PlayerId, PlayerId), u32>, D::Error> {
        let raw = BTreeMap::<String, u32>::deserialize(deserializer)?;
        raw.into_iter()
            .map(|(key, value)| {
                let (a, b) = key
                    .split_once(':')
                    .ok_or_else(|| serde::de::Error::custom("invalid pair-map key"))?;
                let a = a.parse::<u32>().map_err(serde::de::Error::custom)?;
                let b = b.parse::<u32>().map_err(serde::de::Error::custom)?;
                Ok(((PlayerId::new(a), PlayerId::new(b)), value))
            })
            .collect()
    }
}
