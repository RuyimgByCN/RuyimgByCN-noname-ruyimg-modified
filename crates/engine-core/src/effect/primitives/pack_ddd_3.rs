//! Effect primitive implementations — ddd 3 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const RESOLVE_DDD_JIANSHUO_CHOICE: u32 = 0xDCDC_6330;
pub(crate) const DDD_SHICHAO_STATE_TAG: [u8; 4] = *b"SC33";

pub(crate) const RESOLVE_DDD_GUANNING_CHOICE: u32 = 0xDCDC_6340;
pub(crate) const DDD_YOUXUE_MARK: u32 = 2_449_657_606;
pub(crate) const DDD_CHENGJING_DEF: u32 = 4_114_032_104;

pub(crate) const RESOLVE_DDD_CAOSHUANG_CHOICE: u32 = 0xDCDC_6350;

pub(crate) const RESOLVE_DDD_XUELINGYUN_CHOICE: u32 = 0xDCDC_6360;

pub(crate) const RESOLVE_DDD_LIUHONG_CHOICE: u32 = 0xDCDD_6370;

pub(crate) const RESOLVE_DDD_XIAHOUXUAN_CHOICE: u32 = 0xDCDD_6380;
pub(crate) const DDD_LANGHUAI_DEF: u32 = 4_025_119_557;
pub(crate) const DDD_XUANLUN_DEF: u32 = 3_478_177_951;
pub(crate) const DDD_XUANLUN_PENDING: u32 = 1;
pub(crate) const DDD_XUANLUN_FORCE_LANGHUAI: u32 = 2;

pub(crate) const RESOLVE_DDD_ZHANGKAI_CHOICE: u32 = 0xDCDD_6390;
pub(crate) const DDD_JIEXING_DEF: u32 = 3_377_566_778;
pub(crate) const DDD_BAILEI_DEF: u32 = 2_689_073_031;
pub(crate) const DDD_JIEXING_IGNORE_STOLEN_RECOVERY: u64 = 1;

mod ddd_liuba_decode_distribution;
pub(crate) use ddd_liuba_decode_distribution::*;

mod resolve_ddd_jianshuo;
pub(crate) use resolve_ddd_jianshuo::*;
mod resolve_ddd_guanning_choice;
pub(crate) use resolve_ddd_guanning_choice::*;
mod resolve_ddd_liuhong;
pub(crate) use resolve_ddd_liuhong::*;
