//! O2 全量合并批次 1（2026-10-14）：家族 huicui 的 17 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_huicui/original_huicui_1_11_5_dcqiansu_chijian.rs"]
mod original_huicui_1_11_5_dcqiansu_chijian;

#[path = "original_merged_huicui/original_huicui_chenjian.rs"]
mod original_huicui_chenjian;

#[path = "original_merged_huicui/original_huicui_dcchiying.rs"]
mod original_huicui_dcchiying;

#[path = "original_merged_huicui/original_huicui_dcgangu.rs"]
mod original_huicui_dcgangu;

#[path = "original_merged_huicui/original_huicui_dckuizhen.rs"]
mod original_huicui_dckuizhen;

#[path = "original_merged_huicui/original_huicui_dcmanzhi.rs"]
mod original_huicui_dcmanzhi;

#[path = "original_merged_huicui/original_huicui_dcmoyu.rs"]
mod original_huicui_dcmoyu;

#[path = "original_merged_huicui/original_huicui_dcshoucheng.rs"]
mod original_huicui_dcshoucheng;

#[path = "original_merged_huicui/original_huicui_dcxiangshu.rs"]
mod original_huicui_dcxiangshu;

#[path = "original_merged_huicui/original_huicui_dcyingtu.rs"]
mod original_huicui_dcyingtu;

#[path = "original_merged_huicui/original_huicui_dczhongji.rs"]
mod original_huicui_dczhongji;

#[path = "original_merged_huicui/original_huicui_lianzhou.rs"]
mod original_huicui_lianzhou;

#[path = "original_merged_huicui/original_huicui_refuyuan.rs"]
mod original_huicui_refuyuan;

#[path = "original_merged_huicui/original_huicui_rekuanshi.rs"]
mod original_huicui_rekuanshi;

#[path = "original_merged_huicui/original_huicui_xinkuangfu.rs"]
mod original_huicui_xinkuangfu;

#[path = "original_merged_huicui/original_huicui_xizhen.rs"]
mod original_huicui_xizhen;

#[path = "original_merged_huicui/original_huicui_xuxie.rs"]
mod original_huicui_xuxie;
