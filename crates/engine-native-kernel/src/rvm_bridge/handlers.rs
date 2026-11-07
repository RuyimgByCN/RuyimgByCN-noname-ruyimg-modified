//! Package active entry constraints and trigger handlers sharing one logical skill.
use super::modifiers::VmModifierRule;
use super::*;
use crate::catalog::{Catalog, EquipSlot, Suit, TriggerMeta};
use crate::state::GameStateCore;

mod active_specs;
pub use active_specs::*;


fn fnv1a_32_str(s: &str) -> u32 {
    let mut hash = 0x811c_9dc5u32;
    for &b in s.as_bytes() {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

fn owner_has_skill_hash(state: &GameStateCore, catalog: &Catalog, owner: PlayerId, target_hash: u32) -> bool {
    if state
        .vm_aux
        .flags
        .get(&(owner, format!("skill:{target_hash}")))
        .copied()
        .unwrap_or(0)
        != 0
    {
        return true;
    }
    state.skills.iter().any(|((p, def), inst)| {
        *p == owner
            && inst.enabled
            && (*def as u32 == target_hash
                || catalog
                    .skills
                    .get(*def as usize)
                    .is_some_and(|d| {
                        d.behavior.package_skill_id() == Some(target_hash)
                            || fnv1a_32_str(d.name) == target_hash
                    }))
    })
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
    pub require_missing_skill: u32,
    pub require_hand_limit_positive: bool,
    pub require_hand_missing_gaintag: u32,
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
            if sel.unused_category_by_skill_this_play_phase {
                for &cid in selected_cards {
                    let bit = VmActiveSelection::card_category_bit(catalog, cid);
                    if bit != 0 {
                        core.vm_aux
                            .once
                            .insert((owner, format!("unique_cat_play:{skill_name}:{bit}"), 4));
                    }
                }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsWindow {
    PlayPhase,
    Response,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsZone {
    Hand,
    Equip,
    HandEquip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmViewAsFilter {
    Any,
    Red,
    Black,
    Spade,
    Heart,
    Club,
    Diamond,
    OwnerDyingClub,
    OwnerOutsideTurn,
    OwnerOutsideTurnRed,
    OwnerSingleHand,
    OwnerSingleRedHand,
    OwnerHandGtHp,
    OwnerHpLeTwoBlackTrick,
    Basic,
    NonBasic,
    Equip,
    NonEquip,
    Trick,
    DamageTrick,
    Sha,
    Shan,
    Tao,
    Jiu,
    Weapon,
    Armor,
    Mount,
    BlackBasicEquip,
    RedBasic,
    BlackBasic,
    RedTrick,
    BlackTrick,
    RedNonBasic,
    BlackNonBasic,
}

impl VmViewAsFilter {
    pub fn matches(
        self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        card: CardId,
    ) -> bool {
        let Some(def) = catalog.cards.get(card as usize) else {
            return false;
        };
        let kind = def.kind;
        let suit = def.suit;
        let is_red = matches!(suit, Suit::Heart | Suit::Diamond);
        let is_black = matches!(suit, Suit::Spade | Suit::Club);
        let is_trick = !kind.is_basic_kind() && !kind.is_equip();
        let p = state.player(owner);
        match self {
            Self::Any => true,
            Self::Red => is_red,
            Self::Black => is_black,
            Self::Spade => suit == Suit::Spade,
            Self::Heart => suit == Suit::Heart,
            Self::Club => suit == Suit::Club,
            Self::Diamond => suit == Suit::Diamond,
            Self::OwnerDyingClub => p.hp <= 0 && suit == Suit::Club,
            Self::OwnerOutsideTurn => state.turn.current != owner,
            Self::OwnerOutsideTurnRed => state.turn.current != owner && is_red,
            Self::OwnerSingleHand => p.hand.len() == 1,
            Self::OwnerSingleRedHand => {
                is_red
                    && p.hand
                        .iter()
                        .filter(|&&c| {
                            catalog
                                .cards
                                .get(c as usize)
                                .is_some_and(|d| matches!(d.suit, Suit::Heart | Suit::Diamond))
                        })
                        .count()
                        == 1
            }
            Self::OwnerHandGtHp => (p.hand.len() as i32) > p.hp,
            Self::OwnerHpLeTwoBlackTrick => p.hp <= 2 && is_black && is_trick,
            Self::Basic => kind.is_basic_kind(),
            Self::NonBasic => !kind.is_basic_kind(),
            Self::Equip => kind.is_equip(),
            Self::NonEquip => !kind.is_equip(),
            Self::Trick => is_trick,
            Self::DamageTrick => matches!(
                kind,
                crate::catalog::CardKind::SavageAttack
                    | crate::catalog::CardKind::ArcheryAttack
                    | crate::catalog::CardKind::Duel
                    | crate::catalog::CardKind::ShanDian
                    | crate::catalog::CardKind::HongShui
                    | crate::catalog::CardKind::Linghunzhihuo
                    | crate::catalog::CardKind::Shenhuofeiya
                    | crate::catalog::CardKind::Shatang
                    | crate::catalog::CardKind::Yuansuhuimie
                    | crate::catalog::CardKind::Aozuzhilei
                    | crate::catalog::CardKind::Liuxinghuoyu
                    | crate::catalog::CardKind::Jingleishan
                    | crate::catalog::CardKind::Chiyuxi
            ),
            Self::Sha => kind.is_sha_like(),
            Self::Shan => kind == crate::catalog::CardKind::Shan,
            Self::Tao => kind == crate::catalog::CardKind::Tao,
            Self::Jiu => kind == crate::catalog::CardKind::Jiu,
            Self::Weapon => kind.equip_slot() == Some(EquipSlot::Weapon),
            Self::Armor => kind.equip_slot() == Some(EquipSlot::Armor),
            Self::Mount => matches!(
                kind.equip_slot(),
                Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
            ),
            Self::BlackBasicEquip => is_black && (kind.is_basic_kind() || kind.is_equip()),
            Self::RedBasic => is_red && kind.is_basic_kind(),
            Self::BlackBasic => is_black && kind.is_basic_kind(),
            Self::RedTrick => is_red && is_trick,
            Self::BlackTrick => is_black && is_trick,
            Self::RedNonBasic => is_red && !kind.is_basic_kind(),
            Self::BlackNonBasic => is_black && !kind.is_basic_kind(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmViewAsSpec {
    pub skill_def_id: u32,
    pub skill_name: String,
    pub as_kind: crate::catalog::CardKind,
    pub window: VmViewAsWindow,
    pub zone: VmViewAsZone,
    pub filter: VmViewAsFilter,
    pub uses_per_turn: Option<u8>,
    pub ignore_distance: bool,
    pub red_unrespondable: bool,
    pub black_no_use_count: bool,
    pub no_use_count: bool,
    pub target_lacks_source_suit_unrespondable: bool,
    pub refund_use_count_if_no_damage: bool,
    pub post_draw: u8,
}

impl VmViewAsSpec {
    pub fn matches_card(
        &self,
        state: &GameStateCore,
        catalog: &Catalog,
        owner: PlayerId,
        card: CardId,
        in_play: bool,
        target_kind: Option<crate::catalog::CardKind>,
    ) -> bool {
        let p = state.player(owner);
        if !p.alive {
            return false;
        }
        if state
            .vm_aux
            .flags
            .get(&(owner, format!("skill_banned_turn:{}", self.skill_name)))
            .copied()
            .unwrap_or(0)
            != 0
        {
            return false;
        }
        let window_ok = if in_play {
            matches!(self.window, VmViewAsWindow::PlayPhase | VmViewAsWindow::Any)
        } else {
            matches!(self.window, VmViewAsWindow::Response | VmViewAsWindow::Any)
        };
        if !window_ok {
            return false;
        }
        if let Some(tk) = target_kind {
            let kind_ok = if tk == crate::catalog::CardKind::Sha {
                self.as_kind.is_sha_like()
            } else {
                self.as_kind == tk
            };
            if !kind_ok {
                return false;
            }
        } else if in_play {
            if matches!(
                self.as_kind,
                crate::catalog::CardKind::Shan | crate::catalog::CardKind::Nullify
            ) {
                return false;
            }
            if self.as_kind == crate::catalog::CardKind::Tao && p.hp >= p.max_hp as i32 {
                return false;
            }
            if self.as_kind == crate::catalog::CardKind::Jiu && state.turn.jiu_used {
                return false;
            }
        }
        if let Some(max) = self.uses_per_turn {
            let key = format!("view_as:{}", self.skill_name);
            if state.vm_aux.once_count(owner, 1, &key) >= max as usize {
                return false;
            }
        }
        let in_hand = p.hand.contains(&card);
        let in_equip = p.equips.contains(&card);
        let zone_ok = match self.zone {
            VmViewAsZone::Hand => in_hand,
            VmViewAsZone::Equip => in_equip,
            VmViewAsZone::HandEquip => in_hand || in_equip,
        };
        zone_ok && self.filter.matches(state, catalog, owner, card)
    }

    pub fn record_use(&self, core: &mut GameStateCore, owner: PlayerId) {
        core.vm_aux
            .flags
            .insert((owner, "event_view_as_skill".to_string()), self.skill_def_id);
        if self.uses_per_turn.is_some() {
            core.vm_aux
                .once
                .insert((owner, format!("view_as:{}", self.skill_name), 1));
        }
        if self.post_draw > 0 {
            for _ in 0..self.post_draw {
                if let Some(c) = core.deck.pop() {
                    core.player_mut(owner).hand.push(c);
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
    pub view_as: Vec<VmViewAsSpec>,
}

impl VmHandlerSet {
    pub(super) fn available(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> bool {
        let p = ctx.state.player(owner);
        let Some(active) = &self.active else {
            return false;
        };
        let name = &active.skill_name;
        let base_ok = p.alive
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
            && (self.limits.require_missing_skill == 0
                || !owner_has_skill_hash(
                    ctx.state,
                    ctx.catalog,
                    owner,
                    self.limits.require_missing_skill,
                ))
            && (!self.limits.require_hand_limit_positive
                || ctx.state.hand_limit(ctx.catalog, owner) > 0)
            && (self.limits.require_hand_missing_gaintag == 0
                || (ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(
                        owner,
                        format!("gaintag:{}", self.limits.require_hand_missing_gaintag),
                    ))
                    .copied()
                    .unwrap_or(0)
                    == 0
                    && ctx
                        .state
                        .vm_aux
                        .marks
                        .get(&(owner, self.limits.require_hand_missing_gaintag))
                        .copied()
                        .unwrap_or(0)
                        <= 0))
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
            });
        if !base_ok {
            return false;
        }
        let cand_cards = self
            .limits
            .selection
            .map(|sel| sel.candidate_cards(ctx.state, ctx.catalog, owner, name));
        if let (Some(sel), Some(cards)) = (self.limits.selection, &cand_cards) {
            if cards.len() < sel.min_cards as usize {
                return false;
            }
        }
        if let Some(tgts) = self.limits.targets {
            let cand_targets = tgts.candidate_targets(ctx.state, ctx.catalog, owner, name);
            if cand_targets.len() < tgts.min_targets as usize {
                return false;
            }
            if let (Some(sel), Some(cards)) = (self.limits.selection, &cand_cards) {
                let max_cards_avail = cards.len().min(sel.max_cards as usize);
                if tgts.match_selected_card_count {
                    let min_req = sel.min_cards.max(tgts.min_targets) as usize;
                    let max_avail = max_cards_avail
                        .min(cand_targets.len())
                        .min(tgts.effective_max_targets(ctx.state, owner));
                    if max_avail < min_req {
                        return false;
                    }
                }
                if tgts.selected_cards_match_target_equip_count
                    && !cand_targets.iter().any(|&q| {
                        let eq_len = ctx.state.player(q).equips.len();
                        eq_len >= sel.min_cards as usize && eq_len <= max_cards_avail
                    })
                {
                    return false;
                }
            }
        }
        true
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
