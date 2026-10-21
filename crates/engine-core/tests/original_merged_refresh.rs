//! O2 全量合并批次 1（2026-10-14）：家族 refresh 的 50 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_refresh/original_refresh_botu.rs"]
mod original_refresh_botu;

#[path = "original_merged_refresh/original_refresh_dcqiaomeng.rs"]
mod original_refresh_dcqiaomeng;

#[path = "original_merged_refresh/original_refresh_dcshibei.rs"]
mod original_refresh_dcshibei;

#[path = "original_merged_refresh/original_refresh_decadejingce.rs"]
mod original_refresh_decadejingce;

#[path = "original_merged_refresh/original_refresh_decadezhenjun_yujin_yujin.rs"]
mod original_refresh_decadezhenjun_yujin_yujin;

#[path = "original_merged_refresh/original_refresh_fenwei.rs"]
mod original_refresh_fenwei;

#[path = "original_merged_refresh/original_refresh_jiezhong.rs"]
mod original_refresh_jiezhong;

#[path = "original_merged_refresh/original_refresh_new_reluoyi_re_xuzhu.rs"]
mod original_refresh_new_reluoyi_re_xuzhu;

#[path = "original_merged_refresh/original_refresh_new_retuxi.rs"]
mod original_refresh_new_retuxi;

#[path = "original_merged_refresh/original_refresh_new_rewusheng.rs"]
mod original_refresh_new_rewusheng;

#[path = "original_merged_refresh/original_refresh_new_reyiji.rs"]
mod original_refresh_new_reyiji;

#[path = "original_merged_refresh/original_refresh_new_yajiao_old_zhaoyun.rs"]
mod original_refresh_new_yajiao_old_zhaoyun;

#[path = "original_merged_refresh/original_refresh_olduanliang.rs"]
mod original_refresh_olduanliang;

#[path = "original_merged_refresh/original_refresh_oljieming.rs"]
mod original_refresh_oljieming;

#[path = "original_merged_refresh/original_refresh_ollongdan_sp_zhaoyun.rs"]
mod original_refresh_ollongdan_sp_zhaoyun;

#[path = "original_merged_refresh/original_refresh_oltishen.rs"]
mod original_refresh_oltishen;

#[path = "original_merged_refresh/original_refresh_olzhiji.rs"]
mod original_refresh_olzhiji;

#[path = "original_merged_refresh/original_refresh_qiaomeng.rs"]
mod original_refresh_qiaomeng;

#[path = "original_merged_refresh/original_refresh_qinxue_re_lvmeng.rs"]
mod original_refresh_qinxue_re_lvmeng;

#[path = "original_merged_refresh/original_refresh_re_caocao.rs"]
mod original_refresh_re_caocao;

#[path = "original_merged_refresh/original_refresh_re_taishici_hanzhan.rs"]
mod original_refresh_re_taishici_hanzhan;

#[path = "original_merged_refresh/original_refresh_re_zhangfei_olpaoxiao.rs"]
mod original_refresh_re_zhangfei_olpaoxiao;

#[path = "original_merged_refresh/original_refresh_reanguo_re_zhuzhi.rs"]
mod original_refresh_reanguo_re_zhuzhi;

#[path = "original_merged_refresh/original_refresh_rebeige_mobile_re_caiwenji.rs"]
mod original_refresh_rebeige_mobile_re_caiwenji;

#[path = "original_merged_refresh/original_refresh_refanjian.rs"]
mod original_refresh_refanjian;

#[path = "original_merged_refresh/original_refresh_refankui.rs"]
mod original_refresh_refankui;

#[path = "original_merged_refresh/original_refresh_regongao_rejuyi.rs"]
mod original_refresh_regongao_rejuyi;

#[path = "original_merged_refresh/original_refresh_reguanxing.rs"]
mod original_refresh_reguanxing;

#[path = "original_merged_refresh/original_refresh_reguicai.rs"]
mod original_refresh_reguicai;

#[path = "original_merged_refresh/original_refresh_rehuaiyi_re_gongsunyuan.rs"]
mod original_refresh_rehuaiyi_re_gongsunyuan;

#[path = "original_merged_refresh/original_refresh_rejianchu.rs"]
mod original_refresh_rejianchu;

#[path = "original_merged_refresh/original_refresh_relongyin.rs"]
mod original_refresh_relongyin;

#[path = "original_merged_refresh/original_refresh_reluanwu_reweimu.rs"]
mod original_refresh_reluanwu_reweimu;

#[path = "original_merged_refresh/original_refresh_reluoshen.rs"]
mod original_refresh_reluoshen;

#[path = "original_merged_refresh/original_refresh_remoshi_ol_huangzhong.rs"]
mod original_refresh_remoshi_ol_huangzhong;

#[path = "original_merged_refresh/original_refresh_repindi_dc_chenqun.rs"]
mod original_refresh_repindi_dc_chenqun;

#[path = "original_merged_refresh/original_refresh_repojun3.rs"]
mod original_refresh_repojun3;

#[path = "original_merged_refresh/original_refresh_repolu_mobile_re_sunjian.rs"]
mod original_refresh_repolu_mobile_re_sunjian;

#[path = "original_merged_refresh/original_refresh_reqiangxi_mobile_re_dianwei.rs"]
mod original_refresh_reqiangxi_mobile_re_dianwei;

#[path = "original_merged_refresh/original_refresh_reqimou.rs"]
mod original_refresh_reqimou;

#[path = "original_merged_refresh/original_refresh_retieji_re_machao.rs"]
mod original_refresh_retieji_re_machao;

#[path = "original_merged_refresh/original_refresh_retongxin.rs"]
mod original_refresh_retongxin;

#[path = "original_merged_refresh/original_refresh_rewansha.rs"]
mod original_refresh_rewansha;

#[path = "original_merged_refresh/original_refresh_rezhongyong.rs"]
mod original_refresh_rezhongyong;

#[path = "original_merged_refresh/original_refresh_shebian.rs"]
mod original_refresh_shebian;

#[path = "original_merged_refresh/original_refresh_shizhan.rs"]
mod original_refresh_shizhan;

#[path = "original_merged_refresh/original_refresh_view_as.rs"]
mod original_refresh_view_as;

#[path = "original_merged_refresh/original_refresh_xinjiangchi_re_caozhang.rs"]
mod original_refresh_xinjiangchi_re_caozhang;

#[path = "original_merged_refresh/original_refresh_yijue.rs"]
mod original_refresh_yijue;

#[path = "original_merged_refresh/original_refresh_zhaxiang_re_huanggai.rs"]
mod original_refresh_zhaxiang_re_huanggai;
