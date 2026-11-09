//! Batch 26 (Decision 34 Complete 37-Source In-Repo Closure): constant tables for
//! the remaining 9 sources (`U01`, `U05`, `L08`, `L14`, `L19`, `L20`, `L24`, `L25`, `L27`)
//! plus `U07` (`jianbi`, `choudu`).

use engine_native::catalog::{Faction, Suit};
use engine_native::content::standard::Gender;

/// `U01` (`d3-upstream-noname`): all 31 upstream `ddd_*` generals from
/// `archive/upstream/noname/apps/core/extension/3D精选/character/character.js`.
pub const U01_DDD_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 31] = [
    ("ddd_handang", Faction::Wu, Gender::Male, 4, &["dddxianxi"]),
    ("ddd_wuzhi", Faction::Wei, Gender::Male, 3, &["dddlingyong", "dddxuxiao"]),
    ("ddd_xujing", Faction::Shu, Gender::Male, 3, &["dddxuyu", "dddshijian"]),
    ("ddd_caomao", Faction::Wei, Gender::Male, 3, &["dddtaisi", "dddquche", "dddqianlong"]),
    ("ddd_xinxianying", Faction::Wei, Gender::Female, 3, &["ddddongcha", "dddzhijie"]),
    ("ddd_xianglang", Faction::Shu, Gender::Male, 3, &["dddqiahua", "dddfusi", "dddtuoji"]),
    ("ddd_yujin", Faction::Wei, Gender::Male, 4, &["dddzhengjun"]),
    ("ddd_liuye", Faction::Wei, Gender::Male, 3, &["dddchashi", "dddqice"]),
    ("ddd_baosanniang", Faction::Shu, Gender::Female, 3, &["dddzhilian", "dddjijian"]),
    ("ddd_zhenji", Faction::Wei, Gender::Female, 3, &["dddmiaoxing", "dddfushi"]),
    ("ddd_zhaoang", Faction::Wei, Gender::Male, 4, &["dddfenji"]),
    ("ddd_zhouchu", Faction::Wu, Gender::Male, 5, &["dddxiaheng"]),
    ("ddd_liuba", Faction::Shu, Gender::Male, 3, &["dddfengzheng", "dddyulv"]),
    ("ddd_jianshuo", Faction::Qun, Gender::Male, 4, &["dddfenye", "dddshichao"]),
    ("ddd_guanning", Faction::Qun, Gender::Male, 3, &["dddyouxue", "dddchengjing"]),
    ("ddd_dingfeng", Faction::Wu, Gender::Male, 4, &["dddduanbing"]),
    ("ddd_caoshuang", Faction::Wei, Gender::Male, 5, &["dddzhuanshe", "dddweiqiu"]),
    ("ddd_xuelingyun", Faction::Wei, Gender::Female, 3, &["dddlianer", "dddanzhi"]),
    ("ddd_liuhong", Faction::Qun, Gender::Male, 4, &["dddshixing", "ddddanggu", "dddfuzong"]),
    ("ddd_xiahouxuan", Faction::Wei, Gender::Male, 3, &["dddlanghuai", "dddxuanlun"]),
    ("ddd_zhangkai", Faction::Qun, Gender::Male, 4, &["dddjiexing", "dddbailei"]),
    ("ddd_liangxi", Faction::Wei, Gender::Male, 4, &["dddtongyu"]),
    ("ddd_wangkanglvkai", Faction::Shu, Gender::Male, 4, &["dddbingjian"]),
    ("ddd_sunliang", Faction::Wu, Gender::Male, 3, &["ddddiedang", "dddanliu", "dddguiying"]),
    ("ddd_lie", Faction::Wu, Gender::Female, 3, &["dddyeshen", "dddqiaoduan"]),
    ("ddd_kebineng", Faction::Qun, Gender::Male, 4, &["dddxiaoxing", "dddlangzhi", "dddfuyi"]),
    ("ddd_qianzhao", Faction::Wei, Gender::Male, 4, &["dddyuanzhen", "dddzhishu"]),
    ("ddd_zhangmiao", Faction::Qun, Gender::Male, 4, &["dddxiaxing"]),
    ("ddd_zhangcheng", Faction::Wu, Gender::Male, 3, &["dddjuxian", "dddjungui"]),
    ("ddd_liuchong", Faction::Qun, Gender::Male, 4, &["dddjinggou", "dddmoyan"]),
    ("ddd_luoxian", Faction::Shu, Gender::Male, 4, &["dddshilie"]),
];

