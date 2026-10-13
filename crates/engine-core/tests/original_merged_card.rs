//! O2 全量合并批次 1（2026-10-14）：家族 card 的 4 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_card_category_protection {
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
}

mod original_card_distance_unlimited {
//! Golden coverage for the shared all-card distance-unlimited modifier.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const DRLT_JUEYAN2: u32 = 3_629_461_497;
const QINGYI3: u32 = 3_610_775_756;
const ZHANYI_TRICK: u32 = 2_374_118_291;
const FEIYING: u32 = 2_669_566_023;
const QICAI: u32 = 2_286_640_837;
const DUANLIANG: u32 = 2_757_928_964;
const DUANLIANG2: u32 = 2_158_464_053;
const DUANLIANG3: u32 = 4_145_070_079;
const GZDUANLIANG: u32 = 2_148_140_249;

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let skills = tx.state().players.get(player).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

fn inject_hand_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_700 + kind.tag() as u32),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn transaction(package: &[u8], seed: u64) -> (Transaction, PlayerId, PlayerId) {
    transaction_players(package, seed, 4)
}

fn transaction_players(
    package: &[u8],
    seed: u64,
    player_count: u8,
) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + seats.len() / 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    strip_skills(&mut tx, actor);
    strip_skills(&mut tx, far);
    (tx, actor, far)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        response => panic!("unexpected distance response: {response:?}"),
    }
}

#[test]
fn legacy_skills_unlock_sha_shunshou_and_bingliang_at_distance_two() {
    for (case, (package, skill_def)) in [(SHENHUA, DRLT_JUEYAN2), (SP, QINGYI3), (SP, ZHANYI_TRICK)]
        .into_iter()
        .enumerate()
    {
        for (kind_index, kind) in [
            CardKind::Sha,
            CardKind::TrickShunshou,
            CardKind::TrickBingliang,
        ]
        .into_iter()
        .enumerate()
        {
            let (mut tx, actor, far) =
                transaction(package, 20_260_760 + (case * 10 + kind_index) as u64);
            let card = inject_hand_card(&mut tx, actor, kind);
            let command = Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::PlayCard {
                    card,
                    targets: vec![far],
                    card_targets: vec![],
                },
            };

            let (_, rejected) = apply_command(tx.state().clone(), command.clone());
            assert!(
                matches!(rejected.result, CommandResult::Rejected { .. }),
                "{kind:?} should keep its ordinary distance limit"
            );

            run_effects(
                &mut tx,
                vec![EffectRequest::AddSkill {
                    player: actor,
                    skill_def: SkillDefId::new(skill_def),
                }],
            )
            .unwrap();
            assert!(engine_core::query::card_distance_unlimited_for(
                tx.state(),
                actor
            ));
            let (_, accepted) = apply_command(tx.state().clone(), command);
            assert!(
                matches!(accepted.result, CommandResult::Accepted { .. }),
                "{kind:?} should ignore distance with skill {skill_def}: {:?}",
                accepted.result
            );
        }
    }
}

#[test]
fn qicai_only_unlocks_trick_distance() {
    for (index, kind) in [CardKind::TrickShunshou, CardKind::TrickBingliang]
        .into_iter()
        .enumerate()
    {
        let (mut tx, actor, far) = transaction(STANDARD, 20_260_800 + index as u64);
        let card = inject_hand_card(&mut tx, actor, kind);
        assert!(!engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            card,
            far,
            1,
        ));
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(QICAI),
            }],
        )
        .unwrap();
        assert!(engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            card,
            far,
            1,
        ));
        let (_, accepted) = apply_command(
            tx.state().clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::PlayCard {
                    card,
                    targets: vec![far],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    }

    let (mut tx, actor, far) = transaction(STANDARD, 20_260_802);
    let sha = inject_hand_card(&mut tx, actor, CardKind::Sha);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: SkillDefId::new(QICAI),
        }],
    )
    .unwrap();
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        sha,
        far,
        1,
    ));
}

