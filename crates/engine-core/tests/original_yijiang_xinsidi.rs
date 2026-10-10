//! Golden coverage for 曹真「司敌」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{
    card_target_within_distance, card_use_cancelled_for_card,
    discardable_non_basic_matching_equip_color_cards, dying_save_card,
};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CAOZHEN: u32 = 2_266_553_256;
const XINSIDI: u32 = 2_768_966_746;
const XINSIDI2: u32 = 3_172_356_775;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/xinsidi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    attacker: PlayerId,
    target: PlayerId,
    red_equip: CardId,
    red_cost: CardId,
    red_basic: CardId,
    black_non_basic: CardId,
    black_save: CardId,
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

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let attacker = seats[1];
    let target = seats[2];
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
        skill_def: SkillDefId::new(XINSIDI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    let red_equip = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Equip,
        99_201,
        CardKind::Weapon,
        Suit::Heart,
    );
    let red_cost = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        99_202,
        CardKind::TrickWuxie,
        Suit::Diamond,
    );
    let red_basic = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        99_203,
        CardKind::Tao,
        Suit::Heart,
    );
    let black_non_basic = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        99_204,
        CardKind::TrickWuxie,
        Suit::Club,
    );
    let black_save = inject(
        tx.state_mut(),
        target,
        ZoneKind::Hand,
        99_205,
        CardKind::Jiu,
        Suit::Club,
    );
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    Setup {
        state: tx.commit().state,
        owner,
        attacker,
        target,
        red_equip,
        red_cost,
        red_basic,
        black_non_basic,
        black_save,
    }
}

fn helper_id(state: &GameState, target: PlayerId) -> Option<engine_api::SkillId> {
    state
        .players
        .get(target)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == XINSIDI2)
        })
}

fn open_xinsidi(state: GameState, target: PlayerId) -> (GameState, Vec<u32>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let paused = run_effects(&mut tx, effects).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
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

fn locked(seed: u64) -> Setup {
    let mut setup = setup(seed);
    let (state, _) = open_xinsidi(setup.state, setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.red_cost.raw()], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    setup.state = state;
    setup
}

fn end_play(state: GameState, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: target,
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
    tx.commit().state
}

#[test]
fn package_publishes_complete_caozhen_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOZHEN)
        .unwrap();
    assert_eq!(character.skills, [XINSIDI]);
    let compiled = content_compiler::compile_source_named("xinsidi", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 2);
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
fn phase_begin_offers_only_matching_color_non_basic_hand_and_equipment_cards() {
    let setup = setup(2_026_072_201);
    assert_eq!(
        discardable_non_basic_matching_equip_color_cards(&setup.state, setup.owner),
        [setup.red_cost, setup.red_equip]
    );
    let owner = setup.owner;
    let target = setup.target;
    let (state, options) = open_xinsidi(setup.state, target);
    assert_eq!(options, [setup.red_cost.raw(), setup.red_equip.raw()]);
    assert!(!options.contains(&setup.red_basic.raw()));
    assert!(!options.contains(&setup.black_non_basic.raw()));

    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(helper_id(&state, target).is_none());
    assert_eq!(state.cards.get(setup.red_cost).unwrap().owner, owner);
}

#[test]
fn stale_or_forged_cost_is_rejected_without_discard_or_skill_grant() {
    let setup = setup(2_026_072_202);
    let owner = setup.owner;
    let target = setup.target;
    let (state, _) = open_xinsidi(setup.state, target);
    let (state, forged) = resolve(state, owner, vec![setup.black_non_basic.raw()], false, 3);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert!(state.has_pending_choice());
    assert!(helper_id(&state, target).is_none());
    assert_eq!(state.cards.get(setup.black_non_basic).unwrap().owner, owner);
}

#[test]
fn chosen_color_blocks_physical_use_response_and_save_but_not_the_other_color() {
    let mut setup = locked(2_026_072_203);
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(setup.state.cards.get(setup.red_cost).unwrap().zone, discard);
    let helper = helper_id(&setup.state, setup.target).unwrap();
    assert_eq!(setup.state.skills.get(helper).unwrap().state_blob[8], 0b01);

    let red_tao = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_206,
        CardKind::Tao,
        Suit::Diamond,
    );
    let black_trick = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_207,
        CardKind::TrickWuxie,
        Suit::Spade,
    );
    assert!(card_use_cancelled_for_card(
        &setup.state,
        setup.target,
        red_tao
    ));
    assert!(!card_use_cancelled_for_card(
        &setup.state,
        setup.target,
        black_trick
    ));
    assert_eq!(
        dying_save_card(&setup.state, setup.target),
        Some(setup.black_save)
    );

    let red_shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_208,
        CardKind::Shan,
        Suit::Heart,
    );
    let black_shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_209,
        CardKind::Shan,
        Suit::Club,
    );
    let sha = inject(
        &mut setup.state,
        setup.attacker,
        ZoneKind::Hand,
        99_210,
        CardKind::Sha,
        Suit::Spade,
    );
    setup.state.turn.current = setup.attacker;
    setup.state.turn.phase = Phase::Play;
    let (state, played) = apply_command(
        setup.state,
        Command {
            actor: setup.attacker,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = played
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(!options.contains(&red_shan.raw()));
    assert!(options.contains(&black_shan.raw()));
    assert!(state.has_pending_choice());
}

#[test]
fn no_sha_phase_end_retaliates_from_recorded_source_ignoring_distance_then_cleans_up() {
    let mut setup = locked(2_026_072_204);
    let probe_sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_211,
        CardKind::Sha,
        Suit::Club,
    );
    assert!(!card_target_within_distance(
        &setup.state,
        setup.owner,
        probe_sha,
        setup.target,
        1
    ));
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let state = end_play(setup.state, setup.target);
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(helper_id(&state, setup.target).is_none());
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE), 0);
}

#[test]
fn repeated_grants_merge_colors_replace_source_and_sha_use_suppresses_retaliation() {
    let mut setup = locked(2_026_072_205);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddCardColorLockSkill {
            player: setup.target,
            source: setup.attacker,
            color_mask: 0b10,
            skill_def: SkillDefId::new(XINSIDI2),
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let helpers: Vec<_> = setup
        .state
        .players
        .get(setup.target)
        .unwrap()
        .skills
        .iter()
        .copied()
        .filter(|skill| setup.state.skills.get(*skill).unwrap().def.raw() == XINSIDI2)
        .collect();
    assert_eq!(helpers.len(), 1);
    let blob = &setup.state.skills.get(helpers[0]).unwrap().state_blob;
    assert_eq!(blob[8], 0b11);
    assert_eq!(
        u32::from_le_bytes(blob[4..8].try_into().unwrap()),
        setup.attacker.raw()
    );
    assert!(card_use_cancelled_for_card(
        &setup.state,
        setup.target,
        setup.black_save
    ));

    setup
        .state
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let state = end_play(setup.state, setup.target);
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp);
    assert!(helper_id(&state, setup.target).is_none());
}
