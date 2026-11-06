//! Package active entry constraints and trigger handlers sharing one logical skill.
use super::modifiers::VmModifierRule;
use super::*;
use crate::catalog::{Catalog, EquipSlot, Suit, TriggerMeta};
use crate::state::{GameMode, GameStateCore, Role};
use crate::vocabulary::generals::Gender;

/// 主动技每回合可用次数动态来源（对齐 `content-schema::PackageSkillEntry::uses_per_turn_source`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmTurnUseSource {
    OwnerHp,
    OwnerLostHpMinOne,
    LivingRebelCount,
}

impl VmTurnUseSource {
    pub fn evaluate(self, state: &GameStateCore, owner: PlayerId) -> usize {
        let p = state.player(owner);
        match self {
            Self::OwnerHp => p.hp.max(0) as usize,
            Self::OwnerLostHpMinOne => (p.max_hp as i32 - p.hp).max(1) as usize,
            Self::LivingRebelCount => match &state.mode {
                GameMode::Identity { roles } => roles
                    .iter()
                    .enumerate()
                    .filter(|(idx, r)| **r == Role::Rebel && state.player(*idx as u8).alive)
                    .count(),
                _ => state.alive_players().filter(|&q| q != owner).count(),
            },
        }
    }
}

/// 主动技选牌区域（对齐 `content-schema::PackageSelectionSpec::zone`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmSelectionZone {
    Hand,
    HandEquip,
}

/// 主动技选牌过滤器（对齐 `content-schema::PackageSelectionSpec::filter`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmSelectionCardFilter {
    CardEquip,
    CardArmor,
    CardSha,
    CardBasic,
    CardNonBasic,
    CardNonEquip,
    CardRed,
    CardBlack,
    CardSpade,
    CardHeart,
    CardClub,
    CardDiamond,
}

impl VmSelectionCardFilter {
    pub fn matches(self, catalog: &Catalog, card: CardId) -> bool {
        let Some(def) = catalog.cards.get(card as usize) else {
            return false;
        };
        let kind = def.kind;
        let suit = def.suit;
        match self {
            Self::CardEquip => kind.is_equip(),
            Self::CardArmor => kind.equip_slot() == Some(EquipSlot::Armor),
            Self::CardSha => kind.is_sha_like(),
            Self::CardBasic => kind.is_basic_kind(),
            Self::CardNonBasic => !kind.is_basic_kind(),
            Self::CardNonEquip => !kind.is_equip(),
            Self::CardRed => matches!(suit, Suit::Heart | Suit::Diamond),
            Self::CardBlack => matches!(suit, Suit::Spade | Suit::Club),
            Self::CardSpade => suit == Suit::Spade,
            Self::CardHeart => suit == Suit::Heart,
            Self::CardClub => suit == Suit::Club,
            Self::CardDiamond => suit == Suit::Diamond,
        }
    }
}

/// 主动技选牌规格（对齐 `content-schema::PackageSelectionSpec`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmActiveSelection {
    pub min_cards: u8,
    pub max_cards: u8,
    pub zone: VmSelectionZone,
    pub filter: Option<VmSelectionCardFilter>,
    pub unique_category_in_turn: bool,
}

impl VmActiveSelection {
    pub fn card_category_bit(catalog: &Catalog, card: CardId) -> u32 {
        let Some(def) = catalog.cards.get(card as usize) else {
            return 0;
        };
        let k = def.kind;
        if k.is_equip() {
            4
        } else if k.is_basic_kind() {
            1
        } else {
            2
        }
    }

    pub fn candidate_cards(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_name: &str,
    ) -> Vec<CardId> {
        let p = state.player(owner);
        let used_mask = if self.unique_category_in_turn {
            state
                .vm_aux
                .flags
                .get(&(owner, format!("unique_cat_mask:{skill_name}")))
                .or_else(|| state.vm_aux.flags.get(&(owner, "unique_cat_mask".to_string())))
                .copied()
                .unwrap_or(0)
        } else {
            0
        };
        let mut out = Vec::new();
        let iter_equips = matches!(self.zone, VmSelectionZone::HandEquip);
        for &c in p
            .hand
            .iter()
            .chain(if iter_equips { p.equips.as_slice() } else { &[] })
        {
            if !self.filter.map_or(true, |f| f.matches(catalog, c)) {
                continue;
            }
            if self.unique_category_in_turn && (Self::card_category_bit(catalog, c) & used_mask) != 0
            {
                continue;
            }
            out.push(c);
        }
        out
    }
}

