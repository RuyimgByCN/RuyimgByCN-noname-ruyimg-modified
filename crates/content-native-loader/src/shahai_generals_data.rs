//! Authoritative `U05 杀海拾遗` 332 upstream generals (+ `stone_yisela`)
//! extracted from `archive/upstream/noname/apps/core/extension/杀海拾遗/character/*/character.js`.

use engine_native::catalog::Faction;
use engine_native::content::standard::Gender;

/// `U05` `gujian` generals (`18` items).
pub const U05_GUJIAN_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 18] = [
    ("gjqt_bailitusu", Faction::Shu, Gender::Male, 4, &["xuelu", "fanshi", "shahun"]), // character/gujian/character.js:3
    ("gjqt_fengqingxue", Faction::Wu, Gender::Female, 3, &["qinglan", "yuehua", "gjqtwuxie"]), // character/gujian/character.js:10
    ("gjqt_xiangling", Faction::Wu, Gender::Female, 3, &["xlqianhuan", "meihu", "xidie"]), // character/gujian/character.js:16
    ("gjqt_fanglansheng", Faction::Wu, Gender::Male, 3, &["fanyin", "mingkong", "fumo"]), // character/gujian/character.js:22
    ("gjqt_yinqianshang", Faction::Qun, Gender::Male, 4, &["zuiji", "zuizhan"]), // character/gujian/character.js:28
    ("gjqt_hongyu", Faction::Shu, Gender::Female, 4, &["jianwu", "meiying"]), // character/gujian/character.js:34
    ("gjqt_yuewuyi", Faction::Wei, Gender::Male, 4, &["yanjia", "xiuhua", "liuying"]), // character/gujian/character.js:41
    ("gjqt_wenrenyu", Faction::Shu, Gender::Female, 4, &["chizhen", "dangping"]), // character/gujian/character.js:47
    ("gjqt_xiayize", Faction::Qun, Gender::Male, 3, &["xuanning", "liuguang", "yangming"]), // character/gujian/character.js:54
    ("gjqt_aruan", Faction::Wu, Gender::Female, 3, &["zhaolu", "jiehuo", "yuling"]), // character/gujian/character.js:60
    ("gjqt_xunfang", Faction::Shu, Gender::Female, 3, &["manwu", "xfanghua"]), // character/gujian/character.js:68
    ("gjqt_ouyangshaogong", Faction::Shu, Gender::Male, 3, &["yunyin", "shishui", "duhun"]), // character/gujian/character.js:74
    ("gjqt_xieyi", Faction::Qun, Gender::Male, 3, &["lingyan", "xunjian", "humeng"]), // character/gujian/character.js:82
    ("gjqt_yanjiaxieyi", Faction::Qun, Gender::Male, 2, &["xianju"]), // character/gujian/character.js:88
    ("gjqt_chuqi", Faction::Qun, Gender::Male, 2, &["xuanci"]), // character/gujian/character.js:95
    ("gjqt_beiluo", Faction::Qun, Gender::Male, 4, &["lingnu", "zhenying", "cihong"]), // character/gujian/character.js:104
    ("gjqt_yunwuyue", Faction::Wei, Gender::Female, 3, &["yange", "woxue", "lianjing"]), // character/gujian/character.js:110
    ("gjqt_cenying", Faction::Shu, Gender::Female, 3, &["yunyou", "xuanzhen", "qingshu"]), // character/gujian/character.js:116
];

