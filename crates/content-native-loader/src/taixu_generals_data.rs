//! Authentic upstream `L11` (`archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_character.js:6-1489`
//! and `extension_card.js:8-2041`, `extension_skill.js:458`) 320 `txhj_*` NPC/companion generals and 32 cards.

use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::content::standard::Gender;

/// All 320 authentic `L11` (`太虚幻境/extension_character.js:6-1489`) generals: `(id, faction, gender, max_hp, skills)`.
pub const L11_TAIXU_GENERALS: [(&str, Faction, Gender, u32, &[&str]); 320] = [
    ("txhj_reganning", Faction::Wu, Gender::Male, 4, &["qixi", "fenwei"]), // extension_character.js:6
    ("txhj_shenguanyu", Faction::Qun, Gender::Male, 4, &["twwushen", "twwuhun"]), // extension_character.js:7
    ("txhj_spguanyu", Faction::Wei, Gender::Male, 4, &["new_rewusheng", "danji"]), // extension_character.js:8
    ("txhj_caifuren", Faction::Qun, Gender::Female, 3, &["xinqieting", "xianzhou"]), // extension_character.js:9
    ("txhj_liubiao", Faction::Qun, Gender::Male, 4, &["zishou", "rezongshi"]), // extension_character.js:10
    ("txhj_liuqi", Faction::Qun, Gender::Male, 3, &["spwenji", "sptunjiang"]), // extension_character.js:11
    ("txhj_wumazhong", Faction::Wu, Gender::Male, 4, &["xinanjian", "refenxun"]), // extension_character.js:14
    ("txhj_panzhang", Faction::Wu, Gender::Male, 4, &["xinduodao", "jianchu"]), // extension_character.js:15
    ("txhj_wujungongjianshou", Faction::Wu, Gender::Male, 3, &["olzhuiji"]), // extension_character.js:16
    ("txhj_heqi", Faction::Qun, Gender::Male, 4, &["reqizhou", "reshanxi", "reganlu", "rongbei"]), // extension_character.js:17
    ("txhj_lingcao", Faction::Qun, Gender::Male, 4, &["twqueshi", "dujin", "tianshu", "yongjin"]), // extension_character.js:18
    ("txhj_zhuzhi", Faction::Qun, Gender::Male, 5, &["xinanguo"]), // extension_character.js:19
    ("txhj_mosunce", Faction::Qun, Gender::Male, 12, &["rehunzi", "txjiang", "txbenji", "txyongguan"]), // extension_character.js:20
    ("txhj_mouzhouyu", Faction::Qun, Gender::Male, 12, &["refanjian", "manji", "reyingzi", "txyongguan"]), // extension_character.js:21
    ("txhj_motaishici", Faction::Qun, Gender::Male, 11, &["zhuandui", "twjianming", "tianyi", "nsfeixiong", "txyonglve"]), // extension_character.js:22
    ("txhj_redaqiao", Faction::Wu, Gender::Female, 3, &["reguose", "liuli"]), // extension_character.js:23
    ("txhj_moxiaoqiao", Faction::Qun, Gender::Female, 8, &["manji", "xinhongyan", "retianxiang", "dcnutao"]), // extension_character.js:24
    ("txhj_suweihuqiqq", Faction::Qun, Gender::Male, 3, &["txayhanji"]), // extension_character.js:25
    ("txhj_suweihuqiay", Faction::Qun, Gender::Male, 4, &["txayhanji", "fulin"]), // extension_character.js:26
    ("txhj_rexiaoqiao", Faction::Wu, Gender::Female, 3, &["xinhongyan", "retianxiang", "piaoling"]), // extension_character.js:27
    ("txhj_modaqiao", Faction::Qun, Gender::Female, 9, &["twhuangjin", "tiandu", "tx_dqtianzi"]), // extension_character.js:28
    ("txhj_handang", Faction::Qun, Gender::Male, 4, &["txayboji", "txayhanji", "twjiefan"]), // extension_character.js:29
    ("txhj_jiangqing", Faction::Qun, Gender::Male, 4, &["txayboji", "txayhanji", "twxiangyu"]), // extension_character.js:30
    ("txhj_rezhouyu", Faction::Wu, Gender::Male, 3, &["reyingzi", "refanjian"]), // extension_character.js:31
    ("txhj_chengpu", Faction::Qun, Gender::Male, 4, &["twjianming", "chunlao"]), // extension_character.js:32
    ("txhj_huanggai", Faction::Qun, Gender::Male, 4, &["kurou", "zhaxiang"]), // extension_character.js:33
    ("txhj_chenwu", Faction::Qun, Gender::Male, 4, &["twfenming", "fenming", "rexianzhen"]), // extension_character.js:34
    ("txhj_dongxi", Faction::Qun, Gender::Male, 4, &["twyilie", "txayhanji"]), // extension_character.js:35
    ("txhj_lvfan", Faction::Qun, Gender::Male, 4, &["rebeige", "diancai"]), // extension_character.js:36
    ("txhj_lvmeng", Faction::Qun, Gender::Male, 4, &["twxuanhuo", "txkeji", "txmouduan"]), // extension_character.js:37
    ("txhj_zhangzhao", Faction::Qun, Gender::Male, 4, &["twqirang", "guzheng", "drlt_qianjie"]), // extension_character.js:38
    ("txhj_zhanghong", Faction::Qun, Gender::Male, 4, &["txyinli", "zhiman", "rezhijian"]), // extension_character.js:39
    ("txhj_menghuo7", Faction::Qun, Gender::Male, 16, &["tuifeng", "reluoyi", "rechouhai", "txhj_baolian", "rezaiqi"]), // extension_character.js:41
    ("txhj_zhurongx", Faction::Qun, Gender::Female, 6, &["txhj_guihuo", "wurong", "lieren"]), // extension_character.js:42
    ("txhj_wutugu", Faction::Qun, Gender::Male, 21, &["txchengyuan", "sbliegong", "txmanjia", "rezhiheng"]), // extension_character.js:43
    ("txhj_menghuo6", Faction::Qun, Gender::Male, 3, &["spxiangzhen", "rezaiqi", "dualside"]), // extension_character.js:44
    ("txhj_muludawang", Faction::Qun, Gender::Male, 10, &["txzhennu", "tx_yushou", "spxiangzhen", "dualside"]), // extension_character.js:45
    ("txhj_zhurong", Faction::Qun, Gender::Female, 15, &["txhj_guihuo", "wurong", "lieren", "rezaiqi"]), // extension_character.js:46
    ("txhj_gousheng", Faction::Shu, Gender::Male, 12, &["txleizhou", "rejianxiong", "retuxi"]), // extension_character.js:47
    ("txhj_menghuo5", Faction::Qun, Gender::Male, 15, &["shanxie", "reqiangxi", "xinfu_qinguo", "qingxi", "reduodao", "rezaiqi"]), // extension_character.js:48
    ("txhj_manzuzhanxiang", Faction::Qun, Gender::Male, 3, &["juxiang", "xiaoguo", "mobilejingce"]), // extension_character.js:49
    ("txhj_manzuzhanxiangx", Faction::Qun, Gender::Male, 4, &["txjuxiang", "xiaoguo"]), // extension_character.js:50
    ("txhj_menghuo4", Faction::Qun, Gender::Male, 16, &["reshuangxiong", "rejizhi", "jiang", "rezaiqi"]), // extension_character.js:51
    ("txhj_menghuo3", Faction::Qun, Gender::Male, 13, &["zhichi", "redangxian", "xiandeng", "rezaiqi"]), // extension_character.js:52
    ("txhj_mengyou", Faction::Qun, Gender::Male, 7, &["fangquan", "xiangle"]), // extension_character.js:53
    ("txhj_manzucike", Faction::Qun, Gender::Male, 4, &["cihuai", "reqianju"]), // extension_character.js:54
    ("txhj_menghuo2", Faction::Qun, Gender::Male, 5, &["txxiebi", "sbyangwei", "redangxian", "rezaiqi", "dualside"]), // extension_character.js:55
    ("txhj_mangyachang", Faction::Qun, Gender::Male, 9, &["spchijie", "yingjian", "rezhiyi", "dualside"]), // extension_character.js:56
    ("txhj_tengjiabing", Faction::Qun, Gender::Male, 4, &["txmanjia", "spwenji"]), // extension_character.js:57
    ("txhj_manzuhuwei", Faction::Qun, Gender::Male, 3, &["rezhenxing"]), // extension_character.js:58
    ("txhj_menghuo1", Faction::Qun, Gender::Male, 14, &["yiyong", "shanxie", "retieji", "rezaiqi"]), // extension_character.js:59
    ("txhj_manzushoushi", Faction::Qun, Gender::Male, 4, &["txmanzhen", "aocai"]), // extension_character.js:60
    ("txhj_zhaoyuny", Faction::Qun, Gender::Male, 6, &["longdan"]), // extension_character.js:61
    ("txhj_jinhuansanjie", Faction::Qun, Gender::Male, 10, &["jiezi", "txleili", "duanliang", "fuji"]), // extension_character.js:62
    ("txhj_manzuxianfengx", Faction::Qun, Gender::Male, 3, &["kuangfu"]), // extension_character.js:63
    ("txhj_manzuxianfeng", Faction::Qun, Gender::Male, 3, &["redangxian"]), // extension_character.js:64
    ("txhj_manzuyongshi", Faction::Qun, Gender::Male, 4, &["qiangwu"]), // extension_character.js:65
    ("txhj_manzushibing", Faction::Qun, Gender::Male, 4, &["retiaoxin"]), // extension_character.js:66
    ("txhj_manding", Faction::Qun, Gender::Male, 4, &["ganglie", "drlt_qianjie"]), // extension_character.js:67
    ("txhj_manzutanzi", Faction::Qun, Gender::Male, 4, &["txdaoji"]), // extension_character.js:68
    ("txhj_mangu", Faction::Qun, Gender::Female, 3, &["retuxi"]), // extension_character.js:69
    ("txhj_ehuan", Faction::Qun, Gender::Male, 7, &["jiaozi", "reyingzi", "spwenji"]), // extension_character.js:70
    ("txhj_manzujishi", Faction::Qun, Gender::Male, 4, &["lianzhu", "xinjuece"]), // extension_character.js:71
    ("txhj_manzuzhuangding", Faction::Qun, Gender::Male, 4, &["zhongyong"]), // extension_character.js:72
    ("txhj_nanzhongjingrui", Faction::Qun, Gender::Male, 4, &["new_yijue", "yingzi"]), // extension_character.js:73
    ("txhj_nianshou", Faction::Qun, Gender::Male, 10, &["ranshang", "txnianrui", "txmengtai", "txshouyi", "txrenxing"]), // extension_character.js:75
    ("txhj_huanjingdoushi", Faction::Qun, Gender::Male, 4, &["yingjian", "qiaomeng", "zhuandui"]), // extension_character.js:76
    ("txhj_elangx", Faction::Qun, Gender::Male, 5, &["xinkuanggu", "txyicheng"]), // extension_character.js:77
    ("txhj_panjunshizu", Faction::Qun, Gender::Male, 3, &["anjian", "fenxun"]), // extension_character.js:78
    ("txhj_panjunzhenchabing", Faction::Qun, Gender::Male, 2, &["gongxin", "refuyuan"]), // extension_character.js:79
    ("txhj_fennudeyezhu", Faction::Qun, Gender::Male, 4, &["jili", "roulin", "yxs_fanji"]), // extension_character.js:80
    ("txhj_molijue", Faction::Qun, Gender::Male, 21, &["txlangxi", "txkuangyi", "ljxuechi", "zhiheng", "qianglue", "dddqiaoduan"]), // extension_character.js:81
    ("txhj_feixiongjingrui", Faction::Qun, Gender::Male, 4, &["cxyRuiQi", "cxyJingQi", "spmingshi"]), // extension_character.js:82
    ("txhj_handiliuxie", Faction::Qun, Gender::Male, 3, &["tianming", "sphuangen", "txhj_daiyuan"]), // extension_character.js:83
    ("txhj_duanwei", Faction::Qun, Gender::Male, 3, &["langmie", "txhj_wangzu"]), // extension_character.js:84
    ("txhj_kuangcaijun", Faction::Qun, Gender::Male, 4, &["txdaoxi", "zhuixi"]), // extension_character.js:85
    ("txhj_moguosi", Faction::Qun, Gender::Male, 18, &["txtanbei", "xinfu_sidao", "satomi_luodao", "chajue", "dddjiexing", "kyou_zhidian"]), // extension_character.js:86
    ("txhj_wuxi", Faction::Qun, Gender::Male, 3, &["cihuai", "dahe"]), // extension_character.js:87
    ("txhj_situwangyun", Faction::Qun, Gender::Male, 3, &["xinfu_zengdao", "relianji", "zymoucheng", "txhj_daiyuan"]), // extension_character.js:88
    ("txhj_jiawenhe", Faction::Qun, Gender::Male, 7, &["rewansha", "reweimu", "reluanwu", "yjzhenlve", "yjjianshu", "nsyice"]), // extension_character.js:89
    ("txhj_dushi", Faction::Qun, Gender::Male, 3, &["rejuece", "jueqing"]), // extension_character.js:90
    ("txhj_baolveqi", Faction::Qun, Gender::Male, 4, &["dcmoukui", "cxy_BaoYing"]), // extension_character.js:91
    ("txhj_longxiangqi", Faction::Qun, Gender::Male, 4, &["longyin", "huilei"]), // extension_character.js:92
    ("txhj_tdsunjian", Faction::Qun, Gender::Male, 5, &["cxyYingHun", "cxyPoLu", "kuanshi"]), // extension_character.js:93
    ("txhj_jianghuaxiong", Faction::Qun, Gender::Male, 15, &["cxyYaoWu", "cxyMoJun", "cxyMoQu", "sbyangwei", "shizhan"]), // extension_character.js:94
    ("txhj_longxiangjun", Faction::Qun, Gender::Male, 4, &["hgkurou", "dcshushen"]), // extension_character.js:95
    ("txhj_mozhangji", Faction::Qun, Gender::Male, 10, &["cxyJieLve", "cxyMoJun", "txlveming", "gdsanchen", "nslulve"]), // extension_character.js:96
    ("txhj_matengy", Faction::Qun, Gender::Male, 3, &["twxiongzheng", "xiongyi", "mashu"]), // extension_character.js:97
    ("txhj_hubenjun", Faction::Qun, Gender::Male, 4, &["oldfuman", "txhuao"]), // extension_character.js:98
    ("txhj_fanchou", Faction::Qun, Gender::Male, 8, &["xinfu_xingluan", "cxyFanGong", "cxyMoJun", "nsyangwu"]), // extension_character.js:99
    ("txhj_wangfang", Faction::Qun, Gender::Male, 4, &["twkujian", "twshengxi", "yechou"]), // extension_character.js:100
    ("txhj_limeng", Faction::Qun, Gender::Male, 5, &["spyishi", "guanzong", "gnsheque", "dddfenji"]), // extension_character.js:101
    ("txhj_huchier", Faction::Qun, Gender::Male, 3, &["liuli", "quhu"]), // extension_character.js:102
    ("txhj_niufu", Faction::Qun, Gender::Male, 7, &["twjuntun", "twxiongxi", "dcxiaoxi"]), // extension_character.js:103
    ("txhj_fengyaojun", Faction::Qun, Gender::Female, 3, &["cxyFengYing"]), // extension_character.js:104
    ("txhj_dongxie", Faction::Qun, Gender::Female, 5, &["cxyJiaoXia", "cxyTunJun", "jiaojie"]), // extension_character.js:105
    ("txhj_dongmin", Faction::Qun, Gender::Male, 5, &["dclbjiuxian", "tuifeng", "tieji"]), // extension_character.js:106
    ("txhj_dongbai", Faction::Qun, Gender::Female, 4, &["lianzhu", "xiehui", "jsrgshichong"]), // extension_character.js:107
    ("txhj_huangfusongy", Faction::Qun, Gender::Male, 3, &["xinfenyue", "gongxin", "feiying"]), // extension_character.js:108
    ("txhj_feixiongjunyou", Faction::Qun, Gender::Male, 4, &["cxyRuiQi"]), // extension_character.js:109
    ("txhj_feixiongjunzuo", Faction::Qun, Gender::Male, 4, &["cxyJingQi"]), // extension_character.js:110
    ("txhj_xurongy", Faction::Qun, Gender::Male, 5, &["xinfu_xionghuo", "xinfu_shajue", "txhj_daiyuan"]), // extension_character.js:111
    ("txhj_huzheny", Faction::Qun, Gender::Male, 2, &["roulin", "twxiongjun"]), // extension_character.js:112
    ("txhj_jianglijue", Faction::Qun, Gender::Male, 5, &["cxyYangWu", "cxyMoJun", "nsfeixiong"]), // extension_character.js:113
    ("txhj_baolvejun", Faction::Qun, Gender::Male, 3, &["xiadao", "cxy_BaoYing"]), // extension_character.js:114
    ("txhj_dongyue", Faction::Qun, Gender::Male, 6, &["cxyKuangXi", "cxyMoJun", "jiuchi"]), // extension_character.js:115
    ("txhj_xiaoweijunx", Faction::Qun, Gender::Male, 4, &["kaikang"]), // extension_character.js:116
    ("txhj_jinweijunx", Faction::Qun, Gender::Male, 4, &["longdan"]), // extension_character.js:117
    ("txhj_huangjinzhangyan", Faction::Qun, Gender::Male, 7, &["txfeiyan", "lixia", "txmazhan", "xiaoxi_hansui", "txhuangfu"]), // extension_character.js:119
    ("txhj_hanjiangsunjian", Faction::Qun, Gender::Male, 9, &["jiang", "tiaoxin", "shizhi"]), // extension_character.js:120
    ("txhj_huangfusongxx", Faction::Qun, Gender::Male, 6, &["mjmouzhi", "spdengli", "shizhi"]), // extension_character.js:121
    ("txhj_huangjinyuzhong", Faction::Qun, Gender::Male, 2, &["txlianbi"]), // extension_character.js:122
    ("txhj_digongzhangbaox", Faction::Qun, Gender::Male, 25, &["nzry_jieying", "nzry_longnu", "rexingshang", "rezhenxing"]), // extension_character.js:123
    ("txhj_huangfusongx", Faction::Qun, Gender::Male, 6, &["hongde", "shenxing", "bingyi", "feiying"]), // extension_character.js:124
    ("txhj_digongzhangbao", Faction::Qun, Gender::Male, 18, &["tx_moyany", "spshiji", "zhuikong", "luoshen", "txhj_luolei"]), // extension_character.js:125
    ("txhj_gaosheng", Faction::Qun, Gender::Male, 14, &["txxiebi", "tx_modao", "rexianzhen", "wushen"]), // extension_character.js:126
    ("txhj_rengongzhangliang", Faction::Qun, Gender::Male, 18, &["huoji", "rejizhi", "mjmouzhi", "rewurong"]), // extension_character.js:127
    ("txhj_tiangongzhangjiaox", Faction::Qun, Gender::Male, 16, &["buqu", "releiji", "longdan", "chongzhen"]), // extension_character.js:128
    ("txhj_buji", Faction::Qun, Gender::Male, 15, &["jiaozi", "kuangfu", "tx_modao", "zhiheng"]), // extension_character.js:129
    ("txhj_zhangmancheng", Faction::Qun, Gender::Male, 15, &["redangxian", "jiang", "rejunxing", "shibei"]), // extension_character.js:130
    ("txhj_bossbocai", Faction::Qun, Gender::Male, 12, &["ranshang", "fankui", "qiangxi", "moukui"]), // extension_character.js:131
    ("txhj_huangjinqianfengx", Faction::Qun, Gender::Male, 4, &["wushuang", "anguo"]), // extension_character.js:132
    ("txhj_huangjinjingrui", Faction::Qun, Gender::Male, 3, &["twhuangjin", "txhuangfu"]), // extension_character.js:133
    ("txhj_huangjinyushix", Faction::Qun, Gender::Male, 3, &["txhj_guimen", "txleizhen"]), // extension_character.js:134
    ("txhj_huangjinshizu", Faction::Qun, Gender::Male, 3, &["qiangwu", "shixin"]), // extension_character.js:135
    ("txhj_huangjinqibing", Faction::Qun, Gender::Male, 3, &["reluoyi"]), // extension_character.js:136
    ("txhj_huangjintongling", Faction::Qun, Gender::Male, 4, &["reluoyi", "nzry_feijun"]), // extension_character.js:137
    ("txhj_bossdengmao", Faction::Qun, Gender::Male, 30, &["rezhiyi", "jianchu", "benghuai"]), // extension_character.js:138
    ("txhj_huangjinfujiang", Faction::Qun, Gender::Male, 4, &["rejiangchi", "xinfu_sidao"]), // extension_character.js:139
    ("txhj_huangjinyaoshushi", Faction::Qun, Gender::Male, 4, &["twzuhuo", "txleizhen"]), // extension_character.js:140
    ("txhj_huangjinyudang", Faction::Qun, Gender::Male, 3, &["xinduodao"]), // extension_character.js:141
    ("txhj_hanjunluzhix", Faction::Qun, Gender::Male, 4, &["qiaoshui", "qixi"]), // extension_character.js:142
    ("txhj_peiyuanshaox", Faction::Qun, Gender::Male, 6, &["rezhijian", "tianyi"]), // extension_character.js:143
    ("txhj_tiangongzhangjiao", Faction::Qun, Gender::Male, 6, &["txhj_luolei", "yingzi", "txleili"]), // extension_character.js:144
    ("txhj_huangjinqianfeng", Faction::Qun, Gender::Male, 2, &["wushuang", "xinfu_qinguo"]), // extension_character.js:145
    ("txhj_huangjinxiaobing", Faction::Qun, Gender::Male, 2, &["rejinjiu"]), // extension_character.js:146
    ("txhj_huangjindaobing", Faction::Qun, Gender::Male, 2, &["relieren"]), // extension_character.js:147
    ("txhj_hanjunluzhi", Faction::Qun, Gender::Male, 3, &["qiaoshui", "retuxi", "weimu"]), // extension_character.js:148
    ("txhj_tianyu", Faction::Qun, Gender::Male, 4, &["reyingzi", "txzhulian", "twyangshi"]), // extension_character.js:150
    ("txhj_tiankai", Faction::Qun, Gender::Male, 4, &["reyingzi", "txzhulian", "wangxi"]), // extension_character.js:151
    ("txhj_bingzhoulangqi", Faction::Qun, Gender::Male, 3, &["twxiongjun", "mashu"]), // extension_character.js:152
    ("txhj_mayunlu", Faction::Shu, Gender::Female, 4, &["mashu", "twfengpo"]), // extension_character.js:153
    ("txhj_machao", Faction::Shu, Gender::Male, 4, &["mashu", "retieji"]), // extension_character.js:154
    ("txhj_madai", Faction::Shu, Gender::Male, 4, &["mashu", "twqianxi"]), // extension_character.js:155
    ("txhj_mamidi", Faction::Qun, Gender::Male, 7, &["txhj_daiyuan", "lirang"]), // extension_character.js:156
    ("txhj_leibo", Faction::Qun, Gender::Male, 7, &["yingjian", "cuijin", "new_reyaowu"]), // extension_character.js:157
    ("txhj_cike", Faction::Qun, Gender::Male, 3, &["zhuhai", "shangyi"]), // extension_character.js:158
    ("txhj_xianyufu", Faction::Qun, Gender::Male, 10, &["xinkuangfu", "qiaomeng", "txmazhan"]), // extension_character.js:159
    ("txhj_jyyuanshao", Faction::Qun, Gender::Male, 9, &["zhanyi", "zhiyi", "weifeng"]), // extension_character.js:160
    ("txhj_bossquyi", Faction::Qun, Gender::Male, 15, &["jiaozi", "fuqi", "txqyyouji", "jsrgzhenqiao"]), // extension_character.js:161
    ("txhj_beimihu", Faction::Qun, Gender::Female, 17, &["zongkui", "guju", "bmcanshi", "xinfu_lingren", "twxuechang"]), // extension_character.js:162
    ("txhj_nansheme", Faction::Qun, Gender::Male, 4, &["waishi", "renshe", "twyingjia"]), // extension_character.js:163
    ("txhj_kaisa", Faction::Qun, Gender::Male, 5, &["zhengfu", "yiyong", "tongduo"]), // extension_character.js:164
    ("txhj_bianjiangweishi", Faction::Qun, Gender::Male, 4, &["jianchu", "txmazhan"]), // extension_character.js:165
    ("txhj_resunjian", Faction::Wu, Gender::Male, 4, &["gzyinghun", "wulie", "repolu"]), // extension_character.js:166
    ("txhj_zhangzi", Faction::Qun, Gender::Male, 8, &["xinjiewei", "xinfu_limu", "xinjushou"]), // extension_character.js:167
    ("txhj_spdiaochan", Faction::Qun, Gender::Female, 3, &["lihun", "rebiyue"]), // extension_character.js:168
    ("txhj_relvbu", Faction::Qun, Gender::Male, 4, &["wushuang", "liyu"]), // extension_character.js:169
    ("txhj_sunquan", Faction::Wu, Gender::Male, 4, &["zhiheng", "kuanshi"]), // extension_character.js:170
    ("txhj_xganning", Faction::Qun, Gender::Male, 4, &["gnjinfan", "gnsheque"]), // extension_character.js:171
    ("txhj_sufei", Faction::Qun, Gender::Male, 4, &["zhengjian", "gaoyuan"]), // extension_character.js:172
    ("txhj_caocaox", Faction::Qun, Gender::Male, 8, &["xinjuejing", "dujin", "txhj_daiyuan"]), // extension_character.js:173
    ("txhj_caohong", Faction::Qun, Gender::Male, 4, &["kaikang", "twyuanhu", "twliancai"]), // extension_character.js:174
    ("txhj_niufudongxie", Faction::Qun, Gender::Male, 7, &["twjuntun", "twxiongxi", "twxiafeng"]), // extension_character.js:175
    ("txhj_caobao", Faction::Qun, Gender::Male, 5, &["shibei", "kuangfu", "reyingzi"]), // extension_character.js:176
    ("txhj_rezhangfei", Faction::Shu, Gender::Male, 4, &["new_repaoxiao", "twxuhe"]), // extension_character.js:177
    ("txhj_gongsunyue", Faction::Qun, Gender::Male, 6, &["twqianju", "bihuo", "zhuiyi"]), // extension_character.js:178
    ("txhj_gongsunxv", Faction::Qun, Gender::Male, 6, &["jijiu", "bihuo", "zhuide"]), // extension_character.js:179
    ("txhj_songxian", Faction::Qun, Gender::Male, 5, &["txhj_oldhuxiao", "txhj_wangzu"]), // extension_character.js:180
    ("txhj_mozhangliao", Faction::Qun, Gender::Male, 7, &["zhuhai", "twjiange", "weifeng"]), // extension_character.js:181
    ("txhj_gongsunfan", Faction::Qun, Gender::Male, 7, &["spqishe", "twqianju"]), // extension_character.js:182
    ("txhj_yuanshaoyou", Faction::Qun, Gender::Male, 3, &["reluanji", "txhj_daiyuan", "wangzun"]), // extension_character.js:183
    ("txhj_fuwan", Faction::Qun, Gender::Male, 4, &["twmoukui"]), // extension_character.js:184
    ("txhj_liuxie", Faction::Qun, Gender::Male, 3, &["tianming", "mizhao"]), // extension_character.js:185
    ("txhj_lingju", Faction::Qun, Gender::Female, 3, &["jieyuan", "txshixin"]), // extension_character.js:186
    ("txhj_guanping", Faction::Shu, Gender::Male, 4, &["relongyin", "jiezhong"]), // extension_character.js:187
    ("txhj_zhoucang", Faction::Shu, Gender::Male, 4, &["mobilezhongyong"]), // extension_character.js:188
    ("txhj_guanyu", Faction::Shu, Gender::Male, 4, &["new_rewusheng", "new_yijue"]), // extension_character.js:189
    ("txhj_shenganningx", Faction::Qun, Gender::Male, 6, &["drlt_poxi", "drlt_jieying"]), // extension_character.js:190
    ("txhj_xusheng", Faction::Wu, Gender::Male, 4, &["repojun"]), // extension_character.js:191
    ("txhj_liumang", Faction::Qun, Gender::Male, 3, &["zhiyan", "xswuyan"]), // extension_character.js:192
    ("txhj_miheng", Faction::Qun, Gender::Male, 5, &["rekuangcai", "reshejian", "jigong"]), // extension_character.js:193
    ("txhj_hubaoqi", Faction::Qun, Gender::Male, 2, &["qingxi", "fenxun"]), // extension_character.js:194
    ("txhj_sunguan", Faction::Qun, Gender::Male, 6, &["nsxianhai", "twjiange", "jsrgbahu"]), // extension_character.js:195
    ("txhj_zangba", Faction::Wei, Gender::Male, 2, &["twhanyu", "twhengjiang"]), // extension_character.js:196
    ("txhj_houcheng", Faction::Qun, Gender::Male, 4, &["xinniluan", "sbyingzi"]), // extension_character.js:197
    ("txhj_xurong", Faction::Qun, Gender::Male, 20, &["xinfu_xionghuo", "xinfu_shajue", "twxiongjun"]), // extension_character.js:198
    ("txhj_zumao", Faction::Qun, Gender::Male, 5, &["yinbing", "txhj_wangzu"]), // extension_character.js:199
    ("txhj_sunjian", Faction::Qun, Gender::Male, 7, &["gzyinghun", "txhj_daiyuan"]), // extension_character.js:200
    ("txhj_huzhen", Faction::Qun, Gender::Male, 10, &["xinfu_langxi", "jintao", "twxiongjun"]), // extension_character.js:201
    ("txhj_guanqiujian", Faction::Wei, Gender::Male, 5, &["twzhengrong", "twhongju"]), // extension_character.js:202
    ("txhj_wangling", Faction::Wei, Gender::Male, 4, &["twmibei", "twxingqi"]), // extension_character.js:203
    ("txhj_zhugedan", Faction::Wei, Gender::Male, 4, &["gongao", "juyi"]), // extension_character.js:204
    ("txhj_simazhao", Faction::Wei, Gender::Male, 3, &["xinfu_daigong", "xinfu_zhaoxin"]), // extension_character.js:205
    ("txhj_simashi", Faction::Wei, Gender::Male, 4, &["baiyi", "jinglve"]), // extension_character.js:206
    ("txhj_shensimayi", Faction::Qun, Gender::Male, 5, &["renjie", "sbaiyin", "lianpo"]), // extension_character.js:207
    ("txhj_wangyi", Faction::Wei, Gender::Female, 4, &["zhenlie", "miji"]), // extension_character.js:208
    ("txhj_spcaiwenji", Faction::Wei, Gender::Female, 3, &["chenqing", "mozhi"]), // extension_character.js:209
    ("txhj_zhangchunhua", Faction::Wei, Gender::Female, 3, &["jueqing", "shangshi"]), // extension_character.js:210
    ("txhj_qinyilu", Faction::Qun, Gender::Male, 4, &["new_reqingnang", "zhaohuo", "txyirang"]), // extension_character.js:211
    ("txhj_dufuren", Faction::Qun, Gender::Female, 3, &["yise", "shunshi"]), // extension_character.js:212
    ("txhj_spcaocao", Faction::Qun, Gender::Male, 2, &["twlingfa"]), // extension_character.js:213
    ("txhj_houkai", Faction::Qun, Gender::Male, 4, &["mingjian", "reyingzi"]), // extension_character.js:214
    ("txhj_xuelan", Faction::Qun, Gender::Male, 4, &["liegong", "feiying"]), // extension_character.js:215
    ("txhj_yanbaihu", Faction::Qun, Gender::Male, 4, &["zhidao", "jili"]), // extension_character.js:216
    ("txhj_liuyao", Faction::Qun, Gender::Male, 3, &["xinfu_kannan"]), // extension_character.js:217
    ("txhj_wanglang", Faction::Qun, Gender::Male, 3, &["gushe", "jici"]), // extension_character.js:218
    ("txhj_guanyinping", Faction::Shu, Gender::Female, 4, &["huxiao", "xueji"]), // extension_character.js:219
    ("txhj_zhangxingcai", Faction::Shu, Gender::Female, 4, &["shenxian", "qiangwu"]), // extension_character.js:220
    ("txhj_spsunshangxiang", Faction::Shu, Gender::Female, 4, &["liangzhu", "xiaoji"]), // extension_character.js:221
    ("txhj_zhugeguo", Faction::Shu, Gender::Female, 3, &["twqirang", "twyuhua"]), // extension_character.js:222
    ("txhj_huangyueying", Faction::Shu, Gender::Female, 3, &["rejizhi", "reqicai"]), // extension_character.js:223
    ("txhj_zhugeliang", Faction::Shu, Gender::Male, 3, &["reguanxing", "kongcheng"]), // extension_character.js:224
    ("txhj_dengai", Faction::Wei, Gender::Male, 4, &["retuntian", "zaoxian"]), // extension_character.js:225
    ("txhj_zhonghui", Faction::Wei, Gender::Male, 4, &["requanji", "zili"]), // extension_character.js:226
    ("txhj_jiangwei", Faction::Shu, Gender::Male, 4, &["retiaoxin", "zhiji"]), // extension_character.js:227
    ("txhj_luocha", Faction::Qun, Gender::Female, 3, &["tx_modao", "tx_yushou", "yizhong", "tx_moyany"]), // extension_character.js:228
    ("txhj_yecha", Faction::Qun, Gender::Male, 3, &["tx_modao", "tx_mojian", "bazhen", "tx_danshu"]), // extension_character.js:229
    ("txhj_zoudan", Faction::Qun, Gender::Male, 8, &["twqianju", "mashu"]), // extension_character.js:230
    ("txhj_baimaxiaoqi", Faction::Qun, Gender::Male, 4, &["yicong", "txhj_oldhuxiao"]), // extension_character.js:231
    ("txhj_huanjingnantong", Faction::Qun, Gender::Male, 3, &["huaizi"]), // extension_character.js:232
    ("txhj_baimayicong", Faction::Qun, Gender::Male, 4, &["qiaomeng", "reyingzi"]), // extension_character.js:233
    ("txhj_xianzhenying", Faction::Qun, Gender::Male, 3, &["xianzhen", "jinjiu"]), // extension_character.js:234
    ("txhj_huanjingshouhutongzi", Faction::Qun, Gender::Female, 2, &[]), // extension_character.js:237
    ("txhj_elang", Faction::Qun, Gender::Male, 4, &["xinkuanggu"]), // extension_character.js:238
    ("txhj_huanjingweishi", Faction::Qun, Gender::Male, 4, &["yingjian", "qiaomeng", "lieren"]), // extension_character.js:239
    ("txhj_tonghu", Faction::Qun, Gender::Male, 4, &["wushuang", "mengjin"]), // extension_character.js:240
    ("txhj_huanxiangbingnv", Faction::Qun, Gender::Female, 2, &["mashu"]), // extension_character.js:241
    ("txhj_huanxiangbingnan", Faction::Qun, Gender::Male, 2, &["mashu"]), // extension_character.js:242
    ("txhj_shouhudizi", Faction::Qun, Gender::Male, 3, &["zhidao"]), // extension_character.js:243
    ("txhj_lingjitongren", Faction::Qun, Gender::Female, 6, &["txhj_daiyuan"]), // extension_character.js:244
    ("txhj_fuguangtongren", Faction::Qun, Gender::Female, 5, &["txhj_daiyuan"]), // extension_character.js:245
    ("txhj_shenjunbaima", Faction::Qun, Gender::Female, 3, &["qiaomeng", "yicong"]), // extension_character.js:246
    ("txhj_guojiahuanxiang", Faction::Wei, Gender::Male, 4, &["tiandu", "new_reyiji"]), // extension_character.js:247
    ("txhj_zhicaihuanxiang", Faction::Wei, Gender::Male, 4, &["tiandu", "xianfu", "chouce"]), // extension_character.js:248
    ("txhj_wolonghuanxiang", Faction::Shu, Gender::Male, 4, &["qice", "bazhen", "zhenlue"]), // extension_character.js:249
    ("txhj_fengchuhuanxiang", Faction::Shu, Gender::Male, 4, &["rejieming", "rekanpo", "zhenlue"]), // extension_character.js:250
    ("txhj_zhouyuhuanxiang", Faction::Wu, Gender::Male, 4, &["refanjian", "qingzhongx"]), // extension_character.js:251
    ("txhj_luxunhuanxiang", Faction::Wu, Gender::Male, 4, &["relianying", "weijing"]), // extension_character.js:252
    ("txhj_jiaxuhuanxiang", Faction::Qun, Gender::Male, 4, &["luanwu", "wansha", "weimu"]), // extension_character.js:253
    ("txhj_dianweihuanxiang", Faction::Wei, Gender::Male, 4, &["reqiangxi", "xinkuanggu"]), // extension_character.js:254
    ("txhj_xuchuhuanxiang", Faction::Wei, Gender::Male, 4, &["reluoyi"]), // extension_character.js:255
    ("txhj_guanyuhuanxiang", Faction::Shu, Gender::Male, 4, &["new_rewusheng", "new_yijue"]), // extension_character.js:256
    ("txhj_zhangfeihuanxiang", Faction::Shu, Gender::Male, 4, &["new_repaoxiao", "txhj_zuijiu"]), // extension_character.js:257
    ("txhj_ganninghuanxiang", Faction::Wu, Gender::Male, 4, &["qixi", "fenwei"]), // extension_character.js:258
    ("txhj_lingtonghuanxiang", Faction::Wu, Gender::Male, 4, &["rexianzhen", "fenwei"]), // extension_character.js:259
    ("txhj_lvbuhuanxiang", Faction::Qun, Gender::Male, 4, &["wushuang", "repojun", "txhj_baoji"]), // extension_character.js:260
    ("txhj_caorenhuanxiang", Faction::Wei, Gender::Male, 5, &["xinjushou", "xinjiewei", "lizhan"]), // extension_character.js:261
    ("txhj_zhaoyunhuanxiang", Faction::Shu, Gender::Male, 5, &["longdan", "chongzhen", "mashu"]), // extension_character.js:262
    ("txhj_zhoutaihuanxiang", Faction::Wu, Gender::Male, 4, &["buqu", "yinghun"]), // extension_character.js:263
    ("txhj_shangjianghuanxiang", Faction::Qun, Gender::Male, 7, &["mengjin", "yaowu", "kuangfu"]), // extension_character.js:264
    ("txhj_hunduntongren", Faction::Qun, Gender::Male, 4, &["rexingluan", "txhj_juhun"]), // extension_character.js:265
    ("txhj_moou", Faction::Qun, Gender::Male, 10, &["rexingluan", "txhj_guimei", "txhj_baoji"]), // extension_character.js:266
    ("txhj_mowangzhizhong", Faction::Qun, Gender::Male, 6, &["xinfu_langxi", "jintao"]), // extension_character.js:267
    ("txhj_chi", Faction::Qun, Gender::Male, 2, &["kuangfu", "txhj_baoji"]), // extension_character.js:268
    ("txhj_mei", Faction::Qun, Gender::Male, 2, &["kuangfu", "txhj_baoji"]), // extension_character.js:269
    ("txhj_wang", Faction::Qun, Gender::Male, 2, &["kuangfu", "txhj_baoji"]), // extension_character.js:270
    ("txhj_liang", Faction::Qun, Gender::Male, 2, &["kuangfu", "txhj_baoji"]), // extension_character.js:271
    ("txhj_huanshenguanyu", Faction::Qun, Gender::Male, 5, &["retieji", "repojun", "jiaozi", "wushen"]), // extension_character.js:272
    ("txhj_huanshenlvmeng", Faction::Qun, Gender::Male, 4, &["benyu", "keji", "shelie", "gongxin"]), // extension_character.js:273
    ("txhj_huanshenzhouyu", Faction::Qun, Gender::Male, 4, &["reyingzi", "refanjian", "qinyin", "yeyan"]), // extension_character.js:274
    ("txhj_huanshenzhuge", Faction::Qun, Gender::Male, 4, &["reguanxing", "kongcheng", "qice", "qingxian"]), // extension_character.js:275
    ("txhj_huanshencaocao", Faction::Qun, Gender::Male, 5, &["rejianxiong", "yuce", "fuli", "zhichi"]), // extension_character.js:276
    ("txhj_huanshenlvbu", Faction::Qun, Gender::Male, 5, &["xinliegong", "jigong", "baonu", "shenfen"]), // extension_character.js:277
    ("txhj_huanshenzhaoyun", Faction::Qun, Gender::Male, 4, &["kunfen", "reyajiao", "juejing", "longhun"]), // extension_character.js:278
    ("txhj_huanshensima", Faction::Qun, Gender::Male, 4, &["refankui", "reguicai", "rejizhi", "rezhiheng"]), // extension_character.js:279
    ("txhj_yongwuhuanxiang", Faction::Qun, Gender::Male, 6, &["zhuiji", "wushuang", "jiuchi", "nzry_shicai"]), // extension_character.js:280
    ("txhj_jianshouhuanxiang", Faction::Qun, Gender::Male, 8, &["zhuiji", "hanyong", "shibei", "bazhen"]), // extension_character.js:281
    ("txhj_huoxinhuanxiang", Faction::Qun, Gender::Male, 5, &["retuxi", "juece", "mieji", "wangzun"]), // extension_character.js:282
    ("txhj_manyanhuanxiang", Faction::Qun, Gender::Male, 5, &["jianying", "duwu", "qiangzhi"]), // extension_character.js:283
    ("txhj_yinhuanxiang", Faction::Qun, Gender::Male, 4, &["rebiyue", "mingce", "jujian"]), // extension_character.js:284
    ("txhj_yanghuanxiang", Faction::Qun, Gender::Male, 5, &["xinkuanggu", "jianchu"]), // extension_character.js:285
    ("txhj_heiwuchanghuanxiang", Faction::Qun, Gender::Male, 6, &["duwu", "txhj_baolian"]), // extension_character.js:286
    ("txhj_baiwuchanghuanxiang", Faction::Qun, Gender::Male, 6, &["yingjian", "txhj_zuijiu"]), // extension_character.js:287
    ("txhj_niutouhuanxiang", Faction::Qun, Gender::Male, 6, &["xiangle", "refankui", "txhj_beiming"]), // extension_character.js:288
    ("txhj_mamianhuanxiang", Faction::Qun, Gender::Male, 6, &["juece", "retuxi", "txhj_beiming"]), // extension_character.js:289
    ("txhj_qianchaotongren", Faction::Qun, Gender::Male, 6, &["bazhen", "shixin", "xinfu_langxi"]), // extension_character.js:290
    ("txhj_hanlingdi", Faction::Qun, Gender::Male, 6, &["zaiqi", "chouhai", "yongsi", "txhj_daiyuan"]), // extension_character.js:291
    ("txhj_baxiaoweishouling", Faction::Qun, Gender::Male, 4, &["quji", "yinbing"]), // extension_character.js:292
    ("txhj_shiemoxing", Faction::Qun, Gender::Male, 99, &["txhj_guimei", "txhj_luolei", "txhj_guihuo"]), // extension_character.js:293
    ("txhj_wujiaoxinmo", Faction::Qun, Gender::Male, 8, &["rejieming", "olniepan", "drlt_xiongluan", "rehuashen", "xinsheng"]), // extension_character.js:294
    ("txhj_huanjinglingzhu", Faction::Qun, Gender::Male, 12, &["retieji", "xinkuanggu", "olniepan", "qiangzhi", "txhj_zuijiu"]), // extension_character.js:295
    ("txhj_tongrenjumo", Faction::Qun, Gender::Male, 10, &["yizhong", "yongsi", "txhj_juhun"]), // extension_character.js:296
    ("txhj_tiaopibubu", Faction::Qun, Gender::Male, 25, &["new_repaoxiao", "fulin", "ranshang", "xiangle", "txhj_baoji"]), // extension_character.js:297
    ("txhj_tianjijushou", Faction::Qun, Gender::Male, 8, &["new_reqingnang", "txhj_kuangbao", "txhj_tunshi", "txhj_chuanyun"]), // extension_character.js:298
    ("txhj_huangjinxinjun", Faction::Qun, Gender::Male, 3, &["txhj_huangjin", "txhj_zhouzu"]), // extension_character.js:301
    ("txhj_guanhai", Faction::Qun, Gender::Male, 4, &["txhj_huangjin", "qixi", "txhj_zhouzu"]), // extension_character.js:302
    ("txhj_peiyuanshao", Faction::Qun, Gender::Male, 4, &["txhj_huangjin", "txhj_zuijiu", "xinfu_xingluan"]), // extension_character.js:303
    ("txhj_huangjinmengqi", Faction::Qun, Gender::Male, 4, &["txhj_huangjin"]), // extension_character.js:304
    ("txhj_dengmao", Faction::Qun, Gender::Male, 6, &["txhj_huangjin", "shensu", "zhidao", "xinfu_xingluan"]), // extension_character.js:305
    ("txhj_dongzhuo", Faction::Qun, Gender::Male, 8, &["dujin", "benghuai", "jiaozi", "txhj_daiyuan"]), // extension_character.js:306
    ("txhj_bocai", Faction::Qun, Gender::Male, 6, &["txhj_huangjin", "dujin", "duwu"]), // extension_character.js:307
    ("txhj_huangjinyaoshi", Faction::Qun, Gender::Male, 4, &["txhj_huangjin", "txhj_guimen", "txhj_yaoshu"]), // extension_character.js:308
    ("txhj_huangjinyushi", Faction::Qun, Gender::Male, 6, &["txhj_huangjin", "txhj_guimen", "txhj_zhoufa"]), // extension_character.js:309
    ("txhj_huangfusong", Faction::Qun, Gender::Male, 6, &["dujin", "yeyan", "lianpo"]), // extension_character.js:310
    ("txhj_zhujun", Faction::Qun, Gender::Male, 6, &["yizhong", "zhanyi", "repojun"]), // extension_character.js:311
    ("txhj_changyuanshibing", Faction::Qun, Gender::Male, 6, &["txhj_shiyuan"]), // extension_character.js:312
    ("txhj_huanyanzhangliang", Faction::Qun, Gender::Male, 16, &["nzry_chenglve", "txhj_renwang", "txhj_renfang"]), // extension_character.js:313
    ("txhj_huanyanzhangbao", Faction::Qun, Gender::Male, 12, &["leiji", "qiangxi", "txhj_dizhou", "txhj_didun"]), // extension_character.js:314
    ("txhj_huanyanzhangjiaoshi", Faction::Qun, Gender::Male, 12, &["leiji", "txhj_huangjin", "txhj_wangyuan", "txhj_shiyuan", "txhj_zhuhun"]), // extension_character.js:315
    ("txhj_huanyanzhangjiaolei", Faction::Qun, Gender::Male, 12, &["leiji", "qiangxi", "txhj_zhoulei"]), // extension_character.js:316
    ("txhj_huanyanzhangjiaohuo", Faction::Qun, Gender::Male, 15, &["leiji", "txhj_huangjin", "lihuo", "txhj_guiyan", "huoji"]), // extension_character.js:317
    ("txhj_huanyanzhangjiaofeng", Faction::Qun, Gender::Male, 15, &["leiji", "txhj_huangjin", "qiangxi", "txhj_xiefeng"]), // extension_character.js:318
    ("txhj_huanguan", Faction::Qun, Gender::Male, 3, &["txhj_xiehui"]), // extension_character.js:321
    ("txhj_huanguandaofushou", Faction::Qun, Gender::Male, 4, &["txhj_huangmen", "txhj_xiehui", "kuangfu"]), // extension_character.js:322
    ("txhj_hetaihou", Faction::Qun, Gender::Female, 3, &["xinfu_lingren", "fenyin", "zhendu", "txhj_suoming"]), // extension_character.js:323
    ("txhj_dongtaihou", Faction::Qun, Gender::Female, 3, &["xinchenhui", "qiangxi", "xinfu_songsang", "fenji"]), // extension_character.js:324
    ("txhj_baohong", Faction::Qun, Gender::Male, 8, &["twlinglu", "txhj_xiehui"]), // extension_character.js:325
    ("txhj_xiaoweijun", Faction::Qun, Gender::Male, 4, &["txhj_yishi", "kaikang"]), // extension_character.js:326
    ("txhj_chunyuqiong", Faction::Qun, Gender::Male, 12, &["txhj_jinwei", "txhj_jinxing", "jiuchi"]), // extension_character.js:327
    ("txhj_jinweijun", Faction::Qun, Gender::Male, 6, &["txhj_jinwei", "txhj_yishi", "longdan"]), // extension_character.js:328
    ("txhj_zhaorong", Faction::Qun, Gender::Male, 4, &["txzhulian", "txhj_xiehui"]), // extension_character.js:329
    ("txhj_zhaozhong", Faction::Qun, Gender::Male, 12, &["txhj_huangmen", "txhj_jinxing", "txhj_xiehui", "txhj_daiyuan", "huaizi"]), // extension_character.js:330
    ("txhj_yuanshao", Faction::Qun, Gender::Male, 6, &["zhanyi", "zhiyi", "weifeng", "txhj_zhanhuo"]), // extension_character.js:331
    ("txhj_yuanshu", Faction::Qun, Gender::Male, 4, &["duanliang", "quhu", "xinfu_daigong", "xinfu_fuyin"]), // extension_character.js:332
    ("txhj_caocao", Faction::Qun, Gender::Male, 6, &["kaikang", "redangxian", "nzry_kuizhu", "txhj_jiaoxie"]), // extension_character.js:333
    ("txhj_xuyou", Faction::Qun, Gender::Male, 3, &["nzry_chenglve", "nzry_shicai", "nzry_cunmu"]), // extension_character.js:334
    ("txhj_jianshuo", Faction::Qun, Gender::Male, 8, &["txhj_jinwei", "txhj_jinxing", "txhj_zhuosheng", "txzhulian"]), // extension_character.js:335
    ("txhj_huanyanzhangrang", Faction::Qun, Gender::Male, 11, &["txhj_huangmen", "txhj_quanqin", "taoluan", "shefu"]), // extension_character.js:336
    ("txhj_huanyanhejin", Faction::Qun, Gender::Male, 10, &["txhj_mouqiang", "txhj_zhuosheng", "twlinglu", "mouzhu"]), // extension_character.js:337
];

