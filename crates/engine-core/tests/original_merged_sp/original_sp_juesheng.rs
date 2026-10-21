//! Golden coverage for SP 范疆张达「决生」(juesheng).
//! Play-phase view-as: one hand card as 决斗, once per turn.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUESHENG: u32 = 3_463_553_348;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/juesheng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
    card: CardId,
}

fn setup() -> Setup {
    let compiled = content_compiler::compile_source_named("juesheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_03, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUESHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_401),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JUESHENG)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
        card,
    }
}

fn activate(state: GameState, setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.other],
                card_targets: vec![setup.card],
            },
        },
    )
}

fn drain_cancels(mut state: GameState, mut pipe: PipelineResult, mut key: u64) -> GameState {
    while state.has_pending_choice() {
        let choice = pipe.choice.clone().expect("pending choice");
        key += 1;
        let (next, next_pipe) = apply_command(
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
        assert!(
            matches!(next_pipe.result, CommandResult::Accepted { .. }),
            "{:?}",
            next_pipe.result
        );
        state = next;
        pipe = next_pipe;
        if key > 32 {
            break;
        }
    }
    let _ = pipe;
    state
}

#[test]
fn compiled_source_and_npack_publish_the_juedou_view_as() {
    let compiled = content_compiler::compile_source_named("juesheng", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUESHENG && skill.name == "juesheng" && !skill.locked));
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JUESHENG)
        .expect("juesheng view_as");
    assert_eq!(view.as_kind, "juedou");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "any");
    assert!(
        view.source_zone.as_deref().is_none() || view.source_zone.as_deref() == Some("hand"),
        "default/hand source, got {:?}",
        view.source_zone
    );
    assert_eq!(view.uses_per_turn, Some(1));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack_view = committed
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JUESHENG)
        .expect("npack juesheng view_as");
    assert_eq!(pack_view.as_kind, "juedou");
    assert_eq!(pack_view.uses_per_turn, Some(1));
}

#[test]
fn hand_card_converts_to_juedou_and_damages_when_unanswered() {
    let setup = setup();
    let other_hp = setup.state.players.get(setup.other).unwrap().hp;
    let (state, pipe) = activate(setup.state.clone(), &setup, 1);
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let state = if state.has_pending_choice() {
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::WuxieWindow
        );
        drain_cancels(state, pipe, 1)
    } else {
        state
    };
    assert_eq!(state.players.get(setup.other).unwrap().hp, other_hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.card).unwrap().zone, discard);
}