/// `U05` `gwent` generals (`52` items).
pub const U05_GWENT_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 52] = [
    ("gw_huoge", Faction::Qun, Gender::Male, 3, &["gwjinli"]), // character/gwent/character.js:3
    ("gw_aisinie", Faction::Wu, Gender::Female, 3, &["huihun"]), // character/gwent/character.js:9
    ("gw_enxier", Faction::Wei, Gender::Male, 4, &["gwbaquan"]), // character/gwent/character.js:15
    ("gw_kaerweite", Faction::Shu, Gender::Male, 4, &["gwjiquan"]), // character/gwent/character.js:22
    ("gw_falanxisika", Faction::Wu, Gender::Female, 3, &["shewu"]), // character/gwent/character.js:28
    ("gw_haluo", Faction::Qun, Gender::Male, 4, &["nuhou"]), // character/gwent/character.js:34
    ("gw_airuiting", Faction::Wei, Gender::Male, 4, &["kuanglie"]), // character/gwent/character.js:41
    ("gw_laduoweide", Faction::Wei, Gender::Male, 4, &["gwxiaoshou"]), // character/gwent/character.js:47
    ("gw_dagong", Faction::Qun, Gender::Male, 4, &["gwtianbian"]), // character/gwent/character.js:53
    ("gw_bulanwang", Faction::Qun, Gender::Male, 4, &["bolang"]), // character/gwent/character.js:60
    ("gw_kuite", Faction::Qun, Gender::Male, 4, &["gwxuezhan"]), // character/gwent/character.js:66
    ("gw_fuertaisite", Faction::Wei, Gender::Male, 3, &["zhengjun"]), // character/gwent/character.js:72
    ("gw_hengsaite", Faction::Wei, Gender::Male, 3, &["jielue"]), // character/gwent/character.js:78
    ("gw_fulisi", Faction::Qun, Gender::Male, 3, &["lanquan"]), // character/gwent/character.js:84
    ("gw_gaier", Faction::Shu, Gender::Male, 3, &["hunmo"]), // character/gwent/character.js:90
    ("gw_jieluote", Faction::Qun, Gender::Male, 6, &["fayin"]), // character/gwent/character.js:97
    ("gw_yenaifa", Faction::Qun, Gender::Female, 3, &["xuezhou"]), // character/gwent/character.js:103
    ("gw_telisi", Faction::Wei, Gender::Female, 3, &["huandie"]), // character/gwent/character.js:109
    ("gw_xili", Faction::Wu, Gender::Female, 3, &["fengjian"]), // character/gwent/character.js:115
    ("gw_luoqi", Faction::Wei, Gender::Male, 4, &["gwzhanjiang"]), // character/gwent/character.js:121
    ("gw_yioufeisi", Faction::Wu, Gender::Male, 4, &["gwchuanxin"]), // character/gwent/character.js:127
    ("gw_aigeleisi", Faction::Wu, Gender::Female, 3, &["gwshenyu"]), // character/gwent/character.js:135
    ("gw_aokeweisite", Faction::Qun, Gender::Male, 4, &["yunhuo"]), // character/gwent/character.js:141
    ("gw_kaxier", Faction::Shu, Gender::Male, 4, &["gwfengchi"]), // character/gwent/character.js:147
    ("gw_luobo", Faction::Qun, Gender::Male, 3, &["junchi"]), // character/gwent/character.js:153
    ("gw_mieren", Faction::Shu, Gender::Male, 3, &["lingji"]), // character/gwent/character.js:159
    ("gw_sanhanya", Faction::Shu, Gender::Male, 3, &["gwjinyan"]), // character/gwent/character.js:165
    ("gw_shanhu", Faction::Qun, Gender::Female, 3, &["shuijian"]), // character/gwent/character.js:171
    ("gw_zhangyujushou", Faction::Wu, Gender::Male, 4, &["gwjushi"]), // character/gwent/character.js:177
    ("gw_zhuoertan", Faction::Wu, Gender::Male, 3, &["hupeng"]), // character/gwent/character.js:183
    ("gw_meizi", Faction::Wei, Gender::Female, 3, &["gwjieyin"]), // character/gwent/character.js:190
    ("gw_aimin", Faction::Wu, Gender::Female, 3, &["huanshu"]), // character/gwent/character.js:196
    ("gw_puxila", Faction::Qun, Gender::Female, 3, &["gwqinwu"]), // character/gwent/character.js:202
    ("gw_xigedelifa", Faction::Qun, Gender::Female, 3, &["gwfusheng"]), // character/gwent/character.js:209
    ("gw_laomaotou", Faction::Qun, Gender::Male, 4, &["gwchenshui"]), // character/gwent/character.js:215
    ("gw_qigaiwang", Faction::Qun, Gender::Male, 4, &["julian"]), // character/gwent/character.js:221
    ("gw_bierna", Faction::Qun, Gender::Female, 3, &["gwfengshi"]), // character/gwent/character.js:228
    ("gw_haizhiyezhu", Faction::Qun, Gender::Male, 4, &["yangfan"]), // character/gwent/character.js:234
    ("gw_nitelila", Faction::Wei, Gender::Male, 4, &["shuangxi"]), // character/gwent/character.js:240
    ("gw_linjing", Faction::Wu, Gender::Male, 4, &["gwyewu"]), // character/gwent/character.js:247
    ("gw_kanbi", Faction::Qun, Gender::Male, 1, &["gwfutian"]), // character/gwent/character.js:253
    ("gw_nvyemo", Faction::Shu, Gender::Female, 3, &["gwgouhun"]), // character/gwent/character.js:259
    ("gw_kairuisi", Faction::Qun, Gender::Female, 3, &["gwweitu"]), // character/gwent/character.js:266
    ("gw_oudimu", Faction::Shu, Gender::Male, 3, &["gwjingshi"]), // character/gwent/character.js:272
    ("gw_shasixiwusi", Faction::Qun, Gender::Male, 4, &["gwjingtian"]), // character/gwent/character.js:278
    ("gw_yioufeisisp", Faction::Wu, Gender::Male, 3, &["gwminxiang"]), // character/gwent/character.js:285
    ("gw_lanbote", Faction::Qun, Gender::Male, 4, &["gwlangshi"]), // character/gwent/character.js:291
    ("gw_fenghuang", Faction::Shu, Gender::Male, 4, &["gwliaotian"]), // character/gwent/character.js:297
    ("gw_diandian", Faction::Wu, Gender::Male, 3, &["gwhuanbi"]), // character/gwent/character.js:303
    ("gw_yisilinni", Faction::Wu, Gender::Female, 3, &["gwhuanshuang"]), // character/gwent/character.js:309
    ("gw_feilafanruide", Faction::Wei, Gender::Male, 3, &["yinzhang"]), // character/gwent/character.js:315
    ("gw_saqiya", Faction::Shu, Gender::Female, 4, &["sqlongwu"]), // character/gwent/character.js:322
];

