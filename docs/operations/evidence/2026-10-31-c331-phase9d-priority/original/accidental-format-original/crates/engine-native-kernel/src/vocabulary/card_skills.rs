//! 牌种伴随技能和发现类别的不可变描述。

use super::host::*;
use crate::catalog::CardKind;

/// 牌种固有的装备技能（引擎按固定键查找）；包数据声明的装备技追加其后。

pub fn card_kind_host_skills(kind: crate::catalog::CardKind) -> &'static [HostSkillDecl] {
    match kind {
        CardKind::Jinlianzhu => &[HostSkillDecl {
            name: GJ_JINLIANZHU_KEY,
            behavior: HostBehavior::JinlianzhuCancel,
        }],
        CardKind::Tanhuadong => &[HostSkillDecl {
            name: GJ_FOOD_TANHUADONG_KEY,
            behavior: HostBehavior::GujianFoodTanhuadong,
        }],
        CardKind::Yougeng => &[HostSkillDecl {
            name: GJ_FOOD_YOUGENG_KEY,
            behavior: HostBehavior::GujianFoodYougeng,
        }],
        CardKind::Qingtuan => &[HostSkillDecl {
            name: GJ_FOOD_QINGTUAN_KEY,
            behavior: HostBehavior::GujianFoodQingtuan,
        }],
        CardKind::Yuanbaorou => &[HostSkillDecl {
            name: GJ_FOOD_YUANBAOROU_KEY,
            behavior: HostBehavior::GujianFoodYuanbaorou,
        }],
        CardKind::Gudonggeng => &[HostSkillDecl {
            name: GJ_FOOD_GUDONGGENG_KEY,
            behavior: HostBehavior::GujianFoodGudonggeng,
        }],
        CardKind::Mapodoufu => &[HostSkillDecl {
            name: GJ_FOOD_MAPODOUFU_KEY,
            behavior: HostBehavior::GujianFoodMapodoufu,
        }],
        CardKind::Chunbing => &[HostSkillDecl {
            name: GJ_FOOD_CHUNBING_KEY,
            behavior: HostBehavior::GujianFoodChunbing,
        }],
        CardKind::Liyutang => &[HostSkillDecl {
            name: GJ_FOOD_LIYUTANG_KEY,
            behavior: HostBehavior::GujianFoodLiyutang,
        }],
        CardKind::Molicha => &[HostSkillDecl {
            name: GJ_FOOD_MOLICHA_KEY,
            behavior: HostBehavior::GujianFoodMolicha,
        }],
        CardKind::Xiajiao => &[HostSkillDecl {
            name: GJ_FOOD_XIAJIAO_KEY,
            behavior: HostBehavior::GujianFoodXiajiao,
        }],
        CardKind::Mizhilianou => &[HostSkillDecl {
            name: GJ_FOOD_MIZHLIANOU_KEY,
            behavior: HostBehavior::GujianFoodMizhilianou,
        }],
        CardKind::Luyugeng => &[HostSkillDecl {
            name: GJ_FOOD_LUYUGENG_KEY,
            behavior: HostBehavior::GujianFoodLuyugeng,
        }],
        CardKind::Ziyangdan => &[HostSkillDecl {
            name: GJ_FOOD_ZIYANGDAN_KEY,
            behavior: HostBehavior::GujianFoodZiyangdan,
        }],
        CardKind::Yunvyuanshen => &[HostSkillDecl {
            name: GJ_YUNVYUANSHEN_LISTENER_KEY,
            behavior: HostBehavior::GujianYunvyuanshen,
        }],
        CardKind::Shihuifen => &[HostSkillDecl {
            name: GJ_FOOD_SHIHUIFEN_BAN_KEY,
            behavior: HostBehavior::GujianFoodShihuifenBan,
        }],
        CardKind::Mutoumianju => &[HostSkillDecl {
            name: GJ_MUTOU_SKILL_KEY,
            behavior: HostBehavior::GujianMutouEquipSkill,
        }],
        CardKind::Gjyuheng => &[HostSkillDecl {
            name: GJ_YUHENG_SKILL_KEY,
            behavior: HostBehavior::GujianYuhengEquipSkill,
        }],
        CardKind::Qinpendayu => &[HostSkillDecl {
            name: GW_QINPEN_STATE_KEY,
            behavior: HostBehavior::GwentQinpenState,
        }],
        CardKind::Birinongwu => &[HostSkillDecl {
            name: GW_BIRIN_STATE_KEY,
            behavior: HostBehavior::GwentBirinState,
        }],
        CardKind::Ciguhanshuang => &[HostSkillDecl {
            name: GW_CIGU_STATE_KEY,
            behavior: HostBehavior::GwentCiguState,
        }],
        CardKind::Zumoshoukao => &[HostSkillDecl {
            name: GW_FENGYIN_STATE_KEY,
            behavior: HostBehavior::GwentFengyinState,
        }],
        CardKind::Kunenfayin => &[HostSkillDecl {
            name: GW_KUNEN_STATE_KEY,
            behavior: HostBehavior::GwentKunenState,
        }],
        CardKind::Mianju => &[HostSkillDecl {
            name: HUANLE_MIANJI_KEY,
            behavior: HostBehavior::HuanleMianjuCard,
        }],
        CardKind::Monkey => &[HostSkillDecl {
            name: HUANLE_MONKEY_KEY,
            behavior: HostBehavior::HuanleMonkeyCard,
        }],
        CardKind::Xuelunyang => &[HostSkillDecl {
            name: HUANLE_XUELUNYANG_KEY,
            behavior: HostBehavior::HuanleXuelunyangCard,
        }],
        CardKind::Langeguaiyi => &[HostSkillDecl {
            name: SWD_LANGEGUAIYI_SKILL_KEY,
            behavior: HostBehavior::SwdLangeguaiyiEquipSkill,
        }],
        CardKind::Shentoumianju => &[HostSkillDecl {
            name: SWD_SHENTOU_SKILL_KEY,
            behavior: HostBehavior::SwdShentouEquipSkill,
        }],
        CardKind::Yufulu => &[HostSkillDecl {
            name: SWD_YUFULU_SKILL_KEY,
            behavior: HostBehavior::SwdYufuluEquipSkill,
        }],
        CardKind::Longlushanfeng => &[HostSkillDecl {
            name: MTG_LONGSHANFENG_KEY,
            behavior: HostBehavior::MtgLongshanfengLand,
        }],
        CardKind::Haidao => &[HostSkillDecl {
            name: MTG_HAIDAO_KEY,
            behavior: HostBehavior::MtgHaidaoLand,
        }],
        CardKind::Shuimomuxue => &[HostSkillDecl {
            name: MTG_SHUIMOMUXUE_KEY,
            behavior: HostBehavior::MtgShuimomuxueLand,
        }],
        CardKind::Feixu => &[HostSkillDecl {
            name: MTG_FEIXU_KEY,
            behavior: HostBehavior::MtgFeixuLand,
        }],
        CardKind::Yixialan => &[
            HostSkillDecl {
                name: MTG_YIXIALAN_KEY,
                behavior: HostBehavior::MtgYixialanLand,
            },
            HostSkillDecl {
                name: MTG_YIXIALAN_RECAST_KEY,
                behavior: HostBehavior::MtgYixialanRecast,
            },
        ],
        CardKind::Shamolvzhou => &[HostSkillDecl {
            name: MTG_SHAMOLVZHOU_KEY,
            behavior: HostBehavior::MtgShamolvzhouLand,
        }],
        CardKind::Linzhongjianta => &[
            HostSkillDecl {
                name: MTG_LINZHONGJIANTA_KEY,
                behavior: HostBehavior::MtgLinzhongjiantaLand,
            },
            HostSkillDecl {
                name: MTG_LINZHONGJIANTA_SKILL_KEY,
                behavior: HostBehavior::MtgLinzhongjiantaStrike,
            },
        ],
        CardKind::Bingheyaosai => &[HostSkillDecl {
            name: MTG_BINGHEYAOSAI_KEY,
            behavior: HostBehavior::MtgBingheyaosaiLand,
        }],
        CardKind::Youlin => &[
            HostSkillDecl {
                name: MTG_YOULIN_KEY,
                behavior: HostBehavior::MtgYoulinLand,
            },
            HostSkillDecl {
                name: MTG_YOULIN_DISCOVER_KEY,
                behavior: HostBehavior::MtgYoulinDiscard,
            },
        ],
        CardKind::Lindixiliu => &[HostSkillDecl {
            name: MTG_LINDIXILIU_KEY,
            behavior: HostBehavior::MtgLindixiliuLand,
        }],
        CardKind::Jiguan => &[
            HostSkillDecl {
                name: SWD_GOUHUNLUO_SUB_KEY,
                behavior: HostBehavior::SwdGouhunluoSub,
            },
            HostSkillDecl {
                name: SWD_FENGYIN_STATE_KEY,
                behavior: HostBehavior::SwdFengyinState,
            },
            HostSkillDecl {
                name: SWD_JIGUANYUAN_GAIN_KEY,
                behavior: HostBehavior::SwdJiguanyuanGain,
            },
            HostSkillDecl {
                name: SWD_JIGUANYAOSHU_STATE_KEY,
                behavior: HostBehavior::SwdJiguanyaoState,
            },
            HostSkillDecl {
                name: SWD_JIGUANYAO_GAIN_KEY,
                behavior: HostBehavior::SwdJiguanyaoGain,
            },
        ],
        CardKind::Lianyaohu => &[
            HostSkillDecl {
                name: "lianyaohu_shouna",
                behavior: HostBehavior::SwdLianyaohuShouna,
            },
            HostSkillDecl {
                name: "lianyaohu_lianhua",
                behavior: HostBehavior::SwdLianyaohuLianhua,
            },
        ],
        CardKind::Shennongding => &[HostSkillDecl {
            name: "shennongding",
            behavior: HostBehavior::SwdShennongdingCard,
        }],
        CardKind::Nvwashi => &[HostSkillDecl {
            name: "nvwashi",
            behavior: HostBehavior::SwdNvwashiGuard,
        }],
        CardKind::Kongdongyin => &[HostSkillDecl {
            name: "kongdongyin",
            behavior: HostBehavior::SwdKongdongyinGuard,
        }],
        CardKind::Guilingzhitao => &[
            HostSkillDecl {
                name: "nigong_accumulate",
                behavior: HostBehavior::SwdNigongAccumulate,
            },
            HostSkillDecl {
                name: "nigong_strike",
                behavior: HostBehavior::SwdNigongStrike,
            },
        ],
        CardKind::Sifeizhenmian => &[HostSkillDecl {
            name: "yiluan",
            behavior: HostBehavior::SwdSifeizhenmianCard,
        }],
        CardKind::Qinglongzhigui => &[HostSkillDecl {
            name: "qinglongzhigui",
            behavior: HostBehavior::SwdQinglongzhiguiHeld,
        }],
        CardKind::Zhuquezhizhang => &[HostSkillDecl {
            name: "zhuquezhizhang",
            behavior: HostBehavior::SwdZhuquezhizhangHeld,
        }],
        CardKind::Baishouzhihu => &[HostSkillDecl {
            name: "baishouzhihu",
            behavior: HostBehavior::SwdBaishouzhihuHeld,
        }],
        CardKind::Xuanwuzhihuang => &[HostSkillDecl {
            name: "xuanwuzhihuang",
            behavior: HostBehavior::SwdXuanwuzhihuangHeld,
        }],
        CardKind::Huanglinzhicong => &[HostSkillDecl {
            name: "huanglinzhicong",
            behavior: HostBehavior::SwdHuanglinzhicongHeld,
        }],
        CardKind::Shencaojie => &[HostSkillDecl {
            name: "shencaojie",
            behavior: HostBehavior::SwdShencaojieGuard,
        }],
        CardKind::Shenmiguo => &[HostSkillDecl {
            name: "shenmiguo_window",
            behavior: HostBehavior::SwdShenmiguoWindow,
        }],
        CardKind::Shuchui => &[HostSkillDecl {
            name: "shuchui",
            behavior: HostBehavior::SwdShuchuiUse,
        }],
        CardKind::Yuchanqian
        | CardKind::Yuchankun
        | CardKind::Yuchanzhen
        | CardKind::Yuchanxun
        | CardKind::Yuchankan
        | CardKind::Yuchanli
        | CardKind::Yuchangen
        | CardKind::Yuchandui => &[HostSkillDecl {
            name: "yuchan_swap",
            behavior: HostBehavior::SwdYuchanSwap,
        }],
        CardKind::Kunlunjingc => &[HostSkillDecl {
            name: "kunlunjingc",
            behavior: HostBehavior::SwdKunlunjingcCard,
        }],
        CardKind::Donghuangzhong => &[HostSkillDecl {
            name: "donghuangzhong",
            behavior: HostBehavior::SwdDonghuangzhongCard,
        }],
        _ => &[],
    }
}

