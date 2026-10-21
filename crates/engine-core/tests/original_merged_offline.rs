//! O2 全量合并批次 1（2026-10-14）：家族 offline 的 69 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_offline/original_offline_chixin.rs"]
mod original_offline_chixin;

#[path = "original_merged_offline/original_offline_dragchengyuan.rs"]
mod original_offline_dragchengyuan;

#[path = "original_merged_offline/original_offline_dragtaiguan_pangde.rs"]
mod original_offline_dragtaiguan_pangde;

#[path = "original_merged_offline/original_offline_dragxueji.rs"]
mod original_offline_dragxueji;

#[path = "original_merged_offline/original_offline_eu_xiaoguo.rs"]
mod original_offline_eu_xiaoguo;

#[path = "original_merged_offline/original_offline_eu_zhangliao.rs"]
mod original_offline_eu_zhangliao;

#[path = "original_merged_offline/original_offline_gzjinfa.rs"]
mod original_offline_gzjinfa;

#[path = "original_merged_offline/original_offline_hm_qianjun.rs"]
mod original_offline_hm_qianjun;

#[path = "original_merged_offline/original_offline_jdlianhuan.rs"]
mod original_offline_jdlianhuan;

#[path = "original_merged_offline/original_offline_jdsbjiuyuan.rs"]
mod original_offline_jdsbjiuyuan;

#[path = "original_merged_offline/original_offline_jdsbtongye.rs"]
mod original_offline_jdsbtongye;

#[path = "original_merged_offline/original_offline_jdsbzhiheng.rs"]
mod original_offline_jdsbzhiheng;

#[path = "original_merged_offline/original_offline_jun_renwang.rs"]
mod original_offline_jun_renwang;

#[path = "original_merged_offline/original_offline_jun_zhaoshou.rs"]
mod original_offline_jun_zhaoshou;

#[path = "original_merged_offline/original_offline_jy_huanggai.rs"]
mod original_offline_jy_huanggai;

#[path = "original_merged_offline/original_offline_jylianhuan.rs"]
mod original_offline_jylianhuan;

#[path = "original_merged_offline/original_offline_jysuozhou.rs"]
mod original_offline_jysuozhou;

#[path = "original_merged_offline/original_offline_jytongbei_jy_caocao.rs"]
mod original_offline_jytongbei_jy_caocao;

#[path = "original_merged_offline/original_offline_jyyuhuo.rs"]
mod original_offline_jyyuhuo;

#[path = "original_merged_offline/original_offline_nscesuan.rs"]
mod original_offline_nscesuan;

#[path = "original_merged_offline/original_offline_nsfeixiong_ns_lijue.rs"]
mod original_offline_nsfeixiong_ns_lijue;

#[path = "original_merged_offline/original_offline_nsjianglie.rs"]
mod original_offline_nsjianglie;

#[path = "original_merged_offline/original_offline_nslulve.rs"]
mod original_offline_nslulve;

#[path = "original_merged_offline/original_offline_nsyangwu.rs"]
mod original_offline_nsyangwu;

#[path = "original_merged_offline/original_offline_pkwuku.rs"]
mod original_offline_pkwuku;

#[path = "original_merged_offline/original_offline_psdaohe.rs"]
mod original_offline_psdaohe;

#[path = "original_merged_offline/original_offline_pshoufa.rs"]
mod original_offline_pshoufa;

#[path = "original_merged_offline/original_offline_pshuoluan.rs"]
mod original_offline_pshuoluan;

#[path = "original_merged_offline/original_offline_psmoyan.rs"]
mod original_offline_psmoyan;

#[path = "original_merged_offline/original_offline_pszhonghun.rs"]
mod original_offline_pszhonghun;

#[path = "original_merged_offline/original_offline_residual_placeholders.rs"]
mod original_offline_residual_placeholders;

#[path = "original_merged_offline/original_offline_sclcongjian.rs"]
mod original_offline_sclcongjian;

#[path = "original_merged_offline/original_offline_scls_qinzheng.rs"]
mod original_offline_scls_qinzheng;

#[path = "original_merged_offline/original_offline_scls_zhanshen.rs"]
mod original_offline_scls_zhanshen;