/// `U05` `hearth` generals (`110` items).
pub const U05_HEARTH_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 110] = [
    ("hs_jaina", Faction::Wei, Gender::Female, 3, &["huopu", "aoshu", "bingjia"]), // character/hearth/character.js:3
    ("hs_lrexxar", Faction::Shu, Gender::Male, 4, &["yushou"]), // character/hearth/character.js:9
    ("hs_wuther", Faction::Qun, Gender::Male, 4, &["fengxian", "rejieming"]), // character/hearth/character.js:15
    ("hs_jgarrosh", Faction::Shu, Gender::Male, 4, &["zhanhou", "qiangxi"]), // character/hearth/character.js:21
    ("hs_malfurion", Faction::Wu, Gender::Male, 4, &["jihuo"]), // character/hearth/character.js:27
    ("hs_guldan", Faction::Wei, Gender::Male, 3, &["moxie", "fuhua", "hongxi"]), // character/hearth/character.js:33
    ("hs_anduin", Faction::Qun, Gender::Male, 3, &["shengguang", "shijie", "anying"]), // character/hearth/character.js:39
    ("hs_sthrall", Faction::Wu, Gender::Male, 4, &["tuteng", "guozai", "zuling"]), // character/hearth/character.js:45
    ("hs_waleera", Faction::Shu, Gender::Female, 3, &["jianren", "mengun", "wlianji"]), // character/hearth/character.js:51
    ("hs_medivh", Faction::Wei, Gender::Male, 3, &["jingxiang", "zuzhou", "mdzhoufu"]), // character/hearth/character.js:58
    ("hs_alleria", Faction::Wu, Gender::Female, 3, &["fengxing", "xinliegong"]), // character/hearth/character.js:64
    ("hs_magni", Faction::Shu, Gender::Male, 4, &["zhongjia", "dunji"]), // character/hearth/character.js:70
    ("hs_liadrin", Faction::Shu, Gender::Female, 4, &["xueren"]), // character/hearth/character.js:76
    ("hs_morgl", Faction::Wu, Gender::Male, 3, &["s_tuteng"]), // character/hearth/character.js:82
    ("hs_khadgar", Faction::Shu, Gender::Male, 3, &["midian", "fuwen", "jinzhou"]), // character/hearth/character.js:88
    ("hs_tyrande", Faction::Wei, Gender::Female, 3, &["yuelu", "xingluo"]), // character/hearth/character.js:94
    ("hs_neptulon", Faction::Wu, Gender::Male, 4, &["liechao", "qingliu"]), // character/hearth/character.js:101
    ("hs_wvelen", Faction::Qun, Gender::Male, 3, &["shengyan", "xianzhi"]), // character/hearth/character.js:107
    ("hs_antonidas", Faction::Wei, Gender::Male, 3, &["yanshu", "bingshuang"]), // character/hearth/character.js:113
    ("hs_alakir", Faction::Wei, Gender::Male, 3, &["fengnu", "shengdun"]), // character/hearth/character.js:119
    ("hs_zhouzhuo", Faction::Qun, Gender::Male, 4, &["yiwen"]), // character/hearth/character.js:125
    ("hs_yngvar", Faction::Qun, Gender::Male, 3, &["huanwu"]), // character/hearth/character.js:131
    ("hs_bchillmaw", Faction::Wei, Gender::Male, 5, &["hanshuang", "bingshi"]), // character/hearth/character.js:137
    ("hs_malorne", Faction::Wu, Gender::Male, 3, &["enze", "chongsheng"]), // character/hearth/character.js:143
    ("hs_malygos", Faction::Wei, Gender::Male, 4, &["malymowang"]), // character/hearth/character.js:149
    ("hs_xuefashi", Faction::Wei, Gender::Male, 2, &["liehun", "xjumo"]), // character/hearth/character.js:155
    ("hs_loatheb", Faction::Wu, Gender::Male, 5, &["duzhang"]), // character/hearth/character.js:161
    ("hs_trueheart", Faction::Qun, Gender::Female, 3, &["qianghuax"]), // character/hearth/character.js:167
    ("hs_sainaliusi", Faction::Wu, Gender::Male, 4, &["chongsheng", "yulu"]), // character/hearth/character.js:173
    ("hs_lrhonin", Faction::Wei, Gender::Male, 4, &["bingyan", "yufa"]), // character/hearth/character.js:179
    ("hs_bolvar", Faction::Wei, Gender::Male, 4, &["yuanzheng", "bzhuiji"]), // character/hearth/character.js:185
    ("hs_fuding", Faction::Wei, Gender::Male, 4, &["shengdun", "fbeifa"]), // character/hearth/character.js:191
    ("hs_xuanzhuanjijia", Faction::Shu, Gender::Male, 2, &["jixuan"]), // character/hearth/character.js:197
    ("hs_ysera", Faction::Wu, Gender::Female, 4, &["bimeng"]), // character/hearth/character.js:203
    ("hs_alextrasza", Faction::Shu, Gender::Female, 5, &["fushi"]), // character/hearth/character.js:209
    ("hs_nozdormu", Faction::Qun, Gender::Male, 5, &["shixu"]), // character/hearth/character.js:215
    ("hs_sapphiron", Faction::Wei, Gender::Male, 4, &["bingdong", "stuxi"]), // character/hearth/character.js:221
    ("hs_kchromaggus", Faction::Wei, Gender::Male, 4, &["fenlie"]), // character/hearth/character.js:227
    ("hs_lreno", Faction::Shu, Gender::Male, 4, &["tanmi"]), // character/hearth/character.js:233
    ("hs_brann", Faction::Shu, Gender::Male, 3, &["qianghua", "mashu"]), // character/hearth/character.js:239
    ("hs_finley", Faction::Wu, Gender::Male, 3, &["maoxian"]), // character/hearth/character.js:245
    ("hs_kcthun", Faction::Qun, Gender::Male, 4, &["luanji", "xianji"]), // character/hearth/character.js:251
    ("hs_anomalus", Faction::Wei, Gender::Male, 4, &["mobao"]), // character/hearth/character.js:257
    ("hs_blingtron", Faction::Shu, Gender::Male, 3, &["zengli", "xiubu"]), // character/hearth/character.js:263
    ("hs_yogg", Faction::Wu, Gender::Male, 4, &["qianhou"]), // character/hearth/character.js:269
    ("hs_xialikeer", Faction::Shu, Gender::Female, 3, &["duxin"]), // character/hearth/character.js:275
    ("hs_pyros", Faction::Shu, Gender::Female, 2, &["pyuhuo"]), // character/hearth/character.js:281
    ("hs_kalimosi", Faction::Wu, Gender::Male, 4, &["kqizhou"]), // character/hearth/character.js:287
    ("hs_zhishigushu", Faction::Shu, Gender::Male, 4, &["jiaohui"]), // character/hearth/character.js:294
    ("hs_zhanzhenggushu", Faction::Wei, Gender::Male, 6, &["biri"]), // character/hearth/character.js:300
    ("hs_ronghejuren", Faction::Shu, Gender::Male, 8, &["ronghuo"]), // character/hearth/character.js:306
    ("hs_shanlingjuren", Faction::Wu, Gender::Male, 8, &["luoshi"]), // character/hearth/character.js:312
    ("hs_aedwin", Faction::Wu, Gender::Male, 4, &["lianzhan"]), // character/hearth/character.js:318
    ("hs_mijiaojisi", Faction::Wu, Gender::Female, 3, &["kuixin"]), // character/hearth/character.js:324
    ("hs_huzhixiannv", Faction::Wu, Gender::Female, 3, &["jingmeng", "qingliu"]), // character/hearth/character.js:330
    ("hs_totemic", Faction::Wu, Gender::Male, 3, &["peiyu"]), // character/hearth/character.js:336
    ("hs_wujiyuansu", Faction::Wei, Gender::Male, 3, &["hswuji"]), // character/hearth/character.js:342
    ("hs_xsylvanas", Faction::Qun, Gender::Female, 3, &["busi", "xshixin", "xmojian"]), // character/hearth/character.js:348
    ("hs_siwangzhiyi", Faction::Qun, Gender::Male, 12, &["mieshi"]), // character/hearth/character.js:354
    ("hs_bilanyoulong", Faction::Wei, Gender::Male, 4, &["lingzhou"]), // character/hearth/character.js:360
    ("hs_jinglinglong", Faction::Wu, Gender::Male, 3, &["moyao"]), // character/hearth/character.js:366
    ("hs_ruanniguai", Faction::Wu, Gender::Male, 3, &["nianfu", "xiaorong"]), // character/hearth/character.js:372
    ("hs_hudunren", Faction::Shu, Gender::Male, 2, &["hhudun"]), // character/hearth/character.js:378
    ("hs_nate", Faction::Wu, Gender::Male, 4, &["chuidiao"]), // character/hearth/character.js:384
    ("hs_jiaziruila", Faction::Wu, Gender::Male, 4, &["hannu"]), // character/hearth/character.js:390
    ("hs_shifazhe", Faction::Wei, Gender::Male, 3, &["shifa"]), // character/hearth/character.js:396
    ("hs_lafamu", Faction::Shu, Gender::Male, 4, &["xieneng"]), // character/hearth/character.js:402
    ("hs_yelise", Faction::Wei, Gender::Female, 3, &["xunbao", "zhuizong"]), // character/hearth/character.js:408
    ("hs_jiawodun", Faction::Wu, Gender::Male, 4, &["jinhua"]), // character/hearth/character.js:414
    ("hs_fandral", Faction::Shu, Gender::Male, 4, &["nuyan", "chouhuo"]), // character/hearth/character.js:421
    ("hs_hallazeal", Faction::Wei, Gender::Male, 4, &["shengteng", "yuansu"]), // character/hearth/character.js:427
    ("hs_enzoth", Faction::Qun, Gender::Male, 4, &["mengye"]), // character/hearth/character.js:433
    ("hs_walian", Faction::Shu, Gender::Male, 4, &["wzhanyi"]), // character/hearth/character.js:439
    ("hs_pengpeng", Faction::Qun, Gender::Male, 4, &["yindan"]), // character/hearth/character.js:445
    ("hs_yashaji", Faction::Qun, Gender::Male, 4, &["ysjqisha"]), // character/hearth/character.js:451
    ("hs_tanghangu", Faction::Shu, Gender::Male, 5, &["zhongji"]), // character/hearth/character.js:458
    ("hs_aya", Faction::Wu, Gender::Female, 3, &["ayuling", "qingzun"]), // character/hearth/character.js:464
    ("hs_barnes", Faction::Shu, Gender::Male, 4, &["hsnitai"]), // character/hearth/character.js:470
    ("hs_kazhakusi", Faction::Shu, Gender::Male, 3, &["lianjin"]), // character/hearth/character.js:476
    ("hs_shaku", Faction::Wei, Gender::Male, 3, &["shouji"]), // character/hearth/character.js:482
    ("hs_laxiao", Faction::Shu, Gender::Male, 3, &["hsguimou", "yingxi", "hlongyi"]), // character/hearth/character.js:488
    ("hs_mojinbaozi", Faction::Wei, Gender::Male, 3, &["jingcu", "shengzhang"]), // character/hearth/character.js:494
    ("hs_shuiwenxuejia", Faction::Wu, Gender::Male, 3, &["kekao"]), // character/hearth/character.js:500
    ("hs_shizugui", Faction::Wu, Gender::Male, 3, &["szbianshen"]), // character/hearth/character.js:506
    ("hs_hemite", Faction::Wu, Gender::Male, 6, &["zhuilie"]), // character/hearth/character.js:512
    ("hs_laila", Faction::Wu, Gender::Male, 3, &["lieyang"]), // character/hearth/character.js:518
    ("hs_selajin", Faction::Shu, Gender::Male, 3, &["qianfu", "shimo"]), // character/hearth/character.js:525
    ("hs_bannabusi", Faction::Wu, Gender::Male, 14, &["qingtian"]), // character/hearth/character.js:531
    ("hs_amala", Faction::Wu, Gender::Female, 3, &["azaowu", "shouwang"]), // character/hearth/character.js:537
    ("hs_yinggencao", Faction::Wu, Gender::Male, 3, &["lieqi"]), // character/hearth/character.js:543
    ("hs_zhihuanhua", Faction::Wei, Gender::Female, 3, &["huanjue"]), // character/hearth/character.js:550
    ("hs_shirencao", Faction::Wu, Gender::Male, 3, &["srjici"]), // character/hearth/character.js:556
    ("hs_kaituozhe", Faction::Wu, Gender::Female, 3, &["tansuo", "yinzong"]), // character/hearth/character.js:562
    ("hs_fachaotuteng", Faction::Wei, Gender::Male, 3, &["xiyong"]), // character/hearth/character.js:569
    ("hs_huolituteng", Faction::Wei, Gender::Male, 3, &["hllingxi"]), // character/hearth/character.js:575
    ("hs_manyututeng", Faction::Wu, Gender::Male, 3, &["zhaochao"]), // character/hearth/character.js:581
    ("hs_tgolem", Faction::Wu, Gender::Male, 4, &["xinwuyan", "guozaix"]), // character/hearth/character.js:587
    ("hs_heifengqishi", Faction::Qun, Gender::Male, 4, &["hstianqi"]), // character/hearth/character.js:594
    ("hs_yangyanwageli", Faction::Qun, Gender::Female, 3, &["hspuzhao", "hsyanxin"]), // character/hearth/character.js:600
    ("hs_aiqinvyao", Faction::Qun, Gender::Female, 4, &["nsaiqi", "nsbeiming"]), // character/hearth/character.js:607
    ("hs_yelinlonghou", Faction::Qun, Gender::Female, 4, &["ylyuchu"]), // character/hearth/character.js:614
    ("hs_yelinchulong", Faction::Qun, Gender::Male, 1, &[]), // character/hearth/character.js:620
    ("hs_ashamoer", Faction::Wei, Gender::Female, 3, &["asyouzhang"]), // character/hearth/character.js:627
    ("hs_fengjianhuanfengzhe", Faction::Wei, Gender::Male, 3, &["tuteng", "huanfeng"]), // character/hearth/character.js:633
    ("hs_taisi", Faction::Wei, Gender::Female, 3, &["hsxingyi", "hshuanling"]), // character/hearth/character.js:639
    ("hs_hajiasha", Faction::Wu, Gender::Female, 3, &["zhoujiang", "muyin"]), // character/hearth/character.js:646
    ("hs_tuoqi", Faction::Shu, Gender::Female, 3, &["tqchuanyue"]), // character/hearth/character.js:652
    ("hs_siwangxianzhi", Faction::Wei, Gender::Male, 3, &["hualing", "yibian"]), // character/hearth/character.js:658
    ("hs_xukongzhiying", Faction::Qun, Gender::Female, 3, &["wxuying"]), // character/hearth/character.js:664
    ("hs_duyaxinshi", Faction::Wei, Gender::Male, 3, &["hshuanyu"]), // character/hearth/character.js:671
];

