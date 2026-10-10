//! Golden for SHIJI 卞夫人（sp_bianfuren）与「挽危」「约俭」语义（ABI 401）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SP_BIANFUREN: u32 = 3_610_270_819;
const SPWANWEI: u32 = 3_728_402_634;
const SPYUEJIAN: u32 = 2_751_500_626;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    for c in cards {
        engine_core::effect::move_one_card(tx, c, hand, discard, PlayerId::NONE, false).unwrap();
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

#[test]
fn package_closes_sp_bianfuren_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 401);

    // 卞夫人闭包包含「挽危」「约俭」且顺序完全一致。
    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_BIANFUREN)
        .expect("sp_bianfuren character index entry");
    assert_eq!(character.name_key, "sp_bianfuren");
    assert_eq!(character.skills, vec![SPWANWEI, SPYUEJIAN]);
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wei");
}

#[test]
fn spwanwei_active_recovers_target_and_loses_owner_hp() {
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
                rng_seed: 20_260_805,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    // 清空随机选将技能，避免包内新增武将改变种子分配后触发无关选择。
    let strip: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, strip).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 4;

    let _ = add_skill(&mut tx, owner, SPWANWEI);

    // 让 target 受到 2 点伤害。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();

    let owner_initial_hp = tx.state().players.get(owner).unwrap().hp;

    // 发动挽危主动：target 回复 owner.hp (4 点)，owner 失去 4 点体力。
    run_effects(
        &mut tx,
        vec![EffectRequest::SpwanweiActive {
            player: owner,
            target,
        }],
    )
    .unwrap();

    // target 回复到体力上限。
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        tx.state().players.get(target).unwrap().max_hp as i32
    );
    // owner 扣减 num 点体力。
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        owner_initial_hp - owner_initial_hp
    );
}

#[test]
fn spyuejian_discards_two_cards_and_recovers_one_hp_on_dying() {
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
                rng_seed: 20_260_806,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;

    let mut tx = Transaction::begin(engine.state().clone());
    clear_hand(&mut tx, owner);

    let _ = add_skill(&mut tx, owner, SPYUEJIAN);

    // 注入 2 张手牌。
    let _c1 = inject_hand(&mut tx, owner, CardKind::Tao, 1);
    let _c2 = inject_hand(&mut tx, owner, CardKind::Sha, 2);

    assert_eq!(tx.state().hand_count(owner), 2);

    // 进入濒死状态。
    let err = run_effects(&mut tx, vec![EffectRequest::EnterDying { player: owner }]).unwrap_err();

    // 约俭触发可选 ChoiceRequested。
    assert!(matches!(err, EffectError::ChoiceRequested));
}
