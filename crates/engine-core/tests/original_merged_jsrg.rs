//! O2 全量合并批次 1（2026-10-14）：家族 jsrg 的 16 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_jsrg/original_jsrg_chengxu.rs"]
mod original_jsrg_chengxu;

#[path = "original_merged_jsrg/original_jsrg_city_build.rs"]
mod original_jsrg_city_build;

#[path = "original_merged_jsrg/original_jsrg_debate_initiator.rs"]
mod original_jsrg_debate_initiator;

#[path = "original_merged_jsrg/original_jsrg_debate_p1.rs"]
mod original_jsrg_debate_p1;

#[path = "original_merged_jsrg/original_jsrg_debate_p2.rs"]
mod original_jsrg_debate_p2;

#[path = "original_merged_jsrg/original_jsrg_debate_p3.rs"]
mod original_jsrg_debate_p3;

#[path = "original_merged_jsrg/original_jsrg_debate_targets.rs"]
mod original_jsrg_debate_targets;

#[path = "original_merged_jsrg/original_jsrg_jiebing_wudao.rs"]
mod original_jsrg_jiebing_wudao;

#[path = "original_merged_jsrg/original_jsrg_jsrgchiying_gaoxiang.rs"]
mod original_jsrg_jsrgchiying_gaoxiang;

#[path = "original_merged_jsrg/original_jsrg_juxia_jueyin_niluan.rs"]
mod original_jsrg_juxia_jueyin_niluan;

#[path = "original_merged_jsrg/original_jsrg_langan.rs"]
mod original_jsrg_langan;

#[path = "original_merged_jsrg/original_jsrg_lie_family.rs"]
mod original_jsrg_lie_family;

#[path = "original_merged_jsrg/original_jsrg_new_juezhi.rs"]
mod original_jsrg_new_juezhi;

#[path = "original_merged_jsrg/original_jsrg_zhaohan.rs"]
mod original_jsrg_zhaohan;

#[path = "original_merged_jsrg/original_jsrg_zhiheng_zhasi.rs"]
mod original_jsrg_zhiheng_zhasi;

#[path = "original_merged_jsrg/original_jsrg_zuwang.rs"]
mod original_jsrg_zuwang;
