//! Authentic upstream `L11` (`archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension.js,extension_buff.js,extension_servant.js,extension_skill.js}`)
//! 380 skills/buffs and cross-package NPC general skill bindings.

/// All 374 authentic `L11` skills & buffs (`extension.js` 124 + `extension_buff.js` 106 + `extension_servant.js` 4 + `extension_skill.js` 140).
pub const L11_TAIXU_SKILLS: [&str; 374] = [
    "txhj_daiyuan", "txhj_daiyuan2", "txhj_daiyuanA", "txhj_daiyuanB",
    "txhj_ruiSkill1", "txhj_dundunSkill2", "txhj_jiuweiSkill1", "txhj_jiuweiSkill1_disable",
    "txhj_jiuweiSkill2", "txhj_jiuweiSkill3", "txhj_tengsheSkill1", "txhj_tengsheSkill2",
    "txhj_tengsheSkill3", "txhj_yayaSkill1", "txhj_yayaSkill2", "txhj_yayaSkill2_protected",
    "txhj_youyouSkill1", "txhj_youyouSkill2", "txhj_qilinSkill1", "txhj_qilinSkill2",
    "txhj_qilinSkill3", "txhj_minminSkill1", "txhj_minminSkill2", "txhj_ditingSkill1",
    "txhj_ditingSkill2", "txhj_ditingSkill3", "txhj_qiaoqiaoSkill1", "txhj_qiaoqiaoSkill2",
    "tx_mojian", "tx_yushou", "tx_moyany", "tx_danshu",
    "txyirang", "txayboji", "txayhanji", "tx_dqtianzi",
    "xinmingjian", "xinmingjian2", "txmazhan", "txlieji",
    "diycuidu", "diychunxiao", "diyzhongdu", "diyzhuiling",
    "diyfeihua", "txclanxieshu", "txshixin", "txmouduan",
    "txkaikang", "txhuangfu", "txleizhen", "txxiebi",
    "txlianbi", "txfeiyan", "txleili", "cxyMoJun",
    "cxyKuangXi", "txbaobian", "cxy_BaoYing",
    "cxyYangWu", "cxyJingQi", "cxyRuiQi",
    "cxyJiaoXia", "cxyTunJun", "cxyFengYing", "cxyFanGong",
    "cxyJieLve", "cxyMoQu",
    "cxyYaoWu", "cxyYingHun", "cxyPoLu", "oldfuman",
    "oldfuman2", "txhuao", "hgkurou", "hgkurou_effect",
    "gdsanchen", "txlveming", "txdaoxi", "txlangxi",
    "txkuangyi", "txtanbei", "txtanbei_effect3", "txtanbei_effect1",
    "txtanbei_effect2", "ljxuechi", "txzhuxin", "txyicheng",
    "txnianrui", "txmengtai", "txshouyi", "txrenxing",
    "txyinli", "txzhulian", "txkeji", "txkeji_add",
    "txnangce", "txduanzui", "txyingyang", "mdqianxi7",
    "mdqianxi72", "mdqianxi", "zylongdan", "zglkongcheng",
    "txjuxiang", "txdaoji", "txhj_suoming", "txhj_huangmen",
    "txhj_mouqiang", "txhj_suzhi", "txmanjia",
    "txmanjia1", "txmanjia2", "txmanzhen",
    "txfusha", "txleizhou", "txzhennu", "txyongguan",
    "txyongguan_fate", "txchengyuan", "txbenji", "txyonglve",
    "txjiang", "txhj_oldhuxiao", "buff_txhj_aozhan", "buff_txhj_yanhuo",
    "buff_txhj_jinglei", "buff_txhj_xihuo", "buff_txhj_dinglei", "buff_txhj_wuzhongshengsha",
    "buff_txhj_wuzhongshengshan", "buff_txhj_fengyushengcai", "buff_txhj_jipodizhen", "buff_txhj_wenhezhenlue",
    "buff_txhj_wenhejibei", "buff_txhj_yueyingdezhihui", "buff_txhj_changbingguangren", "buff_txhj_fuzu",
    "buff_txhj_xiaojideyongwu", "buff_txhj_xiongziyingfa", "buff_txhj_wuxiangdexinxue", "buff_txhj_juguzhidao",
    "buff_txhj_pozhenzhifeng", "buff_txhj_yanrenzhinu", "buff_txhj_dunjiazhoufa", "buff_txhj_ziyideyongzhan",
    "buff_txhj_yanhuozhiren", "buff_txhj_jingleizhiren", "buff_txhj_yingjicuoshijia", "buff_txhj_yingjicuoshiyi",
    "buff_txhj_xihuozhikai", "buff_txhj_dingleizhikai", "buff_txhj_yueyingdejiqiao", "buff_txhj_taipingzhoufa",
    "buff_txhj_baijujianchu", "buff_txhj_juejingzhice", "buff_txhj_xumingshenyao", "buff_txhj_wuweizhile",
    "buff_txhj_yuanduandilue", "buff_txhj_jiezizhidao", "buff_txhj_huangtiannizhuan", "buff_txhj_dongchazhiyan",
    "buff_txhj_mingjingzhiyan", "buff_txhj_juxiangzhifeng", "buff_txhj_luanjifengshi", "buff_txhj_taoyuanyishi",
    "buff_txhj_haoshidebaochao", "buff_txhj_duocejiyong", "buff_txhj_wolongmice", "buff_txhj_fenchengluetu",
    "buff_txhj_bowenqiangzhi", "buff_txhj_majundegaizao", "buff_txhj_xianhujiqu", "buff_txhj_fuchendangmo",
    "buff_txhj_shenfuhuaxie", "buff_txhj_gongzhibaowu", "buff_txhj_fangzhibaowu", "buff_txhj_suikuiliejia",
    "buff_txhj_yiguzuoqi", "buff_txhj_shangjiangdelvli", "buff_txhj_shenhanxueyin", "buff_txhj_xiuluodebeimin",
    "buff_txhj_cebeibuyu", "buff_txhj_gongtaizhizhi", "buff_txhj_tianxiawushuang", "buff_txhj_jijianchangge",
    "buff_txhj_huangtianzhinu", "buff_txhj_luoshenjuezhang", "buff_txhj_shenyizhidao", "buff_txhj_mowangdejianiang",
    "buff_txhj_hannu", "buff_txhj_gusoejueyan", "buff_txhj_mingyundezhiyi", "buff_txhj_ximengzhili",
    "buff_txhj_hengxingwuji", "buff_txhj_shierqicejin", "buff_txhj_shierqicetui", "buff_txhj_pofuchenzhou",
    "buff_txhj_huangtianyili", "buff_txhj_huangtianzhizu", "buff_txhj_huangtianzhimin", "buff_txhj_tiangongdeyouhu",
    "buff_txhj_digongdeyouhu", "buff_txhj_renigongdeyouhu", "txhj_dundunSkill1", "txhj_datongSkill1",
    "txhj_luluSkill1", "txhj_luluSkill2", "txhj_aleSkill1", "txhj_aleSkill2",
    "txhj_ruiSkill2", "txhj_ruiSkill3", "txhj_yueerSkill1", "txhj_yueerSkill2",
    "txhj_yanSkill1", "txhj_yanSkill2", "txhj_yanSkill3", "txhj_liuliSkill1",
    "txhj_liuliSkill2", "txhj_manmanSkill1", "txhj_manmanSkill2", "txhj_xiaoxiaoSkill1",
    "txhj_xiaoxiaoSkill2", "txhj_xuerenSkill1", "txhj_xuanwuSkill1", "txhj_xuanwuSkill2",
    "txhj_xuanwuSkill3", "txhj_aHaoSkill1", "txhj_aHaoSkill2", "txhj_aHaoSkill3",
    "_slEnterGame", "_slDamage", "_slUseCard", "_slHurt",
    "_txhj_skill_maxCards", "_txhj_skill_Timelimit", "_txhj_skill_addEffectStyle", "_txhj_skill_gaincard",
    "_txhj_skill_discard", "_txhj_skill_usecard", "_txhj_skill_damage", "_txhj_skills_kill",
    "_txhj_equip5", "twsidao", "twsidao_equip", "polu",
    "polu_damage", "guofengyupao", "gx_taijifuchen", "hanbing_skill",
    "cixiong_skill", "wushuangfangtianji_skill", "qilin_skill", "guanshi_skill",
    "sanjian_skill", "yitianjian", "pyzhuren_diamond", "twzhouhu",
    "twharvestinori", "twzuhuo", "_txhj_shenqi", "_txhj_XXXshenqi",
    "_buff_txhj_yanhuo", "_buff_txhj_jinglei", "_buff_txhj_xihuo", "_buff_txhj_dinglei",
    "_buff_txhj_wuzhongshengsha", "_buff_txhj_wuzhongshengshan", "_buff_txhj_fengyushengcai", "_buff_txhj_jipodizhen",
    "_buff_txhj_wenhezhenlue", "_buff_txhj_wenhejibei", "_buff_txhj_yueyingdezhihui", "_buff_txhj_changbingguangren",
    "_buff_txhj_fuzu", "_buff_txhj_xiaojideyongwu", "_buff_txhj_xiongziyingfa", "_buff_txhj_wuxiangdexinxue",
    "_buff_txhj_pozhenzhifeng", "_buff_txhj_yanrenzhinu", "_buff_txhj_dunjiazhoufa", "_buff_txhj_ziyideyongzhan",
    "_buff_txhj_yanhuozhiren", "_buff_txhj_jingleizhiren", "_buff_txhj_yingjichuoshijia", "_buff_txhj_yingjichuoshiyi",
    "_buff_txhj_xihuozhikai", "_buff_txhj_dingleizhikai", "_buff_txhj_yueyingdejiqiao", "_buff_txhj_taipingzhoufa",
    "_buff_txhj_baijujianchu", "_buff_txhj_juejingzhice", "_buff_txhj_xumingshenyao", "_buff_txhj_wuweizhile",
    "_buff_txhj_yuanduandilue", "_buff_txhj_jiezizhidao", "_buff_txhj_huangtiannizhuan", "_buff_txhj_dongchazhiyan",
    "_buff_txhj_mingjingzhiyan", "_buff_txhj_juxiangzhifeng", "_buff_txhj_luanjifengshi", "_buff_txhj_taoyuanyishi",
    "_buff_txhj_haoshidebaochang", "_buff_txhj_duocejiyong", "_buff_txhj_jufuxionghao", "_buff_txhj_shenfuhuaxie",
    "_buff_txhj_gongzhibaowu", "_buff_txhj_fangzhibaowu", "_buff_txhj_suikuiliejia", "_buff_txhj_yiguzuoqi",
    "_buff_txhj_xiuluodelianmin", "_buff_txhj_cebeibuyu", "_buff_txhj_gongtaizhizhi", "_buff_txhj_tianxiawushuang",
    "_buff_txhj_jijianchangge", "_buff_txhj_huangtianzhinu", "_buff_txhj_luoshenjuezhang", "_buff_txhj_shenyizhidao",
    "_buff_txhj_mowangdejianiang", "_buff_txhj_hannu", "_buff_txhj_guosejueyan", "_buff_txhj_mingyundezhiyi",
    "_buff_txhj_ximengzhili", "_buff_txhj_hengxingwuji", "_buff_txhj_shierqicejin", "_buff_txhj_shierqicetui",
    "_buff_txhj_pofuchenzhou", "_buff_txhj_huangtianyili", "_buff_txhj_huangtianzhizu", "_buff_txhj_huangtianzhimin",
    "txhj_kapaikeshi", "txhj_wenhejibei", "txhj_yueyingdezhihui", "txhj_changbingguangren",
    "txhj_xiongziyingfa", "txhj_wuxiangdexinxue", "txhj_pozhenzhifeng", "txhj_wuweizhile",
    "txhj_yuanduandilue", "txhj_huangtiannizhuan", "txhj_taoyuanyishi", "taoyuanyishiX",
    "txhj_jufuxionghao", "txhj_luoshenjuezhang", "txhj_luoshen", "txhj_mowangdejianiang",
    "txhj_jianiang_clear", "txhj_shierqicejin", "txhj_shierqicetui", "txhj_pofuchenzhou",
    "txhj_mouduan", "olshanjia", "boss_xuanlei", "txhj_xuanlve",
    "boss_chuanyun", "boss_baolian", "boss_lingfeng", "dz_gs_yinfu",
    "boss_leili", "boss_fengxing", "boss_yushou", "boss_xixing",
    "boss_moyany", "boss_danshu", "boss_tianyun", "boss_modao",
    "boss_jueji", "boss_mojian", "boss_lianyu", "boss_skonghun",
    "txhj_zhuwei", "boss_zhennu", "txhj_chuhai", "txhj_chuhai2",
];

