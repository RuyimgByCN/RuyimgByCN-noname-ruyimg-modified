//! Authoritative `U05 杀海拾遗` 275 upstream cards and 1,007 upstream skills (+ cross refs)
//! extracted from `archive/upstream/noname/apps/core/extension/杀海拾遗/{card,character,play,main}/`.

use engine_native::catalog::{CardKind, Suit};

/// `U05` `gujian` cards (`36` items).
pub const U05_GUJIAN_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 36] = [
    ("luyugeng", CardKind::Luyugeng, 0, Suit::Spade, 1), // card/gujian/card.js:4
    ("jinlianzhu", CardKind::Jinlianzhu, 0, Suit::Heart, 2), // card/gujian/card.js:26
    ("chunbing", CardKind::Chunbing, 0, Suit::Club, 3), // card/gujian/card.js:57
    ("gudonggeng", CardKind::Gudonggeng, 0, Suit::Diamond, 4), // card/gujian/card.js:88
    ("liyutang", CardKind::Liyutang, 0, Suit::Spade, 5), // card/gujian/card.js:115
    ("mizhilianou", CardKind::Mizhilianou, 0, Suit::Heart, 6), // card/gujian/card.js:145
    ("xiajiao", CardKind::Xiajiao, 0, Suit::Club, 7), // card/gujian/card.js:180
    ("tanhuadong", CardKind::Tanhuadong, 0, Suit::Diamond, 8), // card/gujian/card.js:203
    ("mapodoufu", CardKind::Mapodoufu, 0, Suit::Spade, 9), // card/gujian/card.js:225
    ("qingtuan", CardKind::Qingtuan, 0, Suit::Heart, 10), // card/gujian/card.js:251
    ("yougeng", CardKind::Yougeng, 0, Suit::Club, 11), // card/gujian/card.js:291
    ("molicha", CardKind::Molicha, 0, Suit::Diamond, 12), // card/gujian/card.js:321
    ("yuanbaorou", CardKind::Yuanbaorou, 0, Suit::Spade, 13), // card/gujian/card.js:343
    ("heilonglinpian", CardKind::Heilonglinpian, 0, Suit::Heart, 1), // card/gujian/card.js:383
    ("mutoumianju", CardKind::Mutoumianju, 0, Suit::Club, 2), // card/gujian/card.js:406
    ("gjyuheng", CardKind::Gjyuheng, 0, Suit::Diamond, 3), // card/gujian/card.js:419
    ("gjyuheng_plus", CardKind::Gjyuheng, 0, Suit::Spade, 4), // card/gujian/card.js:435
    ("gjyuheng_pro", CardKind::Gjyuheng, 0, Suit::Heart, 5), // card/gujian/card.js:452
    ("shatang", CardKind::Shatang, 0, Suit::Club, 6), // card/gujian/card.js:466
    ("shujinsan", CardKind::Shujinsan, 0, Suit::Diamond, 7), // card/gujian/card.js:504
    ("dinvxuanshuang", CardKind::Dinvxuanshuang, 0, Suit::Spade, 8), // card/gujian/card.js:551
    ("ziyangdan", CardKind::Ziyangdan, 0, Suit::Heart, 9), // card/gujian/card.js:585
    ("yunvyuanshen", CardKind::Yunvyuanshen, 0, Suit::Club, 10), // card/gujian/card.js:621
    ("bingpotong", CardKind::Bingpotong, 0, Suit::Diamond, 11), // card/gujian/card.js:656
    ("feibiao", CardKind::Feibiao, 0, Suit::Spade, 12), // card/gujian/card.js:781
    ("longxugou", CardKind::Longxugou, 0, Suit::Heart, 13), // card/gujian/card.js:820
    ("qiankunbiao", CardKind::Qiankunbiao, 0, Suit::Club, 1), // card/gujian/card.js:870
    ("shenhuofeiya", CardKind::Shenhuofeiya, 0, Suit::Diamond, 2), // card/gujian/card.js:906
    ("mianlijinzhen", CardKind::Mianlijinzhen, 0, Suit::Spade, 3), // card/gujian/card.js:992
    ("liutouge", CardKind::Liutouge, 0, Suit::Heart, 4), // card/gujian/card.js:1017
    ("liufengsan", CardKind::Liufengsan, 0, Suit::Club, 5), // card/gujian/card.js:1043
    ("shihuifen", CardKind::Shihuifen, 0, Suit::Diamond, 6), // card/gujian/card.js:1092
    ("yanjiadan_heart", CardKind::Jiguan, 0, Suit::Spade, 6), // character/gujian/card.js:4
    ("yanjiadan_diamond", CardKind::Jiguan, 0, Suit::Heart, 7), // character/gujian/card.js:68
    ("yanjiadan_club", CardKind::Jiguan, 0, Suit::Club, 8), // character/gujian/card.js:117
    ("yanjiadan_spade", CardKind::Jiguan, 0, Suit::Diamond, 9), // character/gujian/card.js:149
];

