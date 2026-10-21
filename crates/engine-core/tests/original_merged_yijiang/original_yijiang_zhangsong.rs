//! Golden coverage for 张松「强识」「献图」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHANGSONG: u32 = 2_743_357_358;
const QIANGZHI: u32 = 2_499_367_015;
const XIANTU: u32 = 2_670_783_458;
const QIANGZHI_SOURCE: &str =
    include_str!("../../../../content/original/ported/yijiang/qiangzhi.skill");
const XIANTU_SOURCE: &str =
    include_str!("../../../../content/original/ported/yijiang/xiantu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64, skill_def: u32, current: usize) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[current];
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        other: players[2],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn open_event(state: GameState, event: DomainEvent) -> (GameState, Vec<u32>) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => {
            let paused = run_effects(&mut tx, effects).unwrap_err();
            assert_eq!(paused, EffectError::ChoiceRequested);
        }
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let options = committed
        .output
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    (committed.state, options)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
            },
        },
    )
}

fn begin_xiantu(mut setup: Setup, key: u64) -> Setup {
    let owner_card_a = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_301);
    let owner_card_b = inject(
        &mut setup.state,
        setup.owner,
        CardKind::TrickWuzhong,
        99_302,
    );
    let (state, confirm) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.target,
            phase: TurnPhaseTag::Play,
        },
    );
    assert_eq!(confirm, [0, 1]);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], key);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveExactRecordPhaseTarget
    );
    let options: Vec<_> = confirmed
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&owner_card_a.raw()));
    assert!(options.contains(&owner_card_b.raw()));
    let (state, given) = resolve(
        state,
        setup.owner,
        vec![owner_card_a.raw(), owner_card_b.raw()],
        key + 1,
    );
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(owner_card_a).unwrap().owner, setup.target);
    assert_eq!(state.cards.get(owner_card_b).unwrap().owner, setup.target);
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, setup.owner, XIANTU))
            .unwrap()
            .state_blob
            .get(..4),
        Some(b"PGT1".as_slice())
    );
    setup.state = state;
    setup
}

#[test]
fn package_publishes_complete_zhangsong_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHANGSONG)
        .unwrap();
    assert_eq!(character.skills, [QIANGZHI, XIANTU]);
    for (name, source) in [("qiangzhi", QIANGZHI_SOURCE), ("xiantu", XIANTU_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 53);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn qiangzhi_reveals_and_draws_only_for_the_remembered_play_phase_category() {
    let mut setup = setup(2_026_073_301, QIANGZHI, 0);
    let revealed = inject(
        &mut setup.state,
        setup.target,
        CardKind::TrickWuzhong,
        99_311,
    );
    let used = inject(&mut setup.state, setup.owner, CardKind::TrickWuxie, 99_312);
    let (state, _) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Play,
        },
    );
    let (state, target_choice) = resolve(state, setup.owner, vec![1], 10);
    assert_eq!(
        target_choice.choice.unwrap().kind,
        engine_api::ChoiceKind::SelectTargets
    );
    let (state, card_choice) = resolve(state, setup.owner, vec![setup.target.raw()], 11);
    assert_eq!(
        card_choice.choice.unwrap().kind,
        engine_api::ChoiceKind::SelectCards
    );
    let (state, revealed_result) = resolve(state, setup.owner, vec![revealed.raw()], 12);
    assert!(matches!(
        revealed_result.result,
        CommandResult::Accepted { .. }
    ));
    let instance = state
        .skills
        .get(skill_id(&state, setup.owner, QIANGZHI))
        .unwrap();
    assert_eq!(instance.state_blob, [b"RCG1".as_slice(), &[2]].concat());

    let before = state.hand_count(setup.owner);
    let (state, _) = open_event(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: used,
            targets: vec![setup.other],
        },
    );
    let (state, drawn) = resolve(state, setup.owner, vec![1], 13);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert!(tx
        .state()
        .skills
        .get(skill_id(tx.state(), setup.owner, QIANGZHI))
        .unwrap()
        .state_blob
        .is_empty());
}

#[test]
fn xiantu_gift_without_a_kill_loses_one_hp_at_that_play_phase_end() {
    let setup = begin_xiantu(setup(2_026_073_302, XIANTU, 1), 20);
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.target,
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
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp - 1);
    assert!(state
        .skills
        .get(skill_id(&state, setup.owner, XIANTU))
        .unwrap()
        .state_blob
        .is_empty());
}

#[test]
fn xiantu_target_kill_clears_the_delayed_penalty_and_forged_gift_is_atomic() {
    let mut setup = setup(2_026_073_303, XIANTU, 1);
    inject(&mut setup.state, setup.owner, CardKind::Sha, 99_321);
    inject(&mut setup.state, setup.owner, CardKind::Shan, 99_322);
    let forged = inject(&mut setup.state, setup.other, CardKind::Tao, 99_323);
    let (state, _) = open_event(
        setup.state,
        DomainEvent::TurnStateChanged {
            current: setup.target,
            phase: TurnPhaseTag::Play,
        },
    );
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 30);
    assert!(confirmed.choice.is_some());
    let offered = confirmed.choice.unwrap().options;
    let valid = offered[0].token;
    let before = canonical_state_hash(&state);
    let (state, rejected) = resolve(state, setup.owner, vec![valid, forged.raw()], 31);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let second = offered
        .iter()
        .map(|option| option.token)
        .find(|token| *token != valid)
        .unwrap();
    let (state, _) = resolve(state, setup.owner, vec![valid, second], 32);
    let hp = state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let cleared = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: setup.other,
            killer: setup.target,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, cleared).unwrap();
    let end = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.target,
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
    assert!(end.is_empty());
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
}