/// The 29 `L11` NPC boss skills defined in `太虚幻境/extension.js:554-3632`.
pub const L11_TAIXU_OWN_NPC_SKILLS: [&str; 29] = [
    "tx_modao", "txhj_baoji", "txhj_baolian", "txhj_beiming", "txhj_chuanyun",
    "txhj_didun", "txhj_dizhou", "txhj_guihuo", "txhj_guimei", "txhj_guimen",
    "txhj_guiyan", "txhj_huangjin", "txhj_juhun", "txhj_kuangbao", "txhj_luolei",
    "txhj_renfang", "txhj_renwang", "txhj_shiyuan", "txhj_tunshi", "txhj_wangyuan",
    "txhj_wangzu", "txhj_xiefeng", "txhj_yaoshu", "txhj_zhoufa", "txhj_zhoulei",
    "txhj_zhouzu", "txhj_zhuhun", "txhj_zuijiu", "txqyyouji",
];

/// The 20 equip/summon card-backed skills in `太虚幻境/extension_card.js` (trick cards execute via `PlayCard`).
pub const L11_TAIXU_EQUIP_CARD_SKILLS: [&str; 20] = [
    "txhj_xingtianpojunfu", "txhj_zhuge", "txhj_qilingong", "txhj_fangtianhuaji",
    "txhj_xuwangzhimian", "txhj_piliche", "txhj_liulongcanjia", "txhj_leijimu",
    "txhj_juechenjinge", "txhj_guofengyupao", "txhj_feilong", "txhj_dinglanyemingzhu",
    "txhj_chiyanzhenhunqin", "txhj_lingbaoxianhu", "txhj_chongyingshenfu", "txminguangkai",
    "txhj_taijifuchen", "txhj_baihu", "txzhaoshu_skill", "txzhaoshu_global",
];