/// `U05` `gwent` cards (`69` items).
pub const U05_GWENT_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 69] = [
    ("gw_dieyi", CardKind::Weapon, 2, Suit::Spade, 7), // card/gwent/card.js:4
    ("gw_dieyi_equip1", CardKind::Weapon, 2, Suit::Heart, 8), // card/gwent/card.js:7
    ("gw_dieyi_equip2", CardKind::Armor, 0, Suit::Club, 9), // card/gwent/card.js:24
    ("gw_dieyi_equip3", CardKind::HorsePlus, 0, Suit::Diamond, 10), // card/gwent/card.js:40
    ("gw_dieyi_equip4", CardKind::HorseMinus, 0, Suit::Spade, 11), // card/gwent/card.js:56
    ("gw_dieyi_equip5", CardKind::WuxingPan, 0, Suit::Heart, 12), // card/gwent/card.js:72
    ("gw_dieyi_judge", CardKind::Le, 0, Suit::Club, 13), // card/gwent/card.js:88
    ("gw_hudiewu", CardKind::Wuzhong, 0, Suit::Diamond, 1), // card/gwent/card.js:100
    ("gw_yigeniyin", CardKind::Wuzhong, 0, Suit::Spade, 2), // card/gwent/card.js:158
    ("gw_leizhoushu", CardKind::Wuzhong, 0, Suit::Heart, 3), // card/gwent/card.js:245
    ("gw_aerdeyin", CardKind::Wuzhong, 0, Suit::Club, 4), // card/gwent/card.js:293
    ("gw_ansha", CardKind::Wuzhong, 0, Suit::Diamond, 5), // card/gwent/card.js:356
    ("gw_xinsheng", CardKind::Wuzhong, 0, Suit::Spade, 6), // card/gwent/card.js:398
    ("gw_niuquzhijing", CardKind::Wuzhong, 0, Suit::Heart, 7), // card/gwent/card.js:514
    ("gw_zhongmozhizhan", CardKind::Wuzhong, 0, Suit::Club, 8), // card/gwent/card.js:609
    ("gw_ganhan", CardKind::Wuzhong, 0, Suit::Diamond, 9), // card/gwent/card.js:661
    ("gw_huangjiashenpan", CardKind::Wuzhong, 0, Suit::Spade, 10), // card/gwent/card.js:737
    ("gw_tunshi", CardKind::Wuzhong, 0, Suit::Heart, 11), // card/gwent/card.js:779
    ("gw_chongci", CardKind::Wuzhong, 0, Suit::Club, 12), // card/gwent/card.js:844
    ("gw_youer", CardKind::Wuzhong, 0, Suit::Diamond, 13), // card/gwent/card.js:930
    ("gw_tongdi", CardKind::Wuzhong, 0, Suit::Spade, 1), // card/gwent/card.js:968
    ("gw_fuyuan", CardKind::Wuzhong, 0, Suit::Heart, 2), // card/gwent/card.js:1001
    ("gw_zhuoshao", CardKind::Wuzhong, 0, Suit::Club, 3), // card/gwent/card.js:1027
    ("gw_butianshu", CardKind::Wuzhong, 0, Suit::Diamond, 4), // card/gwent/card.js:1057
    ("gw_shizizhaohuan", CardKind::Wuzhong, 0, Suit::Spade, 5), // card/gwent/card.js:1137
    ("gw_zuihouyuanwang", CardKind::Wuzhong, 0, Suit::Heart, 6), // card/gwent/card.js:1189
    ("gw_zirankuizeng", CardKind::Wuzhong, 0, Suit::Club, 7), // card/gwent/card.js:1228
    ("gw_nuhaifengbao", CardKind::Wuzhong, 0, Suit::Diamond, 8), // card/gwent/card.js:1345
    ("gw_baishuang", CardKind::Wuzhong, 0, Suit::Spade, 9), // card/gwent/card.js:1367
    ("gw_baobaoshu", CardKind::Wuzhong, 0, Suit::Heart, 10), // card/gwent/card.js:1388
    ("gw_guaiwuchaoxue", CardKind::Wuzhong, 0, Suit::Club, 11), // card/gwent/card.js:1411
    ("gw_qinpendayu", CardKind::Qinpendayu, 0, Suit::Diamond, 12), // card/gwent/card.js:1456
    ("gw_birinongwu", CardKind::Birinongwu, 0, Suit::Spade, 13), // card/gwent/card.js:1486
    ("gw_ciguhanshuang", CardKind::Ciguhanshuang, 0, Suit::Heart, 1), // card/gwent/card.js:1516
    ("gw_baoxueyaoshui", CardKind::Baoxueyaoshui, 0, Suit::Club, 2), // card/gwent/card.js:1546
    ("gw_zhihuanjun", CardKind::Zhihuanjun, 0, Suit::Diamond, 3), // card/gwent/card.js:1585
    ("gw_zumoshoukao", CardKind::Zumoshoukao, 0, Suit::Spade, 4), // card/gwent/card.js:1616
    ("gw_aozuzhilei", CardKind::Aozuzhilei, 0, Suit::Heart, 5), // card/gwent/card.js:1648
    ("gw_poxiao", CardKind::Poxiao, 0, Suit::Club, 6), // card/gwent/card.js:1681
    ("gw_kunenfayin", CardKind::Kunenfayin, 0, Suit::Diamond, 7), // card/gwent/card.js:1758
    ("gw_yanziyaoshui", CardKind::Yanziyaoshui, 0, Suit::Spade, 8), // card/gwent/card.js:1785
    ("gw_wenyi", CardKind::Wenyi, 0, Suit::Heart, 9), // card/gwent/card.js:1814
    ("gw_shanbengshu", CardKind::Shanbengshu, 0, Suit::Club, 10), // card/gwent/card.js:1850
    ("gwjinli_jiu", CardKind::Wuzhong, 0, Suit::Spade, 10), // character/gwent/card.js:4
    ("gw_xianzumaijiu", CardKind::Wuzhong, 0, Suit::Heart, 11), // character/gwent/card.js:9
    ("gwmaoxian_yioufeisi", CardKind::Wuzhong, 0, Suit::Club, 12), // character/gwent/card.js:119
    ("gwmaoxian_luoqi", CardKind::Wuzhong, 0, Suit::Diamond, 13), // character/gwent/card.js:158
    ("gwmaoxian_jieluote", CardKind::Wuzhong, 0, Suit::Spade, 1), // character/gwent/card.js:201
    ("gwmaoxian_yenaifa", CardKind::Wuzhong, 0, Suit::Heart, 2), // character/gwent/card.js:261
    ("gwmaoxian_telisi", CardKind::Wuzhong, 0, Suit::Club, 3), // character/gwent/card.js:294
    ("gwmaoxian_hengsaite", CardKind::Wuzhong, 0, Suit::Diamond, 4), // character/gwent/card.js:329
    ("gwmaoxian_fuertaisite", CardKind::Wuzhong, 0, Suit::Spade, 5), // character/gwent/card.js:371
    ("gwmaoxian_laduoweide", CardKind::Wuzhong, 0, Suit::Heart, 6), // character/gwent/card.js:406
    ("gwmaoxian_enxier", CardKind::Wuzhong, 0, Suit::Club, 7), // character/gwent/card.js:438
    ("gwmaoxian_fulisi", CardKind::Wuzhong, 0, Suit::Diamond, 8), // character/gwent/card.js:474
    ("gwmaoxian_kaerweite", CardKind::Wuzhong, 0, Suit::Spade, 9), // character/gwent/card.js:526
    ("gwmaoxian_bulanwang", CardKind::Wuzhong, 0, Suit::Heart, 10), // character/gwent/card.js:561
    ("gwmaoxian_kuite", CardKind::Wuzhong, 0, Suit::Club, 11), // character/gwent/card.js:603
    ("gwmaoxian_haluo", CardKind::Wuzhong, 0, Suit::Diamond, 12), // character/gwent/card.js:637
    ("gwmaoxian_dagong", CardKind::Wuzhong, 0, Suit::Spade, 13), // character/gwent/card.js:668
    ("gwmaoxian_gaier", CardKind::Wuzhong, 0, Suit::Heart, 1), // character/gwent/card.js:704
    ("gwmaoxian_airuiting", CardKind::Wuzhong, 0, Suit::Club, 2), // character/gwent/card.js:751
    ("gwmaoxian_aisinie", CardKind::Wuzhong, 0, Suit::Diamond, 3), // character/gwent/card.js:796
    ("gwmaoxian_falanxisika", CardKind::Wuzhong, 0, Suit::Spade, 4), // character/gwent/card.js:833
    ("gwmaoxian_huoge", CardKind::Wuzhong, 0, Suit::Heart, 5), // character/gwent/card.js:875
    ("gw_wuyao", CardKind::Wuzhong, 0, Suit::Club, 6), // character/gwent/card.js:925
    ("gw_lang", CardKind::Wuzhong, 0, Suit::Diamond, 7), // character/gwent/card.js:938
    ("gw_dudayuanshuai1", CardKind::Wuzhong, 0, Suit::Spade, 8), // character/gwent/card.js:951
    ("gw_dudayuanshuai2", CardKind::Wuzhong, 0, Suit::Heart, 9), // character/gwent/card.js:975
];

