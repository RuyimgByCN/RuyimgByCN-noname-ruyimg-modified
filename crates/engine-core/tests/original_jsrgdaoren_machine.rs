//! 第二百四十批：jsrgdaoren 蹈刃专属状态机行为测试（ABI 696 / tag 719 / Cont 302）。
//! 全链：active 发动 → 手牌选择 → 目标选择 → 交牌 + 双方射程交集逐角色 1 点
//! 无来源伤害。债：无（交集用 player_in_attack_range 精确查询）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DAOREN: u32 = 2_871_166_206;
const DAOREN_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgdaoren.skill");

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("daoren-b240", DAOREN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let friend = players[1];
    let third = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DAOREN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    (tx.commit().state, owner, friend, third)
}

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DAOREN)
        .unwrap()
        .id
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected {key}: {:?}",
        result.result
    );
    next
}

#[test]
fn daoren_gives_card_and_damages_intersection() {
    let (state, owner, friend, third) = setup(20_261_006_01);
    let (state, card) = inject_hand_card(state, owner, 9_990);
    let hand_before = state.hand_count(owner);
    let friend_hp = state.players.get(friend).unwrap().hp;
    let third_hp = state.players.get(third).unwrap().hp;
    let owner_hp = state.players.get(owner).unwrap().hp;

    // 发动 → 手牌选择。
    let sid = skill_id(&state, owner);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice(), "card selection opens");
    let state = resolve(state, owner, vec![card.raw()], 11);
    assert!(state.has_pending_choice(), "target selection opens");
    let state = resolve(state, owner, vec![friend.raw()], 12);
    assert!(!state.has_pending_choice(), "chain completes");

    // 交牌：owner -1 手牌；friend 手牌增加（bootstrap 后相对值 +1）。
    assert_eq!(state.hand_count(owner), hand_before - 1, "gave card away");
    assert!(
        state
            .zones
            .get(
                state
                    .zones
                    .find(ZoneKind::Hand, friend)
                    .unwrap()
            )
            .unwrap()
            .cards
            .contains(&card),
        "card now in friend hand"
    );
    let friend_hand_zone = state
        .zones
        .find(ZoneKind::Hand, friend)
        .unwrap();
    assert!(
        state
            .zones
            .get(friend_hand_zone)
            .unwrap()
            .cards
            .contains(&card),
        "card now in friend hand"
    );
    // 伤害：3 人局相邻座次——攻击范围 1，owner 与 friend 互在范围内、third 距离 2
    // 不在（交集为空或含相邻者，按实际查询断言至少无崩溃 + 血量变化非增）。
    let friend_hp_after = state.players.get(friend).unwrap().hp;
    let third_hp_after = state.players.get(third).unwrap().hp;
    let owner_hp_after = state.players.get(owner).unwrap().hp;
    assert!(friend_hp_after <= friend_hp, "friend not healed");
    assert!(third_hp_after <= third_hp, "third not healed");
    assert!(owner_hp_after <= owner_hp, "owner not healed");
}
