//! Effect primitive implementations — shiji cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// Custom-continuation marker for ABI 423 上议选牌（payload[0]）。
pub(crate) const RESOLVE_SPSHANGYI_CHOICE: u32 = 0xDCDA_4230;

/// 压军（崔琰，ABI 428）续体标记（payload[0]）；stage 在 payload[1]。
pub(crate) const RESOLVE_SPYAJUN_CUSTOM_CHOICE: u32 = 0xDCDA_4280;

/// 尊帝（崔琰，ABI 429）续体标记（payload[0]）；stage 在 payload[1]。
pub(crate) const RESOLVE_SPZUNDI_CUSTOM_CHOICE: u32 = 0xDCDA_4290;

/// 佯解（朱儁，ABI 430）续体标记（payload[0]）；stage 在 payload[1]。
pub(crate) const RESOLVE_XINYANGJIE_CUSTOM_CHOICE: u32 = 0xDCDA_4300;

/// 清雨（糜夫人，ABI 431）续体标记（payload[0]）。
pub(crate) const RESOLVE_QINGYU_CUSTOM_CHOICE: u32 = 0xDCDA_4310;
/// 清雨使命失败授技目标标记（payload[0]）。
pub(crate) const RESOLVE_QINGYU_FAIL_TARGET: u32 = 0xDCDA_4311;

/// 整肃（ABI 432）续体标记（payload[0]）。
pub(crate) const RESOLVE_ZHENGSU_CHOICE: u32 = 0xDCDA_4320;

/// 整肃三技能定义（核心库通用系统，朱儁侯封 / 吕范燕集共用）。
pub(crate) const ZHENGSU_LEIJIN: u32 = 2_584_022_884;
pub(crate) const ZHENGSU_BIANZHEN: u32 = 3_044_720_297;
pub(crate) const ZHENGSU_MINGZHI: u32 = 3_824_072_820;

/// 侯封 / 燕集（ABI 433）续体标记（payload[0]）。
pub(crate) const RESOLVE_HOUFENG_CHOICE: u32 = 0xDCDA_4330;

/// 战海（周处，ABI 435）续体标记（payload[0]）。
pub(crate) const RESOLVE_RECHUHAI_CHOICE: u32 = 0xDCDA_4350;

/// 乡害（周处初始技能，纯 DSL 草稿）def id。
pub(crate) const XIANGHAI_DEF: u32 = 3_356_961_773;
/// 彰名（使命成功获得）def id。
pub(crate) const ZHANGMING_DEF: u32 = 3_550_704_513;

/// 凶竖（贾充，ABI 436）续体标记（payload[0]）。
pub(crate) const RESOLVE_XIONGSHU_CHOICE: u32 = 0xDCDA_4360;

/// 凶竖标记键（= 凶竖技能 def id）。
pub(crate) const XIONGSHU_MARK: u32 = 3_731_458_110;

/// 弃哀（王粲，ABI 439）续体标记（payload[0]）。
pub(crate) const RESOLVE_SPQIAI_CHOICE: u32 = 0xDCDA_4390;

/// 擅袭（王粲，ABI 440）续体标记（payload[0]）。
pub(crate) const RESOLVE_SPSHANXI_CHOICE: u32 = 0xDCDA_4400;
/// 索命（王粲，ABI 441）续体标记（payload[0]）。
pub(crate) const RESOLVE_SUOMING_CHOICE: u32 = 0xDCDA_4410;

/// 檄标记键（= 擅袭技能 def id）。
pub(crate) const SPSHANXI_MARK: u32 = 2_378_382_374;

/// 通夺（刘巴，ABI 442）续体标记（payload[0]）。
pub(crate) const RESOLVE_TONGDUO_CHOICE: u32 = 0xDCDA_4420;

/// 断笔（刘巴，ABI 445）续体标记（payload[0]）。
pub(crate) const RESOLVE_DUANBI_CHOICE: u32 = 0xDCDA_4450;

/// 躁厉（孙翊，ABI 446）续体标记（payload[0]）。
pub(crate) const RESOLVE_ZAOLI_CHOICE: u32 = 0xDCDA_4460;

/// 躁厉标记键（= 躁厉技能 def id）。
pub(crate) const ZAOLI_MARK: u32 = 2_486_461_241;

/// 孤影（向宠，ABI 447）续体标记（payload[0]）。
pub(crate) const RESOLVE_GUYING_CHOICE: u32 = 0xDCDA_4470;

/// Custom-continuation marker for ABI 427 合击二选一（payload[0]）。
pub(crate) const RESOLVE_HEJI_CHOICE: u32 = 0xDCDA_4270;

mod resolve_spchijie;
pub(crate) use resolve_spchijie::*;

mod resolve_xinyangjie;
pub(crate) use resolve_xinyangjie::*;
mod resolve_xiongshu;
pub(crate) use resolve_xiongshu::*;
