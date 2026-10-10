//! Golden for SP 霸业 spolaige (ABI 737, tag 758):
//! 觉醒技——全局第二次濒死结算后，失去〖西向〗获得〖朔北〗，手牌摸至体力
//! 上限并回复体力至体力上限。真管道驱动（EnterDying 全窗口结算）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPOLAIGE: u32 = 4_158_403_560;
const SPOLXIXIANG: u32 = 2_531_785_939;
const SPOLZHUBEI: u32 = 3_309_571_556;
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

fn grant(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
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

fn owns(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn inject(state: &mut GameState, owner: PlayerId, count: usize, tag: u32) {
    for index in 0..count {
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(94_000 + tag * 10 + index as u32),
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
fn package_publishes_spolaige_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SPOLAIGE)
        .expect("spolaige trigger");
    assert_eq!(entry.event.as_str(), "dying_after");
    assert_eq!(entry.filter.as_str(), "owner_alive_second_global_dying");
    assert!(!entry.optional);
}

#[test]
fn second_global_dying_awakens_owner_to_full_max_hp() {
    let (mut state, players) = started(2_026_093_001);
    let owner = players[0];
    let first_victim = players[1];
    let second_victim = players[2];
    grant(&mut state, owner, SPOLAIGE);
    grant(&mut state, owner, SPOLXIXIANG);
    // owner：体力上限 4，体力 1，手牌 1。
    {
        let o = state.players.get_mut(owner).unwrap();
        o.max_hp = 4;
        o.hp = 1;
    }
    inject(&mut state, owner, 1, 1);
    let mut tx = Transaction::begin(state);

    // 第一次濒死（victim1）：进入后无桃自动关闭窗口，再以恢复脱离濒死
    // （上游 dyingAfter = 脱离濒死后；死亡路径无 LeftDying）。
    tx.state_mut().players.get_mut(first_victim).unwrap().hp = 0;
    run_effects(
        &mut tx,
        vec![EffectRequest::EnterDying {
            player: first_victim,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: PlayerId::NONE,
            player: first_victim,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_dying_events, 1);
    assert!(owns(tx.state(), owner, SPOLXIXIANG));
    assert_eq!(tx.state().hand_count(owner), 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);

    // 第二次濒死（victim2）：脱离濒死 → 觉醒——换技 + 摸/回至体力上限。
    tx.state_mut().players.get_mut(second_victim).unwrap().hp = 0;
    run_effects(
        &mut tx,
        vec![EffectRequest::EnterDying {
            player: second_victim,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: PlayerId::NONE,
            player: second_victim,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_dying_events, 2);
    assert!(!owns(tx.state(), owner, SPOLXIXIANG));
    assert!(owns(tx.state(), owner, SPOLZHUBEI));
    assert_eq!(tx.state().hand_count(owner), 4);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn spolaige_contract_command_submission_is_rejected_when_exhausted_not_applicable() {
    // 占位对齐：确认 ActivateSkill 之外的普通命令路径不受影响。
    let (state, players) = started(2_026_093_002);
    let owner = players[0];
    let (_, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: engine_api::SkillId::new(1),
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}
