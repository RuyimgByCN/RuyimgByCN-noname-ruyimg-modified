/// 相克表：`KE[nature]` 为 `nature` 所克的属性（金克木/木克土/水克火/
/// 火克金/土克水，编号 0=金 1=木 2=水 3=火 4=土）。

pub const KE: [u8; 5] = [1, 4, 3, 0, 2];

/// 相生表：`SHENG[nature]` 为 `nature` 所生的属性（金生水/木生火/
/// 水生木/火生土/土生金）。
pub const SHENG: [u8; 5] = [2, 3, 1, 4, 0];

pub const GJ_ZIYANGDAN_HUJIA: u8 = 3;

pub const GJ_FOOD_ROUNDS_CHUNBING: u8 = 5;

pub const GJ_FOOD_ROUNDS_DEFAULT: u8 = 3;

pub const GJ_FOOD_ROUNDS_LIYUTANG: u8 = 2;

pub const GJ_FOOD_ROUNDS_MIZHLIANOU: u8 = 4;

pub const GJ_FOOD_ROUNDS_TANHUADONG: u8 = 3;

pub const GJ_FOOD_ROUNDS_XIAJIAO: u8 = 4;

pub const GJ_FOOD_ROUNDS_YOUGENG: u8 = 3;

pub const D3_JINGGOU_TARGET: &str = "d3_jinggou_target";

pub const D3_MOYAN_TARGET: &str = "d3_moyan_target";

pub const D3_MOYAN_GAIN: &str = "d3_moyan_gain";

pub const GJ_BINGPOTONG_CARD_KEY: &str = "gj:bingpotong";

pub const GJ_BINGPO_SHOW_OWN: &str = "gj_bingpo_show_own";

pub const GJ_BINGPO_SHOW_TARGET: &str = "gj_bingpo_show_target";

pub const GJ_CHUNBING_CARD_KEY: &str = "gj:chunbing";

pub const GJ_DINVXUANSHUANG_CARD_KEY: &str = "gj:dinvxuanshuang";

pub const GJ_DINVXUANSHUANG_RIDER: &str = "gj_dinvxuanshuang_rider";

pub const GJ_FEIBIAO_DISCARD: &str = "gj_feibiao_discard";

pub const GJ_FEIBIAO_KEY: &str = "gj:feibiao";

pub const GJ_FOOD_CHUNBING_KEY: &str = "gj_food_chunbing";

pub const GJ_FOOD_GUDONGGENG_KEY: &str = "gj_food_gudonggeng";

pub const GJ_FOOD_LIYUTANG_KEY: &str = "gj_food_liyutang";

pub const GJ_FOOD_LUYUGENG_KEY: &str = "gj_food_luyugeng";

pub const GJ_FOOD_MAPODOUFU_KEY: &str = "gj_food_mapodoufu";

pub const GJ_FOOD_MIZHLIANOU_KEY: &str = "gj_food_mizhilianou";

pub const GJ_FOOD_MOLICHA_KEY: &str = "gj_food_molicha";

pub const GJ_FOOD_QINGTUAN_KEY: &str = "gj_food_qingtuan";

pub const GJ_FOOD_SHIHUIFEN_BAN_KEY: &str = "gj_food_shihuifen_ban";

pub const GJ_FOOD_TANHUADONG_KEY: &str = "gj_food_tanhuadong";

pub const GJ_FOOD_XIAJIAO_KEY: &str = "gj_food_xiajiao";

pub const GJ_FOOD_YOUGENG_KEY: &str = "gj_food_yougeng";

pub const GJ_FOOD_YUANBAOROU_KEY: &str = "gj_food_yuanbaorou";

pub const GJ_FOOD_ZIYANGDAN_KEY: &str = "gj_food_ziyangdan";

pub const GJ_GUDONGGENG_CARD_KEY: &str = "gj:gudonggeng";

pub const GJ_HEILONGLINPIAN_CARD_KEY: &str = "gj:heilonglinpian";

pub const GJ_JINLIANZHU_CONFIRM: &str = "gj_jinlianzhu_confirm";

pub const GJ_JINLIANZHU_KEY: &str = "gj_jinlianzhu_listener";

pub const GJ_LIUFENGSAN_CARD_KEY: &str = "gj:liufengsan";

pub const GJ_LIUTOUGE_CARD_KEY: &str = "gj:liutouge";

pub const GJ_LIYUTANG_CARD_KEY: &str = "gj:liyutang";

pub const GJ_LONGXUGOU_CARD_KEY: &str = "gj:longxugou";

pub const GJ_LONGXU_PICK: &str = "gj_longxu_pick";

pub const GJ_LUYUGENG_CARD_KEY: &str = "gj:luyugeng";

pub const GJ_LUYUGENG_COST: &str = "gj_luyugeng_cost";

pub const GJ_MAPODOUFU_CARD_KEY: &str = "gj:mapodoufu";

pub const GJ_MIANLI_KEY: &str = "gj:mianlijinzhen";

pub const GJ_MIZHLIANOU_CARD_KEY: &str = "gj:mizhilianou";

pub const GJ_MOLICHA_CARD_KEY: &str = "gj:molicha";

pub const GJ_MUTOU_SKILL_KEY: &str = "gj_mutoumianju_skill";

pub const GJ_QIANKUNBIAO_CARD_KEY: &str = "gj:qiankunbiao";

pub const GJ_QINGTUAN_CARD_KEY: &str = "gj:qingtuan";

pub const GJ_SHATANG_CARD_KEY: &str = "gj:shatang";

