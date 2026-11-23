use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, FrozenPackage, LockedStartMode, VerifiedLockedContent,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::{
    ChoiceOption, ChoiceView, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static DEDUP: AtomicU64 = AtomicU64::new(5_053_000);

pub fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub fn freeze_pkg(dir_name: &str) -> FrozenPackage {
    let pkg_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir_name);
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .unwrap_or_else(|e| panic!("read {dir_name}/package.npack.json: {e}"))
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .unwrap_or_else(|e| panic!("read {dir_name}/{rel_path}: {e}"))
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    validate_and_freeze_raw_package(raw_json, assets)
        .unwrap_or_else(|e| panic!("freeze {dir_name}: {e}"))
}

pub fn start_formal_ow(seed: u64, seats: usize) -> (VerifiedLockedContent, Engine) {
    let verified = prepare_locked_content_with_bridge(vec![freeze_pkg("shaai-ow-upstream-noname")])
        .expect("prepare shaai-ow formal package");
    let engine = start_locked_engine(seed, seats, LockedStartMode::Prototype, &verified)
        .expect("start locked shaai-ow engine");
    (verified, engine)
}

pub fn roundtrip_restore(e: &Engine, verified: &VerifiedLockedContent) -> Engine {
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode NGSA 107 archive");
    let restored = restore_locked_engine(&decoded, verified).expect("restore locked ow engine");
    assert_eq!(restored.state_hash(), e.state_hash());
    restored
}

pub fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().expect("expected open choice").clone();
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    )
}

pub fn answer_cancel(e: &mut Engine) -> CommandOutcome {
    let c = e.open_choice().expect("expected cancelable choice").clone();
    let sel = if c.spec.allow_cancel {
        Vec::new()
    } else if c.spec.options.contains(&ChoiceOption::Pass) {
        vec![ChoiceOption::Pass]
    } else {
        panic!("choice {:?} is neither allow_cancel nor has Pass", c.spec.prompt);
    };
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    )
}

