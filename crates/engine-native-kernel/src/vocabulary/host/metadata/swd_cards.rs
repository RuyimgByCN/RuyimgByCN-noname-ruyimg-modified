//! SWD 扩展卡牌、机关与伴随子技触发元数据。
use super::super::*;

impl HostBehavior {
    pub(super) fn trigger_meta_swd_cards(&self) -> Option<TriggerMeta> {
        match self {
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
            // 偷袭：他人整回合结束（global phaseEnd；optional + check 由内容面确认）
            HostBehavior::SwdTouxi => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            // 偷袭·冷却：自己回合开始移除自身（forced silent）
            HostBehavior::SwdTouxi2 => Some(TriggerMeta {
                event: EventKind::TurnBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 拷贝栏·置顶：由 DrawMachine drawEnd 直推，无独立 TriggerMeta
            HostBehavior::SwdKbolan2 => None,
            // 麻醉·回收：伤害结算后/归零时返还存储黑牌并移除自身（forced silent）
            HostBehavior::SwdLmazui3 => Some(TriggerMeta {
                event: EventKind::DamageEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![EventKind::DamageZero],
            }),
            // 玄影·回合外失牌反击（swd/skill.js:1078）：自己失去牌时（回合外+已横置门在 filter；direct optional 目标选择）
            HostBehavior::SwdXuanyingUse => Some(TriggerMeta {
                event: EventKind::CardsLost,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![
                    EventKind::CardsDiscarded,
                    EventKind::CardsMoved,
                    EventKind::CardUsed,
                    EventKind::CardResponded,
                ],
            }),
            // 破舟·封印到期（swd/skill.js:3463）：目标自己下一回合结束（TurnPhaseAfter）恢复非锁定技并移除自身
            HostBehavior::SwdPozhouSealed => Some(TriggerMeta {
                event: EventKind::TurnPhaseAfter,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            }),
            _ => unreachable!("behavior routed to the wrong trigger_meta module"),
        }
    }
}
