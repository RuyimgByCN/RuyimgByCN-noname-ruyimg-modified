//! Authentic upstream `L25` (`祖安武将`) catalog data extracted from
//! `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`.

use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::content::standard::Gender;

/// All 60 `ssr_*` generals from `祖安武将/extension.js:338-397` with authentic faction, gender, max_hp, and `ssr_*` skills.
pub const L25_SSR_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 60] = [
    // extension.js:338
    ("ssr_luxun", Faction::Wu, Gender::Male, 3, &["ssr_qianxun", "ssr_jueji"]),
    // extension.js:339
    ("ssr_sunquan", Faction::Wu, Gender::Male, 4, &["ssr_zhiheng", "ssr_jianye", "ssr_jiuyuan"]),
    // extension.js:340
    ("ssr_zhaoyun", Faction::Shu, Gender::Male, 4, &["ssr_newlongdan", "ssr_newpozhen"]),
    // extension.js:341
    ("ssr_zhugeliang", Faction::Shu, Gender::Male, 3, &["ssr_jinnang", "ssr_newbazhen"]),
    // extension.js:342
    ("ssr_zhangliao", Faction::Wei, Gender::Male, 4, &["ssr_tuxi", "ssr_newmubing"]),
    // extension.js:343
    ("ssr_ganning", Faction::Wu, Gender::Male, 4, &["ssr_jinfan", "ssr_qixi"]),
    // extension.js:344
    ("ssr_machao", Faction::Shu, Gender::Male, 4, &["ssr_zhuiji", "ssr_tieji"]),
    // extension.js:345
    ("ssr_zhouyu", Faction::Wu, Gender::Male, 3, &["ssr_youqin", "ssr_newfanjian"]),
    // extension.js:346
    ("ssr_liubei", Faction::Shu, Gender::Male, 4, &["ssr_shichou", "ssr_newrende", "ssr_jijiang"]),
    // extension.js:347
    ("ssr_dianwei", Faction::Wei, Gender::Male, 4, &["ssr_qiangxi", "ssr_qingji"]),
    // extension.js:348
    ("ssr_guojia", Faction::Wei, Gender::Male, 3, &["ssr_bailun", "ssr_newyice"]),
    // extension.js:349
    ("ssr_caocao", Faction::Wei, Gender::Male, 4, &["ssr_tumeng", "ssr_zhujiu", "ssr_hujia"]),
    // extension.js:350
    ("ssr_lvbu", Faction::Qun, Gender::Male, 4, &["ssr_shenfen", "ssr_kuangbao", "ssr_wushuang", "ssr_jiwu"]),
    // extension.js:351
    ("ssr_huangzhong", Faction::Shu, Gender::Male, 4, &["ssr_liegong", "ssr_pojia"]),
    // extension.js:352
    ("ssr_xiahoudun", Faction::Wei, Gender::Male, 4, &["ssr_newganglie", "ssr_zuni"]),
    // extension.js:353
    ("ssr_huatuo", Faction::Qun, Gender::Male, 3, &["ssr_chuli", "ssr_xianfang"]),
    // extension.js:354
    ("ssr_yujin", Faction::Wei, Gender::Male, 4, &["ssr_newzhenjun", "ssr_newyizhong"]),
    // extension.js:355
    ("ssr_zuoci", Faction::Qun, Gender::Male, 3, &["ssr_shendao", "ssr_newjiahe"]),
    // extension.js:356
    ("ssr_huaxiong", Faction::Qun, Gender::Male, 4, &["ssr_miedi", "ssr_shiyong"]),
    // extension.js:357
    ("ssr_huanggai", Faction::Wu, Gender::Male, 4, &["ssr_newkurou", "ssr_fenchuan"]),
    // extension.js:358
    ("ssr_diaochan", Faction::Qun, Gender::Female, 3, &["ssr_newlihun", "ssr_biyue"]),
    // extension.js:359
    ("ssr_yuanshu", Faction::Qun, Gender::Male, 4, &["ssr_yjixi", "ssr_yanji", "ssr_cuandi"]),
    // extension.js:360
    ("ssr_guanyu", Faction::Shu, Gender::Male, 4, &["ssr_wusheng", "ssr_yijue"]),
    // extension.js:361
    ("ssr_lvmeng", Faction::Wu, Gender::Male, 4, &["ssr_newbaiyi", "ssr_liecheng"]),
    // extension.js:362
    ("ssr_gongsunzan", Faction::Qun, Gender::Male, 4, &["ssr_newyucong", "ssr_newyicong"]),
    // extension.js:363
    ("ssr_daqiao", Faction::Wu, Gender::Female, 3, &["ssr_newguose", "ssr_newtongque"]),
    // extension.js:364
    ("ssr_xiaoqiao", Faction::Wu, Gender::Female, 3, &["ssr_newtianxiang", "ssr_newtongque"]),
    // extension.js:365
    ("ssr_xuzhu", Faction::Wei, Gender::Male, 4, &["ssr_huchi", "ssr_newluoyi"]),
    // extension.js:366
    ("ssr_zhangfei", Faction::Shu, Gender::Male, 4, &["ssr_zhenhun", "ssr_paoxiao"]),
    // extension.js:367
    ("ssr_zhenji", Faction::Wei, Gender::Female, 3, &["ssr_newluoshen", "ssr_yushui"]),
    // extension.js:368
    ("ssr_sunshangxiang", Faction::Wu, Gender::Female, 3, &["ssr_jieyin", "ssr_weigong"]),
    // extension.js:369
    ("ssr_simayi", Faction::Wei, Gender::Male, 3, &["ssr_tianlang", "ssr_newjilue"]),
    // extension.js:370
    ("ssr_yiji", Faction::Shu, Gender::Male, 3, &["ssr_newjijie", "ssr_newjiyuan"]),
    // extension.js:371
    ("ssr_huangyueying", Faction::Shu, Gender::Female, 3, &["ssr_jiqiao", "ssr_qicai"]),
    // extension.js:372
    ("ssr_jiaxu", Faction::Qun, Gender::Male, 3, &["ssr_newweimu", "ssr_guimou", "ssr_newluanwu"]),
    // extension.js:373
    ("ssr_caiwenji", Faction::Qun, Gender::Female, 3, &["ssr_nianlei", "ssr_newduanxian"]),
    // extension.js:374
    ("ssr_menghuo", Faction::Shu, Gender::Male, 4, &["ssr_manjia", "ssr_zaiqi"]),
    // extension.js:375
    ("ssr_zhurong", Faction::Shu, Gender::Female, 3, &["ssr_manjia", "ssr_lieren"]),
    // extension.js:376
    ("ssr_jiangwei", Faction::Shu, Gender::Male, 4, &["ssr_zhiji", "ssr_weilong"]),
    // extension.js:377
    ("ssr_sunce", Faction::Wu, Gender::Male, 4, &["ssr_hunzi", "ssr_newjiang", "ssr_newzhiba"]),
    // extension.js:378
    ("ssr_taishici", Faction::Wu, Gender::Male, 4, &["ssr_kuiwei", "ssr_newhanzhan"]),
    // extension.js:379
    ("ssr_zhoutai", Faction::Wu, Gender::Male, 4, &["ssr_newbuqu", "ssr_huwei"]),
    // extension.js:380
    ("ssr_xiahouyuan", Faction::Wei, Gender::Male, 4, &["ssr_shensu", "ssr_hubu"]),
    // extension.js:381
    ("ssr_caoren", Faction::Wei, Gender::Male, 4, &["ssr_jushou", "ssr_zhuzhen"]),
    // extension.js:382
    ("ssr_dengai", Faction::Wei, Gender::Male, 4, &["ssr_tuntian", "ssr_zaoxian"]),
    // extension.js:383
    ("ssr_dongzhuo", Faction::Qun, Gender::Male, 8, &["ssr_newhengzheng", "ssr_baonue", "ssr_newjiuchi"]),
    // extension.js:384
    ("ssr_yuji", Faction::Qun, Gender::Male, 3, &["ssr_newmizhou", "ssr_chanyuan"]),
    // extension.js:385
    ("ssr_yuanshao", Faction::Qun, Gender::Male, 4, &["ssr_newluanji", "ssr_newdingsheng", "ssr_newshiyi"]),
    // extension.js:386
    ("ssr_xushu", Faction::Shu, Gender::Male, 4, &["ssr_zhuhai", "ssr_yijian"]),
    // extension.js:387
    ("ssr_fazheng", Faction::Shu, Gender::Male, 3, &["ssr_newfabing", "ssr_xuanhuo"]),
    // extension.js:388
    ("ssr_pangtong", Faction::Shu, Gender::Male, 3, &["ssr_lianhuan", "ssr_newsance"]),
    // extension.js:389
    ("ssr_zhanghe", Faction::Wei, Gender::Male, 4, &["ssr_newxiebian", "ssr_motu"]),
    // extension.js:390
    ("ssr_re_lidian", Faction::Wei, Gender::Male, 4, &["ssr_xunxun", "ssr_wangxi"]),
    // extension.js:391
    ("ssr_xuhuang", Faction::Wei, Gender::Male, 4, &["ssr_jieliang", "ssr_zhiyan"]),
    // extension.js:392
    ("ssr_sunjian", Faction::Wu, Gender::Male, 4, &["ssr_futu", "ssr_newpolu"]),
    // extension.js:393
    ("ssr_lusu", Faction::Wu, Gender::Male, 3, &["ssr_dimeng", "ssr_fuhui"]),
    // extension.js:394
    ("ssr_zhangzhang", Faction::Wu, Gender::Male, 3, &["ssr_zhijian", "ssr_guzheng"]),
    // extension.js:395
    ("ssr_zhangjiao", Faction::Qun, Gender::Male, 3, &["ssr_tianlei", "ssr_gaiming", "ssr_newguidao", "ssr_zhuji"]),
    // extension.js:396
    ("ssr_zhangbao", Faction::Qun, Gender::Male, 3, &["ssr_dihuo", "ssr_gaiming", "ssr_guifu"]),
    // extension.js:397
    ("ssr_zhangliang", Faction::Qun, Gender::Male, 3, &["ssr_renhong", "ssr_gaiming", "ssr_fangtong"]),
];

