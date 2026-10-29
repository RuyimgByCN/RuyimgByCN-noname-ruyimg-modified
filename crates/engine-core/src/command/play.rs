//! PlayCard command handling (Phase 5: basics + equip + simple tricks).

use crate::effect::{run_effects, EffectError};
use crate::query::{query, QueryContext};
use crate::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{CardDefId, CardId, CardMoveReason, PlayerId, SkillId, Suit};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SHA_RESUME_RESPONDABLE: u32 = 1 << 0;
const SHA_RESUME_COUNTED: u32 = 1 << 1;
const SHA_RESUME_REFUNDED: u32 = 1 << 2;
const SHA_TARGET_DAMAGE_BONUS_SHIFT: u32 = 8;
const SHA_TARGET_LOW_BITS_MASK: u32 = (1 << SHA_TARGET_DAMAGE_BONUS_SHIFT) - 1;

const TIESUO_SAVED_TARGETS_MAGIC: u32 = 0x5453_4254;

/// Target shape for a card used from a resolving skill rather than a PlayCard command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SkillCardTargetSpec {
    /// No choice is needed; use these targets.
    Immediate(Vec<PlayerId>),
    /// Select distinct players from `options`.
    Select {
        min: u8,
        max: u8,
        options: Vec<PlayerId>,
    },
    /// Select one prevalidated ordered target pair (借刀杀人).
    Pairs(Vec<(PlayerId, PlayerId)>),
}

pub(crate) const SAVED_WUXIE_SOURCE_CARD_MAGIC: u32 = 0x5758_4344;
const SAVED_CARD_BASE_DAMAGE_MAGIC: u32 = 0x4244_4d47;
const SAVED_TARGET_EXCLUDED_MASK: u32 = 1 << 31;

const INDEPENDENT_TRICK_QUEUE_MAGIC: u32 = 0x4e54_5155;
pub(crate) const MANTIANGUOHAI_CUSTOM_CHOICE: u32 = 0x4d54_5391;
pub(crate) const MANTIANGUOHAI_SUCCESS_MASK: u32 = 1 << 30;
pub(crate) const DIAOBING_POOL_MAGIC: u32 = 0x4442_504c;
pub(crate) const WUGU_POOL_MAGIC: u32 = 0x5747_504c;

/// Deferred non-delayed trick effect tags (after 无懈 window).
pub(crate) mod trick_effect {
    pub const WUZHONG: u32 = 1;
    pub const GUOHE: u32 = 2;
    pub const SHUNSHOU: u32 = 3;
    pub const HUOGONG: u32 = 4;
    pub const JUEDOU: u32 = 5;
    pub const NANMAN: u32 = 6;
    pub const WANJIAN: u32 = 7;
    pub const TAOYUAN: u32 = 8;
    pub const WUGU: u32 = 9;
    pub const TIESUO: u32 = 10;
    pub const JIEDAO: u32 = 11;
    /// Place delayed trick into judge zone: args `[card, target, kind_tag]`.
    pub const DELAYED_PLACE: u32 = 12;
    pub const YUANJIAO: u32 = 13;
    pub const TIAOJIYANMEI: u32 = 14;
    pub const MANTIANGUOHAI: u32 = 15;
    pub const YOUDISHENRU: u32 = 16;
    pub const DIAOBINGQIANJIANG: u32 = 17;
    pub const WANGMEIZHIKE: u32 = 18;
    /// 以逸待劳（国战：对己方全体，各摸两张再弃两张）.
    pub const YIYI: u32 = 19;
    /// 水淹七军（国战：目标弃置全部装备牌或受 1 点雷电伤害）.
    pub const SHUIYANQIJUNX: u32 = 20;
    /// 戮力同心（国战：全部大势力或全部小势力；未横置→横置，已横置→摸一张）.
    pub const LULITONGXIN: u32 = 21;
    /// 克复中原（国战：任意目标，各目标视为使用杀或摸牌；蜀势力伤害+1/摸两张）.
    pub const KEFUZHONGYUAN: u32 = 22;
    /// 固国安邦（国战：自用，摸八弃≥六；吴势力可将弃牌交给其他吴角色）.
    pub const GUGUOANBANG: u32 = 23;
    /// 火烧连营（国战：对下家及其队列所有角色造成 1 点火属性伤害）.
    pub const HUOSHAOLIANYING: u32 = 24;
    /// 联军盛宴（国战：对己方及一敌方势力全体；己方分配摸/回，敌方摸1并重置）.
    pub const LIANJUNSHENGYAN: u32 = 25;
    /// 文和乱武（国战：全角色；使用者选弃两张类型不同手牌或弃其一张手牌，
    /// 群势力空手牌则补至体力至多 5）.
    pub const WENHELUANWU: u32 = 26;
    /// 号令天下（国战：对一名体力值非全场最少者使用；其他角色依次二选一）.
    pub const HAOLINGTIANXIA: u32 = 27;
    /// 敕令（国战：对所有有暗置武将的角色使用；各目标选明置并摸一、失去1点体力
    /// 或弃置一张装备牌）.
    pub const CHILING: u32 = 28;
    /// 出其不意（展示目标一张手牌，与此牌花色不同则造成 1 点伤害）.
    pub const CHUQIBUYI: u32 = 29;
}