pub const GJ_SHENHUOFEIYA_CARD_KEY: &str = "gj:shenhuofeiya";

pub const GJ_SHIHUIFEN_CARD_KEY: &str = "gj:shihuifen";

pub const GJ_SHUJINSAN_DISCARD: &str = "gj_shujinsan_discard";

pub const GJ_SHUJINSAN_KEY: &str = "gj:shujinsan";

pub const GJ_TANHUADONG_CARD_KEY: &str = "gj:tanhuadong";

pub const GJ_XIAJIAO_CARD_KEY: &str = "gj:xiajiao";

pub const GJ_XIAJIAO_DISCARD: &str = "gj_xiajiao_discard";

pub const GJ_YOUGENG_CARD_KEY: &str = "gj:yougeng";

pub const GJ_YUANBAOROU_CARD_KEY: &str = "gj:yuanbaorou";

pub const GJ_YUHENG_SKILL_KEY: &str = "gj_yuheng_skill";

pub const GJ_YUNVYUANSHEN_CARD_KEY: &str = "gj:yunvyuanshen";

pub const GJ_YUNVYUANSHEN_LISTENER_KEY: &str = "gj_yunvyuanshen_listener";

pub const GJ_ZIYANGDAN_CARD_KEY: &str = "gj:ziyangdan";

pub const GW_AOZU_CARD_KEY: &str = "gw:aozuzhilei";

pub const GW_BAOXUE_CARD_KEY: &str = "gw:baoxueyaoshui";

pub const GW_BAOXUE_DISCARD: &str = "gw_baoxue_discard";

pub const GW_BIRIN_CARD_KEY: &str = "gw:birinongwu";

pub const GW_BIRIN_STATE_KEY: &str = "gw_birin_state";

pub const GW_CIGU_CARD_KEY: &str = "gw:ciguhanshuang";

pub const GW_CIGU_STATE_KEY: &str = "gw_cigu_state";

pub const GW_FENGYIN_STATE_KEY: &str = "gw_fengyin_state";

pub const GW_KUNEN_CARD_KEY: &str = "gw:kunenfayin";

pub const GW_KUNEN_STATE_KEY: &str = "gw_kunen_state";

pub const GW_POXIAO_BRANCH: &str = "gw_poxiao_branch";

pub const GW_POXIAO_CARD_KEY: &str = "gw:poxiao";

pub const GW_QINPEN_CARD_KEY: &str = "gw:qinpendayu";

pub const GW_QINPEN_STATE_KEY: &str = "gw_qinpen_state";

pub const GW_SHANBENG_CARD_KEY: &str = "gw:shanbengshu";

pub const GW_WEATHER_GRANT_KEY: &str = "gw:weather_grant";

pub const GW_WENYI_CARD_KEY: &str = "gw:wenyi";

pub const GW_YANZHI_CARD_KEY: &str = "gw:yanziyaoshui";

pub const GW_ZIHUAN_CARD_KEY: &str = "gw:zhihuanjun";

pub const GW_ZUMO_CARD_KEY: &str = "gw:zumoshoukao";

pub const HH_CHUANSONG_CARD_KEY: &str = "hh:chuansongmen";

pub const HH_DUNPAI_CARD_KEY: &str = "hh:dunpaigedang";

pub const HH_DUNPAI_DISCARD: &str = "hh_dunpai_discard";

pub const HH_SHIJIE_CARD_KEY: &str = "hh:shijieshu";

pub const HH_TANSHE_CARD_KEY: &str = "hh:tanshezhiren";

pub const HH_XINGJIE_CARD_KEY: &str = "hh:xingjiegoutong";

pub const HH_YUANSU_CARD_KEY: &str = "hh:yuansuhuimie";

pub const HH_YUANSU_DISCARD: &str = "hh_yuansu_discard";

pub const HH_ZHILIAOBO_CARD_KEY: &str = "hh:zhiliaobo";

pub const HOST_SHOULIJIAN_KEY: &str = "host:shoulijian";

pub const HOST_TARGET_DISCARD_EQUIP: &str = "host_target_discard_equip";

pub const HT_HEARTH_KEY_PREFIX: &str = "ht:";

pub const HT_JIHUO_KEY: &str = "ht:jihuocard";

pub const HT_LINGHUN_KEY: &str = "ht:linghunzhihuo";

pub const HT_ZHAOMING_KEY: &str = "ht:zhaomingdan";

pub const HUANLE_MIANJI_KEY: &str = "huanle:mianju";

pub const HUANLE_MONKEY_KEY: &str = "huanle:monkey";

pub const HUANLE_XUELUNYANG_KEY: &str = "huanle:xuelunyang";

pub const MTG_BINGHEYAOSAI_DISCARD: &str = "mtg_bingheyaosai_discard";

pub const MTG_BINGHEYAOSAI_KEY: &str = "mtg_bingheyaosai_land";

pub const MTG_FEIXU_DISCOVER: &str = "mtg_feixu_discover";

pub const MTG_FEIXU_KEY: &str = "mtg_feixu_land";

pub const MTG_HAIDAO_DISCARD: &str = "mtg_haidao_discard";

pub const MTG_HAIDAO_KEY: &str = "mtg_haidao_land";

pub const MTG_LINDIXILIU_DISCARD: &str = "mtg_lindixiliu_discard";

pub const MTG_LINDIXILIU_KEY: &str = "mtg_lindixiliu_land";

pub const MTG_LINZHONGJIANTA_CARD: &str = "mtg_linzhongjianta_card";

pub const MTG_LINZHONGJIANTA_DISCOVER: &str = "mtg_linzhongjianta_discover";