/// `U05` `mtg` generals (`3` items).
pub const U05_MTG_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 3] = [
    ("mtg_jiding", Faction::Qun, Gender::Male, 4, &["mbaizhan", "msilian"]), // character/mtg/character.js:2
    ("mtg_jiesi", Faction::Wei, Gender::Male, 3, &["mtongnian", "msuoling", "mhuanyi"]), // character/mtg/character.js:8
    ("mtg_lilianna", Faction::Qun, Gender::Female, 3, &["lingyong", "mduohun"]), // character/mtg/character.js:14
];

/// `U05` `ow` generals (`24` items).
pub const U05_OW_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 24] = [
    ("ow_liekong", Faction::Shu, Gender::Female, 3, &["shanxian", "shanhui"]), // character/ow/character.js:3
    ("ow_sishen", Faction::Shu, Gender::Male, 4, &["xiandan", "yihun", "shouge"]), // character/ow/character.js:9
    ("ow_tianshi", Faction::Qun, Gender::Female, 3, &["shouhu", "ziyu", "feiying"]), // character/ow/character.js:15
    ("ow_falaozhiying", Faction::Shu, Gender::Female, 3, &["feidan", "huoyu", "feiying"]), // character/ow/character.js:21
    ("ow_zhixuzhiguang", Faction::Qun, Gender::Female, 3, &["guangshu"]), // character/ow/character.js:27
    ("ow_luxiao", Faction::Wu, Gender::Male, 3, &["yuedong", "kuoyin", "huhuan"]), // character/ow/character.js:33
    ("ow_shibing", Faction::Shu, Gender::Male, 4, &["tuji", "mujing", "lichang"]), // character/ow/character.js:39
    ("ow_yuanshi", Faction::Qun, Gender::Male, 3, &["feiren", "lianpo", "zhanlong"]), // character/ow/character.js:45
    ("ow_chanyata", Faction::Qun, Gender::Male, 3, &["xie", "luan", "sheng"]), // character/ow/character.js:51
    ("ow_dva", Faction::Shu, Gender::Female, 2, &["jijia", "tuijin", "zihui", "chongzhuang"]), // character/ow/character.js:57
    ("ow_mei", Faction::Wei, Gender::Female, 3, &["bingqiang", "jidong", "baoxue"]), // character/ow/character.js:63
    ("ow_ana", Faction::Wei, Gender::Female, 3, &["zhiyuan", "mianzhen", "aqianghua"]), // character/ow/character.js:69
    ("ow_heibaihe", Faction::Qun, Gender::Female, 3, &["juji", "duwen", "dulei"]), // character/ow/character.js:75
    ("ow_maikelei", Faction::Shu, Gender::Male, 4, &["shanguang", "tiandan", "shenqiang"]), // character/ow/character.js:81
    ("ow_kuangshu", Faction::Shu, Gender::Male, 3, &["liudan", "shoujia", "shihuo"]), // character/ow/character.js:87
    ("ow_tuobiang", Faction::Shu, Gender::Male, 3, &["paotai", "maoding"]), // character/ow/character.js:94
    ("ow_baolei", Faction::Qun, Gender::Male, 4, &["bshaowei", "zhencha"]), // character/ow/character.js:100
    ("ow_banzang", Faction::Qun, Gender::Male, 4, &["bfengshi", "yinbo"]), // character/ow/character.js:106
    ("ow_laiyinhate", Faction::Qun, Gender::Male, 4, &["zhongdun", "mengji"]), // character/ow/character.js:112
    ("ow_luba", Faction::Shu, Gender::Male, 4, &["liangou", "xiyang"]), // character/ow/character.js:118
    ("ow_wensidun", Faction::Shu, Gender::Male, 4, &["feitiao", "dianji"]), // character/ow/character.js:124
    ("ow_zhaliya", Faction::Wei, Gender::Female, 4, &["pingzhang", "owliyong"]), // character/ow/character.js:130
    ("ow_heiying", Faction::Wei, Gender::Female, 3, &["qinru", "yinshen", "maichong"]), // character/ow/character.js:137
    ("ow_orisa", Faction::Wu, Gender::Female, 4, &["qianggu", "woliu"]), // character/ow/character.js:143
];