/// 主动技选目标过滤器（对齐 `content-schema::PackageTargetSpec::filter`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmTargetFilter {
    pub other_only: bool,
    pub wounded_only: bool,
    pub hand_nonempty: bool,
    pub equip_nonempty: bool,
    pub hand_equip_nonempty: bool,
    pub hand_equip_judge_nonempty: bool,
    pub within_attack_range: bool,
    pub hand_gt_owner: bool,
    pub hand_ge_owner: bool,
    pub hand_le_owner: bool,
    pub hp_ge_owner: bool,
    pub hp_gt_zero: bool,
    pub max_hp_gt: Option<u8>,
    pub same_group_with_owner: bool,
    pub different_group_from_owner: bool,
    pub male_only: bool,
    pub female_only: bool,
    pub linked_only: bool,
    pub unlinked_only: bool,
    pub unique_in_turn: bool,
    pub unique_in_play_phase: bool,
}

impl VmTargetFilter {
    pub fn matches(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        target: PlayerId,
        skill_name: &str,
    ) -> bool {
        let tp = state.player(target);
        if !tp.alive {
            return false;
        }
        if self.other_only && target == owner {
            return false;
        }
        let op = state.player(owner);
        if self.wounded_only && tp.hp >= tp.max_hp as i32 {
            return false;
        }
        if self.hand_nonempty && tp.hand.is_empty() {
            return false;
        }
        if self.equip_nonempty && tp.equips.is_empty() {
            return false;
        }
        if self.hand_equip_nonempty && tp.hand.is_empty() && tp.equips.is_empty() {
            return false;
        }
        if self.hand_equip_judge_nonempty
            && tp.hand.is_empty()
            && tp.equips.is_empty()
            && tp.judge.is_empty()
        {
            return false;
        }
        if self.within_attack_range
            && (target == owner
                || state.computed_distance(catalog, owner, target) > state.attack_range(catalog, owner))
        {
            return false;
        }
        if self.hand_gt_owner && tp.hand.len() <= op.hand.len() {
            return false;
        }
        if self.hand_ge_owner && tp.hand.len() < op.hand.len() {
            return false;
        }
        if self.hand_le_owner && tp.hand.len() > op.hand.len() {
            return false;
        }
        if self.hp_ge_owner && tp.hp < op.hp {
            return false;
        }
        if self.hp_gt_zero && tp.hp <= 0 {
            return false;
        }
        if let Some(min_ex) = self.max_hp_gt {
            if tp.max_hp <= u32::from(min_ex) {
                return false;
            }
        }
        if self.same_group_with_owner
            && state.effective_faction(catalog, owner) != state.effective_faction(catalog, target)
        {
            return false;
        }
        if self.different_group_from_owner
            && state.effective_faction(catalog, owner) == state.effective_faction(catalog, target)
        {
            return false;
        }
        if self.male_only && state.effective_gender(target) != Gender::Male {
            return false;
        }
        if self.female_only && state.effective_gender(target) != Gender::Female {
            return false;
        }
        if self.linked_only && !tp.linked {
            return false;
        }
        if self.unlinked_only && tp.linked {
            return false;
        }
        let target_raw = u32::from(target) + 1;
        if self.unique_in_turn {
            let rec = state
                .vm_aux
                .flags
                .get(&(owner, format!("unique_cat_target:{skill_name}")))
                .or_else(|| state.vm_aux.flags.get(&(owner, "unique_cat_target".to_string())))
                .copied()
                .unwrap_or(0);
            if rec == target_raw
                || state
                    .vm_aux
                    .once
                    .contains(&(owner, format!("active_target:{skill_name}:{target}"), 1))
            {
                return false;
            }
        }
        if self.unique_in_play_phase {
            let rec = state
                .vm_aux
                .flags
                .get(&(owner, format!("active_target:{skill_name}")))
                .or_else(|| state.vm_aux.flags.get(&(owner, "active_target".to_string())))
                .copied()
                .unwrap_or(0);
            if rec == target_raw
                || state
                    .vm_aux
                    .once
                    .contains(&(owner, format!("active_target:{skill_name}:{target}"), 4))
            {
                return false;
            }
        }
        true
    }
}

/// 主动技选目标规格（对齐 `content-schema::PackageTargetSpec`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmActiveTargets {
    pub min_targets: u8,
    pub max_targets: u8,
    pub filter: VmTargetFilter,
}

