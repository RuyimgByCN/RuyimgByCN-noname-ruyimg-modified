use std::sync::atomic::{AtomicU64, Ordering};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason};
use engine_native::state::GameMode;

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand { actor, expected_revision: e.revision(), dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed), action })
}

fn advance_to_play(e: &mut Engine) {
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play { return; }
        match submit(e, e.turn().current, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } => {},
            other => panic!("advance: {other:?}"),
        }
    }
    panic!("no play phase");
}

fn base() -> Engine {
    let e = Engine::new(953, 4);
    let mut snap = e.save();
    snap.core.mode = GameMode::Kingdoms;
    for p in &mut snap.core.players { p.revealed_generals = [true, false]; }
    snap.core.players[0].general_slots[0] = Some(22); // Liu Bei, Shu actor
    snap.core.players[1].general_slots[0] = Some(5); // Shu
    snap.core.players[2].general_slots[0] = Some(5); // Shu
    snap.core.players[3].general_slots[0] = Some(2); // Wei opponent
    let mut e = Engine::restore(&snap).unwrap();
    advance_to_play(&mut e);
    for p in 0..4 { e.debug_clear_hand(p); }
    e.debug_grant(0, "激将");
    e
}

fn activate(e: &mut Engine) {
    let id = e.catalog().skill_by_name("激将").unwrap();
    assert!(matches!(submit(e, 0, GameAction::ActivateSkill { skill: id }), CommandOutcome::Pending { .. }));
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().unwrap().clone();
    submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection })
}

fn main() {
    let snap = base().save();
    let mut a = Engine::restore(&snap).unwrap();
    let mut b = Engine::restore(&snap).unwrap();
    a.debug_place(1, "sha"); a.debug_place(2, "shan");
    b.debug_place(1, "shan"); b.debug_place(2, "sha");
    assert_eq!(a.view_for(0), b.view_for(0), "owner sees same view before activation");
    activate(&mut a); activate(&mut b);
    let ao = a.open_choice().unwrap().spec.options.clone();
    let bo = b.open_choice().unwrap().spec.options.clone();
    assert_eq!(ao, vec![ChoiceOption::Player(1)]);
    assert_eq!(bo, vec![ChoiceOption::Player(2)]);
    println!("hidden-card leakage: same visible view; helper options {:?} vs {:?}", ao, bo);

    let mut c = Engine::restore(&snap).unwrap();
    c.debug_place(1, "sha"); c.debug_place(2, "sha");
    activate(&mut c);
    assert_eq!(c.open_choice().unwrap().spec.options, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
    assert!(matches!(answer(&mut c, vec![ChoiceOption::Player(1)]), CommandOutcome::Pending { .. }));
    assert!(matches!(answer(&mut c, vec![ChoiceOption::Pass]), CommandOutcome::Accepted { .. }));
    assert!(c.open_choice().is_none());
    println!("multi-helper refusal: first Pass ended skill despite second Shu holding Sha");

    let mut d = Engine::new(951, 3);
    advance_to_play(&mut d);
    for p in 0..3 { d.debug_clear_hand(p); }
    d.debug_grant(0, "仁德");
    d.debug_set_hp(0, 3);
    let one = d.debug_place(0, "shan");
    d.debug_place(0, "shan");
    let rid = d.catalog().skill_by_name("仁德").unwrap();
    assert!(matches!(submit(&mut d, 0, GameAction::ActivateSkill { skill: rid }), CommandOutcome::Pending { .. }));
    assert!(matches!(answer(&mut d, vec![ChoiceOption::Card(one)]), CommandOutcome::Pending { .. }));
    assert!(matches!(answer(&mut d, vec![ChoiceOption::Player(1)]), CommandOutcome::Accepted { .. }));
    assert_eq!(d.debug_hp(0), 3);
    let second = submit(&mut d, 0, GameAction::ActivateSkill { skill: rid });
    assert!(matches!(second, CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)));
    println!("rende cumulative 1+1: second use rejected {:?}; hp={}", second, d.debug_hp(0));

    let mut h = Engine::new(981, 3);
    advance_to_play(&mut h);
    for p in 0..3 { h.debug_clear_hand(p); }
    h.debug_grant(1, "护驾");
    let h_snap = h.save();
    let mut h_shan = Engine::restore(&h_snap).unwrap();
    let mut h_sha = Engine::restore(&h_snap).unwrap();
    let attack_a = h_shan.debug_place(0, "sha");
    h_shan.debug_place(2, "shan");
    let attack_b = h_sha.debug_place(0, "sha");
    h_sha.debug_place(2, "sha");
    assert_eq!(h_shan.view_for(0), h_sha.view_for(0));
    let out_a = submit(&mut h_shan, 0, GameAction::PlayCard { card: attack_a, targets: vec![1] });
    let out_b = submit(&mut h_sha, 0, GameAction::PlayCard { card: attack_b, targets: vec![1] });
    assert!(matches!(out_a, CommandOutcome::Pending { .. }));
    assert_eq!(h_shan.open_choice().unwrap().spec.prompt, "hujia_respond");
    assert_eq!(h_shan.open_choice().unwrap().observer, 2);
    assert!(matches!(out_b, CommandOutcome::Accepted { .. }));
    println!("hujia query leak: same attacker view; helper Shan => public choice {:?}, helper Sha => no choice", h_shan.choice_view_for(Some(0)));

    let mut wrong_phase = Engine::new(982, 3);
    assert_eq!(wrong_phase.turn().phase, Phase::Judge);
    wrong_phase.debug_grant(0, "仁德");
    wrong_phase.debug_place(0, "shan");
    let rid = wrong_phase.catalog().skill_by_name("仁德").unwrap();
    let out = submit(&mut wrong_phase, 0, GameAction::ActivateSkill { skill: rid });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    println!("wrong-phase skill: Rende activated in Judge phase as {:?}", out);

    let card_count = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(card_count, 80);
    println!("native standard test deck: {card_count} cards");
}
