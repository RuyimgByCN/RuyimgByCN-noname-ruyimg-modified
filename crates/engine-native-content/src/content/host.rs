//! G1 规则宿主原型（ADR 0776）：扩展包以**数据声明行为**，宿主以
//! 有界原语执行——扩展机器载荷的独立装载与显式续行状态。
//!（2026-10-10 事故重建：臂库自 db.part 重放+挖掘恢复；部分行为臂
//! 为占位待重写——见 file-loss-recovery-20261010 记忆）

use crate::catalog::{CardKind, QueryCtx, SkillBehavior};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::MachineSnapshot;

mod activation;
mod asks;
mod asks_mtg;
mod asks_wudian;
mod intrinsic;
pub(crate) use asks::active_ask_host;
pub(crate) use intrinsic::card_intrinsic;
mod boss;
mod constants;
mod factory;
mod filtering;
pub(in crate::content) mod step;
mod skill_defs;
mod support;

pub use crate::vocabulary::host::*;
use boss::*;
use constants::*;
use support::*;
pub use support::{
    jifeng_single_target_trick, sbzhaxiang_active, shengong_borrow_options,
    shengong_borrowed_owner, yangpijuan_class,
};
pub use step::step_l01_on_game_start;
pub(crate) use step::{
    host_before_facedown_turn_skip, host_before_gain_hand, host_before_recover,
    host_card_use_after, host_draw_begin_step, host_judge_settled, host_on_death_before,
    host_on_game_start, host_on_hand_card_removed, host_on_phase_use_end, host_on_player_died,
    host_on_recover_before, host_on_round_start, host_on_shanxian_turnover, host_on_turn_begin,
    host_on_turn_phase_after, host_preserves_skills_on_death, host_sha_begin_step,
    host_sha_end_step, on_cards_to_discard_chengye, on_l25_host_event,
    on_outer_settled_taipingyaoshu, on_phase_end_l10_scs, on_skill_removed_all, play_begin_l19,
    play_begin_tianshu, play_end_rui_skill1, reguhuo_consume_resp, reguhuo_has_pending_final,
    reguhuo_response_skill, reguhuo_spawn_final, reguhuo_spawn_response, scsanruo_response_skill,
    u01_phase_skipped_hook, u01_respond_skill_options as u01_respond_skill_options_hook,
    u01_try_activate_respond_skill as u01_try_activate_respond_skill_hook,
};

pub(crate) fn host_play_begin_step(ctx: &mut Ctx<'_>, player: PlayerId) {
    play_begin_tianshu(ctx, player);
    play_begin_l19(ctx, player);
}
pub(crate) use support::{niepan_effect, sync_land_skill};

/// 机关牌名目标规则（按上游 filterTarget 与引擎 PlayCard 校验同口径）。
enum CyqiaoxieRule {
    SelfTarget,
    OneOther,
    AllOthers,
}

