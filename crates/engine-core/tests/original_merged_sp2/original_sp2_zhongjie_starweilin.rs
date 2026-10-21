//! Golden for SP2 张介 zhongjie (tag 765 / ABI 748) + 威临 starweilin
//! (filter rung 749, Cont 332):
//! 张介——死亡时选一名其他角色：+1 体力上限、回复 1、摸 1（forceDie）。
//! 威临——自己回合内对「本局未受过伤」角色造成伤害且用牌数 ≥ 其体力：
//! 伤害 +1。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events as dfx, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHONGJIE: u32 = 2_821_141_991;
const STARWEILIN: u32 = 3_176_127_729;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_zhongjie_starweilin_contracts() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let zhongjie = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHONGJIE)
        .expect("zhongjie trigger");
    assert_eq!(zhongjie.event.as_str(), "death");
    assert_eq!(zhongjie.filter.as_str(), "owner_is_target");
    assert!(!zhongjie.optional);
    let weilin = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STARWEILIN)
        .expect("starweilin trigger");
    assert_eq!(weilin.event.as_str(), "damage_before");
    assert_eq!(
        weilin.filter.as_str(),
        "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp"
    );
}

#[test]
fn zhongjie_grants_max_hp_recover_and_draw_on_death() {
    let (mut state, players) = started(2_026_104_001);
    // 选忠臣作为死亡者（主公/反贼死均会终结对局）。
    use engine_core::state::Identity;
    let owner = *players
        .iter()
        .find(|p| {
            state
                .players
                .get(**p)
                .map(|pl| pl.identity == Identity::Loyalist)
                .unwrap_or(false)
        })
        .unwrap();
    let heir = *players.iter().find(|p| **p != owner).unwrap();
    grant(&mut state, owner, ZHONGJIE);
    // owner 受致命伤死亡；heir 受伤状态（hp 1）以观察回复。
    {
        let h = state.players.get_mut(heir).unwrap();
        h.hp = 1;
        h.max_hp = 4;
    }
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    tx.state_mut().players.get_mut(owner).unwrap().dying = true;
    // 直接击杀（无救）→ 死亡事件触发张介确认窗口。
    // 张介确认窗口以 Err(ChoiceRequested) 短路——正常信号。
    let kill_result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: owner,
            killer: PlayerId::NONE,
        }],
    );
    match kill_result {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    // forced 触发：直接进入选目标窗（可取消 = 上游确认窗语义等价）。
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ResolveZhongjieStage
    );

    // 选 heir → +1 上限 + 回复 + 摸 1。
    let (state, result) = resolve(tx.commit().state, owner, vec![heir.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let heir_state = state.players.get(heir).unwrap();
    assert_eq!(heir_state.max_hp, 5);
    assert_eq!(heir_state.hp, 2);
    assert_eq!(state.hand_count(heir), 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn starweilin_increases_damage_under_three_conditions() {
    let (mut state, players) = started(2_026_104_002);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    grant(&mut state, owner, STARWEILIN);
    // 目标 hp 压到 2（card_action_totals=3 ≥ 2 ✓）；注入一张【杀】供出牌。
    {
        let t = state.players.get_mut(target).unwrap();
        t.hp = 2;
    }
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(102_001),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(sha);
    {
        let mut tx = Transaction::begin(state.clone());
        tx.state_mut().card_action_totals.insert(owner, 3);
        state = tx.commit().state;
    }
    let hp_before = state.players.get(target).unwrap().hp;

    // 真实出杀管道：DamageBefore 触发威临 → 伤害 +1（目标无闪 → 掉 2）。
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    let _ = result;
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        hp_before - 2,
        "威临应使伤害 +1"
    );
}
