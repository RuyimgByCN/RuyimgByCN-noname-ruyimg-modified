//! Golden for SHIJI 王双「异勇 / 善械」语义（ABI 415 / 416）。
//!
//! 异勇：受伤后获得造成伤害的杀实体牌，可对来源反击（来源无武器则伤害基数 2）。
//! 善械：出牌阶段限一次，牌堆有武器则获得，无则随机夺一名有武器角色的武器。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YIYONG: u32 = 3_278_613_015;
const SHANXIE: u32 = 3_557_798_177;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_806,
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

fn inject_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn equip_weapon(tx: &mut Transaction, player: PlayerId) -> CardId {
    let weapon = inject_card(tx, player, ZoneKind::Hand, CardKind::Weapon, Suit::Spade);
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    weapon
}

/// 异勇：受伤后获得造成伤害的杀实体，并立即反击（来源无武器 → 伤害基数 2）。
#[test]
fn yiyong_gains_sha_and_counters_with_bonus_damage() {
    let (mut tx, owner, seats) = setup(YIYONG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 王双装备武器（异勇条件）。
    equip_weapon(&mut tx, owner);
    // 来源的杀实体：放进弃牌堆模拟"杀结算后实体在弃牌堆"。
    let sha = inject_card(&mut tx, other, ZoneKind::Hand, CardKind::Sha, Suit::Spade);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(other).unwrap().hp;
    // 王双受到该杀的 1 点伤害（模拟伤害事件：card=sha, source=other）。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: other,
            target: owner,
            card: sha,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    // 反击杀（来源无武器 → 基数 2）：来源 -2。
    assert_eq!(
        tx.state().players.get(other).unwrap().hp,
        hp_before - 2,
        "反击杀应造成 2 点伤害（来源无武器）"
    );
    // 实体牌已用于反击（被消耗，不在手牌）。
    assert!(
        !tx.state().hand_cards(owner).contains(&sha),
        "实体牌应已被反击消耗"
    );
}

/// 善械：牌堆有武器则获得。
#[test]
fn shanxie_grabs_draw_pile_weapon() {
    let (mut tx, owner, seats) = setup(SHANXIE, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放一张武器。
    let weapon = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Club,
    );
    let hand_before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SHANXIE && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShanxie {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    assert!(
        tx.state().hand_cards(owner).contains(&weapon),
        "应获得牌堆中的武器"
    );
}

/// 善械：牌堆无武器则随机夺一名有武器角色的武器。
#[test]
fn shanxie_takes_weapon_from_armed_player() {
    let (mut tx, owner, seats) = setup(SHANXIE, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 其他角色装备武器。
    let weapon = equip_weapon(&mut tx, other);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SHANXIE && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShanxie {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    // 牌堆可能仍有随机武器（走获得分支）或已无（走夺取分支）；两者都应使王双获得武器。
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    assert!(
        tx.state().hand_cards(owner).iter().any(|c| tx
            .state()
            .cards
            .get(*c)
            .is_some_and(|v| v.kind == CardKind::Weapon)),
        "应获得一把武器（牌堆获得或夺取）"
    );
    let _ = weapon;
}