/// `U05` `swd` generals (`79` items).
pub const U05_SWD_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 79] = [
    ("swd_huzhongxian", Faction::Wu, Gender::Male, 3, &["daofa", "xielv", "hujing"]), // character/swd/character.js:3
    ("swd_anka", Faction::Qun, Gender::Male, 3, &["songci", "anlianying"]), // character/swd/character.js:11
    ("swd_septem", Faction::Qun, Gender::Male, 4, &["jiying", "liaoyuan", "yishan"]), // character/swd/character.js:17
    ("swd_kama", Faction::Qun, Gender::Female, 3, &["yueren", "shangshi"]), // character/swd/character.js:23
    ("swd_miles", Faction::Qun, Gender::Male, 4, &["aojian", "miles_xueyi", "mohua2"]), // character/swd/character.js:29
    ("swd_nicole", Faction::Qun, Gender::Female, 3, &["huanjian", "lingwu", "minjing"]), // character/swd/character.js:36
    ("swd_wangsiyue", Faction::Wei, Gender::Female, 3, &["duishi", "biyue"]), // character/swd/character.js:42
    ("swd_weida", Faction::Qun, Gender::Female, 3, &["yueren", "zhenlie", "duijue"]), // character/swd/character.js:48
    ("swd_xuanyuanjianxian", Faction::Qun, Gender::Male, 4, &["pozhou", "huajian", "xuanyuan"]), // character/swd/character.js:54
    ("swd_chenjingchou", Faction::Wu, Gender::Male, 3, &["youyin", "yihua"]), // character/swd/character.js:62
    ("swd_duguningke", Faction::Qun, Gender::Female, 3, &["nlianji", "touxi"]), // character/swd/character.js:68
    ("swd_guyue", Faction::Wei, Gender::Male, 3, &["gtiandao", "gxianyin", "wangchen"]), // character/swd/character.js:75
    ("swd_tuobayuer", Faction::Shu, Gender::Female, 4, &["swdliuhong", "poyue", "niepan"]), // character/swd/character.js:81
    ("swd_yuwentuo", Faction::Shu, Gender::Male, 4, &["wushuang", "xielei", "kunlunjing"]), // character/swd/character.js:88
    ("swd_yuxiaoxue", Faction::Wei, Gender::Female, 3, &["huanhun", "daixing", "yinyue"]), // character/swd/character.js:95
    ("swd_jiliang", Faction::Wu, Gender::Male, 3, &["yunchou", "gongxin", "jqimou"]), // character/swd/character.js:102
    ("swd_shuijing", Faction::Qun, Gender::Female, 4, &["mojian", "duanyue", "tuzhen"]), // character/swd/character.js:108
    ("swd_quxian", Faction::Qun, Gender::Female, 3, &["mojian", "huanxia"]), // character/swd/character.js:114
    ("swd_xiyan", Faction::Qun, Gender::Male, 3, &["jiefen", "datong"]), // character/swd/character.js:120
    ("swd_cheyun", Faction::Wu, Gender::Female, 3, &["cyxianjiang", "cyqiaoxie", "shengong"]), // character/swd/character.js:126
    ("swd_huanyuanzhi", Faction::Qun, Gender::Male, 3, &["swdtianshu", "lanzhi", "mufeng"]), // character/swd/character.js:132
    ("swd_murongshi", Faction::Shu, Gender::Female, 4, &["duanyi", "guxing"]), // character/swd/character.js:138
    ("swd_jipeng", Faction::Wu, Gender::Male, 3, &["reyingzi", "guozao"]), // character/swd/character.js:145
    ("swd_qi", Faction::Qun, Gender::Male, 3, &["yaotong", "heihuo", "pojian"]), // character/swd/character.js:151
    ("swd_luchengxuan", Faction::Wu, Gender::Male, 4, &["ljifeng", "lxianglong"]), // character/swd/character.js:158
    ("swd_xiarou", Faction::Shu, Gender::Female, 3, &["xianghui", "huiqi"]), // character/swd/character.js:164
    ("swd_moye", Faction::Wu, Gender::Female, 3, &["rexue", "liuli", "senluo"]), // character/swd/character.js:170
    ("swd_zhaoyun", Faction::Shu, Gender::Male, 4, &["longdan", "pozhen", "tanlin"]), // character/swd/character.js:177
    ("swd_hengai", Faction::Shu, Gender::Female, 3, &["funiao", "ningxian", "hlingbo"]), // character/swd/character.js:183
    ("swd_duanmeng", Faction::Shu, Gender::Female, 4, &["xuanying", "lieren"]), // character/swd/character.js:189
    ("swd_jiangwu", Faction::Shu, Gender::Male, 4, &["yijue", "dangping"]), // character/swd/character.js:196
    ("swd_tuwei", Faction::Shu, Gender::Male, 3, &["zhanlu", "susheng"]), // character/swd/character.js:203
    ("swd_yeyaxi", Faction::Shu, Gender::Female, 3, &["rexue", "huopu", "shenyan"]), // character/swd/character.js:209
    ("swd_muyun", Faction::Wei, Gender::Male, 4, &["zhuhai", "polang", "jikong"]), // character/swd/character.js:216
    ("swd_lanyin", Faction::Wei, Gender::Female, 3, &["xingdian", "yulin", "luomei"]), // character/swd/character.js:222
    ("swd_zhiyin", Faction::Wei, Gender::Female, 3, &["xuehuang", "ningshuang", "zhuyu"]), // character/swd/character.js:228
    ("swd_qiner", Faction::Wei, Gender::Female, 3, &["huanyin", "tianhuo", "xuanzhou"]), // character/swd/character.js:234
    ("swd_jiuyou", Faction::Wei, Gender::Male, 3, &["lexue"]), // character/swd/character.js:240
    ("swd_duopeng", Faction::Wu, Gender::Male, 3, &["luanji", "reyingzi"]), // character/swd/character.js:246
    ("swd_fengtianling", Faction::Shu, Gender::Male, 4, &["guiyan", "jiang"]), // character/swd/character.js:253
    ("swd_huyue", Faction::Wu, Gender::Female, 3, &["hyunshen", "fengming"]), // character/swd/character.js:259
    ("swd_jialanduo", Faction::Qun, Gender::Male, 4, &["xianyin", "mailun"]), // character/swd/character.js:265
    ("swd_rongshuang", Faction::Wu, Gender::Female, 3, &["suiyan", "duanxing"]), // character/swd/character.js:271
    ("swd_zhuoshanzhu", Faction::Wu, Gender::Male, 4, &["suiyan", "wanjun"]), // character/swd/character.js:277
    ("swd_jiting", Faction::Wei, Gender::Female, 4, &["guanhu", "lingshi"]), // character/swd/character.js:283
    ("swd_sikongyu", Faction::Wu, Gender::Male, 4, &["sliufeng", "linyun", "hutian"]), // character/swd/character.js:290
    ("swd_muyue", Faction::Wei, Gender::Female, 3, &["xingzhui", "lingxian", "shouyin"]), // character/swd/character.js:297
    ("swd_ziqiao", Faction::Shu, Gender::Female, 3, &["guaili", "fuyan"]), // character/swd/character.js:303
    ("swd_fengyu", Faction::Shu, Gender::Male, 4, &["fzhenwei", "shangxi"]), // character/swd/character.js:309
    ("swd_huanglei", Faction::Qun, Gender::Male, 3, &["jilve", "gongshen", "gaizao"]), // character/swd/character.js:316
    ("swd_kendi", Faction::Qun, Gender::Male, 3, &["zhanxing", "kbolan"]), // character/swd/character.js:322
    ("swd_lilian", Faction::Qun, Gender::Female, 3, &["swd_wuxie", "lqingcheng"]), // character/swd/character.js:329
    ("swd_kangnalishi", Faction::Qun, Gender::Male, 1, &["busi", "xuying", "yinguo"]), // character/swd/character.js:336
    ("swd_xuanyuanjiantong", Faction::Qun, Gender::Male, 3, &["chengjian", "huanling"]), // character/swd/character.js:343
    ("swd_huiyan", Faction::Qun, Gender::Male, 4, &["hwendao", "lingfeng", "hxunzhi"]), // character/swd/character.js:350
    ("swd_chenfu", Faction::Qun, Gender::Male, 4, &["xuanzhou", "bingfeng"]), // character/swd/character.js:357
    ("swd_chengyaojin", Faction::Qun, Gender::Male, 4, &["jiuchi", "jufu"]), // character/swd/character.js:364
    ("swd_shanxiaoxiao", Faction::Wu, Gender::Female, 3, &["shehun", "xiaomoyu"]), // character/swd/character.js:371
    ("swd_yuchiyanhong", Faction::Shu, Gender::Female, 3, &["huanxing", "meihuo"]), // character/swd/character.js:377
    ("swd_qinshubao", Faction::Qun, Gender::Male, 3, &["huajing", "pingxu"]), // character/swd/character.js:384
    ("swd_zhanglie", Faction::Qun, Gender::Male, 4, &["huajin", "poxiao"]), // character/swd/character.js:391
    ("swd_hanluo", Faction::Qun, Gender::Male, 5, &["tiebi"]), // character/swd/character.js:399
    ("swd_fu", Faction::Qun, Gender::Male, 5, &["yudun"]), // character/swd/character.js:405
    ("swd_linyue", Faction::Wei, Gender::Male, 3, &["zhenjiu", "lmazui"]), // character/swd/character.js:412
    ("swd_zidashu", Faction::Wu, Gender::Male, 3, &["shoulie", "hudun"]), // character/swd/character.js:419
    ("swd_maixing", Faction::Wu, Gender::Male, 3, &["toudan", "shending"]), // character/swd/character.js:425
    ("swd_fuyan", Faction::Qun, Gender::Male, 4, &["lianda"]), // character/swd/character.js:431
    ("swd_haidapang", Faction::Wu, Gender::Female, 3, &["bingjian", "rumeng"]), // character/swd/character.js:437
    ("swd_shaowei", Faction::Shu, Gender::Female, 3, &["jianji", "huangyu"]), // character/swd/character.js:443
    ("swd_youzhao", Faction::Shu, Gender::Male, 4, &["longdan", "yuchen"]), // character/swd/character.js:450
    ("swd_shangzhang", Faction::Shu, Gender::Male, 4, &["lianwu"]), // character/swd/character.js:457
    ("swd_situqiang", Faction::Shu, Gender::Female, 3, &["fengze", "lingyue", "jinlin"]), // character/swd/character.js:464
    ("swd_chunyuheng", Faction::Wei, Gender::Male, 2, &["jueqing", "shengshou", "xuying"]), // character/swd/character.js:472
    ("swd_yuli", Faction::Wu, Gender::Female, 3, &["lingxin", "tianxiang"]), // character/swd/character.js:479
    ("swd_zhanggao", Faction::Wei, Gender::Male, 4, &["yicong", "poxing"]), // character/swd/character.js:485
    ("swd_shuwaner", Faction::Shu, Gender::Female, 3, &["sxianjing", "huodan"]), // character/swd/character.js:491
    ("swd_xiaohuanglong", Faction::Wei, Gender::Male, 3, &["yeying", "juxi"]), // character/swd/character.js:497
    ("swd_hupo", Faction::Wu, Gender::Male, 3, &["dunxing", "guiying"]), // character/swd/character.js:505
    ("swd_jiangziya", Faction::Wu, Gender::Male, 3, &["mingfu", "tianlun"]), // character/swd/character.js:511
];

