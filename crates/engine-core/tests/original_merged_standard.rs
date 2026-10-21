//! O2 全量合并批次 1（2026-10-14）：家族 standard 的 25 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_standard/original_standard_fanjian.rs"]
mod original_standard_fanjian;

#[path = "original_merged_standard/original_standard_fankui.rs"]
mod original_standard_fankui;

#[path = "original_merged_standard/original_standard_ganglie.rs"]
mod original_standard_ganglie;

#[path = "original_merged_standard/original_standard_hujia.rs"]
mod original_standard_hujia;

#[path = "original_merged_standard/original_standard_jianxiong.rs"]
mod original_standard_jianxiong;

#[path = "original_merged_standard/original_standard_jijiang.rs"]
mod original_standard_jijiang;

#[path = "original_merged_standard/original_standard_jijiu.rs"]
mod original_standard_jijiu;

#[path = "original_merged_standard/original_standard_jiuyuan.rs"]
mod original_standard_jiuyuan;

#[path = "original_merged_standard/original_standard_lijian.rs"]
mod original_standard_lijian;

#[path = "original_merged_standard/original_standard_liuli.rs"]
mod original_standard_liuli;

#[path = "original_merged_standard/original_standard_luoshen.rs"]
mod original_standard_luoshen;

#[path = "original_merged_standard/original_standard_new_jiangchi.rs"]
mod original_standard_new_jiangchi;

#[path = "original_merged_standard/original_standard_remaining.rs"]
mod original_standard_remaining;

#[path = "original_merged_standard/original_standard_rende.rs"]
mod original_standard_rende;

#[path = "original_merged_standard/original_standard_retongji.rs"]
mod original_standard_retongji;

#[path = "original_merged_standard/original_standard_rewangzun.rs"]
mod original_standard_rewangzun;

#[path = "original_merged_standard/original_standard_stdkuangfu.rs"]
mod original_standard_stdkuangfu;

#[path = "original_merged_standard/original_standard_stdshushen.rs"]
mod original_standard_stdshushen;

#[path = "original_merged_standard/original_standard_stdxiaoguo.rs"]
mod original_standard_stdxiaoguo;

#[path = "original_merged_standard/original_standard_tiandu.rs"]
mod original_standard_tiandu;

#[path = "original_merged_standard/original_standard_tieji.rs"]
mod original_standard_tieji;

#[path = "original_merged_standard/original_standard_tuxi.rs"]
mod original_standard_tuxi;

#[path = "original_merged_standard/original_standard_xf_yiji.rs"]
mod original_standard_xf_yiji;

#[path = "original_merged_standard/original_standard_yaowu.rs"]
mod original_standard_yaowu;

#[path = "original_merged_standard/original_standard_yiji.rs"]
mod original_standard_yiji;