/// Cross-package skills referenced on `L11_TAIXU_GENERALS` (`extension_character.js:6-1489`).
pub const L11_TAIXU_CROSS_SKILLS: [&str; 448] = [
    "anguo", "anjian", "aocai", "baiyi", "baonu",
    "bazhen", "benghuai", "benyu", "bihuo", "bingyi",
    "bmcanshi", "buqu", "chajue", "chenqing", "chongzhen",
    "chouce", "chouhai", "chunlao", "cihuai", "cuijin",
    "dahe", "danji", "dclbjiuxian", "dcmoukui", "dcnutao",
    "dcshushen", "dcxiaoxi", "dddfenji", "dddjiexing", "dddqiaoduan",
    "diancai", "drlt_jieying", "drlt_poxi", "drlt_qianjie", "drlt_xiongluan",
    "dualside", "duanliang", "dujin", "duwu", "fangquan",
    "fankui", "feiying", "fenji", "fenming", "fenwei",
    "fenxun", "fenyin", "fuji", "fuli", "fulin",
    "fuqi", "ganglie", "gaoyuan", "gnjinfan", "gnsheque",
    "gongao", "gongxin", "guanzong", "guju", "gushe",
    "guzheng", "gzyinghun", "hanyong", "hongde", "huaizi",
    "huilei", "huoji", "huxiao", "jianchu", "jiang",
    "jianying", "jiaojie", "jiaozi", "jici", "jieyuan",
    "jiezhong", "jiezi", "jigong", "jijiu", "jili",
    "jinglve", "jinjiu", "jintao", "jiuchi", "jsrgbahu",
    "jsrgshichong", "jsrgzhenqiao", "juece", "juejing", "jueqing",
    "jujian", "juxiang", "juyi", "kaikang", "keji",
    "kongcheng", "kuangfu", "kuanshi", "kunfen", "kurou",
    "kyou_zhidian", "langmie", "leiji", "liangzhu", "lianpo",
    "lianzhu", "liegong", "lieren", "lihun", "lihuo",
    "lirang", "liuli", "lixia", "liyu", "lizhan",
    "longdan", "longhun", "longyin", "luanwu", "luoshen",
    "manji", "mashu", "mengjin", "mieji", "miji",
    "mingce", "mingjian", "mizhao", "mjmouzhi", "mobilejingce",
    "mobilezhongyong", "moukui", "mouzhu", "mozhi", "new_repaoxiao",
    "new_reqingnang", "new_rewusheng", "new_reyaowu", "new_reyiji", "new_yijue",
    "nsfeixiong", "nslulve", "nsxianhai", "nsyangwu", "nsyice",
    "nzry_chenglve", "nzry_cunmu", "nzry_feijun", "nzry_jieying", "nzry_kuizhu",
    "nzry_longnu", "nzry_shicai", "olniepan", "olzhuiji", "piaoling",
    "qianglue", "qiangwu", "qiangxi", "qiangzhi", "qiaomeng",
    "qiaoshui", "qice", "qingxi", "qingxian", "qingzhongx",
    "qinyin", "qixi", "quhu", "quji", "ranshang",
    "rebeige", "rebiyue", "rechouhai", "redangxian", "reduodao",
    "refanjian", "refankui", "refenxun", "refuyuan", "reganlu",
    "reguanxing", "reguicai", "reguose", "rehuashen", "rehunzi",
    "rejiangchi", "rejianxiong", "rejieming", "rejinjiu", "rejizhi",
    "rejuece", "rejunxing", "rekanpo", "rekuangcai", "releiji",
    "relianji", "relianying", "relieren", "relongyin", "reluanji",
    "reluanwu", "reluoyi", "renjie", "renshe", "repojun",
    "repolu", "reqiangxi", "reqianju", "reqicai", "reqizhou",
    "requanji", "reshanxi", "reshejian", "reshuangxiong", "retianxiang",
    "retiaoxin", "retieji", "retuntian", "retuxi", "rewansha",
    "reweimu", "rewurong", "rexianzhen", "rexingluan", "rexingshang",
    "reyajiao", "reyingzi", "rezaiqi", "rezhenxing", "rezhiheng",
    "rezhijian", "rezhiyi", "rezongshi", "rongbei", "roulin",
    "satomi_luodao", "sbaiyin", "sbliegong", "sbyangwei", "sbyingzi",
    "shangshi", "shangyi", "shanxie", "shefu", "shelie",
    "shenfen", "shensu", "shenxian", "shenxing", "shibei",
    "shixin", "shizhan", "shizhi", "shunshi", "spchijie",
    "spdengli", "sphuangen", "spmingshi", "spqishe", "spshiji",
    "sptunjiang", "spwenji", "spxiangzhen", "spyishi", "taoluan",
    "tiandu", "tianming", "tianshu", "tianyi", "tiaoxin",
    "tieji", "tongduo", "tuifeng", "twfengpo", "twfenming",
    "twhanyu", "twhengjiang", "twhongju", "twhuangjin", "twjiange",
    "twjianming", "twjiefan", "twjuntun", "twkujian", "twliancai",
    "twlingfa", "twlinglu", "twmibei", "twmoukui", "twqianju",
    "twqianxi", "twqirang", "twqueshi", "twshengxi", "twwuhun",
    "twwushen", "twxiafeng", "twxiangyu", "twxingqi", "twxiongjun",
    "twxiongxi", "twxiongzheng", "twxuanhuo", "twxuechang", "twxuhe",
    "twyangshi", "twyilie", "twyingjia", "twyuanhu", "twyuhua",
    "twzhengrong", "tx_modao", "txhj_baoji", "txhj_baolian", "txhj_beiming",
    "txhj_chuanyun", "txhj_didun", "txhj_dizhou", "txhj_guihuo", "txhj_guimei",
    "txhj_guimen", "txhj_guiyan", "txhj_huangjin", "txhj_jiaoxie", "txhj_jinwei",
    "txhj_jinxing", "txhj_juhun", "txhj_kuangbao", "txhj_luolei", "txhj_quanqin",
    "txhj_renfang", "txhj_renwang", "txhj_shiyuan", "txhj_tunshi", "txhj_wangyuan",
    "txhj_wangzu", "txhj_xiefeng", "txhj_xiehui", "txhj_yaoshu", "txhj_yishi",
    "txhj_zhanhuo", "txhj_zhoufa", "txhj_zhoulei", "txhj_zhouzu", "txhj_zhuhun",
    "txhj_zhuosheng", "txhj_zuijiu", "txqyyouji", "waishi", "wangxi",
    "wangzun", "wansha", "weifeng", "weijing", "weimu",
    "wulie", "wurong", "wushen", "wushuang", "xiadao",
    "xiandeng", "xianfu", "xiangle", "xianzhen", "xianzhou",
    "xiaoguo", "xiaoji", "xiaoxi_hansui", "xiehui", "xinanguo",
    "xinanjian", "xinchenhui", "xinduodao", "xinfenyue", "xinfu_daigong",
    "xinfu_fuyin", "xinfu_kannan", "xinfu_langxi", "xinfu_limu", "xinfu_lingren",
    "xinfu_qinguo", "xinfu_shajue", "xinfu_sidao", "xinfu_songsang", "xinfu_xingluan",
    "xinfu_xionghuo", "xinfu_zengdao", "xinfu_zhaoxin", "xinhongyan", "xinjiewei",
    "xinjuece", "xinjuejing", "xinjushou", "xinkuangfu", "xinkuanggu",
    "xinliegong", "xinniluan", "xinqieting", "xinsheng", "xiongyi",
    "xswuyan", "xueji", "yaowu", "yechou", "yeyan",
    "yicong", "yinbing", "yinghun", "yingjian", "yingzi",
    "yise", "yiyong", "yizhong", "yjjianshu", "yjzhenlve",
    "yongjin", "yongsi", "yuce", "yxs_fanji", "zaiqi",
    "zaoxian", "zhanyi", "zhaohuo", "zhaxiang", "zhendu",
    "zhengfu", "zhengjian", "zhenlie", "zhenlue", "zhichi",
    "zhidao", "zhiheng", "zhiji", "zhiman", "zhiyan",
    "zhiyi", "zhongyong", "zhuandui", "zhuhai", "zhuide",
    "zhuiji", "zhuikong", "zhuixi", "zhuiyi", "zili",
    "zishou", "zongkui", "zymoucheng",
];

