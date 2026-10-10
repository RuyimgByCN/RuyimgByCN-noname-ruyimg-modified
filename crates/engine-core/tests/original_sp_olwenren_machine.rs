//! Golden for SP 温仁 olwenren (ABI 736, tag 757):
//! 出牌阶段限一次，任选任意名目标，每名目标无手牌摸 1、手牌数不大于
//! owner 再摸 1（可叠加，可摸 0）。active 全管道驱动（ActivateSkill）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLWENREN: u32 = 4_257_407_239;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, count: usize, tag: u32) {
    for index in 0..count {
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(93_000 + tag * 10 + index as u32),
            kind: CardKind::Other,
            suit: Suit::Spade,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        state.zones.get_mut(hand).unwrap().cards.push(card);
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_olwenren_active_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLWENREN)
        .expect("olwenren active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().is_some_and(|targets| {
        targets.allow_self && targets.min == 1 && targets.max == 255
    }));
}

#[test]
fn olwenren_draws_two_for_empty_one_for_lequal_and_zero_for_greater() {
    let (mut state, players) = started(2_026_092_001);
    let owner = players[0];
    let empty_target = players[1];
    let rich_target = players[2];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLWENREN);
    inject(&mut state, owner, 2, 1);
    // empty_target 无手牌（摸 2）；rich_target 3 张 > owner 2 张（摸 0）。
    inject(&mut state, rich_target, 3, 2);

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![empty_target, rich_target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(empty_target), 2);
    assert_eq!(state.hand_count(rich_target), 3);
    assert_eq!(state.hand_count(owner), 2);

    // uses_per_turn 1：本回合再次发动被权威拒绝。
    let (state, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![empty_target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(state.hand_count(empty_target), 2);
}

#[test]
fn olwenren_gives_one_draw_to_target_with_lequal_hand_count() {
    let (mut state, players) = started(2_026_092_002);
    let owner = players[0];
    let lequal_target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLWENREN);
    inject(&mut state, owner, 3, 3);
    // 1 张 ≤ owner 3 张且有手牌（非空）→ 仅满足第二条件 → 摸 1。
    inject(&mut state, lequal_target, 1, 4);

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![lequal_target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(lequal_target), 2);
}
