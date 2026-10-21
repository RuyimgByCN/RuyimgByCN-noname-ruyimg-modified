//! Golden coverage for 韩浩史涣「慎断」「勇略」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HANHAOSHIHUAN: u32 = 4_039_637_603;
const SHENDUAN: u32 = 2_370_759_068;
const YONGLVE: u32 = 2_407_381_445;
const ZHICHI2: u32 = 3_197_261_303;
const SHENDUAN_SOURCE: &str =
    include_str!("../../../../content/original/ported/yijiang/shenduan.skill");
const YONGLVE_SOURCE: &str =
    include_str!("../../../../content/original/ported/yijiang/yonglve.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    far: PlayerId,
}

fn setup(seed: u64, skill_def: u32) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let far = players[2];
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
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        far,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn command(
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
    command(
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

fn open_yonglve(mut state: GameState, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = engine_core::state::Phase::Judge;
    let events = [DomainEvent::TurnStateChanged {
        current: target,
        phase: TurnPhaseTag::Judge,
    }];
    let effects = dispatch_for_events(&mut tx, &events, &mut TriggerFireRecord::new()).unwrap();
    assert!(matches!(
        run_effects(&mut tx, effects),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage
    );
    state
}

#[test]
fn package_publishes_complete_hanhaoshihuan_and_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == HANHAOSHIHUAN)
        .unwrap();
    assert_eq!(character.skills, [SHENDUAN, YONGLVE]);

    for (name, source) in [("shenduan", SHENDUAN_SOURCE), ("yonglve", YONGLVE_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 51);
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
fn shenduan_repeats_over_one_discard_batch_and_ignores_bingliang_distance() {
    let mut setup = setup(2_026_073_101, SHENDUAN);
    let first = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_101,
        CardKind::Sha,
        Suit::Spade,
    );
    let second = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_102,
        CardKind::Shan,
        Suit::Club,
    );
    let red = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_103,
        CardKind::Tao,
        Suit::Heart,
    );
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: setup.owner,
                cards: vec![first, second, red],
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardedBlackBasicAsBingliang
    );
    assert_eq!(state.resolution.continuation_payload.first(), Some(&0));
    let (state, picked) = resolve(state, setup.owner, vec![first.raw()], false, 1);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = picked
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&setup.far.raw()));

    let (state, used) = resolve(state, setup.owner, vec![setup.far.raw()], false, 2);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let judge = state.zones.find(ZoneKind::Judge, setup.far).unwrap();
    assert_eq!(state.cards.get(first).unwrap().zone, judge);
    assert_eq!(
        state.cards.get(first).unwrap().kind,
        CardKind::TrickBingliang
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardedBlackBasicAsBingliang
    );
    assert_eq!(state.resolution.continuation_payload.first(), Some(&0));

    let (state, declined) = resolve(state, setup.owner, Vec::new(), true, 3);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(second).unwrap().zone, discard);
    assert_eq!(state.cards.get(red).unwrap().zone, discard);
}

#[test]
fn shenduan_rejects_a_forged_noncandidate_without_mutation() {
    let mut setup = setup(2_026_073_104, SHENDUAN);
    let black = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_401,
        CardKind::Sha,
        Suit::Spade,
    );
    let red = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        98_402,
        CardKind::Tao,
        Suit::Diamond,
    );
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: setup.owner,
                cards: vec![black, red],
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    let before = canonical_state_hash(&state);
    let (after, rejected) = resolve(state, setup.owner, vec![red.raw()], false, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&after), before);
}

#[test]
fn yonglve_hit_does_not_draw() {
    let mut setup = setup(2_026_073_102, YONGLVE);
    let judge = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Judge,
        98_201,
        CardKind::TrickLe,
        Suit::Heart,
    );
    let state = open_yonglve(setup.state, setup.target);
    let owner_hand = state.hand_count(setup.owner);
    let target_hp = state.players.get(setup.target).unwrap().hp;
    let (state, resolved) = resolve(state, setup.owner, vec![judge.raw()], false, 10);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(setup.owner), owner_hand);
}

#[test]
fn yonglve_dodged_exact_sha_draws_once() {
    let mut setup = setup(2_026_073_103, YONGLVE);
    let judge = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Judge,
        98_301,
        CardKind::TrickLe,
        Suit::Diamond,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        98_302,
        CardKind::Shan,
        Suit::Heart,
    );
    let state = open_yonglve(setup.state, setup.target);
    let owner_hand = state.hand_count(setup.owner);
    let target_hp = state.players.get(setup.target).unwrap().hp;
    let (state, discarded) = resolve(state, setup.owner, vec![judge.raw()], false, 20);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let (state, dodged) = resolve(state, setup.target, vec![shan.raw()], false, 21);
    assert!(matches!(dodged.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp);
    assert_eq!(state.hand_count(setup.owner), owner_hand + 1);
    assert!(state.resolution.sha_no_damage_draw_skills.is_empty());
}

#[test]
fn yonglve_draws_when_the_virtual_sha_is_not_legal() {
    let mut setup = setup(2_026_073_105, YONGLVE);
    let judge = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Judge,
        98_501,
        CardKind::TrickLe,
        Suit::Club,
    );
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.target,
            skill_def: SkillDefId::new(ZHICHI2),
        }],
    )
    .unwrap();
    let state = open_yonglve(tx.commit().state, setup.target);
    let owner_hand = state.hand_count(setup.owner);
    let target_hp = state.players.get(setup.target).unwrap().hp;
    let (state, resolved) = resolve(state, setup.owner, vec![judge.raw()], false, 30);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp);
    assert_eq!(state.hand_count(setup.owner), owner_hand + 1);
}