/// All 130 `ssr_*` general skills from `祖安武将/extension.js:4911-33889`.
pub const L25_SSR_GENERAL_SKILLS: [&str; 130] = [
    "ssr_qianxun",
    "ssr_jueji",
    "ssr_zhiheng",
    "ssr_jianye",
    "ssr_jiuyuan",
    "ssr_newlongdan",
    "ssr_newpozhen",
    "ssr_jinnang",
    "ssr_newbazhen",
    "ssr_tuxi",
    "ssr_newmubing",
    "ssr_jinfan",
    "ssr_qixi",
    "ssr_zhuiji",
    "ssr_tieji",
    "ssr_youqin",
    "ssr_newfanjian",
    "ssr_shichou",
    "ssr_newrende",
    "ssr_jijiang",
    "ssr_qiangxi",
    "ssr_qingji",
    "ssr_bailun",
    "ssr_newyice",
    "ssr_tumeng",
    "ssr_zhujiu",
    "ssr_hujia",
    "ssr_shenfen",
    "ssr_kuangbao",
    "ssr_wushuang",
    "ssr_jiwu",
    "ssr_liegong",
    "ssr_pojia",
    "ssr_newganglie",
    "ssr_zuni",
    "ssr_chuli",
    "ssr_xianfang",
    "ssr_newzhenjun",
    "ssr_newyizhong",
    "ssr_shendao",
    "ssr_newjiahe",
    "ssr_miedi",
    "ssr_shiyong",
    "ssr_newkurou",
    "ssr_fenchuan",
    "ssr_newlihun",
    "ssr_biyue",
    "ssr_yjixi",
    "ssr_yanji",
    "ssr_cuandi",
    "ssr_wusheng",
    "ssr_yijue",
    "ssr_newbaiyi",
    "ssr_liecheng",
    "ssr_newyucong",
    "ssr_newyicong",
    "ssr_newguose",
    "ssr_newtongque",
    "ssr_newtianxiang",
    "ssr_huchi",
    "ssr_newluoyi",
    "ssr_zhenhun",
    "ssr_paoxiao",
    "ssr_newluoshen",
    "ssr_yushui",
    "ssr_jieyin",
    "ssr_weigong",
    "ssr_tianlang",
    "ssr_newjilue",
    "ssr_newjijie",
    "ssr_newjiyuan",
    "ssr_jiqiao",
    "ssr_qicai",
    "ssr_newweimu",
    "ssr_guimou",
    "ssr_newluanwu",
    "ssr_nianlei",
    "ssr_newduanxian",
    "ssr_manjia",
    "ssr_zaiqi",
    "ssr_lieren",
    "ssr_zhiji",
    "ssr_weilong",
    "ssr_hunzi",
    "ssr_newjiang",
    "ssr_newzhiba",
    "ssr_kuiwei",
    "ssr_newhanzhan",
    "ssr_newbuqu",
    "ssr_huwei",
    "ssr_shensu",
    "ssr_hubu",
    "ssr_jushou",
    "ssr_zhuzhen",
    "ssr_tuntian",
    "ssr_zaoxian",
    "ssr_newhengzheng",
    "ssr_baonue",
    "ssr_newjiuchi",
    "ssr_newmizhou",
    "ssr_chanyuan",
    "ssr_newluanji",
    "ssr_newdingsheng",
    "ssr_newshiyi",
    "ssr_zhuhai",
    "ssr_yijian",
    "ssr_newfabing",
    "ssr_xuanhuo",
    "ssr_lianhuan",
    "ssr_newsance",
    "ssr_newxiebian",
    "ssr_motu",
    "ssr_xunxun",
    "ssr_wangxi",
    "ssr_jieliang",
    "ssr_zhiyan",
    "ssr_futu",
    "ssr_newpolu",
    "ssr_dimeng",
    "ssr_fuhui",
    "ssr_zhijian",
    "ssr_guzheng",
    "ssr_tianlei",
    "ssr_gaiming",
    "ssr_newguidao",
    "ssr_zhuji",
    "ssr_dihuo",
    "ssr_guifu",
    "ssr_renhong",
    "ssr_fangtong",
];