pub const MTG_LINZHONGJIANTA_KEY: &str = "mtg_linzhongjianta_land";

pub const MTG_LINZHONGJIANTA_SKILL_KEY: &str = "mtg_linzhongjianta_strike";

pub const MTG_LINZHONGJIANTA_TARGET: &str = "mtg_linzhongjianta_target";

pub const MTG_LONGSHANFENG_GAIN: &str = "mtg_longshanfeng_gain";

pub const MTG_LONGSHANFENG_KEY: &str = "mtg_longshanfeng_land";

pub const MTG_SHAMOLVZHOU_DISCOVER: &str = "mtg_shamolvzhou_discover";

pub const MTG_SHAMOLVZHOU_KEY: &str = "mtg_shamolvzhou_land";

pub const MTG_SHUIMOMUXUE_DISCARD: &str = "mtg_shuimomuxue_discard";

pub const MTG_SHUIMOMUXUE_KEY: &str = "mtg_shuimomuxue_land";

pub const MTG_YIXIALAN_GAIN: &str = "mtg_yixialan_gain";

pub const MTG_YIXIALAN_KEY: &str = "mtg_yixialan_land";

pub const MTG_YIXIALAN_RECAST_CARD: &str = "mtg_yixialan_recast_card";

pub const MTG_YIXIALAN_RECAST_KEY: &str = "mtg_yixialan_recast";

pub const MTG_YOULIN_DISCARD: &str = "mtg_youlin_trick";

pub const MTG_YOULIN_DISCOVER_KEY: &str = "mtg_youlin_discover";

pub const MTG_YOULIN_KEY: &str = "mtg_youlin_land";

pub const MTG_YOULIN_PICK: &str = "mtg_youlin_pick";

pub const SWD_GOUHUNLUO_KEY: &str = "swd_gouhunluo";

pub const SWD_GOUHUNLUO_SUB_KEY: &str = "swd_gouhunluo_sub";

pub const SWD_JIGUANTONG_DISCARD: &str = "swd_jiguantong_discard";

pub const SWD_JIGUANTONG_KEY: &str = "swd_jiguantong";

pub const SWD_JIGUANFENG_KEY: &str = "swd_jiguanfeng";

pub const SWD_FENGYIN_STATE_KEY: &str = "swd_fengyin_state";

/// 枕流·临月触发面内部键（挂附目标的准备阶段取回+回复）。
pub const SWD_ZHENJIU2_KEY: &str = "swd_zhenjiu2";

/// 麻醉·临月触发面内部键（挂附者伤害前置返还+降 1）。
pub const SWD_LMAZUI2_KEY: &str = "swd_lmazui2";

pub const SWD_HUAJIN2_KEY: &str = "swd_huajin2";

/// 幻形在位旗标内部键（授予技能的存续标记——受伤后清）。
pub const SWD_HUANXING_STATE_KEY: &str = "swd_huanxing_state";

pub const SWD_JIGUANFENG_SHAN: &str = "swd_jiguanfeng_shan";

pub const SWD_JIGUANYUAN_KEY: &str = "swd_jiguanyuan";

pub const SWD_JIGUANYUAN_GAIN_KEY: &str = "swd_jiguanyuan_gain";

pub const SWD_JIGUANYUAN_CHOOSE: &str = "swd_jiguanyuan_choose";

pub const SWD_JIGUANYAOSHU_KEY: &str = "swd_jiguanyaoshu";

pub const SWD_JIGUANYAOSHU_STATE_KEY: &str = "swd_jiguanyaoshu_state";

pub const SWD_XUANFENG_KEY: &str = "swd_xuanfengzhiren";

pub const SWD_ZHONGXING_KEY: &str = "swd_zhongxinghujia";

pub const SWD_XINGTIGAIZAO_KEY: &str = "swd_xingtigaizao";

pub const SWD_SHIJIANHUISU_KEY: &str = "swd_shijianhuisu";

pub const SWD_JINJILENGDONG_KEY: &str = "swd_jinjilengdong";

pub const SWD_SHENGXIUHAOJIAO_KEY: &str = "swd_shengxiuhaojiao";

pub const SWD_YINMILICHANG_KEY: &str = "swd_yinmilichang";

pub const SWD_JIGUANYAO_GAIN_KEY: &str = "swd_jiguanyao_gain";

pub const SWD_JIGUANSHU_KEY: &str = "swd_jiguanshu";

pub const SWD_TIANFUZHEN_KEY: &str = "swd_tianfuzhen";

pub const SWD_QIXINGZHEN_KEY: &str = "swd_qixingzhen";

pub const SWD_NIAOXIANGZHEN_KEY: &str = "swd_niaoxiangzhen";

pub const SWD_CHANGSHEZHEN_KEY: &str = "swd_changshezhen";

pub const SWD_POZHENJUE_KEY: &str = "swd_pozhenjue";

pub const SWD_LANGEGUAIYI_SKILL_KEY: &str = "swd_langeguaiyi_skill";

pub const SWD_LIUXING_DISCARD: &str = "swd_liuxing_discard";

pub const SWD_LIUXING_KEY: &str = "gj:swd_liuxinghuoyu";

pub const SWD_MUJIAREN_KEY: &str = "swd_mujiaren";

pub const SWD_SHENTOU_COST: &str = "swd_shentou_cost";

pub const SWD_SHENTOU_SKILL_KEY: &str = "swd_shentou_skill";

pub const SWD_YANGPIJUAN_CATEGORY: &str = "swd_yangpijuan_category";