/// The 9 dangling skill IDs referenced by 11 `HeJinZhuHuan` (`何进诛宦`) NPCs in
/// `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_character.js:321-337`
/// (`/*何进诛宦*/ ... /*新技能*/`), with commented-out draft stubs in
/// `extension_csrank.js:3540-3587` and `extension_framework.js:10545-10562`, that have
/// **zero `lib.skill` definitions anywhere in `archive/upstream/noname` or `archive/upstream/noname-liuli`**.
///
/// Per Decision 34 (`禁止凭空生成上游不存在的技能；禁止用默认摸 1 兜底冒充实现`), these 9 IDs are
/// recorded as inert upstream-undefined metadata tags without fabricating fake skill behaviors.
pub const L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS: [&str; 9] = [
    "txhj_jiaoxie",
    "txhj_jinwei",
    "txhj_jinxing",
    "txhj_quanqin",
    "txhj_xiehui",
    "txhj_yishi",
    "txhj_zhanhuo",
    "txhj_zhuosheng",
    "xinchenhui",
];

/// All 164 upstream-defined cross-package NPC skills referenced by `L11_TAIXU_GENERALS`
/// (`archive/upstream/noname-liuli/noname/character/*.js`, `archive/upstream/noname/apps/core/character/**/*.js`,
/// `dualside` in `game.js:34262`, and `qingzhongx` in `sp/skill.js:35220` / `markskill.js:18544`)
/// that are faithfully implemented in `decision34_l11_npc_a.rs`, `decision34_l11_npc_b.rs`,
/// `d34_l11_mods.rs`, and `d34_view_as.rs`.
pub const L11_TAIXU_IMPLEMENTED_CROSS_SKILLS: [&str; 164] = [
    "aocai", "baiyi", "benyu", "bmcanshi", "buqu", "chenqing", "chouce", "dclbjiuxian",
    "dcmoukui", "dcnutao", "diancai", "dualside", "duwu", "fenji", "fenxun", "gaoyuan",
    "gnsheque", "guanzong", "guju", "gushe", "jici", "jieyuan", "jigong", "jili",
    "jinglve", "jsrgshichong", "jsrgzhenqiao", "jujian", "kyou_zhidian", "leiji", "liangzhu", "lianzhu",
    "lihun", "lirang", "lixia", "liyu", "mingjian", "mizhao", "mjmouzhi", "mouzhu",
    "mozhi", "new_reqingnang", "new_yijue", "nsxianhai", "nzry_chenglve", "nzry_feijun", "nzry_kuizhu", "nzry_shicai",
    "olniepan", "olzhuiji", "piaoling", "qingxian", "qingzhongx", "quji", "reduodao", "refenxun",
    "reganlu", "reguanxing", "reguose", "rejinjiu", "rejuece", "rejunxing", "relianying", "reluanji",
    "reluoyi", "renshe", "repojun", "reqizhou", "requanji", "reshuangxiong", "retianxiang", "retuntian",
    "retuxi", "rewurong", "rexingluan", "rexingshang", "reyajiao", "rezaiqi", "rezhijian", "rongbei",
    "satomi_luodao", "sbliegong", "sbyingzi", "shangyi", "shefu", "shensu", "shizhi", "shunshi",
    "spshiji", "sptunjiang", "spwenji", "spxiangzhen", "spyishi", "tianming", "tianshu", "tuifeng",
    "twfenming", "twhanyu", "twhengjiang", "twhongju", "twhuangjin", "twjianming", "twjiefan", "twjuntun",
    "twliancai", "twlingfa", "twlinglu", "twmibei", "twqianxi", "twqirang", "twqueshi", "twwuhun",
    "twwushen", "twxiangyu", "twxingqi", "twxiongxi", "twxiongzheng", "twxuanhuo", "twxuechang", "twxuhe",
    "twyilie", "twyingjia", "twyuanhu", "twyuhua", "twzhengrong", "waishi", "wangzun", "weifeng",
    "weijing", "wulie", "xianfu", "xiehui", "xinfenyue", "xinfu_daigong", "xinfu_fuyin", "xinfu_kannan",
    "xinfu_limu", "xinfu_shajue", "xinfu_sidao", "xinfu_xingluan", "xinfu_zengdao", "xinfu_zhaoxin", "xinjiewei", "xinjushou",
    "xinkuangfu", "xinsheng", "yechou", "yinbing", "yise", "yjjianshu", "yongjin", "yongsi",
    "yuce", "zhanyi", "zhendu", "zhichi", "zhidao", "zhiman", "zhiyan", "zhongyong",
    "zhuide", "zhuixi", "zili", "zongkui",
];

