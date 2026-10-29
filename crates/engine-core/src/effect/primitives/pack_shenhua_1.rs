//! Effect primitive implementations — shenhua 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// Custom-continuation marker for ABI 471 `resolve_spfengyin`.
pub(crate) const RESOLVE_SPFENGYIN_CUSTOM_CHOICE: u32 = 0xDCFC_4710;

/// Custom-continuation marker for ABI 472 `resolve_xinfu_lianpian`.
pub(crate) const RESOLVE_XINFU_LIANPIAN_CUSTOM_CHOICE: u32 = 0xDCFD_4720;

/// Custom-continuation marker for ABI 473 `resolve_gongjian`.
pub(crate) const RESOLVE_GONGJIAN_CUSTOM_CHOICE: u32 = 0xDCFE_4730;

/// Custom-continuation marker for ABI 474 `resolve_xinfu_qinguo`.
pub(crate) const RESOLVE_XINFU_QINGUO_CUSTOM_CHOICE: u32 = 0xDCFF_4740;

/// Custom-continuation marker for ABI 475 `resolve_dcmanzhi`.
pub(crate) const RESOLVE_DCMANZHI_CUSTOM_CHOICE: u32 = 0xDCD0_4750;

/// Custom-continuation marker for ABI 476 `resolve_xinxingluan`.
pub(crate) const RESOLVE_XINXINGLUAN_CUSTOM_CHOICE: u32 = 0xDCD1_4760;

/// Custom-continuation marker for ABI 477 `resolve_dckuizhen`.
pub(crate) const RESOLVE_DCKUIZHEN_CUSTOM_CHOICE: u32 = 0xDCD3_4770;

/// Custom-continuation marker for ABI 478 `resolve_nsdingzhou`.
pub(crate) const RESOLVE_NSDINGZHOU_CUSTOM_CHOICE: u32 = 0xDCD4_4780;

/// Custom-continuation marker for ABI 479 `resolve_sxrmlanjiao`.
pub(crate) const RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE: u32 = 0xDCD5_4790;

/// Custom-continuation marker for ABI 480 `resolve_rekuanshi`.
pub(crate) const RESOLVE_REKUANSHI_CUSTOM_CHOICE: u32 = 0xDCD6_4800;

/// Custom-continuation marker for ABI 481 `resolve_dcmoyu`.
pub(crate) const RESOLVE_DCMOYU_CUSTOM_CHOICE: u32 = 0xDCD7_4810;

/// Custom-continuation marker for ABI 482 `resolve_xizhen`.
pub(crate) const RESOLVE_XIZHEN_CUSTOM_CHOICE: u32 = 0xDCD8_4820;

/// Custom-continuation marker for ABI 483 `resolve_xuxie`.
pub(crate) const RESOLVE_XUXIE_CUSTOM_CHOICE: u32 = 0xDCD9_4830;

/// Custom-continuation marker for ABI 484 `resolve_dcchiying`.
pub(crate) const RESOLVE_DCCHIYING_CUSTOM_CHOICE: u32 = 0xDCDA_4840;

/// Custom-continuation marker for ABI 485 `resolve_xinkuangfu`.
pub(crate) const RESOLVE_XINKUANGFU_CUSTOM_CHOICE: u32 = 0xDCDB_4850;

/// Custom-continuation marker for ABI 486 `resolve_dcxiangshu`.
pub(crate) const RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE: u32 = 0xDCDC_4860;

/// 双雄 recorded judgment-color mark on the owner (count: 1 = red, 2 = black). ABI 511.
pub(crate) const SHUANGXIONG_COLOR_MARK: u32 = 6_0000_0511;

/// 双雄·视为：本回合由 `resolve_shuangxiong` 授予的临时视为技能定义。ABI 511.
pub(crate) const SHUANGXIONG_VIEW_AS_DEF: u32 = 1761_4351_31;

/// Custom-continuation marker for ABI 512 `resolve_zhijian`.
pub(crate) const RESOLVE_ZHIJIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5120;

