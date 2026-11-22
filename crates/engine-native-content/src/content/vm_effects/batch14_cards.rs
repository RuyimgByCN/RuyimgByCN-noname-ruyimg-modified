//! PA12 批⑭：选牌、拼点、观星、牌堆与移动交互原语（tags 131, 134, 135, 138, 141, 145, 146, 147, 148, 149, 151, 152）。
use super::batch13_targets::seats_from;
use super::*;
use crate::catalog::CardKind as NativeCardKind;

fn matches_card_kind(native: NativeCardKind, filter: u8) -> bool {
    match filter {
        100 => native.is_basic_kind(),
        101 => native.is_basic_trick_or_equip() && !native.is_basic_kind() && !native.is_equip(),
        102 => native.is_equip(),
        1 => matches!(
            native,
            NativeCardKind::Sha | NativeCardKind::FireSha | NativeCardKind::LeiSha
        ),
        2 => native == NativeCardKind::Shan,
        3 => native == NativeCardKind::Tao,
        4 => native == NativeCardKind::Jiu,
        5 => native == NativeCardKind::Duel,
        6 => native == NativeCardKind::Snatch,
        7 => native == NativeCardKind::Dismantle,
        8 => native == NativeCardKind::SavageAttack,
        9 => native == NativeCardKind::ArcheryAttack,
        12 => native == NativeCardKind::Nullify,
        13 => native == NativeCardKind::Le,
        14 => native == NativeCardKind::Bing,
        15 => native == NativeCardKind::ShanDian,
        16 => matches!(native, NativeCardKind::Weapon | NativeCardKind::Sanjian),
        17 => matches!(native, NativeCardKind::Armor | NativeCardKind::Suolianjia),
        18 => native == NativeCardKind::HorsePlus,
        19 => native == NativeCardKind::HorseMinus,
        20 => native == NativeCardKind::WuxingPan,
        21 => native == NativeCardKind::Chain,
        _ => false,
    }
}

fn hand_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

fn equip_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    core.player(owner)
        .equips
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

fn field_options(core: &GameStateCore, dest: u8) -> Vec<ChoiceOption> {
    let mut out = Vec::new();
    for p in seats_from(core, dest) {
        if p == dest || !core.player(p).alive {
            continue;
        }
        let pl = core.player(p);
        out.extend(
            pl.equips
                .iter()
                .chain(pl.judge.iter())
                .copied()
                .map(ChoiceOption::Card),
        );
    }
    out
}

fn resolve_pile_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: u8,
    pile_def: u32,
) -> Option<((u8, String), Vec<u32>)> {
    if pile_def != 0 {
        if let Some(def) = crate::rvm_ops::resolve_skill_def(catalog, pile_def) {
            let key = (owner, catalog.skills[def as usize].name.to_string());
            if let Some(cards) = core.vm_aux.piles.get(&key) {
                if !cards.is_empty() {
                    return Some((key, cards.clone()));
                }
            }
        }
        let key = (owner, format!("pile:{pile_def}"));
        if let Some(cards) = core.vm_aux.piles.get(&key) {
            if !cards.is_empty() {
                return Some((key, cards.clone()));
            }
        }
    }
    for ((p, name), cards) in &core.vm_aux.piles {
        if *p == owner && !cards.is_empty() {
            return Some(((*p, name.clone()), cards.clone()));
        }
    }
    None
}

fn discard_kind_options(
    core: &GameStateCore,
    catalog: &Catalog,
    kind: u8,
) -> Vec<ChoiceOption> {
    let mut out = Vec::new();
    for &c in &core.discard {
        if (c as usize) < catalog.cards.len()
            && matches_card_kind(catalog.cards[c as usize].kind, kind)
            && !out.contains(&ChoiceOption::Card(c))
        {
            out.push(ChoiceOption::Card(c));
        }
    }
    out
}

