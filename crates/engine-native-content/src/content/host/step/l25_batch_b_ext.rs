//! L25 祖安武将 Batch B 后半技能步进（技能 12–22：`ssr_jinfan` 至 `ssr_qingji`）。

use super::l25_batch_b_asks::wait_ask;
use super::*;
use crate::catalog::{CardKind, EquipSlot, Suit};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_l25_batch_b_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "ssr_jinnang" => step_jinnang(m, ctx),
        "ssr_newbazhen" => step_newbazhen(m, ctx),
        "ssr_tuxi" => step_tuxi(m, ctx),
        "ssr_newmubing" => step_newmubing(m, ctx),
        "ssr_jinfan" => step_jinfan(m, ctx),
        "ssr_qixi" => step_qixi(m, ctx),
        "ssr_zhuiji" => step_zhuiji(m, ctx),
        "ssr_tieji" => step_tieji(m, ctx),
        "ssr_youqin" => step_youqin(m, ctx),
        "ssr_newfanjian" => step_newfanjian(m, ctx),
        "ssr_shichou" => step_shichou(m, ctx),
        "ssr_newrende" => step_newrende(m, ctx),
        "ssr_jijiang" => step_jijiang(m, ctx),
        "ssr_qiangxi" => step_qiangxi(m, ctx),
        "ssr_qingji" => step_qingji(m, ctx),
        _ => Flow::Done,
    }
}

fn jinnang_kind_from_tag(tag: u8) -> CardKind {
    match tag {
        0 => CardKind::Wuzhong,
        1 => CardKind::Duel,
        2 => CardKind::Dismantle,
        3 => CardKind::Snatch,
        4 => CardKind::SavageAttack,
        5 => CardKind::ArcheryAttack,
        6 => CardKind::Taoyuan,
        _ => CardKind::Nullify,
    }
}

