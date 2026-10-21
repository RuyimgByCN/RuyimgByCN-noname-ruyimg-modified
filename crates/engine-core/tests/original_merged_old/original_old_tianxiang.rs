//! Golden coverage for 天香（old/shenhua 小乔）伤害重定向（ABI 456）.
//!
//! 旧小乔受到伤害时：可选确认 → 弃一张红桃手牌 → 选择其他角色 → 原伤害取消并以
//! 同源/同量/同属性/同牌重放给新目标；新目标获得「天香·摸牌」并在伤害结算后按
//! 已损失体力值摸牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TIANXIANG: u32 = 3_182_807_327;
const TIANXIANG2: u32 = 4_111_110_111;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/tianxiang.skill");

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
    if !matches!(result.result, CommandResult::Accepted { .. }) {
        panic!("choice rejected: {:?}", result);
    }
    next
}

#[test]
fn redirects_damage_to_chosen_target_and_grants_draw_skill() {
    let package = content_compiler::compile_source_named("tianxiang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 456);
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
        skill_def: SkillDefId::new(TIANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();

    // Give the owner a discardable heart hand card.
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
        owner: owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(heart);
    assert!(tx.state().hand_count(owner) >= 1);

    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let victim_hand = tx.state().hand_count(victim);

    // Fire damage 2 to the owner: optional confirm → heart card → redirect target.
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let state = drive_choice(state, owner, 1); // optional confirm
    let state = drive_choice(state, owner, heart.raw()); // heart card cost
    let state = drive_choice(state, owner, victim.raw()); // redirect target

    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp,
        "owner damage redirected"
    );
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp - 2,
        "victim takes redirected damage"
    );
    assert_eq!(
        state.hand_count(victim),
        victim_hand + 2,
        "victim draws exactly lost-HP cards after the damage"
    );
    assert!(
        !state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == victim && skill.def.raw() == TIANXIANG2),
        "tianxiang2 fires once and removes itself"
    );
    assert!(!state.has_pending_choice());
}