/// Custom-continuation marker for ABI 513 `resolve_guzheng`.
pub(crate) const RESOLVE_GUZHENG_CUSTOM_CHOICE: u32 = 0xDCDC_5130;
/// 制霸 拼点前拒绝/拼点后取牌 自定义续体标记。ABI 523。
pub(crate) const RESOLVE_ZHIBA_CUSTOM_CHOICE: u32 = 0xDCDC_5230;
/// 挑衅（姜维）自定义续体标签。ABI 537。
pub(crate) const RESOLVE_TIAOXIN_CUSTOM_CHOICE: u32 = 0xDCDC_5370;
/// 征荣（毌丘俭）自定义续体标签。ABI 545。
pub(crate) const RESOLVE_ZHENGRONG_CUSTOM_CHOICE: u32 = 0xDCDC_5450;
/// 鸿举（毌丘俭）自定义续体标签。ABI 546。
pub(crate) const RESOLVE_HONGJU_CUSTOM_CHOICE: u32 = 0xDCDC_5460;
/// 清俭（毌丘俭）自定义续体标签。ABI 547。
pub(crate) const RESOLVE_QINGJIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5470;
/// 雷击（张角）自定义续体标签。ABI 549。
pub(crate) const RESOLVE_RELEIJI_CUSTOM_CHOICE: u32 = 0xDCDC_5490;
/// 黄天（张角）自定义续体标签。ABI 551。
pub(crate) const RESOLVE_HUANGTIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5510;
/// 好施（鲁肃）自定义续体标签。ABI 555。
pub(crate) const RESOLVE_HAOSHI_CUSTOM_CHOICE: u32 = 0xDCDC_5550;
/// 缔盟（鲁肃）自定义续体标签。ABI 556。
pub(crate) const RESOLVE_DIMENG_CUSTOM_CHOICE: u32 = 0xDCDC_5560;
pub(crate) const RESOLVE_QIAOBIAN_DISCARD: u32 = 0xDCDC_5590;
pub(crate) const RESOLVE_QIAOBIAN_DRAW_TARGETS: u32 = 0xDCDC_5591;
pub(crate) const RESOLVE_QIAOBIAN_DRAW_PICK: u32 = 0xDCDC_5592;
pub(crate) const RESOLVE_QIAOBIAN_PLAY_CARD: u32 = 0xDCDC_5593;
pub(crate) const RESOLVE_QIAOBIAN_PLAY_TARGET: u32 = 0xDCDC_5594;
pub(crate) const RESOLVE_XIANGLE_CUSTOM_CHOICE: u32 = 0xDCDC_5600;
pub(crate) const RESOLVE_FANGQUAN_DISCARD: u32 = 0xDCDC_5601;
pub(crate) const RESOLVE_FANGQUAN_TARGET: u32 = 0xDCDC_5602;
pub(crate) const RESOLVE_ZUILUN_PICK: u32 = 0xDCDC_5630;
pub(crate) const RESOLVE_ZUILUN_TARGET: u32 = 0xDCDC_5631;
pub(crate) const RESOLVE_JUZHAN_GAIN: u32 = 0xDCDC_5710;
pub(crate) const RESOLVE_QIZHI_TARGET: u32 = 0xDCDC_5730;
pub(crate) const RESOLVE_QIZHI_CARD: u32 = 0xDCDC_5731;
pub(crate) const RESOLVE_JINQU_DISCARD: u32 = 0xDCDC_5732;
pub(crate) const RESOLVE_JIANXIANG_TARGET: u32 = 0xDCDC_5740;
pub(crate) const RESOLVE_SHENSHI_GIVE: u32 = 0xDCDC_5741;
pub(crate) const RESOLVE_SHENSHI_TARGET: u32 = 0xDCDC_5742;
pub(crate) const RESOLVE_SHENSHI_REWARD: u32 = 0xDCDC_5743;
pub(crate) const RESOLVE_MINGREN_PLACE: u32 = 0xDCDC_5760;
pub(crate) const RESOLVE_MINGREN_REPLACE: u32 = 0xDCDC_5761;
pub(crate) const RESOLVE_ZHENLIANG_DISCARD: u32 = 0xDCDC_5762;
pub(crate) const RESOLVE_ZHENLIANG_DAMAGE: u32 = 0xDCDC_5763;
pub(crate) const RESOLVE_ZHENLIANG_DRAW: u32 = 0xDCDC_5764;
pub(crate) const RESOLVE_KUIZHU_DRAW: u32 = 0xDCDC_5770;
pub(crate) const RESOLVE_KUIZHU_DAMAGE: u32 = 0xDCDC_5771;
pub(crate) const RESOLVE_ZHIZHENG_CARD: u32 = 0xDCDC_5772;
pub(crate) const RESOLVE_LIANGYIN_DRAW: u32 = 0xDCDC_5790;
pub(crate) const RESOLVE_LIANGYIN_RECOVER: u32 = 0xDCDC_5791;
pub(crate) const RESOLVE_LIANGYIN_DISCARD: u32 = 0xDCDC_5792;
pub(crate) const RESOLVE_LIANGYIN_TARGET: u32 = 0xDCDC_5793;
pub(crate) const RESOLVE_LIANGYIN_COMMIT: u32 = 0xDCDC_5794;
pub(crate) const RESOLVE_KONGSHENG_PLACE: u32 = 0xDCDC_5795;
pub(crate) const RESOLVE_KONGSHENG_TARGET: u32 = 0xDCDC_5796;
/// 从俭 目标/选牌续体标记。ABI 524。
pub(crate) const RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE: u32 = 0xDCDC_5240;
pub(crate) const RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE: u32 = 0xDCDC_5260;
pub(crate) const RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE: u32 = 0xDCDC_5280;
/// 庸肆弃牌阶段状态（既有 drlt_yongsi1 定义）。ABI 527。
pub(crate) const DRLT_YONGSI1_DEF: u32 = 3870932875;
/// 飞军 目标首次被选标记（置于目标上，永久）。ABI 526。
pub(crate) const FEIJUN_TARGET_MARK: u32 = 6_0000_0526;
/// 魂姿（hunzi）稳定技能定义。ABI 523 觉醒检查。
pub(crate) const HUNZI_DEF: u32 = 4151424761;
/// 界魂姿（rehunzi）稳定技能定义。ABI 523 觉醒检查。
pub(crate) const REHUNZI_DEF: u32 = 3543073615;

