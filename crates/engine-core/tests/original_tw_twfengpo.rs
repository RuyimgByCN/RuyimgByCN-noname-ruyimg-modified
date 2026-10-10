//! Golden coverage for TW 马云騄「凤魄」.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFENGPO: u32 = 4_055_936_832;
const TW_MAYUNLU: u32 = 2_212_792_733;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfengpo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
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

fn setup() -> (GameState, [PlayerId; 3]) {
    let package = content_compiler::compile_source_named("twfengpo-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_207, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
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
            player: players[0],
            skill_def: SkillDefId::new(TWFENGPO),
        },
        EffectRequest::SetTurnState {
            current: players[0],
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.max_hp = 20;
        participant.hp = 15;
    }
    (tx.commit().state, players)
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn choose(
    state: GameState,
    pipeline: PipelineResult,
    actor: PlayerId,
    token: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.unwrap();
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![token],
                cancelled: false,
            },
        },
    )
}

fn activate_and_increase(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(pipeline.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, branch) = choose(state, pipeline, owner, 1, key);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VmResume,
        "after accept: {:?} {:?}",
        branch.result,
        branch.choice
    );
    assert_eq!(branch.choice.as_ref().unwrap().kind, ChoiceKind::Custom);
    choose(state, branch, owner, 2, key + 1)
}

fn cancel_responses(
    mut state: GameState,
    mut pipeline: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult) {
    while let Some(choice) = pipeline.choice.take() {
        let revision = state.revision;
        (state, pipeline) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        key += 1;
    }
    (state, pipeline)
}

#[test]
fn counts_hand_only_on_every_single_target_use_and_switches_to_red_after_kill() {
    std::thread::Builder::new().stack_size(16 << 20).spawn(|| {
        let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
        assert!(committed.manifest.rule_vm_abi_version >= 490);
        assert_eq!(
            committed
                .character_index
                .iter()
                .find(|character| character.id == TW_MAYUNLU)
                .unwrap()
                .skills,
            [2, TWFENGPO]
        );

        let (mut state, [owner, target, victim]) = setup();
        inject(
            &mut state,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            490_001,
        );
        inject(
            &mut state,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            490_002,
        );
        inject(
            &mut state,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            490_003,
        );
        inject(
            &mut state,
            target,
            ZoneKind::Equip,
            CardKind::Armor,
            Suit::Diamond,
            490_004,
        );
        let sha = inject(
            &mut state,
            owner,
            ZoneKind::Hand,
            CardKind::Sha,
            Suit::Spade,
            490_005,
        );
        let hp = state.players.get(target).unwrap().hp;
        let (state, optional) = play(state, owner, sha, target, 1);
        let (state, result) = activate_and_increase(state, optional, owner, 2);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(target).unwrap().hp, hp - 3);

        let mut tx = Transaction::begin(state);
        let effects = engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[engine_api::DomainEvent::PlayerKilled {
                player: victim,
                killer: owner,
                hand_equip_cards: Vec::new(),
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        )
        .unwrap();
        run_effects(&mut tx, effects).unwrap();
        let mut state = tx.commit().state;
        let juedou = inject(
            &mut state,
            owner,
            ZoneKind::Hand,
            CardKind::TrickJuedou,
            Suit::Club,
            490_006,
        );
        let hp = state.players.get(target).unwrap().hp;
        let (state, optional) = play(state, owner, juedou, target, 4);
        let (state, result) = activate_and_increase(state, optional, owner, 5);
        let (state, result) = cancel_responses(state, result, 7);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(target).unwrap().hp, hp - 4);

        let mut tx = Transaction::begin(state);
        let skill = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == TWFENGPO)
            .unwrap()
            .id;
        run_effects(
            &mut tx,
            vec![
                EffectRequest::RemoveSkill { skill },
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(TWFENGPO),
                },
            ],
        )
        .unwrap();
        let replacement = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == TWFENGPO)
            .unwrap();
        assert_eq!(replacement.content_flags, 0);
    });
}
