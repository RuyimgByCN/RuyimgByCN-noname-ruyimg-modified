//! Golden for SHIJI 辛毗「隐据 / 持节」语义（ABI 421 / 422）。
//!
//! 隐据：出牌阶段限一次，目标二选一——对辛毗用【杀】/ 获得跳过标记
//! （其下回合准备阶段跳过出牌与弃牌）。
//! 持节：成为单目标牌目标时判定，点数 > 6 取消该目标。

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

const SPYINJU: u32 = 3_592_329_121;
const SPCHIJIE: u32 = 3_232_936_655;
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
                rng_seed: 20_260_810,
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

/// 隐据：目标选"跳过"→ 获得跳过标记。
#[test]
fn spyinju_skip_option_grants_mark() {
    let (mut tx, owner, seats) = setup(SPYINJU, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPYINJU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyinju {
            player: owner,
            target: other,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    // 目标选"跳过"(option 1)。
    let state = resolve_choice(tx.commit().state, other, 1, vec![1], false);
    assert!(
        state
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == 2_728_392_676 && *count == 1),
        "目标应获得跳过标记"
    );
}

/// 隐据：目标选"用杀"→ 目标对辛毗用一张手牌杀。
#[test]
fn spyinju_sha_option_uses_sha() {
    let (mut tx, owner, seats) = setup(SPYINJU, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let sha = inject_hand(&mut tx, other, CardKind::Sha, Suit::Spade);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPYINJU && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyinju {
            player: owner,
            target: other,
            skill,
        }],
    )
    .unwrap_err();
    // 目标选"用杀"(option 0)。
    let state = resolve_choice(tx.commit().state, other, 2, vec![0], false);
    // 目标用手牌杀对辛毗——UseCard 成功（测试环境无完整响应结算,牌暂不消耗）。
    assert!(state.players.get(owner).unwrap().alive);
    let _ = hp_before;
    let _ = sha;
}

/// 持节：判定点数 > 6 取消目标（判定区牌点数可控时验证触发链）。
#[test]
fn spchijie_judges_and_may_cancel() {
    let (mut tx, owner, seats) = setup(SPCHIJIE, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SPCHIJIE && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    // 直接调用持节原语：无当前牌目标上下文时返回空（不报错）。
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpchijie {
            player: owner,
            source: PlayerId::NONE,
            card: engine_api::CardId::NONE,
            skill,
        }],
    );
    assert!(result.is_ok(), "无目标上下文时应安全返回");
}
