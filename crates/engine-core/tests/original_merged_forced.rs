//! O2 全量合并第二批（2026-10-14）：家族 forced 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_forced_zone_discard {
//! Goldens for MOBILE「烈斥」and YIJIANG「挥泪」rewritten from legacy JS.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const LIECHI: u32 = 2_902_834_870;
const HUILEI: u32 = 2_889_506_960;
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn started(package: &[u8], seed: u64, players: u8) -> Transaction {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut tx = Transaction::begin(engine.state().clone());
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    tx
}

fn grant(tx: &mut Transaction, player: PlayerId, skill: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill),
        }],
    )
    .unwrap();
}

#[test]
fn liechi_forces_the_damage_source_to_discard_before_the_dying_save() {
    let mut tx = started(MOBILE, LIECHI as u64, 2);
    let source = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != source)
        .unwrap();
    grant(&mut tx, target, LIECHI);
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    let save = tx.state().hand_cards(target)[0];
    tx.state_mut().cards.get_mut(save).unwrap().kind = CardKind::Tao;
    for card in tx.state().hand_cards(source) {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
    let discarded = tx.state().hand_cards(source)[0];

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().resolution.pending_observer, source.raw());
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );

    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(discarded).unwrap().zone, discard);
    assert_eq!(state.resolution.pending_observer, target.raw());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
}

#[test]
fn liechi_ignores_unattributed_dying() {
    let mut tx = started(MOBILE, LIECHI as u64 + 1, 2);
    let owner = tx.state().turn.current;
    grant(&mut tx, owner, LIECHI);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;

    run_effects(&mut tx, vec![EffectRequest::EnterDying { player: owner }]).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn huilei_discards_every_hand_and_equipment_card_of_the_living_killer() {
    let mut tx = started(YIJIANG, HUILEI as u64, 4);
    let killer = tx.state().turn.current;
    let victim = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != killer)
        .unwrap();
    grant(&mut tx, victim, HUILEI);

    let equipment = tx.state().hand_cards(killer)[0];
    tx.state_mut().cards.get_mut(equipment).unwrap().kind = CardKind::Armor;
    let hand = tx.state().zones.find(ZoneKind::Hand, killer).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, killer).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equipment],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    let mut cards = tx.state().hand_cards(killer);
    cards.push(equipment);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer,
        }],
    )
    .unwrap();

    assert_eq!(tx.state().hand_count(killer), 0);
    assert!(tx.state().zones.get(equip).unwrap().cards.is_empty());
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .into_iter()
        .all(|card| tx.state().cards.get(card).unwrap().zone == discard));
}
}