pub(super) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match effect {
        EffectRequest::ChooseCompareSelectedHandCard { player, target, .. } => {
            crate::content::host::step::l11_common_b2::vm_pindian_active_ask(
                seat(*player, core)?,
                *target,
                true,
                state,
                core,
                catalog,
            )
        }
        EffectRequest::ChooseHandCompare { player, .. } => {
            crate::content::host::step::l11_common_b2::vm_pindian_active_ask(
                seat(*player, core)?,
                rule_api::PlayerId::NONE,
                false,
                state,
                core,
                catalog,
            )
        }
        EffectRequest::DrawThenGiveOne { player, target, .. } if state == [2] => {
            let owner = seat(*player, core)?;
            let t = seat(*target, core)?;
            if owner == t || !core.player(t).alive {
                return None;
            }
            let options = hand_options(core, owner);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::GuanxingTop { player, .. } if state.len() >= 2 && state[0] == 1 => {
            let owner = seat(*player, core)?;
            let options: Vec<ChoiceOption> =
                state[1..].iter().copied().map(ChoiceOption::Card).collect();
            let max = options.len();
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectCards,
                options,
                0,
                max,
                true,
            ))
        }
        EffectRequest::ChooseOneFromSkillPileDiscard {
            player, pile_def, ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let (_, cards) = resolve_pile_cards(core, catalog, owner, *pile_def)?;
            let options: Vec<ChoiceOption> = cards.into_iter().map(ChoiceOption::Card).collect();
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::ChooseDiscardHandRange {
            player, min, max, ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = hand_options(core, owner);
            if options.is_empty() {
                return None;
            }
            let lo = (*min as usize).max(1).min(options.len());
            let hi = (*max as usize).max(lo).min(options.len());
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                lo,
                hi,
                *min == 0,
            ))
        }
        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { player, count, .. }
            if state == [1] =>
        {
            let owner = seat(*player, core)?;
            let options = cards(core, owner);
            let req = *count as usize;
            if options.is_empty() || options.len() < req {
                return None;
            }
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                req,
                req,
                false,
            ))
        }
        EffectRequest::ChooseHandCardsToSkillPile {
            player,
            min_count,
            max_count,
            ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = hand_options(core, owner);
            if options.is_empty() {
                return None;
            }
            let lo = (*min_count as usize).max(1).min(options.len());
            let hi = (*max_count as usize).max(lo).min(options.len());
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectCards,
                options,
                lo,
                hi,
                true,
            ))
        }
        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { player, .. }
            if state == [1] =>
        {
            let owner = seat(*player, core)?;
            let options = equip_options(core, owner);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_discard_pick",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                true,
            ))
        }
        EffectRequest::ChooseGainDiscardCardByKind { player, kind, .. } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = discard_kind_options(core, catalog, *kind);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::ChooseMoveFieldCardTo { player, target, .. } if state == [1] => {
            let owner = seat(*player, core)?;
            let t = seat(*target, core)?;
            if !core.player(owner).alive || !core.player(t).alive {
                return None;
            }
            let options = field_options(core, t);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                true,
            ))
        }
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { player, .. }
            if state == [1] =>
        {
            let owner = seat(*player, core)?;
            if !core.player(owner).alive {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::Confirm,
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                true,
            ))
        }
        _ => None,
    }
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    skill: &str,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    match effect {
        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            let t = seat(*target, ctx.core)?;
            return Some(step_draw_then_give_one(ctx, owner, t, *count, state));
        }
        EffectRequest::GuanxingTop { player, count, .. } => {
            let owner = seat(*player, ctx.core)?;
            return Some(step_guanxing_top(ctx, owner, *count, state));
        }
        EffectRequest::ChooseCompareSelectedHandCard { player, .. }
        | EffectRequest::ChooseHandCompare { player, .. }
        | EffectRequest::ChooseOneFromSkillPileDiscard { player, .. }
        | EffectRequest::ChooseDiscardHandRange { player, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { player, .. }
        | EffectRequest::ChooseHandCardsToSkillPile { player, .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { player, .. }
        | EffectRequest::ChooseGainDiscardCardByKind { player, .. }
        | EffectRequest::ChooseMoveFieldCardTo { player, .. }
        | EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            if !ctx.core.player(owner).alive {
                return Some(Flow::Done);
            }
            if state.is_empty() {
                if let EffectRequest::ChooseDiscardHandRange { max, .. } = effect {
                    if *max == 0 {
                        return Some(Flow::Done);
                    }
                }
                if let EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { count, .. } =
                    effect
                {
                    if *count == 0 {
                        return Some(Flow::Done);
                    }
                }
                if let EffectRequest::ChooseHandCardsToSkillPile { max_count, .. } = effect {
                    if *max_count == 0 {
                        return Some(Flow::Done);
                    }
                }
                let Some(expected) = active_ask(effect, &[1], ctx.core, ctx.catalog) else {
                    return Some(Flow::Done);
                };
                *state = vec![1];
                return Some(wait(expected));
            }
        }
        _ => return None,
    }
    match effect {
        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(
                crate::content::host::step::l11_common_b2::vm_step_hand_compare(
                    ctx,
                    owner,
                    *target,
                    *card,
                    *win_skill_def,
                    *lose_skill_def,
                    state,
                ),
            )
        }
        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(
                crate::content::host::step::l11_common_b2::vm_step_hand_compare(
                    ctx,
                    owner,
                    rule_api::PlayerId::NONE,
                    rule_api::CardId::NONE,
                    *win_skill_def,
                    *lose_skill_def,
                    state,
                ),
            )
        }
        EffectRequest::ChooseOneFromSkillPileDiscard {
            player, pile_def, ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_skill_pile_discard(ctx, owner, *pile_def))
        }
        EffectRequest::ChooseDiscardHandRange { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_discard_hand_range(ctx, owner))
        }
        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count,
            skill: skill_id,
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_discard_then_remove_skill(
                ctx,
                owner,
                *count,
                skill_id.raw(),
                skill,
            ))
        }
        EffectRequest::ChooseHandCardsToSkillPile { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_hand_to_skill_pile(ctx, owner, skill))
        }
        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_discard_equip_prevent_damage(ctx, owner))
        }
        EffectRequest::ChooseGainDiscardCardByKind { player, kind, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_gain_discard_by_kind(ctx, owner, *kind))
        }
        EffectRequest::ChooseMoveFieldCardTo { player, target, .. } => {
            let owner = seat(*player, ctx.core)?;
            let t = seat(*target, ctx.core)?;
            Some(step_move_field_card_to(ctx, owner, t))
        }
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_judge_sha_unrespondable(
                ctx,
                owner,
                card.raw(),
                *target,
                *retieji,
            ))
        }
        _ => None,
    }
}

