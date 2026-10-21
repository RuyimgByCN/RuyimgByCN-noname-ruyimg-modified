//! Golden for SHIJI 糜夫人「清雨」语义（ABI 431）。
//!
//! 清雨：使命技——受伤前弃 2 张 he 防伤（defend）；准备阶段满血且手牌 < 轮数
//! 使命成功获【选存】（achieve）；濒死使命失败减 1 上限并选一人获【勇决】（fail）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const QINGYU: u32 = 3_126_505_108;
const XUANCUN: u32 = 2_417_220_435;
const MBYONGJUE: u32 = 4_187_437_274;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn setup(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(QINGYU as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_814,
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
        skill_def: SkillDefId::new(QINGYU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    tx.state_mut().zones.get_mut(zone_id).unwrap().cards.clear();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId) -> CardId {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + CardKind::Tao.tag() as u32),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
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

fn skill_instance(tx: &Transaction, owner: PlayerId) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == QINGYU && s.owner == owner)
        .map(|s| s.id)
        .unwrap()
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

fn qingyu_request(player: PlayerId, action: u8, skill: engine_api::SkillId) -> EffectRequest {
    EffectRequest::ResolveQingyu {
        player,
        action,
        xuancun_def: SkillDefId::new(XUANCUN),
        mbyongjue_def: SkillDefId::new(MBYONGJUE),
        target: PlayerId::NONE,
        skill,
    }
}

/// defend：可弃 he ≥ 2 时开强制弃牌 choice；弃后无进一步选择。
#[test]
fn qingyu_defend_discards_two_and_prevents() {
    let (mut tx, owner, _seats) = setup(2);
    let c1 = inject_hand(&mut tx, owner);
    let c2 = inject_hand(&mut tx, owner);
    let skill = skill_instance(&tx, owner);
    let err = run_effects(&mut tx, vec![qingyu_request(owner, 0, skill)]).err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开弃牌 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![c1.raw(), c2.raw()], false);
    assert_eq!(state.resolution.pending_choice_id, 0, "弃牌后直接结算");
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let in_discard = |card: CardId| {
        state
            .zones
            .get(discard)
            .is_some_and(|zone| zone.cards.contains(&card))
    };
    assert!(in_discard(c1) && in_discard(c2), "两张牌应被弃置");
}

/// defend：可弃 he < 2 时不开选择（引擎内权威校验）。
#[test]
fn qingyu_defend_noop_when_insufficient() {
    let (mut tx, owner, _seats) = setup(2);
    clear_hand(&mut tx, owner);
    inject_hand(&mut tx, owner);
    let skill = skill_instance(&tx, owner);
    run_effects(&mut tx, vec![qingyu_request(owner, 0, skill)]).unwrap();
    assert_eq!(
        tx.state().resolution.pending_choice_id,
        0,
        "不足两张不开选择"
    );
}

/// achieve：满血且手牌 < 轮数 → 使命成功，获得【选存】；唤醒后 defend 失效。
#[test]
fn qingyu_achieve_grants_xuancun_and_awakes() {
    let (mut tx, owner, _seats) = setup(2);
    // 准备阶段、满血、手牌为空（0 < 轮数 1）。
    tx.state_mut().turn.phase = Phase::Prepare;
    clear_hand(&mut tx, owner);
    {
        let p = tx.state_mut().players.get_mut(owner).unwrap();
        p.hp = p.max_hp as i32;
    }
    let skill = skill_instance(&tx, owner);
    run_effects(&mut tx, vec![qingyu_request(owner, 1, skill)]).unwrap();
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .any(|s| s.def.raw() == XUANCUN && s.owner == owner),
        "使命成功应获得选存"
    );
    // 唤醒后 defend 分支 no-op（即使有可弃牌也不开选择）。
    let c1 = inject_hand(&mut tx, owner);
    let c2 = inject_hand(&mut tx, owner);
    run_effects(&mut tx, vec![qingyu_request(owner, 0, skill)]).unwrap();
    assert_eq!(
        tx.state().resolution.pending_choice_id,
        0,
        "唤醒后 defend 不再触发"
    );
    let _ = (c1, c2);
}

/// fail：唯一其他角色时直接授予【勇决】并减上限。
#[test]
fn qingyu_fail_grants_mbyongjue() {
    let (mut tx, owner, seats) = setup(2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = skill_instance(&tx, owner);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    run_effects(&mut tx, vec![qingyu_request(owner, 2, skill)]).unwrap();
    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_before - 1,
        "使命失败应减 1 点体力上限"
    );
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .any(|s| s.def.raw() == MBYONGJUE && s.owner == other),
        "唯一其他角色应获得勇决"
    );
}

/// fail：多名其他角色时开选人 choice，选中后授予。
#[test]
fn qingyu_fail_chooses_target() {
    let (mut tx, owner, seats) = setup(3);
    let other = seats[1];
    let skill = skill_instance(&tx, owner);
    let err = run_effects(&mut tx, vec![qingyu_request(owner, 2, skill)]).err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开选人 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![other.raw()], false);
    assert_eq!(state.resolution.pending_choice_id, 0, "授技后直接结算");
    assert!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .any(|s| s.def.raw() == MBYONGJUE && s.owner == other),
        "所选角色应获得勇决"
    );
}
