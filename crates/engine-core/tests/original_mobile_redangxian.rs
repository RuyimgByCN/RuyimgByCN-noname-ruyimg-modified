//! Golden coverage for 手杀界廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REDANGXIAN: u32 = 4_137_190_076;
const REFULI: u32 = 2_837_551_482;
const XIN_LIAOHUA: u32 = 4_141_359_871;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/redangxian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
    draw_sha: CardId,
    discard_sha: CardId,
}

fn inject_shared_card(state: &mut GameState, zone_kind: ZoneKind, def: u32, suit: Suit) -> CardId {
    let zone = state.zones.find(zone_kind, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, REDANGXIAN as u64, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();

    let seats = tx.state().players.alive_by_seat();
    let previous = tx.state().turn.current;
    let previous_index = seats.iter().position(|player| *player == previous).unwrap();
    let owner = seats[(previous_index + 1) % seats.len()];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in seats {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REDANGXIAN),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let draw_sha = inject_shared_card(tx.state_mut(), ZoneKind::DrawPile, 94_101, Suit::Spade);
    let discard_sha =
        inject_shared_card(tx.state_mut(), ZoneKind::DiscardPile, 94_102, Suit::Heart);
    Setup {
        state: tx.commit().state,
        owner,
        previous,
        draw_sha,
        discard_sha,
    }
}

fn end_phase(state: GameState, actor: PlayerId) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("redangxian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 74);
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == REDANGXIAN && skill.name == "redangxian" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [REDANGXIAN, REFULI]);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn turn_begin_gains_sha_from_discard_only_then_opens_extra_play_phase() {
    let setup = setup();
    let (state, result) = end_phase(setup.state, setup.previous);

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert!(state.hand_cards(setup.owner).contains(&setup.discard_sha));
    assert!(!state.hand_cards(setup.owner).contains(&setup.draw_sha));

    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .contains(&setup.draw_sha));
    assert!(!state
        .zones
        .get(discard)
        .unwrap()
        .cards
        .contains(&setup.discard_sha));
}