pub const SWD_YANGPIJUAN_KEY: &str = "swd_yangpijuan";

pub const SWD_YANGPIJUAN_PICK: &str = "swd_yangpijuan_pick";

pub const SWD_YUFULU_SKILL_KEY: &str = "swd_yufulu_skill";

pub const SWD_YUFULU_TARGET: &str = "swd_yufulu_target";

pub const SWD_YUFULU_WEAPON: &str = "swd_yufulu_weapon";

pub const SWD_ZHILUXIAOHU_KEY: &str = "swd_zhiluxiaohu";

pub const WD_CAOCHUAN_BRANCH: &str = "wd_caochuan_branch";

pub const WD_CAOCHUAN_KEY: &str = "wd:caochuanjiejian";

pub const WD_CHENHUO_KEY: &str = "wd:chenhuodajie";

pub const WD_FUDI_COMPARE: &str = "wd_fudi_compare";

pub const WD_FUDI_KEY: &str = "wd:fudichouxin";

pub const WD_FUDI_PLAYER_PICK: &str = "wd_fudi_player_pick";

pub const WD_FUDI_TARGET_PICK: &str = "wd_fudi_target_pick";

pub const WD_SANJIAN_DISCARD: &str = "wd_sanjian_discard";

pub const WD_SANJIAN_TARGET: &str = "wd_sanjian_target";

pub const WD_SHEZHAN_KEY: &str = "wd:shezhanqunru";

pub const WD_SHUIGONG_KEY: &str = "wd:shuigong";

pub const WD_TOULIANG_KEY: &str = "wd:toulianghuanzhu";

pub const WD_TOULIANG_OWN: &str = "wd_touliang_own";

pub const WD_TOULIANG_TARGET: &str = "wd_touliang_target";

pub const WD_WANGMEIZHIKE_KEY: &str = "wd:wangmeizhike";

pub const WUDIAN_BAOLEI_CONFIRM: &str = "wudian_baolei_confirm";

pub const WUDIAN_BAOLEI_KEY: &str = "bshaowei";

pub const WUDIAN_BAOXUE_TARGETS: &str = "wudian_baoxue_targets";

pub const WUDIAN_CHEZHEN_KEY: &str = "chezhen";

pub const WUDIAN_CIQIU_KEY: &str = "ciqiu";

pub const WUDIAN_DUWEN_KEY: &str = "duwen";

pub const WUDIAN_FEIDAN_DISCARD: &str = "wudian_feidan_discard";

pub const WUDIAN_FEIDAN_KEY: &str = "feidan";

pub const WUDIAN_FEIREN_KEY: &str = "feiren";

pub const WUDIAN_FEITIAO_CARDS: &str = "wudian_feitiao_cards";

pub const WUDIAN_FEITIAO_TARGET: &str = "wudian_feitiao_target";

pub const WUDIAN_GEJU_KEY: &str = "geju";

pub const WUDIAN_GUANGSHU_CARD: &str = "wudian_guangshu_card";

pub const WUDIAN_GUANGSHU_CLUB_KEY: &str = "guangshu_club";

pub const WUDIAN_GUANGSHU_DIAMOND_KEY: &str = "guangshu_diamond";

pub const WUDIAN_GUANGSHU_HEART_KEY: &str = "guangshu_heart";

pub const WUDIAN_GUANGSHU_KEY: &str = "guangshu";

pub const WUDIAN_GUANGSHU_SPADE_KEY: &str = "guangshu_spade";

pub const WUDIAN_GUANGSHU_TARGET: &str = "wudian_guangshu_target";

pub const WUDIAN_HUHUAN_CARDS: &str = "wudian_huhuan_cards";

pub const WUDIAN_JIJIA_KEY: &str = "jijia";

pub const WUDIAN_JUEDAO_KEY: &str = "juedao";

pub const WUDIAN_KANGYIN_DISCARD: &str = "wudian_kangyin_discard";

pub const WUDIAN_KANGYIN_DRAW: &str = "wudian_kangyin_draw";

pub const WUDIAN_KANGYIN_KEY: &str = "kangyin";

/// 禁言（玩点论杀续批）：PlayCard 门查询锚。
pub const WUDIAN_JINYAN_KEY: &str = "jinyan";

/// 复仇（玩点论杀批 14）：ShaResolveMachine 内联取消查询锚。
pub const WUDIAN_FUCHOU_KEY: &str = "fuchou";

/// 归汉（玩点论杀批 17）：DyingSaveMachine 内联确认查询锚。
pub const WUDIAN_GUIHAN_KEY: &str = "ycnguihan";

pub const WUDIAN_KANGYIN_TARGET: &str = "wudian_kangyin_target";

pub const WUDIAN_KUOYIN_CARDS: &str = "wudian_kuoyin_cards";

pub const WUDIAN_LIANGOU_CARDS: &str = "wudian_liangou_cards";

pub const WUDIAN_LIANGOU_TARGET: &str = "wudian_liangou_target";

pub const WUDIAN_LONGXIANG_KEY: &str = "longxiang";

pub const WUDIAN_LUEYING_KEY: &str = "lueying";

pub const WUDIAN_MUJING_CARD: &str = "wudian_mujing_card";

pub const WUDIAN_MUJING_KEY: &str = "mujing";

pub const WUDIAN_MUJING_TARGET: &str = "wudian_mujing_target";

pub const WUDIAN_PAOTAI_CARD: &str = "wudian_paotai_card";

pub const WUDIAN_PAOTAI_FIRE_KEY: &str = "paotai_fire";

pub const WUDIAN_PAOTAI_KEY: &str = "paotai";

