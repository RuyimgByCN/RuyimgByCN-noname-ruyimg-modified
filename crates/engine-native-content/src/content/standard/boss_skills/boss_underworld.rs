use super::*;

pub(super) fn skills() -> Vec<SkillDef> {
    vec![
        SkillDef {
            name: BOSS_BAOLIAN,
            // 查询型单面：End 臂摸 2
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_QIANGZHENG,
            // 查询型单面：End 臂——每名其他有手牌角色随机一张→首张确定
            // 性近似登记，获得至手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZUIJIU,
            // 查询型单面：commit_damage 臂——source sha 伤害 +1
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBegin1,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(native_damage::NativeDamageBehavior {
                name: BOSS_ZUIJIU,
                effects: &[crate::event::NativeDamageEffect::Zuijiu],
            }),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XIAOSHOU,
            // 查询型单面：End 臂——座位序首个 hp≥self 其他存活受 3 伤害
            // （目标自选可放弃→恒发动确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_GUIJI,
            // 查询型单面：Judge begin 臂弃自己判定区首张（随机→首张近似）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_TAIPING,
            // 查询型单面：Draw 臂 +2（年瑞同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_SUOMING,
            // 查询型单面：End 臂——全部未链接异侧存活横置（链接机直写+
            // Linked 日志，[1,num] 自选→全部确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XIXING,
            // 查询型单面：Judge begin 臂——座位序首个已链接异侧受 1 雷伤
            // +持有者回 1（目标自选→首个已链接确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_LIANYU,
            // 查询型单面：End 臂——全部其他存活各 1 火伤（无询问直伤）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JUECE,
            // 查询型单面：CardsDiscarded 臂——自己回合内其他角色失去最后
            // 一手牌（弃后空手）→ 1 伤害（chooseBool AI→恒发动近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JIUCHI,
            // 查询型单面：PlayCard kind 改写臂（黑桃手牌→酒，viewAs 转化
            // 武圣/奇袭同族）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_GUIMOU,
            // 查询型单面：End begin 臂——座位序首个其他未混乱存活角色
            // boss_mad 置位（其回合收束解除；随机→首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_YUANCE,
            // 查询型单面：Damage commit 尾臂——来源≠受伤者且双方存活 →
            // 判定（选项一恒定：黑→受伤者失 1 / 红→来源失 1；chooseControl
            // AI 态度面→恒选项一确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_QIZUO,
            // CardUsed AfterTriggers 触发：锦囊复用一步机（QizuoBehavior
            // 门=无懈/铁索排除+延时同名去重+原目标存活；载体槽守卫递归）
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(QizuoBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: JUE_QING,
            // 查询型单面：commit_damage 入口直读臂（急冻前——上游
            // damageBefore 层）——来源伤害改为目标失去等量体力
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(native_damage::NativeDamageBehavior {
                name: JUE_QING,
                effects: &[crate::event::NativeDamageEffect::Jueqing],
            }),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WUXIN,
            // 查询型双面：commit_damage 入口直读臂（受伤→失 1）+
            // PlayCard 校验臂（乐/兵拒——延时禁指定他人）
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 10,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(native_damage::NativeDamageBehavior {
                name: BOSS_WUXIN,
                effects: &[crate::event::NativeDamageEffect::Wuxin],
            }),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_SHANGSHIX,
            // 查询型双面：四 commit 臂补牌（CardsDiscarded/Damage/LoseHp/
            // Recover——手牌<4 补至 4）+ End begin 臂（hp>1 全体各失 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XIAOXIONG,
            // CardUsed AfterTriggers 触发：副本入手一步机（XiaoxiongBehavior
            // 门=他人使用+基本/锦囊白名单；End begin 臂=子技未用牌伤害）
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoxiongBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHANGWU,
            // 查询型单面：Damage commit 尾臂（damageEnd 族）——持有者弃 1
            // 张+来源弃 1 张或受 1 伤害（双方恒弃首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_LINGBO,
            // CardResponded+CardUsed 触发：打出/使用闪后摸 2 一步机
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LingboBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JIAOXIA,
            // 查询型单面：CardUsed commit 臂——成为红色牌目标后摸 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_FANGHUA,
            // 查询型单面：End begin 臂——存在翻面角色则全部各失 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_TASHUI,
            // CardResponded+CardUsed 触发：打出/使用黑牌后翻面一步机
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(TashuiBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_HUOSHEN,
            // 查询型单面：commit_damage 火伤分支直读臂（招祸同位扩展——
            // 免疫火伤+回复 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZYHONGLIAN,
            // 查询型单面：End begin 臂——摸 2+全部其他存活各 1 火伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XIANYIN,
            // 查询型单面：CardsDiscarded 臂（非当前回合）——判定红→座位
            // 序首个其他存活失 1（chooseTarget AI→首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_TIANDAO,
            // 查询型单面：改判机 rewriters 扩展（hes 任意牌改判+获得原判
            // 定牌）——JudgePhase 机器内联，无独立触发
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZJLEIJI,
            // CardResponded+CardUsed 触发：打出/使用【闪】后一步机（首个
            // 其他存活判定黑→1 雷伤+摸 1）
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZjLeijiBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JIDIAN,
            // 查询型单面：Damage commit 尾臂——距受伤者 ≤1 的首个其他存
            // 活判定黑→1 雷伤（链挡旗标不自我递归=登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHIWANG,
            // 查询型单面：CardsMoved commit 臂——其他角色获得牌（非摸牌
            // 阶段摸牌，CardsDrawn 天然隔离）→其弃首张手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_GONGZHENG,
            // 查询型单面：Judge begin 臂——判定区有牌则弃首张（鬼计同款）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XUECHI,
            // 查询型单面：End begin 臂——座位序首个其他存活失 2（随机→
            // 首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_HEISHENG,
            // Death 触发（死亡前置窗口=遗言面）：全体其他横置一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(HeishengBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_SHENGFU,
            // 查询型单面：End begin 臂——首个有坐骑的其他存活弃其一
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_LEIZHOU,
            // 查询型单面：Judge begin 臂——首个其他存活 1 雷伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_LEIFU,
            // 查询型单面：End begin 臂——首个其他存活横置态切换
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_LEIZHU,
            // Death 触发（遗言面，黑绳同位）：座位序逐发雷伤
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LeizhuBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_FUDU,
            // 查询型单面：CardUsed commit 臂——他人用桃随机其他失 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_KUJIU,
            // 查询型单面：Judge begin 臂（global 扫持有者）——其他角色
            // 准备阶段失 1+视为用酒
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_RENAO,
            // Death 触发（遗言面，雷诛同位）：首发其他 3 火伤
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(RenaoBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_LUNHUI,
            // 查询型单面：Judge begin 臂——hp≤2 与首个 hp>2 其他交换体力
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WANGSHENG,
            // 查询型单面：Play begin 臂——恒定虚拟南蛮（全部其他存活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZLFANSHI,
            // 查询型单面：Damage commit 尾臂——terra 旗交替+反噬 1 伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_SHIYOU,
            // 查询型单面：弃牌机 Done 前臂——弃牌阶段弃牌花色互异获得
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WANGHUN,
            // Death 触发（遗言面）：清往事旗+前两个其他各禁用首个技
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WanghunBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WANGSHI,
            // 查询型单面：Judge begin global 臂（恒定禁基本牌）+
            // PlayCard 拒绝位+回合结束全清
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_TIEMIAN,
            // 查询型单面：sha 机仁王块扩展（防具槽空视为仁王盾）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHADAO,
            // 查询型单面：sha 机青釭穿透链扩展（来源持有者即穿透）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHUXIN,
            // Death 触发（遗言面）：最低体力其他存活受 2 伤一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhuxinBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_SUOZU,
            // 查询型单面：Prepare begin 臂——全部其他存活横置
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ABI,
            // 查询型单面：Damage commit 尾臂——来源受 1 火伤（恒定火）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_PINGDENG,
            // Death 触发（遗言面）：最高体力两波火伤（2+1）两步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(PingdengBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_REMEN,
            // 查询型单面：PlayCard 拒绝位（plain 杀）+AOE 机窗口前跳过
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHIFEN,
            // 查询型单面：Prepare begin 臂——获得座位序首个其他首张手
            // 牌+其受 1 火伤（无来源）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_HUOXING,
            // Death 触发（遗言面）：全部其他存活逐座火伤机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(HuoxingBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_BUFO,
            // 查询型单面×2：伤害入口减 1（终值>1）+准备阶段距离 1 火伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WULIANG,
            // 查询型单面×2：结束阶段摸 2+准备阶段体力<3 回复至 3
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_DITING,
            // 查询型单面：computed_distance 双向链（from -1 / to +1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WEIMU,
            // 查询型单面：黑锦囊 PlayCard 拒绝位+黑色 AOE 窗口前跳过
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_BINGFENG,
            // Death 触发（遗言面）：翻面来源一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(BingfengBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_PANGUAN,
            // 查询型单面：延时锦囊 PlayCard 拒绝位
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JUHUN,
            // 查询型单面：End begin 臂——座位序首个其他翻面（恒定翻面）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WANGXIANG,
            // Death 触发（遗言面）：全场装备全弃一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangxiangBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_TIANYU,
            // 查询型单面：End begin 臂——自横置先解除+全场未横置横置
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_QIWU,
            // 查询型单面：CardUsed commit 臂——梅花牌使用后受伤回复 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
    ]
}
