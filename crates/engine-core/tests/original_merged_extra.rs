//! O2 全量合并批次 1（2026-10-14）：家族 extra 的 10 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_extra/original_extra_qixing.rs"]
mod original_extra_qixing;

#[path = "original_merged_extra/original_extra_relonghun.rs"]
mod original_extra_relonghun;

#[path = "original_merged_extra/original_extra_shen_ganning.rs"]
mod original_extra_shen_ganning;

#[path = "original_merged_extra/original_extra_shen_liubei.rs"]
mod original_extra_shen_liubei;

#[path = "original_merged_extra/original_extra_shen_luxun.rs"]
mod original_extra_shen_luxun;

#[path = "original_merged_extra/original_extra_shen_lvmeng.rs"]
mod original_extra_shen_lvmeng;

#[path = "original_merged_extra/original_extra_shen_simayi.rs"]
mod original_extra_shen_simayi;

#[path = "original_merged_extra/original_extra_shen_zhangliao.rs"]
mod original_extra_shen_zhangliao;

#[path = "original_merged_extra/original_extra_shen_zhouyu.rs"]
mod original_extra_shen_zhouyu;

#[path = "original_merged_extra/original_extra_wuhun.rs"]
mod original_extra_wuhun;
