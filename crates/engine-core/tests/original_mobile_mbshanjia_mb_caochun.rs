//! Golden coverage for MOBILE 曹纯「缮甲」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, TargetCardScope};

const MBSHANJIA: u32 = 3_457_547_943;
const MBSHANJIA_NODIS: u32 = 4_049_419_240;
const MB_CAOCHUN: u32 = 2_164_633_103;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mbshanjia.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
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

fn mark_count(state: &GameState, owner: PlayerId) -> u32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == MBSHANJIA)
        .map_or(0, |(_, count)| (*count).max(0) as u32)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("mbshanjia-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(MBSHANJIA),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice {key}: {:?}",
        result.result
    );
    state
}

#[test]
fn package_publishes_mb_caochun_and_both_mbshanjia_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 326);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == MB_CAOCHUN)
            .unwrap()
            .skills,
        [MBSHANJIA]
    );
    for skill in [MBSHANJIA, MBSHANJIA_NODIS] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn equipment_loss_reduces_cost_and_noncategory_discards_grant_both_rewards() {
    let setup = setup(2_026_073_101);
    let mut tx = Transaction::begin(setup.state);
    let equipped = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        99_501,
    );
    let first = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_502,
    );
    let second = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_503,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: setup.owner,
            target: setup.owner,
            cards: vec![equipped],
            scope: TargetCardScope::Equip,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), setup.owner), 1);
    let skill = skill_id(tx.state(), setup.owner, MBSHANJIA).unwrap();
    let lost_equipment_count = mark_count(tx.state(), setup.owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbshanjia {
                player: setup.owner,
                lost_equipment_count,
                distance_skill_def: SkillDefId::new(MBSHANJIA_NODIS),
                stage: 0,
                reward_mask: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveMbshanjia
    );
    assert_eq!(state.resolution.continuation_payload[1], 2);

    let state = resolve(
        state,
        setup.owner,
        vec![first.raw(), second.raw()],
        false,
        1,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(state.resolution.continuation_payload[2], 0);
    assert_eq!(state.resolution.continuation_payload[3], 1);
    assert!(skill_id(&state, setup.owner, MBSHANJIA_NODIS).is_some());
    assert!(engine_core::query::card_distance_unlimited_for(
        &state,
        setup.owner
    ));

    let state = resolve(state, setup.owner, Vec::new(), true, 2);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
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
    assert!(skill_id(tx.state(), setup.owner, MBSHANJIA_NODIS).is_none());
}

#[test]
fn basic_and_trick_discards_suppress_their_rewards_independently() {
    for (kind, expect_distance, expect_sha, seed) in [
        (CardKind::Sha, true, false, 2_026_073_102),
        (CardKind::TrickWuzhong, false, true, 2_026_073_103),
    ] {
        let setup = setup(seed);
        let mut tx = Transaction::begin(setup.state);
        let selected = inject(&mut tx, setup.owner, ZoneKind::Hand, kind, 99_510);
        let skill = skill_id(tx.state(), setup.owner, MBSHANJIA).unwrap();
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveMbshanjia {
                    player: setup.owner,
                    lost_equipment_count: 2,
                    distance_skill_def: SkillDefId::new(MBSHANJIA_NODIS),
                    stage: 1,
                    reward_mask: 0,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let state = resolve(
            tx.commit().state,
            setup.owner,
            vec![selected.raw()],
            false,
            u64::from(kind.tag()),
        );
        assert_eq!(
            skill_id(&state, setup.owner, MBSHANJIA_NODIS).is_some(),
            expect_distance
        );
        assert_eq!(
            state.resolution.continuation_kind == ContinuationKind::VirtualShaTargets,
            expect_sha
        );
        if expect_sha {
            assert_eq!(state.resolution.continuation_payload[2], 0);
            assert_eq!(state.resolution.continuation_payload[3], 1);
        }
    }
}

#[test]
fn three_equipment_losses_require_no_discard_and_grant_both_rewards() {
    let setup = setup(2_026_073_104);
    let mut tx = Transaction::begin(setup.state);
    let skill = skill_id(tx.state(), setup.owner, MBSHANJIA).unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbshanjia {
                player: setup.owner,
                lost_equipment_count: 3,
                distance_skill_def: SkillDefId::new(MBSHANJIA_NODIS),
                stage: 0,
                reward_mask: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert!(skill_id(tx.state(), setup.owner, MBSHANJIA_NODIS).is_some());
}