#[path = "original_merged_offline/original_offline_sclwansha.rs"]
mod original_offline_sclwansha;

#[path = "original_merged_offline/original_offline_spfuluan.rs"]
mod original_offline_spfuluan;

#[path = "original_merged_offline/original_offline_tanhu.rs"]
mod original_offline_tanhu;

#[path = "original_merged_offline/original_offline_tanhu3.rs"]
mod original_offline_tanhu3;

#[path = "original_merged_offline/original_offline_tybenxiang.rs"]
mod original_offline_tybenxiang;

#[path = "original_merged_offline/original_offline_tydangxian.rs"]
mod original_offline_tydangxian;

#[path = "original_merged_offline/original_offline_tyxianshou.rs"]
mod original_offline_tyxianshou;

#[path = "original_merged_offline/original_offline_tyxiyu.rs"]
mod original_offline_tyxiyu;

#[path = "original_merged_offline/original_offline_tyyizhuang.rs"]
mod original_offline_tyyizhuang;

#[path = "original_merged_offline/original_offline_tyzhuan.rs"]
mod original_offline_tyzhuan;

#[path = "original_merged_offline/original_offline_vtbshuli.rs"]
mod original_offline_vtbshuli;

#[path = "original_merged_offline/original_offline_wn_dutan.rs"]
mod original_offline_wn_dutan;

#[path = "original_merged_offline/original_offline_wn_qifeng.rs"]
mod original_offline_wn_qifeng;

#[path = "original_merged_offline/original_offline_wxdlfangzhu.rs"]
mod original_offline_wxdlfangzhu;

#[path = "original_merged_offline/original_offline_wxdlqianqian.rs"]
mod original_offline_wxdlqianqian;

#[path = "original_merged_offline/original_offline_wxdlyishi.rs"]
mod original_offline_wxdlyishi;

#[path = "original_merged_offline/original_offline_xkjubian.rs"]
mod original_offline_xkjubian;

#[path = "original_merged_offline/original_offline_xktanlu_cuilian.rs"]
mod original_offline_xktanlu_cuilian;

#[path = "original_merged_offline/original_offline_yjluoshen.rs"]
mod original_offline_yjluoshen;

#[path = "original_merged_offline/original_offline_yjxuepin.rs"]
mod original_offline_yjxuepin;

#[path = "original_merged_offline/original_offline_yjzhenlve.rs"]
mod original_offline_yjzhenlve;

#[path = "original_merged_offline/original_offline_ylyg_hunzi.rs"]
mod original_offline_ylyg_hunzi;

#[path = "original_merged_offline/original_offline_ylyg_jiang.rs"]
mod original_offline_ylyg_jiang;

#[path = "original_merged_offline/original_offline_ylygqiangxi.rs"]
mod original_offline_ylygqiangxi;

#[path = "original_merged_offline/original_offline_yy_baimaxiaoqi_skill.rs"]
mod original_offline_yy_baimaxiaoqi_skill;

#[path = "original_merged_offline/original_offline_yyfuqi.rs"]
mod original_offline_yyfuqi;

#[path = "original_merged_offline/original_offline_yysudi.rs"]
mod original_offline_yysudi;

#[path = "original_merged_offline/original_offline_yyxuezhan.rs"]
mod original_offline_yyxuezhan;

#[path = "original_merged_offline/original_offline_zj_cuijian.rs"]
mod original_offline_zj_cuijian;

#[path = "original_merged_offline/original_offline_zj_duanbing.rs"]
mod original_offline_zj_duanbing;

#[path = "original_merged_offline/original_offline_zj_guluo.rs"]
mod original_offline_zj_guluo;

#[path = "original_merged_offline/original_offline_zjfuli.rs"]
mod original_offline_zjfuli;

#[path = "original_merged_offline/original_offline_zylianji.rs"]
mod original_offline_zylianji;

#[path = "original_merged_offline/original_offline_zymoucheng.rs"]
mod original_offline_zymoucheng;

#[path = "original_merged_offline/original_offline_zyshangyi.rs"]
mod original_offline_zyshangyi;
