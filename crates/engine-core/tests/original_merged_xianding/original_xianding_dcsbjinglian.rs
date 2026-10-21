//! Golden for XIANDING 新杀谋钟会「精练」refined threshold (lost ≥2 this turn).
//!
//! Drives the shipped `dcsbjinglian` skill via compile_source + real phase_end dispatch
//! and the Host `draw_cards` effect — not a re-implementation of the skill.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCSBJINGLIAN: u32 = 3_379_088_720;
const DC_SB_ZHONGHUI: u32 = 2_619_597_311;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/xianding/dcsbjinglian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn setup_with_skill() -> (Transaction, PlayerId) {
    let compiled = content_compiler::compile_source_named("dcsbjinglian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&compiled)
        .unwrap();
    assert!(
        state.content.tracks_turn_card_loss_history(),
        "filter must enable turn card-loss tracking"
    );
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_811, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCSBJINGLIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // Bootstrap/deals may record losses; zero the turn counter so tests own the threshold.
    // `card_lost_by_player` flag base is 1200 + seat (engine_core::state::turn::turn_flag).
    const CARD_LOST_BY_PLAYER_BASE: u32 = 1200;
    tx.state_mut()
        .turn
        .set_flag(CARD_LOST_BY_PLAYER_BASE + owner.raw(), 0);
    tx.state_mut().turn.lost_cards.clear();
    assert_eq!(tx.state().turn.card_loss_count(owner), 0);
    (tx, owner)
}

#[test]
fn package_publishes_refined_jinglian_filter_and_zhonghui_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|s| { s.skill_def_id == DCSBJINGLIAN && s.name == "dcsbjinglian" }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DCSBJINGLIAN
            && t.event == "phase_end"
            && t.filter == "owner_alive_phase_end_lost_cards_at_least_two"
            && t.optional
    }));
    let zhonghui = package
        .character_index
        .iter()
        .find(|c| c.id == DC_SB_ZHONGHUI)
        .expect("dc_sb_zhonghui");
    assert_eq!(zhonghui.name_key, "dc_sb_zhonghui");
    assert!(zhonghui.skills.contains(&DCSBJINGLIAN));
    // compiled source must agree with npack filter string
    let compiled = content_compiler::compile_source_named("dcsbjinglian", SOURCE).unwrap();
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == DCSBJINGLIAN
            && t.filter == "owner_alive_phase_end_lost_cards_at_least_two"
    }));
}

#[test]
fn draws_three_only_after_at_least_two_card_losses_this_turn() {
    let (mut tx, owner) = setup_with_skill();
    // one loss — should NOT open 精练
    let c1 = inject_hand(tx.state_mut(), owner, 301_001);
    tx.state_mut().turn.record_card_loss(owner, c1);
    assert_eq!(tx.state().turn.card_loss_count(owner), 1);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let one_loss = dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new());
    assert!(one_loss.is_ok(), "失牌仅 1 张时精练不得触发: {one_loss:?}");
    assert!(!tx.state().has_pending_choice());

    // second loss — should offer optional
    let c2 = inject_hand(tx.state_mut(), owner, 301_002);
    tx.state_mut().turn.record_card_loss(owner, c2);
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let offered = tx.commit();
    assert_eq!(
        offered.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (state, accepted) = resolve(offered.state, owner, 1, 42);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        hand_before + 3,
        "精练发动后应摸三张"
    );
}

#[test]
fn does_not_fire_with_zero_losses() {
    let (mut tx, owner) = setup_with_skill();
    assert_eq!(tx.state().turn.card_loss_count(owner), 0);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let hand_before = tx.state().hand_count(owner);
    let r = dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new());
    assert!(r.is_ok(), "{r:?}");
    assert_eq!(tx.state().hand_count(owner), hand_before);
}
