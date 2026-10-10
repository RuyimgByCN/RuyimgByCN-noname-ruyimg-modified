//! Golden for reusable compound card selectors used by historical target protection skills.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WEIMU: u32 = 4_145_129_024;
const MPBISHI: u32 = 3_233_124_559;
const HUOSHOU1: u32 = 2_951_836_249;
const JUXIANG1: u32 = 3_192_375_369;
const OLZHENNAN2: u32 = 2_917_646_531;
const ZHICHI2: u32 = 3_197_261_303;
const YIZHONG: u32 = 2_156_296_205;
const ZYMINGSHI: u32 = 4_161_488_331;
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const NEWJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn loaded(package: &[u8], seed: u64, skill: u32) -> (Transaction, PlayerId, PlayerId) {
    loaded_packages(&[package], seed, skill)
}

fn loaded_packages(packages: &[&[u8]], seed: u64, skill: u32) -> (Transaction, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    for package in packages {
        let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
        engine.load_package(&package).unwrap();
    }
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let user = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != user)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|active| EffectRequest::RemoveSkill { skill: active.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: target,
        skill_def: SkillDefId::new(skill),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = user;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, user, target)
}

fn target_legal(
    tx: &Transaction,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn assert_command_rejected(
    tx: &Transaction,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) {
    let before = tx.state().clone();
    let (after, pipe) = apply_command(
        before.clone(),
        Command {
            actor: user,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(card.raw() as u64),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(after, before);
}

fn equip_armor(tx: &mut Transaction, player: PlayerId) {
    let armor = tx.state().hand_cards(player)[0];
    tx.state_mut().cards.get_mut(armor).unwrap().kind = CardKind::Armor;
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![armor],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
}

#[test]
fn weimu_blocks_black_tricks_but_not_red_tricks_or_black_basic_cards() {
    let (mut tx, user, target) = loaded(SHENHUA, 20_260_718, WEIMU);
    let cards = tx.state().hand_cards(user);
    assert!(cards.len() >= 3);
    let black_trick = cards[0];
    let red_trick = cards[1];
    let black_basic = cards[2];
    {
        let card = tx.state_mut().cards.get_mut(black_trick).unwrap();
        card.kind = CardKind::TrickGuohe;
        card.suit = Suit::Spade;
    }
    {
        let card = tx.state_mut().cards.get_mut(red_trick).unwrap();
        card.kind = CardKind::TrickGuohe;
        card.suit = Suit::Heart;
    }
    {
        let card = tx.state_mut().cards.get_mut(black_basic).unwrap();
        card.kind = CardKind::Sha;
        card.suit = Suit::Club;
    }

    assert!(!target_legal(&tx, user, black_trick, target));
    assert!(target_legal(&tx, user, red_trick, target));
    assert!(target_legal(&tx, user, black_basic, target));
    assert_command_rejected(&tx, user, black_trick, target);
}

#[test]
fn mpbishi_blocks_damage_tricks_but_not_other_tricks() {
    let (mut tx, user, target) = loaded(NEWJIANG, 20_260_719, MPBISHI);
    let cards = tx.state().hand_cards(user);
    assert!(cards.len() >= 3);
    let juedou = cards[0];
    let huogong = cards[1];
    let guohe = cards[2];
    tx.state_mut().cards.get_mut(juedou).unwrap().kind = CardKind::TrickJuedou;
    tx.state_mut().cards.get_mut(huogong).unwrap().kind = CardKind::TrickHuogong;
    tx.state_mut().cards.get_mut(guohe).unwrap().kind = CardKind::TrickGuohe;

    assert!(!target_legal(&tx, user, juedou, target));
    assert!(!target_legal(&tx, user, huogong, target));
    assert!(target_legal(&tx, user, guohe, target));
    assert_command_rejected(&tx, user, juedou, target);
}

#[test]
fn nanman_protection_is_shared_without_blocking_other_tricks() {
    for (package, skill, seed) in [
        (SHENHUA, HUOSHOU1, 20_260_812),
        (SHENHUA, JUXIANG1, 20_260_813),
        (SP, OLZHENNAN2, 20_260_814),
    ] {
        let (mut tx, user, target) = loaded(package, seed, skill);
        let cards = tx.state().hand_cards(user);
        tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::TrickNanman;
        tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickGuohe;

        assert!(!target_legal(&tx, user, cards[0], target));
        assert!(target_legal(&tx, user, cards[1], target));
    }
}

#[test]
fn zhichi_blocks_sha_and_ordinary_tricks_but_not_delayed_tricks() {
    let (mut tx, user, target) = loaded(YIJIANG, 20_260_815, ZHICHI2);
    let cards = tx.state().hand_cards(user);
    for (card, kind) in [
        (cards[0], CardKind::Sha),
        (cards[1], CardKind::TrickGuohe),
        (cards[2], CardKind::TrickLe),
    ] {
        tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
    }

    assert!(!target_legal(&tx, user, cards[0], target));
    assert!(!target_legal(&tx, user, cards[1], target));
    assert!(target_legal(&tx, user, cards[2], target));
}

#[test]
fn armor_empty_protection_matches_black_and_elemental_sha_only() {
    let (mut tx, user, target) = loaded(YIJIANG, 20_260_816, YIZHONG);
    let cards = tx.state().hand_cards(user);
    for (card, kind, suit) in [
        (cards[0], CardKind::Sha, Suit::Spade),
        (cards[1], CardKind::Sha, Suit::Heart),
        (cards[2], CardKind::TrickGuohe, Suit::Club),
    ] {
        let card = tx.state_mut().cards.get_mut(card).unwrap();
        card.kind = kind;
        card.suit = suit;
    }
    assert!(!target_legal(&tx, user, cards[0], target));
    assert!(target_legal(&tx, user, cards[1], target));
    assert!(target_legal(&tx, user, cards[2], target));
    equip_armor(&mut tx, target);
    assert!(target_legal(&tx, user, cards[0], target));

    let (mut tx, user, target) = loaded(OFFLINE, 20_260_817, ZYMINGSHI);
    let cards = tx.state().hand_cards(user);
    for (card, kind) in [
        (cards[0], CardKind::Sha),
        (cards[1], CardKind::ShaFire),
        (cards[2], CardKind::ShaThunder),
    ] {
        tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
    }
    assert!(target_legal(&tx, user, cards[0], target));
    assert!(!target_legal(&tx, user, cards[1], target));
    assert!(!target_legal(&tx, user, cards[2], target));
    equip_armor(&mut tx, target);
    assert!(target_legal(&tx, user, cards[1], target));
    assert!(target_legal(&tx, user, cards[2], target));
}

#[test]
fn old_fuyin_checks_card_name_hand_counts_and_empty_armor_slot() {
    let (mut tx, user, target) = loaded_packages(&[SHENHUA, OLD], 20_260_811, 3_976_662_486);
    let cards = tx.state().hand_cards(user);
    assert!(cards.len() >= 4);
    for (card, kind) in [
        (cards[0], CardKind::Sha),
        (cards[1], CardKind::TrickJuedou),
        (cards[2], CardKind::TrickHuogong),
        (cards[3], CardKind::TrickGuohe),
    ] {
        tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
    }

    assert!(!target_legal(&tx, user, cards[0], target));
    assert!(!target_legal(&tx, user, cards[1], target));
    assert!(!target_legal(&tx, user, cards[2], target));
    assert!(target_legal(&tx, user, cards[3], target));
    assert_command_rejected(&tx, user, cards[0], target);

    let target_draw = tx
        .state()
        .hand_count(user)
        .saturating_sub(tx.state().hand_count(target))
        + 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: target_draw,
        }],
    )
    .unwrap();
    assert!(
        target_legal(&tx, user, cards[0], target),
        "user hand {}, target hand {}",
        tx.state().hand_count(user),
        tx.state().hand_count(target)
    );

    let user_draw = tx
        .state()
        .hand_count(target)
        .saturating_sub(tx.state().hand_count(user));
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: user,
            count: user_draw,
        }],
    )
    .unwrap();
    assert!(!target_legal(&tx, user, cards[0], target));

    equip_armor(&mut tx, target);
    assert!(target_legal(&tx, user, cards[0], target));
}
