use super::*;

pub(super) fn skills() -> Vec<SkillDef> {
    vec![
        SkillDef {
            name: BOSS_XIONGSHOU,
            // 权限型能力三面：杀伤条件臂（kernel emit_sha_damage）、
            // computed_distance 距离 -1（state）、翻面取消（commit 入口
            // 直读跳过——死亡取消型同款），均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WUZANG,
            // 权限型能力两面：摸牌阶段额外量与手牌上限基数归零，均在
            // kernel 阶段臂经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIANGDE,
            // 伤害前置触发：来源有武器对持有者伤害 +1（机器一步直改
            // 载荷，YiShang 同款窗口）
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiangDeBehavior),
        },
        SkillDef {
            name: BOSS_YINZEI,
            // 权限型两面：激活门控与伤害后弃牌在 commit_damage/
            // commit_lose_hp 尾经 skill_by_name 查询（激活=SkillInstance
            // .enabled 直写，初始装配即未激活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHUE,
            // 权限型单面：global damageEnd 双摸在 commit_damage 尾经
            // skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FUTAI,
            // 准备阶段触发（判定阶段开始锚点，神智同口径）：全体受伤
            // 者按座位序各回 1（机器一步，无挂起；锁定=forced 无确认）
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(FutaiBehavior),
        },
        SkillDef {
            name: BOSS_YANDU,
            // 权限型两面：激活门控（commit_damage 尾，隐贼先例同构）与
            // 他人结束阶段获得牌（kernel Phase::End 臂）均经 skill_by_name
            // 查询；装配即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TANYU,
            // 权限型两面：跳过弃牌阶段（kernel Phase::Discard 臂直查）
            // 与结束阶段全场最多手牌失 1 体（Phase::End 臂）均经
            // skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_CANGMU,
            // 权限型单面：摸牌基数=存活角色数（kernel Draw 臂，无脏同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JICAI,
            // 权限型两面：激活门控（commit_damage 尾，延妒并列）与回复
            // 后双摸（commit_recover 新臂）均经 skill_by_name 查询；装配
            // 即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_NITAI,
            // 权限型两面：回合内受伤防止与回合外火伤 +1 均在
            // commit_damage 内联经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LUANCHANG,
            // 两面：准备阶段（判定阶段开始锚点）视为用【南蛮入侵】（触
            // 发机一步 use_virtual_card）；结束阶段视为用【万箭齐发】
            // （kernel Phase::End 臂查询式）；激活门控同隐贼/延妒/积财
            // 先例（装配即未启用）
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LuanChangBehavior),
        },
        SkillDef {
            name: BOSS_MINWAN,
            // 权限型三面：伤害后锁定集（commit_damage 尾）、每用牌摸 1
            //（commit_card_used 尾）、PlayCard 目标校验（engine 校验段）
            // 均经 skill_by_name 查询；锁定集=TurnState.minwan_targets
            //（回合级，更替自然清零）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_CUIDUI,
            // 权限型单面：伤害后授毒（commit_damage 尾经 skill_by_name
            // 查询）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHONGDU,
            // 中毒（运行时授技）：回合开始判定（commit_phase Judge begin
            // 段内联 draw_judge_card，铁骑式确定性判定）——非红桃伤 1、
            // 非黑桃失去此技（实例移除）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YAOSHOU,
            // 权限型单面：computed_distance 距离 -2（state 距离链，凶兽
            // 同款）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIUSHOU,
            // 权限型三面：手牌上限恒 9（state hand_limit）、跳过摸牌与
            // 出牌/结束补至 9（kernel 阶段臂）均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_DUQU,
            // 权限型单面：受击者给来源加蛇毒标记（commit_damage 尾经
            // skill_by_name 查询；PlayerState.shedu 承载）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ECHOU,
            // 权限型两面：激活门控（commit_damage 尾延妒并列）与他人桃
            // /酒获标记（commit_card_used）均经 skill_by_name 查询；装配
            // 即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHEDU,
            // 蛇毒标记结算（运行时触发技）：Judge begin 持有标记>0 →
            // 「弃 X 张牌或失去 X 体力」（挂起选择；取消/空选=失 X）→
            // 标记-1；X=当前标记数
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShedUBehavior),
        },
        SkillDef {
            name: BOSS_BINGXIAN,
            // 权限型单面：他人回合未出杀 → 结束阶段虚拟杀（kernel
            // Phase::End 臂，乱常万箭同款 use_virtual_card）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XUSHI,
            // 权限型两面：出牌阶段结束自翻面（commit_phase Play begin:
            // false 臂）与翻回正面全场伤 1（commit_turn_over_before 尾）
            // 均经 skill_by_name 查询；激活门控同隐贼先例（装配即未激活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JUYUAN,
            // 权限型两面：回合结束体力记录（kernel Phase::End 臂）与杀
            // 目标 mod（engine max_targets）均经 skill_by_name 查询；
            // juyuan_storage=PlayerState 跨回合持久字段
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHAOHUO,
            // 两面：造成伤害视为火（DamageBefore 触发机一步直改 nature
            // ——相德同款窗口）；免疫火伤（commit_damage 入口直读——拟
            // 态回合内免伤同位）
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhaoHuoBehavior),
        },
        SkillDef {
            name: BOSS_HONGLIANX,
            // 权限型两面：红色不计上限+弃牌阶段红牌不可弃（hand_limit
            // 与 Discard 臂查询）；回合开始获红牌/对至多 3 名各 1 火伤
            // （Judge begin 段内联，确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YANYU,
            // 权限型两面：激活门控（commit_damage 尾第八并列臂）与判定
            // 红色火伤（kernel Judge begin 段内联，回合计数至多 3）均经
            // skill_by_name 查询；装配即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FENGDONG,
            // 权限型两面：回合开始他人获封印（Judge begin 段内联，神智
            // 锚）与回合结束全清（Phase::End 臂）经 skill_by_name 查询；
            // PlayCard 手牌校验（engine 校验段）禁用封印者手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XUNYOU,
            // 权限型单面：他人回合开始获得座位序首名有手牌者一张手牌
            //（Judge begin 段内联，确定性近似；装备牌自动使用=直接装上）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SIPU,
            // 权限型两面：激活门控（commit_damage 尾第九并列臂；装配即
            // 未激活）与响应/救禁（respond_shan、dying_save 选项排除）
            // 均经 skill_by_name 查询；使用面由回合模型天然承载；无懈等
            // 其余响应窗未纳入禁令（登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHENYI,
            // 查询型两面：翻面取消（commit_turn_over_before 臂——凶兽
            // 同位，仅正→背取消）与判定阶段延时锦囊无效化（apply_
            // judgement 逐臂条件）均经 skill_by_name 查询；洪荒之力禁
            // 用交互未落（登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RELEIJI,
            // 查询型单面：杀/万箭闪消费位内联臂（releiji_on_shan）经
            // skill_by_name 查询；青木链见 BOSS_QINGMU2/2X（v292）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_KUANGXIAO,
            // 查询型两面：杀无距离（engine sha_range_free 链追加）与目
            // 标数无限（engine kuangxiao_inf 臂，天弦同构）均经 skill_
            // by_name 查询；友方禁指面登记差异（关系网边界）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FENTIAN,
            // 火改写面=ZhaoHuoBehavior 同构复用（数据less nature 改写，
            // 快照变体共用 BossZhaohuo——机器不携带技能身份，恢复路径按
            // 技能表重挂）；其余为查询面（engine 红牌免次数/免距离、
            // kernel 红杀跳八卦）；wuxie 禁面登记差异未挂钩
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhaoHuoBehavior),
        },
        SkillDef {
            name: BOSS_LINGQU,
            // 查询型两面：受击摸 1+计数（commit_damage damageEnd 段）与
            // 大于 1 点伤害归零（commit_damage 入口臂）均经 skill_by_name
            // 查询；手牌上限 +X（hand_limit 查询+PlayerState.lingqu_count
            // 新字段）；玄麟=关卡流程面登记不落
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHENEN,
            // 全局光环查询型三面：engine sha/顺手距离链 boss 方豁免、
            // hand_limit 挑战者 +1（state）、Draw 臂挑战者 +1（kernel）
            // 均经 skill_by_name 查询+is_boss_side 判定；光环激活=场上
            // 存在持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XINGQIU,
            // 查询型单面：Play begin 臂（交替开关+横置敌方，direct 直写
            // linked+日志，索链甲同款）；授 boss_jiding 面登记差异未入库
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_BUCHUN,
            // 主动技两线（ActivateSkill 直调 activate——复活线/恢复线按
            // 场面分派；间隔门控读 buchun_used_round）；快照变体
            // BossBuchun（archive tag 66/机哈希 67）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(BuchunBehavior),
        },
        SkillDef {
            name: BOSS_HUIHUO,
            // 查询型两面：杀上限 +1（engine 限次臂 once_used 额度，元宝
            // 肉同款）与死亡反噬（kernel 死亡结算移除前捕获+PlayerDied
            // 后敌方 3 点火伤）均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FURAN,
            // 查询型单面：濒死救援选项扩展（is_tao_like 臂——敌方红色
            // 牌可当桃救焰灵）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XINGXIA,
            // 主动技（ActivateSkill 直调 activate——损焰灵线/追击线按
            // 挂起阶段推进；间隔门控读 xingxia_used_round）；快照变体
            // BossXingxia（archive tag 67/机哈希 68）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(XingxiaBehavior),
        },
        SkillDef {
            name: BOSS_JUEHONG,
            // 查询型单面：准备阶段臂（Judge begin 段——封冻同位；装备全
            // 弃撤技能+入弃堆，随机弃→首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QINGZHU,
            // 查询型两面：跳弃牌（Discard 臂——贪欲同位）+杀禁用（engine
            // 校验臂——获急叮解除）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIAZU,
            // 查询型单面：准备阶段臂（相邻敌方坐骑全弃）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIDING,
            // 查询型单面：damageEnd 臂（友方受伤→对敌方来源虚拟雷杀，
            // LeiSha kind 载体；蓐收加成尾面登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZAOYI,
            // 查询型三面：死亡触发（DeathMachine 移除后捕获+授技）、
            // targetEnabled 双门控（engine 校验）、造异异变授予（运行时
            // skills.insert，中毒/蛇毒先例）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZAOYI_HP,
            // 查询型单面（运行时授予）：回合开始敌方最低体力全体失去全
            // 部体力（Judge begin 臂）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZIRUN,
            // 查询型单面：准备阶段臂（Judge begin 段——全体摸牌：有装备
            // 2/无装备 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_NIANRUI,
            // 查询型单面：Draw 臂固定摸 2 后补摸 2（藏目同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MENGTAI,
            // 查询型多面：准备置位/摸牌清位/出牌清位/结束补摸 3/弃牌闸门
            // （TurnState.mengtai_* 双旗标承载）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_NBIANSHEN,
            // 查询型单面：Judge begin 轮换臂（assembly 重挂+hp 保留；
            // versus 替补换将先例同构；确定性互斥轮转登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RENXING,
            // 查询型双臂：damageEnd（commit_damage 段）+recoverEnd
            // （commit_recover_skills 头）——非当前回合摸 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RUIZHI,
            // 查询型单面：准备阶段臂（global 修剪：其余角色保留 1 手+1
            // 装，弃其余——保留首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHOUYI,
            // 查询型单面：sha_range_free 链+顺手距离链追加（全牌无距离，
            // 狂笑/神恩同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_DANSHU,
            // 查询型单面：CardsDiscarded 臂（失去手牌后非当前回合且已受
            // 伤判定红回 1）——夜叉批条目复用同名 def
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JINGJUE,
            // 查询型单面：继承胆术（同内容共用 commit 臂按名双查）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MODAO,
            // 查询型单面：Judge begin 臂自摸 2（藏目/年瑞同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MOJIAN,
            // 查询型单面：Play begin 臂虚拟万箭（乱常 End 臂同款
            // use_virtual_card；异侧存活全目标）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YUSHOU,
            // 查询型单面：Play begin 臂虚拟南蛮（魔剑同款 use_virtual_card）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MOYANY,
            // 查询型单面：CardsDiscarded 臂（非当前回合判定红→2 火伤打
            // 座位序首个其他存活——确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_BAOLIAN,
            // 查询型单面：End 臂摸 2
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QIANGZHENG,
            // 查询型单面：End 臂——每名其他有手牌角色随机一张→首张确定
            // 性近似登记，获得至手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZUIJIU,
            // 查询型单面：commit_damage 臂——source sha 伤害 +1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIAOSHOU,
            // 查询型单面：End 臂——座位序首个 hp≥self 其他存活受 3 伤害
            // （目标自选可放弃→恒发动确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_GUIJI,
            // 查询型单面：Judge begin 臂弃自己判定区首张（随机→首张近似）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TAIPING,
            // 查询型单面：Draw 臂 +2（年瑞同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SUOMING,
            // 查询型单面：End 臂——全部未链接异侧存活横置（链接机直写+
            // Linked 日志，[1,num] 自选→全部确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIXING,
            // 查询型单面：Judge begin 臂——座位序首个已链接异侧受 1 雷伤
            // +持有者回 1（目标自选→首个已链接确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LIANYU,
            // 查询型单面：End 臂——全部其他存活各 1 火伤（无询问直伤）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JUECE,
            // 查询型单面：CardsDiscarded 臂——自己回合内其他角色失去最后
            // 一手牌（弃后空手）→ 1 伤害（chooseBool AI→恒发动近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIUCHI,
            // 查询型单面：PlayCard kind 改写臂（黑桃手牌→酒，viewAs 转化
            // 武圣/奇袭同族）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_GUIMOU,
            // 查询型单面：End begin 臂——座位序首个其他未混乱存活角色
            // boss_mad 置位（其回合收束解除；随机→首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YUANCE,
            // 查询型单面：Damage commit 尾臂——来源≠受伤者且双方存活 →
            // 判定（选项一恒定：黑→受伤者失 1 / 红→来源失 1；chooseControl
            // AI 态度面→恒选项一确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QIZUO,
            // CardUsed AfterTriggers 触发：锦囊复用一步机（QizuoBehavior
            // 门=无懈/铁索排除+延时同名去重+原目标存活；载体槽守卫递归）
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(QizuoBehavior),
        },
        SkillDef {
            name: JUE_QING,
            // 查询型单面：commit_damage 入口直读臂（急冻前——上游
            // damageBefore 层）——来源伤害改为目标失去等量体力
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WUXIN,
            // 查询型双面：commit_damage 入口直读臂（受伤→失 1）+
            // PlayCard 校验臂（乐/兵拒——延时禁指定他人）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHANGSHIX,
            // 查询型双面：四 commit 臂补牌（CardsDiscarded/Damage/LoseHp/
            // Recover——手牌<4 补至 4）+ End begin 臂（hp>1 全体各失 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIAOXIONG,
            // CardUsed AfterTriggers 触发：副本入手一步机（XiaoxiongBehavior
            // 门=他人使用+基本/锦囊白名单；End begin 臂=子技未用牌伤害）
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoxiongBehavior),
        },
        SkillDef {
            name: BOSS_ZHANGWU,
            // 查询型单面：Damage commit 尾臂（damageEnd 族）——持有者弃 1
            // 张+来源弃 1 张或受 1 伤害（双方恒弃首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LINGBO,
            // CardResponded+CardUsed 触发：打出/使用闪后摸 2 一步机
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LingboBehavior),
        },
        SkillDef {
            name: BOSS_JIAOXIA,
            // 查询型单面：CardUsed commit 臂——成为红色牌目标后摸 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FANGHUA,
            // 查询型单面：End begin 臂——存在翻面角色则全部各失 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TASHUI,
            // CardResponded+CardUsed 触发：打出/使用黑牌后翻面一步机
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(TashuiBehavior),
        },
        SkillDef {
            name: BOSS_HUOSHEN,
            // 查询型单面：commit_damage 火伤分支直读臂（招祸同位扩展——
            // 免疫火伤+回复 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZYHONGLIAN,
            // 查询型单面：End begin 臂——摸 2+全部其他存活各 1 火伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIANYIN,
            // 查询型单面：CardsDiscarded 臂（非当前回合）——判定红→座位
            // 序首个其他存活失 1（chooseTarget AI→首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TIANDAO,
            // 查询型单面：改判机 rewriters 扩展（hes 任意牌改判+获得原判
            // 定牌）——JudgePhase 机器内联，无独立触发
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZJLEIJI,
            // CardResponded+CardUsed 触发：打出/使用【闪】后一步机（首个
            // 其他存活判定黑→1 雷伤+摸 1）
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZjLeijiBehavior),
        },
        SkillDef {
            name: BOSS_JIDIAN,
            // 查询型单面：Damage commit 尾臂——距受伤者 ≤1 的首个其他存
            // 活判定黑→1 雷伤（链挡旗标不自我递归=登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHIWANG,
            // 查询型单面：CardsMoved commit 臂——其他角色获得牌（非摸牌
            // 阶段摸牌，CardsDrawn 天然隔离）→其弃首张手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_GONGZHENG,
            // 查询型单面：Judge begin 臂——判定区有牌则弃首张（鬼计同款）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XUECHI,
            // 查询型单面：End begin 臂——座位序首个其他存活失 2（随机→
            // 首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_HEISHENG,
            // Death 触发（死亡前置窗口=遗言面）：全体其他横置一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(HeishengBehavior),
        },
        SkillDef {
            name: BOSS_SHENGFU,
            // 查询型单面：End begin 臂——首个有坐骑的其他存活弃其一
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LEIZHOU,
            // 查询型单面：Judge begin 臂——首个其他存活 1 雷伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LEIFU,
            // 查询型单面：End begin 臂——首个其他存活横置态切换
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LEIZHU,
            // Death 触发（遗言面，黑绳同位）：座位序逐发雷伤
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LeizhuBehavior),
        },
        SkillDef {
            name: BOSS_FUDU,
            // 查询型单面：CardUsed commit 臂——他人用桃随机其他失 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_KUJIU,
            // 查询型单面：Judge begin 臂（global 扫持有者）——其他角色
            // 准备阶段失 1+视为用酒
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RENAO,
            // Death 触发（遗言面，雷诛同位）：首发其他 3 火伤
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(RenaoBehavior),
        },
        SkillDef {
            name: BOSS_LUNHUI,
            // 查询型单面：Judge begin 臂——hp≤2 与首个 hp>2 其他交换体力
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WANGSHENG,
            // 查询型单面：Play begin 臂——恒定虚拟南蛮（全部其他存活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZLFANSHI,
            // 查询型单面：Damage commit 尾臂——terra 旗交替+反噬 1 伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHIYOU,
            // 查询型单面：弃牌机 Done 前臂——弃牌阶段弃牌花色互异获得
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WANGHUN,
            // Death 触发（遗言面）：清往事旗+前两个其他各禁用首个技
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WanghunBehavior),
        },
        SkillDef {
            name: BOSS_WANGSHI,
            // 查询型单面：Judge begin global 臂（恒定禁基本牌）+
            // PlayCard 拒绝位+回合结束全清
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TIEMIAN,
            // 查询型单面：sha 机仁王块扩展（防具槽空视为仁王盾）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHADAO,
            // 查询型单面：sha 机青釭穿透链扩展（来源持有者即穿透）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHUXIN,
            // Death 触发（遗言面）：最低体力其他存活受 2 伤一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhuxinBehavior),
        },
        SkillDef {
            name: BOSS_SUOZU,
            // 查询型单面：Prepare begin 臂——全部其他存活横置
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ABI,
            // 查询型单面：Damage commit 尾臂——来源受 1 火伤（恒定火）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_PINGDENG,
            // Death 触发（遗言面）：最高体力两波火伤（2+1）两步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(PingdengBehavior),
        },
        SkillDef {
            name: BOSS_REMEN,
            // 查询型单面：PlayCard 拒绝位（plain 杀）+AOE 机窗口前跳过
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHIFEN,
            // 查询型单面：Prepare begin 臂——获得座位序首个其他首张手
            // 牌+其受 1 火伤（无来源）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_HUOXING,
            // Death 触发（遗言面）：全部其他存活逐座火伤机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(HuoxingBehavior),
        },
        SkillDef {
            name: BOSS_BUFO,
            // 查询型单面×2：伤害入口减 1（终值>1）+准备阶段距离 1 火伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WULIANG,
            // 查询型单面×2：结束阶段摸 2+准备阶段体力<3 回复至 3
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_DITING,
            // 查询型单面：computed_distance 双向链（from -1 / to +1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WEIMU,
            // 查询型单面：黑锦囊 PlayCard 拒绝位+黑色 AOE 窗口前跳过
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_BINGFENG,
            // Death 触发（遗言面）：翻面来源一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(BingfengBehavior),
        },
        SkillDef {
            name: BOSS_PANGUAN,
            // 查询型单面：延时锦囊 PlayCard 拒绝位
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JUHUN,
            // 查询型单面：End begin 臂——座位序首个其他翻面（恒定翻面）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WANGXIANG,
            // Death 触发（遗言面）：全场装备全弃一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangxiangBehavior),
        },
        SkillDef {
            name: BOSS_TIANYU,
            // 查询型单面：End begin 臂——自横置先解除+全场未横置横置
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QIWU,
            // 查询型单面：CardUsed commit 臂——梅花牌使用后受伤回复 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        // D4/B1 四象隐藏链青木打样（HostBehavior tag 411–414）
        crate::content::host::host_skill_def(
            BOSS_QINGMU2.to_string(),
            crate::content::host::HostBehavior::BossQingmu2,
        ),
        crate::content::host::host_skill_def(
            BOSS_QINGMU2X.to_string(),
            crate::content::host::HostBehavior::BossQingmu2x,
        ),
        crate::content::host::host_skill_def(
            BOSS_QINGMU3.to_string(),
            crate::content::host::HostBehavior::BossQingmu3,
        ),
        crate::content::host::host_skill_def(
            BOSS_QINGMU3X.to_string(),
            crate::content::host::HostBehavior::BossQingmu3x,
        ),
        // D4/B2 白芒/赤炎/玄麟 2/2x（HostBehavior tag 415–420）
        crate::content::host::host_skill_def(
            BOSS_BAIMANG2.to_string(),
            crate::content::host::HostBehavior::BossBaimang2,
        ),
        crate::content::host::host_skill_def(
            BOSS_BAIMANG2X.to_string(),
            crate::content::host::HostBehavior::BossBaimang2x,
        ),
        crate::content::host::host_skill_def(
            BOSS_CHIYAN2.to_string(),
            crate::content::host::HostBehavior::BossChiyan2,
        ),
        crate::content::host::host_skill_def(
            BOSS_CHIYAN2X.to_string(),
            crate::content::host::HostBehavior::BossChiyan2x,
        ),
        crate::content::host::host_skill_def(
            BOSS_XUANLIN2.to_string(),
            crate::content::host::HostBehavior::BossXuanlin2,
        ),
        crate::content::host::host_skill_def(
            BOSS_XUANLIN2X.to_string(),
            crate::content::host::HostBehavior::BossXuanlin2x,
        ),
        // D4/B3 白芒/赤炎/玄麟 3/3x（HostBehavior tag 421–426）
        crate::content::host::host_skill_def(
            BOSS_BAIMANG3.to_string(),
            crate::content::host::HostBehavior::BossBaimang3,
        ),
        crate::content::host::host_skill_def(
            BOSS_BAIMANG3X.to_string(),
            crate::content::host::HostBehavior::BossBaimang3x,
        ),
        crate::content::host::host_skill_def(
            BOSS_CHIYAN3.to_string(),
            crate::content::host::HostBehavior::BossChiyan3,
        ),
        crate::content::host::host_skill_def(
            BOSS_CHIYAN3X.to_string(),
            crate::content::host::HostBehavior::BossChiyan3x,
        ),
        crate::content::host::host_skill_def(
            BOSS_XUANLIN3.to_string(),
            crate::content::host::HostBehavior::BossXuanlin3,
        ),
        crate::content::host::host_skill_def(
            BOSS_XUANLIN3X.to_string(),
            crate::content::host::HostBehavior::BossXuanlin3x,
        ),
        crate::content::host::host_skill_def(
            BOSS_BIANSHEN3.to_string(),
            crate::content::host::HostBehavior::BossBianshen3,
        ),
        crate::content::host::host_skill_def(
            BOSS_BIANSHEN3X.to_string(),
            crate::content::host::HostBehavior::BossBianshen3x,
        ),
        crate::content::host::host_skill_def(
            BOSS_BIANSHEN4.to_string(),
            crate::content::host::HostBehavior::BossBianshen4,
        ),
        crate::content::host::host_skill_def(
            BOSS_BIANSHEN4X.to_string(),
            crate::content::host::HostBehavior::BossBianshen4x,
        ),
        crate::content::host::host_skill_def(
            BOSS_HUANREN.to_string(),
            crate::content::host::HostBehavior::BossHuanren,
        ),
        crate::content::host::host_skill_def(
            BOSS_HUANREN_DIE.to_string(),
            crate::content::host::HostBehavior::BossHuanrenDie,
        ),
        crate::content::host::host_skill_def(
            BOSS_QINGYI.to_string(),
            crate::content::host::HostBehavior::BossQingyi,
        ),
        crate::content::host::host_skill_def(
            BOSS_CHIYI.to_string(),
            crate::content::host::HostBehavior::BossChiyi,
        ),
        crate::content::host::host_skill_def(
            BOSS_CHIYI2.to_string(),
            crate::content::host::HostBehavior::BossChiyiMark,
        ),
        crate::content::host::host_skill_def(
            BOSS_BAIYI.to_string(),
            crate::content::host::HostBehavior::BossBaiyi,
        ),
        crate::content::host::host_skill_def(
            BOSS_BAIYI_THUNDER.to_string(),
            crate::content::host::HostBehavior::BossBaiyiThunder,
        ),
        // T11 Boss 庞统族批 118（规则 v306）：涅槃/浴火/神威——尾部追加
        // 涅槃（濒死面可选触发 + 出牌阶段主动面；trigger 由行为元数据给出）
        crate::content::host::host_skill_def(
            BOSS_NIEPAN.to_string(),
            crate::content::host::HostBehavior::Niepan,
        ),
        // 浴火（涅槃后链式授予，无触发时机）
        crate::content::host::host_skill_def(
            BOSS_YUHOU.to_string(),
            crate::content::host::HostBehavior::Yuhuo,
        ),
        // 神威（锁定查询型：摸牌 +N / 手牌上限 +N，kernel 臂）
        crate::content::host::host_skill_def(
            BOSS_SHENWEI.to_string(),
            crate::content::host::HostBehavior::Shenwei,
        ),
    ]
}
