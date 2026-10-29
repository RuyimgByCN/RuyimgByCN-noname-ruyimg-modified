//! Effect primitive implementations — bingshi 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const POTZHOUYU_CHIYUN_DEF: u32 = 4_200_000_566;
pub(crate) const POTZHOUYU_YANHUI_DEF: u32 = 4_200_000_567;
pub(crate) const POTZHOUYU_FENTAO_DEF: u32 = 4_200_000_568;
pub(crate) const POTZHOUYU_DISABLE_ONE: u64 = 1 << 1;
pub(crate) const POTZHOUYU_DISABLE_TWO: u64 = 1 << 2;

#[derive(Default)]
pub(crate) struct PotyanhuiState {
    armed: bool,
    revealed: Vec<(CardId, PlayerId)>,
    discarded: Vec<PlayerId>,
}

const RESOLVE_DCBINGJI_CUSTOM_CHOICE: u32 = 0xDCB0_0001;

const RESOLVE_QIANLONG_MAGIC: u32 = 0x51_4C_4E_01;

const RESOLVE_ZUOZHAN_CUSTOM_CHOICE: u32 = 0x5A55_0001;

/// 「坐瞻」标记（记于所选角色，对应上游 markAuto("jsrgzuozhan") 的目标集合）。
pub(crate) const ZUOZHAN_SIT_MARK: u32 = 0x5A55_0002;

const RESOLVE_ZUOZHAN_GAIN_CUSTOM_CHOICE: u32 = 0x5A55_0003;

const RESOLVE_JSRGDAOREN_CUSTOM_CHOICE: u32 = 0x4A53_0001;

const RESOLVE_PXQIXIAN_CUSTOM_CHOICE: u32 = 0x5078_0001;

const QUANJIA_MARK: u32 = 0x51A_0001;

const RESOLVE_TWMOUZHI_MAGIC: u32 = 0x4D4F_0001;

const TWMOUZHI_COLOR_KEY: u32 = 1;

const RESOLVE_TWDAIGUI_MAGIC: u32 = 0x5444_0001;

const RESOLVE_TWMITU_MAGIC: u32 = 0x544D_0001;

/// 「迷途」gaintag（content_card_tags，对应上游 gaintag "twmitu"）。
pub(crate) const TWMITU_TAG: u32 = 3_195_907_057;

const RESOLVE_QIANTUN_MAGIC: u32 = 0x514E_0001;

const RESOLVE_YANGMING_MAGIC: u32 = 0x594D_0001;

const RESOLVE_CANYUN_MAGIC: u32 = 0x4359_0001;

const RESOLVE_BIAOZHAO3_MAGIC: u32 = 0x425A_0001;

/// 表召族「表」堆 def（biaozhao 自身堆，跨技 def 字面量）。
const BIAOZHAO_PILE_DEF: u32 = 3_640_810_944;

const RESOLVE_BIAOZHAO2_MAGIC: u32 = 0x425A_0002;

const RESOLVE_DCNIJI_MAGIC: u32 = 0x4E4A_0001;

/// 「逆击」gaintag（content_card_tags，对应上游 gaintag "dcniji"）。
pub(crate) const DCNIJI_TAG: u32 = 3_628_735_017;

const RESOLVE_DCMOUKUI_MAGIC: u32 = 0x4D4B_0001;

/// 谋溃「双选已定」标记的 storage 键（slice 2 追罚用）。
const DCMOUKUI_FLAG_KEY: u32 = 1;

const RESOLVE_ZHONGYAN_MAGIC: u32 = 0x5A5A_0001;

const RESOLVE_CHUMEI_MAGIC: u32 = 0x4348_0001;

const RESOLVE_QINMIAN_MAGIC: u32 = 0x514D_0001;

/// 「勤勉」gaintag（content_card_tags，对应上游 gaintag "qinmian"）。
pub(crate) const QINMIAN_TAG: u32 = 3_958_551_236;

const RESOLVE_SHUAIYAN_MAGIC: u32 = 0x5348_0001;

const RESOLVE_CHULAO_MAGIC: u32 = 0x434C_0001;

