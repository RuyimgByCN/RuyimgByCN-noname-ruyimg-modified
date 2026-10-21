//! Golden tests for OFFLINE「缜略」rewritten from inherited legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OFFLINE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/yjzhenlve.skill");
const YJZHENLVE: u32 = 2_568_267_200;

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
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
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, target] {
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
    (tx, source, target)
}

fn add_skill(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(YJZHENLVE),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(68_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(source: PlayerId, card: CardId, targets: Vec<PlayerId>, key: u64) -> Command {
    Command {
        actor: source,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets,
            card_targets: vec![],
        },
    }
}

#[test]
fn committed_skill_matches_the_inherited_zhenlue_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let compiled = content_compiler::compile_source_named("yjzhenlve", SOURCE).unwrap();
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
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YJZHENLVE && skill.locked));
}

#[test]
fn ordinary_tricks_skip_wuxie_and_delayed_tricks_cannot_target_the_owner() {
    let (mut unrespondable, source, target) = loaded(20_260_721);
    add_skill(&mut unrespondable, source);
    let wuzhong = inject(
        &mut unrespondable,
        source,
        CardKind::TrickWuzhong,
        Suit::Heart,
        7,
    );
    let wuxie = inject(
        &mut unrespondable,
        target,
        CardKind::TrickWuxie,
        Suit::Club,
        12,
    );
    let (state, pipeline) = apply_command(
        unrespondable.state().clone(),
        play(source, wuzhong, vec![], 1),
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), 2);
    assert!(state.hand_cards(target).contains(&wuxie));

    let (mut protected, source, target) = loaded(20_260_722);
    add_skill(&mut protected, target);
    let le = inject(&mut protected, source, CardKind::TrickLe, Suit::Spade, 6);
    let (state, pipeline) =
        apply_command(protected.state().clone(), play(source, le, vec![target], 2));
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert!(state.hand_cards(source).contains(&le));
}
