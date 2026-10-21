//! Golden for ONLYOL 蔡夫人「OL窃听」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OL_CAIFUREN: u32 = 2_284_410_050;
const OLQIETING: u32 = 4_244_184_524;
const XIANZHOU: u32 = 4_217_889_451;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    ending: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ending = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLQIETING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = ending;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        ending,
    }
}

fn inject_weapon(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone: equip,
        owner,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    card
}

fn trigger(
    state: GameState,
    ending: PlayerId,
    card_targeted_other: bool,
    damage_sources: u32,
    current_dealt_damage_other: bool,
) -> (GameState, Option<ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending,
            phase: TurnPhaseTag::End,
            damage_sources,
            damaged_players: 0,
            card_targeted_other,
            current_dealt_damage_other,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_ol_caifuren_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 688);
    assert_eq!(package.skill_index.len(), 190);
    assert_eq!(package.character_index.len(), 84);
    assert_eq!(package.trigger_index.len(), 27); // 批162 olrenxin +1
    assert_eq!(package.modules.len(), 30); // 批162 olrenxin +1
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_CAIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "ol_caifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [OLQIETING, XIANZHOU]);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLQIETING)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert_eq!(
        triggers[0].filter,
        "owner_alive_other_turn_ended_no_card_targeted_other"
    );
    assert_eq!(
        triggers[1].filter,
        "owner_alive_other_turn_ended_card_targeted_other_no_damage_other"
    );
}

#[test]
fn no_other_target_allows_move_then_draw_while_damage_history_keeps_single_choice() {
    let mut first = setup(2_026_072_401);
    let weapon = inject_weapon(&mut first.state, first.ending, 99_401);
    let hand = first.state.hand_count(first.owner);
    let (state, choice) = trigger(first.state, first.ending, false, 0, false);
    let choice = choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 2, true)
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, branch) = resolve(state, first.owner, vec![0, 1], false, 1);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(
        branch.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, finished) = resolve(state, first.owner, vec![weapon.raw()], false, 2);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(first.owner), hand + 1);
    let owner_equip = state.zones.find(ZoneKind::Equip, first.owner).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_equip);

    let mut second = setup(2_026_072_402);
    inject_weapon(&mut second.state, second.ending, 99_402);
    let ending_bit = 1_u32 << (second.ending.raw() - 1);
    let (_, choice) = trigger(second.state, second.ending, true, ending_bit, false);
    let choice = choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let blocked_setup = setup(2_026_072_405);
    let (_, blocked) = trigger(blocked_setup.state, blocked_setup.ending, true, 0, true);
    assert!(blocked.is_none());
}

#[test]
fn both_turn_history_branches_remain_cancellable() {
    for (seed, card_targeted_other) in [(2_026_072_403, false), (2_026_072_404, true)] {
        let setup = setup(seed);
        let hand = setup.state.hand_count(setup.owner);
        let (state, choice) = trigger(setup.state, setup.ending, card_targeted_other, 0, false);
        assert!(choice.unwrap().allow_cancel);
        let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, seed);
        assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
        assert_eq!(state.hand_count(setup.owner), hand);
        assert!(!state.has_pending_choice());
    }
}
