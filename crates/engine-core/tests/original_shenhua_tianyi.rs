//! Golden coverage for 太史慈「天义」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, RejectReason, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_target_within_distance, sha_target_limit_for, sha_use_limit_for};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const TAISHICI: u32 = 2_164_149_853;
const TIANYI: u32 = 3_117_995_535;
const TIANYI_WIN: u32 = 4_028_521_539;
const TIANYI_LOSE: u32 = 2_573_090_805;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/shenhua/tianyi.skill"),
    include_str!("../../../content/original/ported/shenhua/tianyi3.skill"),
    include_str!("../../../content/original/ported/shenhua/tianyi2.skill")
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
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
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target, far) = (seats[0], seats[1], seats[2]);
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
            skill_def: SkillDefId::new(TIANYI),
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
        99_301,
        CardKind::Other,
        Suit::Spade,
        owner_rank,
    );
    let target_compare = target_rank
        .map(|rank| {
            inject(
                tx.state_mut(),
                target,
                99_302,
                CardKind::Other,
                Suit::Heart,
                rank,
            )
        })
        .unwrap_or(CardId::NONE);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        far,
        owner_compare,
        target_compare,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
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
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn compare(setup: &Setup, key: u64) -> GameState {
    let active = skill_id(&setup.state, setup.owner, TIANYI).unwrap();
    let (state, choose_owner_card) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
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
    let (state, choose_target_card) =
        resolve(state, setup.owner, vec![setup.owner_compare.raw()], key + 1);
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
        setup.target,
        vec![setup.target_compare.raw()],
        key + 2,
    );
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
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
    tx.commit().state
}

#[test]
fn package_publishes_complete_taishici_and_exact_tianyi_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TAISHICI)
        .unwrap();
    assert_eq!(character.name_key, "taishici");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.skills, [TIANYI]);

    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == TIANYI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(active.selection.as_ref().unwrap().zone, "hand");
    assert!(active.targets.as_ref().unwrap().can_compare);
    assert!(!active.targets.as_ref().unwrap().allow_self);
    for helper in [TIANYI_WIN, TIANYI_LOSE] {
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == helper
                && trigger.event == "turn_ended"
                && trigger.filter == "owner_is_target"
        }));
    }

    let compiled = content_compiler::compile_source_named("tianyi", SOURCE).unwrap();
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
fn strict_win_discards_both_cards_grants_all_sha_bonuses_once_and_expires() {
    let setup = setup(2_026_072_201, 13, Some(1));
    let mut state = compare(&setup, 1);
    assert!(skill_id(&state, setup.owner, TIANYI_WIN).is_some());
    assert!(skill_id(&state, setup.owner, TIANYI_LOSE).is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.owner_compare).unwrap().zone, discard);
    assert_eq!(state.cards.get(setup.target_compare).unwrap().zone, discard);
    assert_eq!(sha_use_limit_for(&state, setup.owner), 2);
    assert_eq!(sha_target_limit_for(&state, setup.owner), 2);

    let sha = inject(
        &mut state,
        setup.owner,
        99_303,
        CardKind::Sha,
        Suit::Club,
        7,
    );
    assert!(card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.far,
        1
    ));

    let next_owner_card = inject(
        &mut state,
        setup.owner,
        99_304,
        CardKind::Other,
        Suit::Diamond,
        8,
    );
    inject(
        &mut state,
        setup.target,
        99_305,
        CardKind::Other,
        Suit::Club,
        9,
    );
    let active = skill_id(&state, setup.owner, TIANYI).unwrap();
    let (_, repeated) = command(
        state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
            card_targets: vec![next_owner_card],
        },
        10,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active skill use limit reached"
    ));

    let state = end_owner_turn(state, setup.owner);
    assert!(skill_id(&state, setup.owner, TIANYI_WIN).is_none());
    assert_eq!(sha_use_limit_for(&state, setup.owner), 1);
    assert_eq!(sha_target_limit_for(&state, setup.owner), 1);
    assert!(!card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.far,
        1
    ));
}

#[test]
fn tie_and_loss_both_forbid_sha_until_turn_end() {
    for (offset, owner_rank, target_rank) in [(0, 7, 7), (1, 1, 13)] {
        let setup = setup(2_026_072_210 + offset, owner_rank, Some(target_rank));
        let mut state = compare(&setup, 20 + offset * 10);
        assert!(skill_id(&state, setup.owner, TIANYI_WIN).is_none());
        assert!(skill_id(&state, setup.owner, TIANYI_LOSE).is_some());
        let sha = inject(
            &mut state,
            setup.owner,
            99_310 + offset as u32,
            CardKind::Sha,
            Suit::Spade,
            5,
        );
        assert_eq!(sha_use_limit_for(&state, setup.owner), 0);
        let (_, blocked) = command(
            state.clone(),
            setup.owner,
            CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
            100 + offset,
        );
        assert!(matches!(
            blocked.result,
            CommandResult::Rejected {
                reason: RejectReason::IllegalAction { .. }
            }
        ));

        let state = end_owner_turn(state, setup.owner);
        assert!(skill_id(&state, setup.owner, TIANYI_LOSE).is_none());
        assert_eq!(sha_use_limit_for(&state, setup.owner), 1);
    }
}

#[test]
fn comparison_rejects_a_target_without_hand_cards() {
    let setup = setup(2_026_072_220, 10, None);
    let active = skill_id(&setup.state, setup.owner, TIANYI).unwrap();
    let (_, result) = command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
            card_targets: vec![setup.owner_compare],
        },
        52,
    );
    assert!(matches!(
        result.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active target has no hand card to compare"
    ));
}