/// `U05` `xianjian` generals (`36` items).
pub const U05_XIANJIAN_GENERALS_FULL: [(&str, Faction, Gender, u32, &[&str]); 36] = [
    ("pal_lixiaoyao", Faction::Qun, Gender::Male, 4, &["tianjian", "xjyufeng"]), // character/xianjian/character.js:3
    ("pal_zhaoliner", Faction::Wei, Gender::Female, 3, &["huimeng", "tianshe"]), // character/xianjian/character.js:9
    ("pal_linyueru", Faction::Wei, Gender::Female, 3, &["guiyuan", "qijian"]), // character/xianjian/character.js:15
    ("pal_anu", Faction::Wu, Gender::Female, 3, &["lingdi", "anwugu"]), // character/xianjian/character.js:21
    ("pal_wangxiaohu", Faction::Qun, Gender::Male, 4, &["husha"]), // character/xianjian/character.js:29
    ("pal_sumei", Faction::Shu, Gender::Female, 3, &["sheying", "dujiang", "huahu"]), // character/xianjian/character.js:35
    ("pal_shenqishuang", Faction::Wei, Gender::Female, 3, &["qixia", "jianzhen", "binxin"]), // character/xianjian/character.js:41
    ("pal_jingtian", Faction::Wu, Gender::Male, 3, &["sajin", "jtjubao"]), // character/xianjian/character.js:48
    ("pal_xuejian", Faction::Shu, Gender::Female, 3, &["xshuangren", "shenmu", "duci"]), // character/xianjian/character.js:54
    ("pal_longkui", Faction::Qun, Gender::Female, 3, &["fenxing", "diewu", "lingyu"]), // character/xianjian/character.js:61
    ("pal_zixuan", Faction::Wei, Gender::Female, 3, &["shuiyun", "wangyou", "changnian"]), // character/xianjian/character.js:67
    ("pal_changqing", Faction::Wei, Gender::Male, 4, &["luanjian", "ctianfu"]), // character/xianjian/character.js:73
    ("pal_nangonghuang", Faction::Wei, Gender::Male, 3, &["zhaoyao", "sheling", "zhangmu"]), // character/xianjian/character.js:81
    ("pal_wenhui", Faction::Shu, Gender::Female, 4, &["huxi", "longxiang"]), // character/xianjian/character.js:88
    ("pal_wangpengxu", Faction::Shu, Gender::Female, 3, &["duxinshu", "feixu"]), // character/xianjian/character.js:94
    ("pal_xingxuan", Faction::Wei, Gender::Male, 3, &["feizhua", "leiyu", "lingxue"]), // character/xianjian/character.js:100
    ("pal_leiyuange", Faction::Shu, Gender::Male, 4, &["feng", "ya", "song"]), // character/xianjian/character.js:107
    ("pal_yuntianhe", Faction::Wu, Gender::Male, 4, &["longxi", "zhuyue", "guanri"]), // character/xianjian/character.js:114
    ("pal_hanlingsha", Faction::Shu, Gender::Female, 3, &["tannang", "tuoqiao"]), // character/xianjian/character.js:120
    ("pal_liumengli", Faction::Wei, Gender::Female, 3, &["tianxian", "runxin", "xjzhimeng"]), // character/xianjian/character.js:126
    ("pal_murongziying", Faction::Wei, Gender::Male, 4, &["xuanning", "poyun", "qianfang"]), // character/xianjian/character.js:132
    ("pal_xuanxiao", Faction::Wei, Gender::Male, 4, &["xuanyan", "ningbin", "xfenxin"]), // character/xianjian/character.js:139
    ("pal_jiangyunfan", Faction::Wei, Gender::Male, 4, &["xunying", "liefeng"]), // character/xianjian/character.js:146
    ("pal_tangyurou", Faction::Shu, Gender::Female, 3, &["txianqu", "qiongguang"]), // character/xianjian/character.js:152
    ("pal_longyou", Faction::Wei, Gender::Male, 4, &["yuexing", "minsha"]), // character/xianjian/character.js:158
    ("pal_xiaoman", Faction::Shu, Gender::Female, 3, &["anwugu", "lingquan", "shenwu"]), // character/xianjian/character.js:164
    ("pal_xiahoujinxuan", Faction::Shu, Gender::Male, 3, &["xuanmo", "danqing"]), // character/xianjian/character.js:172
    ("pal_muchanglan", Faction::Wu, Gender::Female, 3, &["feixia", "lueying"]), // character/xianjian/character.js:179
    ("pal_xia", Faction::Shu, Gender::Female, 3, &["xjzongyu", "fanling"]), // character/xianjian/character.js:185
    ("pal_jiangcheng", Faction::Qun, Gender::Male, 4, &["yanzhan", "fenshi"]), // character/xianjian/character.js:192
    ("pal_yuejinzhao", Faction::Wei, Gender::Male, 4, &["ywuhun", "xjyingfeng"]), // character/xianjian/character.js:199
    ("pal_yueqi", Faction::Wei, Gender::Female, 3, &["tianwu", "liguang", "shiying"]), // character/xianjian/character.js:205
    ("pal_mingxiu", Faction::Shu, Gender::Female, 3, &["linghuo", "guijin", "chengxin"]), // character/xianjian/character.js:211
    ("pal_xianqing", Faction::Qun, Gender::Male, 4, &["xtanxi", "xiaoyue"]), // character/xianjian/character.js:217
    ("pal_luozhaoyan", Faction::Shu, Gender::Female, 4, &["fenglue", "tanhua"]), // character/xianjian/character.js:223
    ("pal_jushifang", Faction::Shu, Gender::Male, 3, &["yujia", "xiepan", "yanshi"]), // character/xianjian/character.js:229
];

