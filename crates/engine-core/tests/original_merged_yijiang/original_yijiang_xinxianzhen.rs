//! Golden coverage for 高顺「陷阵」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    card_target_within_distance, sha_ignored_for_hand_limit, sha_use_limit_for,
    sha_use_unlimited_for_target,
};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GAOSHUN: u32 = 2_843_282_379;
const XINXIANZHEN: u32 = 3_226_686_922;
const XINXIANZHEN_WIN: u32 = 2_960_051_687;
const XINXIANZHEN_LOSE: u32 = 3_077_071_581;
const JINJIU: u32 = 2_885_443_165;
const TEST_ARMOR: u32 = 1_990_000_001;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/xinxianzhen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const ARMOR_SOURCE: &str = r#"
skill xinxianzhen_test_armor(1990000001) {
  modifier armor_damage_taken_amount(fire) add 1;
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    near: PlayerId,
    bound: PlayerId,
    far: PlayerId,
    owner_compare: CardId,
    target_compare: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, owner_rank: u8, target_rank: Option<u8>) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let armor = content_compiler::compile_source_named("xinxianzhen_armor", ARMOR_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&armor)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let near = seats[1];
    let bound = seats[2];
    let far = seats[3];
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
            skill_def: SkillDefId::new(XINXIANZHEN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    let owner_compare = inject(
        tx.state_mut(),
        owner,
        99_101,
        CardKind::Other,
        Suit::Spade,
        owner_rank,
    );
    let target_compare = target_rank
        .map(|rank| {
            inject(
                tx.state_mut(),
                bound,
                99_102,
                CardKind::Other,
                Suit::Heart,
                rank,
            )
        })
        .unwrap_or(CardId::NONE);
    Setup {
        state: tx.commit().state,
        owner,
        near,
        bound,
        far,
        owner_compare,
        target_compare,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
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

fn compare(setup: &Setup, key: u64) -> GameState {
    let active = skill_id(&setup.state, setup.owner, XINXIANZHEN);
    let (state, choose_owner_card) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(
        choose_owner_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let (state, choose_target_card) = resolve(
        state,
        setup.owner,
        vec![setup.owner_compare.raw()],
        false,
        key + 1,
    );
    assert!(matches!(
        choose_target_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let (state, compared) = resolve(
        state,
        setup.bound,
        vec![setup.target_compare.raw()],
        false,
        key + 2,
    );
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

#[test]
fn package_publishes_complete_gaoshun_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GAOSHUN)
        .unwrap();
    assert_eq!(character.skills, [XINXIANZHEN, JINJIU]);
    let compiled = content_compiler::compile_source_named("xinxianzhen", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 1);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn strict_win_binds_only_the_compared_target_for_distance_use_count_and_armor() {
    let setup = setup(2_026_072_101, 13, Some(1));
    let mut state = compare(&setup, 1);
    let win = skill_id(&state, setup.owner, XINXIANZHEN_WIN);
    assert_eq!(state.skills.get(win).unwrap().def.raw(), XINXIANZHEN_WIN);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.owner_compare).unwrap().zone, discard);
    assert_eq!(state.cards.get(setup.target_compare).unwrap().zone, discard);

    let sha = inject(
        &mut state,
        setup.owner,
        99_103,
        CardKind::Sha,
        Suit::Club,
        7,
    );
    assert!(card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.bound,
        1
    ));
    assert!(!card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.far,
        1
    ));
    assert!(sha_use_unlimited_for_target(
        &state,
        setup.owner,
        setup.bound
    ));
    assert!(!sha_use_unlimited_for_target(
        &state,
        setup.owner,
        setup.near
    ));

    state.turn.set_flag(turn_flag::SHA_PLAYED, 1);
    state.turn.set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);
    let (bound_state, bound_use) = command(
        state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(
        matches!(bound_use.result, CommandResult::Accepted { .. }),
        "bound use result: {:?}",
        bound_use.result
    );
    assert_eq!(bound_state.players.get(setup.bound).unwrap().hp, 9);
    let (_, ordinary_target_use) = command(
        state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        11,
    );
    assert!(matches!(
        ordinary_target_use.result,
        CommandResult::Rejected { .. }
    ));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: setup.bound,
                skill_def: SkillDefId::new(TEST_ARMOR),
            },
            EffectRequest::AddSkill {
                player: setup.far,
                skill_def: SkillDefId::new(TEST_ARMOR),
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.bound,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.far,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.bound).unwrap().hp, 9);
    assert_eq!(tx.state().players.get(setup.far).unwrap().hp, 8);
}

