//! Effect primitive implementations — bingshi 3 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// Custom-continuation marker for ABI 456 `choose_discard_heart_redirect_current_damage`.
pub(crate) const RESOLVE_TIANXIANG_CUSTOM_CHOICE: u32 = 0xDCD8_4560;

/// Custom-continuation marker for ABI 457 `choose_discard_heart_cancel_damage_branch`.
pub(crate) const RESOLVE_RETIANXIANG_CUSTOM_CHOICE: u32 = 0xDCE1_4570;

/// Custom-continuation marker for ABI 458 `choose_link_other_same_hp`.
pub(crate) const RESOLVE_LIANZHOU_CUSTOM_CHOICE: u32 = 0xDCE3_4580;

/// Custom-continuation marker for ABI 459 `choose_give_hand_equip_to_event_player`.
pub(crate) const RESOLVE_STDDECHONG_CUSTOM_CHOICE: u32 = 0xDCE7_4590;

/// Custom-continuation marker for ABI 460 `discard_hand_equip_count`.
pub(crate) const DISCARD_HAND_EQUIP_COUNT_CUSTOM_CHOICE: u32 = 0xDCE9_4600;

/// Custom-continuation marker for ABI 596 `choose_discard_hand_range`.
pub(crate) const CHOOSE_DISCARD_HAND_RANGE_CHOICE: u32 = 0xDCDC_5960;

/// Custom-continuation marker for ABI 597 `resolve_dcctjiuxian`.
pub(crate) const RESOLVE_DCCTJIUXIAN_CHOICE: u32 = 0xDCDC_5970;
/// Custom-continuation marker for ABI 598 `resolve_bianyue`.
pub(crate) const RESOLVE_BIANYUE_CHOICE: u32 = 0xDCDC_5980;
/// Custom-continuation marker for ABI 600 `resolve_zhupeilan`.
pub(crate) const RESOLVE_ZHUPEILAN_CHOICE: u32 = 0xDCDC_6000;
pub const RESOLVE_DC_SB_ZHANGXIU_CHOICE: u32 = 0xDCDC_6010;
pub const RESOLVE_DC_SB_ZHOUYU_CHOICE: u32 = 0xDCDC_6020;
pub const RESOLVE_DC_SB_LUSU_CHOICE: u32 = 0xDCDC_6030;
pub const RESOLVE_ZHANGJIAN_CHOICE: u32 = 0xDCDC_6040;

/// Custom-continuation marker for ABI 461 `resolve_jieliang_gain`.
pub(crate) const RESOLVE_JIELIANG_GAIN_CUSTOM_CHOICE: u32 = 0xDCEB_4610;

/// Custom-continuation marker for ABI 462 `choose_other_gain_event_cards`.
pub(crate) const RESOLVE_STDJUJIAN_CUSTOM_CHOICE: u32 = 0xDCED_4620;

/// Custom-continuation marker for ABI 463 `resolve_lulve`.
pub(crate) const RESOLVE_LULVE_CUSTOM_CHOICE: u32 = 0xDCEF_4630;

/// Custom-continuation marker for ABI 464 `resolve_dctingxian`.
pub(crate) const RESOLVE_DCTINGXIAN_CUSTOM_CHOICE: u32 = 0xDCF1_4640;

/// Custom-continuation marker for ABI 465 `choose_other_add_skill`.
pub(crate) const RESOLVE_HFJIEYING_CUSTOM_CHOICE: u32 = 0xDCF3_4650;

/// Custom-continuation marker for ABI 466 `resolve_dcnuchen`.
pub(crate) const RESOLVE_DCNUCHEN_CUSTOM_CHOICE: u32 = 0xDCF5_4660;

/// Custom-continuation marker for ABI 467 `resolve_dczhiheng`.
pub(crate) const RESOLVE_DCZHIHENG_CUSTOM_CHOICE: u32 = 0xDCF7_4670;

/// Custom-continuation marker for ABI 468 `resolve_stdgaobian`.
pub(crate) const RESOLVE_STDGAOBIAN_CUSTOM_CHOICE: u32 = 0xDCF9_4680;

/// Custom-continuation marker for ABI 469 `resolve_spjiedao`.
pub(crate) const RESOLVE_SPJIEDAO_CUSTOM_CHOICE: u32 = 0xDCFA_4690;

/// Custom-continuation marker for ABI 470 `resolve_sibian`.
pub(crate) const RESOLVE_SIBIAN_CUSTOM_CHOICE: u32 = 0xDCFB_4700;

/// Custom-continuation marker for ABI 491 category-card distribution.
pub(crate) const REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE: u32 = 0xDCFB_4910;

/// Custom-continuation marker for ABI 492 deterministic region-card gain.
pub(crate) const RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE: u32 = 0xDCD2_4920;

