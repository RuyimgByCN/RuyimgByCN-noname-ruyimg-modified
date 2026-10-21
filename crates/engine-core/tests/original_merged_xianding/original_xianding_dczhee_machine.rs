//! Golden for xianding dczhee (ABI 746, tag 763):
//! 锁定技——进入游戏/开局从牌堆、每名角色结束阶段从弃牌堆检索【闪电】
//! 挂为判定牌（判定区已有闪电则静默）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCZHEE: u32 = 2_349_242_461;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

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

fn inject_zone(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    tag: u32,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(100_000 + tag),
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

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
fn package_publishes_dczhee_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCZHEE)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .all(|trigger| trigger.filter.as_str() == "owner_alive" && !trigger.optional));
}

#[test]
fn searches_draw_and_discard_piles_for_lightning() {
    let (mut state, players) = started(2_026_102_001);
    let owner = players[0];
    grant(&mut state, owner, DCZHEE);
    // mode 0：牌堆注入闪电 → 挂判定区。
    let bolt_a = inject_zone(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickShandian,
        1,
    );
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDczhee {
            player: owner,
            mode: 0,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert!(tx.state().zones.get(judge).unwrap().cards.contains(&bolt_a));
    let mut state = tx.commit().state;

    // 判定区已有闪电 → 再触发静默（mode 1 弃牌堆有闪电也不动）。
    grant(&mut state, owner, DCZHEE);
    let _bolt_b = inject_zone(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::TrickShandian,
        2,
    );
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDczhee {
            player: owner,
            mode: 1,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 1);
    let mut state = tx.commit().state;

    // 无判定牌的新持有者：mode 1 从弃牌堆检索。
    grant(&mut state, players[1], DCZHEE);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDczhee {
            player: players[1],
            mode: 1,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, players[1]).unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 1);
}

#[test]
fn end_phase_pipeline_hangs_lightning_from_discard_pile() {
    let (mut state, players) = started(2_026_102_002);
    let owner = players[0];
    let other = players[1];
    grant(&mut state, owner, DCZHEE);
    let bolt = inject_zone(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::TrickShandian,
        3,
    );

    // 真管道：任意角色结束阶段（TurnStateChanged(End)）→ owner 挂判定。
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => return panic!("unexpected: {err:?}"),
    }
    let state = tx.commit().state;
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert!(state.zones.get(judge).unwrap().cards.contains(&bolt));
    let _ = submit; // 保留 submit 供后续扩展
}
