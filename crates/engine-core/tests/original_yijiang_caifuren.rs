//! Golden coverage for 蔡夫人「窃听」「献州」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
    TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CAIFUREN: u32 = 2_163_495_034;
const QIETING: u32 = 4_166_951_057;
const XIANZHOU: u32 = 4_217_889_451;
const QIETING_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/qieting.skill");
const XIANZHOU_SOURCE: &str =
    include_str!("../../../content/original/ported/yijiang/xianzhou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    recipient: PlayerId,
    others: [PlayerId; 2],
}

struct Step {
    state: GameState,
    choice: Option<ChoiceRequest>,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let recipient = players[1];
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
    effects.extend(
        [QIETING, XIANZHOU].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = CAIFUREN;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        recipient,
        others: [players[2], players[3]],
    }
}

fn inject_equipment(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    assert!(kind.is_equip());
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) + 1,
        zone: equip,
        owner,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId, skill_def: u32) -> SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id
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
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn activate_xianzhou(
    state: GameState,
    owner: PlayerId,
    recipient: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = skill_instance(&state, owner, XIANZHOU);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![recipient],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn trigger_qieting(state: GameState, ending: PlayerId, card_targeted_other: bool) -> Step {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    Step {
        state: committed.state,
        choice: committed.output.choice,
    }
}

#[test]
fn package_publishes_complete_caifuren_and_exact_abi58_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "caifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [QIETING, XIANZHOU]);

    for (name, source) in [("qieting", QIETING_SOURCE), ("xianzhou", XIANZHOU_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 58);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == QIETING)
        .unwrap();
    assert_eq!(trigger.event, "turn_ended");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_turn_ended_no_card_targeted_other"
    );
    assert!(!trigger.optional);

    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIANZHOU)
        .unwrap();
    assert!(active.equip_nonempty);
    assert!(active.once_per_game);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
}

#[test]
fn qieting_draw_cancel_and_turn_target_history_gate_are_authoritative() {
    let setup = setup(2_026_073_801);
    let blocked = trigger_qieting(setup.state.clone(), setup.recipient, true);
    assert!(blocked.choice.is_none());
    assert!(!blocked.state.has_pending_choice());

    let step = trigger_qieting(setup.state.clone(), setup.recipient, false);
    let choice = step.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, true)
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let hand = step.state.hand_count(setup.owner);
    let (state, drawn) = resolve(step.state, setup.owner, vec![1], false, 1);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand + 1);

    let step = trigger_qieting(setup.state, setup.recipient, false);
    let hand = step.state.hand_count(setup.owner);
    let (state, cancelled) = resolve(step.state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand);
    assert!(!state.has_pending_choice());
}

#[test]
fn qieting_moves_compatible_equipment_replaces_slot_and_rejects_forgery_atomically() {
    let mut setup = setup(2_026_073_802);
    let old_weapon = inject_equipment(&mut setup.state, setup.owner, 98_201, CardKind::Weapon);
    let target_weapon =
        inject_equipment(&mut setup.state, setup.recipient, 98_202, CardKind::Weapon);
    let step = trigger_qieting(setup.state, setup.recipient, false);
    assert_eq!(
        step.choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, branch) = resolve(step.state, setup.owner, vec![0], false, 10);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(
        branch.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseMoveTargetEquipmentOrDraw
    );

    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![old_weapon.raw()], false, 11);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, moved) = resolve(state, setup.owner, vec![target_weapon.raw()], false, 12);
    assert!(matches!(moved.result, CommandResult::Accepted { .. }));
    let owner_equip = state.zones.find(ZoneKind::Equip, setup.owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(old_weapon).unwrap().zone, discard);
    let moved_card = state.cards.get(target_weapon).unwrap();
    assert_eq!(moved_card.zone, owner_equip);
    assert_eq!(moved_card.owner, setup.owner);
}

#[test]
fn xianzhou_gives_all_equipment_cancel_recovers_and_lifetime_limit_persists() {
    let mut setup = setup(2_026_073_803);
    let skill = skill_instance(&setup.state, setup.owner, XIANZHOU);
    let before = canonical_state_hash(&setup.state);
    let (state, rejected) = activate_xianzhou(setup.state, setup.owner, setup.recipient, 20);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    assert_eq!(state.skills.get(skill).unwrap().lifetime_fire_count, 0);
    setup.state = state;

    let weapon = inject_equipment(&mut setup.state, setup.owner, 98_301, CardKind::Weapon);
    let armor = inject_equipment(&mut setup.state, setup.owner, 98_302, CardKind::Armor);
    setup.state.players.get_mut(setup.owner).unwrap().hp = 3;
    let recipient_hand = setup.state.hand_count(setup.recipient);
    let (state, offered) = activate_xianzhou(setup.state, setup.owner, setup.recipient, 21);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.observer, setup.recipient);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 2, true)
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment
    );
    let recipient_hand_zone = state.zones.find(ZoneKind::Hand, setup.recipient).unwrap();
    assert_eq!(state.hand_count(setup.recipient), recipient_hand + 2);
    assert_eq!(state.cards.get(weapon).unwrap().zone, recipient_hand_zone);
    assert_eq!(state.cards.get(armor).unwrap().zone, recipient_hand_zone);
    assert_eq!(state.skills.get(skill).unwrap().lifetime_fire_count, 1);

    let (mut state, recovered) = resolve(state, setup.recipient, Vec::new(), true, 22);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 5);
    inject_equipment(&mut state, setup.owner, 98_303, CardKind::Armor);
    let before = canonical_state_hash(&state);
    let (state, repeated) = activate_xianzhou(state, setup.owner, setup.recipient, 23);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    assert_eq!(state.skills.get(skill).unwrap().lifetime_fire_count, 1);
}

#[test]
fn xianzhou_damage_targets_are_distinct_in_range_and_validated_atomically() {
    let mut setup = setup(2_026_073_804);
    inject_equipment(&mut setup.state, setup.owner, 98_401, CardKind::Weapon);
    inject_equipment(&mut setup.state, setup.owner, 98_402, CardKind::Armor);
    let (state, offered) = activate_xianzhou(setup.state, setup.owner, setup.recipient, 30);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = offered
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [setup.owner.raw(), setup.others[0].raw()]);

    let before = canonical_state_hash(&state);
    let (state, self_target) = resolve(
        state,
        setup.recipient,
        vec![setup.recipient.raw()],
        false,
        31,
    );
    assert!(matches!(self_target.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, out_of_range) = resolve(
        state,
        setup.recipient,
        vec![setup.others[1].raw()],
        false,
        32,
    );
    assert!(matches!(
        out_of_range.result,
        CommandResult::Rejected { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, duplicate) = resolve(
        state,
        setup.recipient,
        vec![setup.owner.raw(), setup.owner.raw()],
        false,
        33,
    );
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let owner_hp = state.players.get(setup.owner).unwrap().hp;
    let other_hp = state.players.get(setup.others[0]).unwrap().hp;
    let (state, damaged) = resolve(
        state,
        setup.recipient,
        vec![setup.others[0].raw(), setup.owner.raw()],
        false,
        34,
    );
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(setup.others[0]).unwrap().hp, other_hp - 1);
    assert_eq!(
        damaged
            .events
            .iter()
            .filter_map(|event| match event {
                DomainEvent::DamageDealt { source, target, .. } if *source == setup.recipient =>
                    Some(*target),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [setup.others[0], setup.owner]
    );
}
