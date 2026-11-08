//! 牌种伴随技能和发现类别的不可变描述。

use crate::catalog::CardKind;
use super::host::*;

/// 牌种固有的装备技能（引擎按固定键查找）；包数据声明的装备技追加其后。

pub fn card_kind_host_skills(kind: crate::catalog::CardKind) -> &'static [HostSkillDecl] {
    match kind {
        CardKind::Jinlianzhu => &[HostSkillDecl {
            name: GJ_JINLIANZHU_KEY,
            behavior: HostBehavior::JinlianzhuCancel,
            presentation_binding: None,
        }],
        CardKind::Tanhuadong => &[HostSkillDecl {
            name: GJ_FOOD_TANHUADONG_KEY,
            behavior: HostBehavior::GujianFoodTanhuadong,
            presentation_binding: None,
        }],
        CardKind::Yougeng => &[HostSkillDecl {
            name: GJ_FOOD_YOUGENG_KEY,
            behavior: HostBehavior::GujianFoodYougeng,
            presentation_binding: None,
        }],
        CardKind::Qingtuan => &[HostSkillDecl {
            name: GJ_FOOD_QINGTUAN_KEY,
            behavior: HostBehavior::GujianFoodQingtuan,
            presentation_binding: None,
        }],
        CardKind::Yuanbaorou => &[HostSkillDecl {
            name: GJ_FOOD_YUANBAOROU_KEY,
            behavior: HostBehavior::GujianFoodYuanbaorou,
            presentation_binding: None,
        }],
        CardKind::Gudonggeng => &[HostSkillDecl {
            name: GJ_FOOD_GUDONGGENG_KEY,
            behavior: HostBehavior::GujianFoodGudonggeng,
            presentation_binding: None,
        }],
        CardKind::Mapodoufu => &[HostSkillDecl {
            name: GJ_FOOD_MAPODOUFU_KEY,
            behavior: HostBehavior::GujianFoodMapodoufu,
            presentation_binding: None,
        }],
        CardKind::Chunbing => &[HostSkillDecl {
            name: GJ_FOOD_CHUNBING_KEY,
            behavior: HostBehavior::GujianFoodChunbing,
            presentation_binding: None,
        }],
        CardKind::Liyutang => &[HostSkillDecl {
            name: GJ_FOOD_LIYUTANG_KEY,
            behavior: HostBehavior::GujianFoodLiyutang,
            presentation_binding: None,
        }],
        CardKind::Molicha => &[HostSkillDecl {
            name: GJ_FOOD_MOLICHA_KEY,
            behavior: HostBehavior::GujianFoodMolicha,
            presentation_binding: None,
        }],
        CardKind::Xiajiao => &[HostSkillDecl {
            name: GJ_FOOD_XIAJIAO_KEY,
            behavior: HostBehavior::GujianFoodXiajiao,
            presentation_binding: None,
        }],
        CardKind::Mizhilianou => &[HostSkillDecl {
            name: GJ_FOOD_MIZHLIANOU_KEY,
            behavior: HostBehavior::GujianFoodMizhilianou,
            presentation_binding: None,
        }],
        CardKind::Luyugeng => &[HostSkillDecl {
            name: GJ_FOOD_LUYUGENG_KEY,
            behavior: HostBehavior::GujianFoodLuyugeng,
            presentation_binding: None,
        }],
        CardKind::Ziyangdan => &[HostSkillDecl {
            name: GJ_FOOD_ZIYANGDAN_KEY,
            behavior: HostBehavior::GujianFoodZiyangdan,
            presentation_binding: None,
        }],
        CardKind::Yunvyuanshen => &[HostSkillDecl {
            name: GJ_YUNVYUANSHEN_LISTENER_KEY,
            behavior: HostBehavior::GujianYunvyuanshen,
            presentation_binding: None,
        }],
        CardKind::Shihuifen => &[HostSkillDecl {
            name: GJ_FOOD_SHIHUIFEN_BAN_KEY,
            behavior: HostBehavior::GujianFoodShihuifenBan,
            presentation_binding: None,
        }],
        CardKind::Mutoumianju => &[HostSkillDecl {
            name: GJ_MUTOU_SKILL_KEY,
            behavior: HostBehavior::GujianMutouEquipSkill,
            presentation_binding: None,
        }],
        CardKind::Gjyuheng => &[HostSkillDecl {
            name: GJ_YUHENG_SKILL_KEY,
            behavior: HostBehavior::GujianYuhengEquipSkill,
            presentation_binding: None,
        }],
        CardKind::Qinpendayu => &[HostSkillDecl {
            name: GW_QINPEN_STATE_KEY,
            behavior: HostBehavior::GwentQinpenState,
            presentation_binding: None,
        }],
        CardKind::Birinongwu => &[HostSkillDecl {
            name: GW_BIRIN_STATE_KEY,
            behavior: HostBehavior::GwentBirinState,
            presentation_binding: None,
        }],
        CardKind::Ciguhanshuang => &[HostSkillDecl {
            name: GW_CIGU_STATE_KEY,
            behavior: HostBehavior::GwentCiguState,
            presentation_binding: None,
        }],
        CardKind::Zumoshoukao => &[HostSkillDecl {
            name: GW_FENGYIN_STATE_KEY,
            behavior: HostBehavior::GwentFengyinState,
            presentation_binding: None,
        }],
        CardKind::Kunenfayin => &[HostSkillDecl {
            name: GW_KUNEN_STATE_KEY,
            behavior: HostBehavior::GwentKunenState,
            presentation_binding: None,
        }],
        CardKind::Mianju => &[HostSkillDecl {
            name: HUANLE_MIANJI_KEY,
            behavior: HostBehavior::HuanleMianjuCard,
            presentation_binding: None,
        }],
        CardKind::Monkey => &[HostSkillDecl {
            name: HUANLE_MONKEY_KEY,
            behavior: HostBehavior::HuanleMonkeyCard,
            presentation_binding: None,
        }],
        CardKind::Xuelunyang => &[HostSkillDecl {
            name: HUANLE_XUELUNYANG_KEY,
            behavior: HostBehavior::HuanleXuelunyangCard,
            presentation_binding: None,
        }],
        CardKind::Langeguaiyi => &[HostSkillDecl {
            name: SWD_LANGEGUAIYI_SKILL_KEY,
            behavior: HostBehavior::SwdLangeguaiyiEquipSkill,
            presentation_binding: None,
        }],
        CardKind::Shentoumianju => &[HostSkillDecl {
            name: SWD_SHENTOU_SKILL_KEY,
            behavior: HostBehavior::SwdShentouEquipSkill,
            presentation_binding: None,
        }],
        CardKind::Yufulu => &[HostSkillDecl {
            name: SWD_YUFULU_SKILL_KEY,
            behavior: HostBehavior::SwdYufuluEquipSkill,
            presentation_binding: None,
        }],
        CardKind::Longlushanfeng => &[HostSkillDecl {
            name: MTG_LONGSHANFENG_KEY,
            behavior: HostBehavior::MtgLongshanfengLand,
            presentation_binding: None,
        }],
        CardKind::Haidao => &[HostSkillDecl {
            name: MTG_HAIDAO_KEY,
            behavior: HostBehavior::MtgHaidaoLand,
            presentation_binding: None,
        }],
        CardKind::Shuimomuxue => &[HostSkillDecl {
            name: MTG_SHUIMOMUXUE_KEY,
            behavior: HostBehavior::MtgShuimomuxueLand,
            presentation_binding: None,
        }],
        CardKind::Feixu => &[HostSkillDecl {
            name: MTG_FEIXU_KEY,
            behavior: HostBehavior::MtgFeixuLand,
            presentation_binding: None,
        }],
        CardKind::Yixialan => &[
            HostSkillDecl {
                name: MTG_YIXIALAN_KEY,
                behavior: HostBehavior::MtgYixialanLand,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: MTG_YIXIALAN_RECAST_KEY,
                behavior: HostBehavior::MtgYixialanRecast,
                presentation_binding: None,
            },
        ],
        CardKind::Shamolvzhou => &[HostSkillDecl {
            name: MTG_SHAMOLVZHOU_KEY,
            behavior: HostBehavior::MtgShamolvzhouLand,
            presentation_binding: None,
        }],
        CardKind::Linzhongjianta => &[
            HostSkillDecl {
                name: MTG_LINZHONGJIANTA_KEY,
                behavior: HostBehavior::MtgLinzhongjiantaLand,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: MTG_LINZHONGJIANTA_SKILL_KEY,
                behavior: HostBehavior::MtgLinzhongjiantaStrike,
                presentation_binding: None,
            },
        ],
        CardKind::Bingheyaosai => &[HostSkillDecl {
            name: MTG_BINGHEYAOSAI_KEY,
            behavior: HostBehavior::MtgBingheyaosaiLand,
            presentation_binding: None,
        }],
        CardKind::Youlin => &[
            HostSkillDecl {
                name: MTG_YOULIN_KEY,
                behavior: HostBehavior::MtgYoulinLand,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: MTG_YOULIN_DISCOVER_KEY,
                behavior: HostBehavior::MtgYoulinDiscard,
                presentation_binding: None,
            },
        ],
        CardKind::Lindixiliu => &[HostSkillDecl {
            name: MTG_LINDIXILIU_KEY,
            behavior: HostBehavior::MtgLindixiliuLand,
            presentation_binding: None,
        }],
        CardKind::Jiguan => &[
            HostSkillDecl {
                name: SWD_GOUHUNLUO_SUB_KEY,
                behavior: HostBehavior::SwdGouhunluoSub,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: SWD_FENGYIN_STATE_KEY,
                behavior: HostBehavior::SwdFengyinState,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: SWD_JIGUANYUAN_GAIN_KEY,
                behavior: HostBehavior::SwdJiguanyuanGain,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: SWD_JIGUANYAOSHU_STATE_KEY,
                behavior: HostBehavior::SwdJiguanyaoState,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: SWD_JIGUANYAO_GAIN_KEY,
                behavior: HostBehavior::SwdJiguanyaoGain,
                presentation_binding: None,
            },
        ],
        CardKind::Lianyaohu => &[
            HostSkillDecl {
                name: "lianyaohu_shouna",
                behavior: HostBehavior::SwdLianyaohuShouna,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: "lianyaohu_lianhua",
                behavior: HostBehavior::SwdLianyaohuLianhua,
                presentation_binding: None,
            },
        ],
        CardKind::Shennongding => &[HostSkillDecl {
            name: "shennongding",
            behavior: HostBehavior::SwdShennongdingCard,
            presentation_binding: None,
        }],
        CardKind::Nvwashi => &[HostSkillDecl {
            name: "nvwashi",
            behavior: HostBehavior::SwdNvwashiGuard,
            presentation_binding: None,
        }],
        CardKind::Kongdongyin => &[HostSkillDecl {
            name: "kongdongyin",
            behavior: HostBehavior::SwdKongdongyinGuard,
            presentation_binding: None,
        }],
        CardKind::Guilingzhitao => &[
            HostSkillDecl {
                name: "nigong_accumulate",
                behavior: HostBehavior::SwdNigongAccumulate,
                presentation_binding: None,
            },
            HostSkillDecl {
                name: "nigong_strike",
                behavior: HostBehavior::SwdNigongStrike,
                presentation_binding: None,
            },
        ],
        CardKind::Sifeizhenmian => &[HostSkillDecl {
            name: "yiluan",
            behavior: HostBehavior::SwdSifeizhenmianCard,
            presentation_binding: None,
        }],
        CardKind::Qinglongzhigui => &[HostSkillDecl {
            name: "qinglongzhigui",
            behavior: HostBehavior::SwdQinglongzhiguiHeld,
            presentation_binding: None,
        }],
        CardKind::Zhuquezhizhang => &[HostSkillDecl {
            name: "zhuquezhizhang",
            behavior: HostBehavior::SwdZhuquezhizhangHeld,
            presentation_binding: None,
        }],
        CardKind::Baishouzhihu => &[HostSkillDecl {
            name: "baishouzhihu",
            behavior: HostBehavior::SwdBaishouzhihuHeld,
            presentation_binding: None,
        }],
        CardKind::Xuanwuzhihuang => &[HostSkillDecl {
            name: "xuanwuzhihuang",
            behavior: HostBehavior::SwdXuanwuzhihuangHeld,
            presentation_binding: None,
        }],
        CardKind::Huanglinzhicong => &[HostSkillDecl {
            name: "huanglinzhicong",
            behavior: HostBehavior::SwdHuanglinzhicongHeld,
            presentation_binding: None,
        }],
        CardKind::Cangchizhibi => &[HostSkillDecl {
            name: "cangchizhibi",
            behavior: HostBehavior::SwdCangchizhibiHeld,
            presentation_binding: None,
        }],
        CardKind::Shencaojie => &[HostSkillDecl {
            name: "shencaojie",
            behavior: HostBehavior::SwdShencaojieGuard,
            presentation_binding: None,
        }],
        CardKind::Shenmiguo => &[HostSkillDecl {
            name: "shenmiguo_window",
            behavior: HostBehavior::SwdShenmiguoWindow,
            presentation_binding: None,
        }],
        CardKind::Shuchui => &[HostSkillDecl {
            name: "shuchui",
            behavior: HostBehavior::SwdShuchuiUse,
            presentation_binding: None,
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
            presentation_binding: None,
        }],
        CardKind::Kunlunjingc => &[HostSkillDecl {
            name: "kunlunjingc",
            behavior: HostBehavior::SwdKunlunjingcCard,
            presentation_binding: None,
        }],
        CardKind::Donghuangzhong => &[HostSkillDecl {
            name: "donghuangzhong",
            behavior: HostBehavior::SwdDonghuangzhongCard,
            presentation_binding: None,
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