/// All 84 global/mode/rule skills from `祖安武将/extension.js:18984-49741`.
pub const L25_ALL_RULE_SKILLS: [&str; 84] = [
    "g_du",
    "_player_xxgg_zhishixian_xxxxx_ggg",
    "_player_xxgg_zhishixian2_xxxxx_ggg",
    "_player_xxgg_zhishixian3_xxxxx_ggg",
    "_jszsx",
    "_strengthencardskill",
    "_strengthenstandardskill",
    "_ssrnohuahensskill",
    "_ssrchangehpskill",
    "_diycharacterskill",
    "_emojiRebate1",
    "_emojiRebate2",
    "_zuan_damageSendEgg",
    "_zuan_sendWine",
    "_zuan_damageSendFlower",
    "_zuan_killSendFlower",
    "_zuanwashhandcard",
    "_newbiankuang",
    "_clearBackground",
    "_newBackground",
    "_newBackgroundAtart",
    "_zuanRoundTX",
    "_newZuanPhaseTX",
    "_zuanphaseTX",
    "_keepideskill",
    "_zuanidentityskill",
    "_zuanidentitychgneiskill",
    "_zuanidentitydieafterskill",
    "_zuanidentityshowskill",
    "_zuanidentitydyingskill",
    "_zuanidentitytodieskill",
    "_zuanidentitydamskill",
    "_zuanidentityrecskill",
    "_zuanidentityuseskill",
    "_yuanqiskill",
    "_anyeskill",
    "_taohuaskill",
    "_lunjianskill",
    "_jianghunskill",
    "_dietime",
    "_huanhaiskill",
    "_thunderjudgemodeskill",
    "_thundermodeskill",
    "_guixinmodeskill",
    "_guanmodeskill",
    "_guandieskill",
    "_qunxiongzhengbaskill",
    "_zhongzhubuff",
    "_jingchengadddie",
    "_zhongfandieskill",
    "_tamodeskill",
    "_zhudieskill",
    "_levelskill",
    "_expskill",
    "_exp2skill",
    "_useskill",
    "_killskill",
    "_zuanSTARTsayskill",
    "_zuanUSECARDsayskill",
    "_zuanCHANGEHPsayskill",
    "_zuansaidskill",
    "_recoversay",
    "_damagesay",
    "_diesay",
    "_tosay",
    "_fromsay",
    "_selfsay",
    "_startgamesay",
    "_basicjuedgeskill",
    "_goodluckjuedgeskill",
    "_bingleAddJudge",
    "_bingleLoseJudge",
    "_bingleDying",
    "_binglediecls",
    "_bingletexiao",
    "_clearthunder",
    "_bingletexiaochangehp",
    "_bingletexiao1",
    "_bingletexiao2",
    "wumou",
    "new_guixin",
    "gongxin",
    "g_nzry_jieying",
    "g_chenhuodajie",
];

/// Authentic `ssr_newsance_*`, `ssr_yongsi_yuxi_card`, `ssr_jinfan`, and `ssr_yongsi` cards from `祖安武将/extension.js:6535-28385`.
pub const L25_ZUAN_FORMAL_CARDS: [(&str, CardKind, Suit, u8); 6] = [
    ("ssr_newsance_xicheng", CardKind::Wuzhong, Suit::Spade, 7),
    ("ssr_newsance_weicheng", CardKind::Wuzhong, Suit::Club, 8),
    ("ssr_newsance_shoucheng", CardKind::Wuzhong, Suit::Heart, 9),
    ("ssr_yongsi_yuxi_card", CardKind::WuxingPan, Suit::Heart, 1),
    ("ssr_jinfan", CardKind::Wuzhong, Suit::Spade, 9),
    ("ssr_yongsi", CardKind::WuxingPan, Suit::Heart, 1),
];

/// All 5 top-level card entries inventoried for `L25` (`extension.js:6535-28355`).
pub const L25_ZUAN_CARD_ENTRIES: [&str; 5] = [
    "ssr_newsance_xicheng",
    "ssr_newsance_weicheng",
    "ssr_newsance_shoucheng",
    "ssr_jinfan",
    "ssr_yongsi",
];