pub const WUDIAN_PINGZHANG_CARDS: &str = "wudian_pingzhang_cards";

pub const WUDIAN_QIANGGU_CARDS: &str = "wudian_qianggu_cards";

pub const WUDIAN_QIANGGU_GUARD_ASK: &str = "wudian_qianggu_guard_ask";

pub const WUDIAN_SHOUHU_CARD: &str = "wudian_shouhu_card";

pub const WUDIAN_SHOUHU_KEY: &str = "shouhu";

pub const WUDIAN_SHOUHU_TARGET: &str = "wudian_shouhu_target";

pub const WUDIAN_SHUAIYAN_CHOICE: &str = "wudian_shuaiyan_choice";

pub const WUDIAN_SHUAIYAN_DISCARD: &str = "wudian_shuaiyan_discard";

pub const WUDIAN_SHUAIYAN_KEY: &str = "nsshuaiyan";

pub const WUDIAN_TUJI_KEY: &str = "tuji";

pub const WUDIAN_XIANDAN_KEY: &str = "xiandan";

pub const WUDIAN_XJ_CHANGNIAN_TARGET: &str = "xj_changnian_target";

pub const WUDIAN_XJ_CTIANFU_CARD: &str = "xj_ctianfu_card";

pub const WUDIAN_XJ_CTIANFU_TARGET: &str = "xj_ctianfu_target";

pub const WUDIAN_XJ_DIEWU_CARDS: &str = "xj_diewu_cards";

pub const WUDIAN_XJ_DIEWU_TARGET: &str = "xj_diewu_target";

pub const WUDIAN_XJ_DUCI_TARGET: &str = "xj_duci_target";

pub const WUDIAN_XJ_FENGLUE_TARGET: &str = "xj_fenglue_target";

pub const WUDIAN_XJ_GUANRI_CARDS: &str = "xj_guanri_cards";

pub const WUDIAN_XJ_GUANRI_TARGET: &str = "xj_guanri_target";

pub const WUDIAN_XJ_GUIJIN_CARDS: &str = "xj_guijin_cards";

pub const WUDIAN_XJ_GUIJIN_TARGET: &str = "xj_guijin_target";

pub const WUDIAN_XJ_GUIYUAN_CARDS: &str = "xj_guiyuan_cards";

pub const WUDIAN_XJ_HUAHU_TARGETS: &str = "xj_huahu_targets";

pub const WUDIAN_XJ_HUSHA_MODE: &str = "xj_husha_mode";

pub const WUDIAN_XJ_HUSHA_TARGET: &str = "xj_husha_target";

pub const WUDIAN_XJ_LIGUANG_CARDS: &str = "xj_liguang_cards";

pub const WUDIAN_XJ_LINGQUAN_CARDS: &str = "xj_lingquan_cards";

pub const WUDIAN_XJ_LINGYU_TARGET: &str = "xj_lingyu_target";

pub const WUDIAN_XJ_LUANJIAN_CARDS: &str = "xj_luanjian_cards";

pub const WUDIAN_XJ_LUANJIAN_TARGETS: &str = "xj_luanjian_targets";

pub const WUDIAN_XJ_MINSHA_CARDS: &str = "xj_minsha_cards";

pub const WUDIAN_XJ_MINSHA_TARGET: &str = "xj_minsha_target";

pub const WUDIAN_XJ_POYUN_CARDS: &str = "xj_poyun_cards";

pub const WUDIAN_XJ_QIANFANG_CARDS: &str = "xj_qianfang_cards";

pub const WUDIAN_XJ_RUNXIN_TARGET: &str = "xj_runxin_target";

pub const WUDIAN_XJ_SAJIN_CARDS: &str = "xj_sajin_cards";

pub const WUDIAN_XJ_SAJIN_TARGETS: &str = "xj_sajin_targets";

pub const WUDIAN_XJ_SHUIYUN5_CARDS: &str = "xj_shuiyun5_cards";

pub const WUDIAN_XJ_SHUIYUN_CARDS: &str = "xj_shuiyun_cards";

pub const WUDIAN_XJ_TIANJIAN_CARDS: &str = "xj_tianjian_cards";

pub const WUDIAN_XJ_TIANXIAN_KEY: &str = "tianxian";

/// 锦鳞·退（swd 司空蔷，swd deferred 消解批 114）：目录技能名（包 `skills`
/// 段声明 `swd_jinlin2` → `host:jinlin2`，运行时由锦鳞授予实例）。
pub const SWD_JINLIN2_KEY: &str = "swd_jinlin2";

/// 禳星（T11 Boss 诸葛亮族批 115）：目录技能名。
pub const HLSS_XIANGXING_KEY: &str = "hlss_xiangxing";

/// 隐月（同批）：目录技能名（隐月·判为其 group 技能）。
pub const HLSS_YUEYIN_KEY: &str = "hlss_yueyin";

/// 隐月·判（同批）：目录技能名。
pub const HLSS_YUEYIN2_KEY: &str = "hlss_yueyin2";

/// 隐月记录位：雷属性伤害。
pub const HLSS_DAMAGE_THUNDER: u8 = 1;

/// 隐月记录位：火属性伤害。
pub const HLSS_DAMAGE_FIRE: u8 = 2;

/// 隐月记录位：锦囊牌伤害。
pub const HLSS_DAMAGE_TRICK: u8 = 4;

/// 起风（同批）：目录技能名。
pub const HLSS_FENGQI_KEY: &str = "hlss_fengqi";

/// 烈弓（L20 标记补充批 3）：目录技能名（条件直伤）。
pub const L20_FZ_LIEGONG_KEY: &str = "fz_liegong";