#[test]
fn feiying_only_increases_distance_to_its_owner() {
    let (mut tx, actor, far) = transaction(STANDARD, 20_260_803);
    assert_eq!(distance(&tx, actor, far), 2);
    assert_eq!(distance(&tx, far, actor), 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: far,
            skill_def: SkillDefId::new(FEIYING),
        }],
    )
    .unwrap();
    assert_eq!(distance(&tx, actor, far), 3);
    assert_eq!(distance(&tx, far, actor), 2);
}

#[test]
fn duanliang2_extends_only_bingliang_to_distance_two() {
    let (mut tx, actor, far) = transaction(SHENHUA, 20_260_804);
    assert_eq!(distance(&tx, actor, far), 2);
    let bingliang = inject_hand_card(&mut tx, actor, CardKind::TrickBingliang);
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        bingliang,
        far,
        1,
    ));
    add_skill(&mut tx, actor, GZDUANLIANG);
    assert!(engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        bingliang,
        far,
        1,
    ));
    let (_, accepted) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::PlayCard {
                card: bingliang,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));

    let shunshou = inject_hand_card(&mut tx, actor, CardKind::TrickShunshou);
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        shunshou,
        far,
        1,
    ));

    let (mut tx, actor, far) = transaction_players(SHENHUA, 20_260_805, 6);
    assert_eq!(distance(&tx, actor, far), 3);
    let bingliang = inject_hand_card(&mut tx, actor, CardKind::TrickBingliang);
    add_skill(&mut tx, actor, DUANLIANG2);
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        actor,
        bingliang,
        far,
        1,
    ));
    let (_, rejected) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::PlayCard {
                card: bingliang,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn duanliang3_unlocks_only_bingliang_for_targets_with_at_least_as_many_hand_cards() {
    for (seed, skill_def) in [(20_260_806, DUANLIANG3), (20_260_807, DUANLIANG)] {
        let (mut tx, actor, far) = transaction(SHENHUA, seed);
        assert_eq!(distance(&tx, actor, far), 2);
        let bingliang = inject_hand_card(&mut tx, actor, CardKind::TrickBingliang);
        add_skill(&mut tx, actor, skill_def);

        while tx.state().hand_cards(far).len() >= tx.state().hand_cards(actor).len() {
            inject_hand_card(&mut tx, actor, CardKind::Other);
        }
        assert!(!engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            bingliang,
            far,
            1,
        ));

        while tx.state().hand_cards(far).len() < tx.state().hand_cards(actor).len() {
            inject_hand_card(&mut tx, far, CardKind::Other);
        }
        assert!(engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            bingliang,
            far,
            1,
        ));
        let (_, accepted) = apply_command(
            tx.state().clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(5),
                kind: CommandKind::PlayCard {
                    card: bingliang,
                    targets: vec![far],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));

        let shunshou = inject_hand_card(&mut tx, actor, CardKind::TrickShunshou);
        assert!(!engine_core::query::card_target_within_distance(
            tx.state(),
            actor,
            shunshou,
            far,
            1,
        ));
    }
}
}

