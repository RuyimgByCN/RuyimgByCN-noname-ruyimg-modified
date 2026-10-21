//! Golden for SP2 论道 nhlundao (ABI 752, tag 768, Cont 333):
//! 受伤后若来源存活且手牌数不同：手牌多则摸 1，否则弃来源一张（he 区，
//! owner 选）。真管道（裸 DealDamage 派发 Damage 触发）。

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NHLUNDAO: u32 = 3_109_389_590;
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
    count: usize,
    rank: u8,
    tag: u32,
) {
    for index in 0..count {
        let zone = state.zones.find(zone, owner).unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(105_000 + tag * 10 + index as u32),
            kind: CardKind::Other,
            suit: engine_api::Suit::Club,
            rank,
            weapon_range: 0,
            zone,
            owner,
            face_up: false,
        });
        state.zones.get_mut(zone).unwrap().cards.push(card);
    }
}

fn damage(
    state: &mut GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<(), EffectError> {
    let mut tx = Transaction::begin(state.clone());
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    *state = tx.commit().state;
    result.map(|_| ())
}

#[test]
fn package_publishes_nhlundao_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NHLUNDAO)
        .expect("nhlundao trigger");
    assert_eq!(entry.event.as_str(), "damage");
    assert_eq!(entry.filter.as_str(), "owner_is_target");
    assert!(!entry.optional);
}

#[test]
fn larger_hand_draws_one_after_damage() {
    let (mut state, players) = started(2_026_108_001);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, NHLUNDAO);
    // owner 手牌 3 > source 手牌 1 → 受伤后摸 1。
    inject(&mut state, owner, ZoneKind::Hand, 3, 7, 1);
    inject(&mut state, source, ZoneKind::Hand, 1, 7, 2);
    let owner_before = state.hand_count(owner);
    damage(&mut state, source, owner, 1).unwrap();
    assert_eq!(state.hand_count(owner), owner_before + 1);
}

#[test]
fn smaller_hand_discards_one_source_card() {
    let (mut state, players) = started(2_026_108_002);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, NHLUNDAO);
    // owner 手牌 0 < source 手牌 2 → 受伤后 owner 弃 source 一张（开选牌窗）。
    inject(&mut state, source, ZoneKind::Hand, 2, 7, 3);
    let source_before = state.hand_count(source);
    let res = damage(&mut state, source, owner, 1);
    assert!(matches!(res, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveNhlundaoStage
    );
    // 选 source 第一张手牌弃置。
    let hand = state.zones.find(ZoneKind::Hand, source).unwrap();
    let pick = state.zones.get(hand).unwrap().cards[0];
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        state,
        engine_api::Command {
            actor: owner,
            expected_revision: engine_api::ExpectedRevision(0),
            idempotency_key: engine_api::IdempotencyKey(7),
            kind: engine_api::CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        result.result,
        engine_api::CommandResult::Accepted { .. }
    ));
    assert_eq!(state.hand_count(source), source_before - 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn equal_hands_are_quiet() {
    let (mut state, players) = started(2_026_108_003);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, NHLUNDAO);
    inject(&mut state, owner, ZoneKind::Hand, 2, 7, 4);
    inject(&mut state, source, ZoneKind::Hand, 2, 7, 5);
    let owner_before = state.hand_count(owner);
    damage(&mut state, source, owner, 1).unwrap();
    assert_eq!(state.hand_count(owner), owner_before);
}