fn step_jinnang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        let holder = m.target.unwrap_or(owner);
        let pack = flag(ctx.core, holder, "l25_jinnang_pack");
        if pack == 0 {
            return Flow::Done;
        }
        set_flag(ctx.core, holder, "l25_jinnang_pack", 0);
        if let Some(pile) = ctx.core.vm_aux.piles.remove(&(holder, "ssr_jinnang".to_string())) {
            for c in pile {
                ctx.to_discard(c);
            }
        }
        let kind = jinnang_kind_from_tag((pack - 1) as u8);
        set_flag(ctx.core, holder, "l25_jinnang_opened", pack);
        if !ctx.core.deck.is_empty() {
            let tgts = match kind {
                CardKind::Wuzhong | CardKind::Nullify => vec![holder],
                CardKind::Taoyuan => alive_all(ctx.core, holder),
                CardKind::SavageAttack | CardKind::ArcheryAttack => alive_others(ctx.core, holder),
                _ => alive_others(ctx.core, holder)
                    .into_iter()
                    .take(1)
                    .collect(),
            };
            if !tgts.is_empty() {
                ctx.use_virtual_card(holder, kind, tgts);
                m.card = Some(101);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
        }
        return Flow::Done;
    }
    if m.card == Some(101) {
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_JINNANG_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards = vec![c];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_JINNANG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_JINNANG_MODE)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let tgt = m.target.unwrap_or(owner);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jinnang");
            set_flag(ctx.core, owner, "l25_jinnang_used", cnt);
            if let Some(&c) = m.cards.first() {
                discard_he_cards(ctx, owner, &[c]);
            }
            set_flag(ctx.core, tgt, "l25_jinnang_pack", u32::from(tag) + 1);
            if let Some(def) = ctx.catalog.skill_by_name("ssr_jinnang") {
                if !ctx.core.has_skill(tgt, def) {
                    ctx.ensure_skill(
                        (tgt, def),
                        crate::state::SkillInstance {
                            owner: tgt,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_newbazhen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        if let Some(old) = ctx.core.vm_aux.piles.remove(&(owner, "ssr_newbazhen".to_string())) {
            for c in old {
                ctx.to_discard(c);
            }
        }
        let mut new_pile = Vec::new();
        for suit in [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond] {
            if let Some(c) = fetch_card_matching(ctx, |cid, cat| {
                cat.card_kind(cid).is_basic_kind()
                    && cat.cards.get(cid as usize).is_some_and(|d| d.suit == suit)
            }) {
                new_pile.push(c);
            }
        }
        let len = new_pile.len() as u32;
        if !new_pile.is_empty() {
            ctx.core
                .vm_aux
                .piles
                .insert((owner, "ssr_newbazhen".to_string()), new_pile);
        }
        set_flag(ctx.core, owner, "l25_bazhen_count", len);
        set_flag(ctx.core, owner, "l25_bazhen_shan_guard", if len > 0 { 1 } else { 0 });
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_BAZHEN_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards = vec![c];
            if ctx.catalog.card_kind(c).is_sha_like() {
                return wait_ask(m, ctx, HostStage::TargetChoice, ASK_BAZHEN_TARGET);
            }
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            use_bazhen_pile_card(m, ctx, c, vec![owner])
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let Some(&c) = m.cards.first() else {
                return Flow::Done;
            };
            use_bazhen_pile_card(m, ctx, c, vec![tgt])
        }
        _ => Flow::Done,
    }
}

fn use_bazhen_pile_card(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    c: CardId,
    targets: Vec<PlayerId>,
) -> Flow {
    let owner = m.owner;
    if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(owner, "ssr_newbazhen".to_string())) {
        pile.retain(|&x| x != c);
    }
    let rem = bazhen_pile(ctx.core, owner).len() as u32;
    set_flag(ctx.core, owner, "l25_bazhen_count", rem);
    set_flag(ctx.core, owner, "l25_bazhen_shan_guard", if rem > 0 { 1 } else { 0 });
    ctx.core.player_mut(owner).hand.push(c);
    let kind = ctx.catalog.card_kind(c);
    let nature = match kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(c),
        player: owner,
        targets,
        nature,
        use_as: Some(kind),
        cancelled: false,
    }));
    m.stage = HostStage::CardsAsk;
    Flow::Yield
}

