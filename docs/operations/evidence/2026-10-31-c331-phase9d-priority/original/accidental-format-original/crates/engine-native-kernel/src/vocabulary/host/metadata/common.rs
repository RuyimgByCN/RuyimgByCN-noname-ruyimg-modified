use super::super::*;

impl HostBehavior {
    pub(super) fn trigger_meta_common(&self) -> Option<TriggerMeta> {
        match self {
            HostBehavior::ShaDamageForceDiscard => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::PhaseEndRecoverOrDraw => Some(TriggerMeta {
                // 上游 `player:'phaseEnd'`（自己每个阶段结束）
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouXicai => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouDiykuanggu => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouXiongzi => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouYaliang => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 纵火：伤害前置窗（nature 写回须在 commit 前生效）
            HostBehavior::YunchouZonghuo => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouShaoying => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouLuweiyan => None,
            HostBehavior::YunchouJieyan => None,
            HostBehavior::YunchouYiesheng => None,

            // 理断：获得结算后（CardsMoved after）

            // 坚壁：锁定查询面（AOE 目标段查询，无触发帧）

            // 复仇：目录锚（取消逻辑在 ShaResolveMachine 内联）

            // 良策·余获：锁定查询面（五谷机器内联查询，无触发帧）

            // 归汉：目录锚（limited 确认逻辑在 DyingSaveMachine 内联）

            // 禳星：自己受伤害结算后 / 失去体力后（silent forced）
            HostBehavior::HlssXiangxing => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::LoseHp],
            }),
            // 隐月：伤害前置窗（num+1 写回 + 记录伤害类型）
            HostBehavior::HlssYueyin => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 起风：准备阶段开始（Judge 阶段 = 上游 phaseZhunbei），可选
            HostBehavior::HlssFengqi => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 起风·闭 / 改命：锁定旗标（无触发时机；效果内联）
            HostBehavior::HlssFengqi2 => None,
            HostBehavior::HlssGaiming => None,
            // 潜龙：开局首阶段（phase_number==1）+ 获得牌 + 伤害两向
            HostBehavior::MbQianlong => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::First,
                priority: 6,
                order_selectable: true,
                optional: false,
                extra_events: vec![
                    EventKind::CardsDrawn,
                    EventKind::CardsMoved,
                    EventKind::Damage,
                ],
            }),
            // 涅槃：自己濒死（可选；出牌阶段面走 activate）
            HostBehavior::Niepan => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 清正：出牌阶段开始（可选）
            HostBehavior::MbcmQingzheng => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 烈弓：被动（杀机器内联，无触发时机）
            HostBehavior::FzLiegong => None,
            // 狂骨：自己造成伤害后（可选）
            HostBehavior::FzXinKuanggu => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 苦肉：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Kurou => None,
            // 主亡忠继：任意角色死亡（锁定）
            HostBehavior::ZhuWanzhongJi => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 摸牌翻倍：被动（摸牌收束内联，无触发时机）
            HostBehavior::XiguaMopaiFanbei => None,
            // 血量翻倍：首次阶段开始（一次性；近似 gameStart）
            HostBehavior::XiguaXueliangFanbei => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 门神：自己结束阶段（可选）
            HostBehavior::YxsMenshen => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 门神·清除：任意阶段开始 + 任意角色死亡（锁定）
            HostBehavior::YxsMenshen3 => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::Death],
            }),
            // 门神·标记持有：被动（无触发时机）
            HostBehavior::YxsMenshen2 => None,
            // 反击：自己受到伤害后（可选）
            HostBehavior::YxsFanji => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 反击·红杀直伤：被动（杀机器内联，无触发时机）
            HostBehavior::YxsFanji2 => None,
            // 灾变：自己出牌阶段开始（锁定）
            HostBehavior::Zaibian => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 誓师：自己击杀角色后（锁定）
            HostBehavior::Shishi => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 循梦：自己造成伤害（**伤害前置窗**；上游 damageBegin 同窗）
            HostBehavior::Xunmeng => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 集舍：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Jishe => None,
            // 诈降：被动（mod/摸牌/直伤面内联，无触发时机）
            HostBehavior::SbZhaXiang => None,
            // 每步：任意角色出牌阶段开始（可选）
            HostBehavior::NewMeibu => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 每步·距离：被动（无触发时机）
            HostBehavior::NewMeibuRange => None,
            // 止息：自己使用牌（锁定；firstDo）
            HostBehavior::NewZhixi => Some(TriggerMeta {
                event: EventKind::CardUsed,
                // 上游 firstDo:true → 同事件内最先结算
                timing: Timing::First,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 撷芳：全局死亡结束 / 摸牌阶段结束（锁定）
            HostBehavior::Xiefang => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::Death],
            }),
            // 让节：自己受到伤害后（可选）
            HostBehavior::Rangjie => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 蛊惑：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Reguhuo => None,
            // 攻心：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Gongxin => None,
            // 魔化：自己进入濒死（锁定）
            HostBehavior::Mohua2 => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 修罗：自己准备阶段开始（可选）
            HostBehavior::Xiuluo => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 灭道·弃：自己弃牌阶段开始（锁定）
            HostBehavior::Miedao => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 魔焰：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Moyan => None,
            // 疾风：自己使用多目标单目标牌（结算窗；锁定）
            HostBehavior::Jifeng => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 昆仑镜：阶段收束（锁定记录快照）+ 阶段开始（体力低于快照值
            // 时**机器内确认门**回滚——上游两面可选性不同，故不用
            // TriggerMeta.optional）
            HostBehavior::Kunlunjing => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::PhaseEnd],
            }),
            // 神弓：被动修饰（无触发时机；响应窗内联候选）
            HostBehavior::Shengong => None,
            // 轩辕剑·伤增：装备者造成伤害前（锁定）
            HostBehavior::XuanyuanjianDamage => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 轩辕剑·反噬：装备者造成伤害后（锁定）
            HostBehavior::XuanyuanjianBacklash => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 轩辕：自己结束阶段（限定；可选）
            HostBehavior::Xuanyuan => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 燎原：自己使用【杀】（结算窗；可选）
            HostBehavior::Liaoyuan => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 改造：装备前置窗（DamageBefore 类；filter 检查 Equipped）
            HostBehavior::Gaizao => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![EventKind::Equipped],
            }),
            // 李香龙：自己的杀被闪避后（可选）
            HostBehavior::Lxianglong => Some(TriggerMeta {
                event: EventKind::ShaDodged,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 移花：成为单目标牌的目标（使用结算窗；可选）
            HostBehavior::Yihua => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 贞烈：成为他人【杀】/锦囊目标（使用结算窗；可选）
            HostBehavior::Zhenlie => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 天伦：判定机改判窗驱动（无触发时机）
            HostBehavior::Tianlun => None,
            // 脉轮：任意阶段开始（可选）
            HostBehavior::Mailun => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 极略：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Jilve => None,
            // 革新印：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Gxianyin => None,
            // 冰封·封：自己翻回正面时（自移除）
            HostBehavior::Bingfeng2 => Some(TriggerMeta {
                event: EventKind::TurnOvered,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 冰封：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Bingfeng => None,
            // 此桥邪：自己失去装备牌后（频繁技——恒发动，无确认门）
            HostBehavior::Cyqiaoxie => Some(TriggerMeta {
                event: EventKind::CardsLost,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 魂殉志 / 殉：无触发时机（主动面 + 内联臂）
            HostBehavior::Hxunzhi => None,
            HostBehavior::Hxunzhi2 => None,
            // 护天：自己阶段结束（可选）
            HostBehavior::Hutian => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 天香：自己受到伤害前（可选）
            HostBehavior::Tianxiang => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 天香·受：自己受到伤害结算后（运行时授予）
            HostBehavior::Tianxiang2 => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 颂词：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Songci => None,
            // 浴火 / 神威：无触发时机（浴火由涅槃收束内联；神威为 kernel 臂）
            HostBehavior::Yuhuo => None,
            HostBehavior::Shenwei => None,
            // 卫统：开局首阶段（登记面）
            HostBehavior::MbWeitong => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::First,
                priority: 10,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 隐月·判：翻面前置取消（不能被翻面；判定不生效由 kernel 内联）
            HostBehavior::HlssYueyin2 => Some(TriggerMeta {
                event: EventKind::TurnOverBefore,
                timing: Timing::Normal,
                priority: 20,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 鬼蛇旗/仙露回：trick 卡牌机（元数据不进触发计划）
            HostBehavior::GuisheqiCard => None,
            HostBehavior::XianluhuiCard => None,
            // 青木：死亡前置 First（上游 dieBegin hide）
            HostBehavior::BossQingmu2 => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::First,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 青木·转阶：死亡前置 Normal priority -10（上游 dieAfter）
            HostBehavior::BossQingmu2x => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: -10,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::BossQingmu3 => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::First,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::BossQingmu3x => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: -10,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 白芒/赤炎/玄麟 2/3 + 后援3/4：同青木 dieBegin First
            HostBehavior::BossBaimang2
            | HostBehavior::BossChiyan2
            | HostBehavior::BossXuanlin2
            | HostBehavior::BossBaimang3
            | HostBehavior::BossChiyan3
            | HostBehavior::BossXuanlin3
            | HostBehavior::BossBianshen3
            | HostBehavior::BossBianshen4 => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::First,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 白芒/赤炎/玄麟 2x/3x + 后援3/4·转阶：同青木·转阶 dieAfter Normal -10
            HostBehavior::BossBaimang2x
            | HostBehavior::BossChiyan2x
            | HostBehavior::BossXuanlin2x
            | HostBehavior::BossBaimang3x
            | HostBehavior::BossChiyan3x
            | HostBehavior::BossXuanlin3x
            | HostBehavior::BossBianshen3x
            | HostBehavior::BossBianshen4x => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: -10,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 还人：任意阶段结束即开局（原生无 gameStart 帧）
            HostBehavior::BossHuanren => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 还人·转关：同隐藏链 dieAfter
            HostBehavior::BossHuanrenDie => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: -10,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 青翼/赤翼：准备阶段锚 = 判定阶段开始
            HostBehavior::BossQingyi | HostBehavior::BossChiyi => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 赤仪2：伤害前置 +1
            HostBehavior::BossChiyiMark => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 白翼弃牌：第 5 轮判定阶段开始（roundStart 近似）
            HostBehavior::BossBaiyi => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::BossBaiyiThunder => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 悲歌：任意角色受伤结算后（global；确认门=optional）
            HostBehavior::YunchouBeige => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 复仇·清算：自己回合结束（标记逐个结算）

            // 蝉缘·变体：死亡结算后（来源减上限）
            HostBehavior::YunchouDiychanyuan => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouLuweiyan2 => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 蛊惑·变体：回合（判定阶段）开始
            HostBehavior::YunchouDiyguhuo => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 狡黠：成为牌目标（CardUsed+targets，frequent 自动）
            HostBehavior::YunchouJiaoxia => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::YunchouHonglian => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::TargetDiscardEquipOrDamage => None,
            HostBehavior::ShengkeCheck => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::ChangeCardNature => None,

            HostBehavior::MaxRangeWeaponDamage => Some(TriggerMeta {
                event: EventKind::Equipped,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 末焱：判定阶段开始（一轮一次）+ 伤害前置增幅
            HostBehavior::D3Moyan => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::DamageBefore],
            }),
            HostBehavior::WangmeiZhikeCard => None,
            HostBehavior::SijiDiscardDraw => Some(TriggerMeta {
                event: EventKind::CardsDiscarded,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::SanjianWeaponDamage => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::MoshouImmunity => None,
            HostBehavior::ShuigongCard => None,
            HostBehavior::FudichouxinCompare => None,
            HostBehavior::ChenhuodajieCard => None,
            HostBehavior::TouliangSwap => None,
            HostBehavior::LinghunzhihuoCard => None,
            HostBehavior::ZhaomingdanCard => None,
            HostBehavior::JihuocardDraw => None,
            HostBehavior::GeanguanhuoCompare => None,
            HostBehavior::CaochuanCard => None,
            HostBehavior::ShezhanqunruCompare => None,
            HostBehavior::GujianShujinsanCard => None,
            HostBehavior::GujianMianliCard => None,
            HostBehavior::GujianFeibiaoCard => None,
            HostBehavior::JinlianzhuCancel => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodTanhuadong => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodYougeng => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodGrant => None,
            HostBehavior::GujianFoodQingtuan => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodYuanbaorou => None,
            HostBehavior::GujianFoodGudonggeng => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodMapodoufu => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodChunbing => None,
            HostBehavior::GujianFoodLiyutang => Some(TriggerMeta {
                // 上游子技 `liyutang`: `trigger:{player:'phaseEnd'}`（第 991 轮）
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodMolicha => None,
            HostBehavior::GujianFoodXiajiao => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodMizhilianou => None,
            HostBehavior::GujianFoodLuyugeng => None,
            HostBehavior::JiguanBingpoCard => None,
            HostBehavior::JiguanQiankunCard => None,
            HostBehavior::JiguanLongxuCard => None,
            HostBehavior::JiguanShenhuoCard => None,
            HostBehavior::GujianFoodZiyangdan => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianYunvyuanshen => Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GujianFoodShihuifenBan => None,
            HostBehavior::GujianShihuifenCard => None,
            HostBehavior::GujianLiufengsanCard => None,
            HostBehavior::JiguanShatangCard => None,
            HostBehavior::JiguanHeilongCard => None,
            HostBehavior::JiguanLiutougeCard => None,
            HostBehavior::GujianMutouEquipSkill => None,
            HostBehavior::JiguanDinvCard => None,
            HostBehavior::GujianYuhengEquipSkill => None,
            HostBehavior::HearthDunpaiCard => None,
            HostBehavior::HearthZhiliaoboCard => None,
            HostBehavior::HearthYuansuCard => None,
            HostBehavior::HearthXingjieCard => None,
            HostBehavior::HearthChuansongCard => None,
            HostBehavior::HearthTansheCard => None,
            HostBehavior::HearthShijieshuCard => None,
            HostBehavior::GwentYanzhiCard => None,
            HostBehavior::GwentAozuCard => None,
            HostBehavior::GwentWenyiCard => None,
            HostBehavior::GwentZhihuanCard => None,
            HostBehavior::GwentBaoxueCard => None,
            HostBehavior::GwentQinpenState => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentBirinState => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentCiguState => Some(TriggerMeta {
                // 上游 `gw_ciguhanshuang`: `{player:'phaseDrawBegin'}`（第 938 轮）
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentWeatherGrant => None,
            HostBehavior::GwentKunenState => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentKunenCard => None,
            HostBehavior::GwentShanbengCard => None,
            HostBehavior::GwentPoxiaoCard => None,
            HostBehavior::GwentFengyinState => Some(TriggerMeta {
                event: EventKind::PhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::GwentZumoCard => None,
            HostBehavior::HuanleMonkeyCard => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::HuanleXuelunyangCard => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::HuanleMianjuCard => Some(TriggerMeta {
                event: EventKind::TurnOverBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::ZhangbaSha => None,
            HostBehavior::MtgLongshanfengLand => None,
            HostBehavior::MtgHaidaoLand => None,
            HostBehavior::MtgShuimomuxueLand => None,
            HostBehavior::MtgFeixuLand => None,
            HostBehavior::MtgYixialanLand => None,
            HostBehavior::MtgYixialanRecast => None,
            HostBehavior::MtgShamolvzhouLand => None,
            HostBehavior::MtgLinzhongjiantaLand => None,
            HostBehavior::MtgLinzhongjiantaStrike => None,
            HostBehavior::MtgBingheyaosaiLand => None,
            HostBehavior::MtgYoulinLand => None,
            HostBehavior::MtgYoulinDiscard => None,
            HostBehavior::MtgLindixiliuLand => None,

            _ => unreachable!("behavior routed to the wrong trigger_meta module"),
        }
    }
}
