//! Golden tests for OFFLINE「血战」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::effective_card_kind;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/offline/yyxuezhan.skill");
const YYXUEZHAN: u32 = 3_105_237_184;
const YYWENCHOU: u32 = 2_778_716_453;
const YYYAZHEN: u32 = 3_406_865_651;

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
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in [source, target] {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: source,
        skill_def: SkillDefId::new(YYXUEZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(69_000 + kind.tag() as u32 * 20 + rank as u32),
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

#[test]
fn committed_package_contains_the_rewritten_skill_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let compiled = content_compiler::compile_source_named("yyxuezhan", SOURCE).unwrap();

    assert!(compiled.skill_index[0].locked);
    assert_eq!(compiled.modifier_index.len(), 2);
    assert_eq!(compiled.modifier_index[0].target, "card_name");
    assert_eq!(
        compiled.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(compiled.modifier_index[0].value, 23);
    assert_eq!(compiled.modifier_index[1].target, "card_use_unrespondable");
    assert_eq!(
        compiled.modifier_index[1].selector.as_deref(),
        Some("juedou")
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YYXUEZHAN && skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YYWENCHOU)
        .unwrap();
    assert_eq!(character.name_key, "yy_wenchou");
    assert_eq!(character.skills, vec![YYXUEZHAN, YYYAZHEN]);
}

#[test]
fn ordinary_trick_becomes_unwuxieable_duel_but_still_asks_for_sha() {
    let (mut tx, source, target) = loaded(20_260_722);
    let physical_trick = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart, 7);
    let sha = inject(&mut tx, target, CardKind::Sha, Suit::Spade, 8);
    let wuxie = inject(&mut tx, target, CardKind::TrickWuxie, Suit::Club, 12);

    assert_eq!(
        effective_card_kind(
            tx.state(),
            source,
            tx.state().cards.get(physical_trick).unwrap()
        ),
        CardKind::TrickJuedou
    );
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: physical_trick,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );

    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JuedouDuel
    );
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, sha.raw());
    assert!(state.hand_cards(target).contains(&wuxie));
    assert_eq!(
        state.cards.get(physical_trick).unwrap().kind,
        CardKind::TrickWuzhong
    );
}

#[test]
fn delayed_tricks_keep_their_physical_name() {
    let (mut tx, source, _) = loaded(20_260_723);
    let delayed = inject(&mut tx, source, CardKind::TrickLe, Suit::Diamond, 6);

    assert_eq!(
        effective_card_kind(tx.state(), source, tx.state().cards.get(delayed).unwrap()),
        CardKind::TrickLe
    );
}