pub const U01_DDD_SKILLS: [&str; 64] = [
    "dddyuanzhen", "dddzhishu", "dddxiaxing", "dddjuxian", "dddjungui",
    "dddjinggou", "dddmoyan", "dddshilie", "dddyeshen", "dddqiaoduan",
    "ddddiedang", "dddanliu", "dddguiying", "dddbingjian", "dddtongyu",
    "dddzhilian", "dddjijian", "dddjijian_old", "dddzhengjun", "dddxianxi",
    "dddlingyong", "dddxuxiao", "dddxuyu", "dddshijian", "dddxuyu_old",
    "dddshijian_old", "dddtaisi", "dddquche", "dddqianlong", "ddddongcha",
    "dddzhijie", "dddqiahua", "dddxunxun", "dddfusi", "dddfusi_global",
    "dddtuoji", "dddchashi", "dddqice", "dddmiaoxing", "dddfushi",
    "dddfenji", "dddxiaheng", "dddfengzheng", "dddyulv", "dddyulv_old",
    "dddfenye", "dddshichao", "dddyouxue", "dddchengjing", "dddduanbing",
    "dddxiaoxing", "dddlangzhi", "dddfuyi", "dddzhuanshe", "dddweiqiu",
    "dddlianer", "dddanzhi", "dddshixing", "ddddanggu", "dddfuzong",
    "dddlanghuai", "dddxuanlun", "dddjiexing", "dddbailei",
];

pub const U05_SWD_REMAINING_SKILLS: [&str; 2] = ["duijue", "tanlin"];
pub const U05_YUNCHOU_REMAINING_SKILLS: [&str; 3] = ["ycndiyjianxiong", "xinfu_jingxie", "ycnxinfu_qiaosi"];
pub const U05_MTG_LAND_CARDS: [(&str, Suit, u8, &str); 2] = [
    ("mtg_duzhao", Suit::Spade, 1, "g_mtg_duzhao"),
    ("mtg_cangbaohaiwan", Suit::Heart, 1, "g_mtg_cangbaohaiwan"),
];
pub const U05_ZHENFA_CARDS: [(&str, Suit, u8, &str); 1] = [("pozhenjue", Suit::Club, 7, "g_pozhenjue")];

pub const U05_OW_SKILLS: [&str; 26] = [
    "lichang", "feiying", "maoding", "bingqiang", "lianpo",
    "zhanlong", "huoyu", "mianzhen", "aqianghua", "shanguang",
    "tiandan", "liudan", "shoujia", "zhongdun", "bfengshi",
    "woliu", "dianji", "qinru", "maichong", "yihun",
    "shouge", "tuijin", "zihui", "chongzhuang", "shanxian",
    "shanhui",
];