/// 相术 secret-number mark on the owner (count = the chosen number).
pub(crate) const DCXIANG_SHU_NUMBER_MARK: u32 = 6_0000_0005;

pub(crate) const RESOLVE_DCYINGTU_CUSTOM_CHOICE: u32 = 0xDCDA_4870;

/// Custom-continuation marker for ABI 410 霆威选择该杀目标阶段（payload[0]）。
pub(crate) const RESOLVE_TINGWEI_SELECT: u32 = 0xDCDA_4100;
/// Custom-continuation marker for ABI 410 霆威目标四选一阶段（payload[0]）。
pub(crate) const RESOLVE_TINGWEI_OPTION: u32 = 0xDCDA_4101;
/// Custom-continuation marker for ABI 411 新名士强制弃牌结算（payload[0]）。
pub(crate) const RESOLVE_XINMINGSHI_CUSTOM_CHOICE: u32 = 0xDCDA_4110;
/// Custom-continuation marker for ABI 412 殉义选择义目标（payload[0]）。
pub(crate) const RESOLVE_XUNYI_SELECT: u32 = 0xDCDA_4120;
/// Custom-continuation marker for ABI 414 恶荐二选一（payload[0]）。
pub(crate) const RESOLVE_EJIAN_CHOICE: u32 = 0xDCDA_4140;
/// Custom-continuation marker for ABI 417 舍医选牌防伤（payload[0]）。
pub(crate) const RESOLVE_SHEYI_CHOICE: u32 = 0xDCDA_4170;
/// Custom-continuation marker for ABI 583 杨彪「举讹」确认/选目标（payload[0]）。
pub(crate) const RESOLVE_DCJUE_CHOICE: u32 = 0xDCDA_5830;
/// Custom-continuation marker for ABI 584 阮籍「酒遁」分支（payload[0]）。
pub(crate) const RESOLVE_RUANJI_CHOICE: u32 = 0xDCDA_5840;
pub(crate) const RESOLVE_ZHENGBI_CHOICE: u32 = 0xDCDA_5850;
pub(crate) const RESOLVE_QIAOJIAN_CHOICE: u32 = 0xDCDA_5860;
pub(crate) const RESOLVE_XICHA_CHOICE: u32 = 0xDCDA_5861;
pub(crate) const RESOLVE_HUZUN_CHOICE: u32 = 0xDCDC_5870;
pub(crate) const RESOLVE_CHENGLVE_DISCARD: u32 = 0xDCDC_5890;
pub(crate) const RESOLVE_SHICAI_PLACE: u32 = 0xDCDC_5891;
/// 踞襄（张绣）装备牌选择续体标记。ABI 590。
pub const RESOLVE_TWJUXIANG_CARD: u32 = 0xDCDC_5900;
/// 旧不屈（旧周泰）恢复移除创牌续体标记。ABI 591。
pub const RESOLVE_GZBUQU_RECOVER_CARD: u32 = 0xDCDC_5910;
pub(crate) const RESOLVE_ZHUGEMENGXUE_CHOICE: u32 = 0xDCDC_5880;
pub(crate) const RESOLVE_BAILINGYUN_CHOICE: u32 = 0xDCDC_5930;
pub(crate) const RESOLVE_DCDICHOU_CHOICE: u32 = 0xDCDC_5940;
pub(crate) const RESOLVE_DCDUFENG_CHOICE: u32 = 0xDCDC_5950;
/// Custom-continuation marker for ABI 420 曲迪三选一（payload[0]）。
pub(crate) const RESOLVE_DBQUEDI_CHOICE: u32 = 0xDCDA_4200;
/// Custom-continuation marker for ABI 421 隐据二选一（payload[0]）。
pub(crate) const RESOLVE_SPYINJU_CHOICE: u32 = 0xDCDA_4210;