fn step_draw_then_give_one(
    ctx: &mut Ctx<'_>,
    owner: u8,
    target: u8,
    count: u32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        if count > 0 {
            ctx.start_draw(owner, count);
        }
        if owner == target || !ctx.core.player(target).alive {
            return Flow::Done;
        }
        *state = vec![1];
        return Flow::Yield;
    }
    if state == &[1] {
        if owner == target || !ctx.core.player(target).alive || ctx.core.player(owner).hand.is_empty()
        {
            return Flow::Done;
        }
        let options = hand_options(ctx.core, owner);
        *state = vec![2];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectCards,
            options,
            1,
            1,
            false,
        ));
    }
    if state == &[2] {
        if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
            if ctx.core.player(owner).hand.contains(&card) && ctx.core.player(target).alive {
                ctx.give_hand_card(owner, target, card);
            }
        }
    }
    Flow::Done
}

fn step_guanxing_top(
    ctx: &mut Ctx<'_>,
    owner: u8,
    count: u32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive || count == 0 {
        return Flow::Done;
    }
    if state.is_empty() {
        let take = (count as usize).min(5);
        let mut peeked = Vec::new();
        for _ in 0..take {
            if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
                let mut cards = std::mem::take(&mut ctx.core.discard);
                ctx.core.rng.shuffle(&mut cards);
                ctx.core.deck = cards;
                ctx.out.push(crate::machine::LogEvent::DeckReshuffled);
            }
            if let Some(c) = ctx.core.deck.pop() {
                peeked.push(c);
            } else {
                break;
            }
        }
        if peeked.is_empty() {
            return Flow::Done;
        }
        let options: Vec<ChoiceOption> = peeked.iter().copied().map(ChoiceOption::Card).collect();
        let max = options.len();
        *state = std::iter::once(1u32).chain(peeked).collect();
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectCards,
            options,
            0,
            max,
            true,
        ));
    }
    if state.len() >= 2 && state[0] == 1 {
        let peeked = &state[1..];
        let mut top = Vec::new();
        for opt in ctx.answer() {
            if let ChoiceOption::Card(c) = *opt {
                if peeked.contains(&c) && !top.contains(&c) {
                    top.push(c);
                }
            }
        }
        let bottom: Vec<u32> = peeked
            .iter()
            .copied()
            .filter(|c| !top.contains(c))
            .collect();
        for c in top.into_iter().rev() {
            ctx.core.deck.push(c);
        }
        for c in bottom {
            ctx.core.deck.insert(0, c);
        }
    }
    Flow::Done
}

