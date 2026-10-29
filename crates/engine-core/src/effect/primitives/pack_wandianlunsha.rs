//! Effect primitive implementations — wandianlunsha cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub const RESOLVE_ZHUGERUOXUE_CHOICE: u32 = 0xDCDC_6050;
pub const RESOLVE_CAOYI_CHOICE: u32 = 0xDCDC_6060;
pub const RESOLVE_MALINGLI_CHOICE: u32 = 0xDCDC_6070;

pub const RESOLVE_NSSHUAIYAN_CHOICE: u32 = 0xDCDC_6080;

pub const RESOLVE_WDB_LIUFU_CHOICE: u32 = 0xDCDC_6100;

pub const RESOLVE_WDB_LIUZAN_CHOICE: u32 = 0xDCDC_6120;

pub(crate) const RESOLVE_WDB_TIANYU_CHOICE: u32 = 0xDCDC_6130;

pub(crate) const RESOLVE_WDB_FUCHOU_CHOICE: u32 = 0xDCDC_6140;

pub(crate) const RESOLVE_WDB_YANGYI_CHOICE: u32 = 0xDCDC_6150;

mod resolve_dc_sb_lusu_choice;
pub(crate) use resolve_dc_sb_lusu_choice::*;

mod remove_empty_malingli_helper;
pub(crate) use remove_empty_malingli_helper::*;
mod resolve_wdb_liufu_choice;
pub(crate) use resolve_wdb_liufu_choice::*;
mod resolve_wdb_xizhenxihong;
pub(crate) use resolve_wdb_xizhenxihong::*;
mod resolve_wdb_yangyi_choice;
pub(crate) use resolve_wdb_yangyi_choice::*;
