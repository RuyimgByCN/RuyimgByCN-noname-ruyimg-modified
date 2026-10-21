//! Golden coverage for 界天香（shenhua 小乔）取消伤害 + 分支结算（ABI 457）.
//!
//! 小乔受到伤害时：可选确认 → 弃红桃手牌 → 选其他角色 → 拥有者二选一：
//! 0 = 目标受伤害来源 1 点伤害后摸 min(5, 已损失体力) 张；1 = 目标失去 1 点体力
//! 后获得弃置的红桃牌。原伤害被取消。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RETIANXIANG: u32 = 3_203_696_812;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/retianxiang.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn prepared(branch: u8) -> (Transaction, PlayerId, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("retianxiang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 457);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, victim, source) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(RETIANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .expect("owner hand zone");
    let heart = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(heart);
    let _ = branch;
    (tx, owner, victim, source, heart)
}

fn run_branch_full(
    mut tx: Transaction,
    owner: PlayerId,
    victim: PlayerId,
    heart: CardId,
    branch: u32,
) -> (engine_core::state::GameState, i32) {
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let mut state = tx.commit().state;
    state = drive_choice(state, owner, 1); // optional confirm
    state = drive_choice(state, owner, heart.raw()); // heart card cost
    state = drive_choice(state, owner, victim.raw()); // branch target
    state = drive_choice(state, owner, branch); // branch
    (state, owner_hp_before)
}

#[test]
fn branch_zero_damages_from_source_then_draws_lost_hp_capped_five() {
    let (tx, owner, victim, _source, heart) = prepared(0);
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let victim_max = tx.state().players.get(victim).unwrap().max_hp;
    let victim_hand = tx.state().hand_count(victim);
    let (state, owner_hp) = run_branch_full(tx, owner, victim, heart, 0);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp,
        "original damage cancelled"
    );
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp.saturating_sub(1),
        "victim takes 1 damage from the source"
    );
    let lost = ((victim_max as i32 - (victim_hp - 1)).max(0) as u32).min(5);
    assert_eq!(
        state.hand_count(victim),
        victim_hand + lost,
        "victim draws min(5, lost HP)"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn branch_one_loses_hp_and_gains_discarded_card() {
    let (tx, owner, victim, _source, heart) = prepared(1);
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let victim_hand = tx.state().hand_count(victim);
    let (state, owner_hp) = run_branch_full(tx, owner, victim, heart, 1);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp,
        "original damage cancelled"
    );
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp.saturating_sub(1),
        "victim loses 1 HP"
    );
    assert!(
        state.hand_cards(victim).contains(&heart),
        "victim gains the discarded heart card"
    );
    assert_eq!(state.hand_count(victim), victim_hand + 1);
    assert!(!state.has_pending_choice());
}