fn step_skill_pile_discard(ctx: &mut Ctx<'_>, owner: u8, pile_def: u32) -> Flow {
    let Some((key, _)) = resolve_pile_cards(ctx.core, ctx.catalog, owner, pile_def) else {
        return Flow::Done;
    };
    if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
        if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&key) {
            if let Some(pos) = pile.iter().position(|&c| c == card) {
                pile.remove(pos);
                ctx.core.discard.push(card);
                ctx.out.push(crate::machine::LogEvent::CardDiscarded {
                    player: owner,
                    cards: vec![card],
                });
                ctx.emit(EventData::CardsDiscarded {
                    player: owner,
                    cards: vec![card],
                });
            }
        }
    }
    Flow::Done
}

fn step_discard_hand_range(ctx: &mut Ctx<'_>, owner: u8) -> Flow {
    let mut picked = Vec::new();
    for opt in ctx.answer() {
        if let ChoiceOption::Card(c) = *opt {
            if ctx.core.player(owner).hand.contains(&c) && !picked.contains(&c) {
                picked.push(c);
            }
        }
    }
    if !picked.is_empty() {
        discard(ctx, owner, &picked);
    }
    Flow::Done
}

fn step_discard_then_remove_skill(
    ctx: &mut Ctx<'_>,
    owner: u8,
    count: u32,
    skill_raw: u32,
    skill_name: &str,
) -> Flow {
    let req = count as usize;
    let options = cards(ctx.core, owner);
    let mut picked = Vec::new();
    for opt in ctx.answer() {
        if let ChoiceOption::Card(c) = *opt {
            if options.contains(&ChoiceOption::Card(c)) && !picked.contains(&c) {
                picked.push(c);
            }
        }
    }
    if picked.len() == req {
        discard(ctx, owner, &picked);
        if let Some(def) = crate::rvm_ops::resolve_skill_def(ctx.catalog, skill_raw)
            .or_else(|| ctx.catalog.skill_by_name(skill_name))
        {
            ctx.remove_skill(owner, def);
        }
    }
    Flow::Done
}

fn step_hand_to_skill_pile(ctx: &mut Ctx<'_>, owner: u8, skill: &str) -> Flow {
    let mut picked = Vec::new();
    for opt in ctx.answer() {
        if let ChoiceOption::Card(c) = *opt {
            if ctx.core.player(owner).hand.contains(&c) && !picked.contains(&c) {
                picked.push(c);
            }
        }
    }
    for c in picked {
        ctx.take_hand_card(owner, c);
        ctx.core
            .vm_aux
            .piles
            .entry((owner, skill.to_string()))
            .or_default()
            .push(c);
    }
    Flow::Done
}

fn step_discard_equip_prevent_damage(ctx: &mut Ctx<'_>, owner: u8) -> Flow {
    if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
        if ctx.core.player(owner).equips.contains(&card) {
            discard(ctx, owner, &[card]);
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::Damage(d) = &mut frame.data {
                    d.prevented = true;
                    d.amount = 0;
                    break;
                }
            }
        }
    }
    Flow::Done
}

