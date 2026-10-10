//! Golden coverage for 标孙皓「残蚀」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const STDCANSHI: u32 = 2_290_683_272;
const STDCANSHI_EFFECT: u32 = 2_335_115_700;
const STD_SUNHAO: u32 = 4_237_199_211;
const GUIMING: u32 = 3_828_399_214;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sixiang/stdcanshi.skill"),
    "\nskill guiming(3828399214) locked lord_only {}\n"
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("stdcanshi_{id}"),
        max_hp: 5,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut package = content_compiler::compile_source_named("stdcanshi-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_222_001, "wu"),
        character(9_222_002, "wu"),
        character(9_222_003, "wu"),
        character(9_222_004, "qun"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_724),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_724,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let mut tx = Transaction::begin(engine.state().clone());
    let seats = tx.state().players.alive_by_seat();
    let owner = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let others: Vec<_> = seats
        .iter()
        .copied()
        .filter(|player| *player != owner)
        .collect();
    for (player, character) in [owner]
        .into_iter()
        .chain(others.iter().copied())
        .zip(9_222_001..=9_222_004)
    {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }

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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDCANSHI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUIMING),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_abi145_and_complete_std_sunhao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| { dependency.namespace == "official" && dependency.package_id == "sp" }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == STDCANSHI_EFFECT
            && modifier.target == "draw_count"
            && modifier.value_source.as_deref() == Some("alive_wounded_or_guiming_wu_count_min_one")
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDCANSHI_EFFECT
            && trigger.event == "card_targeted"
            && trigger.filter
                == "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNHAO)
        .unwrap();
    assert_eq!(character.name_key, "std_sunhao");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, [STDCANSHI, 2_265_775_915, GUIMING]);
}

#[test]
fn draw_phase_counts_wounded_and_guiming_wu_players_then_expires() {
    let (mut tx, owner, others) = setup();
    assert!(has_skill(tx.state(), owner, STDCANSHI_EFFECT));
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    tx.state_mut().players.get_mut(others[2]).unwrap().hp -= 1;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 4);

    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Rebel;
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Lord;

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
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
    assert!(!has_skill(tx.state(), owner, STDCANSHI_EFFECT));
}

#[test]
fn targeting_a_wounded_player_with_sha_forces_one_owner_discard() {
    let (mut tx, owner, others) = setup();
    let target = others[0];
    let used = inject_hand(&mut tx, owner, CardKind::Sha, 145_001);
    let discarded = inject_hand(&mut tx, owner, CardKind::Other, 145_002);
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: used,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let offered = tx.commit();
    let choice = offered.output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == discarded.raw()));
    let choice_id = ChoiceId::new(offered.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        offered.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(discarded).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
