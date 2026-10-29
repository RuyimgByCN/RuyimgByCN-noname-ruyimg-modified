//! Effect primitive implementations — ddd 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const RESOLVE_WDB_ZAOZHIRENJUN_CHOICE: u32 = 0xDCDC_6160;
pub(crate) const RESOLVE_DDD_XIANXI_CHOICE: u32 = 0xDCDC_6170;
pub(crate) const RESOLVE_DDD_XIAHENG_CHOICE: u32 = 0xDCDC_6180;
pub(crate) const RESOLVE_DDD_LIUCHONG_CHOICE: u32 = 0xDCDC_6190;
pub(crate) const RESOLVE_DDD_DUANBING_CHOICE: u32 = 0xDCDC_6210;
pub(crate) const RESOLVE_DDD_WUZHI_CHOICE: u32 = 0xDCDC_6220;
pub(crate) const DDD_XIANXI_STATE_MAGIC: u32 = 0xDDD0_6170;
pub(crate) const DDD_XIAHENG_STATE_MAGIC: u32 = 0xDDD0_6180;
pub(crate) const DDD_MOYAN_STATE_MAGIC: u32 = 0xDDD0_6190;

pub(crate) const RESOLVE_DDD_XUJING_CHOICE: u32 = 0xDCDC_6230;
pub(crate) const DDD_XUYU_STATE: &[u8; 4] = b"DXU1";
pub(crate) const DDD_SHIJIAN_STATE: &[u8; 4] = b"DSJ1";

pub(crate) const RESOLVE_DDD_CAOMAO_CHOICE: u32 = 0xDCDC_6240;
pub(crate) const DDD_QUCHE_EFFECT_DEF: u32 = 3_815_717_800;

pub(crate) const RESOLVE_DDD_XINXIANYING_CHOICE: u32 = 0xDCDC_6250;
pub(crate) const DDD_DONGCHA_DEF: u32 = 3_036_718_178;

mod current_wugu_revealed;
pub(crate) use current_wugu_revealed::*;

mod save_ddd_xiaheng_state;
pub(crate) use save_ddd_xiaheng_state::*;
mod resolve_ddd_duanbing;
pub(crate) use resolve_ddd_duanbing::*;
mod resolve_ddd_xujing;
pub(crate) use resolve_ddd_xujing::*;
mod resolve_ddd_xinxianying_choice;
pub(crate) use resolve_ddd_xinxianying_choice::*;
