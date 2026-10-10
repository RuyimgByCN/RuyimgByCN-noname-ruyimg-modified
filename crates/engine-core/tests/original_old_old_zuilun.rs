//! Golden coverage for 旧诸葛瞻「罪论」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLD_ZUILUN: u32 = 2_251_707_589;
const OLD_ZHUGEZHAN: u32 = 3_427_958_194;
const SOURCE: &str = include_str!("../../../content/original/ported/old/old_zuilun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    hand_card: CardId,
    equip_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("old-zuilun-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_703, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let target = tx.state().players.alive_by_seat()[1];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in [owner, target] {
        for kind in [ZoneKind::Hand, ZoneKind::Equip] {
            let zone = tx.state().zones.find(kind, player).unwrap();
            let cards = tx.state().zones.get(zone).unwrap().cards.clone();
            if !cards.is_empty() {
                effects.push(EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(zone),
                });
            }
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_ZUILUN),
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
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLD_ZUILUN)
        .unwrap();
    let hand_card = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 1);
    let equip_card = inject(&mut tx, target, ZoneKind::Equip, CardKind::Armor, 2);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
        hand_card,
        equip_card,
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(setup: &Setup, state: GameState, key: u64) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_rewritten_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLD_ZUILUN && skill.name == "old_zuilun"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OLD_ZHUGEZHAN)
        .unwrap();
    assert_eq!(character.skills, [OLD_ZUILUN, 3_976_662_486]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn gains_from_hand_then_only_equipment_and_target_draws_after_each_gain() {
    let setup = setup();
    let owner_hand_before = setup.state.hand_count(setup.owner);
    let target_hand_before = setup.state.hand_count(setup.target);
    let (state, activated) = activate(&setup, setup.state.clone(), 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw
    );
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == setup.hand_card.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == setup.equip_card.raw()));

    let (state, gained_hand) = resolve(state, setup.owner, setup.hand_card, 2);
    assert!(matches!(gained_hand.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), owner_hand_before + 1);
    assert_eq!(state.hand_count(setup.target), target_hand_before);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        1
    );

    let (state, activated) = activate(&setup, state, 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, setup.equip_card.raw());
    let (state, gained_equip) = resolve(state, setup.owner, setup.equip_card, 4);
    assert!(matches!(
        gained_equip.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        3
    );

    let (_, third) = activate(&setup, state, 5);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
}