fn step_gain_discard_by_kind(ctx: &mut Ctx<'_>, owner: u8, kind: u8) -> Flow {
    let options = discard_kind_options(ctx.core, ctx.catalog, kind);
    if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
        if options.contains(&ChoiceOption::Card(card)) {
            if let Some(pos) = ctx.core.discard.iter().position(|&c| c == card) {
                ctx.core.discard.remove(pos);
                ctx.core.player_mut(owner).hand.push(card);
                ctx.out.push(crate::machine::LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![card],
                });
            }
        }
    }
    Flow::Done
}

fn step_move_field_card_to(ctx: &mut Ctx<'_>, owner: u8, target: u8) -> Flow {
    if !ctx.core.player(owner).alive || !ctx.core.player(target).alive {
        return Flow::Done;
    }
    if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
        for from in seats_from(ctx.core, target) {
            if from == target || !ctx.core.player(from).alive {
                continue;
            }
            if let Some(pos) = ctx.core.player(from).equips.iter().position(|&c| c == card) {
                ctx.core.player_mut(from).equips.remove(pos);
                ctx.core.player_mut(target).equips.push(card);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![card],
                    from,
                    to: target,
                });
                break;
            } else if let Some(pos) = ctx.core.player(from).judge.iter().position(|&c| c == card) {
                ctx.core.player_mut(from).judge.remove(pos);
                ctx.core.player_mut(target).judge.push(card);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![card],
                    from,
                    to: target,
                });
                break;
            }
        }
    }
    Flow::Done
}

fn step_judge_sha_unrespondable(
    ctx: &mut Ctx<'_>,
    owner: u8,
    card_raw: u32,
    target: rule_api::PlayerId,
    retieji: bool,
) -> Flow {
    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
        let mut unrespondable = retieji;
        if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
            let jc = ctx.draw_judge_card_with_elo(
                owner,
                engine_native_kernel::kernel::EloJudgePref::RedPositive,
            );
            let (suit, _) = ctx.inline_judge_suit_rank(owner, jc);
            let is_red = matches!(
                suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            let _ = ctx.settle_inline_judge_card(owner, suit, jc);
            unrespondable = retieji || is_red;
        }
        if unrespondable {
            let seats = ctx.core.players.len() as u8;
            if let Some(t) = seat(target, ctx.core) {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("sha_unrespondable:{card_raw}:{t}")), 1);
            } else {
                for t in 0..seats {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((owner, format!("sha_unrespondable:{card_raw}:{t}")), 1);
                }
            }
        }
    }
    Flow::Done
}

pub(super) fn validate(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    card_count: usize,
) -> bool {
    let seats = core.players.len();
    match effect {
        EffectRequest::DrawThenGiveOne { player, target, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && target.raw() > 0
                && (target.raw() as usize) <= seats
                && (state == [1] || state == [2])
        }
        EffectRequest::GuanxingTop { player, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && state.len() >= 2
                && state.len() <= 6
                && state[0] == 1
                && state[1..].iter().all(|&c| (c as usize) < card_count)
        }
        EffectRequest::ChooseMoveFieldCardTo { player, target, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && target.raw() > 0
                && (target.raw() as usize) <= seats
                && state == [1]
        }
        EffectRequest::ChooseCompareSelectedHandCard { player, .. }
        | EffectRequest::ChooseHandCompare { player, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && (state == [1]
                    || (state.len() == 8
                        && (2..=5).contains(&state[0])
                        && (state[1] as usize) < seats
                        && (state[2] as usize) <= card_count
                        && state[3] <= 1
                        && (state[4] as usize) <= card_count
                        && state[5] <= 1
                        && state[6] <= 3
                        && state[7] <= 3))
        }
        EffectRequest::ChooseOneFromSkillPileDiscard { player, .. }
        | EffectRequest::ChooseDiscardHandRange { player, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { player, .. }
        | EffectRequest::ChooseHandCardsToSkillPile { player, .. }
        | EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { player, .. }
        | EffectRequest::ChooseGainDiscardCardByKind { player, .. }
        | EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed { player, .. } => {
            player.raw() > 0 && (player.raw() as usize) <= seats && state == [1]
        }
        _ => false,
    }
}