/// `U05` `hearth` cards (`57` items).
pub const U05_HEARTH_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 57] = [
    ("linghunzhihuo", CardKind::Linghunzhihuo, 0, Suit::Diamond, 11), // card/hearth/card.js:4
    ("jihuocard", CardKind::Jihuocard, 0, Suit::Spade, 12), // card/hearth/card.js:47
    ("zhaomingdan", CardKind::Zhaomingdan, 0, Suit::Heart, 13), // card/hearth/card.js:70
    ("shijieshu", CardKind::Shijieshu, 0, Suit::Club, 1), // card/hearth/card.js:135
    ("shandianjian", CardKind::Wuzhong, 0, Suit::Diamond, 2), // card/hearth/card.js:210
    ("shihuawuqi", CardKind::Tao, 0, Suit::Spade, 3), // card/hearth/card.js:342
    ("dunpaigedang", CardKind::Dunpaigedang, 0, Suit::Heart, 4), // card/hearth/card.js:371
    ("chuansongmen", CardKind::Chuansongmen, 0, Suit::Club, 5), // card/hearth/card.js:399
    ("tanshezhiren", CardKind::Tanshezhiren, 0, Suit::Diamond, 6), // card/hearth/card.js:458
    ("xingjiegoutong", CardKind::Xingjiegoutong, 0, Suit::Spade, 7), // card/hearth/card.js:508
    ("shenenshu", CardKind::Wuzhong, 0, Suit::Heart, 8), // card/hearth/card.js:549
    ("zhiliaobo", CardKind::Zhiliaobo, 0, Suit::Club, 9), // card/hearth/card.js:591
    ("yuansuhuimie", CardKind::Yuansuhuimie, 0, Suit::Diamond, 10), // card/hearth/card.js:638
    ("hsfashu_anyingjingxiang", CardKind::Wuzhong, 0, Suit::Club, 10), // character/hearth/card.js:4
    ("hsfashu_buwendingyibian", CardKind::Wuzhong, 0, Suit::Diamond, 11), // character/hearth/card.js:11
    ("hstianqi_dalian", CardKind::Weapon, 2, Suit::Spade, 12), // character/hearth/card.js:94
    ("hstianqi_shali", CardKind::Armor, 0, Suit::Heart, 13), // character/hearth/card.js:107
    ("hstianqi_nazigelin", CardKind::HorseMinus, 0, Suit::Club, 1), // character/hearth/card.js:120
    ("hstianqi_suolasi", CardKind::HorsePlus, 0, Suit::Diamond, 2), // character/hearth/card.js:136
    ("hsjixie_zhadan", CardKind::Wuzhong, 0, Suit::Spade, 3), // character/hearth/card.js:155
    ("hsqizhou_feng", CardKind::Wuzhong, 0, Suit::Heart, 4), // character/hearth/card.js:184
    ("hsqizhou_shui", CardKind::Wuzhong, 0, Suit::Club, 5), // character/hearth/card.js:236
    ("hsqizhou_huo", CardKind::Wuzhong, 0, Suit::Diamond, 6), // character/hearth/card.js:263
    ("hsqizhou_tu", CardKind::Wuzhong, 0, Suit::Spade, 7), // character/hearth/card.js:287
    ("hsqingyu_feibiao", CardKind::Wuzhong, 0, Suit::Heart, 8), // character/hearth/card.js:309
    ("hsqingyu_zhanfang", CardKind::Wuzhong, 0, Suit::Club, 9), // character/hearth/card.js:348
    ("hsqingyu_hufu", CardKind::Wuzhong, 0, Suit::Diamond, 10), // character/hearth/card.js:379
    ("hsqingyu_shandian", CardKind::Wuzhong, 0, Suit::Spade, 11), // character/hearth/card.js:400
    ("hsqingyu_zhao", CardKind::Weapon, 2, Suit::Heart, 12), // character/hearth/card.js:424
    ("hsdusu_xueji", CardKind::Wuzhong, 0, Suit::Club, 13), // character/hearth/card.js:440
    ("hsdusu_kuyecao", CardKind::Wuzhong, 0, Suit::Diamond, 1), // character/hearth/card.js:468
    ("hsdusu_huangxuecao", CardKind::Wuzhong, 0, Suit::Spade, 2), // character/hearth/card.js:503
    ("hsdusu_huoyanhua", CardKind::Wuzhong, 0, Suit::Heart, 3), // character/hearth/card.js:525
    ("hsdusu_shinancao", CardKind::Wuzhong, 0, Suit::Club, 4), // character/hearth/card.js:550
    ("hsbaowu_cangbaotu", CardKind::Wuzhong, 0, Suit::Diamond, 5), // character/hearth/card.js:578
    ("hsbaowu_huangjinyuanhou", CardKind::Wuzhong, 0, Suit::Spade, 6), // character/hearth/card.js:602
    ("hsshenqi_nengliangzhiguang", CardKind::Wuzhong, 0, Suit::Heart, 7), // character/hearth/card.js:663
    ("hsshenqi_kongbusangzhong", CardKind::Wuzhong, 0, Suit::Club, 8), // character/hearth/card.js:699
    ("hsshenqi_morijingxiang", CardKind::Wuzhong, 0, Suit::Diamond, 9), // character/hearth/card.js:733
    ("hsmengjing_feicuiyoulong", CardKind::Wuzhong, 0, Suit::Spade, 10), // character/hearth/card.js:775
    ("hsmengjing_suxing", CardKind::Wuzhong, 0, Suit::Heart, 11), // character/hearth/card.js:797
    ("hsmengjing_mengye", CardKind::Wuzhong, 0, Suit::Club, 12), // character/hearth/card.js:823
    ("hsmengjing_mengjing", CardKind::Wuzhong, 0, Suit::Diamond, 13), // character/hearth/card.js:848
    ("hsmengjing_huanxiaojiemei", CardKind::Wuzhong, 0, Suit::Spade, 1), // character/hearth/card.js:892
    ("hszuzhou_nvwudeganguo", CardKind::Wuzhong, 0, Suit::Heart, 2), // character/hearth/card.js:915
    ("hszuzhou_nvwudepingguo", CardKind::Wuzhong, 0, Suit::Club, 3), // character/hearth/card.js:957
    ("hszuzhou_nvwudexuetu", CardKind::Wuzhong, 0, Suit::Diamond, 4), // character/hearth/card.js:975
    ("hszuzhou_wushushike", CardKind::Wuzhong, 0, Suit::Spade, 5), // character/hearth/card.js:1009
    ("hszuzhou_guhuo", CardKind::Wuzhong, 0, Suit::Heart, 6), // character/hearth/card.js:1060
    ("tuteng1", CardKind::Wuzhong, 0, Suit::Club, 7), // character/hearth/card.js:1087
    ("tuteng2", CardKind::Wuzhong, 0, Suit::Diamond, 8), // character/hearth/card.js:1094
    ("tuteng3", CardKind::Wuzhong, 0, Suit::Spade, 9), // character/hearth/card.js:1101
    ("tuteng4", CardKind::Wuzhong, 0, Suit::Heart, 10), // character/hearth/card.js:1108
    ("tuteng5", CardKind::Wuzhong, 0, Suit::Club, 11), // character/hearth/card.js:1115
    ("tuteng6", CardKind::Wuzhong, 0, Suit::Diamond, 12), // character/hearth/card.js:1122
    ("tuteng7", CardKind::Wuzhong, 0, Suit::Spade, 13), // character/hearth/card.js:1129
    ("tuteng8", CardKind::Wuzhong, 0, Suit::Heart, 1), // character/hearth/card.js:1136
];

