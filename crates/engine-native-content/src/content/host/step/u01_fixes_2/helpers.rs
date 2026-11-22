//! Shared predicates, target filters, card/trick tables, and card-move helpers for `u01_fixes_2`.

use super::*;

pub(super) fn encode_respond_need(need: CardKind) -> u32 {
    if need.is_sha_like() {
        1
    } else if need == CardKind::Nullify {
        3
    } else if need == CardKind::Tao {
        4
    } else {
        2
    }
}

pub(super) fn decode_respond_need(code: u32) -> CardKind {
    match code {
        1 => CardKind::Sha,
        3 => CardKind::Nullify,
        4 => CardKind::Tao,
        _ => CardKind::Shan,
    }
}

pub(super) fn is_u01_shown_card(core: &GameStateCore, owner: PlayerId, card: CardId) -> bool {
    flag(core, owner, &format!("u01_shown_card:{card}")) != 0
}

pub(super) fn dddfusi_all_shown(core: &GameStateCore, owner: PlayerId) -> bool {
    let hand = &core.player(owner).hand;
    !hand.is_empty() && hand.iter().all(|&c| is_u01_shown_card(core, owner, c))
}

pub(super) fn dddfusi_unshown_hand(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| !is_u01_shown_card(core, owner, c))
        .collect()
}

pub(super) fn dddfusi_sources_for(
    core: &GameStateCore,
    catalog: &Catalog,
    borrower: PlayerId,
) -> Vec<PlayerId> {
    let Some(fusi_def) = catalog.skill_by_name("dddfusi") else {
        return Vec::new();
    };
    let n = core.players.len() as PlayerId;
    let cur = core.turn.current;
    (0..n)
        .map(|off| (cur + off) % n)
        .filter(|&p| {
            p != borrower
                && core.player(p).alive
                && core.has_skill(p, fusi_def)
                && !catalog.is_tieji_sealed(core, p)
        })
        .collect()
}

pub(super) fn dddfusi_play_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    borrower: PlayerId,
    card: CardId,
) -> Option<Vec<PlayerId>> {
    if catalog.blocks_all_cards(core, borrower) {
        return None;
    }
    let kind = catalog.card_kind(card);
    if kind == CardKind::Shan || kind == CardKind::Nullify {
        return None;
    }
    if crate::rvm_bridge::modifiers::eval_card_use_canceled(
        core,
        catalog,
        borrower,
        card,
        kind,
        DamageNature::Normal,
    ) {
        return None;
    }
    if kind.is_sha_like() {
        if !can_use_sha_play_phase(core, catalog, borrower) {
            return None;
        }
        let tgts = virtual_legal_targets(core, catalog, borrower, kind);
        return (!tgts.is_empty()).then_some(tgts);
    }
    if kind == CardKind::Jiu {
        let fentian_red_jiu = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        ) && catalog
            .skill_by_name(crate::content::standard::BOSS_FENTIAN)
            .is_some_and(|def| core.has_skill(borrower, def));
        let bushi_unlimited = crate::rvm_bridge::modifiers::d34_l28_mods::bushi_unlimited_card(
            core,
            catalog,
            borrower,
            Some(card),
        );
        if core.turn.jiu_used && !fentian_red_jiu && !bushi_unlimited {
            return None;
        }
    }
    if kind.is_equip() {
        return Some(vec![borrower]);
    }
    if is_delayed(kind) {
        let tgts: Vec<PlayerId> = match kind {
            CardKind::ShanDian | CardKind::HongShui => {
                if !has_delayed_kind(core, catalog, borrower, kind)
                    && !is_target_protected(core, borrower)
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core,
                        catalog,
                        borrower,
                        borrower,
                        card,
                        kind,
                        DamageNature::Normal,
                    )
                {
                    vec![borrower]
                } else {
                    Vec::new()
                }
            }
            _ => core
                .alive_players()
                .filter(|&p| {
                    p != borrower
                        && !has_delayed_kind(core, catalog, p, kind)
                        && !is_target_protected(core, p)
                        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            core,
                            catalog,
                            borrower,
                            p,
                            card,
                            kind,
                            DamageNature::Normal,
                        )
                })
                .collect(),
        };
        return (!tgts.is_empty()).then_some(tgts);
    }
    if !virtual_kind_can_use_play(core, catalog, borrower, kind) {
        return None;
    }
    let tgts = virtual_legal_targets(core, catalog, borrower, kind);
    (!tgts.is_empty()).then_some(tgts)
}