mod original_card_target_draw {
//! Golden for legacy CardUsed target/source draw skills.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const TSUMUGI_MUGYU: u32 = 3_191_570_097;
const YUI_JIANG: u32 = 2_788_570_580;
const JIANG: u32 = 2_402_221_407;
const NONAME_JIANG: u32 = 4_012_037_429;
const NSSHUANGXIONG: u32 = 2_353_402_107;
const AO_XISHI: u32 = 4_100_198_765;
const SPDENGLI: u32 = 3_419_124_885;
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SHIJI: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");
const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn setup(package: &[u8], skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_714 + skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_card_used(
    tx: &mut Transaction,
    user: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn run_dispatched(tx: &mut Transaction, events: &[DomainEvent]) {
    let effects = dispatch_for_events(tx, events, &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

fn confirm(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn tsumugi_mugyu_requires_owner_target_and_hand_below_max_hp() {
    let (mut tx, owner, seats) = setup(KEY, TSUMUGI_MUGYU, 2);
    let source = seats.into_iter().find(|player| *player != owner).unwrap();
    let max_hp = tx.state().hand_count(owner).max(1);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = max_hp;
    let discard_count = tx
        .state()
        .hand_count(owner)
        .saturating_sub(max_hp.saturating_sub(1));
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: discard_count,
        }],
    )
    .unwrap();
    let card = inject(&mut tx, source, CardKind::Other, Suit::Spade);
    let before = tx.state().hand_count(owner);
    assert!(before < max_hp);

    assert!(dispatch_card_used(&mut tx, source, card, vec![source])
        .unwrap()
        .is_empty());
    assert!(!tx.state().has_pending_choice());

    assert_eq!(
        dispatch_card_used(&mut tx, source, card, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(tx, owner, 1);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(state.hand_count(owner), max_hp);

    let mut full = Transaction::begin(state);
    assert!(dispatch_card_used(&mut full, source, card, vec![owner])
        .unwrap()
        .is_empty());
    assert!(!full.state().has_pending_choice());
}

fn assert_jiang_semantics(package: &[u8], skill_def: u32) {
    let (mut negative, owner, seats) = setup(package, skill_def, 3);
    let source = seats
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    let black_sha = inject(&mut negative, source, CardKind::Sha, Suit::Spade);
    let red_other = inject(&mut negative, source, CardKind::Other, Suit::Heart);
    assert!(
        dispatch_card_used(&mut negative, source, black_sha, vec![owner])
            .unwrap()
            .is_empty()
    );
    assert!(
        dispatch_card_used(&mut negative, source, red_other, vec![owner])
            .unwrap()
            .is_empty()
    );

    let (mut targeted, owner, seats) = setup(package, skill_def, 3);
    let source = seats
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    let red_sha = inject(&mut targeted, source, CardKind::Sha, Suit::Diamond);
    let before = targeted.state().hand_count(owner);
    assert_eq!(
        dispatch_card_used(&mut targeted, source, red_sha, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(targeted, owner, 2);
    assert_eq!(state.hand_count(owner), before + 1);

    let (mut used, owner, seats) = setup(package, skill_def, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let juedou = inject(&mut used, owner, CardKind::TrickJuedou, Suit::Club);
    let before = used.state().hand_count(owner);
    assert_eq!(
        dispatch_card_used(&mut used, owner, juedou, targets).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(used, owner, 3);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn inherited_jiang_skills_match_juedou_and_red_sha_once_per_card() {
    assert_jiang_semantics(KEY, YUI_JIANG);
    assert_jiang_semantics(SHENHUA, JIANG);
    assert_jiang_semantics(DIY, NONAME_JIANG);
}

#[test]
fn nsshuangxiong_only_offers_for_owner_participating_in_juedou() {
    let (mut targeted, owner, seats) = setup(DIY, NSSHUANGXIONG, 3);
    let source = seats.into_iter().find(|player| *player != owner).unwrap();
    let red_sha = inject(&mut targeted, source, CardKind::Sha, Suit::Heart);
    assert!(
        dispatch_card_used(&mut targeted, source, red_sha, vec![owner])
            .unwrap()
            .is_empty()
    );

    let juedou = inject(&mut targeted, source, CardKind::TrickJuedou, Suit::Club);
    assert_eq!(
        dispatch_card_used(&mut targeted, source, juedou, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(targeted, owner, 6);
    assert!(state.players.get(owner).unwrap().turned_over);

    let (mut used, owner, seats) = setup(DIY, NSSHUANGXIONG, 3);
    let target = seats.into_iter().find(|player| *player != owner).unwrap();
    let juedou = inject(&mut used, owner, CardKind::TrickJuedou, Suit::Spade);
    assert_eq!(
        dispatch_card_used(&mut used, owner, juedou, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(used, owner, 7);
    assert!(state.players.get(owner).unwrap().turned_over);
}

#[test]
fn spdengli_fires_once_for_each_equal_hp_sha_target() {
    let (mut tx, owner, seats) = setup(SHIJI, SPDENGLI, 3);
    for player in &seats {
        tx.state_mut().players.get_mut(*player).unwrap().hp = 3;
    }
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_card_used(&mut tx, owner, sha, targets).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let state = confirm(tx, owner, 4);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(state.has_pending_choice());
    let state = confirm(Transaction::begin(state), owner, 5);
    assert_eq!(state.hand_count(owner), before + 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn ao_xishi_covers_diamond_use_target_and_response_once_each() {
    let (mut tx, owner, seats) = setup(KEY, AO_XISHI, 3);
    let source = seats
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    let others: Vec<_> = seats
        .iter()
        .copied()
        .filter(|player| *player != owner)
        .collect();

    let used = inject(&mut tx, owner, CardKind::Other, Suit::Diamond);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: used,
            targets: others,
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before + 1);

    let targeted = inject(&mut tx, source, CardKind::Other, Suit::Diamond);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card: targeted,
            targets: vec![owner],
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before + 1);

    let response = inject(&mut tx, owner, CardKind::Shan, Suit::Diamond);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: response,
            response_kind: CardKind::Shan.tag(),
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before + 1);

    let club = inject(&mut tx, owner, CardKind::Shan, Suit::Club);
    let before = tx.state().hand_count(owner);
    run_dispatched(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: club,
            response_kind: CardKind::Shan.tag(),
        }],
    );
    assert_eq!(tx.state().hand_count(owner), before);
}
}

mod original_card_use_history {
//! Golden tests for legacy skills driven by per-player card-use history.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const NSLONGYUE: u32 = 2_643_354_275;
const NSLONGYUE_AI: u32 = 3_436_136_112;
const JDJIZHI: u32 = 3_847_328_245;
const DCJIANYING: u32 = 2_862_591_465;
const REJIZHI: u32 = 3_401_109_480;
const REQICAI: u32 = 2_792_486_309;
const RE_HUANGYUEYING: u32 = 4_247_764_745;
const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(package: &[u8], skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == skill_def));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(u64::from(skill_def)),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: u64::from(skill_def),
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    // roster 变动会平移 bootstrap 身份分配（offline 新将批 +3）——排空开局可选
    // 选择，避免后续 run_effects 被挂起 ChoiceRequested 打断（qimen 模式）。
    let mut drain_key = u64::from(skill_def).wrapping_mul(2654435761);
    while engine.state().has_pending_choice() {
        drain_key += 1;
        let choice_id = engine.state().resolution.pending_choice_id;
        let revision = engine.state().revision;
        let observer = engine.state().resolution.pending_observer;
        let _ = engine.submit(Command {
            actor: PlayerId::new(observer),
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(drain_key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![],
                cancelled: true,
            },
        });
    }
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn use_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> Result<(), EffectError> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000 + kind.tag() as u32 * 16 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        tx,
        vec![EffectRequest::UseCard {
            player,
            card,
            targets: vec![],
        }],
    )
    .map(|_| ())
}

fn resolve(tx: Transaction, owner: PlayerId, accept: bool, key: u64) -> Transaction {
    let state = tx.state();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: accept.then_some(1).into_iter().collect(),
                cancelled: !accept,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    Transaction::begin(state)
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected hand-limit integer, got {other:?}"),
    }
}

#[test]
fn refresh_package_closes_re_huangyueying_with_physical_rejizhi() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 723);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == RE_HUANGYUEYING)
            .unwrap()
            .skills,
        [REJIZHI, REQICAI]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REQICAI));
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == REJIZHI)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "card_used"
            && trigger.filter == "owner_source_physical_ordinary_trick"
            && trigger.optional
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "turn_ended" && trigger.filter == "owner_alive" && !trigger.optional
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == REJIZHI
            && modifier.target == "hand_limit"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_skill_mark")
    }));
}

