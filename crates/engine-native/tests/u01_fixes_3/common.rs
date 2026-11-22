use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

pub use content_native_loader::{bridge_pack, decision34_pack9_for_coexistence};
pub use engine_native::ai::BoundedPlanner;
pub use engine_native::archive::{decode_archive, encode_archive};
pub use engine_native::catalog::{CardDef, CardKind, Faction, Suit};
pub use engine_native::content::extension::ExtensionPack;
pub use engine_native::event::Phase;
pub use engine_native::vocabulary::generals::Gender;
pub use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

static DEDUP: AtomicU64 = AtomicU64::new(9_300_000);

pub fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

pub fn build_u01_packs() -> (ExtensionPack, ExtensionPack) {
    let d3_json = std::fs::read_to_string(
        repo_extensions_root().join("d3-upstream-noname/assets/character.json"),
    )
    .expect("read d3-upstream-noname character.json")
    .replace("\r\n", "\n");
    let d3_pack = bridge_pack(&d3_json).expect("bridge d3 character.json").pack;
    let p9_pack = decision34_pack9_for_coexistence(&[&d3_pack]);
    (d3_pack, p9_pack)
}

pub fn new_clean_engine(seed: u64, seats: u8) -> (Engine, ExtensionPack, ExtensionPack) {
    let (d3_pack, p9_pack) = build_u01_packs();
    let mut e = Engine::new_with_extensions(seed, usize::from(seats), &[&d3_pack, &p9_pack])
        .expect("new u01 engine");
    e.state_core_mut().skills.clear();
    for p in 0..seats {
        clear_hand(&mut e, p);
        clear_equips(&mut e, p);
        clear_judge(&mut e, p);
    }
    (e, d3_pack, p9_pack)
}

pub fn new_identity_clean_engine(seed: u64, seats: u8) -> (Engine, ExtensionPack, ExtensionPack) {
    let (d3_pack, p9_pack) = build_u01_packs();
    let mut e =
        Engine::try_new_identity_with_extensions(seed, usize::from(seats), &[&d3_pack, &p9_pack])
            .expect("new u01 identity engine");
    e.state_core_mut().skills.clear();
    for p in 0..seats {
        clear_hand(&mut e, p);
        clear_equips(&mut e, p);
        clear_judge(&mut e, p);
    }
    (e, d3_pack, p9_pack)
}

pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub fn answer(e: &mut Engine, actor: u8, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().expect("open_choice expected").clone();
    assert_eq!(c.observer, actor, "unexpected observer for {}", c.spec.prompt);
    let out = submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "unexpected reject answering {}: {out:?}",
        c.spec.prompt
    );
    out
}

#[allow(dead_code)]
pub fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.allow_cancel {
            Vec::new()
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

pub fn find_card_by<F: Fn(u32, &CardDef) -> bool>(e: &Engine, pred: F) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .skip(1)
        .find(|(id, def)| pred(*id as u32, def))
        .map(|(id, _)| id as u32)
        .expect("matching card in catalog")
}

pub fn find_nth_card_by<F: Fn(u32, &CardDef) -> bool>(e: &Engine, nth: usize, pred: F) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(id, def)| pred(*id as u32, def))
        .nth(nth)
        .map(|(id, _)| id as u32)
        .expect("nth matching card in catalog")
}

pub fn remove_card_from_all_zones(e: &mut Engine, card: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|c| *c != card);
    core.discard.retain(|c| *c != card);
    core.table.retain(|c| *c != card);
    for p in &mut core.players {
        p.hand.retain(|c| *c != card);
        p.equips.retain(|c| *c != card);
        p.judge.retain(|c| *c != card);
        p.off_court.retain(|c| *c != card);
    }
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|c| *c != card);
    }
}

pub fn put_in_hand(e: &mut Engine, seat: u8, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().player_mut(seat).hand.push(card);
}

pub fn put_in_equip(e: &mut Engine, seat: u8, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().player_mut(seat).equips.push(card);
}

#[allow(dead_code)]
pub fn put_in_judge(e: &mut Engine, seat: u8, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().player_mut(seat).judge.push(card);
}

pub fn put_deck_top(e: &mut Engine, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().deck.push(card);
}

#[allow(dead_code)]
pub fn put_deck_bottom(e: &mut Engine, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().deck.insert(0, card);
}

pub fn clear_hand(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    e.state_core_mut().deck.extend(cards);
}

pub fn clear_equips(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    e.state_core_mut().deck.extend(cards);
}

pub fn clear_judge(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).judge.drain(..).collect();
    e.state_core_mut().deck.extend(cards);
}

pub fn advance_to_prepare(e: &mut Engine, target_seat: u8, cur_round: u32) {
    let seats = e.state_core().players.len() as u8;
    let prev = (target_seat + seats - 1) % seats;
    e.debug_set_turn(prev, Phase::End, cur_round);
    submit(e, prev, GameAction::NextPhase);
}

pub fn flag(e: &Engine, seat: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(seat, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub fn has_skill(e: &Engine, seat: u8, name: &str) -> bool {
    e.catalog()
        .skill_by_name(name)
        .is_some_and(|def| e.state_core().has_skill(seat, def))
}

pub fn find_general_with_skill(e: &Engine, skill: &str) -> Option<u16> {
    (0..e.catalog().general_count() as u16).find(|&idx| {
        e.catalog()
            .general_meta(idx)
            .is_some_and(|m| m.2.contains(&skill))
    })
}

pub fn assert_mid_choice_packs(
    e: &Engine,
    packs: &[&ExtensionPack],
    selection: Vec<ChoiceOption>,
) {
    let choice = e.open_choice().expect("open choice must exist").clone();
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("decode archive");
    let mut restored = Engine::restore_with_extensions(&snap, packs).expect("restore");
    assert_eq!(e.state_hash(), restored.state_hash(), "restore hash mismatch");
    assert_eq!(
        e.open_choice()
            .map(|c| (&c.spec.prompt, &c.spec.options, c.observer)),
        restored
            .open_choice()
            .map(|c| (&c.spec.prompt, &c.spec.options, c.observer)),
        "restored open choice mismatch"
    );

    let planner = BoundedPlanner::new(8);
    let plan = e.plan_action(choice.observer, &planner);
    assert!(
        matches!(
            plan.action,
            GameAction::Answer { ref selection, .. }
            if choice.spec.accepts(selection)
        ),
        "AI must produce legal answer for {}: {:?}",
        choice.spec.prompt,
        plan.action
    );

    let mut live = Engine::restore_with_extensions(&e.save(), packs).expect("clone");
    let log_start = live.log().len();
    let _ = submit(
        &mut live,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection: selection.clone(),
        },
    );
    let _ = submit(
        &mut restored,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    );
    assert_eq!(live.state_hash(), restored.state_hash(), "post-answer hash mismatch");
    assert_eq!(&live.log()[log_start..], restored.log(), "post-answer log mismatch");
}

pub fn assert_mid_choice_archive_and_ai(
    e: &Engine,
    d3: &ExtensionPack,
    p9: &ExtensionPack,
    selection: Vec<ChoiceOption>,
) {
    assert_mid_choice_packs(e, &[d3, p9], selection);
}
