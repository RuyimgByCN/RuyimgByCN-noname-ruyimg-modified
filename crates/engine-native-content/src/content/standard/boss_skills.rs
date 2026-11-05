use super::*;
mod boss_host;
mod boss_underworld;

pub(super) fn skills() -> Vec<SkillDef> {
    let mut skills = vec![
        SkillDef {
            name: BOSS_XIONGSHOU,
            // 权限型能力三面：杀伤条件臂（kernel emit_sha_damage）、
            // computed_distance 距离 -1（state）、翻面取消（commit 入口
            // 直读跳过——死亡取消型同款），均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_WUZANG,
            // 权限型能力两面：摸牌阶段额外量与手牌上限基数归零，均在
            // kernel 阶段臂经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XIANGDE,
            // 伤害前置触发：来源有武器对持有者伤害 +1（机器一步直改
            // 载荷，YiShang 同款窗口）
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBegin3,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiangDeBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_YINZEI,
            // 权限型两面：激活门控与伤害后弃牌在 commit_damage/
            // commit_lose_hp 尾经 skill_by_name 查询（激活=SkillInstance
            // .enabled 直写，初始装配即未激活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHUE,
            // 权限型单面：global damageEnd 双摸在 commit_damage 尾经
            // skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_FUTAI,
            // 准备阶段触发（A2 S12：过滤收紧 Phase::Prepare；原锚判定
            // 阶段开始，神智同口径）：全体受伤者按座位序各回 1（机器一
            // 步，无挂起；锁定=forced 无确认）
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(FutaiBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_YANDU,
            // 权限型两面：激活门控（commit_damage 尾，隐贼先例同构）与
            // 他人结束阶段获得牌（kernel Phase::End 臂）均经 skill_by_name
            // 查询；装配即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_TANYU,
            // 权限型两面：跳过弃牌阶段（kernel Phase::Discard 臂直查）
            // 与结束阶段全场最多手牌失 1 体（Phase::End 臂）均经
            // skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_CANGMU,
            // 权限型单面：摸牌基数=存活角色数（kernel Draw 臂，无脏同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JICAI,
            // 权限型两面：激活门控（commit_damage 尾，延妒并列）与回复
            // 后双摸（commit_recover 新臂）均经 skill_by_name 查询；装配
            // 即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_NITAI,
            // 权限型两面：回合内受伤防止与回合外火伤 +1 均在
            // commit_damage 内联经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_LUANCHANG,
            // 两面：准备阶段（A2 S13：过滤收紧 Phase::Prepare；原锚判定
            // 阶段开始）视为用【南蛮入侵】（触发机一步 use_virtual_card）；
            // 结束阶段视为用【万箭齐发】（kernel Phase::End 臂查询式）；
            // 激活门控同隐贼/延妒/积财先例（装配即未启用）
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LuanChangBehavior),
            original_source: None,
            presentation_binding: None,
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
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_CUIDUI,
            // 权限型单面：伤害后授毒（commit_damage 尾经 skill_by_name
            // 查询）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHONGDU,
            // 中毒（运行时授技）：回合开始判定（commit_phase Judge begin
            // 段内联 draw_judge_card，铁骑式确定性判定）——非红桃伤 1、
            // 非黑桃失去此技（实例移除）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_YAOSHOU,
            // 权限型单面：computed_distance 距离 -2（state 距离链，凶兽
            // 同款）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JIUSHOU,
            // 权限型三面：手牌上限恒 9（state hand_limit）、跳过摸牌与
            // 出牌/结束补至 9（kernel 阶段臂）均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_DUQU,
            // 权限型单面：受击者给来源加蛇毒标记（commit_damage 尾经
            // skill_by_name 查询；PlayerState.shedu 承载）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ECHOU,
            // 权限型两面：激活门控（commit_damage 尾延妒并列）与他人桃
            // /酒获标记（commit_card_used）均经 skill_by_name 查询；装配
            // 即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_SHEDU,
            // 蛇毒标记结算（运行时触发技，A2 S14）：上游 player
            // phaseBegin＝自己回合开始（原锚 Judge begin）持有标记>0 →
            // 「弃 X 张牌或失去 X 体力」（挂起选择；取消/空选=失 X）→
            // 标记-1；X=当前标记数
            trigger: Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShedUBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_BINGXIAN,
            // 权限型单面：他人回合未出杀 → 结束阶段虚拟杀（kernel
            // Phase::End 臂，乱常万箭同款 use_virtual_card）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XUSHI,
            // 权限型两面：出牌阶段结束自翻面（commit_phase Play begin:
            // false 臂）与翻回正面全场伤 1（commit_turn_over_before 尾）
            // 均经 skill_by_name 查询；激活门控同隐贼先例（装配即未激活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JUYUAN,
            // 权限型两面：回合结束体力记录（kernel Phase::End 臂）与杀
            // 目标 mod（engine max_targets）均经 skill_by_name 查询；
            // juyuan_storage=PlayerState 跨回合持久字段
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZHAOHUO,
            // 两面：造成伤害视为火（DamageBefore 触发机一步直改 nature
            // ——相德同款窗口）；免疫火伤（commit_damage 入口直读——拟
            // 态回合内免伤同位）
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBegin1,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhaoHuoBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_HONGLIANX,
            // 权限型两面：红色不计上限+弃牌阶段红牌不可弃（hand_limit
            // 与 Discard 臂查询）；回合开始获红牌/对至多 3 名各 1 火伤
            // （Judge begin 段内联，确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_YANYU,
            // 权限型两面：激活门控（commit_damage 尾第八并列臂）与判定
            // 红色火伤（kernel Judge begin 段内联，回合计数至多 3）均经
            // skill_by_name 查询；装配即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_FENGDONG,
            // 权限型两面：回合开始他人获封印（Judge begin 段内联，神智
            // 锚）与回合结束全清（Phase::End 臂）经 skill_by_name 查询；
            // PlayCard 手牌校验（engine 校验段）禁用封印者手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XUNYOU,
            // 权限型单面：他人回合开始获得座位序首名有手牌者一张手牌
            //（Judge begin 段内联，确定性近似；装备牌自动使用=直接装上）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
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
            original_source: None,
            presentation_binding: None,
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
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_RELEIJI,
            // 查询型单面：杀/万箭闪消费位内联臂（releiji_on_shan）经
            // skill_by_name 查询；青木链见 BOSS_QINGMU2/2X（v292）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_KUANGXIAO,
            // 查询型两面：杀无距离（engine sha_range_free 链追加）与目
            // 标数无限（engine kuangxiao_inf 臂，天弦同构）均经 skill_
            // by_name 查询；友方禁指面登记差异（关系网边界）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
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
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhaoHuoBehavior),
            original_source: None,
            presentation_binding: None,
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
            original_source: None,
            presentation_binding: None,
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
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XINGQIU,
            // 查询型单面：Play begin 臂（交替开关+横置敌方，direct 直写
            // linked+日志，索链甲同款）；授 boss_jiding 面登记差异未入库
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_BUCHUN,
            // 主动技两线（ActivateSkill 直调 activate——复活线/恢复线按
            // 场面分派；间隔门控读 buchun_used_round）；快照变体
            // BossBuchun（archive tag 66/机哈希 67）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(BuchunBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_HUIHUO,
            // 查询型两面：杀上限 +1（engine 限次臂 once_used 额度，元宝
            // 肉同款）与死亡反噬（kernel 死亡结算移除前捕获+PlayerDied
            // 后敌方 3 点火伤）均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_FURAN,
            // 查询型单面：濒死救援选项扩展（is_tao_like 臂——敌方红色
            // 牌可当桃救焰灵）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_XINGXIA,
            // 主动技（ActivateSkill 直调 activate——损焰灵线/追击线按
            // 挂起阶段推进；间隔门控读 xingxia_used_round）；快照变体
            // BossXingxia（archive tag 67/机哈希 68）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(XingxiaBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JUEHONG,
            // 查询型单面：准备阶段臂（Judge begin 段——封冻同位；装备全
            // 弃撤技能+入弃堆，随机弃→首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_QINGZHU,
            // 查询型两面：跳弃牌（Discard 臂——贪欲同位）+杀禁用（engine
            // 校验臂——获急叮解除）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JIAZU,
            // 查询型单面：准备阶段臂（相邻敌方坐骑全弃）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JIDING,
            // 查询型单面：damageEnd 臂（友方受伤→对敌方来源虚拟雷杀，
            // LeiSha kind 载体；蓐收加成尾面登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZAOYI,
            // 查询型三面：死亡触发（DeathMachine 移除后捕获+授技）、
            // targetEnabled 双门控（engine 校验）、造异异变授予（运行时
            // skills.insert，中毒/蛇毒先例）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZAOYI_HP,
            // 查询型单面（运行时授予）：回合开始敌方最低体力全体失去全
            // 部体力（Judge begin 臂）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_ZIRUN,
            // 查询型单面：准备阶段臂（Judge begin 段——全体摸牌：有装备
            // 2/无装备 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_NIANRUI,
            // 查询型单面：Draw 臂固定摸 2 后补摸 2（藏目同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_MENGTAI,
            // 查询型多面：准备置位/摸牌清位/出牌清位/结束补摸 3/弃牌闸门
            // （TurnState.mengtai_* 双旗标承载）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_NBIANSHEN,
            // 查询型单面：Judge begin 轮换臂（assembly 重挂+hp 保留；
            // versus 替补换将先例同构；确定性互斥轮转登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_RENXING,
            // 查询型双臂：damageEnd（commit_damage 段）+recoverEnd
            // （commit_recover_skills 头）——非当前回合摸 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_RUIZHI,
            // 查询型单面：准备阶段臂（global 修剪：其余角色保留 1 手+1
            // 装，弃其余——保留首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_SHOUYI,
            // 查询型单面：sha_range_free 链+顺手距离链追加（全牌无距离，
            // 狂笑/神恩同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_DANSHU,
            // 查询型单面：CardsDiscarded 臂（失去手牌后非当前回合且已受
            // 伤判定红回 1）——夜叉批条目复用同名 def
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_JINGJUE,
            // 查询型单面：继承胆术（同内容共用 commit 臂按名双查）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_MODAO,
            // 查询型单面：Judge begin 臂自摸 2（藏目/年瑞同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_MOJIAN,
            // 查询型单面：Play begin 臂虚拟万箭（乱常 End 臂同款
            // use_virtual_card；异侧存活全目标）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_YUSHOU,
            // 查询型单面：Play begin 臂虚拟南蛮（魔剑同款 use_virtual_card）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
        SkillDef {
            name: BOSS_MOYANY,
            // 查询型单面：CardsDiscarded 臂（非当前回合判定红→2 火伤打
            // 座位序首个其他存活——确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
            original_source: None,
            presentation_binding: None,
        },
    ];
    skills.extend(boss_underworld::skills());
    skills.extend(boss_host::skills());
    skills
}