/// 狂骨（L20 批 3）：目录技能名（造成伤害后回复/摸牌）。
pub const L20_FZ_XINKUANGGU_KEY: &str = "fz_xinkuanggu";

/// 苦肉（L20 标记补充批 2）：目录技能名（失 1 体力摸 2 张）。
pub const L20_KUROU_KEY: &str = "kurou";

/// 主亡忠继（L20 批 2）：目录技能名（主公阵亡时忠臣继位）。
pub const L20_ZHUWANZHONGJI_KEY: &str = "zhuwanzhongji";

/// 摸牌翻倍（L20 标记补充批 1）：目录技能名（摸牌 +2）。
pub const L20_MOPAI_KEY: &str = "xigua_mopaifanbei";

/// 血量翻倍（L20 批 1）：目录技能名（开局上限与体力翻倍）。
pub const L20_XUELIANG_KEY: &str = "xigua_xueliangfanbei";

/// 门神（U08 英雄杀批 2）：目录技能名（结束阶段给他人门神标记）。
pub const U08_YXS_MENSHEN_KEY: &str = "yxs_menshen";

/// 门神·清除（U08 批 2）：目录技能名（阶段开始/死亡时清除标记）。
pub const U08_YXS_MENSHEN3_KEY: &str = "yxs_menshen3";

/// 门神·标记持有（U08 批 2）：目录技能名（主体效果待批次）。
pub const U08_YXS_MENSHEN2_KEY: &str = "yxs_menshen2";

/// 反击（U08 英雄杀批 1）：目录技能名（受伤后反击）。
pub const U08_YXS_FANJI_KEY: &str = "yxs_fanji";

/// 反击·红杀直伤（U08 批 1）：目录技能名（红杀不可闪避）。
pub const U08_YXS_FANJI2_KEY: &str = "yxs_fanji2";

/// 灾变（L19 末日乱斗批 2）：目录技能名（出牌阶段开始按身份数摸牌）。
pub const L19_ZAIBIAN_KEY: &str = "zaibian";

/// 誓师（L19 末日乱斗批 1）：目录技能名（击杀后上限 +1 并回复 1）。
pub const L19_SHISHI_KEY: &str = "shishi";

/// 循梦（L19 末日乱斗批 1）：目录技能名（杀伤害 +1，自己失去 1 体力）。
pub const L19_XUNMENG_KEY: &str = "xunmeng";

/// 集舍（L27 选将美化批 5）：目录技能名（摸 1 + 标记；手牌上限面上游已注释）。
pub const L27_JISHE_KEY: &str = "jishe";

/// 诈降（L27 选将美化批 4）：目录技能名（用牌数 < 已损失体力 → 无限制+直伤）。
pub const L27_SBZHAXIANG_KEY: &str = "sbzhaxiang";

/// 每步（L27 选将美化批 3）：目录技能名（弃牌令他人本回合止息）。
pub const L27_NEW_MEIBU_KEY: &str = "new_meibu";

/// 每步·距离（L27 批 3）：目录技能名（到弃牌者距离视为 1）。
pub const L27_NEW_MEIBU_RANGE_KEY: &str = "new_meibu_range";

/// 止息（L27 选将美化批 2）：目录技能名（用牌计数 + 锦囊记录 → 封锁出牌）。
pub const L27_NEW_ZHIXI_KEY: &str = "new_zhixi";

/// 止息·锦囊记录（L27 批 2）：标记键所用技能名。
pub const L27_NEW_ZHIXI_MARK2_KEY: &str = "new_zhixi_mark2";

/// 撷芳（L27 选将美化批 1）：目录技能名（距离 -存活女性数 + 女性数标记）。
pub const L27_XIEFANG_KEY: &str = "xiefang";

/// 让节（史诗卡牌批 2）：目录技能名（受伤后获得指定类型牌/移动场上牌）。
pub const EPIC_RANGJIE_KEY: &str = "rangjie";

/// 蛊惑（史诗卡牌批 1）：目录技能名（弃手牌声明任意基本/锦囊牌使用）。
pub const EPIC_REGUHUO_KEY: &str = "reguhuo";

/// 攻心（swd 批 155）：目录技能名（看手牌选红桃：弃置或置牌堆顶）。
pub const SWD_GONGXIN_KEY: &str = "gongxin";

/// 魔化（swd 批 154）：目录技能名（濒死变身撒旦形态）。
pub const SWD_MOHUA2_KEY: &str = "mohua2";

/// 修罗（swd 批 153）：目录技能名（弃手牌清同花色判定区延时锦囊）。
pub const SWD_XIULUO_KEY: &str = "swd_xiuluo";

/// 灭道（swd 批 152）：目录技能名（摸牌多摸/弃牌多弃「已损失体力」）。
pub const SWD_MIEDAO_KEY: &str = "miedao";

/// 魔焰（swd 批 151）：目录技能名（弃红牌对等量目标各 1 点火焰伤害）。
pub const SWD_MOYAN_KEY: &str = "moyan";

/// 疾风（swd 批 150）：目录技能名（单目标牌可多目标，多目标时概率失误）。
pub const SWD_JIFENG_KEY: &str = "jifeng";

/// 昆仑镜（swd 批 149）：目录技能名（阶段收束快照 / 体力下降时回滚）。
pub const SWD_KUNLUNJING_KEY: &str = "kunlunjing";

/// 神弓（swd 批 148）：目录技能名（借用他人防具/武器响应）。
pub const SWD_SHENGONG_KEY: &str = "shengong";