/// Map each of the 164 `L11` cross-package NPC skills to its authentic upstream trigger window.
pub fn classify_l11_npc_skill_trigger(
    name: &str,
) -> Option<(
    engine_native::event::EventKind,
    engine_native::rvm_bridge::VmFilterPolicy,
)> {
    use engine_native::event::EventKind;
    use engine_native::rvm_bridge::VmFilterPolicy;
    match name {
        // === 1. Dying Window ===
        "buqu" | "chenqing" | "dualside" | "olniepan" | "xinfu_shajue" | "twjiefan" => {
            Some((EventKind::Dying, VmFilterPolicy::OwnerIsTarget))
        }
        // === 2. Death Window ===
        "rexingshang" | "twwuhun" | "yechou" | "zhuide" | "twxuechang" => {
            Some((EventKind::Death, VmFilterPolicy::OwnerIsTarget))
        }
        // === 3. Damage Taken (OwnerIsTarget) ===
        "benyu" | "chouce" | "guju" | "jieyuan" | "mjmouzhi" | "qingxian" | "renshe"
        | "requanji" | "retianxiang" | "shunshi" | "tuifeng" | "xianfu" | "xinfu_daigong"
        | "xinsheng" | "yuce" | "zhichi" | "twyilie" => {
            Some((EventKind::Damage, VmFilterPolicy::OwnerIsTarget))
        }
        // === 4. Damage Dealt (OwnerIsSource) ===
        "liyu" | "nsxianhai" | "spshiji" | "spyishi" | "twzhengrong" | "zhidao" | "zhiman" => {
            Some((EventKind::Damage, VmFilterPolicy::OwnerIsSource))
        }
        // === 5. Cards Lost ===
        "relianying" | "retuntian" | "lirang" => {
            Some((EventKind::CardsLost, VmFilterPolicy::OwnerIsTarget))
        }
        // === 6. Draw Phase Begin 1 ===
        "gnsheque" | "sbyingzi" | "yongsi" => {
            Some((EventKind::DrawPhaseBegin1, VmFilterPolicy::OwnerIsTarget))
        }
        // === 7. Draw Phase Begin 2 ===
        "retuxi" | "reluoyi" | "reshuangxiong" => {
            Some((EventKind::DrawPhaseBegin2, VmFilterPolicy::OwnerIsTarget))
        }
        // === 8. Card Used / Responded (OwnerIsSource) ===
        "dcmoukui" | "dcnutao" | "jsrgshichong" | "jsrgzhenqiao" | "kyou_zhidian" | "leiji"
        | "nzry_shicai" | "olzhuiji" | "repojun" | "reqizhou" | "rexingluan" | "reyajiao"
        | "satomi_luodao" | "sbliegong" | "twhengjiang" | "twjianming" | "twmibei"
        | "twxiangyu" | "twxuhe" | "weifeng" | "xinfu_sidao" | "xinfu_xingluan" | "zhongyong"
        | "aocai" | "reluanji" | "rejinjiu" | "rewurong" | "twwushen" => {
            Some((EventKind::CardUsed, VmFilterPolicy::OwnerIsSource))
        }
        // === 9. Card Used / Responded (OwnerIsTarget) ===
        "bmcanshi" | "gaoyuan" | "jili" | "reduodao" | "spxiangzhen" | "tianming"
        | "twhuangjin" | "xinfu_fuyin" => {
            Some((EventKind::CardUsed, VmFilterPolicy::OwnerIsTarget))
        }
        // === 10. Phase End ===
        "diancai" | "lixia" | "mozhi" | "nzry_kuizhu" | "piaoling" | "rejuece" | "rezaiqi"
        | "shefu" | "sptunjiang" | "twliancai" | "twyingjia" | "wulie" | "xinjushou"
        | "yinbing" | "zhiyan" | "twyuanhu" | "rezhijian" | "twfenming" | "rongbei"
        | "xinfu_zengdao" | "jujian" | "twyuhua" => {
            Some((EventKind::PhaseEnd, VmFilterPolicy::OwnerIsTarget))
        }
        // === 11. Turn Begin ===
        "twhanyu" | "twjuntun" | "twlingfa" | "twqueshi" | "twxiongzheng" | "zongkui"
        | "reguanxing" | "twqianxi" | "zili" | "wangzun" | "twhongju" | "twxuanhuo"
        | "xinjiewei" | "baiyi" | "guanzong" | "liangzhu" | "lianzhu" | "mingjian"
        | "nzry_chenglve" | "nzry_feijun" | "qingzhongx" | "tianshu" | "twqirang"
        | "weijing" | "xiehui" | "twlinglu" | "dclbjiuxian" => {
            Some((EventKind::TurnBegin, VmFilterPolicy::OwnerIsTarget))
        }
        // === 12. Active Play Phase (enable: "phaseUse") ===
        "duwu" | "fenji" | "fenxun" | "gushe" | "jici" | "jigong" | "jinglve"
        | "lihun" | "mizhao" | "mouzhu" | "new_reqingnang" | "new_yijue" | "quji"
        | "refenxun" | "reganlu" | "reguose" | "rejunxing" | "shangyi" | "shensu"
        | "shizhi" | "spwenji" | "twxingqi" | "twxiongxi" | "waishi" | "xinfenyue"
        | "xinfu_kannan" | "xinfu_limu" | "xinfu_zhaoxin" | "xinkuangfu" | "yise"
        | "yjjianshu" | "yongjin" | "zhanyi" | "zhendu" | "zhuixi" => {
            Some((EventKind::PhaseBegin, VmFilterPolicy::OwnerIsTarget))
        }
        // Explicit rejection: no arbitrary fallback
        _ => None,
    }
}


