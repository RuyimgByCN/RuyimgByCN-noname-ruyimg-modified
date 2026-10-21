//! Golden coverage for 双雄（shenhua 颜良文丑，ABI 511）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHUANGXIONG: u32 = 3_926_154_071;
const SHUANGXIONG_VIEW_AS: u32 = 1_761_435_131;
/// Stable mark id recorded by ABI 511 (count: 1 = red, 2 = black).
const SHUANGXIONG_COLOR_MARK: u32 = 6_0000_0511;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/shuangxiong.skill");

fn fire_phase(tx: Transaction, current: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_draw_judgment_contract() {
    let compiled = content_compiler::compile_source_named("shuangxiong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 511);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHUANGXIONG)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target_draw_not_replaced");
    assert!(trigger.optional);
    let view_as = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == SHUANGXIONG_VIEW_AS)
        .unwrap();
    assert_eq!(view_as.as_kind, "juedou");
    assert_eq!(view_as.filter, "color_ne_shuangxiong");
    // `from hand` is the default source zone and is normalized away.
    assert_eq!(view_as.source_zone, None);
}

#[test]
fn draw_phase_judges_gains_card_and_arms_opposite_color_view_as() {
    let package = content_compiler::compile_source_named("shuangxiong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHUANGXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;

    // The draw phase begins: the optional confirm opens.
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Draw);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, owner, vec![1], 1);

    // 双雄 replaced the draw: draw is fixed to zero and the judgment card was gained.
    assert!(state.turn.draw_phase_is_fixed(), "draw must be replaced");
    assert_eq!(
        state.hand_count(owner),
        1,
        "only the judgment card is in hand"
    );
    let judgment_card = state.hand_cards(owner)[0];
    let judgment_suit = state.cards.get(judgment_card).unwrap().suit;
    let expected_color = match judgment_suit {
        Suit::Spade | Suit::Club => 2,
        _ => 1,
    };
    let mark = state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == SHUANGXIONG_COLOR_MARK)
        .map(|(_, count)| *count)
        .unwrap_or(0);
    assert_eq!(
        mark, expected_color,
        "recorded color must match the judgment suit"
    );
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == SkillDefId::new(SHUANGXIONG_VIEW_AS))
            }),
        "the turn-long view-as skill must be armed"
    );
}

#[test]
fn second_draw_phase_does_not_rejudge_when_draw_is_fixed() {
    // Once the draw phase is fixed (e.g. by another skill), 双雄 must not trigger.
    let package = content_compiler::compile_source_named("shuangxiong-test2", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHUANGXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().turn.set_draw_phase_fixed_count(2);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Draw);
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "fixed draw must not ask for 双雄"
    );
    assert_eq!(state.hand_count(owner), 0, "no judgment card gained");
}
