//! Golden coverage for SP 孙皓「残蚀」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RECANSHI: u32 = 2_466_532_461;
const RECANSHI2: u32 = 2_932_254_333;
const RECHOUHAI: u32 = 4_121_233_014;
const GUIMING: u32 = 3_828_399_214;
const SUNHAO: u32 = 2_778_414_395;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/sp/recanshi.skill"),
    "\nskill rechouhai(4121233014) locked {}\nskill guiming(3828399214) locked lord_only {}\n"
);
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    qun: PlayerId,
    wu: PlayerId,
}

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        voice_assets: Default::default(),
        id,
        name_key: format!("recanshi_{id}"),
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

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(seed: u64) -> Setup {
    let mut package = content_compiler::compile_source_named("recanshi-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_266_001, "wu"),
        character(9_266_002, "qun"),
        character(9_266_003, "wu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let others: Vec<_> = players
        .iter()
        .copied()
        .filter(|player| *player != owner)
        .collect();
    for (player, character) in [
        (owner, 9_266_001),
        (others[0], 9_266_002),
        (others[1], 9_266_003),
    ] {
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECANSHI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUIMING),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    Setup {
        state: tx.commit().state,
        owner,
        qun: others[0],
        wu: others[1],
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn inject_event_sha(state: &mut GameState, def: u32) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_current_sp_abi_and_complete_sunhao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == RECANSHI
            && trigger.event == "phase_draw"
            && trigger.filter == "owner_is_target_alive_wounded_or_guiming_wu_exists"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == RECANSHI2
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SUNHAO)
        .unwrap();
    assert_eq!(character.name_key, "sunhao");
    assert_eq!(character.max_hp, 5);
    assert!(character.is_lord);
    assert_eq!(character.skills, [RECANSHI, RECHOUHAI, GUIMING]);
}

#[test]
fn wounded_and_guiming_wu_union_adds_draws_then_sha_forces_discard_and_expires() {
    let mut setup = setup(2_026_072_503);
    setup.state.players.get_mut(setup.owner).unwrap().hp -= 1;
    setup.state.players.get_mut(setup.qun).unwrap().hp -= 1;
    assert_eq!(
        engine_core::query::alive_wounded_or_guiming_wu_count(&setup.state, setup.owner),
        3
    );
    let before = setup.state.hand_count(setup.owner);
    let (offered_state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (mut state, accepted) = resolve(offered_state, setup.owner, vec![1], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 5);
    assert!(skill_id(&state, setup.owner, RECANSHI2).is_some());

    state.turn.phase = Phase::Play;
    let sha = inject_event_sha(&mut state, 190_001);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: setup.owner,
            card: sha,
            targets: vec![setup.wu],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let pending = tx.commit();
    let choice = pending.output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    let cost = choice.options[0].token;
    let hand_before = pending.state.hand_count(setup.owner);
    let (state, result) = resolve(pending.state, setup.owner, vec![cost], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before - 1);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
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
    assert!(skill_id(tx.state(), setup.owner, RECANSHI2).is_none());
}

#[test]
fn no_eligible_player_means_no_optional_offer() {
    let mut setup = setup(2_026_072_504);
    setup.state.players.get_mut(setup.owner).unwrap().identity = Identity::Rebel;
    assert_eq!(
        engine_core::query::alive_wounded_or_guiming_wu_count(&setup.state, setup.owner),
        0
    );
    let (state, result) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(skill_id(&state, setup.owner, RECANSHI2).is_none());
}
