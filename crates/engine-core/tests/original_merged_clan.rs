//! O2 全量合并批次 1（2026-10-14）：家族 clan 的 12 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_clan/original_clan_clan_family_batch.rs"]
mod original_clan_clan_family_batch;

#[path = "original_merged_clan/original_clan_clanbaichu.rs"]
mod original_clan_clanbaichu;

#[path = "original_merged_clan/original_clan_clanjianbai.rs"]
mod original_clan_clanjianbai;

#[path = "original_merged_clan/original_clan_clanjianzhi.rs"]
mod original_clan_clanjianzhi;

#[path = "original_merged_clan/original_clan_clanqieyi.rs"]
mod original_clan_clanqieyi;

#[path = "original_merged_clan/original_clan_clanqingjue.rs"]
mod original_clan_clanqingjue;

#[path = "original_merged_clan/original_clan_clanshenjun.rs"]
mod original_clan_clanshenjun;

#[path = "original_merged_clan/original_clan_clanxsyingxiang.rs"]
mod original_clan_clanxsyingxiang;

#[path = "original_merged_clan/original_clan_clanyuzhi.rs"]
mod original_clan_clanyuzhi;

#[path = "original_merged_clan/original_clan_clanzhongliu.rs"]
mod original_clan_clanzhongliu;

#[path = "original_merged_clan/original_clan_draw_to.rs"]
mod original_clan_draw_to;

#[path = "original_merged_clan/original_clan_name_length_batch.rs"]
mod original_clan_name_length_batch;
