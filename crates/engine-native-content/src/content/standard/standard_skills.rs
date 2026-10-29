use super::*;

pub(super) fn skills() -> Vec<SkillDef> {
    vec![
        SkillDef {
            name: SAN_CAI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(SanCaiBehavior),
        },
        SkillDef {
            name: YI_SHANG,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YiShangBehavior),
        },
        SkillDef {
            name: SUI_DUN,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(SuiDunBehavior),
        },
        SkillDef {
            name: SU_QING,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::First,
                priority: 100,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(SuQingBehavior),
        },
        SkillDef {
            name: SHOU_YI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShouYiBehavior),
        },
        SkillDef {
            name: FAN_KUI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(FanKuiBehavior { _p: () }),
        },
        SkillDef {
            name: JIAN_XIONG,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JianXiongBehavior),
        },
        SkillDef {
            name: JI_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiJiBehavior),
        },
        SkillDef {
            name: GANG_LIE,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(GangLieBehavior),
        },
        SkillDef {
            name: TU_XI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(TuXiBehavior),
        },
        SkillDef {
            name: LUO_YI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(LuoYiBehavior),
        },
        SkillDef {
            name: GUAN_XING,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(GuanXingBehavior),
        },
        // 空城/谦逊为锁定门控：无触发机器，出牌校验侧经 skill_by_name 查询。
        SkillDef {
            name: LIAN_YING,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: vec![
                    EventKind::CardResponded,
                    EventKind::CardsMoved,
                    EventKind::CardsDiscarded,
                ],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LianYingBehavior),
        },
        SkillDef {
            name: KONG_CHENG,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QIAN_XUN,
            // 权限型能力：乐不思蜀目标校验经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: REN_DE,
            // 主动技：出牌阶段可多次发动（上游 enable: phaseUse 无次数
            // 限制）；累计给牌数入回合状态（跨过 2 张回复 1 点）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(RendeBehavior),
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
        },
        SkillDef {
            name: MA_SHU,
            // 权限型能力：computed_distance 距离修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QING_GUO,
            // 权限型能力：杀响应选项并入黑色装备经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QI_CAI,
            // 权限型能力：锦囊距离限制豁免经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JIU_YUAN,
            // 权限型能力：濒死桃回复量修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JI_JIU,
            // 权限型能力：濒死结算桃判定扩面经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: HU_JIA,
            // 权限型能力：出闪响应选项扩面（护驾借闪）经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: LIU_LI,
            // 权限型能力：杀转移询问（ShaResolve 内嵌流离段）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
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
        },
        SkillDef {
            name: TIE_JI,
            // 锁定式判定：ShaResolve 结算内经 skill_by_name 查询来源持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: PAO_XIAO,
            // 权限型能力：出牌阶段杀限检查经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QI_XI,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: WU_SHUANG,
            // 修改型能力：杀结算/决斗机器经 skill_by_name 查询来源持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JI_ZHI,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiZhiBehavior),
        },
        SkillDef {
            name: KE_JI,
            // 条件型手牌上限修正：判定阶段（弃牌）流程经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: GUO_SE,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: WU_SHENG,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: LONG_DAN,
            // 权限型能力：出牌/出闪结算经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BI_YUE,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(BiYueBehavior),
        },
        SkillDef {
            name: LUO_SHEN,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LuoShenBehavior),
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
        },
        SkillDef {
            name: YING_ZI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YingZiBehavior),
        },
        SkillDef {
            name: XIAO_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::Equipped,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoJiBehavior),
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
        },
        SkillDef {
            name: GUI_CAI,
            // 权限型能力：判定流程经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: GUI_DAO,
            // 权限型能力：判定流程经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: TIAN_DU,
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JU_SHOU,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JushouBehavior),
        },
        SkillDef {
            name: XIAO_GUO,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoGuoBehavior),
        },
        SkillDef {
            name: KUANG_FU,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(KuangFuBehavior),
        },
        SkillDef {
            name: YAO_WU,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YaoWuBehavior),
        },
        SkillDef {
            name: SHU_SHEN,
            trigger: Some(TriggerMeta {
                event: EventKind::Recover,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShuShenBehavior),
        },
        SkillDef {
            name: SHEN_ZHI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShenZhiBehavior),
        },
        SkillDef {
            name: XUN_XUN,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XunXunBehavior),
        },
        SkillDef {
            name: WANG_XI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangXiBehavior),
        },
        SkillDef {
            name: YI_CONG,
            // 权限型能力：computed_distance 动态距离修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JI_JIE,
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(JiJieBehavior),
        },
        SkillDef {
            name: JI_YUAN,
            trigger: Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiYuanBehavior),
        },
        SkillDef {
            name: WANG_ZUN,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangZunBehavior),
        },
        SkillDef {
            name: WANG_ZUN_MARK,
            // 权限型能力：弃牌阶段手牌上限 -1（妄尊授予的回合级印记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: TONG_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(TongJiBehavior),
        },
        SkillDef {
            name: ZHANGBA,
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(ZhangbaBehavior),
        },
    ]
}