#[test]
fn rejizhi_discards_the_basic_draw_for_hand_limit_until_turn_end() {
    let (mut tx, owner, other) = setup(REFRESH, REJIZHI);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let drawn = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(drawn).unwrap().kind = CardKind::Sha;
    let base_limit = hand_limit(&tx, owner);

    assert_eq!(
        use_card(&mut tx, owner, CardKind::TrickWuzhong, Suit::Heart, 7),
        Err(EffectError::ChoiceRequested)
    );
    let tx = resolve(tx, owner, true, 10);
    assert!(tx.state().has_pending_choice());
    let mut tx = resolve(tx, owner, true, 11);
    assert!(!tx.state().hand_cards(owner).contains(&drawn));
    assert_eq!(hand_limit(&tx, owner), base_limit + 1);
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == REJIZHI)
            .map(|(_, count)| *count),
        Some(1)
    );

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(hand_limit(&tx, owner), base_limit);
}

#[test]
fn nslongyue_only_offers_for_each_players_first_ordinary_trick() {
    let package = content_schema::ContentPackage::from_json_bytes(DIY).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSLONGYUE_AI));
    assert!(package
        .trigger_index
        .iter()
        .all(|trigger| trigger.skill_def_id != NSLONGYUE_AI));

    let (mut tx, owner, source) = setup(DIY, NSLONGYUE);
    let initial = tx.state().hand_count(source);
    let paused = use_card(&mut tx, source, CardKind::TrickWuzhong, Suit::Spade, 7).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let mut tx = resolve(tx, owner, true, 1);
    assert_eq!(tx.state().hand_count(source), initial + 1);
    use_card(&mut tx, source, CardKind::TrickGuohe, Suit::Heart, 8).unwrap();
    assert!(!tx.state().has_pending_choice());

    let (mut tx, _, source) = setup(DIY, NSLONGYUE);
    use_card(&mut tx, source, CardKind::TrickLe, Suit::Club, 6).unwrap();
    use_card(&mut tx, source, CardKind::TrickWuzhong, Suit::Diamond, 9).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn jdjizhi_draws_and_uses_its_full_u32_skill_id_as_a_turn_mark() {
    let (mut tx, owner, current) = setup(OFFLINE, JDJIZHI);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let base_limit = hand_limit(&tx, owner);
    use_card(&mut tx, owner, CardKind::TrickWuzhong, Suit::Heart, 7).unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);
    assert_eq!(hand_limit(&tx, owner), base_limit + 1);
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == JDJIZHI)
            .map(|(_, count)| *count),
        Some(1)
    );
    use_card(&mut tx, owner, CardKind::TrickLe, Suit::Spade, 8).unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);
    assert_eq!(hand_limit(&tx, owner), base_limit + 1);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(hand_limit(&tx, owner), base_limit);
}

#[test]
fn dcjianying_compares_with_the_previous_used_card_across_turns() {
    let (mut tx, owner, other) = setup(REFRESH, DCJIANYING);
    let initial = tx.state().hand_count(owner);
    use_card(&mut tx, owner, CardKind::Other, Suit::Spade, 7).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Prepare.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    let paused = use_card(&mut tx, owner, CardKind::Other, Suit::Heart, 7).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let mut tx = resolve(tx, owner, true, 2);
    assert_eq!(tx.state().hand_count(owner), initial + 1);

    let paused = use_card(&mut tx, owner, CardKind::Other, Suit::Heart, 8).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let tx = resolve(tx, owner, false, 3);
    assert_eq!(tx.state().hand_count(owner), initial + 1);
}
}
