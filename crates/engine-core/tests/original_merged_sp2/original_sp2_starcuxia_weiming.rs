//! Golden for SP2 粗遐 starcuxia / 未命 starweiming (ABI 750, tag 766):
//! 孪生锁定技——其他角色使用牌指定 owner 后：条件成立则其随机弃 1 手牌，
//! 条件仍成立则 owner 摸 1。真管道（CardUsed 事件派发）。

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STARCUXIA: u32 = 3_366_017_041;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
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
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    tag: u32,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(103_000 + tag),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn fire_card_used(state: &mut GameState, source: PlayerId, card: CardId, target: PlayerId) {
    let mut tx = Transaction::begin(state.clone());
    // dispatch 产出待执行效果，须 run_effects 落地（含机器选牌窗短路）。
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    *state = tx.commit().state;
}

fn fire_machine(state: &mut GameState, owner: PlayerId, source: PlayerId) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveStarcuxiaWeiming {
            player: owner,
            source,
            variant: 0,
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

#[test]
fn package_publishes_starcuxia_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STARCUXIA)
        .expect("starcuxia trigger");
    assert_eq!(entry.event.as_str(), "card_targeted");
    assert!(!entry.optional);
}

#[test]
fn starcuxia_forces_random_discard_when_source_dominant_and_damaged_owner() {
    let (mut state, players) = started(2_026_105_001);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, STARCUXIA);
    // source hp 压到高于 owner；source 伤害过 owner（damage_history）。
    {
        let s = state.players.get_mut(source).unwrap();
        s.hp = 4;
        let o = state.players.get_mut(owner).unwrap();
        o.hp = 2;
    }
    state.damage_history.insert((source, owner));
    let used = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 1);
    let spare = inject(&mut state, source, ZoneKind::Hand, CardKind::Shan, 2);
    let source_before = state.hand_count(source);
    let owner_before = state.hand_count(owner);

    // 机器直调：粗遐随机弃 1 + 条件复验摸 1。
    let _played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 3);
    fire_machine(&mut state, owner, source);

    // 注入 played 在 source_before 之后：机器时手牌 = before+1，弃 1 后 = before。
    assert_eq!(state.hand_count(source), source_before);
    assert!(
        !state.hand_cards(source).contains(&used) || !state.hand_cards(source).contains(&spare),
        "随机弃 1 后两张至少一张离开手牌"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_before + 1,
        "条件复验成立 owner 摸 1"
    );
    let _ = spare;
}

#[test]
fn starcuxia_real_pipeline_fires_through_card_targeted() {
    let (mut state, players) = started(2_026_105_003);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, STARCUXIA);
    {
        let s = state.players.get_mut(source).unwrap();
        s.hp = 4;
        let o = state.players.get_mut(owner).unwrap();
        o.hp = 2;
    }
    state.damage_history.insert((source, owner));
    let used = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 6);
    let spare = inject(&mut state, source, ZoneKind::Hand, CardKind::Shan, 7);
    let source_before = state.hand_count(source);
    let owner_before = state.hand_count(owner);

    // 真管道：CardUsed 事件 → CardTargeted 逐目标 ctx → 威临 filter eval
    // → 机器执行（随机弃 1 + 条件复验摸 1）。
    let played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 8);
    fire_card_used(&mut state, source, played, owner);

    assert_eq!(state.hand_count(source), source_before + 1 - 1, "随机弃 1");
    assert_eq!(state.hand_count(owner), owner_before + 1, "复验成立摸 1");
    let _ = (used, spare); // 被弃的是三张之一（随机），len 断言已覆盖。
}

#[test]
fn starcuxia_gated_when_neither_condition_holds() {
    let (mut state, players) = started(2_026_105_002);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, STARCUXIA);
    // source hp 低于 owner 且从未伤害过 owner → 不触发。
    {
        let s = state.players.get_mut(source).unwrap();
        s.hp = 1;
        let o = state.players.get_mut(owner).unwrap();
        o.hp = 3;
    }
    let _played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 4);
    let source_before = state.hand_count(source);
    let owner_before = state.hand_count(owner);
    let _played = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 5);

    fire_machine(&mut state, owner, source);
    assert_eq!(state.hand_count(source), source_before);
    assert_eq!(state.hand_count(owner), owner_before);
}
