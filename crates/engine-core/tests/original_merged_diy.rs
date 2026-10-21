//! O2 全量合并批次 1（2026-10-14）：家族 diy 的 16 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_diy/original_diy_batch1.rs"]
mod original_diy_batch1;

#[path = "original_merged_diy/original_diy_junkjizhi.rs"]
mod original_diy_junkjizhi;

#[path = "original_merged_diy/original_diy_junkrende.rs"]
mod original_diy_junkrende;

#[path = "original_merged_diy/original_diy_nsdingzhou.rs"]
mod original_diy_nsdingzhou;

#[path = "original_merged_diy/original_diy_nsduijue.rs"]
mod original_diy_nsduijue;

#[path = "original_merged_diy/original_diy_nsdunxing.rs"]
mod original_diy_nsdunxing;

#[path = "original_merged_diy/original_diy_nsfuge.rs"]
mod original_diy_nsfuge;

#[path = "original_merged_diy/original_diy_nshuaishuang.rs"]
mod original_diy_nshuaishuang;

#[path = "original_merged_diy/original_diy_nsjihui.rs"]
mod original_diy_nsjihui;

#[path = "original_merged_diy/original_diy_nsnongquan.rs"]
mod original_diy_nsnongquan;

#[path = "original_merged_diy/original_diy_nsqiyue.rs"]
mod original_diy_nsqiyue;

#[path = "original_merged_diy/original_diy_nsshijun.rs"]
mod original_diy_nsshijun;

#[path = "original_merged_diy/original_diy_nstianji.rs"]
mod original_diy_nstianji;

#[path = "original_merged_diy/original_diy_nswangfeng.rs"]
mod original_diy_nswangfeng;

#[path = "original_merged_diy/original_diy_nsyuanchou.rs"]
mod original_diy_nsyuanchou;

#[path = "original_merged_diy/original_diy_tiangong2.rs"]
mod original_diy_tiangong2;