#[test]
fn success_helper_appends_bound_target_to_sha_and_independent_trick() {
    let setup = setup(2_026_072_102, 12, Some(2));
    let mut state = compare(&setup, 20);
    let sha = inject(
        &mut state,
        setup.owner,
        99_104,
        CardKind::Sha,
        Suit::Diamond,
        8,
    );
    let (state_after_play, add_sha) = command(
        state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        23,
    );
    assert!(matches!(add_sha.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state_after_play.resolution.continuation_kind,
        ContinuationKind::ChooseAddSkillBoundCardTarget,
        "play result={:?} events={:?}",
        add_sha.result,
        add_sha.events
    );
    let (sha_done, added) = resolve(
        state_after_play,
        setup.owner,
        vec![setup.bound.raw()],
        false,
        24,
    );
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == setup.bound && targets == &[setup.near, setup.bound]
    )));
    assert_eq!(sha_done.players.get(setup.near).unwrap().hp, 9);
    assert_eq!(sha_done.players.get(setup.bound).unwrap().hp, 9);

    let duel = inject(
        &mut state,
        setup.owner,
        99_105,
        CardKind::TrickJuedou,
        Suit::Spade,
        9,
    );
    let (duel_state, add_duel) = command(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: duel,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        25,
    );
    assert!(matches!(add_duel.result, CommandResult::Accepted { .. }));
    assert_eq!(
        duel_state.resolution.continuation_kind,
        ContinuationKind::ChooseAddSkillBoundCardTarget
    );
    let (duel_done, added) = resolve(duel_state, setup.owner, vec![setup.bound.raw()], false, 26);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == setup.bound && targets == &[setup.near, setup.bound]
    )));
    assert_eq!(duel_done.players.get(setup.near).unwrap().hp, 9);
    assert_eq!(duel_done.players.get(setup.bound).unwrap().hp, 9);
}

#[test]
fn tie_is_failure_sha_is_disabled_and_temporary_helpers_expire() {
    let setup = setup(2_026_072_103, 7, Some(7));
    let state = compare(&setup, 30);
    assert!(state
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).unwrap().def.raw() == XINXIANZHEN_LOSE));
    assert_eq!(sha_use_limit_for(&state, setup.owner), 0);
    assert!(sha_ignored_for_hand_limit(&state, setup.owner));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: setup.owner,
                phase: Phase::Idle.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .all(|skill| {
            !matches!(
                tx.state().skills.get(*skill).unwrap().def.raw(),
                XINXIANZHEN_WIN | XINXIANZHEN_LOSE
            )
        }));
}

#[test]
fn success_helper_appends_bound_target_to_single_target_tiesuo() {
    let setup = setup(2_026_072_105, 11, Some(3));
    let mut state = compare(&setup, 50);
    let tiesuo = inject(
        &mut state,
        setup.owner,
        99_106,
        CardKind::TrickTiesuo,
        Suit::Club,
        6,
    );
    let (state, add_target) = command(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: tiesuo,
            targets: vec![setup.near],
            card_targets: Vec::new(),
        },
        53,
    );
    assert!(matches!(add_target.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddSkillBoundCardTarget
    );
    let (state, added) = resolve(state, setup.owner, vec![setup.bound.raw()], false, 54);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardTargetAdded { target, targets, .. }
            if *target == setup.bound && targets == &[setup.near, setup.bound]
    )));
    for target in [setup.near, setup.bound] {
        assert!(state
            .players
            .get(target)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count == 1));
    }
}

#[test]
fn comparison_target_without_hand_is_rejected_before_owner_card_choice() {
    let setup = setup(2_026_072_104, 11, None);
    let active = skill_id(&setup.state, setup.owner, XINXIANZHEN);
    let (state, result) = command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        40,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert!(!state.has_pending_choice());
}
