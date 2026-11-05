use super::*;
use crate::vocabulary::generals::BuiltinOriginalKey;

const fn orig(pack: &'static str, raw_symbol: &'static str) -> Option<BuiltinOriginalKey> {
    Some(BuiltinOriginalKey { pack, raw_symbol })
}

pub(super) fn skills() -> Vec<SkillDef> {
    vec![
        SkillDef {
            name: SAN_CAI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(SanCaiBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: YI_SHANG,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YiShangBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: SUI_DUN,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(SuiDunBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: SU_QING,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::First,
                priority: 100,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(SuQingBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: SHOU_YI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShouYiBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: FAN_KUI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(FanKuiBehavior { _p: () }),
            original_source: orig("standard", "fankui"),
            presentation_binding: None,
        },
        SkillDef {
            name: JIAN_XIONG,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JianXiongBehavior),
            original_source: orig("standard", "jianxiong"),
            presentation_binding: None,
        },
        SkillDef {
            name: JI_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiJiBehavior),
            original_source: orig("standard", "yiji"),
            presentation_binding: None,
        },
        SkillDef {
            name: GANG_LIE,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(GangLieBehavior),
            original_source: orig("standard", "ganglie"),
            presentation_binding: None,
        },
        SkillDef {
            name: TU_XI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(TuXiBehavior),
            original_source: orig("standard", "tuxi"),
            presentation_binding: None,
        },
        SkillDef {
            name: LUO_YI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(LuoYiBehavior),
            original_source: orig("standard", "luoyi"),
            presentation_binding: None,
        },
        SkillDef {
            name: GUAN_XING,
            // A2 S03：上游 player phaseZhunbeiBegin——过滤在 skills/
            // guan_xing.rs 收紧 Phase::Prepare（原锚判定阶段开始）。
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(GuanXingBehavior),
            original_source: orig("standard", "guanxing"),
            presentation_binding: None,
        },
        // 空城/谦逊为锁定门控：无触发机器，出牌校验侧经 skill_by_name 查询。
        SkillDef {
            name: LIAN_YING,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![
                    EventKind::CardResponded,
                    EventKind::CardsMoved,
                    EventKind::CardsDiscarded,
                ],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LianYingBehavior),
            original_source: orig("standard", "lianying"),
            presentation_binding: None,
        },
        SkillDef {
            name: KONG_CHENG,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "kongcheng"),
            presentation_binding: None,
        },
        SkillDef {
            name: QIAN_XUN,
            // 权限型能力：乐不思蜀目标校验经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "qianxun"),
            presentation_binding: None,
        },
        SkillDef {
            name: REN_DE,
            // 主动技：出牌阶段可多次发动（上游 enable: phaseUse 无次数
            // 限制）；累计给牌数入回合状态（跨过 2 张回复 1 点）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(RendeBehavior),
            original_source: orig("standard", "rende"),
            presentation_binding: None,
        },
        SkillDef {
            name: JI_JIANG,
            // 主公技主动化：激将上游入口是 chooseToUse/chooseToRespond
            // （需要使用/打出杀时发动），不绑定出牌阶段——目录声明例外
            // （once_per_turn 为原生逐回合限一次的登记约束）。
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                active_outside_play: true,
            },
            behavior: Box::new(JijiangBehavior),
            original_source: orig("standard", "jijiang"),
            presentation_binding: None,
        },
        SkillDef {
            name: MA_SHU,
            // 权限型能力：computed_distance 距离修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "mashu"),
            presentation_binding: None,
        },
        SkillDef {
            name: QING_GUO,
            // 权限型能力：杀响应选项并入黑色装备经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "qingguo"),
            presentation_binding: None,
        },
        SkillDef {
            name: QI_CAI,
            // 权限型能力：锦囊距离限制豁免经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "qicai"),
            presentation_binding: None,
        },
        SkillDef {
            name: JIU_YUAN,
            // 权限型能力：濒死桃回复量修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "jiuyuan"),
            presentation_binding: None,
        },
        SkillDef {
            name: JI_JIU,
            // 权限型能力：濒死结算桃判定扩面经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "jijiu"),
            presentation_binding: None,
        },
        SkillDef {
            name: HU_JIA,
            // 权限型能力：出闪响应选项扩面（护驾借闪）经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "hujia"),
            presentation_binding: None,
        },
        SkillDef {
            name: LIU_LI,
            // 权限型能力：杀转移询问（ShaResolve 内嵌流离段）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "liuli"),
            presentation_binding: None,
        },
        SkillDef {
            name: JIE_YIN,
            // 主动技：出牌阶段限一次（activate 形态）
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(JieYinBehavior),
            original_source: orig("standard", "jieyin"),
            presentation_binding: None,
        },
        SkillDef {
            name: FAN_JIAN,
            // 主动技：出牌阶段限一次（activate 形态）
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(FanJianBehavior),
            original_source: orig("standard", "fanjian"),
            presentation_binding: None,
        },
        SkillDef {
            name: LI_JIAN,
            // 主动技：出牌阶段限一次（activate 形态，内嵌决斗直结算）
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(LiJianBehavior),
            original_source: orig("standard", "lijian"),
            presentation_binding: None,
        },
        SkillDef {
            name: TIE_JI,
            // 锁定式判定：ShaResolve 结算内经 skill_by_name 查询来源持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "tieji"),
            presentation_binding: None,
        },
        SkillDef {
            name: PAO_XIAO,
            // 权限型能力：出牌阶段杀限检查经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "paoxiao"),
            presentation_binding: None,
        },
        SkillDef {
            name: QI_XI,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "qixi"),
            presentation_binding: None,
        },
        SkillDef {
            name: WU_SHUANG,
            // 修改型能力：杀结算/决斗机器经 skill_by_name 查询来源持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "wushuang"),
            presentation_binding: None,
        },
        SkillDef {
            name: JI_ZHI,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiZhiBehavior),
            original_source: orig("standard", "jizhi"),
            presentation_binding: None,
        },
        SkillDef {
            name: KE_JI,
            // 条件型手牌上限修正：判定阶段（弃牌）流程经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "keji"),
            presentation_binding: None,
        },
        SkillDef {
            name: GUO_SE,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "guose"),
            presentation_binding: None,
        },
        SkillDef {
            name: WU_SHENG,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "wusheng"),
            presentation_binding: None,
        },
        SkillDef {
            name: LONG_DAN,
            // 权限型能力：出牌/出闪结算经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "longdan"),
            presentation_binding: None,
        },
        SkillDef {
            name: BI_YUE,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(BiYueBehavior),
            original_source: orig("standard", "biyue"),
            presentation_binding: None,
        },
        SkillDef {
            name: LUO_SHEN,
            // A2 S05：上游 player phaseZhunbeiBegin——过滤在 skills/
            // luo_shen.rs 收紧 Phase::Prepare（原锚判定阶段开始）。
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LuoShenBehavior),
            original_source: orig("standard", "luoshen"),
            presentation_binding: None,
        },
        SkillDef {
            name: KU_ROU,
            // 主动技：经 GameAction::ActivateSkill 发动（每回合限一次）
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(KuRouBehavior),
            original_source: orig("standard", "kurou"),
            presentation_binding: None,
        },
        SkillDef {
            name: ZHI_HENG,
            // 主动技：经 GameAction::ActivateSkill 发动（每回合限一次）
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(ZhiHengBehavior),
            original_source: orig("standard", "zhiheng"),
            presentation_binding: None,
        },
        SkillDef {
            name: YING_ZI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YingZiBehavior),
            original_source: orig("standard", "yingzi"),
            presentation_binding: None,
        },
        SkillDef {
            name: XIAO_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::Equipped,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoJiBehavior),
            original_source: orig("standard", "xiaoji"),
            presentation_binding: None,
        },
        SkillDef {
            name: QING_NANG,
            // 主动技：出牌阶段限一次
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(QingNangBehavior),
            original_source: orig("standard", "qingnang"),
            presentation_binding: None,
        },
        SkillDef {
            name: GUI_CAI,
            // 权限型能力：判定流程经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "guicai"),
            presentation_binding: None,
        },
        SkillDef {
            name: GUI_DAO,
            // 权限型能力：判定流程经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("shenhua", "guidao"),
            presentation_binding: None,
        },
        SkillDef {
            name: TIAN_DU,
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("standard", "tiandu"),
            presentation_binding: None,
        },
        SkillDef {
            name: JU_SHOU,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JushouBehavior),
            original_source: orig("shenhua", "jushou"),
            presentation_binding: None,
        },
        SkillDef {
            name: XIAO_GUO,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoGuoBehavior),
            original_source: orig("standard", "stdxiaoguo"),
            presentation_binding: None,
        },
        SkillDef {
            name: KUANG_FU,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(KuangFuBehavior),
            original_source: orig("standard", "stdkuangfu"),
            presentation_binding: None,
        },
        SkillDef {
            name: YAO_WU,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YaoWuBehavior),
            original_source: orig("standard", "yaowu"),
            presentation_binding: None,
        },
        SkillDef {
            name: SHU_SHEN,
            trigger: Some(TriggerMeta {
                event: EventKind::Recover,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShuShenBehavior),
            original_source: orig("standard", "stdshushen"),
            presentation_binding: None,
        },
        SkillDef {
            name: SHEN_ZHI,
            // A2 S09：上游 player phaseZhunbeiBegin——过滤在 skills/
            // shen_zhi.rs 收紧 Phase::Prepare（原锚判定阶段开始）。
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShenZhiBehavior),
            original_source: orig("sp", "shenzhi"),
            presentation_binding: None,
        },
        SkillDef {
            name: XUN_XUN,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XunXunBehavior),
            original_source: orig("refresh", "xunxun"),
            presentation_binding: None,
        },
        SkillDef {
            name: WANG_XI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangXiBehavior),
            original_source: orig("refresh", "wangxi"),
            presentation_binding: None,
        },
        SkillDef {
            name: YI_CONG,
            // 权限型能力：computed_distance 动态距离修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: orig("refresh", "reyicong"),
            presentation_binding: None,
        },
        SkillDef {
            name: JI_JIE,
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(JiJieBehavior),
            original_source: orig("standard", "xinfu_jijie"),
            presentation_binding: None,
        },
        SkillDef {
            name: JI_YUAN,
            trigger: Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiYuanBehavior),
            original_source: orig("standard", "xinfu_jiyuan"),
            presentation_binding: None,
        },
        SkillDef {
            name: WANG_ZUN,
            // A2 S11：上游 global phaseZhunbeiBegin＝他人回合——过滤在
            // skills/wang_zun.rs 收紧 Phase::Prepare＋player≠owner（原锚
            // 他人判定阶段开始）。
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangZunBehavior),
            original_source: orig("standard", "rewangzun"),
            presentation_binding: None,
        },
        SkillDef {
            name: WANG_ZUN_MARK,
            // 权限型能力：弃牌阶段手牌上限 -1（妄尊授予的回合级印记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: TONG_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(TongJiBehavior),
            original_source: orig("standard", "retongji"),
            presentation_binding: None,
        },
        SkillDef {
            name: ZHANGBA,
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(ZhangbaBehavior),
            original_source: None,
            presentation_binding: None,
        },
    ]
}
