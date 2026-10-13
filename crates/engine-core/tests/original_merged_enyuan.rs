//! O2 全量合并第二批（2026-10-14）：家族 enyuan 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_enyuan_fuzai_dilve {
//! Golden coverage for the 2026-08-21 cross-pack batch: onlyOL 覆载 / 恩怨2,
//! KEY 底略, yijiang 新恩怨2 方向修复。零新原语（ABI 不变：onlyOL 569 / key 651 /
//! yijiang 637）。覆载/恩怨2(OL) noop 注记系误标，本批一并纠正。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLFUZAI: u32 = 3_863_403_605;
const OLENYUAN2: u32 = 2_463_972_895;
const YUIKO_DILVE: u32 = 3_357_924_726;
const XINENYUAN2: u32 = 4_121_865_946;

const FUZAI_SRC: &str = include_str!("../../../content/original/ported/onlyOL/olfuzai.skill");
const OLENYUAN_SRC: &str =
    include_str!("../../../content/original/ported/onlyOL/olenyuan2.skill");
const DILVE_SRC: &str = include_str!("../../../content/original/ported/key/yuiko_dilve.skill");
const ENYUAN_SRC: &str =
    include_str!("../../../content/original/ported/yijiang/xinenyuan2.skill");
const OL_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
const KEY_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_discard(tx: &mut Transaction, def: u32) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 8,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn fuzai_source_and_npack_publish_equip_conversion() {
    let compiled = content_compiler::compile_source_named("olfuzai", FUZAI_SRC).unwrap();
    let mut pairs: Vec<(String, String, String)> = compiled
        .view_as_index
        .iter()
        .filter(|entry| entry.skill_def_id == OLFUZAI)
        .map(|entry| {
            (
                entry.as_kind.clone(),
                entry.filter.clone(),
                entry.source_zone.clone().unwrap_or_default(),
            )
        })
        .collect();
    pairs.sort();
    assert_eq!(
        pairs,
        vec![
            ("jiedao".into(), "card_equip".into(), "hand_equip".into()),
            ("wuzhong".into(), "card_equip".into(), "hand_equip".into()),
        ]
    );
    let pack = content_schema::ContentPackage::from_json_bytes(OL_PACKAGE).unwrap();
    assert_eq!(
        pack.view_as_index
            .iter()
            .filter(|entry| entry.skill_def_id == OLFUZAI)
            .count(),
        2
    );
}

#[test]
fn dilve_gains_compare_cards_from_discard() {
    let (mut tx, owner, other) = bootstrap(DILVE_SRC, "dilve-test", YUIKO_DILVE);
    let first = inject_discard(&mut tx, 333_101);
    let second = inject_discard(&mut tx, 333_102);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::HandCompareFinished {
                player: owner,
                target: other,
                player_card: first,
                target_card: second,
                winner: owner,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), hand_before + 2);
}

#[test]
fn xinenyuan2_source_gives_damaged_owner_a_card() {
    let (mut tx, owner, other) = bootstrap(ENYUAN_SRC, "enyuan-test", XINENYUAN2);
    let gift = inject_hand(&mut tx, other, 333_103);
    let owner_before = tx.state().hand_count(owner);
    let source_before = tx.state().hand_count(other);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    // optional confirm（受伤者确认）→ 来源选一张手牌交出（方向回归：来源 -1、自己 +1）。
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    let state = resolve(state, other, vec![u32::from(gift.raw())], 4);
    assert_eq!(state.hand_count(owner), owner_before + 1);
    assert_eq!(state.hand_count(other), source_before - 1);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert!(state.zones.get(hand).unwrap().cards.contains(&gift));
}

#[test]
fn olenyuan2_source_and_npack_publish_direction() {
    let compiled =
        content_compiler::compile_source_named("olenyuan2", OLENYUAN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLENYUAN2)
        .expect("olenyuan2 trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(OL_PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLENYUAN2 && trigger.event == "damage"
    }));
    let _ = KEY_PACKAGE;
    let _ = YIJIANG_PACKAGE;
}
}
