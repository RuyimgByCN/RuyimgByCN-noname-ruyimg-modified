use super::super::*;

impl HostBehavior {
    pub(super) fn trigger_meta_swd(&self) -> Option<TriggerMeta> {
        match self {
            HostBehavior::SwdGTiandao => None,
            // 魂身：自己使用/打出闪（上游 respond/useCard，frequent）
            HostBehavior::SwdHyunshen => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::CardResponded],
            }),
            // 魂身·收束（A2 H25）：上游 player phaseBegin＝自己回合开始
            //（原锚自己判定阶段开始）
            HostBehavior::SwdHyunshen2 => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 待行：自己结束阶段（direct）
            HostBehavior::SwdDaixing => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 待行·清甲（A2 H26）：上游 player phaseBegin＝自己回合开始
            //（原锚自己判定阶段开始）（silent）
            HostBehavior::SwdDaixing2 => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 待行·吸收：自己受伤后（silent）
            HostBehavior::SwdDaixing3 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 聚悉：主动技（无触发时机）
            HostBehavior::SwdJuxi => None,
            // 聚悉·计数：任意角色弃牌后（global）
            HostBehavior::SwdJuxiCount => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 破舟：自己受伤后
            HostBehavior::SwdPozhou => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 破舟·封印：主动技（无触发时机）
            HostBehavior::SwdPozhou2 => None,
            // 陷阱：主动技（无触发时机）
            HostBehavior::SwdSxianjing => None,
            // 陷阱·触发：他人使用牌（目标含自己）
            HostBehavior::SwdSxianjingGain => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 陷阱·取回：自己受伤后
            HostBehavior::SwdSxianjingDamage => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 孤行：主动技（无触发时机）
            HostBehavior::SwdGuxing => None,
            // 孤行·计数：自己以杀造成伤害后
            HostBehavior::SwdGuxing3 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 灵矢：锁定旗标（无触发时机；四条效果内联于距离/杀限次/
            // 直中/摸牌四臂，天弦同款）
            HostBehavior::SwdLingshi => None,
            // 锦鳞：主动技（limited 一次性；无触发时机）
            HostBehavior::SwdJinlin => None,
            // 天书：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::SwdTianshu => None,
            // 诛害：任意角色结束阶段开始（可选）
            HostBehavior::SwdZhuhai => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 林韵：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::SwdLinyun => None,
            // 集控（A2 H28）：上游 player phaseBegin＝自己回合开始 或
            // 自己失去手牌（可用：每回合一次；原锚任意自己阶段开始）
            HostBehavior::SwdJikong => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![EventKind::CardsLost],
            }),
            // 魅惑：自己失去装备牌后（可选）
            HostBehavior::SwdMeihuo => Some(TriggerMeta {
                event: EventKind::CardsLost,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 守印：任意角色濒死（可选）
            HostBehavior::SwdShouyin => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 义山：自己受伤结算后（可选；上游 notemp 直发，原生可选窗）
            HostBehavior::SwdYishan => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 入梦：任意角色出牌阶段开始前（可选）
            HostBehavior::SwdRumeng => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 神焰：伤害前置窗（+1 并记录蔓延目标）+ 伤害结算后（蔓延）
            // 上游 `shenyan` 无 `forced`/`direct`（且 `limited: true`）⇒ 应当**询问**（第 1026 轮）
            HostBehavior::SwdShenyan => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![EventKind::Damage],
            }),
            // 灵武：无触发时机（效果内联于阶段收束）
            HostBehavior::SwdLingwu => None,
            // 锦鳞·退（A2 H30）：上游 player phaseBegin＝持有者回合开始
            //（原锚任意自己阶段开始＝每回合 5→6 次）（silent）
            HostBehavior::SwdJinlin2 => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdShehun => None,
            HostBehavior::SwdNlianji => None,
            // 铁壁：成为杀目标时（CardUsed 帧等价承载，登记差异：上游
            // target shaBefore，原生以 CardUsed+targets 等价承载）
            HostBehavior::SwdTiebi => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 大同（A2 H62）：上游 global phaseEnd＝任意玩家整回合结束
            //（global；frequent 自动无门）
            HostBehavior::SwdDatong => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 承剑：其他角色受伤结算后（global；确认门=optional）
            HostBehavior::SwdChengjian => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 缚斧：伤害前置窗（num 直写）
            HostBehavior::SwdJufu => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 花镜：自己造成伤害结算后（frequent 自动无门）
            HostBehavior::SwdHuajing => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 护盾：自己造成伤害结算后（forced 无门）
            HostBehavior::SwdHudun => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 破星：伤害前置窗（num 直写）
            HostBehavior::SwdPoxing => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 流虹：使用杀（frequent 自动无门）
            HostBehavior::SwdLiuhong => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 墨剑：自己使用杀（可选确认门；上游 shaBegin）
            HostBehavior::SwdMojian => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 道法：其他角色受伤结算后（global；确认门=optional）
            HostBehavior::SwdDaofa => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 玉鳞：受伤前置窗口（可选；弃装备牌免伤）
            HostBehavior::SwdYulin => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 凝霜：成为黑牌目标（可选；上游 useCardToBegin）
            HostBehavior::SwdNingshuang => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 工神：其他角色用基本/锦囊牌（可选；上游 global useCard）
            HostBehavior::SwdGongshen => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 平恤：无触发面（computed_distance 字段消费）
            HostBehavior::SwdPingxu => None,
            // 还灵（A2 H63）：上游 player phaseEnd＝自己整回合结束
            //（可选确认门）
            HostBehavior::SwdHuanling => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 断月：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdDuanyue => None,
            // 屠阵：自己造成伤害结算后（可选；上游 source damageAfter）
            HostBehavior::SwdTuzhen => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 落梅：使用或打出（频率技；上游 useCard+respond 双事件）
            HostBehavior::SwdLuomei => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::CardResponded],
            }),
            // 星点：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXingdian => None,
            // 协律：弃牌阶段结束=End 帧入场（可选；上游 phaseDiscardEnd）
            HostBehavior::SwdXielv => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 雪煌：出牌主动技（ActivateSkill 通道，limited bool 门）
            HostBehavior::SwdXuehuang => None,
            // 还侠：无触发面（PlayCard 转化臂+kernel 跟踪钩承载）
            HostBehavior::SwdHuanxia => None,
            // 破月：无触发面（PlayCard 臂+kernel 计数/闪询问钩承载）
            HostBehavior::SwdPoyue => None,
            // 收纳：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLianyaohuShouna => None,
            // 炼化：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLianyaohuLianhua => None,
            // 神农鼎：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdShennongdingCard => None,
            // 昆仑镜/东皇钟：出牌主动技/阶段末触发（东皇钟 direct forced）
            HostBehavior::SwdKunlunjingcCard => None,
            HostBehavior::SwdDonghuangzhongCard => Some(TriggerMeta {
                // A2 H64：上游 player phaseEnd＝自己整回合结束（direct forced）
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 青龙之圭/黄麟之琮（A2 H32/H33）：上游 player phaseBegin＝
            // 自己回合开始（原锚自己判定阶段开始）forced
            HostBehavior::SwdQinglongzhiguiHeld => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdHuanglinzhicongHeld => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 朱雀之璋：受伤结算后 forced
            HostBehavior::SwdZhuquezhizhangHeld => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 白兽之琥：弃牌阶段结束 direct（蝶舞口径）
            HostBehavior::SwdBaishouzhihuHeld => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 玉蝉换形：自己用牌结算后 silent forced
            HostBehavior::SwdYuchanSwap => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Last,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 疾影：锁定 mod 无触发面（sha_range_free 链查询）
            HostBehavior::SwdJiyingHeld => None,
            // 沐风（A2 H66）：上游 player phaseEnd＝自己整回合结束
            //（frequent 上游→确认近似）
            HostBehavior::SwdMufengHeld => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 望尘：弃牌阶段结束确认门（Discard→End 入场近似）
            HostBehavior::SwdWangchenHeld => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 神草结：锦囊伤害前置窗口（确认门；上游 chooseToUse）
            HostBehavior::SwdShencaojieGuard => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 玄武之璜：自己造成伤害结算后 forced
            HostBehavior::SwdXuanwuzhihuangHeld => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 逆攻积攒：受伤结算后 forced
            HostBehavior::SwdNigongAccumulate => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 逆攻发动：出牌主动技（标记门在 activate）
            HostBehavior::SwdNigongStrike => None,
            // 四非珍面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdSifeizhenmianCard => None,
            // 空洞印：死亡前置窗口（forced；上游 dieBefore cancel 同口径）
            HostBehavior::SwdKongdongyinGuard => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::First,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 女娲石：他人濒死（确认门；上游 global dying priority 6）
            HostBehavior::SwdNvwashiGuard => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 壶境（A2 H35）：上游 player phaseBegin＝自己回合开始（原锚
            // 自己判定阶段开始）forced
            HostBehavior::SwdHujing => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 朱羽：横置者受伤前置窗口（可选；上游 global damageBegin）
            HostBehavior::SwdZhuyu => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 分舵：出牌主动技（ActivateSkill 通道，每回合限一次走 limits）
            HostBehavior::SwdJiefen => None,
            // 神秘果窗口：出牌阶段用牌结算后（确认门=optional——上游
            // chooseToUse 即确认）
            HostBehavior::SwdShenmiguoWindow => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 鼠槌：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdShuchuiUse => None,
            // 连舞：锁定 marker（engine max_targets 链+kernel 红杀 directHit
            // 臂查询），无触发面
            HostBehavior::SwdLianwu => None,
            // 浴尘：回合外用牌/响应结算后（双事件；确认门=optional）
            HostBehavior::SwdYuchen => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![EventKind::CardResponded],
            }),
            // 挟雷：使用/打出杀结算后（双事件；确认门=optional）
            HostBehavior::SwdXielei => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![EventKind::CardResponded],
            }),
            // 对诗：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdDuishi => None,
            // 连打：杀结算完毕后（确认门=optional）
            HostBehavior::SwdLianda => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 隳延：造成伤害结算后（确认门=optional）
            HostBehavior::SwdSuiyan => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 断行：装备牌入装结算后（确认门=optional）
            HostBehavior::SwdDuanxing => Some(TriggerMeta {
                event: EventKind::Equipped,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 破浪：造成伤害结算后（确认门=optional）
            HostBehavior::SwdPolang => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 诱引：其他角色弃牌结算后（frequent 锁定式=非 optional）
            HostBehavior::SwdYouyin => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 湛卢：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdZhanlu => None,
            // 苏生：死亡前置窗口（Timing::First 同空洞印；确认门=
            // optional；每回合限一次走 limits——OptionalGate 消费 once_used）
            HostBehavior::SwdSusheng => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::First,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 相惠：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXianghui => None,
            // 挥琪：受伤结算后（确认门=optional）
            HostBehavior::SwdHuiqi => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 万钧：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdWanjun => None,
            // 森罗：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdSenluo => None,
            // 火扑：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdHuopu => None,
            // 夜影：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdYeying => None,
            // 归影：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdGuiying => None,
            // 凤泽：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdFengze => None,
            // 玲月：锁定（sha 机器内联臂查询），无触发面
            HostBehavior::SwdLingyue => None,
            // 暗莲映：其他/自己弃牌结算后（frequent 锁定式=非 optional）
            HostBehavior::SwdAnlianying => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 疾风：锁定 marker（engine max_targets 链+state computed_distance
            // 链查询），无触发面
            HostBehavior::SwdLjifeng => None,
            // 断义：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdDuanyi => None,
            // 乐跃：任意角色回复结算后（确认门=optional）
            HostBehavior::SwdYinyue => Some(TriggerMeta {
                event: EventKind::Recover,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 灵心（A2 H67）：上游 player phaseEnd＝自己整回合结束
            //（frequent 锁定式=非 optional）
            HostBehavior::SwdLingxin => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 渔人：锁定（sha 机器内联臂查询），无触发面
            HostBehavior::SwdYueren => None,
            // 幻剑：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdHuanjian => None,
            // 虚影：伤害前置窗口（锁定取消+失去体力——Boss 绝情同窗）
            HostBehavior::SwdXuying => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 乖戾：双事件（DamageBefore +1 与 Damage 后弃牌同机分流）
            HostBehavior::SwdGuaili => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::Damage],
            }),
            // 无邪：锁定 marker（engine 延时校验臂查询），无触发面
            HostBehavior::SwdWuxie => None,
            // 观虎：杀伤害结算后（确认门=optional）
            HostBehavior::SwdGuanhu => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 敷衍·护甲面：受伤结算后（确认门=optional）
            HostBehavior::SwdFuyan => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 凌波·恒爱面：闪响应/使用结算后（双事件 frequent=非 optional）
            HostBehavior::SwdHlingbo => Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            // 血裔·Miles 面：伤害前置窗口（锁定取消+失去体力）
            HostBehavior::SwdMilesXueyi => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 鸣凤：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdFengming => None,
            // 小莫雨·山小小面：造成伤害结算后（锁定强制；每回合一次走
            // limits once_per_turn——机器臂消费 inst.once_used）
            HostBehavior::SwdXiaomoyu => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 冰箭：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdBingjian => None,
            // 神鼎·麦星面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdShending => None,
            // 鬼眼：出牌主动技（ActivateSkill 通道，usable:1 走 limits；
            // guiyan2 濒死复活 defer）
            HostBehavior::SwdGuiyan => None,
            // 袭西·凤钰面：自己出牌阶段开始（direct 直发窗口）
            HostBehavior::SwdShangxi => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 花间·剑仙面：出牌阶段结束（direct 直发窗口）
            HostBehavior::SwdHuajian => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 星坠·慕月面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXingzhui => None,
            // 焰羽·少微面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdHuangyu => None,
            // 摄引：出牌主动技（ActivateSkill 通道，无 limits——上游循环面）
            HostBehavior::SwdXianyin => None,
            // 破茧·齐面：自己弃牌后手牌空（锁定 frequent=非 optional）
            HostBehavior::SwdPojian => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 横刀·慧彦面：回合外用牌/响应结算后（双事件；确认门=optional
            // ——上游 check 态度门近似）
            HostBehavior::SwdHwendao => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![EventKind::CardResponded],
            }),
            // 凌锋（A2 H69）：上游 player phaseEnd＝自己整回合结束
            //（frequent 锁定式=非 optional；原钉出牌阶段结束＝错位族随迁）
            HostBehavior::SwdLingfeng => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 天火·秦壬面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdTianhuo => None,
            // 运筹·姬亮面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdYunchou => None,
            // 破晓·张烈面：锁定 marker（viewAs 链+距离链+目标上限链三查询）
            HostBehavior::SwdPoxiao => None,
            // 疾风·司空宇面：锁定 marker（engine 杀距离校验臂查询），无触发面
            HostBehavior::SwdSliufeng => None,
            // 乱击·多鹏面：出牌主动技（viewAs 双卡链+usable:1 走 limits）
            HostBehavior::SwdLuanji => None,
            // 聒噪·疾鹏面：自己以外的任意角色受到伤害结算后（global；
            // 距离与手牌候选在机器步求取；锁定=非 optional）
            HostBehavior::SwdGuozao => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 英姿·刷新面：自己摸牌阶段开始时额外摸 1（上游 phaseDrawBegin2
            // num++ 同口径；手牌上限=体力上限在 state.hand_limit 查询）
            HostBehavior::SwdReyingzi => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 愚钝·福面：锁定 marker（PlayCard 禁锦囊臂+双卡 viewAs 臂查询），
            // 无触发面
            HostBehavior::SwdYudun => None,
            // 占星·肯迪面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdZhanxing => None,
            // 乐学·久悠面：自己回合开始（授技在机器步按当前回合分流——
            // 自己回合授{制衡/集智}、他人回合授{遗计/急救/鬼道/反馈}；
            // 回合结束清候选在 PhaseEnd 面另入 filter 分流）
            // 乐学（A2 H38）：上游 lexue1 player phaseBegin＝回合开始。
            HostBehavior::SwdLexue => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 枕流·临月面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdZhenjiu => None,
            // 枕流触发面（A2 H39）：上游 player phaseBegin＝自己回合开始
            //（原锚自己判定阶段开始；off_court 红牌入手+回 1）
            HostBehavior::SwdZhenjiu2 => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 麻醉·临月面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLmazui => None,
            // 麻醉触发面：伤害前置（off_court 黑牌返还+降 1）
            HostBehavior::SwdLmazui2 => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 激昂·凤天凌面：判定为「成为目标」面——原生经 CardUsed+targets
            // 表达（frequent=非 optional）
            HostBehavior::SwdJiang => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 圣手·淳于恒面：锁定 marker（engine PlayCard viewAs 链查询）
            HostBehavior::SwdShengshou => None,
            // 活弹·舒莞儿面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdHuodan => None,
            // 义绝：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdYijue => None,
            // 荡平：自己造成伤害结算后（origin=source damageAfter 同口径；
            // 每回合一次走 limits）
            HostBehavior::SwdDangping => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 幻形（A2 H40）：上游 player phaseBegin＝自己回合开始（原锚
            // 自己判定阶段开始）direct 非 optional
            HostBehavior::SwdHuanxing => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 幻形·移除面：自己受伤结算后（forced）
            HostBehavior::SwdHuanxing2 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 命符·姜子牙面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdMingfu => None,
            // 仙匠·车芸面：使用牌指定唯一目标后（CardUsed；frequent 面=
            // direct optional 确认门）
            HostBehavior::SwdCyxianjiang => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 父技只授予 sha/use，不授予 damage；loseEnd-use 尚待接线。
            HostBehavior::SwdXuanying => None,
            // 玄影伤害子技：自己受伤结算后（已横置且来源存活可反击）
            HostBehavior::SwdXuanyingDamage => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 烈刃：自己杀造成伤害结算后（每触发与目标拼点，optional）
            HostBehavior::SwdLieren => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 极谋·姬良面：自己受伤结算后（回合外门在 filter；frequent=非
            // optional）
            HostBehavior::SwdJqimou => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 剑击·少微面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdJianji => None,
            // 头弹·麦星面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdToudan => None,

            // 黑火：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdHeihuo => None,
            // 真威：其他角色响应结算后（确认门=optional）
            HostBehavior::SwdFzhenwei => Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 遁行：成为其他角色卡牌目标（确认门=optional；CardUsed 帧
            // 目标侧）
            HostBehavior::SwdDunxing => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 孵鸟：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdFuniao => None,
            // 凝弦：自己受伤结算后（确认门=optional）
            HostBehavior::SwdNingxian => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 妖童：锁定 marker（engine viewAs 链查询），无触发面
            HostBehavior::SwdYaotong => None,
            // 拷贝栏：锁定（Draw 内联钩），无触发面
            HostBehavior::SwdKbolan => None,
            // 奥剑：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdAojian => None,
            // 化金：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdHuajin => None,
            // 化金触发面（A2 H41）：伤害前置 +1；自己回合开始自清
            //（extra PhaseBegin→TurnBegin；原锚自己判定阶段开始）
            HostBehavior::SwdHuajin2 => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::TurnBegin],
            }),
            // 明镜：自己受到伤害前置（无防具 + 1/3）
            HostBehavior::SwdMinjing => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 兰芷：使用梅花牌后（上游含 check 可放弃→原生恒发动近似）
            HostBehavior::SwdLanzhi => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 热血（A2 审计 H42）：上游 trigger {global:'phaseBegin'}＝
            // 他人回合开始，每回合一次（原锚任意阶段开始＝每回合 5→6 次）。
            HostBehavior::SwdRexue => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 幻音：他人对你使用牌（上游 useCardToBefore target，priority 5.9）
            HostBehavior::SwdHuanyin => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 6,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 灵弦：自己使用或打出牌（上游 respond/useCard；非自己回合）
            HostBehavior::SwdLingxian => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::CardResponded],
            }),
            // 玄咒：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXuanzhou => None,
            // 因果：他人伤害前置（确认门=optional；priority 6 同上游）
            HostBehavior::SwdYinguo => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 6,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 唤魂：任意角色濒死（确认门=optional；priority 6 同上游）
            HostBehavior::SwdHuanhun => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 6,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 倾城（A2 H70）：上游 player phaseEnd＝自己整回合结束
            //（frequent 锁定式=非 optional）
            HostBehavior::SwdLqingcheng => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 狩猎：可选（Sha 机器内联臂承载——filter/machine 无触发面）
            HostBehavior::SwdShoulie => None,
            // 不死：自己濒死（锁定强制；priority 7 先于鬼眼复活面/救援）
            HostBehavior::SwdBusi => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 7,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 破阵：自己受伤结算后（确认门=optional）
            HostBehavior::SwdPozhen => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 鬼眼·复活面：自己濒死（锁定强制；priority 6 同上游先于救援）
            HostBehavior::SwdGuiyan2 => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 6,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 机关凤封印（A2 H43）：拥有者下一回合开始解除（上游
            // addTempSkill {player:'phaseBegin'}＝回合开始；原锚任意自己
            // 阶段开始）
            HostBehavior::SwdFengyinState => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 机关怨待获得（A2 H71）：上游 jiguanyuan player phaseEnd＝
            // 拥有者整回合结束强制获得置牌
            HostBehavior::SwdJiguanyuanGain => Some(TriggerMeta {
                // 第 933 轮核实：上游 card/swd/skill.js:498 jiguanyuan 为 player: phaseEnd（旧台账 G23b 误归因 EventKind::Damage）
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdJiguanfengCard => None,
            HostBehavior::SwdJiguanyuanCard => None,
            // 机关妖术防重标记：纯状态，无触发
            HostBehavior::SwdJiguanyaoshuCard => None,
            HostBehavior::SwdJiguanyaoState => None,
            HostBehavior::SwdJiguanshuCard => None,
            HostBehavior::SwdTianfuzhenCard => None,
            HostBehavior::SwdQixingzhenCard => None,
            HostBehavior::SwdNiaoxiangzhenCard => None,
            HostBehavior::SwdChangshezhenCard => None,
            HostBehavior::SwdPozhenjueCard => None,
            // 机关妖术补偿：弃牌事件（含装备牌时触发，登记近似差异）
            HostBehavior::SwdJiguanyaoGain => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdXuanfengzhirenCard => None,
            HostBehavior::SwdZhongxinghujiaCard => None,
            HostBehavior::SwdXingtigaizaoCard => None,
            HostBehavior::SwdShijianhuisuCard => None,
            HostBehavior::SwdJinjilengdongCard => None,
            HostBehavior::SwdShengxiuhaojiaoCard => None,
            HostBehavior::SwdYinmilichangCard => None,
            HostBehavior::SwdLiuxinghuoyuCard => None,
            HostBehavior::SwdShentouEquipSkill => None,
            HostBehavior::SwdYufuluEquipSkill => None,
            HostBehavior::SwdYangpijuanCard => None,
            HostBehavior::SwdLangeguaiyiEquipSkill => None,
            // swd 机关牌族（v189-v192）：卡牌机器（无懈窗包装，intrinsic 分派——无触发）
            HostBehavior::SwdZhiluxiaohuCard => None,
            HostBehavior::SwdMujiarenCard => None,
            HostBehavior::SwdGouhunluoCard => None,
            // 勾魂锣·倒计时（A2 H44）：来源回合开始（global phaseBegin＝
            // 仅来源 player 的回合；原锚任意阶段开始）
            HostBehavior::SwdGouhunluoSub => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdJiguantongCard => None,
            _ => unreachable!("behavior routed to the wrong trigger_meta module"),
        }
    }
}