/// 轩辕（swd 批 146）：目录技能名（结束阶段弃黑桃取【轩辕剑】）。
pub const SWD_XUANYUAN_KEY: &str = "xuanyuan";

/// 燎原（swd 批 145）：目录技能名（弃同花色牌换额外闪需求与加伤）。
pub const SWD_LIAOYUAN_KEY: &str = "liaoyuan";

/// 改造（swd 批 143）：目录技能名（装备牌改放到空栏位）。
pub const SWD_GAIZAO_KEY: &str = "gaizao";

/// 天书（swd 批 142）：目录技能名（获取目标武将的一个未持有技能）。
pub const SWD_TIANSHU_KEY: &str = "swdtianshu";

/// 李香龙（swd 批 141）：目录技能名（杀被闪避后对射程内目标用虚拟杀）。
pub const SWD_LXIANGLONG_KEY: &str = "lxianglong";

/// 移花（swd 批 140）：目录技能名（单目标牌反弹：弃 2 张手牌对调使用/目标）。
pub const SWD_YIHUA_KEY: &str = "yihua";

/// 贞烈（swd 批 139）：目录技能名（成为杀/锦囊目标时失体力并移出目标）。
pub const SWD_ZHENLIE_KEY: &str = "zhenlie";

/// 天伦（swd 批 138）：目录技能名（判定区牌替换本次判定）。
pub const SWD_TIANLUN_KEY: &str = "tianlun";

/// 脉轮（swd 批 137）：目录技能名（阶段开始 8 选 1 效果）。
pub const SWD_MAILUN_KEY: &str = "mailun";

/// 极略（swd 批 136）：目录技能名（牌堆顶 3 选 1 使用，每阶段 3 次）。
pub const SWD_JILVE_KEY: &str = "jilve";

/// 革新印（swd 批 135）：目录技能名（弃一花色手牌换另一花色等量牌）。
pub const SWD_GXIANYIN_KEY: &str = "gxianyin";

/// 冰封（swd 批 134）：目录技能名（翻面封锁限定技）。
pub const SWD_BINGFENG_KEY: &str = "bingfeng";

/// 冰封·封（同批）：目录技能名（出牌封锁旗标；翻回正面自移除）。
pub const SWD_BINGFENG2_KEY: &str = "bingfeng2";

/// 此桥邪（swd 批 133）：目录技能名（失去装备后使用机关牌）。
pub const SWD_CYQIAOXIE_KEY: &str = "cyqiaoxie";

/// 魂殉志（swd 批 132）：目录技能名（万箭 + 授予武圣/咆哮 + 出牌阶段末死亡）。
pub const SWD_HXUNZHI_KEY: &str = "hxunzhi";

/// 魂殉志·殉（同批）：目录技能名（旗标；内联臂于出牌阶段收束）。
pub const SWD_HXUNZHI2_KEY: &str = "hxunzhi2";

/// 护天（swd 批 131）：目录技能名（阶段结束寄存牌作护体）。
pub const SWD_HUTIAN_KEY: &str = "hutian";

/// 天香（swd 批 130）：目录技能名（受伤前弃红桃转移伤害）。
pub const SWD_TIANXIANG_KEY: &str = "tianxiang";

/// 天香·受（同批）：目录技能名（运行时授予；按失去体力摸牌后移除）。
pub const SWD_TIANXIANG2_KEY: &str = "tianxiang2";

/// 颂词（swd 批 129）：目录技能名（选未记录角色：弃 2 或摸 2）。
pub const SONGCI_KEY: &str = "songci";

/// 诛害（swd 批 128）：目录技能名（结束阶段角色造成过伤害则对其用杀）。
pub const SWD_ZHUHAI_KEY: &str = "zhuhai";

/// 林韵（swd 批 127）：目录技能名（两张牌当杀）。
pub const SWD_LINYUN_KEY: &str = "linyun";

/// 集控（swd 批 126）：目录技能名（阶段开始/失去手牌后虚拟雷杀；使用时消费次数）。
pub const SWD_JIKONG_KEY: &str = "jikong";

/// 魅惑（swd 批 125）：目录技能名（失去装备后取他人一张牌）。
pub const SWD_MEIHUO_KEY: &str = "meihuo";

/// 守印（swd 批 124）：目录技能名（濒死全员回复限定技）。
pub const SWD_SHOUYIN_KEY: &str = "shouyin";

/// 义山（swd 批 123）：目录技能名（受伤取回最近失去的牌）。
pub const SWD_YISHAN_KEY: &str = "yishan";

/// 入梦（swd 闻人绘，批 122）：目录技能名（他人出牌阶段前弃牌/跳阶段）。
pub const SWD_RUMENG_KEY: &str = "rumeng";

/// 神焰（swd 夜鸦熙，批 121）：目录技能名（火伤 +1 + 邻位蔓延；两面同机）。
pub const SWD_SHENYAN_KEY: &str = "shenyan";

/// 灵武（swd 妮可，批 120）：目录技能名（额外阶段；效果内联于阶段收束）。
pub const SWD_LINGWU_KEY: &str = "lingwu";

/// 清正（批 119）：目录技能名（道心值 25 阈值解锁；手杀包外部技能已转换）。
pub const MBCM_QINGZHENG_KEY: &str = "mbcmqingzheng";

/// 神威（T11 Boss 庞统族批 118）：目录技能名（浴火授予；摸牌/手牌上限臂）。
pub const BOSS_SHENWEI_KEY: &str = "神威";