const YOUDISHENRU_SAVED_MAGIC: u32 = 0x5944_5352;
pub(crate) const YOUDISHENRU_FORCE_CHOICE: u32 = 0x5944_5343;

#[cfg(test)]
mod tests {
    use super::{
        pack_damage_card_context, sha_queue_target_respondable, unpack_damage_card_context,
    };

    #[test]
    fn damage_card_context_keeps_zero_flag_payloads_backward_compatible() {
        assert_eq!(pack_damage_card_context(2, 0), 2);
        let packed = pack_damage_card_context(1, rule_api::damage_card_flag::JIU_BOOSTED);
        assert_eq!(
            unpack_damage_card_context(packed),
            (1, rule_api::damage_card_flag::JIU_BOOSTED)
        );
    }

    #[test]
    fn multi_sha_queue_keeps_one_response_flag_per_target() {
        let payload = [3, 1, 1, 0, 2, 0, 1, 0, 10, 11, 0, 1];
        assert!(!sha_queue_target_respondable(&payload, 8, 2, 0));
        assert!(sha_queue_target_respondable(&payload, 8, 2, 1));

        let old_payload = [3, 1, 1, 0, 2, 0, 1, 0, 10, 11, 0];
        assert!(!sha_queue_target_respondable(&old_payload, 8, 2, 0));
        assert!(!sha_queue_target_respondable(&old_payload, 8, 2, 1));
    }
}

mod pack_damage_card_context;
pub(crate) use pack_damage_card_context::*;

mod redirect_saved_sha_target;
pub(crate) use redirect_saved_sha_target::*;
mod current_card_use_enhancement_extra_targets;
pub(crate) use current_card_use_enhancement_extra_targets::*;
mod handle_card_with_policies_and_flags;
pub(crate) use handle_card_with_policies_and_flags::*;
mod source_free_ordinary_trick_kinds;
pub(crate) use source_free_ordinary_trick_kinds::*;
mod skill_card_target_spec_with_options;
pub(crate) use skill_card_target_spec_with_options::*;
mod play_as_kind_with_extra_damage_flags;
pub(crate) use play_as_kind_with_extra_damage_flags::*;
mod source_free_wuzhong_has_legal_use;
pub(crate) use source_free_wuzhong_has_legal_use::*;
mod saved_independent_trick_damage_card_and_bonus;
pub(crate) use saved_independent_trick_damage_card_and_bonus::*;
mod open_sha_response_step;
pub(crate) use open_sha_response_step::*;
mod use_youdishenru_for_saved_sha;
pub(crate) use use_youdishenru_for_saved_sha::*;
mod apply_deferred_trick;
pub(crate) use apply_deferred_trick::*;
mod saved_aoe_base_damage;
pub(crate) use saved_aoe_base_damage::*;
mod play_prompted_sha_source;
pub(crate) use play_prompted_sha_source::*;
