//! Golden coverage for 旧关兴张苞「父魂」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLD_FUHUN: u32 = 4_126_064_689;
const OLD_GUANZHANG: u32 = 3_002_051_488;
const WUSHENG: u32 = 16;
const PAOXIAO: u32 = 23;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/old_fuhun.skill");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    top: [CardId; 2],
}

fn setup(first: Suit, second: Suit, permanent_wusheng: bool) -> Setup {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let fuhun = content_compiler::compile_source_named("old-fuhun-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&standard)
        .unwrap();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&fuhun)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_702, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(OLD_FUHUN),
    });
    if permanent_wusheng {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WUSHENG),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, 0);

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let top = [cards[cards.len() - 2], cards[cards.len() - 1]];
    tx.state_mut().cards.get_mut(top[0]).unwrap().suit = first;
    tx.state_mut().cards.get_mut(top[1]).unwrap().suit = second;
    Setup { tx, owner, top }
}

fn fire_draw(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let event = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::Draw,
    };
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new())?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == def)
}

#[test]
fn packages_publish_complete_character_and_skill_without_legacy_js() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert!(yijiang.manifest.rule_vm_abi_version >= 287);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLD_FUHUN && skill.name == "old_fuhun"));
    assert!(yijiang
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "standard"));

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_GUANZHANG)
        .unwrap();
    assert_eq!(character.skills, [OLD_FUHUN]);
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    assert!(!serde_json::to_string(&(yijiang, old))
        .unwrap()
        .contains(".js"));
}

#[test]
fn replaced_draw_suppresses_prompt_and_decline_preserves_normal_draw() {
    let Setup { mut tx, owner, top } = setup(Suit::Heart, Suit::Spade, false);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, 1);
    fire_draw(&mut tx, owner).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert!(top
        .iter()
        .all(|card| tx.state().cards.get(*card).unwrap().zone
            == tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .unwrap()));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::DRAW_PHASE_DRAW_ZEROED, 0);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = tx.commit().state;
    let (state, result) = resolve(state, owner, Vec::new(), true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED), 0);
    assert!(top.iter().all(|card| {
        state.cards.get(*card).unwrap().zone
            == state
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .unwrap()
    }));
}

#[test]
fn same_color_cards_are_publicly_revealed_and_gained_without_skills() {
    let Setup { mut tx, owner, top } = setup(Suit::Heart, Suit::Diamond, false);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, result) = resolve(tx.commit().state, owner, vec![1], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED), 1);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert!(top
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == hand));
    assert!(result.events.iter().any(
        |event| matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.len() == 2 && top.iter().all(|card| cards.contains(card)) && to.is_empty())
    ));
    assert!(!has_skill(&state, owner, WUSHENG));
    assert!(!has_skill(&state, owner, PAOXIAO));
}

#[test]
fn different_colors_grant_both_skills_after_gain_and_expire_at_turn_end() {
    let Setup { mut tx, owner, top } = setup(Suit::Heart, Suit::Spade, false);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (mut state, result) = resolve(tx.commit().state, owner, vec![1], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, WUSHENG));
    assert!(has_skill(&state, owner, PAOXIAO));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let gain_index = result
        .events
        .iter()
        .position(|event| {
            matches!(event, DomainEvent::CardsMoved { cards, to, .. } if *to == hand && top.iter().all(|card| cards.contains(card)))
        })
        .unwrap();
    let skill_index = result
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::SkillAdded { player, .. } if *player == owner),
        )
        .unwrap();
    assert!(gain_index < skill_index);

    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, ended) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, owner, WUSHENG));
    assert!(!has_skill(&state, owner, PAOXIAO));
}

#[test]
fn permanent_wusheng_is_not_made_temporary() {
    let Setup {
        mut tx,
        owner,
        top: _,
    } = setup(Suit::Diamond, Suit::Club, true);
    assert_eq!(fire_draw(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (mut state, result) = resolve(tx.commit().state, owner, vec![1], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, WUSHENG));
    assert!(has_skill(&state, owner, PAOXIAO));
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, ended) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(6),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, WUSHENG));
    assert!(!has_skill(&state, owner, PAOXIAO));
}