fn step_tuxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        let mks = tuxi_marks(ctx.core, owner);
        set_tuxi_marks(ctx.core, owner, mks);
        return Flow::Done;
    }
    if m.card == Some(101) {
        let mks = tuxi_marks(ctx.core, owner) + 1;
        set_tuxi_marks(ctx.core, owner, mks);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_TUXI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let mks = tuxi_marks(ctx.core, owner).saturating_sub(1);
            set_tuxi_marks(ctx.core, owner, mks);
            let stolen = he_cards(ctx.core, tgt);
            for &c in &stolen {
                ctx.take_owned_card(tgt, c);
                ctx.core.player_mut(owner).hand.push(c);
            }
            if !stolen.is_empty() {
                ctx.out.push(LogEvent::CardsTaken {
                    from: tgt,
                    to: owner,
                    cards: stolen,
                    public: false,
                });
            }
            if ctx.core.player(tgt).hp > ctx.core.player(owner).hp {
                set_flag(ctx.core, tgt, "l20_diaohulishan_exiled", 1);
                set_flag(ctx.core, tgt, "diaohulishan_active", 1);
            }
            set_flag(ctx.core, owner, "l25_tuxi_used", u32::from(tgt) + 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_newmubing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.core.deck.len() < 3 && !ctx.core.discard.is_empty() {
                let mut extra = std::mem::take(&mut ctx.core.discard);
                ctx.core.rng.shuffle(&mut extra);
                extra.append(&mut ctx.core.deck);
                ctx.core.deck = extra;
                ctx.out.push(LogEvent::DeckReshuffled);
            }
            let top3: Vec<CardId> = ctx.core.deck.iter().rev().take(3).copied().collect();
            if top3.is_empty() {
                return Flow::Done;
            }
            m.cards = top3;
            wait_ask(m, ctx, HostStage::SelectCard, ASK_MUBING_CARDS)
        }
        HostStage::SelectCard => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) if m.cards.contains(&c) => Some(c),
                    _ => None,
                })
                .collect();
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if !picked.is_empty() {
                for &c in &picked {
                    ctx.core.deck.retain(|&x| x != c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
                ctx.out.push(LogEvent::CardsDrawn {
                    player: owner,
                    cards: picked.clone(),
                });
                set_flag(ctx.core, owner, "l25_mubing_gained", picked.len() as u32);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_jinfan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        let ling = flag(ctx.core, owner, "l25_jinfan_ling")
            .max(jinfan_pile(ctx.core, owner).len() as u32);
        for &tgt in &m.targets {
            if (ctx.core.player(tgt).hand.len() as u32) < ling {
                set_flag(ctx.core, tgt, "qinggang_disable_armor", 1);
                set_flag(ctx.core, owner, "l25_jinfan_triggered", u32::from(tgt) + 1);
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_JINFAN_DISCARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_jinfan");
            set_flag(ctx.core, owner, "l25_jinfan_used", cnt);
            discard_he_cards(ctx, owner, &[c]);
            let mut added = Vec::new();
            for _ in 0..2 {
                if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                    let top = ctx.draw_judge_card();
                    added.push(top);
                }
            }
            if !added.is_empty() {
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "ssr_jinfan".to_string()))
                    .or_default()
                    .extend(added);
            }
            let ling = jinfan_pile(ctx.core, owner).len() as u32;
            set_flag(ctx.core, owner, "l25_jinfan_ling", ling);
            set_flag(ctx.core, owner, "mark:ssr_jinfan", ling);
            m.stage = HostStage::CardsAsk;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn resolve_qixi(m: &mut HostMachine, ctx: &mut Ctx<'_>, tgt: PlayerId) -> Flow {
    let owner = m.owner;
    let pile = ctx
        .core
        .vm_aux
        .piles
        .remove(&(owner, "ssr_jinfan".to_string()))
        .unwrap_or_default();
    let flag_ling = flag(ctx.core, owner, "l25_jinfan_ling");
    let (basic_cnt, trick_cnt, equip_cnt, total_ling) = if !pile.is_empty() {
        let mut b = 0usize;
        let mut t = 0usize;
        let mut e = 0usize;
        for &c in &pile {
            let k = ctx.catalog.card_kind(c);
            if k.is_basic_kind() {
                b += 1;
            } else if k.is_equip() {
                e += 1;
            } else {
                t += 1;
            }
        }
        let tot = pile.len() as u32;
        for c in pile {
            ctx.to_discard(c);
        }
        (b, t, e, tot)
    } else {
        (1, 1, 1, flag_ling.max(1))
    };
    set_flag(ctx.core, owner, "l25_jinfan_ling", 0);
    set_flag(ctx.core, owner, "mark:ssr_jinfan", 0);
    set_flag(ctx.core, owner, "l25_qixi_triggered", u32::from(tgt) + 1);

    if basic_cnt > 0 && !ctx.core.player(tgt).hand.is_empty() {
        let to_disc: Vec<CardId> = ctx
            .core
            .player(tgt)
            .hand
            .iter()
            .take(basic_cnt)
            .copied()
            .collect();
        discard_he_cards(ctx, tgt, &to_disc);
    }
    if trick_cnt > 0 && !ctx.core.player(tgt).equips.is_empty() {
        let to_disc: Vec<CardId> = ctx
            .core
            .player(tgt)
            .equips
            .iter()
            .take(trick_cnt)
            .copied()
            .collect();
        discard_he_cards(ctx, tgt, &to_disc);
    }
    m.card = Some(total_ling);
    if equip_cnt > 0 && ctx.core.player(tgt).alive {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            tgt,
            equip_cnt as i32,
            DamageNature::Normal,
            None,
        )));
        m.stage = HostStage::DrawSettle;
        return Flow::Yield;
    }
    if total_ling > 0 && ctx.core.player(owner).alive {
        ctx.start_draw(owner, total_ling);
        m.stage = HostStage::CardsAsk;
        return Flow::Yield;
    }
    Flow::Done
}

