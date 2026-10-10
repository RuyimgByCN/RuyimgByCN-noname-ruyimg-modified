use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::content::extension::{example_extension, merge_extension_cards, ExtensionRegistry};
use engine_native::view::{PolicyInput, SeatPolicy};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot, Phase,
};

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action,
    })
}

fn play_ready(seed: u64, seats: u8) -> Engine {
    let mut e = Engine::new(seed, seats as usize);
    for _ in 0..12 {
        if e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        assert!(!matches!(submit(&mut e, actor, GameAction::NextPhase), CommandOutcome::Rejected(_)));
    }
    for p in 0..seats {
        e.debug_clear_hand(p);
    }
    e
}

struct Passive;
impl SeatPolicy for Passive {
    fn act(&mut self, _: &PolicyInput) -> Option<GameAction> { None }
}

fn main() {
    let mut e = Engine::new(2301, 3);
    assert!(!matches!(e.submit(GameCommand {
        actor: 0, expected_revision: e.revision(), dedup_id: 1_000_000,
        action: GameAction::NextPhase,
    }), CommandOutcome::Rejected(_)));
    let saved = e.save();
    let mut policy_engine = Engine::restore(&saved).unwrap();
    let policy_out = policy_engine.run_policy(0, &mut Passive);
    assert!(matches!(policy_out, CommandOutcome::Accepted { .. }));
    let mut plan_engine = Engine::restore(&saved).unwrap();
    let plan = plan_engine.plan_action(0, &BoundedPlanner::new(64));
    let plan_out = plan_engine.apply_plan(&plan).unwrap();
    assert!(matches!(plan_out, CommandOutcome::Accepted { .. }));
    println!("FIXED_AI_DEDUP: policy=accepted; plan=accepted");

    for (name, prompt, response) in [
        ("duel", "duel_sha", "sha"),
        ("savage", "aoe_response", "sha"),
        ("snatch", "nullify_window", "nullify"),
    ] {
        let mut e = play_ready(2401, 3);
        e.debug_place(1, response);
        let card = e.debug_place(0, name);
        let targets = if name == "savage" { vec![] } else { vec![1] };
        assert!(!matches!(submit(&mut e, 0, GameAction::PlayCard { card, targets }), CommandOutcome::Rejected(_)));
        assert_eq!(e.open_choice().unwrap().spec.prompt, prompt);
        let decoded = decode_archive(&encode_archive(&e.save())).unwrap();
        if prompt == "nullify_window" {
            let error = Engine::restore(&decoded).err().expect("nullify mismatch");
            println!("OPEN_ARCHIVE_CHOICE: {prompt} byte restore rejected: {error:?}");
        } else {
            Engine::restore(&decoded).expect("choice byte roundtrip");
            println!("FIXED_ARCHIVE_CHOICE: {prompt} byte roundtrip restores");
        }
    }

    let mut e = play_ready(2004, 3);
    e.debug_grant(1, "流离");
    e.debug_place(1, "tao");
    let sha = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "liuli_transfer");
    let decoded = decode_archive(&encode_archive(&e.save())).unwrap();
    Engine::restore(&decoded).expect("liuli byte roundtrip");
    println!("FIXED_LIULI_ARCHIVE: byte roundtrip restores");

    let e = Engine::new(443, 4);
    let mut snapshot = e.save();
    let highest = e.catalog().cards.len() as u32 - 1;
    for p in &mut snapshot.core.players {
        p.hand.retain(|c| *c != highest);
        p.equips.retain(|c| *c != highest);
        p.judge.retain(|c| *c != highest);
    }
    snapshot.core.deck.retain(|c| *c != highest);
    snapshot.core.discard.retain(|c| *c != highest);
    snapshot.core.table.retain(|c| *c != highest);
    assert!(Engine::restore(&snapshot).is_ok());
    println!("OPEN_TRAILING_MISSING: Card({highest}) absent, restore accepted");

    let mut e = Engine::new(2304, 3);
    redact_opponent_hands(&mut e, 0);
    let error = Engine::restore(&e.save()).err().unwrap();
    println!("OPEN_AI_SAMPLED_OWNERSHIP: {error:?}");

    let mut e = play_ready(2303, 3);
    let sha = e.debug_place(0, "sha");
    e.debug_place(1, "shan");
    e.debug_place(2, "sha");
    let mut snapshot = e.save();
    snapshot.core.discard.append(&mut snapshot.core.deck);
    let mut e = Engine::restore(&snapshot).unwrap();
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    let plan = e.plan_action(1, &BoundedPlanner::new(64));
    assert!(matches!(plan.action, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Pass]));
    println!("OPEN_AI_CANDIDATES: legal Shan omitted; selected {:?}", plan.action);

    let mut e = play_ready(2601, 3);
    e.debug_place(0, "sha");
    let low_plan = e.plan_action(0, &BoundedPlanner::new(64));
    let mut snapshot = e.save();
    snapshot.last_dedup = 2_000_000;
    let high = Engine::restore(&snapshot).unwrap();
    let high_plan = high.plan_action(0, &BoundedPlanner::new(64));
    assert!(matches!(low_plan.action, GameAction::PlayCard { .. }));
    assert!(matches!(high_plan.action, GameAction::NextPhase));
    println!("OPEN_AI_SIM_DEDUP: low={:?}, high={:?}", low_plan.action, high_plan.action);

    let mut e = play_ready(2201, 3);
    e.debug_grant(1, "护驾");
    e.debug_place(0, "shan");
    let sha = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "hujia_respond");
    println!("HUJIA_HOLDER_RESTORE: {:?}", Engine::restore(&e.save()).err().expect("holder mismatch"));

    let mut e = play_ready(2402, 3);
    e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    let mut snapshot = e.save();
    let mut changed = false;
    for park in &mut snapshot.resolution.parks {
        if let MachineSnapshot::Sha { hujia_holder, .. } = &mut park.machine {
            *hujia_holder = Some(254);
            changed = true;
        }
    }
    assert!(changed);
    assert!(Engine::restore(&snapshot).is_ok());
    println!("HUJIA_INVALID_HOLDER_ACCEPTED: true");

    let mut e = play_ready(2202, 4);
    e.debug_grant(1, "护驾");
    e.debug_place(2, "shan");
    e.debug_place(3, "shan");
    let sha = e.debug_place(0, "sha");
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().observer, 2);
    let choice = e.open_choice().unwrap().clone();
    submit(&mut e, 2, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Pass] });
    assert!(e.open_choice().is_none());
    println!("HUJIA_SECOND_HOLDER_ASKED: {}; defender_hp={}", e.open_choice().is_some(), e.debug_hp(1));

    let mut e = play_ready(2501, 4);
    let mut snapshot = e.save();
    for (p, g) in [(1, 2), (2, 9), (3, 5)] {
        snapshot.core.players[p].general_slots = [Some(g), None];
    }
    e = Engine::restore(&snapshot).unwrap();
    e.debug_grant(0, "离间");
    let card = e.debug_place(0, "tao");
    let skill = e.catalog().skill_by_name("离间").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill });
    let choice = e.open_choice().unwrap().clone();
    submit(&mut e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(card)] });
    println!("LIJIAN_GENDER_RESTORE: {:?}", Engine::restore(&e.save()).err().expect("gender mismatch"));

    let mut registry = ExtensionRegistry::new();
    registry.enable("not_installed");
    assert!(registry.is_enabled("not_installed") && !registry.is_installed("not_installed"));
    let ext = example_extension();
    let cards = vec![ext.cards[0].clone(), ext.cards[0].clone()];
    let merged = merge_extension_cards(vec![], &cards);
    assert_eq!(merged[0].name, merged[1].name);
    println!("OPEN_EXTENSION_VALIDATION: uninstalled enabled; duplicate extension cards accepted");
}
