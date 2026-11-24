use crate::catalog::{CardKind, EquipSlot, Suit};
use crate::engine::Engine;
use crate::ids::{CardId, PlayerId};

/// Display style for `L26` `Fool_liegong` live Sha target hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoolLiegongStyle {
    Close,
    Shousha,
    Olten,
    Ol,
}

impl FoolLiegongStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Shousha => "shousha",
            Self::Olten => "olten",
            Self::Ol => "ol",
        }
    }

    pub fn from_config_str(value: &str) -> Option<Self> {
        match value {
            "close" => Some(Self::Close),
            "shousha" => Some(Self::Shousha),
            "olten" => Some(Self::Olten),
            "ol" => Some(Self::Ol),
            _ => None,
        }
    }
}

fn suit_matches_heart(suit: Suit) -> bool {
    suit == Suit::Heart
}

fn suit_is_red(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

fn suit_is_black(suit: Suit) -> bool {
    matches!(suit, Suit::Spade | Suit::Club)
}

fn card_rank(engine: &Engine, card: CardId) -> u32 {
    u32::from(engine.catalog().cards[card as usize].rank)
}

impl Engine {
    /// Persist `L26` `Fool_liegong` style into `vm_aux.flags` (survives save/restore).
    pub fn set_fool_liegong_style(&mut self, style: FoolLiegongStyle) {
        let key = (0u8, "l26_fool_liegong_style".to_string());
        match style {
            FoolLiegongStyle::Close => {
                self.state.core.vm_aux.flags.remove(&key);
            }
            FoolLiegongStyle::Shousha => {
                self.state.core.vm_aux.flags.insert(key, 1);
            }
            FoolLiegongStyle::Olten => {
                self.state.core.vm_aux.flags.insert(key, 2);
            }
            FoolLiegongStyle::Ol => {
                self.state.core.vm_aux.flags.insert(key, 3);
            }
        }
    }

    /// Read configured `L26` `Fool_liegong` style from engine state.
    pub fn fool_liegong_style(&self) -> FoolLiegongStyle {
        match self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l26_fool_liegong_style".to_string()))
            .copied()
            .unwrap_or(0)
        {
            1 => FoolLiegongStyle::Shousha,
            2 => FoolLiegongStyle::Olten,
            3 => FoolLiegongStyle::Ol,
            _ => FoolLiegongStyle::Close,
        }
    }

    /// Enable or disable `L26` `Fool_skill` rank-sum helper in engine state.
    pub fn set_fool_skill_enabled(&mut self, enabled: bool) {
        let key = (0u8, "l26_fool_skill".to_string());
        if enabled {
            self.state.core.vm_aux.flags.insert(key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&key);
        }
    }

    /// Whether `L26` `Fool_skill` candidate filtering is enabled in engine state.
    pub fn fool_skill_enabled(&self) -> bool {
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l26_fool_skill".to_string()))
            .copied()
            .unwrap_or(0)
            != 0
    }

    /// Compute live `L26` `Fool_liegong` hint text for using Sha from `viewer` targeting `target`.
    pub fn query_fool_sha_hints(
        &self,
        viewer: PlayerId,
        target: PlayerId,
        card: Option<CardId>,
        selected_count: usize,
    ) -> Option<String> {
        let style = self.fool_liegong_style();
        if style == FoolLiegongStyle::Close || viewer == target {
            return None;
        }
        let core = &self.state.core;
        let cat = &self.catalog;
        if core.turn.current != viewer || !core.player(viewer).alive || !core.player(target).alive {
            return None;
        }
        let kind = card.map(|c| cat.card_kind(c)).unwrap_or(CardKind::Sha);
        let original_filter = core.legal_targets(cat, kind, viewer).2.contains(&target);
        let in_range = core.computed_distance(cat, viewer, target) <= core.attack_range(cat, viewer);
        let has_skill = |names: &[&str]| {
            names.iter().any(|name| {
                cat.skill_by_name(name)
                    .is_some_and(|def| core.has_skill(viewer, def))
            })
        };

        let p_viewer = core.player(viewer);
        let p_target = core.player(target);
        let v_hand = p_viewer.hand.len() as i64;
        let t_hand = p_target.hand.len() as i64;
        let v_hp = i64::from(p_viewer.hp);
        let t_hp = i64::from(p_target.hp);
        let atk_range = i64::from(core.attack_range(cat, viewer));
        let sel = selected_count as i64;

        let mut liegong = false;
        let mut qiangming = false;
        let mut shanghai = false;

        let has_diy_lg = has_skill(&["DIY_liegong"]);
        let has_lg = has_skill(&["liegong", "烈弓"]);
        let has_xin_lg = has_skill(&["xinliegong", "新烈弓"]);
        if has_diy_lg || has_lg || has_xin_lg {
            let cond = (has_diy_lg && t_hand <= v_hand - sel)
                || (has_lg && (t_hand >= v_hp || t_hand <= atk_range))
                || (has_xin_lg && (t_hand <= v_hand - sel || t_hp >= v_hp));
            if cond && in_range {
                liegong = true;
                if has_xin_lg {
                    if t_hand <= v_hand - sel {
                        qiangming = true;
                    }
                    if t_hp >= v_hp {
                        shanghai = true;
                    }
                }
            }
        }

        let suit = card.map(|c| cat.cards[c as usize].suit);
        let is_black = suit.is_some_and(suit_is_black);
        let is_red = suit.is_some_and(suit_is_red);
        let is_heart = suit.is_some_and(suit_matches_heart);

        let spshidi_mark = cat
            .skill_by_name("spshidi")
            .and_then(|d| core.vm_aux.marks.get(&(viewer, u32::from(d))).copied())
            .unwrap_or(0);
        if has_skill(&["spshidi"]) && is_black && spshidi_mark % 2 == 0 && in_range && original_filter
        {
            qiangming = true;
        }
        if has_skill(&["fuqi"])
            && in_range
            && original_filter
            && core.computed_distance(cat, target, viewer) <= 1
        {
            qiangming = true;
        }
        if has_skill(&["refuqi"])
            && in_range
            && original_filter
            && core.computed_distance(cat, viewer, target) <= 1
        {
            qiangming = true;
        }
        if has_skill(&["wushen", "武神"]) && is_heart && in_range && original_filter {
            qiangming = true;
        }
        if has_skill(&["sbpaoxiao"]) && core.turn.sha_used_play_phase && in_range && original_filter {
            qiangming = true;
        }
        if has_skill(&["liyong2"]) && in_range && original_filter {
            qiangming = true;
        }
        if has_skill(&["zhaxiang2"]) && is_red && in_range && original_filter {
            qiangming = true;
        }
        let damaged = i64::from(p_viewer.max_hp as i32 - p_viewer.hp).max(0);
        if has_skill(&["sbzhaxiang"])
            && i64::from(core.turn.cards_used_turn) < damaged
            && in_range
            && original_filter
        {
            qiangming = true;
        }
        let empty_slots_2_to_6 = !p_viewer
            .equips
            .iter()
            .any(|c| !matches!(cat.card_slot_of(core, *c), Some(EquipSlot::Weapon) | None));
        if has_skill(&["relinglong"]) && empty_slots_2_to_6 && in_range && original_filter {
            qiangming = true;
        }
        let phase_sha_count = core
            .vm_aux
            .flags
            .get(&(viewer, "q4_play_sha_count".to_string()))
            .copied()
            .unwrap_or_else(|| u32::from(core.turn.sha_used_play_phase));
        if has_skill(&["jintao"]) && (phase_sha_count == 0 || phase_sha_count == 1) {
            if phase_sha_count == 0 {
                shanghai = true;
            } else {
                qiangming = true;
            }
        }
        let mark_basic = core
            .vm_aux
            .flags
            .get(&(viewer, "dcgonghu_basic".to_string()))
            .copied()
            .unwrap_or(0)
            != 0;
        let mark_damage = core
            .vm_aux
            .flags
            .get(&(viewer, "dcgonghu_damage".to_string()))
            .copied()
            .unwrap_or(0)
            != 0;
        if has_skill(&["dcgonghu"])
            && mark_basic
            && mark_damage
            && is_red
            && in_range
            && original_filter
        {
            qiangming = true;
        }

        let mut text = String::new();
        let is_shousha = style == FoolLiegongStyle::Shousha;
        if liegong {
            text.push_str("可烈弓");
            if qiangming {
                if is_shousha {
                    text = "烈弓强中".into();
                } else {
                    text.push_str(" 不可响应");
                }
                if shanghai {
                    if is_shousha {
                        text = "强中加伤".into();
                    } else {
                        text.push_str(" 伤害+1");
                    }
                }
            } else if shanghai {
                if is_shousha {
                    text = "烈弓加伤".into();
                } else {
                    text.push_str(" 伤害+1");
                }
            }
        } else if qiangming {
            text = "不可响应".into();
            if shanghai {
                if is_shousha {
                    text = "强中加伤".into();
                } else {
                    text.push_str(" 伤害+1");
                }
            }
        } else if shanghai {
            if is_shousha {
                text = "加伤".into();
            } else {
                text.push_str(" 伤害+1");
            }
        }
        if style == FoolLiegongStyle::Ol && !text.is_empty() && liegong {
            text = "可烈弓".into();
        }
        Some(text)
    }

    /// Filter legal next cards for `L26` `ziyuan` / `minsi` (<= 13) or `xinfu_fangtong` (<= 36).
    pub fn fool_skill_filter_candidates(
        &self,
        seat: PlayerId,
        skill: &str,
        selected: &[CardId],
    ) -> Vec<CardId> {
        let core = &self.state.core;
        let pl = core.player(seat);
        let enabled = self.fool_skill_enabled();
        let sum: u32 = selected.iter().map(|c| card_rank(self, *c)).sum();
        match skill {
            "ziyuan" => pl
                .hand
                .iter()
                .copied()
                .filter(|c| !selected.contains(c) && (!enabled || sum + card_rank(self, *c) <= 13))
                .collect(),
            "minsi" => pl
                .hand
                .iter()
                .chain(pl.equips.iter())
                .copied()
                .filter(|c| !selected.contains(c) && (!enabled || sum + card_rank(self, *c) <= 13))
                .collect(),
            "xinfu_fangtong" => {
                let fang_pile = core
                    .vm_aux
                    .piles
                    .get(&(seat, "xinfu_jijun".to_string()))
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                if !enabled {
                    let source = if selected.is_empty() {
                        pl.hand
                            .iter()
                            .chain(pl.equips.iter())
                            .copied()
                            .collect::<Vec<_>>()
                    } else {
                        fang_pile.to_vec()
                    };
                    return source
                        .into_iter()
                        .filter(|c| !selected.contains(c))
                        .collect();
                }
                if selected.is_empty() {
                    pl.hand
                        .iter()
                        .chain(pl.equips.iter())
                        .copied()
                        .filter(|c| card_rank(self, *c) <= 36)
                        .collect()
                } else {
                    fang_pile
                        .iter()
                        .copied()
                        .filter(|c| !selected.contains(c) && sum + card_rank(self, *c) <= 36)
                        .collect()
                }
            }
            _ => Vec::new(),
        }
    }

    /// Recommend a deterministic card combination (`[he_card, fang_cards...]`) summing to 36
    /// for `xinfu_fangtong` when `Fool_skill` is active on `seat`.
    pub fn recommend_fangtong_combo(&self, seat: PlayerId) -> Option<Vec<CardId>> {
        let pl = self.state.core.player(seat);
        let fang_pile = self
            .state
            .core
            .vm_aux
            .piles
            .get(&(seat, "xinfu_jijun".to_string()))?;
        if fang_pile.is_empty() {
            return None;
        }
        let he_cards: Vec<CardId> = pl.hand.iter().chain(pl.equips.iter()).copied().collect();
        for he in he_cards {
            let need = 36u32.checked_sub(card_rank(self, he))?;
            if let Some(sub) = find_subset_sum(self, fang_pile, need) {
                if !sub.is_empty() {
                    let mut out = Vec::with_capacity(1 + sub.len());
                    out.push(he);
                    out.extend(sub);
                    return Some(out);
                }
            }
        }
        None
    }
}

fn find_subset_sum(engine: &Engine, items: &[CardId], target: u32) -> Option<Vec<CardId>> {
    if target == 0 {
        return Some(Vec::new());
    }
    let mut chosen = Vec::new();
    fn dfs(
        engine: &Engine,
        items: &[CardId],
        idx: usize,
        rem: u32,
        chosen: &mut Vec<CardId>,
    ) -> bool {
        if rem == 0 {
            return !chosen.is_empty();
        }
        for i in idx..items.len() {
            let r = card_rank(engine, items[i]);
            if r <= rem {
                chosen.push(items[i]);
                if dfs(engine, items, i + 1, rem - r, chosen) {
                    return true;
                }
                chosen.pop();
            }
        }
        false
    }
    if dfs(engine, items, 0, target, &mut chosen) {
        Some(chosen)
    } else {
        None
    }
}
