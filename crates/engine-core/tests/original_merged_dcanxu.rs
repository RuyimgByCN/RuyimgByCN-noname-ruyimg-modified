//! O2 全量合并第二批（2026-10-14）：家族 dcanxu 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_dcanxu_active {
//! 第二百七十三批：dcanxu 暗蓄状态机行为测试（ABI 722 / tag 746 /
//! Cont 321）。链路：出牌阶段限一次，选两名手牌数不同的角色 → 手牌少者从
//! 多者处明选一张手牌 → 非黑桃所得 owner 摸 1 / 手牌数相等 owner 回 1。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCANXU: u32 = 3_012_521_444;
const DCANXU_SRC: &str =
    include_str!("../../../content/original/ported/refresh/dcanxu.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    high: PlayerId,
    low: PlayerId,
    skill: engine_api::SkillId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_300 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let compiled = content_compiler::compile_source_named("dcanxu-test", DCANXU_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_913_501, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, high, low) = (players[0], players[1], players[2]);
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCANXU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == DCANXU)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        high,
        low,
        skill,
    }
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state.hand_cards(player)
}

fn activate(setup: &Setup, targets: Vec<PlayerId>, key: u64) -> (GameState, PipelineResult) {
    submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets,
            card_targets: vec![],
        },
        key,
    )
}

#[test]
fn package_publishes_dcanxu_active_pair() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 723);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCANXU)
        .expect("dcanxu active entry");
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(targets.min, 2);
    assert_eq!(targets.max, 2);
    assert!(!targets.allow_self);
}

#[test]
fn spade_gain_equalizes_and_owner_recovers() {
    let mut setup = setup();
    // owner 受伤以便观察回复。
    {
        let state = setup.state.clone();
        let mut tx = Transaction::begin(state);
        let state_mut = tx.state_mut().players.get_mut(setup.owner).unwrap();
        state_mut.hp = i32::try_from(state_mut.max_hp - 1).unwrap();
        setup.state = tx.commit().state;
    }
    let mut tx = Transaction::begin(setup.state.clone());
    for rank in [5u8, 6, 7] {
        inject(&mut tx, setup.high, Suit::Spade, rank);
    }
    let heart = inject(&mut tx, setup.low, Suit::Heart, 5);
    setup.state = tx.commit().state;
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    assert_eq!(owner_hp, i32::try_from(setup.state.players.get(setup.owner).unwrap().max_hp - 1).unwrap());

    let (state, gain_choice) = activate(&setup, vec![setup.high, setup.low], 1);
    let gain_choice = gain_choice.choice.unwrap();
    assert_eq!(gain_choice.observer, setup.low, "fewer-hand player picks");
    assert_eq!(gain_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(gain_choice.options.len(), 3);

    // 选黑桃 → owner 不摸；手牌数相等（2==2）→ owner 回 1。
    let pick = hand_of(&state, setup.high)[0].raw();
    let (state, done) = submit(
        state,
        setup.low,
        CommandKind::ResolveChoice {
            choice_id: gain_choice.choice_id,
            selections: vec![pick],
            cancelled: false,
        },
        2,
    );
    assert!(
        matches!(done.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        done.result
    );
    assert_eq!(hand_of(&state, setup.high).len(), 2);
    assert!(hand_of(&state, setup.low).contains(&heart));
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        owner_hp + 1,
        "equalized hands recover owner"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn nonspade_gain_draws_owner_without_recovery() {
    let mut setup = setup();
    {
        let state = setup.state.clone();
        let mut tx = Transaction::begin(state);
        let state_mut = tx.state_mut().players.get_mut(setup.owner).unwrap();
        state_mut.hp = i32::try_from(state_mut.max_hp - 1).unwrap();
        setup.state = tx.commit().state;
    }
    let mut tx = Transaction::begin(setup.state.clone());
    for rank in [5u8, 6, 7, 8] {
        inject(&mut tx, setup.high, Suit::Club, rank);
    }
    let heart = inject(&mut tx, setup.low, Suit::Heart, 5);
    setup.state = tx.commit().state;
    let owner_hand_before = hand_of(&setup.state, setup.owner).len();

    let (state, gain_choice) = activate(&setup, vec![setup.high, setup.low], 1);
    let gain_choice = gain_choice.choice.unwrap();

    // 选非黑桃（梅花）→ owner 摸 1；手牌数 3 vs 2 不相等 → 不回复。
    let pick = hand_of(&state, setup.high)[0].raw();
    let (state, done) = submit(
        state,
        setup.low,
        CommandKind::ResolveChoice {
            choice_id: gain_choice.choice_id,
            selections: vec![pick],
            cancelled: false,
        },
        2,
    );
    assert!(
        matches!(done.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        done.result
    );
    assert_eq!(hand_of(&state, setup.low).len(), 2);
    assert!(hand_of(&state, setup.low).contains(&heart), "heart stays with low");
    assert_eq!(
        hand_of(&state, setup.owner).len(),
        owner_hand_before + 1,
        "non-spade gain draws owner one"
    );
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        i32::try_from(setup.state.players.get(setup.owner).unwrap().max_hp - 1).unwrap(),
        "hands not equal, no recovery"
    );
}

#[test]
fn equal_hand_counts_activation_is_rejected() {
    let mut setup = setup();
    let mut tx = Transaction::begin(setup.state.clone());
    for rank in [5u8, 6] {
        inject(&mut tx, setup.high, Suit::Spade, rank);
    }
    inject(&mut tx, setup.low, Suit::Heart, 5);
    inject(&mut tx, setup.low, Suit::Club, 9);
    setup.state = tx.commit().state;

    let (_, result) = activate(&setup, vec![setup.high, setup.low], 1);
    assert!(
        matches!(result.result, CommandResult::Rejected { .. }),
        "equal hand counts must be rejected: {:?}",
        result.result
    );
}
}