/// `U05` `mtg` cards (`12` items).
pub const U05_MTG_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 12] = [
    ("mtg_lindixiliu", CardKind::Lindixiliu, 0, Suit::Spade, 11), // card/mtg/card.js:4
    ("mtg_longlushanfeng", CardKind::Longlushanfeng, 0, Suit::Heart, 12), // card/mtg/card.js:63
    ("mtg_cangbaohaiwan", CardKind::Wuzhong, 0, Suit::Club, 13), // card/mtg/card.js:99
    ("mtg_linzhongjianta", CardKind::Linzhongjianta, 0, Suit::Diamond, 1), // card/mtg/card.js:165
    ("mtg_youlin", CardKind::Youlin, 0, Suit::Spade, 2), // card/mtg/card.js:201
    ("mtg_haidao", CardKind::Haidao, 0, Suit::Heart, 3), // card/mtg/card.js:235
    ("mtg_yixialan", CardKind::Yixialan, 0, Suit::Club, 4), // card/mtg/card.js:275
    ("mtg_shuimomuxue", CardKind::Shuimomuxue, 0, Suit::Diamond, 5), // card/mtg/card.js:311
    ("mtg_feixu", CardKind::Feixu, 0, Suit::Spade, 6), // card/mtg/card.js:351
    ("mtg_shamolvzhou", CardKind::Shamolvzhou, 0, Suit::Heart, 7), // card/mtg/card.js:371
    ("mtg_duzhao", CardKind::Wuzhong, 0, Suit::Club, 8), // card/mtg/card.js:391
    ("mtg_bingheyaosai", CardKind::Bingheyaosai, 0, Suit::Diamond, 9), // card/mtg/card.js:441
];

/// `U05` `swd` cards (`79` items).
pub const U05_SWD_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 79] = [
    ("hufu", CardKind::Hufu, 0, Suit::Spade, 10), // card/swd/card.js:4
    ("liuxinghuoyu", CardKind::Liuxinghuoyu, 0, Suit::Heart, 11), // card/swd/card.js:16
    ("dujian", CardKind::Tao, 0, Suit::Club, 12), // card/swd/card.js:99
    ("yangpijuan", CardKind::Yangpijuan, 0, Suit::Diamond, 13), // card/swd/card.js:189
    ("shencaojie", CardKind::Shencaojie, 0, Suit::Spade, 1), // card/swd/card.js:242
    ("yuruyi", CardKind::Yuruyi, 0, Suit::Heart, 2), // card/swd/card.js:282
    ("fengyinzhidan", CardKind::Fengyinzhidan, 0, Suit::Club, 3), // card/swd/card.js:293
    ("yuchanqian", CardKind::Yuchanqian, 0, Suit::Diamond, 4), // card/swd/card.js:364
    ("yuchankun", CardKind::Yuchankun, 0, Suit::Spade, 5), // card/swd/card.js:375
    ("yuchanzhen", CardKind::Yuchanzhen, 0, Suit::Heart, 6), // card/swd/card.js:386
    ("yuchanxun", CardKind::Yuchanxun, 0, Suit::Club, 7), // card/swd/card.js:400
    ("yuchankan", CardKind::Yuchankan, 0, Suit::Diamond, 8), // card/swd/card.js:412
    ("yuchanli", CardKind::Yuchanli, 0, Suit::Spade, 9), // card/swd/card.js:454
    ("yuchangen", CardKind::Yuchangen, 0, Suit::Heart, 10), // card/swd/card.js:468
    ("yuchandui", CardKind::Yuchandui, 0, Suit::Club, 11), // card/swd/card.js:479
    ("mujiaren", CardKind::Jiguan, 0, Suit::Diamond, 12), // card/swd/card.js:490
    ("zhiluxiaohu", CardKind::Jiguan, 0, Suit::Spade, 13), // card/swd/card.js:533
    ("xuejibingbao", CardKind::Xuejibingbao, 0, Suit::Heart, 1), // card/swd/card.js:567
    ("gouhunluo", CardKind::Jiguan, 0, Suit::Club, 2), // card/swd/card.js:592
    ("zhuquezhizhang", CardKind::Zhuquezhizhang, 0, Suit::Diamond, 3), // card/swd/card.js:615
    ("xuanwuzhihuang", CardKind::Xuanwuzhihuang, 0, Suit::Spade, 4), // card/swd/card.js:624
    ("huanglinzhicong", CardKind::Huanglinzhicong, 0, Suit::Heart, 5), // card/swd/card.js:633
    ("cangchizhibi", CardKind::Cangchizhibi, 0, Suit::Club, 6), // card/swd/card.js:642
    ("qinglongzhigui", CardKind::Qinglongzhigui, 0, Suit::Diamond, 7), // card/swd/card.js:651
    ("baishouzhihu", CardKind::Baishouzhihu, 0, Suit::Spade, 8), // card/swd/card.js:660
    ("jiguantong", CardKind::Jiguan, 0, Suit::Heart, 9), // card/swd/card.js:669
    ("donghuangzhong", CardKind::Donghuangzhong, 0, Suit::Club, 10), // card/swd/card.js:750
    ("xuanyuanjian", CardKind::Xuanyuanjian, 0, Suit::Diamond, 11), // card/swd/card.js:762
    ("pangufu", CardKind::Weapon, 4, Suit::Spade, 12), // card/swd/card.js:785
    ("lianyaohu", CardKind::Lianyaohu, 0, Suit::Heart, 13), // card/swd/card.js:798
    ("haotianta", CardKind::Haotianta, 0, Suit::Club, 1), // card/swd/card.js:823
    ("fuxiqin", CardKind::WuxingPan, 0, Suit::Diamond, 2), // card/swd/card.js:835
    ("shennongding", CardKind::Shennongding, 0, Suit::Spade, 3), // card/swd/card.js:850
    ("kongdongyin", CardKind::Kongdongyin, 0, Suit::Heart, 4), // card/swd/card.js:865
    ("kunlunjingc", CardKind::Kunlunjingc, 0, Suit::Club, 5), // card/swd/card.js:888
    ("nvwashi", CardKind::Nvwashi, 0, Suit::Diamond, 6), // card/swd/card.js:903
    ("guisheqi", CardKind::Guisheqi, 0, Suit::Spade, 7), // card/swd/card.js:915
    ("jiguanfeng", CardKind::Jiguan, 0, Suit::Heart, 8), // card/swd/card.js:936
    ("jiguanyuan", CardKind::Jiguan, 0, Suit::Club, 9), // card/swd/card.js:975
    ("shenmiguo", CardKind::Shenmiguo, 0, Suit::Diamond, 10), // card/swd/card.js:1027
    ("qinglianxindeng", CardKind::Armor, 0, Suit::Spade, 11), // card/swd/card.js:1051
    ("lingjiandai", CardKind::Jiguan, 0, Suit::Heart, 12), // card/swd/card.js:1062
    ("jiguanshu", CardKind::Jiguan, 0, Suit::Club, 13), // card/swd/card.js:1099
    ("jiguanyaoshu", CardKind::Jiguan, 0, Suit::Diamond, 1), // card/swd/card.js:1212
    ("hslingjian_xuanfengzhiren", CardKind::Hslingjian, 0, Suit::Spade, 2), // card/swd/card.js:1258
    ("hslingjian_zhongxinghujia", CardKind::Hslingjian, 0, Suit::Heart, 3), // card/swd/card.js:1280
    ("hslingjian_xingtigaizao", CardKind::Hslingjian, 0, Suit::Club, 4), // card/swd/card.js:1358
    ("hslingjian_shijianhuisu", CardKind::Hslingjian, 0, Suit::Diamond, 5), // card/swd/card.js:1392
    ("hslingjian_jinjilengdong", CardKind::Hslingjian, 0, Suit::Spade, 6), // card/swd/card.js:1445
    ("hslingjian_shengxiuhaojiao", CardKind::Hslingjian, 0, Suit::Heart, 7), // card/swd/card.js:1495
    ("hslingjian_yinmilichang", CardKind::Hslingjian, 0, Suit::Club, 8), // card/swd/card.js:1525
    ("xingjunyan", CardKind::Xingjunyan, 0, Suit::Diamond, 9), // card/swd/card.js:1558
    ("qinglonglingzhu", CardKind::Qinglonglingzhu, 0, Suit::Spade, 10), // card/swd/card.js:1569
    ("baihupifeng", CardKind::Armor, 0, Suit::Heart, 11), // card/swd/card.js:1580
    ("fengxueren", CardKind::Fengxueren, 0, Suit::Club, 12), // card/swd/card.js:1597
    ("chilongya", CardKind::Chilongya, 0, Suit::Diamond, 13), // card/swd/card.js:1609
    ("daihuofenglun", CardKind::Daihuofenglun, 0, Suit::Spade, 1), // card/swd/card.js:1621
    ("xiayuncailing", CardKind::Xiayuncailing, 0, Suit::Heart, 2), // card/swd/card.js:1633
    ("shentoumianju", CardKind::Shentoumianju, 0, Suit::Club, 3), // card/swd/card.js:1639
    ("xianluhui", CardKind::Xianluhui, 0, Suit::Diamond, 4), // card/swd/card.js:1651
    ("xiangyuye", CardKind::Tao, 0, Suit::Spade, 5), // card/swd/card.js:1685
    ("caoyao", CardKind::Caoyao, 0, Suit::Heart, 6), // card/swd/card.js:1722
    ("tianxianjiu", CardKind::Tianxianjiu, 0, Suit::Club, 7), // card/swd/card.js:1747
    ("huanpodan", CardKind::Tao, 0, Suit::Diamond, 8), // card/swd/card.js:1855
    ("langeguaiyi", CardKind::Langeguaiyi, 0, Suit::Spade, 9), // card/swd/card.js:1890
    ("guiyoujie", CardKind::Guiyoujie, 0, Suit::Heart, 10), // card/swd/card.js:1902
    ("yufulu", CardKind::Yufulu, 0, Suit::Club, 11), // card/swd/card.js:1936
    ("xixueguizhihuan", CardKind::Xixueguizhihuan, 0, Suit::Diamond, 12), // card/swd/card.js:1947
    ("zhufangshenshi", CardKind::Zhufangshenshi, 0, Suit::Spade, 13), // card/swd/card.js:1958
    ("jingleishan", CardKind::Jingleishan, 0, Suit::Heart, 1), // card/swd/card.js:1995
    ("chiyuxi", CardKind::Chiyuxi, 0, Suit::Club, 2), // card/swd/card.js:2076
    ("guangshatianyi", CardKind::Armor, 0, Suit::Diamond, 3), // card/swd/card.js:2159
    ("guilingzhitao", CardKind::Guilingzhitao, 0, Suit::Spade, 4), // card/swd/card.js:2170
    ("qipoguyu", CardKind::WuxingPan, 0, Suit::Heart, 5), // card/swd/card.js:2198
    ("sadengjinhuan", CardKind::Sadengjinhuan, 0, Suit::Club, 6), // card/swd/card.js:2212
    ("sifeizhenmian", CardKind::Sifeizhenmian, 0, Suit::Diamond, 7), // card/swd/card.js:2223
    ("shuchui", CardKind::Shuchui, 0, Suit::Spade, 8), // card/swd/card.js:2235
    ("guiyanfadao", CardKind::Weapon, 2, Suit::Heart, 9), // card/swd/card.js:2247
    ("qiankundai", CardKind::Qiankundai, 0, Suit::Club, 10), // card/swd/card.js:2259
];

