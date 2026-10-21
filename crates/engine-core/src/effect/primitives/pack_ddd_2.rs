//! Effect primitive implementations — ddd 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const RESOLVE_DDD_XIANGLANG_CHOICE: u32 = 0xDCDC_6260;
pub(crate) const DDD_XUNXUN_DEF: engine_api::SkillDefId =
    engine_api::SkillDefId::new(1_673_829_014);
pub(crate) const DDD_FUSI_GLOBAL_DEF: engine_api::SkillDefId =
    engine_api::SkillDefId::new(crate::command::skill::DDD_FUSI_GLOBAL_DEF);
pub(crate) const DDD_TUOJI_DEF: engine_api::SkillDefId = engine_api::SkillDefId::new(1_700_500_735);

pub(crate) const RESOLVE_DDD_YUJIN_CHOICE: u32 = 0xDCDC_6270;
pub(crate) const DDD_ZHENGJUN_HP: u8 = 1;
pub(crate) const DDD_ZHENGJUN_HAND: u8 = 2;
pub(crate) const DDD_ZHENGJUN_EQUIP: u8 = 4;

pub(crate) const RESOLVE_DDD_LIUYE_CHOICE: u32 = 0xDCDC_6280;

pub(crate) const RESOLVE_DDD_BAOSANNIANG_CHOICE: u32 = 0xDCDC_6290;

pub(crate) const RESOLVE_DDD_ZHENJI_CHOICE: u32 = 0xDCDC_6300;
pub(crate) const DDD_MIAOXING_PILE_1: u32 = 919_529_013;
pub(crate) const DDD_MIAOXING_PILE_2: u32 = 2_949_101_455;

pub(crate) const RESOLVE_DDD_ZHAOANG_CHOICE: u32 = 0xDCDC_6310;

pub(crate) const RESOLVE_DDD_LIUBA_CHOICE: u32 = 0xDCDC_6320;
pub(crate) const DDD_YULV_DEF: u32 = 694_263_366;

mod resolve_ddd_xianglang;
pub(crate) use resolve_ddd_xianglang::*;
mod resolve_ddd_jijian_kind_choice;
pub(crate) use resolve_ddd_jijian_kind_choice::*;
mod resolve_ddd_liuba;
pub(crate) use resolve_ddd_liuba::*;