fn step_qixi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let others = alive_others(ctx.core, owner);
            if others.is_empty() {
                return Flow::Done;
            }
            if others.len() == 1 {
                return resolve_qixi(m, ctx, others[0]);
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIXI_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            resolve_qixi(m, ctx, tgt)
        }
        HostStage::DrawSettle => {
            let total_ling = m.card.unwrap_or(0);
            if total_ling > 0 && ctx.core.player(owner).alive {
                ctx.start_draw(owner, total_ling);
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn gain_zhuiji_card(ctx: &mut Ctx<'_>, owner: PlayerId, tgt: PlayerId, c: CardId) {
    if he_cards(ctx.core, tgt).contains(&c) {
        ctx.take_owned_card(tgt, c);
        ctx.core.player_mut(owner).hand.push(c);
        ctx.out.push(LogEvent::CardsTaken {
            from: tgt,
            to: owner,
            cards: vec![c],
            public: false,
        });
        set_flag(ctx.core, owner, "l25_zhuiji_gained", u32::from(c) + 1);
    }
}

fn step_zhuiji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tgt = m.target.unwrap_or(owner);
    match m.stage {
        HostStage::Act => {
            let he = he_cards(ctx.core, tgt);
            if he.is_empty() {
                return Flow::Done;
            }
            if he.len() == 1 {
                gain_zhuiji_card(ctx, owner, tgt, he[0]);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHUIJI_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            gain_zhuiji_card(ctx, owner, tgt, c);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_tieji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(2) {
        set_flag(ctx.core, owner, "l25_tieji_unresp", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_TIEJI_CONFIRM),
        HostStage::SelectNature => {
            let ok = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm)
            );
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if !ok {
                return Flow::Done;
            }
            for &tgt in &m.targets {
                set_flag(ctx.core, tgt, "tieji_sealed", 1);
                set_flag(ctx.core, tgt, "l25_tieji_sealed", 1);
            }
            if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                let jc = ctx.draw_judge_card();
                let jsuit = ctx.catalog.effective_card_suit(ctx.core, owner, jc);
                let existing = tieji_pile(ctx.core, owner);
                let dup_suit = existing.iter().any(|&c| {
                    ctx.catalog.effective_card_suit(ctx.core, owner, c) == jsuit
                });
                if dup_suit {
                    ctx.to_discard(jc);
                    set_flag(ctx.core, owner, "l25_tieji_unresp", 1);
                } else {
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((owner, "ssr_tieji".to_string()))
                        .or_default()
                        .push(jc);
                    let len = tieji_pile(ctx.core, owner).len() as u32;
                    set_flag(ctx.core, owner, "l25_tieji_jin", len);
                    set_flag(ctx.core, owner, "mark:ssr_tieji", len);
                }
            }
            set_flag(ctx.core, owner, "l25_tieji_triggered", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_youqin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) {
        if flag(ctx.core, owner, "l25_youqin_hun") == 0 {
            set_flag(ctx.core, owner, "l25_youqin_hun", 1);
            set_flag(ctx.core, owner, "l25_youqin_zi", 1);
            set_flag(ctx.core, owner, "l25_youqin_active", 1);
        }
        return Flow::Done;
    }
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let hun = flag(ctx.core, owner, "l25_youqin_hun").max(1);
    let zi = flag(ctx.core, owner, "l25_youqin_zi").max(1);
    set_flag(ctx.core, owner, "l25_youqin_hun", hun);
    set_flag(ctx.core, owner, "l25_youqin_zi", zi);
    set_flag(ctx.core, owner, "l25_youqin_active", 1);

    let mut fetched = Vec::new();
    if let Some(c1) = fetch_card_matching(ctx, |cid, cat| {
        let k = cat.card_kind(cid);
        if hun == 1 {
            is_normal_trick_kind(k)
        } else {
            is_delayed_trick_kind(k)
        }
    }) {
        fetched.push(c1);
    }
    if let Some(c2) = fetch_card_matching(ctx, |cid, cat| {
        let k = cat.card_kind(cid);
        if zi == 1 {
            k.is_basic_kind()
        } else {
            k.is_equip()
        }
    }) {
        fetched.push(c2);
    }
    if !fetched.is_empty() {
        for &c in &fetched {
            ctx.core.player_mut(owner).hand.push(c);
        }
        ctx.out.push(LogEvent::CardsDrawn {
            player: owner,
            cards: fetched.clone(),
        });
        set_flag(ctx.core, owner, "l25_youqin_fetched", fetched.len() as u32);
    }
    Flow::Done
}

fn resolve_fanjian(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    tgt: PlayerId,
    owner_card: CardId,
    target_card: CardId,
) -> Flow {
    let owner = m.owner;
    ctx.out.push(LogEvent::CardsRevealed {
        cards: vec![owner_card, target_card],
        to: Vec::new(),
    });
    let s1 = ctx.catalog.effective_card_suit(ctx.core, owner, owner_card);
    let s2 = ctx.catalog.effective_card_suit(ctx.core, tgt, target_card);
    set_flag(ctx.core, owner, "l25_fanjian_used", u32::from(tgt) + 1);
    if s1 == s2 {
        discard_he_cards(ctx, owner, &[owner_card]);
        discard_he_cards(ctx, tgt, &[target_card]);
        let next_zi = if flag(ctx.core, owner, "l25_youqin_zi") == 2 {
            1
        } else {
            2
        };
        set_flag(ctx.core, owner, "l25_youqin_zi", next_zi);
        m.stage = HostStage::CardsAsk;
        Flow::Yield
    } else {
        set_flag(ctx.core, tgt, "l25_fanjian_forbid_suit", suit_code(s1));
        let next_hun = if flag(ctx.core, owner, "l25_youqin_hun") == 2 {
            1
        } else {
            2
        };
        set_flag(ctx.core, owner, "l25_youqin_hun", next_hun);
        if ctx.core.player(tgt).alive {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Normal,
                Some(owner_card),
            )));
            m.stage = HostStage::CardsAsk;
            return Flow::Yield;
        }
        Flow::Done
    }
}

