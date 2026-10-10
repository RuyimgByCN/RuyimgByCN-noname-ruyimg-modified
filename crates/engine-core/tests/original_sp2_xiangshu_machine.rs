//! Golden for SP2 襄戍 xiangshu (ABI 753, tag 769):
//! 限定技——结束阶段主动：令一名受伤角色回复 X 摸 X（X=min(5, 本回合
//! owner 造成伤害数)）。active 全管道 + 机器直调混合。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIANGSHU: u32 = 2_433_141_736;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
            def: CardDefId::new(106_000 + tag * 10 + index as u32),
            kind: CardKind::Other,
            suit: engine_api::Suit::Club,
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
fn package_publishes_xiangshu_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIANGSHU)
        .expect("xiangshu active entry");
    assert!(active.once_per_game);
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| targets.wounded && targets.min == 1 && targets.max == 1));
}

#[test]
fn xiangshu_recovers_and_draws_by_damage_dealt_then_latches() {
    let (mut state, players) = started(2_026_109_001);
    let owner = players[0];
    let wounded = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, XIANGSHU);
    // 受伤目标 hp 1（上限 3 → 最多回 2）；owner 本回合造成伤害 2 次 → X=2。
    {
        let w = state.players.get_mut(wounded).unwrap();
        w.hp = 1;
        w.max_hp = 3;
    }
    {
        let mut tx = Transaction::begin(state.clone());
        // 直写 damage_pair_counts（来源对 owner 的伤害计数 +2）。
        *tx.state_mut()
            .turn
            .damage_pair_counts
            .entry((owner, wounded))
            .or_default() += 2;
        state = tx.commit().state;
    }
    inject(&mut state, wounded, 4, 1);
    let hand_before = state.hand_count(wounded);

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![wounded],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 回复至满（X=2 足够回 2）+ 摸 2。
    assert_eq!(state.players.get(wounded).unwrap().hp, 3);
    assert_eq!(state.hand_count(wounded), hand_before + 2);

    // once_per_game：二次发动被拒。
    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![wounded],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
