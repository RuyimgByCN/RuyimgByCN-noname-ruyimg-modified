//! Golden coverage for 神话董卓「肉林」「崩坏」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DONGZHUO: u32 = 3_126_184_051;
const JIUCHI: u32 = 2_316_672_021;
const ROULIN: u32 = 3_168_437_611;
const BENGHUAI: u32 = 3_365_741_831;
const BAONUE: u32 = 3_659_854_073;
const MULTI_SHA: u32 = 4_290_000_089;
const OWNER_CHARACTER: u32 = 9_289_010;
const FEMALE_CHARACTER: u32 = 9_289_011;
const MALE_CHARACTER: u32 = 9_289_012;
const DOUBLE_CHARACTER: u32 = 9_289_013;
const ROULIN_SOURCE: &str =
    include_str!("../../../../content/original/ported/shenhua/roulin.skill");
const BENGHUAI_SOURCE: &str =
    include_str!("../../../../content/original/ported/shenhua/benghuai.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

fn character(id: u32, sex: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        voice_assets: Default::default(),
        id,
        name_key: format!("dongzhuo_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: sex.into(),
        group: "qun".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    female: PlayerId,
    male: PlayerId,
    double: PlayerId,
}

fn setup(skill: u32, multi_sha: bool) -> Setup {
    let source = format!(
        "{ROULIN_SOURCE}\n{BENGHUAI_SOURCE}\n\
         skill multi_sha({MULTI_SHA}) {{ modifier card_distance_unlimited(sha) override 1; modifier sha_target_limit add 1; }}"
    );
    let mut package = content_compiler::compile_source_named("dongzhuo-test", &source).unwrap();
    package.character_index = vec![
        character(OWNER_CHARACTER, "male"),
        character(FEMALE_CHARACTER, "female"),
        character(MALE_CHARACTER, "male"),
        character(DOUBLE_CHARACTER, "double"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_722_89,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let [female, male, double] = others.as_slice() else {
        panic!("expected three players other than the owner");
    };
    let (female, male, double) = (*female, *male, *double);
    for (player, character_def) in [
        (owner, OWNER_CHARACTER),
        (female, FEMALE_CHARACTER),
        (male, MALE_CHARACTER),
        (double, DOUBLE_CHARACTER),
    ] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character_def;
    }

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    if multi_sha {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MULTI_SHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    Setup {
        tx,
        owner,
        female,
        male,
        double,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_890 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_card(
    state: GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    card: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

fn phase_end(tx: &mut Transaction, player: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve_benghuai(tx: &Transaction, option: u32, key: u64) -> GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: tx.state().turn.current,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![option],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_dongzhuo_and_exact_skill_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DONGZHUO)
        .unwrap();
    assert_eq!(character.name_key, "dongzhuo");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 8);
    assert!(character.is_lord);
    assert_eq!(character.skills, [JIUCHI, ROULIN, BENGHUAI, BAONUE]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == ROULIN
            && modifier.target == "sha_response_count_if_other_female"
            && modifier.selector.is_none()
            && modifier.mode == "add"
            && modifier.value == 1
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == BENGHUAI)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target_not_min_hp");
    assert!(!trigger.optional);
}

#[test]
fn roulin_applies_in_both_directions_only_opposite_female_or_double_sex() {
    let setup = setup(ROULIN, false);
    for (source, target, expected) in [
        (setup.owner, setup.female, 2),
        (setup.owner, setup.double, 2),
        (setup.owner, setup.male, 1),
        (setup.female, setup.owner, 2),
        (setup.double, setup.owner, 2),
        (setup.male, setup.owner, 1),
    ] {
        assert_eq!(
            engine_core::query::sha_response_count_for(setup.tx.state(), source, target, 1),
            expected
        );
    }
}

#[test]
fn multi_target_sha_recomputes_roulin_for_each_current_target() {
    let mut setup = setup(ROULIN, true);
    for player in [setup.owner, setup.female, setup.male, setup.double] {
        run_effects(
            &mut setup.tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    let sha = inject(&mut setup.tx, setup.owner, CardKind::Sha, 7);
    let female_first = inject(&mut setup.tx, setup.female, CardKind::Shan, 8);
    let female_second = inject(&mut setup.tx, setup.female, CardKind::Shan, 9);
    let male_only = inject(&mut setup.tx, setup.male, CardKind::Shan, 10);
    let female_hp = setup.tx.state().players.get(setup.female).unwrap().hp;
    let male_hp = setup.tx.state().players.get(setup.male).unwrap().hp;
    setup.tx.state_mut().turn.phase = Phase::Play;

    let (state, result) = apply_command(
        setup.tx.state().clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.female, setup.male],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let first = result.choice.unwrap();
    assert_eq!(first.observer, setup.female);
    let (state, result) = resolve_card(state, setup.female, first.choice_id, female_first, 2);
    let second = result.choice.unwrap();
    assert_eq!(second.observer, setup.female);
    let (state, result) = resolve_card(state, setup.female, second.choice_id, female_second, 3);
    let third = result.choice.unwrap();
    assert_eq!(third.observer, setup.male);
    let (state, result) = resolve_card(state, setup.male, third.choice_id, male_only, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.female).unwrap().hp, female_hp);
    assert_eq!(state.players.get(setup.male).unwrap().hp, male_hp);
}

#[test]
fn benghuai_forces_hp_or_max_hp_loss_only_when_owner_is_not_tied_minimum() {
    let mut lose_hp = setup(BENGHUAI, false);
    lose_hp
        .tx
        .state_mut()
        .players
        .get_mut(lose_hp.owner)
        .unwrap()
        .hp = 3;
    lose_hp
        .tx
        .state_mut()
        .players
        .get_mut(lose_hp.female)
        .unwrap()
        .hp = 2;
    assert_eq!(
        phase_end(&mut lose_hp.tx, lose_hp.owner),
        Err(EffectError::ChoiceRequested)
    );
    let max_hp = lose_hp
        .tx
        .state()
        .players
        .get(lose_hp.owner)
        .unwrap()
        .max_hp;
    let lost_hp = resolve_benghuai(&lose_hp.tx, 1, 5);
    assert_eq!(lost_hp.players.get(lose_hp.owner).unwrap().hp, 2);
    assert_eq!(lost_hp.players.get(lose_hp.owner).unwrap().max_hp, max_hp);

    let mut lose_max = setup(BENGHUAI, false);
    lose_max
        .tx
        .state_mut()
        .players
        .get_mut(lose_max.owner)
        .unwrap()
        .hp = 3;
    lose_max
        .tx
        .state_mut()
        .players
        .get_mut(lose_max.female)
        .unwrap()
        .hp = 2;
    assert_eq!(
        phase_end(&mut lose_max.tx, lose_max.owner),
        Err(EffectError::ChoiceRequested)
    );
    let max_hp = lose_max
        .tx
        .state()
        .players
        .get(lose_max.owner)
        .unwrap()
        .max_hp;
    let lost_max = resolve_benghuai(&lose_max.tx, 2, 6);
    assert_eq!(lost_max.players.get(lose_max.owner).unwrap().hp, 3);
    assert_eq!(
        lost_max.players.get(lose_max.owner).unwrap().max_hp,
        max_hp - 1
    );

    let mut tied = setup(BENGHUAI, false);
    tied.tx.state_mut().players.get_mut(tied.owner).unwrap().hp = 2;
    tied.tx.state_mut().players.get_mut(tied.female).unwrap().hp = 2;
    let effects = phase_end(&mut tied.tx, tied.owner).unwrap();
    assert!(effects.is_empty());
    assert!(!tied.tx.state().has_pending_choice());

    let mut other_turn = setup(BENGHUAI, false);
    other_turn
        .tx
        .state_mut()
        .players
        .get_mut(other_turn.owner)
        .unwrap()
        .hp = 3;
    other_turn
        .tx
        .state_mut()
        .players
        .get_mut(other_turn.female)
        .unwrap()
        .hp = 2;
    let effects = phase_end(&mut other_turn.tx, other_turn.female).unwrap();
    assert!(effects.is_empty());
}