/// Map each of the 423 `L11` authentic Taixu skills/buffs/shop items to its authentic trigger.
pub fn classify_l11_taixu_skill_trigger(
    name: &str,
) -> Option<(
    engine_native::event::EventKind,
    engine_native::rvm_bridge::VmFilterPolicy,
)> {
    use engine_native::event::EventKind;
    use engine_native::rvm_bridge::VmFilterPolicy;
    match name {
        // === L11 Dying (2) ===
        "cxy_BaoYing" | "txzhuxin" => Some((EventKind::Dying, VmFilterPolicy::OwnerIsTarget)),
        // === L11 Death (3) ===
        "_txhj_skills_kill" | "cxyPoLu" | "txleizhen" => Some((EventKind::Death, VmFilterPolicy::OwnerIsTarget)),
        // === L11 DamageTarget (19) ===
        "_buff_txhj_cebeibuyu" | "_buff_txhj_dinglei" | "_buff_txhj_dingleizhikai" | "_buff_txhj_shenfuhuaxie" | "_buff_txhj_xihuo" | "_buff_txhj_xihuozhikai"
        | "_txhj_skill_damage" | "cxyMoJun" | "cxyYaoWu" | "diyzhuiling" | "polu_damage" | "tx_dqtianzi"
        | "txhj_mouqiang" | "txhuangfu" | "txmanjia2" | "txrenxing" | "txshouyi" | "txxiebi"
        | "txyongguan" => Some((EventKind::Damage, VmFilterPolicy::OwnerIsTarget)),
        // === L11 DamageSource (25) ===
        "_buff_txhj_baijujianchu" | "_buff_txhj_huangtianzhinu" | "_buff_txhj_jinglei" | "_buff_txhj_jingleizhiren" | "_buff_txhj_jipodizhen" | "_buff_txhj_suikuiliejia"
        | "_buff_txhj_tianxiawushuang" | "_buff_txhj_yanhuo" | "_buff_txhj_yanhuozhiren" | "_slDamage" | "boss_leili" | "cxyJieLve"
        | "diycuidu" | "hanbing_skill" | "pyzhuren_diamond" | "qilin_skill" | "sanjian_skill" | "txayhanji"
        | "txclanxieshu" | "txfusha" | "txhj_chuhai2" | "txhj_mowangdejianiang" | "txleili" | "wushuangfangtianji_skill"
        | "yitianjian" => Some((EventKind::Damage, VmFilterPolicy::OwnerIsSource)),
        // === L11 CardUsedSource (40) ===
        "_buff_txhj_dunjiazhoufa" | "_buff_txhj_duocejiyong" | "_buff_txhj_hannu" | "_buff_txhj_haoshidebaochang" | "_buff_txhj_hengxingwuji" | "_buff_txhj_jijianchangge"
        | "_buff_txhj_juejingzhice" | "_buff_txhj_jufuxionghao" | "_buff_txhj_juxiangzhifeng" | "_buff_txhj_luanjifengshi" | "_buff_txhj_mingyundezhiyi" | "_buff_txhj_mowangdejianiang"
        | "_buff_txhj_shenyizhidao" | "_buff_txhj_taipingzhoufa" | "_buff_txhj_taoyuanyishi" | "_buff_txhj_wenhezhenlue" | "_buff_txhj_yanrenzhinu" | "_buff_txhj_yiguzuoqi"
        | "_buff_txhj_ziyideyongzhan" | "_slUseCard" | "_txhj_skill_usecard" | "cixiong_skill" | "guanshi_skill" | "guofengyupao"
        | "gx_taijifuchen" | "txayboji" | "txfeiyan" | "txhj_changbingguangren" | "txhj_jufuxionghao" | "txhj_luoshenjuezhang"
        | "txhj_pozhenzhifeng" | "txhj_wuxiangdexinxue" | "txhj_xiongziyingfa" | "txjiang" | "txkeji_add" | "txkuangyi"
        | "txlieji" | "txmazhan" | "txnangce" | "zylongdan" => Some((EventKind::CardUsed, VmFilterPolicy::OwnerIsSource)),
        // === L11 CardUsedTarget (13) ===
        "_buff_txhj_dongchazhiyan" | "_buff_txhj_mingjingzhiyan" | "cxyFanGong" | "txhj_shierqicejin" | "txhj_shierqicetui" | "txkaikang"
        | "txmanjia" | "txmanjia1" | "txmanzhen" | "txqyyouji" | "txshixin" | "txyicheng"
        | "txyonglve" => Some((EventKind::CardUsed, VmFilterPolicy::OwnerIsTarget)),
        // === L11 CardsLost (12) ===
        "_buff_txhj_pozhenzhifeng" | "_buff_txhj_xiaojideyongwu" | "_buff_txhj_yingjichuoshijia" | "_buff_txhj_yingjichuoshiyi" | "_slHurt" | "_txhj_XXXshenqi"
        | "boss_danshu" | "boss_moyany" | "olshanjia" | "tx_danshu" | "tx_moyany" | "txhj_xuanlve" => Some((EventKind::CardsLost, VmFilterPolicy::OwnerIsTarget)),
        // === L11 DrawPhase (5) ===
        "_buff_txhj_xiongziyingfa" | "boss_jueji" | "boss_lingfeng" | "cxyRuiQi" | "txnianrui" => Some((EventKind::DrawPhaseBegin1, VmFilterPolicy::OwnerIsTarget)),
        // === L11 PhaseEnd (19) ===
        "_buff_txhj_fangzhibaowu" | "_buff_txhj_wuzhongshengshan" | "boss_baolian" | "boss_chuanyun" | "boss_lianyu" | "boss_tianyun"
        | "cxyMoQu" | "diychunxiao" | "txduanzui" | "txhj_huangmen" | "txhj_jianiang_clear" | "txhj_mouduan"
        | "txhj_pofuchenzhou" | "txhj_suoming" | "txhj_suzhi" | "txjuxiang" | "txmouduan" | "txyinli"
        | "txyongguan_fate" => Some((EventKind::PhaseEnd, VmFilterPolicy::OwnerIsTarget)),
        // === L11 TurnBegin (6) ===
        "_buff_txhj_fengyushengcai" | "_buff_txhj_gongtaizhizhi" | "_txhj_skill_Timelimit" | "_txhj_skill_maxCards" | "cxyTunJun" | "txhj_luoshen" => Some((EventKind::TurnBegin, VmFilterPolicy::OwnerIsTarget)),
        // === L11 PhaseBegin (279) ===
        "_buff_txhj_changbingguangren" | "_buff_txhj_fuzu" | "_buff_txhj_gongzhibaowu" | "_buff_txhj_guosejueyan" | "_buff_txhj_huangtiannizhuan" | "_buff_txhj_huangtianyili"
        | "_buff_txhj_huangtianzhimin" | "_buff_txhj_huangtianzhizu" | "_buff_txhj_jiezizhidao" | "_buff_txhj_luoshenjuezhang" | "_buff_txhj_pofuchenzhou" | "_buff_txhj_shierqicejin"
        | "_buff_txhj_shierqicetui" | "_buff_txhj_wenhejibei" | "_buff_txhj_wuweizhile" | "_buff_txhj_wuxiangdexinxue" | "_buff_txhj_wuzhongshengsha" | "_buff_txhj_ximengzhili"
        | "_buff_txhj_xiuluodelianmin" | "_buff_txhj_xumingshenyao" | "_buff_txhj_yuanduandilue" | "_buff_txhj_yueyingdejiqiao" | "_buff_txhj_yueyingdezhihui" | "_slEnterGame"
        | "_txhj_equip5" | "_txhj_shenqi" | "_txhj_skill_addEffectStyle" | "_txhj_skill_discard" | "_txhj_skill_gaincard" | "boss_fengxing"
        | "boss_modao" | "boss_mojian" | "boss_skonghun" | "boss_xixing" | "boss_xuanlei" | "boss_yushou"
        | "boss_zhennu" | "buff_txhj_aozhan" | "buff_txhj_baijujianchu" | "buff_txhj_bowenqiangzhi" | "buff_txhj_cebeibuyu" | "buff_txhj_changbingguangren"
        | "buff_txhj_digongdeyouhu" | "buff_txhj_dinglei" | "buff_txhj_dingleizhikai" | "buff_txhj_dongchazhiyan" | "buff_txhj_dunjiazhoufa" | "buff_txhj_duocejiyong"
        | "buff_txhj_fangzhibaowu" | "buff_txhj_fenchengluetu" | "buff_txhj_fengyushengcai" | "buff_txhj_fuchendangmo" | "buff_txhj_fuzu" | "buff_txhj_gongtaizhizhi"
        | "buff_txhj_gongzhibaowu" | "buff_txhj_gusoejueyan" | "buff_txhj_hannu" | "buff_txhj_haoshidebaochao" | "buff_txhj_hengxingwuji" | "buff_txhj_huangtiannizhuan"
        | "buff_txhj_huangtianyili" | "buff_txhj_huangtianzhimin" | "buff_txhj_huangtianzhinu" | "buff_txhj_huangtianzhizu" | "buff_txhj_jiezizhidao" | "buff_txhj_jijianchangge"
        | "buff_txhj_jinglei" | "buff_txhj_jingleizhiren" | "buff_txhj_jipodizhen" | "buff_txhj_juejingzhice" | "buff_txhj_juguzhidao" | "buff_txhj_juxiangzhifeng"
        | "buff_txhj_luanjifengshi" | "buff_txhj_luoshenjuezhang" | "buff_txhj_majundegaizao" | "buff_txhj_mingjingzhiyan" | "buff_txhj_mingyundezhiyi" | "buff_txhj_mowangdejianiang"
        | "buff_txhj_pofuchenzhou" | "buff_txhj_pozhenzhifeng" | "buff_txhj_renigongdeyouhu" | "buff_txhj_shangjiangdelvli" | "buff_txhj_shenfuhuaxie" | "buff_txhj_shenhanxueyin"
        | "buff_txhj_shenyizhidao" | "buff_txhj_shierqicejin" | "buff_txhj_shierqicetui" | "buff_txhj_suikuiliejia" | "buff_txhj_taipingzhoufa" | "buff_txhj_taoyuanyishi"
        | "buff_txhj_tiangongdeyouhu" | "buff_txhj_tianxiawushuang" | "buff_txhj_wenhejibei" | "buff_txhj_wenhezhenlue" | "buff_txhj_wolongmice" | "buff_txhj_wuweizhile"
        | "buff_txhj_wuxiangdexinxue" | "buff_txhj_wuzhongshengsha" | "buff_txhj_wuzhongshengshan" | "buff_txhj_xianhujiqu" | "buff_txhj_xiaojideyongwu" | "buff_txhj_xihuo"
        | "buff_txhj_xihuozhikai" | "buff_txhj_ximengzhili" | "buff_txhj_xiongziyingfa" | "buff_txhj_xiuluodebeimin" | "buff_txhj_xumingshenyao" | "buff_txhj_yanhuo"
        | "buff_txhj_yanhuozhiren" | "buff_txhj_yanrenzhinu" | "buff_txhj_yiguzuoqi" | "buff_txhj_yingjicuoshijia" | "buff_txhj_yingjicuoshiyi" | "buff_txhj_yuanduandilue"
        | "buff_txhj_yueyingdejiqiao" | "buff_txhj_yueyingdezhihui" | "buff_txhj_ziyideyongzhan" | "cxyFengYing" | "cxyJiaoXia" | "cxyJingQi"
        | "cxyKuangXi" | "cxyYangWu" | "cxyYingHun" | "diyfeihua" | "diyzhongdu" | "dz_gs_yinfu"
        | "gdsanchen" | "hgkurou" | "hgkurou_effect" | "ljxuechi" | "mdqianxi" | "mdqianxi7"
        | "mdqianxi72" | "oldfuman" | "oldfuman2" | "polu" | "taoyuanyishiX" | "twharvestinori"
        | "twsidao" | "twsidao_equip" | "twzhouhu" | "twzuhuo" | "tx_modao" | "tx_mojian"
        | "tx_yushou" | "txbaobian" | "txbenji" | "txchengyuan" | "txdaoji" | "txdaoxi"
        | "txhj_aHaoSkill1" | "txhj_aHaoSkill2" | "txhj_aHaoSkill3" | "txhj_aleSkill1" | "txhj_aleSkill2" | "txhj_baihu"
        | "txhj_baoji" | "txhj_baolian" | "txhj_beiming" | "txhj_chiyanzhenhunqin" | "txhj_chongyingshenfu" | "txhj_chuanyun"
        | "txhj_chuhai" | "txhj_daiyuan" | "txhj_daiyuan2" | "txhj_daiyuanA" | "txhj_daiyuanB" | "txhj_datongSkill1"
        | "txhj_didun" | "txhj_dinglanyemingzhu" | "txhj_ditingSkill1" | "txhj_ditingSkill2" | "txhj_ditingSkill3" | "txhj_dizhou"
        | "txhj_dundunSkill1" | "txhj_dundunSkill2" | "txhj_fangtianhuaji" | "txhj_feilong" | "txhj_guihuo" | "txhj_guimei"
        | "txhj_guimen" | "txhj_guiyan" | "txhj_guofengyupao" | "txhj_huangjin" | "txhj_huangtiannizhuan" | "txhj_jiuweiSkill1"
        | "txhj_jiuweiSkill1_disable" | "txhj_jiuweiSkill2" | "txhj_jiuweiSkill3" | "txhj_juechenjinge" | "txhj_juhun" | "txhj_kapaikeshi"
        | "txhj_kuangbao" | "txhj_leijimu" | "txhj_lingbaoxianhu" | "txhj_liuliSkill1" | "txhj_liuliSkill2" | "txhj_liulongcanjia"
        | "txhj_luluSkill1" | "txhj_luluSkill2" | "txhj_luolei" | "txhj_manmanSkill1" | "txhj_manmanSkill2" | "txhj_minminSkill1"
        | "txhj_minminSkill2" | "txhj_oldhuxiao" | "txhj_piliche" | "txhj_qiaoqiaoSkill1" | "txhj_qiaoqiaoSkill2" | "txhj_qilinSkill1"
        | "txhj_qilinSkill2" | "txhj_qilinSkill3" | "txhj_qilingong" | "txhj_renfang" | "txhj_renwang" | "txhj_ruiSkill1"
        | "txhj_ruiSkill2" | "txhj_ruiSkill3" | "txhj_shiyuan" | "txhj_taijifuchen" | "txhj_taoyuanyishi" | "txhj_tengsheSkill1"
        | "txhj_tengsheSkill2" | "txhj_tengsheSkill3" | "txhj_tunshi" | "txhj_wangyuan" | "txhj_wangzu" | "txhj_wenhejibei"
        | "txhj_wuweizhile" | "txhj_xiaoxiaoSkill1" | "txhj_xiaoxiaoSkill2" | "txhj_xiefeng" | "txhj_xingtianpojunfu" | "txhj_xuanwuSkill1"
        | "txhj_xuanwuSkill2" | "txhj_xuanwuSkill3" | "txhj_xuerenSkill1" | "txhj_xuwangzhimian" | "txhj_yanSkill1" | "txhj_yanSkill2"
        | "txhj_yanSkill3" | "txhj_yaoshu" | "txhj_yayaSkill1" | "txhj_yayaSkill2" | "txhj_yayaSkill2_protected" | "txhj_youyouSkill1"
        | "txhj_youyouSkill2" | "txhj_yuanduandilue" | "txhj_yueerSkill1" | "txhj_yueerSkill2" | "txhj_yueyingdezhihui" | "txhj_zhoufa"
        | "txhj_zhoulei" | "txhj_zhouzu" | "txhj_zhuge" | "txhj_zhuhun" | "txhj_zhuwei" | "txhj_zuijiu"
        | "txhuao" | "txkeji" | "txlangxi" | "txleizhou" | "txlianbi" | "txlveming"
        | "txmengtai" | "txminguangkai" | "txtanbei" | "txtanbei_effect1" | "txtanbei_effect2" | "txtanbei_effect3"
        | "txyingyang" | "txyirang" | "txzhaoshu_global" | "txzhaoshu_skill" | "txzhennu" | "txzhulian"
        | "xinmingjian" | "xinmingjian2" | "zglkongcheng" => Some((EventKind::PhaseBegin, VmFilterPolicy::OwnerIsTarget)),
        _ => None,
    }
}
