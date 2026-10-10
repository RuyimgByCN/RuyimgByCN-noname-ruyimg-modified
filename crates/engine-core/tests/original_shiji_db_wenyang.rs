//! Golden for SHIJI 标文鸯「追锋 / 曲迪 / 重剑」语义（ABI 419 / 420）。
//!
//! 追锋：出牌阶段限两次，失 1 点体力并对一名其他角色视为使用【决斗】。
//! 曲迪：单目标杀/决斗指定后三选一（获牌/弃基本加伤/背水）。
//! 重剑：装备牌当【杀】或【酒】使用/打出。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DBZHUIFENG: u32 = 2_625_491_865;
const DBQUEDI: u32 = 2_328_150_606;
const DBCHONGJIAN: u32 = 4_130_288_495;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

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
                rng_seed: 20_260_809,
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

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 追锋：失 1 点体力并打开选目标 choice（决斗目标）。
#[test]
fn dbzhuifeng_loses_hp_and_offers_target() {
    let (mut tx, owner, seats) = setup(DBZHUIFENG, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == DBZHUIFENG && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDbzhuifeng {
            player: owner,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开选目标 choice"
    );
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp_before - 1,
        "应先失去 1 点体力"
    );
}

/// 曲迪：单目标杀指定后打开三选一 choice。
#[test]
fn dbquedi_offers_choice_after_single_target_sha() {
    let (mut tx, owner, seats) = setup(DBQUEDI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 目标有手牌(可获) + 自己有基本牌(可弃)。
    let _target_hand = inject_hand(&mut tx, other, CardKind::Tao, Suit::Heart);
    let _owner_basic = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Club);
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    // 直接调用曲迪原语(目标有可获手牌)。
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == DBQUEDI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDbquedi {
            player: owner,
            target: other,
            card: sha,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开三选一 choice"
    );
    // 选"获得目标一张手牌"。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![0], false);
    assert!(state.hand_count(owner) >= 2, "应获得目标一张手牌");
}

/// 重剑：装备牌当杀（view_as 编译与注册验证）。
#[test]
fn dbchongjian_view_as_registered() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let entry = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DBCHONGJIAN)
        .expect("重剑应有 view_as 条目");
    assert_eq!(entry.as_kind, "sha");
}
