//! Golden for YIJIANG `jiangchi3`: a held Sha cannot answer Nanman.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const JIANGCHI3: u32 = 3_940_214_578;
const XIYING2: u32 = 3_555_033_084;

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_800 + kind.tag() as u32 + rank as u32),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn pass_wuxie(mut state: GameState, mut pipe: PipelineResult) -> (GameState, PipelineResult) {
    let mut key = 10;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipe.choice.clone().expect("wuxie choice");
        let result = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        state = result.0;
        pipe = result.1;
        key += 1;
    }
    (state, pipe)
}

fn play_nanman(
    package: &[u8],
    blocked_skill: Option<u32>,
) -> (GameState, PipelineResult, PlayerId, u32, i32) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_770,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    // This Golden isolates the injected response modifier from whichever
    // character skills the growing package roster assigned at StartGame.
    let initial_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    run_effects(
        &mut tx,
        initial_skills
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill })
            .collect(),
    )
    .unwrap();
    if let Some(skill) = blocked_skill {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(skill),
            }],
        )
        .unwrap();
    }
    let sha = inject(&mut tx, target, CardKind::Sha, 6);
    let nanman = inject(&mut tx, actor, CardKind::TrickNanman, 7);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: nanman,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let (state, pipe) = pass_wuxie(state, pipe);
    (state, pipe, target, sha.raw(), hp)
}

#[test]
fn jiangchi3_removes_held_sha_from_nanman_response_options() {
    let (ordinary, ordinary_pipe, target, sha, hp) = play_nanman(YIJIANG, None);
    assert_eq!(
        ordinary.resolution.continuation_kind,
        ContinuationKind::RespondAoeQueue
    );
    assert_eq!(ordinary.players.get(target).unwrap().hp, hp);
    assert!(ordinary_pipe
        .choice
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == sha));

    let (blocked, blocked_pipe, target, _, hp) = play_nanman(YIJIANG, Some(JIANGCHI3));
    assert!(blocked_pipe.choice.is_none());
    assert_eq!(blocked.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn xiying2_removes_all_card_response_options() {
    let (blocked, blocked_pipe, target, _, hp) = play_nanman(SP2, Some(XIYING2));
    assert!(blocked_pipe.choice.is_none());
    assert_eq!(blocked.players.get(target).unwrap().hp, hp - 1);
}