fn step_newfanjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        for p in 0..ctx.core.players.len() as PlayerId {
            set_flag(ctx.core, p, "l25_fanjian_forbid_suit", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_FANJIAN_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards = vec![c];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_FANJIAN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            if ctx.core.player(tgt).hand.len() == 1 {
                set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
                let tc = ctx.core.player(tgt).hand[0];
                let oc = m.cards.first().copied().unwrap_or(tc);
                return resolve_fanjian(m, ctx, tgt, oc, tc);
            }
            wait_ask(m, ctx, HostStage::JiefenOwnerDiscard, ASK_FANJIAN_SHOW)
        }
        HostStage::JiefenOwnerDiscard => {
            let tgt = m.target.unwrap_or(owner);
            let Some(ChoiceOption::Card(tc)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let oc = m.cards.first().copied().unwrap_or(tc);
            resolve_fanjian(m, ctx, tgt, oc, tc)
        }
        _ => Flow::Done,
    }
}

fn step_shichou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(2) {
        for p in 0..ctx.core.players.len() as PlayerId {
            set_flag(ctx.core, p, "l25_shichou_forbid_suit", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_SHICHOU_CONFIRM),
        HostStage::SelectNature => {
            let ok = matches!(
                ctx.answer().first().copied(),
                Some(ChoiceOption::Tag(1)) | Some(ChoiceOption::Confirm)
            );
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if !ok {
                return Flow::Done;
            }
            let jc = ctx.draw_judge_card();
            ctx.to_discard(jc);
            let sc = suit_code(ctx.catalog.effective_card_suit(ctx.core, owner, jc));
            for &tgt in &m.targets {
                set_flag(ctx.core, tgt, "l25_shichou_forbid_suit", sc);
            }
            set_flag(ctx.core, owner, "l25_shichou_triggered", sc.max(1));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_newrende(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        if let Some(p) = m.target {
            set_flag(ctx.core, p, "l25_rende_forbid_dmg_to", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_RENDE_CARD),
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards = vec![c];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_RENDE_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_newrende");
            set_flag(ctx.core, owner, "l25_rende_used", cnt);
            if let Some(&c) = m.cards.first() {
                if ctx.core.player(owner).hand.contains(&c) {
                    ctx.give_hand_card(owner, tgt, c);
                }
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_RENDE_TARGET_CHOICE)
        }
        HostStage::SelectNature => {
            let tgt = m.target.unwrap_or(owner);
            let choice = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(v)) => v,
                _ => 0,
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            if choice == 0 {
                ctx.recover(owner, 1, None);
                set_flag(
                    ctx.core,
                    tgt,
                    "l25_rende_forbid_dmg_to",
                    u32::from(owner) + 1,
                );
                Flow::Done
            } else if !ctx.core.deck.is_empty() && ctx.core.player(tgt).alive {
                ctx.use_virtual_card(owner, CardKind::Sha, vec![tgt]);
                m.stage = HostStage::CardsAsk;
                Flow::Yield
            } else {
                Flow::Done
            }
        }
        _ => Flow::Done,
    }
}

fn step_jijiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(2) {
        if let Some(p) = m.target {
            set_flag(ctx.core, p, "l25_jijiang_sha_plus1", 0);
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_JIJIANG_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            set_flag(ctx.core, tgt, "l25_jijiang_sha_plus1", 1);
            set_flag(ctx.core, owner, "l25_jijiang_target", u32::from(tgt) + 1);
            if let Some(def) = ctx.catalog.skill_by_name("ssr_shichou") {
                ctx.add_skill_until_turn_end(tgt, def);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_qiangxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(100) {
        let mks = qiangxi_marks(ctx.core, owner);
        set_qiangxi_marks(ctx.core, owner, mks);
        return Flow::Done;
    }
    if m.card == Some(101) {
        let add = m.cards.first().copied().unwrap_or(1) as u32;
        let mks = qiangxi_marks(ctx.core, owner) + add;
        set_qiangxi_marks(ctx.core, owner, mks);
        return Flow::Done;
    }
    if m.card == Some(102) {
        let add = m
            .cards
            .iter()
            .filter(|&&c| ctx.catalog.card_kind(c).equip_slot() == Some(EquipSlot::Weapon))
            .count()
            .max(1) as u32;
        let mks = qiangxi_marks(ctx.core, owner) + add;
        set_qiangxi_marks(ctx.core, owner, mks);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIANGXI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l25_batch_b_ask_id", 0);
            let cnt = inc_used_this_turn(ctx.core, owner, "ssr_qiangxi");
            set_flag(ctx.core, owner, "l25_qiangxi_used", cnt);
            let mks = qiangxi_marks(ctx.core, owner).saturating_sub(1);
            set_qiangxi_marks(ctx.core, owner, mks);
            if ctx.core.player(tgt).alive {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    tgt,
                    1,
                    DamageNature::Normal,
                    None,
                )));
                m.stage = HostStage::CardsAsk;
                return Flow::Yield;
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_qingji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l25_qingji_used", 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                owner,
                1,
                DamageNature::Normal,
                None,
            )));
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                if let Some(w) = fetch_card_matching(ctx, |cid, cat| {
                    cat.card_kind(cid).equip_slot() == Some(EquipSlot::Weapon)
                }) {
                    ctx.core.player_mut(owner).hand.push(w);
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: owner,
                        cards: vec![w],
                    });
                    set_flag(ctx.core, owner, "l25_qingji_weapon", u32::from(w) + 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
