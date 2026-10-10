//! Golden for 界限突破「破军」伤害子技能的逐目标手牌、装备比较语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::intent::{resolve_damage_intent_with_kind, DamageIntent};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const REPOJUN3: u32 = 2_423_810_510;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REPOJUN3 && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(REPOJUN3 as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: REPOJUN3 as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(REPOJUN3),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1])
}

fn damage_amount(tx: &Transaction, source: PlayerId, target: PlayerId, logical_sha: bool) -> u32 {
    resolve_damage_intent_with_kind(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
        logical_sha,
    )
    .amount
}

fn equip_one_hand_card(tx: &mut Transaction, player: PlayerId) {
    let card = tx.state().hand_cards(player)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Armor;
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
}

#[test]
fn bonus_requires_both_hand_and_equipment_counts_to_be_no_smaller() {
    let (mut tx, source, target) = isolated();

    assert_eq!(damage_amount(&tx, source, target, true), 2);

    equip_one_hand_card(&mut tx, target);
    assert_eq!(damage_amount(&tx, source, target, true), 1);

    equip_one_hand_card(&mut tx, source);
    assert_eq!(damage_amount(&tx, source, target, true), 2);

    let source_hand = tx.state().hand_count(source);
    let target_hand = tx.state().hand_count(target);
    let discard_count = source_hand.saturating_sub(target_hand).saturating_add(1) as usize;
    let cards = tx.state().hand_cards(source)[..discard_count].to_vec();
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
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
    assert_eq!(damage_amount(&tx, source, target, true), 1);
}

#[test]
fn bonus_is_limited_to_logical_sha_damage() {
    let (tx, source, target) = isolated();
    assert_eq!(damage_amount(&tx, source, target, true), 2);
    assert_eq!(damage_amount(&tx, source, target, false), 1);
}
