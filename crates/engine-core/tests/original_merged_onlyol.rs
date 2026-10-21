//! O2 全量合并批次 1（2026-10-14）：家族 onlyol 的 9 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_onlyol/original_onlyol_oldangxian.rs"]
mod original_onlyol_oldangxian;

#[path = "original_merged_onlyol/original_onlyol_olfuli.rs"]
mod original_onlyol_olfuli;

#[path = "original_merged_onlyol/original_onlyol_oljiangchi.rs"]
mod original_onlyol_oljiangchi;

#[path = "original_merged_onlyol/original_onlyol_oljingce.rs"]
mod original_onlyol_oljingce;

#[path = "original_merged_onlyol/original_onlyol_oljuece.rs"]
mod original_onlyol_oljuece;

#[path = "original_merged_onlyol/original_onlyol_olqieting.rs"]
mod original_onlyol_olqieting;

#[path = "original_merged_onlyol/original_onlyol_olqingxi.rs"]
mod original_onlyol_olqingxi;

#[path = "original_merged_onlyol/original_onlyol_olrenxin.rs"]
mod original_onlyol_olrenxin;

#[path = "original_merged_onlyol/original_onlyol_qixian.rs"]
mod original_onlyol_qixian;
