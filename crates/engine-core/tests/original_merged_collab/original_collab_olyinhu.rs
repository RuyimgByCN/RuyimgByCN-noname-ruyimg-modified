//! Golden coverage for COLLAB「寅虎」and the complete ol_yinhu closure.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLYINHU: u32 = 4_066_124_366;
const OL_YINHU: u32 = 3_117_465_792;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    // Roster-independent: the standard roster gained choice-bearing characters
    // (gongsunzan 义从 / old_re_lidian 询询+忘隙) whose booted presence changes
    // range/hand-limit/trigger answers; this test grants its own skills.
    let booted: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted.is_empty() {
        run_effects(&mut tx, booted).unwrap();
    }

    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLYINHU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLYINHU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

fn enter_extra_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_abi110_play_phase_category_contract_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 156);
    assert_eq!(package.character_index.len(), 90);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLYINHU && skill.name == "olyinhu"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLYINHU)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 1)
    );
    assert!(selection.unused_category_by_skill_this_play_phase);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_YINHU
            && character.skills == vec![OLYINHU]
            && character.max_hp == 4
            && character.sex == "male"
            && character.group == "qun"
    }));
}

#[test]
fn categories_are_unique_per_play_phase_but_targets_may_repeat() {
    let mut setup = setup(2_026_072_304);
    let basic = inject_hand(&mut setup.state, setup.owner, 99_001, CardKind::Sha);
    let repeated_basic = inject_hand(&mut setup.state, setup.owner, 99_002, CardKind::Shan);
    let trick = inject_hand(
        &mut setup.state,
        setup.owner,
        99_003,
        CardKind::TrickWuzhong,
    );
    let equip = inject_hand(&mut setup.state, setup.owner, 99_004, CardKind::Weapon);
    let target = setup.targets[0];
    // Fixed HP so three one-damage hits never enter dying and ban the skill.
    setup.state.players.get_mut(target).unwrap().max_hp = 5;
    setup.state.players.get_mut(target).unwrap().hp = 5;
    let hp = 5;

    let (state, first) = activate(setup.state, setup.owner, target, setup.skill, basic, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b001
    );

    let before = canonical_state_hash(&state);
    let (state, repeated) = activate(state, setup.owner, target, setup.skill, repeated_basic, 2);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, second) = activate(state, setup.owner, target, setup.skill, trick, 3);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    let (mut state, third) = activate(state, setup.owner, target, setup.skill, equip, 4);
    assert!(matches!(third.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 3);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b111
    );
    assert!(!state.skills.get(setup.skill).unwrap().active_turn_banned);

    state.players.get_mut(target).unwrap().hp = hp;
    let state = enter_extra_play_phase(state, setup.owner);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0
    );
    let (state, next_phase) = activate(state, setup.owner, target, setup.skill, repeated_basic, 5);
    assert!(matches!(next_phase.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b001
    );
}

#[test]
fn exact_dying_bans_for_the_turn_even_across_an_extra_play_phase() {
    let mut setup = setup(2_026_072_305);
    let lethal = inject_hand(&mut setup.state, setup.owner, 99_011, CardKind::Sha);
    let retry = inject_hand(
        &mut setup.state,
        setup.owner,
        99_012,
        CardKind::TrickWuzhong,
    );
    let dying_target = setup.targets[0];
    let surviving_target = setup.targets[1];
    let _tao = inject_hand(&mut setup.state, surviving_target, 99_013, CardKind::Tao);
    setup.state.players.get_mut(dying_target).unwrap().hp = 1;

    let (dying, accepted) = activate(
        setup.state,
        setup.owner,
        dying_target,
        setup.skill,
        lethal,
        10,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::EnteredDying { player, source }
            if *player == dying_target && *source == setup.owner
    )));
    assert!(dying.skills.get(setup.skill).unwrap().active_turn_banned);
    assert_eq!(
        dying
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0b001
    );

    let choice = accepted.choice.expect("dying save choice");
    let (state, saved) = apply_command(
        dying,
        Command {
            actor: choice.observer,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());

    let state = enter_extra_play_phase(state, setup.owner);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        0
    );
    assert!(state.skills.get(setup.skill).unwrap().active_turn_banned);
    let before = canonical_state_hash(&state);
    let (mut state, rejected) =
        activate(state, setup.owner, surviving_target, setup.skill, retry, 12);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    state.turn.phase = Phase::End;
    let (next_turn, advanced) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
    assert_ne!(next_turn.turn.current, setup.owner);
    assert!(
        !next_turn
            .skills
            .get(setup.skill)
            .unwrap()
            .active_turn_banned
    );
}