/// `U05` `yunchou` cards (`10` items).
pub const U05_YUNCHOU_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 10] = [
    ("caochuanjiejian", CardKind::Caochuanjiejian, 0, Suit::Diamond, 11), // card/yunchou/card.js:4
    ("geanguanhuo", CardKind::Geanguanhuo, 0, Suit::Spade, 12), // card/yunchou/card.js:62
    ("shezhanqunru", CardKind::Shezhanqunru, 0, Suit::Heart, 13), // card/yunchou/card.js:132
    ("suolianjia", CardKind::Suolianjia, 0, Suit::Club, 1), // card/yunchou/card.js:245
    ("chenhuodajie", CardKind::Chenhuodajie, 0, Suit::Diamond, 2), // card/yunchou/card.js:266
    ("fudichouxin", CardKind::Fudichouxin, 0, Suit::Spade, 3), // card/yunchou/card.js:288
    ("shuigong", CardKind::Shuigong, 0, Suit::Heart, 4), // card/yunchou/card.js:333
    ("toulianghuanzhu", CardKind::Toulianghuanzhu, 0, Suit::Club, 5), // card/yunchou/card.js:363
    ("huoshan", CardKind::Le, 0, Suit::Diamond, 6), // card/yunchou/card.js:442
    ("hongshui", CardKind::HongShui, 0, Suit::Spade, 7), // card/yunchou/card.js:505
];

/// `U05` `zhenfa` cards (`11` items).
pub const U05_ZHENFA_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 11] = [
    ("pozhenjue", CardKind::Pozhenjue, 0, Suit::Heart, 8), // card/zhenfa/card.js:4
    ("changshezhen", CardKind::Changshezhen, 0, Suit::Club, 9), // card/zhenfa/card.js:24
    ("tianfuzhen", CardKind::Tianfuzhen, 0, Suit::Diamond, 10), // card/zhenfa/card.js:70
    ("dizaizhen", CardKind::Dizaizhen, 0, Suit::Spade, 11), // card/zhenfa/card.js:96
    ("fengyangzhen", CardKind::Wuzhong, 0, Suit::Heart, 12), // card/zhenfa/card.js:123
    ("yunchuizhen", CardKind::Wuzhong, 0, Suit::Club, 13), // card/zhenfa/card.js:144
    ("qixingzhen", CardKind::Qixingzhen, 0, Suit::Diamond, 1), // card/zhenfa/card.js:165
    ("shepanzhen", CardKind::Wuzhong, 0, Suit::Spade, 2), // card/zhenfa/card.js:204
    ("longfeizhen", CardKind::Wuzhong, 0, Suit::Heart, 3), // card/zhenfa/card.js:237
    ("huyizhen", CardKind::Wuzhong, 0, Suit::Club, 4), // card/zhenfa/card.js:269
    ("niaoxiangzhen", CardKind::Niaoxiangzhen, 0, Suit::Diamond, 5), // card/zhenfa/card.js:320
];

/// `U05` `wuxing` cards (`1` items).
pub const U05_WUXING_ALL_CARDS: [(&str, CardKind, u32, Suit, u8); 1] = [
    ("wuxingpan", CardKind::WuxingPan, 0, Suit::Club, 2), // play/wuxing/card.js:2
];

