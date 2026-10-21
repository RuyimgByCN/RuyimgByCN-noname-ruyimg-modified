//! 第一百五十批：sp 程昱「贲育」（benyu）零新原语真实语义重写。
//! 上游：当你受到伤害后，你可选择：①将手牌摸至与伤害来源手牌数相同（至多摸至五张）；
//! ②弃置大于伤害来源手牌数的牌，然后对其造成 1 点伤害。
//! 近似口径：optional 确认 + 强制二选一（效果面条件等价）；分支②确定性弃牌。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BENYU: u32 = 3_440_506_159;
const BENYU_SRC: &str = include_str!("../../../../content/original/ported/sp/benyu.skill");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("benyu", BENYU_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(BENYU), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
        skill_def: SkillDefId::new(BENYU),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    (state, owner, other)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
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
        "resolve rejected: {:?}",
        result.result
    );
    state
}

/// 受伤并驱动贲育确认：返回确认后的状态（此时 choose(1,2) 分支选择已挂起）。
fn take_damage_and_confirm(
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    key: u64,
) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    ) {
        Ok(_) => panic!("expected the benyu optional confirm to open"),
        Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }
    let state = tx.commit().state;
    resolve(state, owner, vec![1], key)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn hand_of(state: &GameState, player: PlayerId) -> u32 {
    state.hand_count(player)
}

#[test]
fn benyu_draw_to_arm_tops_up_to_min_source_hand_and_five() {
    let (state, owner, other) = bootstrap();
    let mut tx = Transaction::begin(state);
    // 来源 6 张、owner 1 张：摸至 min(6,5)=5。
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: other,
                count: 6,
            },
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
        ],
    )
    .unwrap();
    let hp_before = hp_of(&tx.state(), owner);
    let state = tx.commit().state;

    let state = take_damage_and_confirm(state, owner, other, 10);
    let state = resolve(state, owner, vec![1], 11);

    assert_eq!(hp_of(&state, owner), hp_before - 1, "damage lands first");
    assert_eq!(hand_of(&state, owner), 5, "draw-to arm tops up to 5");
}

#[test]
fn benyu_discard_arm_discards_and_damages_source() {
    let (state, owner, other) = bootstrap();
    let mut tx = Transaction::begin(state);
    // 来源 0 张：分支②需弃 1 张；owner 3 张可弃。
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();
    let other_hp = hp_of(&tx.state(), other);
    let state = tx.commit().state;

    let state = take_damage_and_confirm(state, owner, other, 20);
    let state = resolve(state, owner, vec![2], 21);

    assert_eq!(hand_of(&state, owner), 2, "exactly need=0+1 card discarded");
    assert_eq!(hp_of(&state, other), other_hp - 1, "source takes 1 damage");
}
