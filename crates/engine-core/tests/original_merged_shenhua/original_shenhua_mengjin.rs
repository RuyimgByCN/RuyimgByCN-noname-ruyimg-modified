//! Golden coverage for SHENHUA「猛进」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, move_one_card, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MENGJIN: u32 = 2_910_124_062;
const PANGDE: u32 = 4_110_795_715;
const TEST_MULTI_SHA: u32 = 2_910_124_063;
const TEST_DOUBLE_RESPONSE: u32 = 2_910_124_064;
const SHENHUA: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/shenhua/mengjin.skill"),
    "\nskill test_multi_sha(2910124063) locked { modifier sha_target_limit add 1; }\n",
    "skill test_double_response(2910124064) locked { modifier response_count add 1; }\n",
);

fn setup(player_count: u8, extra_skills: &[u32]) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("mengjin-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_718, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MENGJIN),
    });
    effects.extend(extra_skills.iter().map(|skill| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, seats)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn packages_publish_exact_skill_character_and_dependencies() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    let trigger = shenhua
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MENGJIN)
        .unwrap();
    assert_eq!(trigger.event, "sha_dodged");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_dodged_target_alive_has_discardable_hand_equip"
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    for dependency in ["standard", "shenhua"] {
        assert!(old.manifest.dependencies.iter().any(|entry| {
            entry.namespace == "official"
                && entry.package_id == dependency
                && entry.version == "0.1.0"
        }));
    }
    let pangde = old
        .character_index
        .iter()
        .find(|character| character.id == PANGDE)
        .unwrap();
    assert_eq!(pangde.name_key, "pangde");
    assert_eq!(pangde.max_hp, 4);
    assert_eq!(pangde.skills, [2, MENGJIN]);
}

#[test]
fn full_dodge_can_be_declined_or_discard_exactly_one_hand_or_equip_card() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_000);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_001);
    let hand = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_002);
    let mount = inject(&mut tx, target, ZoneKind::Equip, CardKind::HorseOff, 96_003);

    let (state, result) = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected play result: {:?}",
        result.result
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    let (offered, result) = resolve(state, target, vec![shan.raw()], false, 2);
    let confirm = result.choice.as_ref().unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(confirm.observer, owner);

    let (declined, result) = resolve(offered.clone(), owner, vec![], true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(declined.cards.get(hand).unwrap().owner, target);
    assert_eq!(declined.cards.get(mount).unwrap().owner, target);

    let (selecting, result) = resolve(offered, owner, vec![1], false, 4);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert!(options.contains(&hand.raw()));
    assert!(options.contains(&mount.raw()));
    let (discarded, result) = resolve(selecting, owner, vec![mount.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = discarded
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = discarded.cards.get(mount).unwrap();
    assert_eq!(card.zone, discard);
    assert_eq!(card.owner, PlayerId::NONE);
    assert!(!discarded.hand_cards(owner).contains(&mount));
}

#[test]
fn filter_skips_hit_and_dodge_without_a_remaining_target_card() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_010);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_011);
    let (state, _) = play(tx.commit().state, owner, sha, vec![target], 10);
    let (dodged, result) = resolve(state, target, vec![shan.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(dodged.players.get(target).unwrap().hp, 4);

    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_012);
    let remaining = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_013);
    let (hit, result) = play(tx.commit().state, owner, sha, vec![target], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(hit.players.get(target).unwrap().hp, 3);
    assert_eq!(hit.cards.get(remaining).unwrap().owner, target);
}

#[test]
fn wanjian_shan_is_not_a_sha_dodge() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let wanjian = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWanjian,
        96_020,
    );
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_021);
    let remaining = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_022);
    let (state, result) = play(tx.commit().state, owner, wanjian, vec![], 20);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    let (resolved, result) = resolve(state, target, vec![shan.raw()], false, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.cards.get(remaining).unwrap().owner, target);
}

#[test]
fn two_shan_requirement_emits_only_after_the_second_response() {
    let (state, owner, seats) = setup(2, &[TEST_DOUBLE_RESPONSE]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_030);
    let shan1 = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_031);
    let shan2 = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_032);
    inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_033);
    let (state, _) = play(tx.commit().state, owner, sha, vec![target], 30);
    let (state, result) = resolve(state, target, vec![shan1.raw()], false, 31);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (_, result) = resolve(state, target, vec![shan2.raw()], false, 32);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
}

#[test]
fn multi_target_sha_resumes_after_the_dodged_targets_discard() {
    let (state, owner, seats) = setup(3, &[TEST_MULTI_SHA]);
    let (dodger, next_target) = (seats[1], seats[2]);
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_040);
    let shan = inject(&mut tx, dodger, ZoneKind::Hand, CardKind::Shan, 96_041);
    let discarded = inject(&mut tx, dodger, ZoneKind::Hand, CardKind::Other, 96_042);
    let next_hp = tx.state().players.get(next_target).unwrap().hp;
    let (state, result) = play(tx.commit().state, owner, sha, vec![dodger, next_target], 40);
    assert_eq!(result.choice.as_ref().unwrap().observer, dodger);
    let (state, result) = resolve(state, dodger, vec![shan.raw()], false, 41);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![1], false, 42);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (resolved, result) = resolve(state, owner, vec![discarded.raw()], false, 43);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(dodger).unwrap().hp, 4);
    assert_eq!(resolved.players.get(next_target).unwrap().hp, next_hp - 1);
}

#[test]
fn target_card_choice_revalidates_the_selected_card_at_resume() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_050);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_051);
    let selected = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_052);
    let (state, _) = play(tx.commit().state, owner, sha, vec![target], 50);
    let (state, _) = resolve(state, target, vec![shan.raw()], false, 51);
    let (selecting, result) = resolve(state, owner, vec![1], false, 52);
    assert!(result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == selected.raw()));

    let mut tx = Transaction::begin(selecting);
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    move_one_card(&mut tx, selected, hand, discard, PlayerId::NONE, true).unwrap();
    let moved = tx.commit().state;
    let (same, result) = resolve(moved, owner, vec![selected.raw()], false, 53);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(same.cards.get(selected).unwrap().zone, discard);
}
