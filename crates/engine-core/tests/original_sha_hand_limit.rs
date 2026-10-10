//! Golden coverage for authoritative hand-limit modifiers and protected Sha cards.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const CHANGANDAJIAN_EQUIP5: u32 = 3_727_522_753;
const XINXIANZHEN3: u32 = 3_077_071_581;
const NEW_JIANGCHI3: u32 = 3_349_403_784;
const CANGZHUO2: u32 = 3_573_995_787;
const QIAOSHUI4: u32 = 3_521_041_512;

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> engine_api::CardId {
    inject_with_suit(tx, owner, kind, Suit::Heart, rank)
}

fn inject_with_suit(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_900 + kind.tag() as u32 + rank as u32),
        kind,
        suit,
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

#[test]
fn generic_card_selectors_compose_for_real_discard_options() {
    const KEEP_TRICKS: u32 = 9_910_001;
    const KEEP_HEARTS: u32 = 9_910_002;
    let package = content_compiler::compile_source_named(
        "selector-test",
        r#"
skill keep_tricks(9910001) {
  modifier card_ignored_hand_limit(trick) override 1;
}
skill keep_hearts(9910002) {
  modifier card_ignored_hand_limit(heart) override 1;
}
"#,
    )
    .unwrap();
    let bytes = package.to_json_bytes().unwrap();
    let (mut tx, owner) = discard_state(&bytes, 20_260_783);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;

    let trick = inject_with_suit(&mut tx, owner, CardKind::TrickGuohe, Suit::Spade, 1);
    let heart = inject_with_suit(&mut tx, owner, CardKind::Sha, Suit::Heart, 2);
    let ordinary = inject_with_suit(&mut tx, owner, CardKind::Other, Suit::Club, 3);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(KEEP_TRICKS),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(KEEP_HEARTS),
            },
        ],
    )
    .unwrap();

    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        trick
    ));
    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        heart
    ));
    assert!(!engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        ordinary
    ));
    assert_eq!(
        engine_core::query::discard_to_limit_tokens(tx.state(), owner),
        vec![ordinary]
    );

    let (_, pipe) = end_discard(tx.state().clone(), owner);
    let choice = pipe.choice.expect("one ordinary card must be discarded");
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, ordinary.raw());
}

#[test]
fn migrated_trick_selectors_protect_normal_and_delayed_tricks() {
    for (package, skill_def, seed) in [
        (REFRESH, CANGZHUO2, 20_260_784),
        (YIJIANG, QIAOSHUI4, 20_260_785),
    ] {
        let (mut tx, owner) = discard_state(package, seed);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
        let normal = inject_with_suit(&mut tx, owner, CardKind::TrickGuohe, Suit::Spade, 1);
        let delayed = inject_with_suit(&mut tx, owner, CardKind::TrickLe, Suit::Diamond, 2);
        let ordinary = inject_with_suit(&mut tx, owner, CardKind::Sha, Suit::Club, 3);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            }],
        )
        .unwrap();

        assert!(engine_core::query::card_ignored_for_hand_limit(
            tx.state(),
            owner,
            normal
        ));
        assert!(engine_core::query::card_ignored_for_hand_limit(
            tx.state(),
            owner,
            delayed
        ));
        assert_eq!(
            engine_core::query::discard_to_limit_tokens(tx.state(), owner),
            vec![ordinary]
        );
        let (_, pipe) = end_discard(tx.state().clone(), owner);
        let choice = pipe.choice.expect("only the ordinary card is discardable");
        assert_eq!(choice.options.len(), 1);
        assert_eq!(choice.options[0].token, ordinary.raw());
    }
}

fn discard_state(package: &[u8], seed: u64) -> (Transaction, PlayerId) {
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
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Discard;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let skills = tx.state().players.get(owner).unwrap().skills.clone();
    for skill in skills {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().hand_cards(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    (tx, owner)
}

fn end_discard(
    state: engine_core::state::GameState,
    owner: PlayerId,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn ordinary_hand_limit_modifier_controls_the_real_discard_phase() {
    let (mut tx, owner) = discard_state(EXTRA, 20_260_780);
    for rank in 1..=4 {
        inject(&mut tx, owner, CardKind::Other, rank);
    }
    let (_, ordinary) = end_discard(tx.state().clone(), owner);
    assert!(ordinary.choice.is_some(), "four cards over a limit of two");

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHANGANDAJIAN_EQUIP5),
        }],
    )
    .unwrap();
    let (modified, modified_pipe) = end_discard(tx.state().clone(), owner);
    assert!(matches!(
        modified_pipe.result,
        CommandResult::Accepted { .. }
    ));
    assert!(modified_pipe.choice.is_none());
    assert_eq!(modified.turn.phase, Phase::End);
}

#[test]
fn protected_sha_neither_counts_nor_can_be_selected_for_phase_discard() {
    let (mut tx, owner) = discard_state(YIJIANG, 20_260_781);
    let sha1 = inject(&mut tx, owner, CardKind::Sha, 1);
    let sha2 = inject(&mut tx, owner, CardKind::ShaFire, 2);
    let keep = [sha1.raw(), sha2.raw()];
    for rank in 3..=5 {
        inject(&mut tx, owner, CardKind::Other, rank);
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINXIANZHEN3),
        }],
    )
    .unwrap();

    assert_eq!(
        engine_core::query::hand_count_for_limit(tx.state(), owner),
        3
    );
    let (state, pipe) = end_discard(tx.state().clone(), owner);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.min_count, 1);
    assert_eq!(choice.max_count, 1);
    assert!(choice
        .options
        .iter()
        .all(|option| !keep.contains(&option.token)));

    let (_, rejected) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![sha1.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let valid = choice.options[0].token;
    let (_, accepted) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![valid],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
}

#[test]
fn standard_new_jiangchi3_combines_all_three_sha_restrictions() {
    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_782,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_JIANGCHI3),
        }],
    )
    .unwrap();

    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 0);
    assert!(engine_core::query::sha_response_cancelled_for(
        tx.state(),
        owner
    ));
    assert!(engine_core::query::sha_ignored_for_hand_limit(
        tx.state(),
        owner
    ));
}
