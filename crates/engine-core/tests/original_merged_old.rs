//! O2 全量合并批次 1（2026-10-14）：家族 old 的 20 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_old/original_old_chegu.rs"]
mod original_old_chegu;

#[path = "original_merged_old/original_old_gaoshun_xianzhen.rs"]
mod original_old_gaoshun_xianzhen;

#[path = "original_merged_old/original_old_guhuo.rs"]
mod original_old_guhuo;

#[path = "original_merged_old/original_old_jianrou.rs"]
mod original_old_jianrou;

#[path = "original_merged_old/original_old_junkguixin.rs"]
mod original_old_junkguixin;

#[path = "original_merged_old/original_old_liubiao_oldzishou.rs"]
mod original_old_liubiao_oldzishou;

#[path = "original_merged_old/original_old_new_reyaowu.rs"]
mod original_old_new_reyaowu;

#[path = "original_merged_old/original_old_old_jijun.rs"]
mod original_old_old_jijun;

#[path = "original_merged_old/original_old_old_zuilun.rs"]
mod original_old_old_zuilun;

#[path = "original_merged_old/original_old_oldanxu.rs"]
mod original_old_oldanxu;

#[path = "original_merged_old/original_old_oldgongji.rs"]
mod original_old_oldgongji;

#[path = "original_merged_old/original_old_oldhuisheng.rs"]
mod original_old_oldhuisheng;

#[path = "original_merged_old/original_old_oldjiefan.rs"]
mod original_old_oldjiefan;

#[path = "original_merged_old/original_old_oldmingjian.rs"]
mod original_old_oldmingjian;

#[path = "original_merged_old/original_old_oldqinqing.rs"]
mod original_old_oldqinqing;

#[path = "original_merged_old/original_old_oldxuanfeng.rs"]
mod original_old_oldxuanfeng;

#[path = "original_merged_old/original_old_tianxiang.rs"]
mod original_old_tianxiang;

#[path = "original_merged_old/original_old_weiyan_kuanggu.rs"]
mod original_old_weiyan_kuanggu;

#[path = "original_merged_old/original_old_yao_yuanshu.rs"]
mod original_old_yao_yuanshu;

#[path = "original_merged_old/original_old_yj_y_xunxu.rs"]
mod original_old_yj_y_xunxu;