/// All 32 authentic `L11` (`太虚幻境/extension_card.js:8-2041` & `extension_skill.js:458`) cards & card-backed skills:
/// `(id, kind, weapon_range, suit, rank)`.
pub const L11_TAIXU_CARDS: [(&str, CardKind, u32, Suit, u8); 32] = [
    ("txhj_xingtianpojunfu", CardKind::Weapon, 4, Suit::Diamond, 12), // extension_card.js:8
    ("txhj_zhuge", CardKind::Weapon, 1, Suit::Club, 1), // extension_card.js:22
    ("txhj_qilingong", CardKind::Weapon, 5, Suit::Heart, 5), // extension_card.js:108
    ("txhj_fangtianhuaji", CardKind::Weapon, 4, Suit::Diamond, 12), // extension_card.js:171
    ("txhj_zhijizhibi", CardKind::Wuzhong, 0, Suit::Club, 3), // extension_card.js:233
    ("txhj_yuanjiao", CardKind::Wuzhong, 0, Suit::Heart, 9), // extension_card.js:305
    ("txhj_yiyi", CardKind::Wuzhong, 0, Suit::Heart, 4), // extension_card.js:334
    ("txhj_xuwangzhimian", CardKind::WuxingPan, 0, Suit::Club, 4), // extension_card.js:379
    ("txhj_piliche", CardKind::Weapon, 9, Suit::Diamond, 9), // extension_card.js:438
    ("txhj_liulongcanjia", CardKind::HorsePlus, 0, Suit::Heart, 13), // extension_card.js:501
    ("txhj_leijimu", CardKind::Weapon, 4, Suit::Spade, 6), // extension_card.js:580
    ("txhj_juechenjinge", CardKind::HorsePlus, 0, Suit::Spade, 5), // extension_card.js:642
    ("txhj_jiedao", CardKind::Duel, 0, Suit::Club, 12), // extension_card.js:704
    ("txhj_huoshaolianying", CardKind::ArcheryAttack, 0, Suit::Heart, 3), // extension_card.js:775
    ("txhj_guofengyupao", CardKind::Armor, 0, Suit::Spade, 9), // extension_card.js:821
    ("txhj_feilong", CardKind::Weapon, 2, Suit::Spade, 2), // extension_card.js:880
    ("txhj_dinglanyemingzhu", CardKind::WuxingPan, 0, Suit::Diamond, 6), // extension_card.js:923
    ("txhj_diaohu", CardKind::Wuzhong, 0, Suit::Heart, 2), // extension_card.js:951
    ("txhj_chiyanzhenhunqin", CardKind::Weapon, 4, Suit::Diamond, 1), // extension_card.js:1013
    ("txhj_caomu", CardKind::Dismantle, 0, Suit::Spade, 10), // extension_card.js:1076
    ("txhj_lingbaoxianhu", CardKind::Weapon, 3, Suit::Heart, 7), // extension_card.js:1123
    ("txhj_chongyingshenfu", CardKind::Armor, 0, Suit::Club, 2), // extension_card.js:1137
    ("txminguangkai", CardKind::Armor, 0, Suit::Heart, 1), // extension_card.js:1152
    ("txhj_taijifuchen", CardKind::Weapon, 5, Suit::Heart, 11), // extension_card.js:1201
    ("txhj_baihu", CardKind::HorsePlus, 0, Suit::Club, 8), // extension_card.js:1263
    ("txhj_wangmeizhike", CardKind::WangmeiZhike, 0, Suit::Club, 7), // extension_card.js:1287
    ("txhj_shengdong", CardKind::Wuzhong, 0, Suit::Spade, 8), // extension_card.js:1356
    ("txhj_shuiyan", CardKind::Shuigong, 0, Suit::Spade, 7), // extension_card.js:1437
    ("txhj_jiuchiroulin", CardKind::SavageAttack, 0, Suit::Club, 11), // extension_card.js:1897
    ("txzhaoshu_skill", CardKind::Wuzhong, 0, Suit::Diamond, 3), // extension_card.js:1977
    ("txzhaoshu_global", CardKind::Wuzhong, 0, Suit::Diamond, 4), // extension_card.js:2041
    ("huogong", CardKind::Wuzhong, 0, Suit::Heart, 2), // extension_skill.js:458
];

/// All 32 `L11` card entry IDs for catalog & G0 verification.
pub const L11_TAIXU_CARD_IDS: [&str; 32] = [
    "txhj_xingtianpojunfu",
    "txhj_zhuge",
    "txhj_qilingong",
    "txhj_fangtianhuaji",
    "txhj_zhijizhibi",
    "txhj_yuanjiao",
    "txhj_yiyi",
    "txhj_xuwangzhimian",
    "txhj_piliche",
    "txhj_liulongcanjia",
    "txhj_leijimu",
    "txhj_juechenjinge",
    "txhj_jiedao",
    "txhj_huoshaolianying",
    "txhj_guofengyupao",
    "txhj_feilong",
    "txhj_dinglanyemingzhu",
    "txhj_diaohu",
    "txhj_chiyanzhenhunqin",
    "txhj_caomu",
    "txhj_lingbaoxianhu",
    "txhj_chongyingshenfu",
    "txminguangkai",
    "txhj_taijifuchen",
    "txhj_baihu",
    "txhj_wangmeizhike",
    "txhj_shengdong",
    "txhj_shuiyan",
    "txhj_jiuchiroulin",
    "txzhaoshu_skill",
    "txzhaoshu_global",
    "huogong",
];