/// `U05` `gujian` skills (`82` items).
pub const U05_GUJIAN_ALL_SKILLS: [&str; 82] = [
    "ziyangdan", "luyugeng", "xiajiao", "xiajiao2", "mizhilianou", "chunbing",
    "gudonggeng", "qingtuan", "liyutang", "yougeng", "molicha", "yuanbaorou",
    "tanhuadong", "mapodoufu", "yunvyuanshen_skill", "heilonglinpian", "mutoumianju_skill", "gjyuheng_skill",
    "gjyuheng_plus_skill", "gjyuheng_pro_skill", "shihuifen", "g_shihuifen", "g_jinlianzhu", "qingshu",
    "yunyou", "xuanzhen", "xuanci", "xianju", "humeng", "xunjian",
    "lingyan", "woxue", "yange", "lianjing", "cihong", "zhenying",
    "lingnu", "zuiji", "manwu", "xfanghua", "duhun", "yunyin",
    "shishui", "chizhen", "dangping", "xiuhua", "liuying", "yanjia",
    "meiying", "jianwu", "zuizhan", "xidie", "xidie2", "meihu",
    "xlqianhuan", "fumo", "fanyin", "mingkong", "mingkong2", "gjqtwuxie",
    "yuehua", "qinglan", "fanshi", "xuelu", "shahun", "shahun2",
    "xuanning", "xuanning1", "xuanning2", "liuguang", "yangming", "yangming2",
    "zhaolu", "jiehuo", "jiehuo2", "yuling", "yuling1", "yuling2",
    "yuling3", "yuling4", "yuling5", "yuling6",
];

/// `U05` `gwent` skills (`80` items).
pub const U05_GWENT_ALL_SKILLS: [&str; 80] = [
    "gw_aerdeyin", "gw_kunenfayin", "gw_baobaoshu", "gw_nuhaifengbao", "gw_youer", "gw_qinpendayu",
    "gw_birinongwu", "gw_ciguhanshuang", "gw_dieyi", "gw_leizhoushu", "_gainspell", "sqlongyin",
    "sqlongnu", "sqlonghuo", "sqlongwu", "jielue", "gwmaoxian_hengsaite_sha", "gwhuanshuang",
    "gw_xianzumaijiu", "gwjinli", "gwliaotian", "gwhuanbi", "gwminxiang", "gwlangshi",
    "gwjingtian", "gwjingshi", "gwweitu", "gwgouhun", "gwfutian", "gwzhongmo",
    "gwyewu", "g_gw_yewu", "shuangxi", "yangfan", "gwfengshi", "gwchenshui",
    "gwliedi", "julian", "gwfusheng", "gwqinwu", "gwqinwu2", "huanshu",
    "huanshu2", "huanshu3", "gwjieyin", "zhengjun", "gwxuezhan", "bolang",
    "gwjushi", "gwjushi2", "gwfengchi", "lingji", "gwjinyan", "gwshenyu",
    "junchi", "hupeng", "hupeng2", "hupeng3", "hupeng4", "hunmo",
    "shuijian", "yunhuo", "yinzhang", "gwtianbian", "gwxiaoshou", "kuanglie",
    "gwjiquan", "nuhou", "shewu", "gwzhanjiang", "gwzhanjiang3", "gwchuanxin",
    "fengjian", "huandie", "xuezhou", "xuezhou_hp", "fayin", "gwbaquan",
    "huihun", "lanquan",
];

/// `U05` `hearth` skills (`217` items).
pub const U05_HEARTH_ALL_SKILLS: [&str; 217] = [
    "shihuawuqi", "jihuocard2", "hshuanyu", "wxuying", "buwendingyibian_lose", "yibian",
    "hualing", "muyin", "tqchuanyue", "oldhuanjue", "hshuanling", "hsxingyi",
    "asyouzhang", "ylyuchu", "nsaiqi", "nsbeiming", "hsnitai", "hspuzhao",
    "hsyanxin", "hstianqi", "hstianqi_dalian", "hstianqi_shali", "ysjqisha", "yindan",
    "hllingxi", "zhaochao", "zhaochao2", "xiyong", "srjici", "yinzong",
    "tansuo", "lieqi", "azaowu", "shouwang", "shouwang2", "qingtian",
    "qianfu", "qianfu2", "shimo", "lieyang", "lieyang2", "zhuilie",
    "szbianshen", "kekao", "jinhua", "kqizhou", "jingcu", "shengzhang",
    "pyuhuo", "mengye", "mengye2", "lianzhan", "kuixin", "fuhua",
    "fuhua2", "fuhua3", "moxie", "qianhou", "hlongyi", "zhongji",
    "fuwen", "jinzhou", "midian", "xingluo", "yuelu", "yushou",
    "yushou_lose", "yushou_misha", "yushou_huofu", "yushou_leiouke", "qingzun", "ayuling",
    "aoshu", "bzhuiji", "lianjin", "shouji", "yingxi", "yingxi2",
    "hsguimou", "peiyu", "peiyu2", "wzhanyi", "shengteng", "yuansu",
    "chouhuo", "nuyan2", "nuyan", "duxin", "oldduxin", "hsdusu_shinancao",
    "xiubu", "zengli", "mobao", "mobao2", "mobao3", "xianji",
    "xianji2", "xianji3", "tanmi", "xueren", "maoxian", "yiwen",
    "qianghuax", "zhuizong", "xunbao", "xunbao2", "hsbaowu_cangbaotu", "hsbaowu_huangjinyuanhou",
    "hsbaowu_huangjinyuanhou2", "xieneng", "fbeifa", "oldfbeifa", "yufa", "yufa2",
    "yufa3", "bingyan", "shifa", "oldshifa", "yuanzheng", "yulu",
    "oldyulu", "duzhang", "duzhang2", "hannu", "chuidiao", "hhudun",
    "fenlie", "oldfenlie", "nianfu", "xiaorong", "shixu", "jixuan",
    "qianghua", "biri", "stuxi", "stuxi2", "bingdong", "luoshi",
    "ronghuo", "ronghuo2", "fushi", "oldfushi", "moyao", "jiaohui",
    "bimeng", "zhoujiang", "liehun", "xjumo", "malymowang", "oldmalymowang",
    "lingzhou", "mieshi", "xmojian", "xshixin", "enze", "oldenze",
    "chongsheng", "guozai", "guozai2", "guozaix", "guozaix2", "hanshuang",
    "oldhanshuang", "bingshi", "bingshi2", "huanwu", "fengnu", "shengdun",
    "jingmeng", "hswuji", "bingshuang", "yanshu", "oldyanshu", "shengyan",
    "liechao", "qingliu", "zhongjia", "dunji", "fengxing", "fengxian",
    "hsmengjing_mengye", "zhanhou", "oldzhanhou", "shijie", "shengguang", "xinci",
    "anying", "huanjue", "bingjia", "bingjia2", "zuzhou", "mdzhoufu",
    "mdzhoufu2", "xianzhi", "jingxiang", "wlianji", "mengun", "mengun2",
    "jianren", "jihuo", "tuteng_s", "s_tuteng", "tuteng", "zuling",
    "huanfeng", "tuteng_h", "tuteng_lose", "tuteng1", "tuteng2", "tuteng3",
    "tuteng4", "tuteng5", "tuteng6", "tuteng7", "tuteng8", "hongxi",
    "stone_chenshui",
];

