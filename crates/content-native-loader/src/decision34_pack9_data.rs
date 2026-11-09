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

/// `U07` (`wudian-upstream-noname`): all 9 `wdb_*` generals from
/// `archive/upstream/noname/apps/core/extension/玩点论杀/character/character.js:5-62`.
pub const U07_WUDIAN_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 9] = [
    ("wdb_feishi", Faction::Shu, Gender::Male, 3, &["nsshuaiyan", "moshou"]),
    ("wdb_hanlong", Faction::Wei, Gender::Male, 4, &["siji", "ciqiu"]),
    ("wdb_liufu", Faction::Wei, Gender::Male, 3, &["zhucheng", "duoqi"]),
    ("wdb_liuyan", Faction::Qun, Gender::Male, 3, &["juedao", "geju"]),
    ("wdb_liuzan", Faction::Wu, Gender::Male, 4, &["kangyin"]),
    ("wdb_tianyu", Faction::Wei, Gender::Male, 4, &["chezhen", "youzhan"]),
    ("wdb_xizhenxihong", Faction::Shu, Gender::Male, 4, &["fuchou", "jinyan"]),
    ("wdb_zaozhirenjun", Faction::Wei, Gender::Male, 3, &["liangce", "jianbi", "diyjuntun"]),
    ("wdb_yangyi", Faction::Shu, Gender::Male, 3, &["choudu", "liduan"]),
];

/// `U07` (`wudian-upstream-noname`): all skills from `玩点论杀/character/skill.js` and `card/skill.js`.
pub const U07_WUDIAN_ALL_SKILLS: [&str; 30] = [
    "nsshuaiyan", "moshou", "siji", "ciqiu", "ciqiu_dying", "ciqiu_destroy",
    "zhucheng", "duoqi", "juedao", "geju", "kangyin", "kangyin2",
    "chezhen", "youzhan", "fuchou", "fuchou2", "fuchou3", "jinyan",
    "liangce", "jianbi", "diyjuntun", "diyjuntun2", "juntun", "juntun_global",
    "choudu", "liduan", "liduan2", "youdishenru", "g_youdishenru", "diaobingqianjiang",
];

