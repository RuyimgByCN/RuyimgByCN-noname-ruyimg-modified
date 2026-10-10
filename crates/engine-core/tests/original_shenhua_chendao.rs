//! Golden coverage for SHENHUA 陈到「往烈」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{
    card_distance_unlimited_for, card_targets_self_only_for, card_use_unrespondable_for_card,
};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCWANGLIE: u32 = 3_434_983_038;
const DCWANGLIE_EFFECT: u32 = 4_183_988_931;
const DCWANGLIE2: u32 = 2_388_106_243;
const CHENDAO: u32 = 3_186_469_990;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const MAIN: &str = include_str!("../../../content/original/ported/shenhua/dcwanglie.skill");
const COUNTER: &str =
    include_str!("../../../content/original/ported/shenhua/dcwanglie_effect.skill");
const LOCK: &str = include_str!("../../../content/original/ported/shenhua/dcwanglie2.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named(
        "dcwanglie-test",
        &format!("{MAIN}\n{COUNTER}\n{LOCK}"),
    )
    .unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(DCWANGLIE), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCWANGLIE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(has_skill(&tx, owner, DCWANGLIE_EFFECT));
    Setup { tx, owner, other }
}

fn inject(tx: &mut Transaction, player: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn has_skill(tx: &Transaction, owner: PlayerId, skill_def: u32) -> bool {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn dispatch_card(
    tx: &mut Transaction,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

#[test]
fn package_publishes_chendao_in_current_shenhua_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let chendao = package
        .character_index
        .iter()
        .find(|character| character.id == CHENDAO)
        .unwrap();
    assert_eq!(chendao.skills, [DCWANGLIE]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DCWANGLIE_EFFECT
            && modifier.target == "card_distance_unlimited"
            && modifier.value_source.as_deref() == Some("owner_skill_play_phase_fire_count_below_2")
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DCWANGLIE2
            && modifier.target == "card_use_unrespondable_for_skill_card_policy"
    }));
}

#[test]
fn first_two_cards_targeting_others_ignore_distance() {
    let mut setup = setup();
    let self_card = inject(&mut setup.tx, setup.owner, CardKind::Shan, 96_001);
    assert!(card_distance_unlimited_for(setup.tx.state(), setup.owner));
    let effects = dispatch_card(&mut setup.tx, setup.owner, self_card, vec![setup.owner]).unwrap();
    run_effects(&mut setup.tx, effects).unwrap();
    assert!(card_distance_unlimited_for(setup.tx.state(), setup.owner));

    for index in 0..2 {
        let card = inject(&mut setup.tx, setup.owner, CardKind::Shan, 96_010 + index);
        let effects = dispatch_card(&mut setup.tx, setup.owner, card, vec![setup.other]).unwrap();
        run_effects(&mut setup.tx, effects).unwrap();
        assert_eq!(
            card_distance_unlimited_for(setup.tx.state(), setup.owner),
            index == 0
        );
    }
}

#[test]
fn accepted_use_marks_only_current_card_unrespondable_and_cleans_both_helpers() {
    let mut setup = setup();
    let current = inject(&mut setup.tx, setup.owner, CardKind::TrickWuzhong, 96_101);
    let later = inject(&mut setup.tx, setup.owner, CardKind::TrickWuzhong, 96_102);
    assert_eq!(
        dispatch_card(&mut setup.tx, setup.owner, current, vec![setup.other]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = setup.tx.state().clone();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    assert!(has_skill(&tx, setup.owner, DCWANGLIE2));
    assert!(card_targets_self_only_for(tx.state(), setup.owner));
    assert!(card_use_unrespondable_for_card(
        tx.state(),
        setup.owner,
        current,
        CardKind::TrickWuzhong,
        Suit::Spade,
    ));
    assert!(!card_use_unrespondable_for_card(
        tx.state(),
        setup.owner,
        later,
        CardKind::TrickWuzhong,
        Suit::Spade,
    ));

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(!has_skill(&tx, setup.owner, DCWANGLIE_EFFECT));
    assert!(!has_skill(&tx, setup.owner, DCWANGLIE2));
    assert!(!card_targets_self_only_for(tx.state(), setup.owner));
}
