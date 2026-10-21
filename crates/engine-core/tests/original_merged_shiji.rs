//! O2 全量合并批次 1（2026-10-14）：家族 shiji 的 29 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_shiji/original_shiji_boming_ejian.rs"]
mod original_shiji_boming_ejian;

#[path = "original_merged_shiji/original_shiji_db_wenyang.rs"]
mod original_shiji_db_wenyang;

#[path = "original_merged_shiji/original_shiji_feiyi.rs"]
mod original_shiji_feiyi;

#[path = "original_merged_shiji/original_shiji_mbdiaodu.rs"]
mod original_shiji_mbdiaodu;

#[path = "original_merged_shiji/original_shiji_qingyu.rs"]
mod original_shiji_qingyu;

#[path = "original_merged_shiji/original_shiji_qinzheng.rs"]
mod original_shiji_qinzheng;

#[path = "original_merged_shiji/original_shiji_shameng.rs"]
mod original_shiji_shameng;

#[path = "original_merged_shiji/original_shiji_shen_sunce.rs"]
mod original_shiji_shen_sunce;

#[path = "original_merged_shiji/original_shiji_sheyi_tianyin.rs"]
mod original_shiji_sheyi_tianyin;

#[path = "original_merged_shiji/original_shiji_sp_bianfuren.rs"]
mod original_shiji_sp_bianfuren;

#[path = "original_merged_shiji/original_shiji_sp_cuiyan.rs"]
mod original_shiji_sp_cuiyan;

#[path = "original_merged_shiji/original_shiji_sp_jiangqing.rs"]
mod original_shiji_sp_jiangqing;

#[path = "original_merged_shiji/original_shiji_sp_xinpi.rs"]
mod original_shiji_sp_xinpi;

#[path = "original_merged_shiji/original_shiji_spfenming_chendong.rs"]
mod original_shiji_spfenming_chendong;

#[path = "original_merged_shiji/original_shiji_spjungong_gaolan.rs"]
mod original_shiji_spjungong_gaolan;

#[path = "original_merged_shiji/original_shiji_spmiewu.rs"]
mod original_shiji_spmiewu;

#[path = "original_merged_shiji/original_shiji_spwuku.rs"]
mod original_shiji_spwuku;

#[path = "original_merged_shiji/original_shiji_spxizhan_huaman.rs"]
mod original_shiji_spxizhan_huaman;

#[path = "original_merged_shiji/original_shiji_wujing.rs"]
mod original_shiji_wujing;

#[path = "original_merged_shiji/original_shiji_xianghai.rs"]
mod original_shiji_xianghai;

#[path = "original_merged_shiji/original_shiji_xinlirang_xinmingshi.rs"]
mod original_shiji_xinlirang_xinmingshi;

#[path = "original_merged_shiji/original_shiji_xinyangjie.rs"]
mod original_shiji_xinyangjie;

#[path = "original_merged_shiji/original_shiji_xuancun.rs"]
mod original_shiji_xuancun;

#[path = "original_merged_shiji/original_shiji_xunyi.rs"]
mod original_shiji_xunyi;

#[path = "original_merged_shiji/original_shiji_yiyong_shanxie.rs"]
mod original_shiji_yiyong_shanxie;

#[path = "original_merged_shiji/original_shiji_yuli_tingwei.rs"]
mod original_shiji_yuli_tingwei;

#[path = "original_merged_shiji/original_shiji_zhengsu.rs"]
mod original_shiji_zhengsu;

#[path = "original_merged_shiji/original_shiji_zhibian_yuyan.rs"]
mod original_shiji_zhibian_yuyan;

#[path = "original_merged_shiji/original_shiji_zjjuxiang.rs"]
mod original_shiji_zjjuxiang;