pub fn drain_cancel(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.allow_cancel {
            Vec::new()
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
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

pub fn advance_to_play(e: &mut Engine, target: u8) {
    for _ in 0..96 {
        if e.turn().current == target && e.turn().phase == Phase::Play && e.open_choice().is_none()
        {
            break;
        }
        if e.open_choice().is_some() {
            drain_cancel(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, target, "expected turn of seat {target}");
    assert_eq!(e.turn().phase, Phase::Play, "expected Play phase");
}

pub fn advance_one_turn_to_play(e: &mut Engine, target: u8) {
    let cur = e.turn().current;
    let _ = submit(e, cur, GameAction::NextPhase);
    drain_cancel(e);
    advance_to_play(e, target);
}

pub fn clear_all_hands(e: &mut Engine) {
    let n = e.state_core().players.len() as u8;
    for p in 0..n {
        e.debug_clear_hand(p);
    }
}

pub fn activate(e: &mut Engine, actor: u8, skill_name: &str) -> CommandOutcome {
    let def = e
        .catalog()
        .skill_by_name(skill_name)
        .unwrap_or_else(|| panic!("skill {skill_name} not in catalog"));
    submit(e, actor, GameAction::ActivateSkill { skill: def })
}

pub fn play(e: &mut Engine, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    submit(e, actor, GameAction::PlayCard { card, targets })
}

pub fn advance_to_turn_end_prompt(e: &mut Engine, target: u8, expected_prompt: &str) {
    for _ in 0..8 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == expected_prompt {
                return;
            }
            drain_cancel(e);
            continue;
        }
        if e.turn().current != target {
            break;
        }
        let _ = submit(e, target, GameAction::NextPhase);
    }
    assert_open_prompt(e, expected_prompt, target);
}

pub fn place_any_suit(e: &mut Engine, player: u8, suit: Suit) -> u32 {
    let matches_card = |cid: u32| -> bool {
        let def = &e.catalog().cards[cid as usize];
        def.suit == suit && !def.name.starts_with("ow_")
    };
    let deck_id = e.state_core().deck.iter().copied().find(|&c| matches_card(c));
    let cid = if let Some(c) = deck_id {
        e.state_core_mut().deck.retain(|&x| x != c);
        c
    } else if let Some(c) = e
        .state_core()
        .discard
        .iter()
        .copied()
        .find(|&c| matches_card(c))
    {
        e.state_core_mut().discard.retain(|&x| x != c);
        c
    } else {
        panic!("no non-ow card matching suit {suit:?} in deck/discard");
    };
    e.state_core_mut().player_mut(player).hand.push(cid);
    cid
}

pub fn place_kind(e: &mut Engine, player: u8, kind: CardKind) -> u32 {
    let matches_card = |cid: u32| -> bool {
        let def = &e.catalog().cards[cid as usize];
        def.kind == kind && !def.name.starts_with("ow_")
    };
    let deck_id = e.state_core().deck.iter().copied().find(|&c| matches_card(c));
    let cid = if let Some(c) = deck_id {
        e.state_core_mut().deck.retain(|&x| x != c);
        c
    } else if let Some(c) = e
        .state_core()
        .discard
        .iter()
        .copied()
        .find(|&c| matches_card(c))
    {
        e.state_core_mut().discard.retain(|&x| x != c);
        c
    } else {
        panic!("no non-ow card matching kind {kind:?} in deck/discard");
    };
    e.state_core_mut().player_mut(player).hand.push(cid);
    cid
}

pub fn place_kind_suit(e: &mut Engine, player: u8, kind: CardKind, suit: Suit) -> u32 {
    let matches_card = |cid: u32| -> bool {
        let def = &e.catalog().cards[cid as usize];
        def.kind == kind && def.suit == suit && !def.name.starts_with("ow_")
    };
    let deck_id = e.state_core().deck.iter().copied().find(|&c| matches_card(c));
    let cid = if let Some(c) = deck_id {
        e.state_core_mut().deck.retain(|&x| x != c);
        c
    } else if let Some(c) = e
        .state_core()
        .discard
        .iter()
        .copied()
        .find(|&c| matches_card(c))
    {
        e.state_core_mut().discard.retain(|&x| x != c);
        c
    } else {
        panic!("no card matching ({kind:?}, {suit:?}) in deck/discard");
    };
    e.state_core_mut().player_mut(player).hand.push(cid);
    cid
}

pub fn assert_card_conservation(e: &Engine, ctx: &str) {
    let core = e.state_core();
    let mut seen = BTreeSet::new();
    let mut count = 0usize;
    let mut add = |cid: u32, zone: &str| {
        assert!(
            seen.insert(cid),
            "[{ctx}] duplicate card {cid} ({}) in zone {zone}",
            e.catalog().cards[cid as usize].name
        );
        count += 1;
    };
    for (idx, p) in core.players.iter().enumerate() {
        for &c in &p.hand {
            add(c, &format!("p{idx}.hand"));
        }
        for &c in &p.equips {
            add(c, &format!("p{idx}.equips"));
        }
        for &c in &p.judge {
            add(c, &format!("p{idx}.judge"));
        }
        for &c in &p.zhucheng {
            add(c, &format!("p{idx}.zhucheng"));
        }
        for &c in &p.lianyaohu_store {
            add(c, &format!("p{idx}.lianyaohu"));
        }
    }
    for &c in &core.deck {
        add(c, "deck");
    }
    for &c in &core.discard {
        add(c, "discard");
    }
    for &c in &core.table {
        add(c, "table");
    }
    for &(c, _) in &core.detached_cards {
        add(c, "detached_cards");
    }
    for ((p, k), pile) in &core.vm_aux.piles {
        for &c in pile {
            add(c, &format!("vm_aux.piles[({p},{k})]"));
        }
    }
    assert_eq!(
        count,
        e.catalog().cards.len(),
        "[{ctx}] tracked cards {count} != catalog cards {}",
        e.catalog().cards.len()
    );
}

pub fn assert_open_prompt(e: &Engine, expected_prompt: &str, expected_observer: u8) {
    let c = e
        .open_choice()
        .unwrap_or_else(|| panic!("expected open prompt {expected_prompt}, got None"));
    assert_eq!(c.spec.prompt, expected_prompt);
    assert_eq!(c.observer, expected_observer);
    let ChoiceView::Own { .. } = e.ai_session(expected_observer).choice().unwrap() else {
        panic!("expected Own choice view for observer {expected_observer}");
    };
}
