//! Golden for DIY 吕雉「弄权」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NSNONGQUAN: u32 = 2_685_398_351;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 341);
    assert!(package.manifest.rule_vm_abi_version <= engine_api::RULE_VM_ABI_VERSION);
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == NSNONGQUAN
            && view.as_kind == "wuzhong"
            && view.window == "play_phase"
            && view.filter == "owner_single_hand"
            && view.source_zone.is_none()
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    run_effects(
        &mut tx,
        skills
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill })
            .collect(),
    )
    .unwrap();
    for player in tx.state().players.alive_by_seat() {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSNONGQUAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NSNONGQUAN)
        .unwrap();
    (tx.commit().state, owner, skill)
}

fn inject_hand(state: &mut GameState, owner: PlayerId, suit: Suit, rank: u8) -> engine_api::CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_900 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(owner: PlayerId, skill: SkillId, card: engine_api::CardId) -> Command {
    Command {
        actor: owner,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::ActivateSkill {
            skill,
            targets: vec![owner],
            card_targets: vec![card],
        },
    }
}

fn pass_wuxie(mut state: GameState, mut pipe: PipelineResult) -> GameState {
    let mut key = 1;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipe.choice.unwrap();
        key += 1;
        (state, pipe) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    }
    state
}

#[test]
fn last_hand_card_becomes_wuzhong_but_two_hand_cards_are_rejected() {
    let (mut state, owner, skill) = setup(20_260_731);
    let black = inject_hand(&mut state, owner, Suit::Club, 9);
    let (state, pipe) = apply_command(state, activate(owner, skill, black));
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let state = pass_wuxie(state, pipe);
    assert_eq!(state.hand_count(owner), 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(black).unwrap().zone, discard);

    let (mut state, owner, skill) = setup(20_260_732);
    let first = inject_hand(&mut state, owner, Suit::Heart, 3);
    inject_hand(&mut state, owner, Suit::Spade, 4);
    let (_, pipe) = apply_command(state, activate(owner, skill, first));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
}
