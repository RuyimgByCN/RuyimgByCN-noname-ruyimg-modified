//! Golden coverage for 吴懿「奔袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, sha_unrespondable_for_card, QueryContext};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WUYI: u32 = 3_716_905_283;
const OLBENXI: u32 = 2_954_453_328;
const OLBENXI_EFFECT: u32 = 2_946_386_821;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/olbenxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    far: PlayerId,
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

fn setup(seed: u64, players: u8) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let far = seats[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLBENXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        far,
    }
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

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("distance query failed: {other:?}"),
    }
}

fn helper_id(state: &GameState, owner: PlayerId) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == OLBENXI_EFFECT)
        })
}

fn play_card(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets,
            card_targets: Vec::new(),
        },
        key,
    )
}

#[test]
fn package_publishes_complete_wuyi_and_exact_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WUYI)
        .unwrap();
    assert_eq!(character.skills, [OLBENXI]);
    let compiled = content_compiler::compile_source_named("olbenxi", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
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
fn each_own_turn_card_use_reduces_distance_and_the_value_is_zero_off_turn() {
    let mut setup = setup(2_026_072_901, 4);
    assert_eq!(distance(&setup.state, setup.owner, setup.far), 2);
    let weapon = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_301,
        CardKind::Weapon,
        Suit::Spade,
    );
    let (mut state, played) = play_card(setup.state, setup.owner, weapon, Vec::new(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.turn.flag(turn_flag::card_used_by_player(setup.owner)),
        1
    );
    assert_eq!(distance(&state, setup.owner, setup.far), 1);
    state.turn.current = setup.target;
    assert_eq!(distance(&state, setup.owner, setup.far), 2);
}

#[test]
fn trigger_waits_until_every_other_player_is_within_one() {
    let mut setup = setup(2_026_072_902, 6);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_302,
        CardKind::Sha,
        Suit::Spade,
    );
    let (state, played) = play_card(setup.state, setup.owner, sha, vec![setup.target], 2);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != setup.owner)
        .any(|player| distance(&state, setup.owner, player) > 1));
    assert!(helper_id(&state, setup.owner).is_none());
}

#[test]
fn forced_options_reject_forgery_then_add_targets_to_sha_and_ordinary_trick() {
    let mut sha_setup = setup(2_026_072_903, 4);
    let sha = inject(
        &mut sha_setup.state,
        sha_setup.owner,
        ZoneKind::Hand,
        99_303,
        CardKind::Sha,
        Suit::Diamond,
    );
    let (state, offered) = play_card(
        sha_setup.state,
        sha_setup.owner,
        sha,
        vec![sha_setup.target],
        3,
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCurrentCardUseEnhancements
    );
    let options: Vec<_> = offered
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [0, 1, 2, 3]);

    let (state, forged) = resolve(state, sha_setup.owner, vec![1, 1], false, 4);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert!(state.has_pending_choice());
    assert!(helper_id(&state, sha_setup.owner).is_none());

    let (state, picked) = resolve(state, sha_setup.owner, vec![0, 3], false, 5);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentCardTarget
    );
    let target = picked.choice.unwrap().options[0].token;
    let added_target = PlayerId::new(target);
    let (state, added) = resolve(state, sha_setup.owner, vec![target], false, 6);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { card, target, targets, .. }
            if *card == sha && *target == added_target
                && targets == &[sha_setup.target, added_target]
    )));
    assert_eq!(state.players.get(sha_setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(added_target).unwrap().hp, 9);
    assert_eq!(state.hand_count(sha_setup.owner), 2);

    let mut trick_setup = setup(2_026_072_904, 4);
    let duel = inject(
        &mut trick_setup.state,
        trick_setup.owner,
        ZoneKind::Hand,
        99_304,
        CardKind::TrickJuedou,
        Suit::Club,
    );
    let (state, _) = play_card(
        trick_setup.state,
        trick_setup.owner,
        duel,
        vec![trick_setup.target],
        7,
    );
    let (state, picked) = resolve(state, trick_setup.owner, vec![0], false, 8);
    let target = picked.choice.unwrap().options[0].token;
    let added_target = PlayerId::new(target);
    let (state, added) = resolve(state, trick_setup.owner, vec![target], false, 9);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { card, target, targets, .. }
            if *card == duel && *target == added_target
                && targets == &[trick_setup.target, added_target]
    )));
    assert_eq!(state.players.get(trick_setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(added_target).unwrap().hp, 9);
}

#[test]
fn exact_card_can_ignore_armor_and_responses_then_helper_expires_at_turn_end() {
    let mut setup = setup(2_026_072_905, 4);
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        engine_core::content::DEF_TENGJIA,
        CardKind::Armor,
        Suit::Club,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_305,
        CardKind::Shan,
        Suit::Heart,
    );
    let fire_sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_306,
        CardKind::ShaFire,
        Suit::Diamond,
    );
    let (state, _) = play_card(setup.state, setup.owner, fire_sha, vec![setup.target], 10);
    let hp = state.players.get(setup.target).unwrap().hp;
    let (mut state, enhanced) = resolve(state, setup.owner, vec![1, 2], false, 11);
    assert!(matches!(enhanced.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert_eq!(state.cards.get(shan).unwrap().owner, setup.target);
    assert!(sha_unrespondable_for_card(&state, setup.owner, fire_sha));

    let other_fire_sha = inject(
        &mut state,
        setup.owner,
        ZoneKind::Hand,
        99_307,
        CardKind::ShaFire,
        Suit::Heart,
    );
    assert!(!sha_unrespondable_for_card(
        &state,
        setup.owner,
        other_fire_sha
    ));
    let hp = state.players.get(setup.target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: setup.owner,
            target: setup.target,
            card: other_fire_sha,
            amount: 1,
            nature: rule_api::damage_nature::FIRE,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.target).unwrap().hp, hp - 2);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: TurnPhaseTag::End,
            damage_sources: 1,
            damaged_players: 1,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(helper_id(tx.state(), setup.owner).is_none());
}

#[test]
fn unrespondable_ordinary_trick_skips_wuxie_for_that_exact_use() {
    let mut setup = setup(2_026_072_906, 4);
    let wuxie = inject(
        &mut setup.state,
        setup.far,
        ZoneKind::Hand,
        99_308,
        CardKind::TrickWuxie,
        Suit::Spade,
    );
    let duel = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_309,
        CardKind::TrickJuedou,
        Suit::Diamond,
    );
    let (state, _) = play_card(setup.state, setup.owner, duel, vec![setup.target], 12);
    let hp = state.players.get(setup.target).unwrap().hp;
    let (state, enhanced) = resolve(state, setup.owner, vec![2], false, 13);
    assert!(matches!(enhanced.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::WuxieWindow,
        "the original Wuxie window must be skipped"
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert_eq!(state.cards.get(wuxie).unwrap().owner, setup.far);
}
