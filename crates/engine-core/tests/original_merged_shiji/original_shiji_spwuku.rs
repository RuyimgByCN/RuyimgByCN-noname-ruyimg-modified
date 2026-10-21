//! Golden for SHIJI「武库」的装备牌与三枚标记上限语义。
//!
//! 杜预（sp_duyu）已由「武库」+「三陈」+「灭吴」闭环入选将池。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPWUKU: u32 = 2_480_520_461;
const SPSANCHEN: u32 = 3_539_331_531;
const SP_DUYU: u32 = 2_415_684_974;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: kind.is_equip().then_some(2).unwrap_or(0),
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == SPWUKU)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_releases_wuku_and_closed_duyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // 武库以独立稳定 ID 发布，装备触发过滤器为权威语义来源。
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == SPWUKU
            && skill.locked
            && package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == SPWUKU
                    && trigger.event == "card_used"
                    && trigger.filter == "owner_alive_equip_used_skill_mark_below_three"
            })
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_DUYU)
        .expect("sp_duyu closed character");
    assert_eq!(character.skills, vec![SPWUKU, SPSANCHEN]);
}

#[test]
fn only_equipment_uses_add_wuku_and_the_mark_caps_at_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let user = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(SPWUKU),
    });
    run_effects(&mut tx, effects).unwrap();

    // 非装备牌的使用不获得“武库”。
    let sha = inject(&mut tx, user, CardKind::Sha, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: user,
            card: sha,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(mark_count(&tx, owner), 0);

    // 连续四张装备牌使用后标记封顶为三。
    for rank in 2..=5 {
        let equip = inject(&mut tx, user, CardKind::Weapon, rank);
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: user,
                card: equip,
                targets: vec![user],
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(&tx, owner), 3);
}