/// `U05` `mtg` skills (`19` items).
pub const U05_MTG_ALL_SKILLS: [&str; 19] = [
    "mtg_bingheyaosai_skill", "mtg_duzhao_skill", "mtg_shamolvzhou_skill", "mtg_haidao_skill", "mtg_yixialan_skill", "mtg_shuimomuxue_skill",
    "mtg_feixu_skill", "mtg_youlin_skill", "mtg_linzhongjianta_skill", "mtg_cangbaohaiwan_skill", "mtg_longlushanfeng_skill", "mtg_lindixiliu_skill",
    "mduohun", "lingyong", "mhuanyi", "msuoling", "mtongnian", "mbaizhan",
    "msilian",
];

/// `U05` `swd_card` skills (`135` items).
pub const U05_SWD_CARD_SKILLS: [&str; 135] = [
    "qiankundai", "g_hufu_sha", "g_hufu_shan", "g_hufu_jiu", "zhiluxiaohu", "zhufangshenshi",
    "g_zhufangshenshi", "huanpodan_skill", "g_yuchan_swap", "g_yuchan_equip", "lianyaohu_skill", "g_shencaojie",
    "g_shenmiguo", "yuruyi", "shuchui", "shuchui2", "xuejibingbao", "gouhunluo",
    "gouhunluo2", "jiguanyuan", "g_qinglongzhigui", "g_baishouzhihu", "g_zhuquezhizhang", "g_xuanwuzhihuang",
    "g_huanglinzhicong", "g_cangchizhibi", "cangchizhibi_equip1", "cangchizhibi_equip2", "cangchizhibi_equip3", "cangchizhibi_equip4",
    "cangchizhibi_equip5", "huanglinzhicong_equip1", "huanglinzhicong_equip2", "huanglinzhicong_equip3", "huanglinzhicong_equip4", "huanglinzhicong_equip5",
    "xuanwuzhihuang_equip1", "xuanwuzhihuang_equip2", "xuanwuzhihuang_equip3", "xuanwuzhihuang_equip4", "xuanwuzhihuang_equip5", "zhuquezhizhang_equip1",
    "zhuquezhizhang_equip2", "zhuquezhizhang_equip3", "zhuquezhizhang_equip4", "zhuquezhizhang_equip5", "baishouzhihu_equip1", "baishouzhihu_equip2",
    "baishouzhihu_equip3", "baishouzhihu_equip4", "baishouzhihu_equip5", "qinglongzhigui_equip1", "qinglongzhigui_equip2", "qinglongzhigui_equip3",
    "qinglongzhigui_equip4", "qinglongzhigui_equip5", "kunlunjingc", "lianhua", "shouna", "donghuangzhong",
    "xuanyuanjian", "xuanyuanjian2", "xuanyuanjian3", "pangufu", "haotianta", "shennongding",
    "kongdongyin", "nvwashi", "kongxin", "kongxin2", "qinglianxindeng", "yiluan",
    "hslingjian_xuanfengzhiren_equip1", "hslingjian_xuanfengzhiren_equip2", "hslingjian_xuanfengzhiren_equip3", "hslingjian_xuanfengzhiren_equip3_dist", "hslingjian_xuanfengzhiren_equip4", "hslingjian_xuanfengzhiren_equip4_dist",
    "hslingjian_xuanfengzhiren_equip5", "hslingjian_zhongxinghujia_equip1", "hslingjian_zhongxinghujia_equip2", "hslingjian_zhongxinghujia_equip3", "hslingjian_zhongxinghujia_equip4", "hslingjian_zhongxinghujia_equip5",
    "hslingjian_jinjilengdong_equip1", "hslingjian_jinjilengdong_equip2", "hslingjian_jinjilengdong_equip3", "hslingjian_jinjilengdong_equip4", "hslingjian_jinjilengdong_equip5", "hslingjian_yinmilichang_equip1",
    "hslingjian_yinmilichang_equip2", "hslingjian_yinmilichang_equip3", "hslingjian_yinmilichang_equip4", "hslingjian_yinmilichang_equip5", "hslingjian_xingtigaizao_equip1", "hslingjian_xingtigaizao_equip2",
    "hslingjian_xingtigaizao_equip3", "hslingjian_xingtigaizao_equip4", "hslingjian_xingtigaizao_equip5", "hslingjian_shengxiuhaojiao_equip1", "hslingjian_shengxiuhaojiao_equip2", "hslingjian_shengxiuhaojiao_equip3",
    "hslingjian_shengxiuhaojiao_equip4", "hslingjian_shengxiuhaojiao_equip5", "hslingjian_shijianhuisu_equip1", "hslingjian_shijianhuisu_equip2", "hslingjian_shijianhuisu_equip3", "hslingjian_shijianhuisu_equip4",
    "hslingjian_shijianhuisu_equip5", "jiguanyaoshu_skill", "lingjianduanzao", "_lingjianduanzao", "hslingjian_chaofeng", "hslingjian_xingtigaizao",
    "hslingjian_jinjilengdong", "qinglonglingzhu", "xingjunyan", "baihupifeng", "fengxueren", "chilongya",
    "chilongya2", "chilongfengxue", "shentou", "old_longfan", "longfan", "touzhi",
    "xixue", "guangshatianyi", "nigong", "nigong2", "nigong3", "sadengjinhuan",
    "guiyanfadao", "guiyanfadao2", "tianxianjiu",
];