/// `U05` `yunchou` generals (`10` items).
pub const U05_YUNCHOU_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 10] = [
    ("diy_caocao", Faction::Wei, Gender::Male, 4, &["xicai", "diyjianxiong", "hujia"]), // character/yunchou/character.js:5
    ("diy_weiyan", Faction::Shu, Gender::Male, 4, &["diykuanggu"]), // character/yunchou/character.js:12
    ("diy_sp_zhouyu", Faction::Wu, Gender::Male, 3, &["xiongzi", "yaliang"]), // character/yunchou/character.js:19
    ("diy_luxun", Faction::Wu, Gender::Male, 3, &["shaoying", "zonghuo"]), // character/yunchou/character.js:27
    ("diy_yuji", Faction::Qun, Gender::Male, 3, &["diyguhuo", "diychanyuan"]), // character/yunchou/character.js:34
    ("diy_zhouyu", Faction::Wu, Gender::Male, 3, &["jieyan", "honglian"]), // character/yunchou/character.js:40
    ("diy_caiwenji", Faction::Qun, Gender::Female, 3, &["beige", "guihan"]), // character/yunchou/character.js:47
    ("diy_lukang", Faction::Wu, Gender::Male, 4, &["luweiyan", "qianxun"]), // character/yunchou/character.js:54
    ("diy_zhenji", Faction::Wei, Gender::Female, 3, &["diy_jiaoxia", "yiesheng"]), // character/yunchou/character.js:61
    ("old_majun", Faction::Wei, Gender::Male, 3, &["xinfu_jingxie", "xinfu_qiaosi"]), // character/yunchou/character.js:67
];

/// `U05` `stone` mode companion general (`main/content.js:384-395`).
pub const U05_STONE_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 1] = [
    ("stone_yisela", Faction::Qun, Gender::Female, 4, &["stone_chenshui", "stonesha", "stoneshan", "stonedraw"]),
];