const RESOLVE_DINGCE_MAGIC: u32 = 0x4443_0002;

const RESOLVE_DCQIAOLVE_MAGIC: u32 = 0x514C_0001;

/// 专属状态机 (批 287, ADR 0641 §8.72)：雄军——锁定技，owner 造成伤害后，
/// 每名拥有〖雄军〗的角色各摸一张牌（按 def 遍历存活持有者）。单发无续体。
/// 雄军去重键：记录已处理的 damage seq（DealDamage 对 DamageDealt 存在
/// 双分派——批 287 实测；同一伤害只结算一次）。
const RESOLVE_TW_XIONGJUN_LAST_DAMAGE: u32 = 0x584A_0002;

const RESOLVE_MEIYONG_MAGIC: u32 = 0x4D59_0001;

const RESOLVE_XINFU_QIAI_MAGIC: u32 = 0x5149_0001;

const SPOLZHUBEI_DEF: u32 = 3_309_571_556;
const SPOLXIXIANG_DEF: u32 = 2_531_785_939;

const RESOLVE_OLZHIJIE_MAGIC: u32 = 0x5A4A_0001;

/// 「巧织」gaintag（content_card_tags，对应上游 gaintag "olqiaozhi"）。
pub(crate) const OLQIAOZHI_TAG: u32 = 1_909_178_934;

const RESOLVE_OLQIAOZHI_MAGIC: u32 = 0x515A_0001;

const RESOLVE_ZHONGJIE_MAGIC: u32 = 0x5A4A_0002;

const RESOLVE_NHLUNDAO_MAGIC: u32 = 0x4C44_0001;

const RESOLVE_QIANGYONG_MAGIC: u32 = 0x5159_0001;

/// 羌勇防重复触发键：记录已处理过的 damage id（引擎对同一伤害可能重复
/// 交付 damage_before 事件）。
const RESOLVE_QIANGYONG_LAST_DAMAGE: u32 = 0x5159_0002;

const RESOLVE_MBZHENANN_MAGIC: u32 = 0x5A4E_0001;

const RESOLVE_DCANXU_MAGIC: u32 = 0x4443_0001;

const RESOLVE_DUODUAN_MAGIC: u32 = 0x4444_0001;

const RESOLVE_GAOYUAN_MAGIC: u32 = 0x4747_0001;

const RESOLVE_MBBEIMING_MAGIC: u32 = 0x4242_0001;

const RESOLVE_ZHENGJIAN_MAGIC: u32 = 0x5A4A_0001;

/// 「诤荐」用牌计数标记（mode 1/2 累计，mode 3 结算清空）。
pub(crate) const ZHENGJIAN_MARK: u32 = 2_446_053_157;

const RESOLVE_LIANGYIN_MAGIC: u32 = 0x524C_0001;

mod begin_choose_compare_selected_hand_card;
pub(crate) use begin_choose_compare_selected_hand_card::*;

mod choose_draw_or_discard_target_card;
pub(crate) use choose_draw_or_discard_target_card::*;
mod choose_current_card_use_enhancements;
pub(crate) use choose_current_card_use_enhancements::*;
mod set_potyinzhan_entries;
pub(crate) use set_potyinzhan_entries::*;
mod resolve_potzhouyu_choice;
pub(crate) use resolve_potzhouyu_choice::*;
mod zuozhan_distinct_basics_from_discard;
pub(crate) use zuozhan_distinct_basics_from_discard::*;
mod resolve_dcbingji;
pub(crate) use resolve_dcbingji::*;
mod twmouzhi_card_color;
pub(crate) use twmouzhi_card_color::*;
mod resolve_twmitu_round;
pub(crate) use resolve_twmitu_round::*;
mod resolve_yangming_round;
pub(crate) use resolve_yangming_round::*;
mod resolve_dcmoukui_stage;
pub(crate) use resolve_dcmoukui_stage::*;
mod resolve_shuaiyan;
pub(crate) use resolve_shuaiyan::*;
mod resolve_olqiaozhi;
pub(crate) use resolve_olqiaozhi::*;
mod resolve_mbzhenann;
pub(crate) use resolve_mbzhenann::*;