/// 涅槃（T11 Boss 庞统族批 118）：目录技能名。
pub const BOSS_NIEPAN_KEY: &str = "涅槃";

/// 浴火（同批）：目录技能名（涅槃后授予三技 + 关卡循环型）。
pub const BOSS_YUHOU_KEY: &str = "浴火";

/// 潜龙（D5 龙血玄黄批 117）：目录技能名（道心值系统）。
pub const HLSS_MBQIANLONG_KEY: &str = "mbqianlong";

/// 卫统（同批）：目录技能名（主公技登记面）。
pub const HLSS_MBWEITONG_KEY: &str = "mbweitong";

/// 起风·闭（同批）：目录技能名（锁定——你的牌不可被无懈）。
pub const HLSS_FENGQI2_KEY: &str = "hlss_fengqi2";

/// 改命（同批）：目录技能名（锁定旗标——判定逻辑内联于判定机）。
pub const HLSS_GAIMING_KEY: &str = "hlss_gaiming";

pub const WUDIAN_XJ_WANGYOU_CARDS: &str = "xj_wangyou_cards";

pub const WUDIAN_XJ_XIEPAN_CARDS: &str = "xj_xiepan_cards";

pub const WUDIAN_XJ_XTANXI_CARDS: &str = "xj_xtanxi_cards";

pub const WUDIAN_XJ_XTANXI_TARGET: &str = "xj_xtanxi_target";

pub const WUDIAN_XJ_XUANMO_CARDS: &str = "xj_xuanmo_cards";

pub const WUDIAN_XJ_YINGFENG_KEY: &str = "xjyingfeng";

pub const WUDIAN_XJ_ZHAOYAO_CARDS: &str = "xj_zhaoyao_cards";

pub const WUDIAN_XJ_ZHIMENG_TARGET: &str = "xj_zhimeng_target";

pub const WUDIAN_XJ_ZHUYUE_CARDS: &str = "xj_zhuyue_cards";

pub const WUDIAN_XJ_ZHUYUE_TARGETS: &str = "xj_zhuyue_targets";

pub const WUDIAN_YINBO_CARDS: &str = "wudian_yinbo_cards";

pub const WUDIAN_YUEDONG_TARGETS: &str = "wudian_yuedong_targets";

pub const WUDIAN_ZHENCHA_CHOICE: &str = "wudian_zhencha_choice";

pub const WUDIAN_ZHENCHA_DRAW_KEY: &str = "zhencha";

pub const WUDIAN_ZHUCHENG_CONFIRM: &str = "wudian_zhucheng_confirm";

pub const WUDIAN_ZHUCHENG_GUARD_ASK: &str = "wudian_zhucheng_guard";

pub const WUDIAN_ZHUCHENG_GUARD_KEY: &str = "zhucheng2";

pub const WUDIAN_ZHUCHENG_KEY: &str = "zhucheng";

pub const WUDIAN_ZIYU_CHOICE: &str = "wudian_ziyu_choice";

pub const WUDIAN_ZIYU_KEY: &str = "ziyu";

pub const WUXING_SELECT_CARD: &str = "wuxing_select_card";

pub const WUXING_SELECT_NATURE: &str = "wuxing_select_nature";

pub const XJ_ANWUGU2_KEY: &str = "anwugu2";

pub const XJ_FANLING_KEY: &str = "fanling";

pub const XJ_TANNANG_KEY: &str = "tannang";

pub const XJ_YUEXING_KEY: &str = "yuexing";

pub const YC_GEAN_KEY: &str = "yc:geanguanhuo";

pub const YC_GEAN_T1_PICK: &str = "yc_gean_t1_pick";

pub const YC_GEAN_T2_PICK: &str = "yc_gean_t2_pick";

pub const WUDIAN_DUOQI_KEY: &str = "duoqi";

pub const WUDIAN_DUOQI_CARD: &str = "wudian_duoqi_card";

pub const WUDIAN_YOUZHAN_KEY: &str = "youzhan";

pub const WUDIAN_YOUZHAN_DISCARD: &str = "wudian_youzhan_discard";

pub const WUDIAN_CHOUDU_KEY: &str = "choudu";

pub const WUDIAN_CHOUDU_CARD: &str = "wudian_choudu_card";

pub const WUDIAN_CHOUDU_TARGET: &str = "wudian_choudu_target";

pub const WUDIAN_DIAOBING_KEY: &str = "diaobingqianjiang";

pub const WUDIAN_DIAOBING_SWAP: &str = "wudian_diaobing_swap";

pub const HLSS_MBCMJIUSHI_KEY: &str = "mbcmjiushi";
pub const HLSS_MBCMFANGZHU_KEY: &str = "mbcmfangzhu";
pub const HLSS_MBJUEJIN_KEY: &str = "mbjuejin";
pub const MBCMJIUSHI_GAIN_CONFIRM: &str = "mbcmjiushi_gain_confirm";
pub const MBCMJIUSHI_TURNBACK_CONFIRM: &str = "mbcmjiushi_turnback_confirm";
pub const MBCMFANGZHU_TARGET: &str = "mbcmfangzhu_target";
pub const MBCMFANGZHU_OPTION: &str = "mbcmfangzhu_option";

pub const EPIC_K_DANGGU_KEY: &str = "k_danggu";
pub const EPIC_K_MOWANG_KEY: &str = "k_mowang";
pub const K_DANGGU_PARTNER: &str = "k_danggu_partner";

pub const WUDIAN_LIDUAN_GIVE: &str = "wudian_liduan_give";

pub const WUDIAN_YOUDISHENRU_SHA: &str = "wudian_youdishenru_sha";