/// 牌种固有的装备技能（引擎按固定键查找）；包数据声明的装备技追加其后。
pub fn card_kind_equip_skills(kind: crate::catalog::CardKind) -> &'static [&'static str] {
    match kind {
        CardKind::Mutoumianju => &[GJ_MUTOU_SKILL_KEY],
        CardKind::Gjyuheng => &[GJ_YUHENG_SKILL_KEY],
        CardKind::Monkey => &[HUANLE_MONKEY_KEY],
        CardKind::Xuelunyang => &[HUANLE_XUELUNYANG_KEY],
        CardKind::Mianju => &[HUANLE_MIANJI_KEY],
        _ => &[],
    }
}

/// 羊皮卷发现类别池（swd 第十三批 v97）：上游 `get.inpile(type)` 按通用
/// type 枚举；原生按目录牌种枚举——gwent/炉石/古剑机关/食物/欢乐等
/// 专用牌种不入池（登记边界）。Tag：0=基本 1=锦囊 2=装备。
pub fn yangpijuan_class(kind: CardKind) -> Option<u8> {
    use CardKind as K;
    match kind {
        K::Sha
        | K::FireSha
        | K::LeiSha
        | K::Shan
        | K::Tao
        | K::Jiu
        | K::Hufu
        | K::Caoyao
        | K::Tianxianjiu => Some(0),
        K::SavageAttack
        | K::ArcheryAttack
        | K::Duel
        | K::Nullify
        | K::Le
        | K::Bing
        | K::ShanDian
        | K::Dismantle
        | K::Snatch
        | K::Chain
        | K::Wuzhong
        | K::Taoyuan
        | K::Wugu
        | K::Jiedao
        | K::WangmeiZhike
        | K::Shuigong
        | K::Fudichouxin
        | K::Chenhuodajie
        | K::HongShui
        | K::Toulianghuanzhu
        | K::Youdishenru
        | K::Geanguanhuo
        | K::Caochuanjiejian
        | K::Guiyoujie
        | K::Zhufangshenshi
        | K::Liuxinghuoyu => Some(1),
        _ if kind.equip_slot().is_some() => Some(2),
        _ => None,
    }
}