/// `U08` (`hero-kill-upstream-noname`): all 43 `yxs_*` generals from
/// `archive/upstream/noname/apps/core/extension/英雄杀/character/character.js:3-287`.
pub const U08_YXS_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 43] = [
    ("yxs_qinqiong", Faction::Wei, Gender::Male, 4, &["yxs_fanji", "yxs_menshen"]),
    ("yxs_wuzetian", Faction::Wu, Gender::Female, 4, &["nvquan", "qiandu", "yxsweiyi"]),
    ("yxs_caocao", Faction::Wei, Gender::Male, 4, &["zhulu", "xieling"]),
    ("yxs_mozi", Faction::Qun, Gender::Male, 3, &["jieyong", "feigong", "jianai"]),
    ("yxs_bole", Faction::Wu, Gender::Male, 3, &["bolehuiyan", "xiangma"]),
    ("yxs_aijiyanhou", Faction::Qun, Gender::Female, 3, &["seyou", "sheshi"]),
    ("yxs_diaochan", Faction::Qun, Gender::Female, 3, &["fengyi", "wange"]),
    ("yxs_yangyuhuan", Faction::Wu, Gender::Female, 3, &["fengyan", "nichang"]),
    ("yxs_baosi", Faction::Wu, Gender::Female, 3, &["jieyin", "fenghuo"]),
    ("yxs_napolun", Faction::Wei, Gender::Male, 4, &["tongling", "fanpu"]),
    ("yxs_kaisa", Faction::Shu, Gender::Male, 4, &["ducai"]),
    ("yxs_zhuyuanzhang", Faction::Wu, Gender::Male, 4, &["qiangyun"]),
    ("yxs_jinke", Faction::Qun, Gender::Male, 3, &["cike", "qiangxi"]),
    ("yxs_libai", Faction::Qun, Gender::Male, 3, &["miaobi", "zhexian"]),
    ("yxs_luban", Faction::Wu, Gender::Male, 3, &["guifu", "lshengong"]),
    ("yxs_lvzhi", Faction::Shu, Gender::Female, 4, &["zhensha", "xumou"]),
    ("yxs_goujian", Faction::Wu, Gender::Male, 3, &["keji", "tuqiang"]),
    ("yxs_lishimin", Faction::Qun, Gender::Male, 4, &["kongju"]),
    ("yxs_huamulan", Faction::Shu, Gender::Female, 3, &["xiaoji", "yizhuang"]),
    ("yxs_luobinhan", Faction::Wu, Gender::Male, 4, &["xiadao", "sheshu", "lguiyin"]),
    ("yxs_chengjisihan", Faction::Qun, Gender::Male, 4, &["mashu", "qianglue"]),
    ("yxs_mingchenghuanghou", Faction::Shu, Gender::Female, 3, &["tiewan", "chajue"]),
    ("yxs_wangzhaojun", Faction::Wei, Gender::Female, 3, &["heqin", "wluoyan"]),
    ("yxs_luocheng", Faction::Wu, Gender::Male, 4, &["hanqiang", "biaoqi"]),
    ("yxs_direnjie", Faction::Wei, Gender::Male, 3, &["shentan", "kanpo"]),
    ("yxs_sunwu", Faction::Wu, Gender::Male, 3, &["bingsheng", "taolue"]),
    ("yxs_chengyaojin", Faction::Shu, Gender::Male, 4, &["sanbanfu"]),
    ("yxs_yujix", Faction::Shu, Gender::Female, 3, &["ysheshen", "yxschangnian"]),
    ("yxs_xiangyu", Faction::Shu, Gender::Male, 4, &["wushuang", "yxsciqiu"]),
    ("yxs_yingzheng", Faction::Qun, Gender::Male, 4, &["jianxiong", "batu"]),
    ("yxs_yuefei", Faction::Qun, Gender::Male, 4, &["longdan", "wumu"]),
    ("yxs_fuermosi", Faction::Wei, Gender::Male, 3, &["yanyi", "jiean"]),
    ("yxs_guiguzi", Faction::Qun, Gender::Male, 3, &["baihe", "yinyang", "xiushen"]),
    ("yxs_xiaoqiao", Faction::Wu, Gender::Female, 3, &["chujia", "zhijie"]),
    ("yxs_luzhishen", Faction::Wei, Gender::Male, 4, &["yxsdili", "kuangchan"]),
    ("yxs_zhaoyong", Faction::Shu, Gender::Male, 3, &["zyhufu", "hanbei"]),
    ("yxs_yangguang", Faction::Qun, Gender::Male, 3, &["shiqin", "yaoyi"]),
    ("yxs_tangbohu", Faction::Qun, Gender::Male, 3, &["luobi", "fengliu"]),
    ("yxs_zhangsanfeng", Faction::Wei, Gender::Male, 4, &["zbudao", "taiji"]),
    ("yxs_nandinggeer", Faction::Shu, Gender::Female, 3, &["huli", "xianqu", "yixin"]),
    ("yxs_weizhongxian", Faction::Qun, Gender::Male, 3, &["zhuxin", "wlianhuan"]),
    ("yxs_meixi", Faction::Shu, Gender::Female, 3, &["liebo", "yaoji"]),
    ("yxs_lanlinwang", Faction::Shu, Gender::Male, 4, &["guimian", "lyuxue"]),
];