pub const U05_XIANJIAN_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 36] = [
    ("pal_lixiaoyao", Faction::Qun, Gender::Male, 4, &["tianjian", "xjyufeng"]),
    ("pal_zhaoliner", Faction::Wei, Gender::Female, 3, &["huimeng", "tianshe"]),
    ("pal_linyueru", Faction::Wei, Gender::Female, 3, &["guiyuan", "qijian"]),
    ("pal_anu", Faction::Wu, Gender::Female, 3, &["lingdi", "anwugu"]),
    ("pal_wangxiaohu", Faction::Qun, Gender::Male, 4, &["husha"]),
    ("pal_sumei", Faction::Shu, Gender::Female, 3, &["sheying", "dujiang", "huahu"]),
    ("pal_shenqishuang", Faction::Wei, Gender::Female, 3, &["qixia", "jianzhen", "binxin"]),
    ("pal_jingtian", Faction::Wu, Gender::Male, 3, &["sajin", "jtjubao"]),
    ("pal_xuejian", Faction::Shu, Gender::Female, 3, &["xshuangren", "shenmu", "duci"]),
    ("pal_longkui", Faction::Qun, Gender::Female, 3, &["fenxing", "diewu", "lingyu"]),
    ("pal_zixuan", Faction::Wei, Gender::Female, 3, &["shuiyun", "wangyou", "changnian"]),
    ("pal_changqing", Faction::Wei, Gender::Male, 4, &["luanjian", "ctianfu"]),
    ("pal_nangonghuang", Faction::Wei, Gender::Male, 3, &["zhaoyao", "sheling", "zhangmu"]),
    ("pal_wenhui", Faction::Shu, Gender::Female, 4, &["huxi", "longxiang"]),
    ("pal_wangpengxu", Faction::Shu, Gender::Female, 3, &["duxinshu", "feixu"]),
    ("pal_xingxuan", Faction::Wei, Gender::Male, 3, &["feizhua", "leiyu", "lingxue"]),
    ("pal_leiyuange", Faction::Shu, Gender::Male, 4, &["feng", "ya", "song"]),
    ("pal_yuntianhe", Faction::Wu, Gender::Male, 4, &["longxi", "zhuyue", "guanri"]),
    ("pal_hanlingsha", Faction::Shu, Gender::Female, 3, &["tannang", "tuoqiao"]),
    ("pal_liumengli", Faction::Wei, Gender::Female, 3, &["tianxian", "runxin", "xjzhimeng"]),
    ("pal_murongziying", Faction::Wei, Gender::Male, 4, &["xuanning", "poyun", "qianfang"]),
    ("pal_xuanxiao", Faction::Wei, Gender::Male, 4, &["xuanyan", "ningbin", "xfenxin"]),
    ("pal_jiangyunfan", Faction::Wei, Gender::Male, 4, &["xunying", "liefeng"]),
    ("pal_tangyurou", Faction::Shu, Gender::Female, 3, &["txianqu", "qiongguang"]),
    ("pal_longyou", Faction::Wei, Gender::Male, 4, &["yuexing", "minsha"]),
    ("pal_xiaoman", Faction::Shu, Gender::Female, 3, &["anwugu", "lingquan", "shenwu"]),
    ("pal_xiahoujinxuan", Faction::Shu, Gender::Male, 3, &["xuanmo", "danqing"]),
    ("pal_muchanglan", Faction::Wu, Gender::Female, 3, &["feixia", "lueying"]),
    ("pal_xia", Faction::Shu, Gender::Female, 3, &["xjzongyu", "fanling"]),
    ("pal_jiangcheng", Faction::Qun, Gender::Male, 4, &["yanzhan", "fenshi"]),
    ("pal_yuejinzhao", Faction::Wei, Gender::Male, 4, &["ywuhun", "xjyingfeng"]),
    ("pal_yueqi", Faction::Wei, Gender::Female, 3, &["tianwu", "liguang", "shiying"]),
    ("pal_mingxiu", Faction::Shu, Gender::Female, 3, &["linghuo", "guijin", "chengxin"]),
    ("pal_xianqing", Faction::Qun, Gender::Male, 4, &["xtanxi", "xiaoyue"]),
    ("pal_luozhaoyan", Faction::Shu, Gender::Female, 4, &["fenglue", "tanhua"]),
    ("pal_jushifang", Faction::Shu, Gender::Male, 3, &["yujia", "xiepan", "yanshi"]),
];

pub const U05_XIANJIAN_SKILLS: [&str; 26] = [
    "feizhua", "leiyu", "qixia", "jianzhen", "xjzongyu",
    "huxi", "qijian", "tuoqiao", "duxinshu", "tianjian",
    "xfenxin", "fenxing", "feixia", "lingdi", "qiongguang",
    "yanzhan", "longxi", "tianwu", "danqing", "dujiang",
    "xiaoyue", "tanhua", "xuanning", "ywuhun", "yujia",
    "yanshi",
];