pub(crate) const POTTUNTIAN_CUSTOM_CHOICE: u32 = 0x5054_5431;
pub(crate) const POTJIXI_CUSTOM_CHOICE: u32 = 0x504a_5431;
pub(crate) const POTCEDUAN_CUSTOM_CHOICE: u32 = 0x5043_5441;
pub(crate) const POTTAISHICI_CUSTOM_CHOICE: u32 = 0x5054_5343;
pub(crate) const MBZHONGHUI_CUSTOM_CHOICE: u32 = 0x4d42_5a48;
/// 势辛宪英诫节/清识自定义续体标签（Codex ABI 554 在建占位）。
pub(crate) const POTJIEJIE_CUSTOM_CHOICE: u32 = 0x504a_4a45;
pub(crate) const POTQINGSHI_CUSTOM_CHOICE: u32 = 0x5051_5348;
pub(crate) const POTWEIYAN_CUSTOM_CHOICE: u32 = 0x5057_594e;
pub(crate) const POTZHOUYU_CUSTOM_CHOICE: u32 = 0x505a_5955;
pub(crate) const POTLVYI_CUSTOM_CHOICE: u32 = 0x504c_5659;
pub(crate) const POTCAOSHUANG_CUSTOM_CHOICE: u32 = 0x5043_5348;

pub(crate) const POTWEIYAN_KUANGGU_DEF: u32 = 4_200_000_562;
pub(crate) const POTWEIYAN_ZHUANGSHI_DEF: u32 = 4_200_000_560;
pub(crate) const POTWEIYAN_KUNFEN_DEF: u32 = 3_536_332_483;
pub(crate) const POTWEIYAN_DIRECT_DEF: u32 = 4_200_000_563;
pub(crate) const POTWEIYAN_LIMIT_DEF: u32 = 4_200_000_564;
pub(crate) const POTWEIYAN_KUANGGU_LIMIT_DEF: u32 = 4_200_000_565;

mod reveal_hand_to_player;
pub(crate) use reveal_hand_to_player::*;

mod resolve_zhiba;
pub(crate) use resolve_zhiba::*;
mod guanxing_top_by_falu_marks;
pub(crate) use guanxing_top_by_falu_marks::*;
mod resolve_haoshi;
pub(crate) use resolve_haoshi::*;