/// Custom-continuation marker for ABI 493 `resolve_refuhai`.
pub(crate) const REFUHAI_CUSTOM_CHOICE: u32 = 0xDCD5_4930;

/// Custom-continuation marker for ABI 494 `choose_gain_target_hand_ceil_half_difference`.
pub(crate) const YANGWU_GAIN_HAND_CUSTOM_CHOICE: u32 = 0xDCD6_4940;

/// Custom-continuation marker for ABI 495 `resolve_gzjinfa`.
pub(crate) const GZJINFA_CUSTOM_CHOICE: u32 = 0xDCD7_4950;

/// Custom-continuation marker for ABI 497 `yuanlve`.
pub(crate) const YUANLVE_CUSTOM_CHOICE: u32 = 0xDCD9_4970;

/// Custom-continuation marker for ABI 499 `jiebing`.
pub(crate) const JIEBING_CUSTOM_CHOICE: u32 = 0xDCDA_4990;

/// Custom-continuation marker for ABI 500 `mobiledanshou`.
pub(crate) const MOBILEDANSHOU_CUSTOM_CHOICE: u32 = 0xDCDB_5000;

/// Custom-continuation marker for ABI 501 `jsrgchiying`.
pub(crate) const RESOLVE_JSRGCHIYING_CUSTOM_CHOICE: u32 = 0xDCDC_5010;

/// Custom-continuation markers for ABI 503 国渊 skills.
pub(crate) const MBQINGDAO_CUSTOM_CHOICE: u32 = 0xDCDD_5031;
pub(crate) const MBXIUGENG_CUSTOM_CHOICE: u32 = 0xDCDD_5032;
pub(crate) const MBCHENSHE_CUSTOM_CHOICE: u32 = 0xDCDD_5033;
pub(crate) const MBRENXING_CUSTOM_CHOICE: u32 = 0xDCDD_5041;
pub(crate) const MBCHIZHANG_CUSTOM_CHOICE: u32 = 0xDCDD_5051;
pub(crate) const MBDUANYANG_CUSTOM_CHOICE: u32 = 0xDCDD_5052;
/// Custom-continuation marker for ABI 529 势桓阶「共谋」。
pub(crate) const POTHUANJIE_CUSTOM_CHOICE: u32 = 0xDCDD_5291;
pub(crate) const POTNIGU_CUSTOM_CHOICE: u32 = 0xDCDD_5321;
pub(crate) const POTLULIAN_CUSTOM_CHOICE: u32 = 0xDCDD_5322;
/// Custom-continuation marker for ABI 535 势鲁肃「缔盟」。
pub(crate) const POTDIMENG_CUSTOM_CHOICE: u32 = 0xDCDD_5351;
pub(crate) const POTHAOSHI_CUSTOM_CHOICE: u32 = 0xDCDD_5361;

pub(crate) const MBRUNWEI_CUSTOM_CHOICE: u32 = 0xDCDD_5061;
pub(crate) const MBSHUANGHUAI_CUSTOM_CHOICE: u32 = 0xDCDD_5062;
pub(crate) const MBXIONGTU_CUSTOM_CHOICE: u32 = 0xDCDD_5071;
pub(crate) const MBSIJIAN_CUSTOM_CHOICE: u32 = 0xDCDD_5081;
pub(crate) const MBJIANJI_CUSTOM_CHOICE: u32 = 0xDCDD_5091;
pub(crate) const MBFEIJING_CUSTOM_CHOICE: u32 = 0xDCDD_5101;
pub(crate) const MBXUYE_CUSTOM_CHOICE: u32 = 0xDCDD_5141;
pub(crate) const POTCHENGPU_CUSTOM_CHOICE: u32 = 0xDCDD_5151;
pub(crate) const POTJUGU_CUSTOM_CHOICE: u32 = 0xDCDD_5161;
pub(crate) const POTSIFENG_CUSTOM_CHOICE: u32 = 0xDCDD_5171;
pub(crate) const POTCHENQUN_CUSTOM_CHOICE: u32 = 0xDCDD_5181;
pub(crate) const POTCHENDAO_CUSTOM_CHOICE: u32 = 0xDCDD_5191;
pub(crate) const MBZHUJI_CUSTOM_CHOICE: u32 = 0xDCDD_5211;
pub(crate) const POTPOXI_CUSTOM_CHOICE: u32 = 0xDCDD_5221;
pub(crate) const POTFUAN_CUSTOM_CHOICE: u32 = 0xDCDD_5251;
pub(crate) const MBYUANMO_CUSTOM_CHOICE: u32 = 0xDCDD_5092;

mod twyuyan_gift_cards;
pub(crate) use twyuyan_gift_cards::*;

mod resolve_tianxiang;
pub(crate) use resolve_tianxiang::*;
mod resolve_mbrenxing;
pub(crate) use resolve_mbrenxing::*;
mod mbxiongtu_damage;
pub(crate) use mbxiongtu_damage::*;