impl VmActiveTargets {
    pub fn candidate_targets(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_name: &str,
    ) -> Vec<PlayerId> {
        let n = state.players.len();
        let mut out = Vec::new();
        for step in 0..n {
            let q = ((owner as usize + step) % n) as PlayerId;
            if self.filter.matches(state, catalog, owner, q, skill_name) {
                out.push(q);
            }
        }
        out
    }
}

/// 可选触发的确认决策角色（对齐 `content-schema::PackageTriggerEntry::choice_actor`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VmChoiceActor {
    #[default]
    Owner,
    EventPlayer,
    EventSource,
}

impl VmChoiceActor {
    pub fn resolve(self, owner: PlayerId, event: &EventData) -> PlayerId {
        match self {
            Self::Owner => owner,
            Self::EventPlayer => trigger_ctx_of(event).event_player.unwrap_or(owner),
            Self::EventSource => trigger_ctx_of(event).event_source.unwrap_or(owner),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VmActiveLimits {
    pub once_per_turn: bool,
    pub once_per_play_phase: bool,
    pub once_per_game: bool,
    pub uses_per_turn: Option<u8>,
    pub uses_per_play_phase: Option<u8>,
    pub uses_per_turn_source: Option<VmTurnUseSource>,
    pub hand_nonempty: bool,
    pub equip_nonempty: bool,
    pub hand_gt_hp: bool,
    pub hand_gt_max_hp: bool,
    pub face_up: bool,
    pub owner_unlinked: bool,
    pub mark_at_least: Option<(u32, u32)>,
    pub target_other: bool,
    pub target_max_hp_gt: Option<u8>,
    pub selection: Option<VmActiveSelection>,
    pub targets: Option<VmActiveTargets>,
}

impl VmActiveLimits {
    pub fn record_active_halt(
        &self,
        core: &mut GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        skill_name: &str,
        selected_cards: &[u32],
        selected_targets: &[u32],
        seats: u8,
    ) {
        if let Some(sel) = self.selection {
            if sel.unique_category_in_turn {
                let key = (owner, format!("unique_cat_mask:{skill_name}"));
                let mut mask = core.vm_aux.flags.get(&key).copied().unwrap_or(0);
                for &cid in selected_cards {
                    mask |= VmActiveSelection::card_category_bit(catalog, cid);
                }
                core.vm_aux.flags.insert(key, mask);
            }
        }
        if let Some(tgts) = self.targets {
            for &raw in selected_targets {
                if let Some(q) = crate::rvm_ops::seat_of(rule_api::PlayerId::new(raw), seats) {
                    if tgts.filter.unique_in_turn {
                        core.vm_aux
                            .once
                            .insert((owner, format!("active_target:{skill_name}:{q}"), 1));
                    }
                    if tgts.filter.unique_in_play_phase {
                        core.vm_aux
                            .once
                            .insert((owner, format!("active_target:{skill_name}:{q}"), 4));
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmTriggerProgram {
    pub program: Arc<VmSkillProgram>,
    pub meta: TriggerMeta,
    pub filter: VmFilterPolicy,
    pub choice_actor: VmChoiceActor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmHandlerSet {
    pub skill_def_id: u32,
    pub active: Option<Arc<VmSkillProgram>>,
    pub limits: VmActiveLimits,
    pub triggers: Vec<VmTriggerProgram>,
    pub modifiers: Vec<VmModifierRule>,
}

impl VmHandlerSet {
    pub(super) fn available(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
        let p = ctx.state.player(owner);
        let Some(active) = &self.active else {
            return false;
        };
        let name = &active.skill_name;
        p.alive
            && ctx.state.turn.current == owner
            && ctx.state.turn.phase == crate::event::Phase::Play
            && ctx
                .state
                .vm_aux
                .flags
                .get(&(owner, format!("skill_banned_turn:{name}")))
                .copied()
                .unwrap_or(0)
                == 0
            && (!self.limits.once_per_game || ctx.state.vm_aux.once_count(owner, 3, name) == 0)
            && (!self.limits.once_per_turn || ctx.state.vm_aux.once_count(owner, 1, name) == 0)
            && (!self.limits.once_per_play_phase
                || ctx.state.vm_aux.once_count(owner, 4, name) == 0)
            && self
                .limits
                .uses_per_turn
                .is_none_or(|max| ctx.state.vm_aux.once_count(owner, 1, name) < max as usize)
            && self
                .limits
                .uses_per_play_phase
                .is_none_or(|max| ctx.state.vm_aux.once_count(owner, 4, name) < max as usize)
            && self.limits.uses_per_turn_source.is_none_or(|src| {
                let cap = src.evaluate(ctx.state, owner);
                cap > 0 && ctx.state.vm_aux.once_count(owner, 1, name) < cap
            })
            && (!self.limits.hand_nonempty || !p.hand.is_empty())
            && (!self.limits.equip_nonempty || !p.equips.is_empty())
            && (!self.limits.hand_gt_hp || p.hand.len() as i64 > p.hp as i64)
            && (!self.limits.hand_gt_max_hp || p.hand.len() > p.max_hp as usize)
            && (!self.limits.face_up || !p.face_down)
            && (!self.limits.owner_unlinked || !p.linked)
            && (!self.limits.target_other
                || ctx.state.players.iter().enumerate().any(|(idx, q)| {
                    idx as u8 != owner
                        && q.alive
                        && self
                            .limits
                            .target_max_hp_gt
                            .is_none_or(|min_exclusive| q.max_hp > min_exclusive as u32)
                }))
            && self.limits.mark_at_least.is_none_or(|(mark, count)| {
                ctx.state
                    .vm_aux
                    .marks
                    .get(&(owner, mark))
                    .copied()
                    .unwrap_or(0) as i64
                    >= count as i64
            })
            && self.limits.selection.is_none_or(|sel| {
                sel.candidate_cards(ctx.state, ctx.catalog, owner, name)
                    .len()
                    >= sel.min_cards as usize
            })
            && self.limits.targets.is_none_or(|tgts| {
                tgts.candidate_targets(ctx.state, ctx.catalog, owner, name)
                    .len()
                    >= tgts.min_targets as usize
            })
    }

    pub(super) fn matching<'a>(
        &'a self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> Vec<&'a VmTriggerProgram> {
        self.triggers
            .iter()
            .filter(|entry| {
                (entry.meta.event == kind || entry.meta.extra_events.contains(&kind))
                    && VmSkillBehavior {
                        program: entry.program.clone(),
                        filter: entry.filter.clone(),
                    }
                    .filter(ctx, owner, event)
            })
            .collect()
    }
}

pub(super) fn machine(
    program: Arc<VmSkillProgram>,
    filter: &VmFilterPolicy,
    owner: PlayerId,
    event: &EventData,
) -> VmMachine {
    fn named_meta(f: &VmFilterPolicy, skill_name: &str) -> (Vec<(String, u8)>, bool) {
        match f {
            VmFilterPolicy::Named(p) => (p.consume.clone(), p.count_hits),
            VmFilterPolicy::GameStarted { inner } => {
                let (mut c, h) = named_meta(inner, skill_name);
                c.push((format!("game_started:{skill_name}"), 3));
                (c, h)
            }
            VmFilterPolicy::Handlers(set) => {
                let mut c = Vec::new();
                if set.limits.once_per_game {
                    c.push((String::new(), 3));
                }
                if set.limits.once_per_turn
                    || set.limits.uses_per_turn.is_some()
                    || set.limits.uses_per_turn_source.is_some()
                {
                    c.push((String::new(), 1));
                }
                if set.limits.once_per_play_phase || set.limits.uses_per_play_phase.is_some() {
                    c.push((String::new(), 4));
                }
                if set.limits.target_other {
                    c.push((
                        "__target_other".to_string(),
                        set.limits.target_max_hp_gt.unwrap_or(0),
                    ));
                }
                (c, false)
            }
            VmFilterPolicy::AtPhase { inner, .. }
            | VmFilterPolicy::DrawPhaseSkipped { inner }
            | VmFilterPolicy::PhaseAnySkipped { inner }
            | VmFilterPolicy::Declared { inner, .. } => named_meta(inner, skill_name),
            _ => (Vec::new(), false),
        }
    }
    let (consume, count_hits) = named_meta(filter, &program.skill_name);
    VmMachine {
        owner,
        program,
        trig: trigger_ctx_of(event),
        event: event.clone(),
        pending_effects: Vec::new(),
        paused: None,
        asked: false,
        settled: false,
        consume,
        count_hits,
        commit_pause: None,
        following: Vec::new(),
        commit_state: Vec::new(),
        committing: false,
    }
}