/// `U08` (`hero-kill-upstream-noname`): all 107 skills from
/// `archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:5-3135`.
pub const U08_YXS_SKILLS: [&str; 107] = [
    "yxs_fanji", "yxs_fanji2", "yxs_menshen3", "yxs_menshen", "yxs_menshen2",
    "guimian", "lyuxue", "lyuxue2", "yaoji", "liebo",
    "zhuxin", "wlianhuan", "huli", "yixin", "xianqu",
    "zbudao", "zbudao2", "taiji", "fengliu", "luobi",
    "yaoyi", "shiqin", "zyhufu", "hanbei", "sheshu",
    "lguiyin", "tongyu", "zhijie", "yxsdili", "kuangchan",
    "chujia", "baihe", "yinyang", "xiushen", "jiean",
    "yanyi", "batu", "wumu", "ysheshen", "yxschangnian",
    "yxschangnian2", "yxsciqiu", "sanbanfu", "sanbanfu2", "sanbanfu3",
    "bingsheng", "taolue", "shentan", "hanqiang", "biaoqi",
    "wluoyan", "heqin", "heqin2", "heqin3", "chajue",
    "chajue2", "tiewan", "qianglue", "jimin", "xiadao",
    "lzhangyi", "yizhuang", "yizhuang2", "kongju", "tuqiang",
    "xumou", "zhensha", "ducai", "ducai2", "ducai3",
    "tongling", "fanpu", "fenghuo", "nichang", "nichang2",
    "fengyan", "fengyi", "wange", "wange2", "seyou",
    "sheshi", "bolehuiyan", "bolehuiyan2", "bolehuiyan3", "oldbolehuiyan",
    "oldbolehuiyan2", "xiangma", "yxsweiyi", "qiandu", "nvquan",
    "nvquan1", "nvquan2", "nvquan3", "feigong", "jianai",
    "jieyong", "jieyong2", "jieyong3", "zhulu", "xieling",
    "qiangyun", "cike", "miaobix", "miaobi", "zhexian",
    "guifu", "lshengong",
];

pub const L08_DECADE_SKILLS: [&str; 9] = [
    "_changeEquipExt", "_ui_auto_cheat", "_kill_framebg", "_rarity", "_tiesuo_ying",
    "_usecard", "_decadeUI_usecardBegin", "_discard", "_decadeUI_dieKillEffect",
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

/// Upstream `archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:331-332`
/// defines exactly 2 zombie generals (`jiangshifz`, `jiangshinj`).
pub const L19_DOOMSDAY_CHARACTERS: [&str; 2] = ["jiangshifz", "jiangshinj"];

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

/// Upstream `archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6319-6321`
/// defines 0 generals and 3 `zhengsu_*` cards.
pub const L20_MARK_SUPPLEMENT_CARDS: [&str; 3] = ["zhengsu_leijin", "zhengsu_mingzhi", "zhengsu_bianzhen"];

pub const L20_MARK_SUPPLEMENT_SKILLS: [&str; 74] = [
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
    "wengua5", "zuoxing", "kurou", "_chongzhu",
];

pub const L24_SKIN_SWITCH_SKILLS: [&str; 23] = [
    "_tsx", "_gj", "_ts", "_playAudioToQueue", "_checkDcdChuChang",
    "_checkDcdShan", "_fix_yh", "_fix_phase_yh", "_pfqh_check_roundCount", "_check_die_yh",
    "_pfqh_reapply_saved_params", "_pfqh_reapply_on_init", "_pfqh_check_hp_change", "_pfqh_check_damage_times", "_pfqh_check_damage",
    "_pfqh_check_jisha", "_pfqh_check_changeGroup", "_qhlyChangeGroupSkin", "_pfqh_check_phaseBegin", "_pfqh_check_phaseEnd",
    "_qhly_autoc", "_qhly_addButton", "_skin_switch_hook",
];

/// Upstream `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:338-397`
/// defines 60 `ssr_*` generals.
pub const L25_ZUAN_CHARACTERS: [&str; 60] = [
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

pub const L27_SELECT_BEAUTIFY_SKILLS: [&str; 43] = [
    "baonvezhi", "baonvezhi_mark", "bolan", "clandianzhan", "dcxiaoxi",
    "dinghan", "drlt_duorui", "huamu", "jinghe", "jishe",
    "jishe2", "kuangcai", "mbcmqingzheng", "mbwuling", "new_meibu",
    "new_zhixi", "new_zhixi_recover", "potzhuangshi", "pro_drlt_duorui", "qimou",
    "quanji", "regushe", "rehuashen", "rehuashen_mark", "reqimou",
    "rexinsheng", "sbjianxiong", "sbqingzheng", "sbtongye", "sbzhaxiang",
    "shouxi", "spyanjiao", "taoluan", "twfengji", "twharvestinori",
    "twxiafeng", "twxingzhui", "twzhouhu", "twzhouzu", "twzuhuo",
    "xiefang", "xinfu_xionghuo", "xingchong",
];