pub const U07_WUDIAN_REMAINING_SKILLS: [&str; 2] = ["jianbi", "choudu"];

pub const L08_DECADE_SKILLS: [&str; 9] = [
    "_changeEquipExt", "_ui_auto_cheat", "_kill_framebg", "_rarity", "_tiesuo_ying",
    "_usecard", "_decadeUI_usecardBegin", "_discard", "_decadeUI_dieKillEffect",
];

pub const L08_DECADE_CHARACTERS: [&str; 9] = [
    "decade_caocao", "decade_liubei", "decade_sunquan", "decade_lvbu", "decade_diaochan",
    "decade_zhugeliang", "decade_zhouyu", "decade_simayi", "decade_zhaoyun",
];

pub const L14_SHOUSHA_SKILLS: [&str; 48] = [
    "_dieNoTuoguan", "_menuCheat", "_jindutiaoO", "_jindutiaoA", "_jindutiaoMB",
    "_jindutiaoKS", "_jindutiao_close", "_chupaiA", "_chupaiB", "_chupaiC",
    "_chupaiD", "_chupaiE", "_chupaiF", "_chupaiG", "_chupaiH",
    "_chupaiM", "_chupaiN", "_chupaiO", "_chupaiP", "_chupaiI",
    "_chupaiJ", "_chupaiK", "_chupaiL", "_chupaiMX", "_chupaiNX",
    "_skilltip_closeB", "_jd_ddxyA", "_jd_ddxyB", "_jd_ddxyC", "_jd_ddxyD",
    "_jd_ddxyF", "_jd_ddxyAll", "_jd_shiyongshanD", "_jd_ddxyE", "_jd_wjsw",
    "_jd_swxs", "_jd_dfsk", "_jd_hhks", "_jd_pdjd", "_jd_mpjd",
    "_jd_cpjd", "_jd_qpjd", "_jd_hhjs", "_jd_phaseAfter", "_jindutiao",
    "_jindutiaopl", "_wmkzSayChange", "_shuohuaSayWordphase",
];

pub const L14_SHOUSHA_CHARACTERS: [&str; 3] = ["sp_zhanghe", "sp_xiahoudun", "sp_ganning"];
pub const L14_SHOUSHA_CARDS: [&str; 1] = ["shousha_qiankunbagua"];

pub const L19_DOOMSDAY_CHARACTERS: [&str; 24] = [
    "jiangshifz", "jiangshinj", "vir_zombie", "vir_tyrant", "vir_licker",
    "vir_hunter", "vir_nemesis", "vir_wesker", "vir_birkin", "vir_spitter",
    "vir_charger", "vir_boomer", "vir_smoker", "vir_witch", "vir_tank",
    "vir_jockey", "vir_screamer", "vir_stalker", "vir_bloater", "vir_clicker",
    "vir_ratking", "vir_shambler", "vir_reaper", "vir_patient0",
];

pub const L19_DOOMSDAY_SKILLS: [&str; 66] = [
    "baozou", "shishi", "xunmeng", "zaibian", "ganran",
    "ganran2", "_jisuangailv", "_tuizhi", "_tuizhi2", "_jiangshi",
    "_jiangshi2", "_jiangshi3", "_jiangshi4", "_jiangshiTx", "_jiangshiTx2",
    "_jiangshiTx3", "_huzhu", "chanceIdentity", "chanceIdentity2", "chanceIdentity3",
    "chanceIdentity4", "chanceIdentity5", "chanceIdentity6", "chanceIdentity7", "chanceIdentity8",
    "chanceIdentity9", "chanceIdentity10", "chanceIdentity11", "chanceIdentity12", "chanceIdentity13",
    "chanceIdentity14", "chanceIdentity15", "chanceIdentity0", "_gainSkill", "_wujinmoshiRE",
    "wujinmoshiRE1", "wujinmoshiRE2", "_wujinmoshiDamage", "_wujinmoshiDamageE", "_wujinmoshiMaxHandCard",
    "_wujinmoshiMaxHandCardE", "_wujinmoshiDraw", "_wujinmoshiDrawE", "_wujinmoshiRecover", "_wujinmoshiRecoverE",
    "_keepideskill", "_levelskill", "_expskill", "_exp2skill", "_useskill",
    "_killskill", "_relifeskill", "_yuanqiskill", "_zhongzhubuff", "_jingchengadddie",
    "_zhongfandieskill", "_tamodeskill", "_zhudieskill", "_qunxiongzhengbaskill", "_guanmodeskill",
    "_guandieskill", "_guixinmodeskill", "_huanhaiskill", "_seadreammodeskill", "_dietime",
    "_anshaskill",
];

