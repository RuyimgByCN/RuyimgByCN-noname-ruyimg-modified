//! 技能机器运行上下文与流控。
//!
//! `Ctx` 是机器推进时的唯一可变入口：核心状态、执行位置、目录与宿主
//! 日志各自独立借用。机器通过 `ctx.emit` 压入子事件（自身自动挂起，
//! 子事件全部弹栈后继续），通过返回 `Flow::WaitChoice` 请求选择
//! （引擎校验应答后恢复，机器从 `ctx.answer` 读取）。

use crate::catalog::{Catalog, QueryCtx};
use crate::choice::{ChoiceOption, ChoiceSpec, Selection};
use crate::event::{CardUseData, DamageData, EventData, Phase};
use crate::ids::{CardId, PlayerId, SkillDefId};
use crate::resolution::Resolution;
use crate::state::GameStateCore;

mod helpers;
pub mod presentation;
mod skills;
pub use helpers::{
    card_use, card_use_as, card_use_with_nature, damage_data, damage_data_with, note_cards_lost,
};

/// 宿主展示/录像日志：引擎侧数据，不进状态、不进哈希。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogEvent {
    DamageDealt {
        source: Option<PlayerId>,
        target: PlayerId,
        amount: u32,
        redirected_from: Vec<PlayerId>,
        /// 造成伤害的牌（无牌伤害为 None）。
        card: Option<CardId>,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    DamagePrevented {
        target: PlayerId,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    CardsDrawn {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    VirtualCardPlayed {
        kind: crate::catalog::CardKind,
        player: PlayerId,
        targets: Vec<PlayerId>,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    CardPlayed {
        card: CardId,
        player: PlayerId,
        targets: Vec<PlayerId>,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    CardResponded {
        card: CardId,
        player: PlayerId,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    /// 体力上限减少（蝉缘类——max_hp 减面，运筹武将面批 9 起承载）。
    MaxHpLost {
        player: PlayerId,
        amount: u32,
    },
    CardDiscarded {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    CardsTaken {
        from: PlayerId,
        to: PlayerId,
        cards: Vec<CardId>,
        public: bool,
    },
    CardsGivenPublic {
        from: PlayerId,
        to: PlayerId,
        cards: Vec<CardId>,
    },
    CardsGiven {
        from: PlayerId,
        to: PlayerId,
        cards: Vec<CardId>,
    },
    HpRecovered {
        player: PlayerId,
        amount: u32,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    /// 护甲抵扣伤害（G1 扩展）。
    HujiaAbsorbed {
        player: PlayerId,
        amount: u32,
    },
    /// 失去体力（逐技对照批三 v77；非伤害——无来源/无属性）。
    HpLost {
        player: PlayerId,
        amount: u32,
    },
    /// Boss 重整 tick（逐技对照批三-a v82）：死亡玩家回合计数/回血。
    BossChongzhengTick {
        player: PlayerId,
        count: u8,
    },
    /// Boss 重整复活（计数达阈值）。
    BossRevived {
        player: PlayerId,
    },
    DyingEntered {
        player: PlayerId,
    },
    DyingSaved {
        player: PlayerId,
        by_card: CardId,
    },
    PlayerDied {
        player: PlayerId,
        killer: Option<PlayerId>,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    ChoiceOpened {
        id: u64,
        observer: PlayerId,
        prompt: &'static str,
    },
    ChoiceAnswered {
        id: u64,
        options: Vec<String>,
    },
    SkillTriggered {
        skill: &'static str,
        owner: PlayerId,
        presentation: Option<Box<engine_api::PublicPresentationEvent>>,
    },
    SkillToggled {
        player: PlayerId,
        skill: &'static str,
        enabled: bool,
    },
    PhaseChanged {
        player: PlayerId,
        phase: Phase,
        begin: bool,
    },
    TurnEnded {
        player: PlayerId,
    },
    /// R05（K21）：整回合结束窗两段（交接前，player＝刚结束回合的角色）。
    TurnPhaseEnded {
        player: PlayerId,
    },
    TurnPhaseAfter {
        player: PlayerId,
    },
    DeckReshuffled,
    Equipped {
        player: PlayerId,
        card: CardId,
        replaced: Option<CardId>,
    },
    Linked {
        player: PlayerId,
        linked: bool,
    },
    /// 锦囊被无懈抵消（窗口关闭时的最终状态）。
    CardNullified {
        card: CardId,
        player: PlayerId,
    },
    /// 判定：翻开一张判定牌。
    Judged {
        player: PlayerId,
        card: CardId,
    },
    /// 鬼道改判：`player` 打出 `card` 代替判定牌，被替换的 `replaced` 归其所有。
    JudgementRewritten {
        player: PlayerId,
        card: CardId,
        replaced: CardId,
    },
    /// 判定牌确定（改判询问全部结束后、效果生效前）。
    JudgementFinalized {
        player: PlayerId,
        card: CardId,
    },
    /// 天妒：`player` 将最终判定牌 `card` 收入手牌。
    JudgementGained {
        player: PlayerId,
        card: CardId,
    },
    /// 八卦阵防具判定：success=红桃（视为已出闪，伤害免除）。
    ArmorJudged {
        player: PlayerId,
        card: CardId,
        success: bool,
    },
    /// VM 亮牌：牌不移动，只进入权威日志（表现层不重复展示牌面）。
    CardsRevealed {
        cards: Vec<CardId>,
        to: Vec<PlayerId>,
    },
}

impl LogEvent {
    /// Occurrence-time committed public presentation event, if any.
    pub fn public_presentation(&self) -> Option<&engine_api::PublicPresentationEvent> {
        match self {
            LogEvent::DamageDealt { presentation, .. }
            | LogEvent::DamagePrevented { presentation, .. }
            | LogEvent::VirtualCardPlayed { presentation, .. }
            | LogEvent::CardPlayed { presentation, .. }
            | LogEvent::CardResponded { presentation, .. }
            | LogEvent::HpRecovered { presentation, .. }
            | LogEvent::PlayerDied { presentation, .. }
            | LogEvent::SkillTriggered { presentation, .. } => presentation.as_deref(),
            _ => None,
        }
    }
}

/// 机器单步结果。
pub enum Flow {
    /// 本机器完结。
    Done,
    /// 已 emit 子事件或被调度器挂起，等栈回到自身深度后继续。
    Yield,
    /// 请求一次选择；引擎校验应答后恢复。
    WaitChoice {
        observer: PlayerId,
        spec: ChoiceSpec,
    },
}

/// VM 执行故障（NC04）：可定位的错误类型与来源，供命令结果与日志使用。
///
/// 故障发生时该段必须整体回滚（排队效果丢弃、段状态不提交），命令以
/// `RejectReason::VmFault` 拒绝——不得以 `Flow::Done` 吞掉故障后按正常
/// 规则继续。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmFault {
    /// 触发故障的 VM 技能名。
    pub skill: String,
    /// 技能持有者座位。
    pub owner: crate::ids::PlayerId,
    /// 触发事件类别（如 `PhaseEnd`/`Damage`）。
    pub event: String,
    /// 错误类别（`VmError` 变体名）。
    pub kind: String,
    /// 错误详情（`VmError` 的显示文本）。
    pub detail: String,
}

/// 机器推进上下文。
pub struct Ctx<'a> {
    pub core: &'a mut GameStateCore,
    pub res: &'a mut Resolution,
    pub catalog: &'a Catalog,
    pub out: &'a mut Vec<LogEvent>,
}

impl<'a> Ctx<'a> {
    /// 只读视图（传给目录中的过滤器）。
    pub fn query(&self) -> QueryCtx<'_> {
        QueryCtx {
            state: self.core,
            catalog: self.catalog,
        }
    }

    // ---- 事件栈 ----

    /// 当前事件帧号：机器只在自己的恢复深度上运行，栈顶即触发帧；但机器
    /// 在一步内可能发射**失去牌子帧**（`CardsLost`——仅用于触发失去牌类
    /// 技能，不改变调用方正在处理的事件），此访问器跳过它，保证「我正处理
    /// 的那个事件」语义稳定。需要裸栈顶时用 `top_frame_id`。
    pub fn current_frame_id(&self) -> crate::ids::EventId {
        self.res
            .stack
            .iter()
            .rev()
            .find(|f| {
                f.kind != crate::event::EventKind::CardsLost
                    && !(f.kind == crate::event::EventKind::CardsDiscarded
                        && f.stage == crate::event::Stage::Commit
                        && !f.planned)
            })
            .or_else(|| self.res.stack.last())
            .expect("machine runs with a frame on stack")
            .id
    }

    /// 裸栈顶帧号（含失去牌子帧）。
    pub fn top_frame_id(&self) -> crate::ids::EventId {
        self.res
            .stack
            .last()
            .expect("machine runs with a frame on stack")
            .id
    }

    /// 当前帧的 CardUsed 载荷只读访问（同疾读目标等）。
    pub fn current_card_use(&self) -> Result<&CardUseData, ()> {
        let id = self.current_frame_id();
        self.res
            .frame(id)
            .ok_or(())
            .and_then(|f| f.card_use().ok_or(()))
    }

    /// 当前帧的 CardUsed 载荷可变访问（猴子偷桃等使用前取消写回）。
    pub fn current_card_use_mut(&mut self) -> &mut CardUseData {
        let id = self.current_frame_id();
        self.res
            .frame_mut(id)
            .expect("current frame exists")
            .card_use_mut()
    }

    /// 当前帧的 TurnOverBefore 载荷可变访问（面具取消写回）。
    pub fn current_turn_over_before_mut(&mut self) -> (&mut PlayerId, &mut bool) {
        let id = self.current_frame_id();
        self.res
            .frame_mut(id)
            .expect("current frame exists")
            .turn_over_before_mut()
    }

    /// 发射 `TurnOverBefore` 翻面前置帧（经 `mianju` 等取消防守提交翻面）。
    pub fn turn_over(&mut self, player: PlayerId) {
        self.emit(crate::event::EventData::TurnOverBefore {
            player,
            cancelled: false,
        });
    }

    /// 仅在背面朝上时经 `TurnOverBefore` 翻回正面（受面具等翻面前置取消约束）。
    pub fn turn_face_up(&mut self, player: PlayerId) {
        if self.core.player(player).face_down {
            self.emit(crate::event::EventData::TurnOverBefore {
                player,
                cancelled: false,
            });
        }
    }

    /// Damage handlers read their checked root payload while their queue retains the child frame.
    pub fn current_event_frame_id(&self) -> crate::ids::EventId {
        let id = self.current_frame_id();
        match &self.res.frame(id).expect("current frame").data {
            EventData::DamageWindow { root, .. } => *root,
            _ => id,
        }
    }

    /// 当前帧的伤害载荷只读访问（hlss 隐月记录/禳星计数等；非伤害帧 Err）。
    pub fn current_damage(&self) -> Result<&DamageData, ()> {
        let root = self.current_event_frame_id();
        match self.res.frame(root).map(|f| &f.data) {
            Some(crate::event::EventData::Damage(d)) => Ok(d),
            _ => Err(()),
        }
    }

    /// 当前帧的失去体力载荷（hlss 禳星计数；非 LoseHp 帧 Err）。
    pub fn current_lose_hp(&self) -> Result<u32, ()> {
        let id = self.current_frame_id();
        match self.res.frame(id).map(|f| &f.data) {
            Some(crate::event::EventData::LoseHp { amount, .. }) => Ok(*amount),
            _ => Err(()),
        }
    }

    pub fn current_damage_mut(&mut self) -> &mut DamageData {
        let id = self.current_event_frame_id();
        self.res
            .frame_mut(id)
            .expect("current frame exists")
            .damage_mut()
    }

    /// Reject signed source-quantity overflow through the existing atomic command fault path.
    pub fn fail_damage_amount(&mut self) {
        self.res.fault = Some(VmFault {
            skill: "native_damage_modifier".into(),
            owner: 0,
            event: "Damage".into(),
            kind: "DamageAmountOverflow".into(),
            detail: "canonical source amount exceeds i32".into(),
        });
    }
    pub fn modify_damage_amount(&mut self, delta: i32) -> bool {
        let Ok(d) = self.current_damage() else {
            return false;
        };
        let Some(amount) = d.amount.checked_add(delta) else {
            self.fail_damage_amount();
            return false;
        };
        self.current_damage_mut().amount = amount;
        true
    }

    /// Request a durable result for one emitted damage frame; nested damage has a different id.
    pub fn damage_with_receipt(
        &mut self,
        observer: PlayerId,
        damage: DamageData,
    ) -> crate::ids::EventId {
        let id = self.res.next_event_id;
        self.core
            .vm_aux
            .flags
            .insert((observer, format!("vm_damage_receipt:{id}")), 0);
        self.emit(EventData::Damage(damage));
        id
    }

    pub fn record_phase_skipped(&mut self, player: PlayerId) {
        if self.catalog.vm_card_history {
            *self
                .core
                .vm_aux
                .flags
                .entry((player, "vm_skipped_phase_count".to_string()))
                .or_default() += 1;
        }
        (self.catalog.hooks.on_phase_skipped)(self, player);
    }

    /// 压入子事件帧；调用方随后应返回 `Flow::Yield`。
    /// 机器挂起深度在启动时已记录，子事件弹栈后自动恢复。
    /// 伤害帧以「前置触发窗口」为初始阶段，其余帧先提交。
    pub fn emit(&mut self, data: EventData) {
        if let EventData::CardResponded { card, player } = &data {
            self.log(LogEvent::CardResponded {
                card: *card,
                player: *player,
                presentation: None,
            });
        }
        if matches!(
            data,
            EventData::Damage(_)
                | EventData::DamageWindow { .. }
                | EventData::DrawPhase { .. }
                | EventData::Draw { .. }
                | EventData::DrawGain { .. }
                | EventData::TurnEndWindow { .. }
        ) {
            // R05：D/R/G 与伤害族同走 push_frame（起步 Intrinsic）。
            crate::resolution::push_frame(self.res, data);
            return;
        }
        if matches!(data, EventData::CardUsed(_)) {
            crate::resolution::push_card_used_frame(self.core, self.catalog, self.res, data);
            return;
        }
        let id = self.res.next_event_id;
        self.res.next_event_id += 1;
        let parent = self.res.stack.last().map(|f| f.id);
        let kind = data.kind();
        let stage = match kind {
            // 伤害与翻面带前置窗口（伤害前置 / 翻面前置取消），机器发射的
            // 子事件同样必须从 BeforeTriggers 起步——否则取消写回发生在
            // Commit 之后（E03 帧生命周期缺陷根因）。
            crate::event::EventKind::Damage
            | crate::event::EventKind::TurnOverBefore
            | crate::event::EventKind::Death
            | crate::event::EventKind::PhaseBegin
            | crate::event::EventKind::TurnBegin
            | crate::event::EventKind::Recover => {
                // 死亡前置窗口（规则 v170）：机器发射的 Death 同样从
                // BeforeTriggers 起步（遗言/拦截技在 DeathMachine 前）。
                // PhaseBegin 前置窗口（PA12 批⑤-2，§2 K）：阶段开始类
                // 技能（奋励 skip 系）须在入场行为 begin_* 之前执行。
                crate::event::Stage::BeforeTriggers
            }
            _ => crate::event::Stage::Commit,
        };
        self.res.stack.push(crate::event::Frame {
            id,
            parent,
            kind,
            data,
            stage,
            queue: Vec::new(),
            planned: false,
            intrinsic_done: false,
        });
    }

    // ---- 选择 ----

    /// 机器读取已校验的应答。只在 `Flow::WaitChoice` 恢复后存在。
    pub fn answer(&self) -> &Selection {
        self.res.answer.as_ref().expect("answer present on resume")
    }

    pub fn answer_first(&self) -> ChoiceOption {
        *self.answer().first().expect("non-empty selection")
    }

    // ---- 牌区操作 ----

    /// 摸 n 张；牌库不足时先以弃牌堆洗回（规则随机流）。
    /// R05（规则 333）：发射一次真实抽牌（R 帧）供「发射后不需立即读结果」
    /// 的调用方使用——R/G 子树由调度器在当前机器/提交步让出后推进（§3：
    /// caller 设后续 step 并 Yield；本助手不取牌、不入手、不动 RNG）。
    /// `count == 0` 时不发射任何帧（§3：0 无事件无 RNG）。
    pub fn start_draw(&mut self, player: PlayerId, count: u32) {
        if count == 0 {
            return;
        }
        crate::resolution::push_frame(
            self.res,
            EventData::Draw {
                player,
                requested: count,
                held_cards: Vec::new(),
                terminal: crate::event::DrawTerminal::Running,
            },
        );
    }

    /// 翻判定牌：牌库顶；牌库不足时先洗回弃牌堆。不进入任何玩家手牌。
    pub fn draw_judge_card(&mut self) -> CardId {
        if self.core.deck.is_empty() {
            let mut cards = std::mem::take(&mut self.core.discard);
            crate::kernel::cleanup_discard_to_deck(self.core, &mut cards);
            self.core.rng.shuffle(&mut cards);
            crate::kernel::elo_on_wash_deck(self.core, &cards);
            self.core.deck = cards;
            self.out.push(LogEvent::DeckReshuffled);
        }
        self.core
            .deck
            .pop()
            .expect("judge draw: deck or discard non-empty")
    }

    /// 翻判定牌并按明确 `EloJudgePref` 触发 `_eloDraw`（存在偏好时才调序，绝不伪造默认牌种）。
    pub fn draw_judge_card_with_opt_elo(
        &mut self,
        player: PlayerId,
        pref: Option<crate::kernel::EloJudgePref>,
    ) -> CardId {
        if self.core.deck.is_empty() {
            let mut cards = std::mem::take(&mut self.core.discard);
            crate::kernel::cleanup_discard_to_deck(self.core, &mut cards);
            self.core.rng.shuffle(&mut cards);
            crate::kernel::elo_on_wash_deck(self.core, &cards);
            self.core.deck = cards;
            self.out.push(LogEvent::DeckReshuffled);
        }
        if let Some(pref) = pref {
            crate::kernel::elo_on_skill_judge_begin(self.core, self.catalog, player, pref);
        }
        self.core
            .deck
            .pop()
            .expect("judge draw: deck or discard non-empty")
    }

    /// 翻判定牌并按指定 `EloJudgePref` 触发 `_eloDraw`。
    pub fn draw_judge_card_with_elo(
        &mut self,
        player: PlayerId,
        pref: crate::kernel::EloJudgePref,
    ) -> CardId {
        self.draw_judge_card_with_opt_elo(player, Some(pref))
    }

    /// 从手牌或装备区移除一张（鬼道改判用：装备区黑牌可打出）。
    /// 不存在则 panic：调用方先验证。
    pub fn take_owned_card(&mut self, player: PlayerId, card: CardId) {
        let is_equip = self.catalog.resolved_card_kind(self.core, card).is_equip();
        let in_hand = self
            .core
            .player(player)
            .hand
            .iter()
            .position(|c| *c == card);
        if let Some(_pos) = in_hand {
            self.take_hand_card(player, card);
            self.note_cards_lost(player, &[card]);
            self.emit(EventData::CardsLost {
                player,
                cards: vec![card],
                origins: vec![crate::event::LostOrigin::Hand],
            });
            return;
        }
        let in_equips = self
            .core
            .player(player)
            .equips
            .iter()
            .position(|c| *c == card);
        if let Some(pos) = in_equips {
            self.core.player_mut(player).equips.remove(pos);
            if is_equip {
                crate::kernel::revoke_equip_skills(self.core, self.catalog, player, card);
            }
            self.note_cards_lost(player, &[card]);
            self.emit(EventData::CardsLost {
                player,
                cards: vec![card],
                origins: vec![crate::event::LostOrigin::Equip],
            });
            return;
        }
        let in_judge = self
            .core
            .player(player)
            .judge
            .iter()
            .position(|c| *c == card);
        if let Some(pos) = in_judge {
            self.core.player_mut(player).judge.remove(pos);
            self.note_cards_lost(player, &[card]);
            self.emit(EventData::CardsLost {
                player,
                cards: vec![card],
                origins: vec![crate::event::LostOrigin::Judge],
            });
            return;
        }
        panic!("card {card} not in hand, equips, or judge of {player}");
    }

    /// 从手牌移除一张（不存在则 panic：调用方先验证）。
    pub fn take_hand_card(&mut self, player: PlayerId, card: CardId) {
        let hand = &mut self.core.player_mut(player).hand;
        let pos = hand
            .iter()
            .position(|c| *c == card)
            .unwrap_or_else(|| panic!("card {card} not in hand of {player}"));
        hand.remove(pos);
        (self.catalog.hooks.on_hand_card_removed)(self, player, card);
        if self
            .catalog
            .skill_by_name("dddshilie")
            .is_some_and(|def| self.core.has_skill(player, def))
        {
            self.core
                .vm_aux
                .flags
                .remove(&(player, format!("dddshilie_shown_card:{card}")));
        }
        self.core
            .vm_aux
            .flags
            .remove(&(player, format!("u01_shown_card:{card}")));
    }

    /// 入弃牌堆（**牌区唯一性**：同一张牌不得同时属于两个区域）。
    ///
    /// NC07 修正（NC12 基线 §5.1 缺陷根因）：旧实现是无条件 `push`，当技能在
    /// 一张牌「正在被使用/结算」期间把它换出（如 swd 玉蝉换形）或某条路径重复
    /// 入堆时，同一张牌会在弃牌堆出现两次——引擎允许的活状态随即被自己的所有权
    /// 校验（`card_in_multiple_zones`）判非法，读档 30/30 被拒。现在：
    /// - 已在弃牌堆 → 幂等返回（不重复入堆）；
    /// - 从其它区域移出仍由调用方负责（`discard_from_hand`/`take_hand_card` 等），
    ///   本原语只保证**不重复入堆**——不改变既有调用方的区域语义。
    pub fn to_discard(&mut self, card: impl Into<Option<CardId>>) {
        let Some(card) = card.into() else { return };
        if self.core.discard.contains(&card)
            || self.core.detached_cards.iter().any(|(c, _)| *c == card)
            || self.core.players.iter().any(|p| {
                p.hand.contains(&card)
                    || p.equips.contains(&card)
                    || p.judge.contains(&card)
                    || p.off_court.contains(&card)
            })
            || self.core.vm_aux.piles.values().any(|v| v.contains(&card))
        {
            return;
        }
        let is_stj = |c: CardId| {
            matches!(
                self.catalog.card_kind(c),
                crate::catalog::CardKind::Shan
                    | crate::catalog::CardKind::Tao
                    | crate::catalog::CardKind::Jiu
            )
        };
        let xiangsicunwei = self
            .core
            .vm_aux
            .flags
            .iter()
            .any(|((_, k), &v)| k == "mbjuejin_xiangsicunwei" && v > 0);
        if xiangsicunwei {
            let mut swept: Vec<CardId> = Vec::new();
            self.core.discard.retain(|&c| {
                if is_stj(c) {
                    swept.push(c);
                    false
                } else {
                    true
                }
            });
            for c in swept {
                if !self.core.detached_cards.iter().any(|(dc, _)| *dc == c) {
                    self.core
                        .detached_cards
                        .push((c, crate::event::DetachedKind::Destroyed));
                }
            }
            if is_stj(card) {
                if !self.core.detached_cards.iter().any(|(dc, _)| *dc == card) {
                    self.core
                        .detached_cards
                        .push((card, crate::event::DetachedKind::Destroyed));
                }
                return;
            }
        }
        if self
            .catalog
            .cards
            .get(card as usize)
            .is_some_and(|cd| cd.name == "txhj_piliche")
            || (self.catalog.hooks.is_hearth_vanish_card)(self.catalog, card)
        {
            if !self.core.detached_cards.iter().any(|(dc, _)| *dc == card) {
                self.core
                    .detached_cards
                    .push((card, crate::event::DetachedKind::Destroyed));
            }
            return;
        }
        self.core.discard.push(card);
        if self.catalog.vm_card_history {
            self.core
                .vm_aux
                .flags
                .insert((0, format!("turn_discard_card:{card}")), 1);
        }
        crate::kernel::release_yanjia_merged_to_discard(self.core, card);
        crate::rvm_bridge::modifiers::d34_l28_mods::note_pro_qinyin_discard(
            self.core,
            self.catalog,
            card,
        );
        let has_yulv = self
            .catalog
            .skill_by_name("dddyulv")
            .is_some_and(|d| self.core.skills.keys().any(|(_, sd)| *sd == d))
            || self
                .catalog
                .skill_by_name("dddyulv_old")
                .is_some_and(|d| self.core.skills.keys().any(|(_, sd)| *sd == d));
        if has_yulv && self.core.turn.turn_opened {
            let key = (0, "u01_yulv_q_len".to_string());
            let idx = self.core.vm_aux.flags.get(&key).copied().unwrap_or(0);
            self.core
                .vm_aux
                .flags
                .insert((0, format!("u01_yulv_q:{idx}")), card);
            self.core.vm_aux.flags.insert(key, idx + 1);
        }
    }

    pub fn discard_from_hand(&mut self, player: PlayerId, card: CardId) {
        self.take_hand_card(player, card);
        self.to_discard(card);
        self.note_cards_lost(player, &[card]);
        if self.catalog.vm_card_history
            && !self
                .core
                .vm_aux
                .flags
                .contains_key(&(player, "in_discard_to_limit".to_string()))
        {
            self.core
                .vm_aux
                .flags
                .insert((player, format!("u01_discard_reason:{card}")), 1);
        }
        if self.catalog.skill_by_name("yanshu").is_some_and(|d| {
            self.catalog.skills[d as usize]
                .behavior
                .as_host_behavior()
                == Some(crate::vocabulary::host::HostBehavior::U05Hearth2BatchD)
        }) {
            // LIFO stack runs `CardsLost` first, then `CardsDiscarded` (matching PhaseDiscard / `discardAfter`).
            self.emit(EventData::CardsDiscarded {
                player,
                cards: vec![card],
            });
        }
        self.emit(EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Hand],
        });
        self.out.push(LogEvent::CardDiscarded {
            player,
            cards: vec![card],
        });
    }

    /// 义山记录（swd 批 123，规则 v310）：持有者失去牌时登记（上游
    /// `yishan2` loseEnd `storage.yishan.unshift` 同口径，新在前）。
    /// 覆盖原生三类丢失路径：弃置/交出/装备离场（登记差异：其余丢失路径
    /// 如移出游戏不记）。
    pub fn note_cards_lost(&mut self, player: PlayerId, cards: &[CardId]) {
        note_cards_lost(self.core, self.catalog, player, cards);
    }

    /// 手牌置武将牌上（离场区，规则 v162 基建）：实体牌守恒——离开全部
    /// 正常区、随所有者存放（死亡清理由 DeathMachine leftover 之外的
    /// off_court 消费技自理，此处不做）。
    pub fn hand_to_off_court(&mut self, player: PlayerId, card: CardId) {
        self.core.player_mut(player).off_court.push(card);
        self.take_hand_card(player, card);
    }

    /// 离场区牌弃置（消费技用）：出区入弃堆。
    pub fn off_court_discard(&mut self, player: PlayerId, card: CardId) {
        self.core
            .player_mut(player)
            .off_court
            .retain(|x| *x != card);
        self.to_discard(card);
    }

    /// 手牌直接给予另一玩家（不进弃牌堆）——单张便捷形式，见 `give_hand_cards`。
    pub fn give_hand_card(&mut self, from: PlayerId, to: PlayerId, card: CardId) {
        self.give_hand_cards(from, to, &[card]);
    }

    /// 一次给牌动作：`cards` 全部由 `from` 交给 `to`（不进弃牌堆）。
    ///
    /// `CardsLost`／`CardsMoved` 仍逐张发帧（既有口径；连营等失去牌监听不变）。
    /// `CardsGiven` 触发帧（DSL `cards_given`）**每次给牌动作只发一个**并携带
    /// 全部牌（规则 v326，裁决第 10 项）：事件主玩家=recipient、来源=giver、
    /// amount=张数，与旧线 `give_cards` 单事件同口径。一次给出多张的技能
    /// （仁德、蝶舞）须调用本方法，而不是按张循环 `give_hand_card`。
    pub fn give_hand_cards(&mut self, from: PlayerId, to: PlayerId, cards: &[CardId]) {
        if from == to || cards.is_empty() {
            return;
        }
        let delivered = (self.catalog.hooks.on_before_gain_hand)(self, Some(from), to, cards);
        for &card in &delivered {
            self.core.player_mut(to).hand.push(card);
        }
        for &card in cards {
            self.take_hand_card(from, card);
            self.note_cards_lost(from, &[card]);
            self.emit(EventData::CardsLost {
                player: from,
                cards: vec![card],
                origins: vec![crate::event::LostOrigin::Hand],
            });
        }
        for &card in &delivered {
            self.emit(EventData::CardsMoved {
                cards: vec![card],
                from,
                to,
            });
        }
        if !delivered.is_empty() {
            self.emit(EventData::CardsGiven {
                giver: from,
                recipient: to,
                cards: delivered.clone(),
            });
            self.out.push(LogEvent::CardsGiven {
                from,
                to,
                cards: delivered,
            });
        }
    }

    // ---- 生命周期 ----

    /// 回复体力（封顶 max_hp）。返回实际回复量。
    /// 护甲增减（G1 扩展）：下限 0；正数获得、负数失去。
    /// 海岛地图效果（mtg 第二批，规则 v101）：锁定、全局（上游
    /// addGlobalSkill 同口径）——current_land 为海岛时，目标获得护甲
    /// 若已受伤改为回复等量体力（失去护甲不触发）。海岛自身受伤分支
    /// 的取消路线为上游 _temp 标记口径（路径级抑制），由调用方绕过。
    pub fn change_hujia(&mut self, player: PlayerId, delta: i32) -> u8 {
        let p = self.core.player(player);
        if delta > 0
            && self.core.current_land == Some(crate::catalog::CardKind::Haidao)
            && p.alive
            && p.hp < p.max_hp as i32
        {
            self.recover(player, delta as u32, None);
            return self.core.player(player).hujia;
        }
        let cur = self.core.player(player).hujia as i32;
        let next = (cur + delta).max(0) as u8;
        self.core.player_mut(player).hujia = next;
        next
    }

    /// 体力上限减少（max_hp 减面，运筹武将面批 9）：上限 -n 并钳体力
    ///（hp>max 时收齐）；不入 Recover 事件流（无 MaxHpChanged 事件种
    /// ——新事件种九处接线成本，无消费者，登记边界）。
    pub fn lose_max_hp(&mut self, player: PlayerId, amount: u32) {
        let p = self.core.player_mut(player);
        // 无限体力（上游 Infinity）：体力上限不减（Infinity - n = Infinity）。
        if p.infinite_hp() {
            self.out.push(LogEvent::MaxHpLost { player, amount: 0 });
            return;
        }
        let cut = (p.max_hp.min(amount)) as i32;
        p.max_hp -= cut as u32;
        if p.hp > p.max_hp as i32 {
            p.hp = p.max_hp as i32;
        }
        self.out.push(LogEvent::MaxHpLost {
            player,
            amount: cut as u32,
        });
    }

    pub fn recover(&mut self, player: PlayerId, amount: u32, by_card: Option<CardId>) -> u32 {
        if self.catalog.has_diaohulishan(self.core, player)
            || crate::rvm_bridge::modifiers::has_yaya_protected(self.core, self.catalog, player)
        {
            return 0;
        }
        if !(self.catalog.hooks.before_recover)(self, player) {
            return 0;
        }
        let mut amount = amount;
        (self.catalog.hooks.on_recover_before)(self, player, &mut amount);
        let Some(amount) = helpers::apply_guanfu_disrecover(self, player, amount) else {
            return 0;
        };
        let p = self.core.player_mut(player);
        if !p.alive || p.infinite_hp() {
            return 0;
        }
        let before = p.hp;
        let after = before.saturating_add(amount as i32).min(p.max_hp as i32);
        let gained = (after - before).max(0) as u32;
        if gained > 0 {
            p.hp = after;
            if p.hp > 0 {
                p.dying = false;
            }
            self.emit(EventData::Recover {
                player,
                amount: gained,
                by_card,
            });
            if let Some(fid) = self.res.stack.last().map(|f| f.id) {
                self.core
                    .vm_aux
                    .flags
                    .insert((0, format!("u01_recover_preapplied:{fid}")), gained);
                if self.catalog.vm_card_history {
                    crate::resolution::note_recover_source(self.core, self.res, player, fid);
                }
            }
        }
        gained
    }

    pub fn cancel_draw_phase(&mut self, root: crate::ids::EventId) -> bool {
        if let Some(frame) = self.res.frame_mut(root) {
            if let EventData::DrawPhase {
                pending,
                outcome,
                ..
            } = &mut frame.data
            {
                *pending = 0;
                *outcome = crate::event::DrawOutcome::Cancelled;
                return true;
            }
        }
        false
    }

    // ---- 日志 ----

    pub fn log(&mut self, mut event: LogEvent) {
        presentation::attach_occurrence_presentation(
            self.core,
            self.res,
            self.catalog,
            &mut event,
        );
        self.out.push(event);
    }

    pub fn log_skill_triggered(&mut self, skill: &'static str, owner: PlayerId, def: SkillDefId) {
        let mut event = LogEvent::SkillTriggered {
            skill,
            owner,
            presentation: None,
        };
        presentation::attach_occurrence_presentation_with_skill_def(
            self.core,
            self.res,
            self.catalog,
            Some(def),
            &mut event,
        );
        self.out.push(event);
    }
}
