//! Golden tests for SP「缜略」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/zhenlue.skill");
const ZHENLUE: u32 = 3_469_981_264;

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn loaded(seed: u64) -> (Transaction, [PlayerId; 3]) {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(start(seed));
    let actor = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let pos = order.iter().position(|player| *player == actor).unwrap();
    let players = [
        actor,
        order[(pos + 1) % order.len()],
        order[(pos + 2) % order.len()],
    ];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in players {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.phase = Phase::Play;
    (tx, players)
}

fn add_zhenlue(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(ZHENLUE),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_900 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let id = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(id);
    id
}

fn play(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn committed_package_matches_rewritten_source() {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    let compiled = content_compiler::compile_source_named("zhenlue", SOURCE).unwrap();
    assert!(compiled.skill_index[0].locked);
    assert_eq!(compiled.modifier_index[0].target, "card_use_unrespondable");
    assert_eq!(
        compiled.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(compiled.modifier_index[1].target, "card_target_cancel");
    assert_eq!(
        compiled.modifier_index[1].selector.as_deref(),
        Some("delayed_trick")
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHENLUE && skill.locked));
}

#[test]
fn ordinary_trick_skips_the_whole_wuxie_window() {
    let (mut tx, [source, responder, _]) = loaded(20_260_720);
    add_zhenlue(&mut tx, source);
    let wuzhong = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart, 7);
    let wuxie = inject(&mut tx, responder, CardKind::TrickWuxie, Suit::Club, 12);

    let (state, pipeline) = play(&tx, source, wuzhong, vec![]);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), 2);
    assert!(state.hand_cards(responder).contains(&wuxie));
}

#[test]
fn a_zhenlue_wuxie_cannot_be_countered() {
    let (mut tx, [source, first, second]) = loaded(20_260_721);
    add_zhenlue(&mut tx, first);
    let wuzhong = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart, 7);
    let first_wuxie = inject(&mut tx, first, CardKind::TrickWuxie, Suit::Club, 12);
    let second_wuxie = inject(&mut tx, second, CardKind::TrickWuxie, Suit::Spade, 13);

    let (state, pipeline) = play(&tx, source, wuzhong, vec![]);
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, first);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: first,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![first_wuxie.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), 0);
    assert!(state.hand_cards(second).contains(&second_wuxie));
}

#[test]
fn delayed_tricks_cannot_target_owner_but_remain_respondable_when_owner_uses_them() {
    let (mut blocked, [source, owner, _]) = loaded(20_260_722);
    add_zhenlue(&mut blocked, owner);
    let le = inject(&mut blocked, source, CardKind::TrickLe, Suit::Heart, 6);
    let (_, blocked_pipeline) = play(&blocked, source, le, vec![owner]);
    assert!(matches!(
        blocked_pipeline.result,
        CommandResult::Rejected { .. }
    ));

    let (mut allowed, [owner, target, _]) = loaded(20_260_723);
    add_zhenlue(&mut allowed, owner);
    let le = inject(&mut allowed, owner, CardKind::TrickLe, Suit::Heart, 6);
    inject(&mut allowed, target, CardKind::TrickWuxie, Suit::Club, 12);
    let (state, pipeline) = play(&allowed, owner, le, vec![target]);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::WuxieWindow
    );
    assert_eq!(pipeline.choice.unwrap().observer, target);
}