pub const L20_MARK_SUPPLEMENT_CHARACTERS: [&str; 12] = [
    "ms_huangzhong", "ms_weiyan", "ms_xiaoqiao", "ms_zhouyu", "ms_zhugeliang",
    "ms_pangtong", "ms_luxun", "ms_lvmeng", "ms_simayi", "ms_guojia",
    "ms_zhangliao", "ms_ganning",
];

pub const L20_MARK_SUPPLEMENT_CARDS: [&str; 3] = ["zhengsu_leijin", "zhengsu_mingzhi", "zhengsu_bianzhen"];

pub const L20_MARK_SUPPLEMENT_SKILLS: [&str; 75] = [
    "_xigua_feiyanbahu_", "_shiqihujia_", "zhengsu", "zhenyi_spade", "xinhongyan",
    "_guozhanChooseControlCHG", "diaohulishan", "_guozhan_marks", "xianqu_mark", "zhulianbihe_mark",
    "yinyang_mark", "yexinjia_mark", "_zhulianbihe_mark_tao", "_yinyang_mark_add", "_guozhan_diebiaoji_",
    "_guozhan_duiweibiaoji_gua_", "_guozhan_duiweibiaoji_gua_gua_", "_aozhan_judge", "_guozhanhalfchghp", "_xigua_xueliangfanbei_",
    "_xigua_mopaifanbei_", "_xigua_zhuwanzhongji_", "_shoushajisha_", "_shoushajisha_delete_", "_diankuangtulu_gua",
    "_wanjunqushou_gua", "_o_o_recovertrigger", "_o_o_miaoshouhuichun", "_o_o_yishugaochao", "_losehp_",
    "_mianshang_", "_player_baojiA", "_player_baojiB", "_player_baojiC", "_player_baojiD",
    "_player_baojiE", "_player_baojiF", "_player_baojiG", "_player_baojiH", "_jiubuff_",
    "_jiubuff_xiaoshi_", "_player_game_guagua", "_guozhan_quanjubiaoji_guagua", "_shiyongkapaitexiao_", "_shiyongkapaitexiao_xi_gua_",
    "_shanghaipaishiyongtexiao_", "_player_daojianfu_xxgg_fuzi_", "_binglintu", "_dysp", "xinfu_langxi",
    "tspowei", "qingyu", "mibei", "fz_new_rewusheng", "fz_gzpaoxiao",
    "fz_new_longdan", "fz_new_tieji", "fz_liegong", "fz_xinkuanggu", "rekuanshi",
    "kuanshi", "kuanshi2", "shouxi", "rechuhai", "xinfu_lingren",
    "sbtieji", "sbduanliang", "shouye", "mbmowang", "wengua",
    "wengua5", "zuoxing", "kurou", "_chongzhu", "ms_longmark",
];