pub(super) fn dddfusi_card_kind_usable_play(
    core: &GameStateCore,
    catalog: &Catalog,
    borrower: PlayerId,
    card: CardId,
) -> bool {
    dddfusi_play_targets(core, catalog, borrower, card).is_some()
}

pub(super) fn dddfusi_usable_play_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    borrower: PlayerId,
) -> Vec<CardId> {
    if catalog.blocks_all_cards(core, borrower) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for src in dddfusi_sources_for(core, catalog, borrower) {
        for &c in &core.player(src).hand {
            if is_u01_shown_card(core, src, c)
                && dddfusi_card_kind_usable_play(core, catalog, borrower, c)
            {
                out.push(c);
            }
        }
    }
    out
}

pub(super) fn dddfusi_card_matches_need(
    core: &GameStateCore,
    catalog: &Catalog,
    borrower: PlayerId,
    card: CardId,
    need: CardKind,
) -> bool {
    let kind = catalog.card_kind(card);
    if need == CardKind::Tao {
        if kind == CardKind::Tao {
            return true;
        }
        return kind == CardKind::Jiu && core.player(borrower).dying;
    }
    if need.is_sha_like() {
        return kind.is_sha_like();
    }
    kind == need
}

pub(super) fn dddfusi_respond_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    borrower: PlayerId,
    need: CardKind,
) -> Vec<CardId> {
    if catalog.blocks_all_cards(core, borrower) {
        return Vec::new();
    }
    if need == CardKind::Tao
        && core
            .vm_aux
            .flags
            .get(&(borrower, "tao_futai2_banned".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
    {
        return Vec::new();
    }
    let mut out = Vec::new();
    for src in dddfusi_sources_for(core, catalog, borrower) {
        for &c in &core.player(src).hand {
            if is_u01_shown_card(core, src, c)
                && dddfusi_card_matches_need(core, catalog, borrower, c, need)
            {
                out.push(c);
            }
        }
    }
    out
}

pub(super) fn find_shown_card_owner(
    core: &GameStateCore,
    catalog: &Catalog,
    borrower: PlayerId,
    card: CardId,
) -> Option<PlayerId> {
    dddfusi_sources_for(core, catalog, borrower)
        .into_iter()
        .find(|&src| core.player(src).hand.contains(&card) && is_u01_shown_card(core, src, card))
}

pub(super) fn ddd_jian_cards(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, "ddd_jian".to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn dddzhijie_can_respond(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    need: CardKind,
) -> bool {
    let cards = ddd_jian_cards(core, owner);
    if cards.len() < 2 {
        return false;
    }
    let reds = cards
        .iter()
        .filter(|&&c| effective_is_red(core, catalog, owner, c))
        .count();
    let blacks = cards.len() - reds;
    match need {
        CardKind::Shan => reds >= 2 || blacks >= 2,
        CardKind::Nullify => reds >= 1 && blacks >= 1,
        _ => false,
    }
}

pub(super) fn card_kind_group(kind: CardKind) -> u32 {
    if kind.is_basic_kind() {
        1
    } else if kind.is_equip() {
        3
    } else {
        2
    }
}

pub(super) fn chashi_card_matches(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    u: &CardUseData,
) -> bool {
    if flag(core, owner, "dddchashi_armed") == 0 {
        return false;
    }
    if flag(core, owner, "dddchashi_target") != u32::from(u.player) + 1 {
        return false;
    }
    let want_suit = flag(core, owner, "dddchashi_suit");
    let want_group = flag(core, owner, "dddchashi_kind");
    let eff_kind = u.effective_kind(catalog);
    let kind_match = card_kind_group(eff_kind) == want_group;
    let suit_match = u
        .card
        .is_some_and(|c| effective_suit_bit(core, catalog, u.player, c) == want_suit);
    kind_match || suit_match
}

pub(super) fn effective_judge_kind(
    core: &GameStateCore,
    catalog: &Catalog,
    card: CardId,
) -> CardKind {
    core.declared_kinds
        .get(&card)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(card))
}

pub(super) fn has_bingliang(core: &GameStateCore, catalog: &Catalog, target: PlayerId) -> bool {
    core.player(target)
        .judge
        .iter()
        .any(|&c| effective_judge_kind(core, catalog, c) == CardKind::Bing)
}

pub(super) fn has_delayed_kind(
    core: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    kind: CardKind,
) -> bool {
    core.player(target)
        .judge
        .iter()
        .any(|&c| effective_judge_kind(core, catalog, c) == kind)
}

pub(super) fn dddxuxiao_valid_targets(core: &GameStateCore, catalog: &Catalog) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| !has_bingliang(core, catalog, p) && !is_target_protected(core, p))
        .collect()
}