/// 宿主技能续执行机器：显式类型化状态（挂起/恢复载荷）。
#[derive(Debug, Clone)]
pub struct HostMachine {
    /// 定义键（目录技能名；String 便于快照编解码，无泄漏风险）。
    pub def_key: String,
    pub behavior: HostBehavior,
    pub owner: PlayerId,
    pub target: Option<PlayerId>,
    /// 关联卡牌（手里剑：伤害归因）。
    pub card: Option<CardId>,
    /// 关联卡牌组（思继：本次弃置的牌）。
    pub cards: Vec<CardId>,
    /// 多目标行为的目标列表（水攻）。
    pub targets: Vec<PlayerId>,
    /// 显式续行阶段（入快照；恢复后从该阶段继续）。
    pub stage: HostStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStage {
    /// 执行行为效果。
    Act,
    /// 手里剑：等待目标选择（弃装备 / 受伤）。
    TargetChoice,
    /// 五行盘：等待选择要改属性的手牌。
    SelectCard,
    /// 五行盘：等待选择新属性（PickTag 0..=4）。
    SelectNature,
    /// 拼点：等待目标选牌。
    CompareTargetPick,
    /// 拼点：等待己方选牌。
    ComparePlayerPick,
    /// 丈八蛇矛：等待选择两张手牌。
    CardsAsk,
    /// 分舵（习嫣 jiefen）：等待选交牌目标。
    JiefenTarget,
    /// 分舵：等待目标选交出的手牌。
    JiefenTargetDiscard,
    /// 分舵：等待自己选交出的手牌。
    JiefenOwnerDiscard,
    /// 神秘果窗口（卡牌面窗口机批）：载体用牌结算后——原体上桌面重发射。
    ShenmiguoRestage,
    /// 鼠槌（equip5）：等待目标选择（攻击范围内）。
    ShuchuiTarget,
    /// 鼠槌（equip5）：目标已选定——逐张用杀循环（凤略 Yield 同型）。
    ShuchuiTargetLoop,
    /// 还人：等待选下一关武将（PickTag 下标）。
    HuanrenPick,
    /// R05-C2 续行位：本机已发 start_draw 并 Yield；R 子树弹栈后从本
    /// 阶段恢复执行该行为的「摸后尾段」（每个行为各自的尾段，同一行为
    /// 同时至多一张在飞 R 帧——阶段语义由 behavior 唯一确定）。
    DrawSettle,
    /// L10 蛊惑：询问场上未带 rechanyuan 的其他角色是否质疑。
    ReguhuoGuessPick,
    /// L10 蛊惑：真牌结算后质疑者二选一（弃置手牌或失 1 点体力）。
    ReguhuoFinalPick,
}

impl HostStage {
    pub const fn tag(self) -> u8 {
        match self {
            HostStage::Act => 0,
            HostStage::TargetChoice => 1,
            HostStage::SelectCard => 2,
            HostStage::SelectNature => 3,
            HostStage::CompareTargetPick => 4,
            HostStage::ComparePlayerPick => 5,
            HostStage::CardsAsk => 6,
            HostStage::JiefenTarget => 7,
            HostStage::JiefenTargetDiscard => 8,
            HostStage::JiefenOwnerDiscard => 9,
            HostStage::ShenmiguoRestage => 10,
            HostStage::ShuchuiTargetLoop => 11,
            HostStage::ShuchuiTarget => 12,
            HostStage::HuanrenPick => 13,
            HostStage::DrawSettle => 14,
            HostStage::ReguhuoGuessPick => 15,
            HostStage::ReguhuoFinalPick => 16,
        }
    }
    pub fn from_tag(t: u8) -> Option<Self> {
        Some(match t {
            0 => HostStage::Act,
            1 => HostStage::TargetChoice,
            2 => HostStage::SelectCard,
            3 => HostStage::SelectNature,
            4 => HostStage::CompareTargetPick,
            5 => HostStage::ComparePlayerPick,
            6 => HostStage::CardsAsk,
            7 => HostStage::JiefenTarget,
            8 => HostStage::JiefenTargetDiscard,
            9 => HostStage::JiefenOwnerDiscard,
            10 => HostStage::ShenmiguoRestage,
            11 => HostStage::ShuchuiTargetLoop,
            12 => HostStage::ShuchuiTarget,
            13 => HostStage::HuanrenPick,
            14 => HostStage::DrawSettle,
            15 => HostStage::ReguhuoGuessPick,
            16 => HostStage::ReguhuoFinalPick,
            _ => return None,
        })
    }
}

/// 宿主行为实现：打包为 SkillDef 的行为载体。
pub struct HostSkillBehavior {
    pub name: String,
    pub behavior: HostBehavior,
}

impl SkillBehavior for HostSkillBehavior {
    fn native_damage_effect(
        &self,
        kind: crate::event::EventKind,
    ) -> Option<crate::event::NativeDamageEffect> {
        crate::event::NativeDamageEffect::from_host(self.behavior)
            .filter(|e| e.window().event_kind() == kind)
    }
    fn filter_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: crate::event::EventKind,
    ) -> bool {
        if let Some(effect) = self.native_damage_effect(kind) {
            return matches!(event, EventData::Damage(d) if effect.matches(owner,d,ctx,None,None));
        }
        self.filter(ctx, owner, event)
    }
    fn filter_in_window_with_frames(
        &self,
        ctx: &QueryCtx<'_>,
        _frames: &[crate::event::Frame],
        frame: &crate::event::Frame,
        owner: PlayerId,
        event: &EventData,
        kind: crate::event::EventKind,
    ) -> bool {
        if self.native_damage_effect(kind).is_some() {
            return self.filter_in_window(ctx, owner, event, kind);
        }
        match self.behavior {
            HostBehavior::MbQianlong => match kind {
                crate::event::EventKind::Damage => false,
                crate::event::EventKind::DamageSource => matches!(
                    event,
                    EventData::Damage(d)
                        if !d.prevented
                            && d.amount > 0
                            && d.source == Some(owner)
                            && ctx.state.player(owner).mbqianlong_marks < 99
                ),
                crate::event::EventKind::DamageEnd => matches!(
                    event,
                    EventData::Damage(d)
                        if !d.prevented
                            && d.amount > 0
                            && d.target == owner
                            && ctx.state.player(owner).mbqianlong_marks < 99
                ),
                _ => self.filter(ctx, owner, event),
            },
            HostBehavior::MbcmJiushi => match kind {
                crate::event::EventKind::Damage => false,
                crate::event::EventKind::DamageBegin3 => matches!(
                    event,
                    EventData::Damage(d)
                        if !d.prevented
                            && d.amount > 0
                            && d.target == owner
                            && ctx.state.player(owner).face_down
                ),
                crate::event::EventKind::DamageEnd => {
                    let EventData::DamageWindow { root, .. } = frame.data else {
                        return false;
                    };
                    matches!(
                        event,
                        EventData::Damage(d)
                            if !d.prevented
                                && d.amount > 0
                                && d.target == owner
                                && ctx
                                    .state
                                    .vm_aux
                                    .flags
                                    .get(&(owner, format!("mbcmjiushi_dmg:{root}")))
                                    .copied()
                                    .unwrap_or(0)
                                    > 0
                    )
                }
                _ => self.filter(ctx, owner, event),
            },
            HostBehavior::L25ZuanBatchE if self.name == "pyzhuren_diamond" => {
                step::l25_batch_e::pyzhuren_diamond_filter(ctx, _frames, frame, owner, event, kind)
            }
            _ => self.filter_with_frames(ctx, _frames, frame, owner, event),
        }
    }
    fn is_forever(&self) -> bool {
        matches!(
            self.behavior,
            HostBehavior::MbQianlong
                | HostBehavior::MbWeitong
                | HostBehavior::MbcmQingzheng
                | HostBehavior::MbcmJiushi
                | HostBehavior::MbcmFangzhu
                | HostBehavior::MbJuejin
        )
    }
    fn is_charlotte(&self) -> bool {
        matches!(
            self.behavior,
            HostBehavior::ShengkeCheck
                | HostBehavior::GwentFengyinState
                | HostBehavior::SwdFengyinState
                | HostBehavior::GujianFoodShihuifenBan
                | HostBehavior::SwdQinglongzhiguiHeld
                | HostBehavior::SwdZhuquezhizhangHeld
                | HostBehavior::SwdBaishouzhihuHeld
                | HostBehavior::SwdXuanwuzhihuangHeld
                | HostBehavior::SwdHuanglinzhicongHeld
                | HostBehavior::SwdCangchizhibiHeld
                | HostBehavior::SwdShencaojieGuard
                | HostBehavior::SwdYuchanSwap
                | HostBehavior::SwdShenmiguoWindow
                | HostBehavior::MtgYixialanRecast
                | HostBehavior::MtgLinzhongjiantaStrike
                | HostBehavior::MtgYoulinDiscard
                | HostBehavior::Rechanyuan
        ) || matches!(
            self.name.as_str(),
            "th_zhengjing2"
                | "th_tiansuan2"
                | "th_tiansuan2_0"
                | "th_tiansuan2_1"
                | "th_tiansuan2_2"
                | "th_tiansuan2_3"
                | "th_tiansuan2_4"
                | "th_tiansuan2_damage"
                | "th_tiansuan2_ai"
                | "th_tiansuan2_fire"
                | "th_gushe2"
                | "th_taipingyaoshu"
                | "th_g_taipingyaoshu_ai"
                | "th_pcaudio_shuijing_card"
                | "th_pcaudio_xuanjian_card"
                | "txhj_kapaikeshi"
                | "txhj_huangtiannizhuan"
                | "xiajiao2"
                | "xiajiao3"
                | "gjyuheng_plus_temp"
                | "dangping2"
                | "mingkong2"
                | "shahun2"
                | "yangming2"
                | "jiehuo2"
                | "yuling5"
                | "yuling6"
                | "shihuifen"
                | "g_shihuifen"
        )
    }
    fn hand_limit_base(
        &self,
        core: &crate::state::GameStateCore,
        owner: PlayerId,
    ) -> Option<usize> {
        (self.name == "txhj_xiongziyingfa").then(|| core.player(owner).max_hp as usize)
    }
    fn window_trigger_meta(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        _event: &EventData,
        kind: crate::event::EventKind,
    ) -> Option<crate::catalog::TriggerMeta> {
        if kind == crate::event::EventKind::CardUsed
            && matches!(
                self.behavior,
                HostBehavior::ShengkeCheck | HostBehavior::HuanleMonkeyCard
            )
        {
            // 上游 `useCardToBegin` 按 `(current_player 顺座位序, priority)` 统一排队（如
            // `useCardToBegin` 窗口中 `_shengke` priority 0、同一座次的 `monkey` priority -25
            // 先于下家 `_shengke`）；内核用 `seat_steps_from` 编码进 `Timing::Last` 优先级。
            let seat_steps = i32::from(ctx.state.seat_steps_from(ctx.state.turn.current, owner));
            let base: i32 = if self.behavior == HostBehavior::ShengkeCheck {
                0
            } else {
                -25
            };
            return Some(crate::catalog::TriggerMeta {
                event: crate::event::EventKind::CardUsed,
                timing: crate::event::Timing::Last,
                priority: (32 - seat_steps) * 100 + base,
                order_selectable: self.behavior == HostBehavior::HuanleMonkeyCard,
                optional: false,
                extra_events: Vec::new(),
            });
        }
        self.kind_trigger_meta(kind)
    }
    fn kind_trigger_meta(
        &self,
        kind: crate::event::EventKind,
    ) -> Option<crate::catalog::TriggerMeta> {
        match self.behavior {
            // 苦无（欢乐卡牌/card/skill.js:55）：source: "damageSource"（Dying 后伤害来源窗，保留静态声明锁）
            HostBehavior::ShaDamageForceDiscard => Some(crate::catalog::TriggerMeta {
                event: crate::event::EventKind::DamageSource,
                timing: crate::event::Timing::Normal,
                priority: -25,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            // 五行生克（wuxing/extension.js:64）：useCardToBegin 于 Timing::Last 结算，保证追加/移除目标先完成；
            // 装备牌使用通过 `EventKind::Equipped`（`AfterTriggers`）同步进入 `_shengke`。
            HostBehavior::ShengkeCheck if kind == crate::event::EventKind::CardUsed => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::CardUsed,
                    timing: crate::event::Timing::Last,
                    priority: 0,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                })
            }
            HostBehavior::ShengkeCheck if kind == crate::event::EventKind::Equipped => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::Equipped,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                })
            }
            // 猴子偷桃（欢乐卡牌/card/skill.js:5）：useCardToBegin（`optional: false` + 内部 RespondCard 确认）
            HostBehavior::HuanleMonkeyCard if kind == crate::event::EventKind::CardUsed => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::CardUsed,
                    timing: crate::event::Timing::Last,
                    priority: -25,
                    order_selectable: true,
                    optional: false,
                    extra_events: Vec::new(),
                })
            }
            // U08 霓裳/挽歌/藉用：无条件覆盖运行期触发窗（保留静态内容锁同时避免旧 PhaseBegin/CardUsed 回落）
            HostBehavior::U08YxsBatchC => match self.name.as_str() {
                "nichang" => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::DrawPhaseBefore,
                    timing: crate::event::Timing::Normal,
                    priority: 1,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                "wange" => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::DrawPhaseBegin,
                    timing: crate::event::Timing::Normal,
                    priority: 1,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                "nichang2" | "wange2" => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::TurnPhaseEnd,
                    timing: crate::event::Timing::Normal,
                    priority: 1,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                _ => None,
            },
            HostBehavior::U08YxsBatchD if self.name == "jieyong" => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::TurnBegin,
                    timing: crate::event::Timing::Normal,
                    priority: 1,
                    order_selectable: false,
                    optional: false,
                    extra_events: vec![crate::event::EventKind::TurnEnded],
                })
            }
            // U07 筑城护卫（玩点论杀）：运行时可选触发，保留静态内容锁
            HostBehavior::WudianZhuchengGuard if kind == crate::event::EventKind::CardUsed => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::CardUsed,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: true,
                    optional: true,
                    extra_events: Vec::new(),
                })
            }
            // L29 龙血玄黄：潜龙拆分 damageSource / damageEnd；清正 direct:true；
            // 酒诗改为 damageBegin3 (silent firstDo) + damageEnd + TurnOvered
            HostBehavior::MbQianlong => match kind {
                crate::event::EventKind::CardsDrawn
                | crate::event::EventKind::CardsMoved
                | crate::event::EventKind::DamageSource
                | crate::event::EventKind::DamageEnd => Some(crate::catalog::TriggerMeta {
                    event: kind,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: true,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                crate::event::EventKind::Damage => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::PhaseBegin,
                    timing: crate::event::Timing::First,
                    priority: 6,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                _ => None,
            },
            HostBehavior::MbcmQingzheng if kind == crate::event::EventKind::PhaseBegin => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::PhaseBegin,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: true,
                    optional: false,
                    extra_events: Vec::new(),
                })
            }
            HostBehavior::MbcmJiushi => match kind {
                crate::event::EventKind::DamageBegin3 => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::DamageBegin3,
                    timing: crate::event::Timing::First,
                    priority: 0,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                crate::event::EventKind::DamageEnd => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::DamageEnd,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: true,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                crate::event::EventKind::TurnOvered => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::TurnOvered,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: true,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                crate::event::EventKind::Damage => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::TurnOvered,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: false,
                    optional: false,
                    extra_events: Vec::new(),
                }),
                _ => None,
            },
            HostBehavior::L19DoomsdayBatchA => match self.name.as_str() {
                "_wujinmoshiRE" if kind == crate::event::EventKind::Death => {
                    Some(crate::catalog::TriggerMeta {
                        event: crate::event::EventKind::Death,
                        timing: crate::event::Timing::Normal,
                        priority: 22,
                        order_selectable: false,
                        optional: false,
                        extra_events: Vec::new(),
                    })
                }
                "_jiangshi2" if kind == crate::event::EventKind::TurnBegin => {
                    Some(crate::catalog::TriggerMeta {
                        event: crate::event::EventKind::TurnBegin,
                        timing: crate::event::Timing::Normal,
                        priority: 15,
                        order_selectable: false,
                        optional: false,
                        extra_events: Vec::new(),
                    })
                }
                _ => None,
            },
            HostBehavior::L19DoomsdayBatchB => match self.name.as_str() {
                "_guandieskill" if kind == crate::event::EventKind::Death => {
                    Some(crate::catalog::TriggerMeta {
                        event: crate::event::EventKind::Death,
                        timing: crate::event::Timing::Normal,
                        priority: -99,
                        order_selectable: false,
                        optional: false,
                        extra_events: Vec::new(),
                    })
                }
                _ => None,
            },
            HostBehavior::FzLiegong if kind == crate::event::EventKind::CardUsed => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::CardUsed,
                    timing: crate::event::Timing::Normal,
                    priority: 0,
                    order_selectable: true,
                    optional: false,
                    extra_events: Vec::new(),
                })
            }
            HostBehavior::L20MarkBatchA if self.name == "wengua" || self.name == "wengua5" => {
                Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::PhaseBegin,
                    timing: crate::event::Timing::Normal,
                    priority: 1,
                    order_selectable: false,
                    optional: false,
                    extra_events: vec![
                        crate::event::EventKind::PhaseEnd,
                        crate::event::EventKind::TurnEnded,
                        crate::event::EventKind::TurnPhaseAfter,
                    ],
                })
            }
            HostBehavior::L20MarkBatchB => match (self.name.as_str(), kind) {
                ("fz_liegong", crate::event::EventKind::CardUsed) => {
                    Some(crate::catalog::TriggerMeta {
                        event: crate::event::EventKind::CardUsed,
                        timing: crate::event::Timing::Normal,
                        priority: 0,
                        order_selectable: true,
                        optional: false,
                        extra_events: Vec::new(),
                    })
                }
                ("mibei", _) => Some(crate::catalog::TriggerMeta {
                    event: crate::event::EventKind::CardUsed,
                    timing: crate::event::Timing::Normal,
                    priority: 1,
                    order_selectable: false,
                    optional: false,
                    extra_events: vec![
                        crate::event::EventKind::PhaseBegin,
                        crate::event::EventKind::Equipped,
                        crate::event::EventKind::TurnEnded,
                        crate::event::EventKind::TurnPhaseAfter,
                    ],
                }),
                ("kuanshi", crate::event::EventKind::DamageBefore) => {
                    Some(crate::catalog::TriggerMeta {
                        event: crate::event::EventKind::PhaseBegin,
                        timing: crate::event::Timing::Normal,
                        priority: 1,
                        order_selectable: false,
                        optional: false,
                        extra_events: vec![
                            crate::event::EventKind::TurnBegin,
                            crate::event::EventKind::Death,
                        ],
                    })
                }
                ("kuanshi2", crate::event::EventKind::DamageBefore) => {
                    Some(crate::catalog::TriggerMeta {
                        event: crate::event::EventKind::DamageBefore,
                        timing: crate::event::Timing::Last,
                        priority: -100,
                        order_selectable: false,
                        optional: false,
                        extra_events: vec![
                            crate::event::EventKind::TurnBegin,
                            crate::event::EventKind::PhaseBegin,
                            crate::event::EventKind::Death,
                        ],
                    })
                }
                _ => None,
            },
            // U01 `dddmoyan` (`roundStart` = TurnBegin) 与 U01 Sub-batch 1：运行期覆盖触发窗，保留静态内容锁
            HostBehavior::D3Moyan => Some(crate::catalog::TriggerMeta {
                event: crate::event::EventKind::TurnBegin,
                timing: crate::event::Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: vec![crate::event::EventKind::DamageBefore],
            }),
            HostBehavior::U01DddBatchA => {
                step::u01_fixes_1::trigger_meta_u01_fix(&self.name)
            }
            HostBehavior::U01DddBatchB | HostBehavior::U01DddBatchC | HostBehavior::U01DddBatchD => {
                step::u01_fixes_1::trigger_meta_u01_fix(&self.name)
                    .or_else(|| step::u01_fixes_2::trigger_meta_u01_fix2(&self.name))
                    .or_else(|| step::u01_fixes_3::trigger_meta_u01_fix3(&self.name))
            }
            HostBehavior::U05OwBatchA
            | HostBehavior::U05OwBatchB
            | HostBehavior::U05OwBatchC
            | HostBehavior::U05OwBatchD => {
                step::u05_ow::window_trigger_meta_u05_ow(self.behavior, &self.name, kind)
            }
            _ => None,
        }
    }

    fn runs_before_phase_begin(&self) -> bool {
        step::u01_fixes_2::runs_before_phase_begin_2(&self.name)
            || step::u01_fixes_3::runs_before_phase_begin_3(&self.name)
    }
    fn start_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: crate::event::EventKind,
    ) -> Box<dyn crate::catalog::SkillMachine> {
        if let Some(effect) = self.native_damage_effect(kind) {
            return Box::new(crate::kernel::NativeDamageSkill {
                owner,
                def: ctx
                    .catalog
                    .skill_by_name(&self.name)
                    .expect("host declaration"),
                window: effect.window(),
            });
        }
        let mut machine = self.machine(owner, event);
        match self.behavior {
            HostBehavior::MbQianlong => {
                machine.card = Some(match kind {
                    crate::event::EventKind::PhaseBegin => 0,
                    crate::event::EventKind::CardsDrawn | crate::event::EventKind::CardsMoved => 1,
                    crate::event::EventKind::DamageEnd => 2,
                    crate::event::EventKind::DamageSource => 3,
                    _ => 0,
                });
            }
            HostBehavior::MbcmJiushi => {
                machine.card = Some(match kind {
                    crate::event::EventKind::DamageEnd => 1,
                    crate::event::EventKind::TurnOvered => 2,
                    crate::event::EventKind::DamageBegin3 => 3,
                    _ => 1,
                });
            }
            _ => {}
        }
        Box::new(machine)
    }

    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        self.filter_behavior(ctx, owner, event)
    }
    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.machine(owner, event))
    }
    fn activate(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        self.activate_behavior(ctx, owner)
    }
    fn as_host_behavior(&self) -> Option<HostBehavior> {
        Some(self.behavior)
    }
    fn blocks_delayed_trick_target(&self) -> bool {
        self.blocks_delayed_trick()
    }
    fn blocks_all_cards(&self) -> bool {
        matches!(self.behavior, HostBehavior::Bingfeng2)
    }
    fn needs_vm_card_history(&self) -> bool {
        matches!(
            self.behavior,
            HostBehavior::L11BuffsA
                | HostBehavior::L11BuffsB
                | HostBehavior::L11BuffHelpersA
                | HostBehavior::L11BuffsC
                | HostBehavior::L11BuffsD
                | HostBehavior::L11BuffHelpersB
                | HostBehavior::L11SystemHooks
                | HostBehavior::L11Weapons
                | HostBehavior::L11GeneralBossC
                | HostBehavior::L11BossLingfeng
        )
    }
    /// 止息（L27 批 2）：**计数标记 ≥ 体力** 或 **已使用过锦囊**（记录标记
    /// 非零）→ 封锁出牌/响应。两个标记分别以 `new_zhixi` 与
    /// `new_zhixi_mark2` 的技能定义为键存于 `vm_aux.marks`。
    fn blocks_all_cards_when(
        &self,
        catalog: &crate::catalog::Catalog,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        if self.behavior == HostBehavior::U05OwBatchB && self.name == "shanguang2" {
            return true;
        }
        if self.behavior != HostBehavior::NewZhixi {
            return self.blocks_all_cards();
        }
        if core.turn.current != p || core.turn.phase != crate::event::Phase::Play {
            return false;
        }
        let mark_of = |name: &str| -> i32 {
            catalog
                .skill_by_name(name)
                .and_then(|d| core.vm_aux.marks.get(&(p, u32::from(d))).copied())
                .unwrap_or(0)
        };
        let count = mark_of(L27_NEW_ZHIXI_KEY);
        let trick = mark_of(L27_NEW_ZHIXI_MARK2_KEY) > 0;
        trick || count >= core.player(p).hp
    }
}

impl crate::catalog::SkillMachine for HostMachine {
    fn actor_owner(&self) -> Option<crate::ids::PlayerId> {
        Some(self.owner)
    }
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Host {
            def_key: self.def_key.clone(),
            behavior: self.behavior.tag(),
            stage: self.stage.tag(),
            owner: self.owner,
            target: self.target,
            card: self.card,
            cards: self.cards.clone(),
            targets: self.targets.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        self.step_behavior(ctx)
    }
}

pub use skill_defs::{host_skill_def, host_skill_limits, host_skill_trigger_meta};
pub use crate::vocabulary::card_skills::{card_kind_equip_skills, card_kind_host_skills};