pub const L24_SKIN_SWITCH_SKILLS: [&str; 23] = [
    "_tsx", "_gj", "_ts", "_playAudioToQueue", "_checkDcdChuChang",
    "_checkDcdShan", "_fix_yh", "_fix_phase_yh", "_pfqh_check_roundCount", "_check_die_yh",
    "_pfqh_reapply_saved_params", "_pfqh_reapply_on_init", "_pfqh_check_hp_change", "_pfqh_check_damage_times", "_pfqh_check_damage",
    "_pfqh_check_jisha", "_pfqh_check_changeGroup", "_qhlyChangeGroupSkin", "_pfqh_check_phaseBegin", "_pfqh_check_phaseEnd",
    "_qhly_autoc", "_qhly_addButton", "_skin_switch_hook",
];

pub const L25_ZUAN_CHARACTERS: [&str; 120] = [
    "ssr_luxun", "ssr_sunquan", "ssr_zhaoyun", "ssr_zhugeliang", "ssr_zhangliao",
    "ssr_ganning", "ssr_machao", "ssr_zhouyu", "ssr_liubei", "ssr_dianwei",
    "ssr_guojia", "ssr_caocao", "ssr_lvbu", "ssr_huangzhong", "ssr_xiahoudun",
    "ssr_huatuo", "ssr_yujin", "ssr_zuoci", "ssr_huaxiong", "ssr_huanggai",
    "ssr_diaochan", "ssr_yuanshu", "ssr_guanyu", "ssr_lvmeng", "ssr_gongsunzan",
    "ssr_daqiao", "ssr_xiaoqiao", "ssr_xuzhu", "ssr_zhangfei", "ssr_zhenji",
    "ssr_sunshangxiang", "ssr_simayi", "ssr_yiji", "ssr_huangyueying", "ssr_jiaxu",
    "ssr_caiwenji", "ssr_menghuo", "ssr_zhurong", "ssr_jiangwei", "ssr_sunce",
    "ssr_taishici", "ssr_zhoutai", "ssr_xiahouyuan", "ssr_caoren", "ssr_dengai",
    "ssr_dongzhuo", "ssr_yuji", "ssr_yuanshao", "ssr_xushu", "ssr_fazheng",
    "ssr_pangtong", "ssr_zhanghe", "ssr_re_lidian", "ssr_xuhuang", "ssr_sunjian",
    "ssr_lusu", "ssr_zhangzhang", "ssr_zhangjiao", "ssr_zhangbao", "ssr_zhangliang",
    "zuan_caocao", "zuan_liubei", "zuan_sunquan", "zuan_lvbu", "zuan_diaochan",
    "zuan_zhugeliang", "zuan_zhouyu", "zuan_simayi", "zuan_guojia", "zuan_xunyu",
    "zuan_zhangliao", "zuan_xiahoudun", "zuan_xiahouyuan", "zuan_xuzhu", "zuan_dianwei",
    "zuan_zhenji", "zuan_caopi", "zuan_caozhi", "zuan_caorui", "zuan_zhonghui",
    "zuan_dengai", "zuan_jiangwei", "zuan_zhaoyun", "zuan_guanyu", "zuan_zhangfei",
    "zuan_machao", "zuan_huangzhong", "zuan_weiyan", "zuan_pangtong", "zuan_huangyueying",
    "zuan_menghuo", "zuan_zhurong", "zuan_liushan", "zuan_fazheng", "zuan_xushu",
    "zuan_masu", "zuan_guanping", "zuan_zhoucang", "zuan_liaohua", "zuan_mazhong",
    "zuan_ganning", "zuan_lvmeng", "zuan_huanggai", "zuan_daqiao", "zuan_xiaoqiao",
    "zuan_luxun", "zuan_sunshangxiang", "zuan_sunjian", "zuan_sunce", "zuan_taishici",
    "zuan_diy01", "zuan_diy02", "zuan_diy03", "zuan_diy04", "zuan_diy05",
    "zuan_diy06", "zuan_diy07", "zuan_diy08", "zuan_diy09", "zuan_diy10",
];