pub(super) fn dddxuxiao_valid_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    candidates: &[CardId],
) -> Vec<CardId> {
    if dddxuxiao_valid_targets(core, catalog).is_empty() {
        return Vec::new();
    }
    candidates
        .iter()
        .copied()
        .filter(|&c| {
            core.discard.contains(&c)
                && is_black(catalog, c)
                && catalog.card_kind(c).is_basic_kind()
        })
        .collect()
}

const CORE_FENJI_TRICKS: [(u8, CardKind); 10] = [
    (0, CardKind::Duel),
    (1, CardKind::Dismantle),
    (2, CardKind::Snatch),
    (3, CardKind::Wuzhong),
    (4, CardKind::Taoyuan),
    (5, CardKind::Wugu),
    (6, CardKind::SavageAttack),
    (7, CardKind::ArcheryAttack),
    (8, CardKind::Chain),
    (9, CardKind::Jiedao),
];

pub(super) fn dddfenji_trick_table(catalog: &Catalog) -> Vec<(u8, CardKind)> {
    let mut table: Vec<(u8, CardKind)> = CORE_FENJI_TRICKS.to_vec();
    let mut next_tag = 10u8;
    for def in &catalog.cards {
        if def.off_deck {
            continue;
        }
        let k = def.kind;
        if k.is_basic_kind() || k.is_equip() || is_delayed(k) || k == CardKind::Nullify {
            continue;
        }
        if !k.is_basic_trick_or_equip() {
            continue;
        }
        if table.iter().any(|(_, existing)| *existing == k) {
            continue;
        }
        if next_tag < 32 {
            table.push((next_tag, k));
            next_tag += 1;
        }
    }
    table
}

pub(super) fn dddfenji_kind_for_tag(catalog: &Catalog, tag: u8) -> Option<CardKind> {
    dddfenji_trick_table(catalog)
        .into_iter()
        .find_map(|(t, k)| (t == tag).then_some(k))
}

pub(super) fn dddfenji_legal_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    kind: CardKind,
) -> Vec<PlayerId> {
    if kind == CardKind::Taoyuan {
        return core
            .alive_players()
            .filter(|&t| !is_target_protected(core, t))
            .collect();
    }
    virtual_legal_targets(core, catalog, user, kind)
}

pub(super) fn dddfenji_usable_tricks(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<(u8, CardKind)> {
    if catalog.blocks_all_cards(core, owner) {
        return Vec::new();
    }
    let used_mask = flag(core, owner, "dddfenji_used_mask");
    dddfenji_trick_table(catalog)
        .into_iter()
        .filter(|&(tag, kind)| {
            (used_mask & (1u32 << tag)) == 0
                && !crate::rvm_bridge::modifiers::eval_card_use_canceled(
                    core,
                    catalog,
                    owner,
                    0,
                    kind,
                    DamageNature::Normal,
                )
                && !dddfenji_legal_targets(core, catalog, owner, kind).is_empty()
        })
        .collect()
}

pub(super) fn dddlingyong_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
) -> Option<Vec<PlayerId>> {
    let kind = catalog.card_kind(card);
    if kind.is_sha_like() || kind == CardKind::Shan || kind == CardKind::Nullify {
        return None;
    }
    if catalog.blocks_all_cards(core, owner) {
        return None;
    }
    if kind.is_equip() {
        return Some(vec![owner]);
    }
    if is_delayed(kind) {
        let targets: Vec<PlayerId> = match kind {
            CardKind::ShanDian => {
                if !has_delayed_kind(core, catalog, owner, kind) && !is_target_protected(core, owner)
                {
                    vec![owner]
                } else {
                    Vec::new()
                }
            }
            _ => core
                .alive_players()
                .filter(|&p| {
                    p != owner
                        && !has_delayed_kind(core, catalog, p, kind)
                        && !is_target_protected(core, p)
                })
                .collect(),
        };
        return (!targets.is_empty()).then_some(targets);
    }
    if !virtual_kind_can_use_play(core, catalog, owner, kind) {
        return None;
    }
    let targets = virtual_legal_targets(core, catalog, owner, kind);
    (!targets.is_empty()).then_some(targets)
}