/// `U05` `swd_char` skills (`230` items).
pub const U05_SWD_CHAR_SKILLS: [&str; 230] = [
    "cyqiaoxie", "cyxianjiang", "gxianyin", "yeying", "juxi", "jiefen",
    "datong", "huodan", "sxianjing", "zhanxing", "kbolan", "kbolan2",
    "hujing", "gaizao", "lingshi", "tiebi", "shenyan", "shenyan2",
    "senluo", "xuanying", "hwendao", "lingfeng", "hxunzhi", "hxunzhi2",
    "lmazui", "lmazui2", "lmazui3", "hyunshen", "hyunshen2", "hlingbo",
    "gtiandao", "jinlin", "jinlin2", "lingyue", "fengze", "huanxia",
    "huajing", "pingxu", "jufu", "bingfeng", "bingfeng2", "yudun",
    "guozao", "heihuo", "yaotong", "yaotong1", "yaotong2", "yaotong3",
    "yaotong4", "pojian", "huajin", "huajin2", "huajin3", "yuchen",
    "bingjian", "rumeng", "lianda", "huiqi", "xianghui", "fzhenwei",
    "shangxi", "fuyan", "fuyan2", "guaili", "guaili2", "xingzhui",
    "lingxian", "shouyin", "sliufeng", "linyun", "linyun2", "linyun3",
    "hutian", "hutian2", "hutian3", "chengjian", "huanxing", "huanxing2",
    "guiying", "suiyan", "ningxian", "xuanyuan", "xuanyuan_ai", "jilve",
    "jilve6", "jilve2", "jilve4", "jilve5", "pozhou", "pozhou2",
    "duanyue", "tuzhen", "mojian", "swdliuhong", "poyue", "poyue2",
    "jianji", "huangyu", "gongshen", "duishi", "lianwu", "mingfu",
    "mufeng", "jiying", "minjing", "touxi", "touxi2", "nlianji",
    "lianji2", "lianji3", "lianji4", "miedao", "miedao1", "miedao2",
    "aojian", "moyan", "wangchen", "wangchen2", "wangchen3", "yihua",
    "youyin", "anlianying", "zhanlu", "huopu", "rexue", "shengshou",
    "huanjian", "moyu", "susheng", "kunlunjing", "kunlunjing1", "kunlunjing2",
    "oldliaoyuan", "oldliaoyuan2", "shehun", "liaoyuan", "liaoyuan2", "liaoyuan3",
    "dunxing", "duanxing", "meihuo", "yishan", "yishan2", "guanhu",
    "poxing", "huanhun", "yinyue", "daofa", "daixing", "daixing2",
    "daixing3", "swd_wuxie", "lqingcheng", "lingxin", "lingwu", "shengong",
    "huajian", "polang", "jikong", "xielei", "pozhen", "tanlin_defence",
    "tanlin", "tanlin2", "tanlin3", "tanlin4", "yunchou", "jqimou",
    "lexue", "lexue1", "lexue2", "swdtianshu2", "swdtianshu3", "swdtianshu",
    "swdtianshu_remove", "luomei", "xingdian", "yulin", "funiao", "xuehuang",
    "zhuyu", "ningshuang", "xielv", "xiaomoyu", "tianhuo", "huanyin",
    "xuanzhou", "tianlun", "lanzhi", "duanyi", "guxing", "guxing1",
    "guxing3", "miles_xueyi", "jifeng", "swd_xiuluo", "mohua2", "duijue",
    "duijue3", "duijue2", "yueren", "busi", "xuying", "yinguo",
    "yinguo3", "guiyan", "guiyan2", "xianyin", "mailun", "mailun31",
    "mailun32", "mailun41", "mailun42", "mailun5", "mailun6", "mailun7",
    "mailun8", "fengming", "wanjun", "huanling", "huanling2", "ljifeng",
    "lxianglong", "zhenjiu", "zhenjiu2", "shoulie", "hudun", "toudan",
    "shending", "poxiao",
];

/// `U05` `yunchou` skills (`19` items).
pub const U05_YUNCHOU_ALL_SKILLS: [&str; 19] = [
    "suolianjia", "g_chenhuodajie", "xicai", "diyjianxiong", "diykuanggu", "xiongzi",
    "yaliang", "diy_jiaoxia", "yiesheng", "guihan", "luweiyan", "luweiyan2",
    "jieyan", "honglian", "diyguhuo", "diychanyuan", "zonghuo", "shaoying",
    "xinfu_qiaosi",
];

/// `U05` `wuxing_play` skills (`2` items).
pub const U05_WUXING_PLAY_SKILLS: [&str; 2] = [
    "_shengke", "wuxingpan_skill",
];

/// `U05` `ow` skills (`103` items).
pub const U05_OW_ALL_SKILLS: [&str; 103] = [
    "woliu", "woliu2", "qianggu", "qianggu2", "dianji", "feitiao",
    "feitiao2", "zhencha", "bshaowei", "pingzhang", "owliyong", "liangou",
    "liangou2", "xiyang", "qinru", "yinshen", "maichong", "mengji",
    "zhongdun", "maoding", "maoding2", "paotai", "paotai2", "paotai3",
    "bfengshi", "bfengshi2", "bfengshi3", "yinbo", "aqianghua", "aqianghua2",
    "shihuo", "shoujia", "shoujia2", "shoujia3", "liudan", "shenqiang",
    "tiandan", "shanguang", "shanguang2", "baoxue", "mianzhen", "mianzhen2",
    "zhiyuan", "duwen", "duwen2", "juji", "juji2", "juji3",
    "dulei", "dulei2", "bingqiang", "bingqiang_remove", "bingqiang2", "bingqiang3",
    "bingqiang4", "bingqiang5", "jidong", "jidong2", "chongzhuang", "tuijin",
    "tuijin2", "jijia", "zihui", "xiandan", "xiandan2", "shouge",
    "tuji", "mujing", "mujing2", "lichang", "lichang2", "zhanlong",
    "zhanlong2", "feiren", "feiren3", "xie", "xie2", "xie3",
    "xie4", "luan", "luan2", "luan3", "luan4", "sheng",
    "sheng2", "yihun", "yihun2", "huoyu", "feidan", "yuedong",
    "huhuan", "kuoyin", "kuoyin2", "guangshu", "guangshu_diamond", "guangshu_heart",
    "guangshu_club", "guangshu_spade", "ziyu", "shouhu", "shanxian", "shanxian2",
    "shanhui",
];

/// `U05` `xianjian` skills (`120` items).
pub const U05_XIANJIAN_ALL_SKILLS: [&str; 120] = [
    "lingquan", "shenwu", "qiongguang", "txianqu", "xunying", "liefeng",
    "yuexing", "yuexing2", "minsha", "lingdi", "xiaoyue", "xiaoyue2",
    "huanlei", "anwugu", "anwugu2", "xtanxi", "linghuo", "guijin",
    "guijin2", "chengxin", "chengxin2", "tianwu", "shiying", "liguang",
    "xiepan", "yujia", "yanshi", "tanhua", "xjyingfeng", "ywuhun",
    "fenglue", "xjzongyu", "fanling", "dujiang", "sheying", "huahu",
    "binxin", "qixia", "jianzhen", "husha", "fenshi", "longhuo",
    "yanzhan", "yanzhan2", "yanzhan3", "yanzhan4", "xjyufeng", "feixia",
    "lueying", "tianjian", "feng", "ya", "song", "longxiang",
    "huxi", "xuanmo", "danqing", "danqing_count", "sheling", "zhaoyao",
    "zhangmu", "zhangmu_ai", "leiyu", "leiyu2", "leiyu3", "leiyu4",
    "feizhua", "lingxue", "diesha", "guijiang", "guijiang2", "guijiang3",
    "fenxing", "diewu", "lingyu", "duxinshu", "feixu", "xuanyan",
    "xuanyan2", "xuanyan3", "ningbin", "xfenxin", "xfenxin2", "luanjian",
    "luanjian2", "ctianfu", "ctianfu2", "ctianfu3", "shuiyun", "shuiyun5",
    "wangyou", "wangyou2", "changnian", "changnian2", "sajin", "jtjubao",
    "duci", "xshuangren", "xshuangren2", "guiyuan", "qijian", "shenmu",
    "qianfang", "qianfang2", "poyun", "poyun2", "zhuyue", "zhuyue2",
    "longxi", "guanri", "tianxian", "runxin", "xjzhimeng", "xjzhimeng2",
    "xjzhimeng3", "tannang", "tuoqiao", "huimeng", "tianshe", "tianshe2",
];

/// `U05` cross-referenced standard/stone skills on generals (`33` items).
pub const U05_CROSS_SKILLS: [&str; 33] = [
    "beige", "biyue", "feiying", "gongxin", "hujia", "jiang",
    "jiuchi", "jueqing", "lianpo", "lieren", "liuli", "longdan",
    "luanji", "mashu", "niepan", "qiangxi", "qianxun", "rejieming",
    "reyingzi", "shangshi", "songci", "stonedraw", "stonesha", "stoneshan",
    "tianxiang", "wushuang", "xinfu_jingxie", "xinliegong", "xinwuyan", "yicong",
    "yijue", "zhenlie", "zhuhai",
];