pub const L25_ZUAN_SKILLS: [&str; 122] = [
    "g_du", "_player_xxgg_zhishixian_xxxxx_ggg", "_player_xxgg_zhishixian2_xxxxx_ggg", "_player_xxgg_zhishixian3_xxxxx_ggg", "_jszsx",
    "_zuanattacklinecheck", "_strengthencardskill", "_strengthenstandardskill", "_ssrnohuahensskill", "_ssrchangehpskill",
    "_diycharacterskill", "_emojiRebate1", "_emojiRebate2", "_zuan_damageSendEgg", "_zuan_sendWine",
    "_zuan_damageSendFlower", "_zuan_killSendFlower", "_extraphaseskill", "_extradieskill", "_extraturnoverskill",
    "_extradamageskill", "_extradlosehpskill", "_extraloseskill", "_extragainskill", "_extraloseCardskill",
    "_extracompareskill", "_extrauseskill", "_extrajudgeskill", "_extralinkskill", "_comparesayskill",
    "_equipuseskill", "_damsayskill", "_saveskill", "_dieskill", "_shanskill",
    "_useCardsayskill", "_phasebeginskill", "_phaseendskill", "_judgeskill", "_linkskill",
    "_zuanwashhandcard", "_newbiankuang", "_clearBackground", "_newBackground", "_newBackgroundUse",
    "_newBackgroundAtart", "_zuanboss", "_zuanRoundTX", "_newZuanPhaseTX", "_zuanphaseTX",
    "_helasisyskill", "_keepideskill", "_zuanidentityskill", "_zuanidentitychgneiskill", "_zuanidentitydieafterskill",
    "_zuanidentityshowskill", "_zuanidentitydyingskill", "_zuanidentitytodieskill", "_zuanidentitydamskill", "_zuanidentityrecskill",
    "_zuanidentityuseskill", "_yuanqiskill", "_anyeskill", "_taohuaskill", "_lunjianskill",
    "_jianghunskill", "_dietime", "_anshaskill", "_huanhaiskill", "_seadreammodeskill",
    "_thunderjudgemodeskill", "_thundermodeskill", "_guixinmodeskill", "_guanmodeskill", "_guandieskill",
    "_qunxiongzhengbaskill", "_zhongzhubuff", "_jingchengadddie", "_zhongfandieskill", "_tamodeskill",
    "_zhudieskill", "_relifeskill", "_levelskill", "_expskill", "_exp2skill",
    "_useskill", "_killskill", "_zuanSTARTsayskill", "_zuanUSECARDsayskill", "_zuanCHANGEHPsayskill",
    "_zuansaidskill", "_recoversay", "_damagesay", "_diesay", "_tosay",
    "_fromsay", "_selfsay", "_startgamesay", "_haoyunlai", "_basicjuedgeskill",
    "_goodluckjuedgeskill", "_bingleAddJudge", "_bingleLoseJudge", "_bingleBigScreen", "_bingleDying",
    "_binglediecls", "_bingletexiaoshandian", "_bingletexiaohuoshan", "_bingletexiaohongshui", "_bingletexiaofulei",
    "_bingletexiaole", "_bingletexiaobingliang", "_bingletexiao", "_thundertexiao", "_clearthunder",
    "_thundertexiao2", "_bingletexiaochangehp", "_bingletexiao1", "_bingletexiao2", "_diyskill",
    "wumou", "new_guixin",
];

pub const L27_SELECT_BEAUTIFY_SKILLS: [&str; 37] = [
    "baonvezhi", "baonvezhi_mark", "bolan", "clandianzhan", "dcxiaoxi",
    "dinghan", "drlt_duorui", "jinghe", "jishe", "kuangcai",
    "new_meibu", "new_zhixi", "new_zhixi_recover", "pro_drlt_duorui", "qimou",
    "quanji", "regushe", "rehuashen", "rehuashen_mark", "reqimou",
    "rexinsheng", "sbjianxiong", "sbtongye", "sbzhaxiang", "shouxi",
    "spyanjiao", "taoluan", "twfengji", "twharvestinori", "twxiafeng",
    "twxingzhui", "twzhouhu", "twzhouzu", "twzuhuo", "xiefang",
    "xinfu_xionghuo", "xingchong",
];