pub(super) fn dddlingyong_needs_target_prompt(catalog: &Catalog, card: CardId) -> bool {
    let kind = catalog.card_kind(card);
    virtual_kind_needs_target_prompt(kind) || matches!(kind, CardKind::Le | CardKind::Bing)
}

pub(super) fn dddzhengjun_match_targets(
    core: &GameStateCore,
    owner: PlayerId,
    affected: PlayerId,
    candidates: &[PlayerId],
    branch: u8,
) -> Vec<PlayerId> {
    let matches_stat = |p: PlayerId| -> bool {
        if !core.player(p).alive {
            return false;
        }
        match branch {
            1 => core.player(p).hp == core.player(owner).hp,
            2 => core.player(p).hand.len() == core.player(owner).hand.len(),
            3 => core.player(p).equips.len() == core.player(owner).equips.len(),
            _ => false,
        }
    };
    let mut out = Vec::new();
    let pool: &[PlayerId] = if candidates.is_empty() {
        core::slice::from_ref(&affected)
    } else {
        candidates
    };
    for &p in pool {
        if matches_stat(p) && !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

pub(super) fn dddzhengjun_movable_field_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    src: PlayerId,
) -> Vec<CardId> {
    if !core.player(src).alive {
        return Vec::new();
    }
    let mut cards = Vec::new();
    for &c in core
        .player(src)
        .equips
        .iter()
        .chain(core.player(src).judge.iter())
    {
        if !zhishu_valid_dests(core, catalog, c).is_empty() {
            cards.push(c);
        }
    }
    cards
}

pub(super) fn dddtaisi_discard_cards(core: &GameStateCore) -> Vec<CardId> {
    core.discard
        .iter()
        .copied()
        .filter(|&c| flag(core, 0, &format!("turn_discard_card:{c}")) != 0)
        .collect()
}

pub(super) fn dddmiaoxing_split_3(cards: &[CardId]) -> [Vec<CardId>; 3] {
    let n = cards.len();
    let base = n / 3;
    let rem = n % 3;
    let end0 = base + usize::from(rem > 0);
    let end1 = end0 + base + usize::from(rem > 1);
    [
        cards[..end0].to_vec(),
        cards[end0..end1].to_vec(),
        cards[end1..].to_vec(),
    ]
}

pub(super) fn dddmiaoxing_mismatch_tags(core: &GameStateCore, owner: PlayerId) -> Vec<u8> {
    let hand_len = core.player(owner).hand.len();
    let mut tags = Vec::new();
    for (tag, key) in [(0u8, "dddmiaoxing_1"), (1u8, "dddmiaoxing_2")] {
        let pile_len = core
            .vm_aux
            .piles
            .get(&(owner, key.to_string()))
            .map_or(0, Vec::len);
        if pile_len != hand_len {
            tags.push(tag);
        }
    }
    tags
}

pub(super) fn wait_ask_2(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = build_ask_2(
        ask_id,
        machine.owner,
        machine.target,
        &machine.targets,
        machine.card,
        &machine.cards,
        ctx.core,
        ctx.catalog,
    ) {
        machine.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        Flow::Done
    }
}

pub(super) fn dddquche_red_cards(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<CardId> {
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| effective_is_red(core, catalog, owner, c))
        .collect()
}

pub(super) fn dddquche_black_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| !effective_is_red(core, catalog, owner, c))
        .collect()
}

pub(super) fn draw_into_deck_if_needed(core: &mut GameStateCore) -> Option<CardId> {
    if core.deck.is_empty() && !core.discard.is_empty() {
        let mut reshuffled = std::mem::take(&mut core.discard);
        core.rng.shuffle(&mut reshuffled);
        crate::kernel::elo_on_wash_deck(core, &reshuffled);
        core.deck.extend(reshuffled);
    }
    core.deck.pop()
}

pub(super) fn dddxianxi_eligible_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    existing: &[PlayerId],
    nature: DamageNature,
) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&p| {
            !existing.contains(&p)
                && p != owner
                && !is_sha_target_blocked(core, catalog, owner, p, CardKind::Sha, nature)
        })
        .collect()
}
