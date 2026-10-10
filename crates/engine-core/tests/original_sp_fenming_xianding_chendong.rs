//! Golden for SP「奋命」and the XIANDING 陈董 cross-package character closure.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENMING: u32 = 4_273_591_874;
const DCDUANXIE: u32 = 4_278_513_633;
const CHENDONG: u32 = 2_934_854_435;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/fenming.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const XIANDING_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    linked: PlayerId,
    empty_linked: PlayerId,
    unlinked: PlayerId,
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, def: u32) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: if zone == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Other
        },
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone: zone_id,
        owner,
        face_up: zone != ZoneKind::Hand,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn setup(owner_linked: bool) -> Setup {
    let package = content_compiler::compile_source_named("fenming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_502, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(FENMING),
        },
        EffectRequest::AddMark {
            player: players[1],
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        },
        EffectRequest::AddMark {
            player: players[2],
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        },
        EffectRequest::SetTurnState {
            current: players[0],
            phase: Phase::Discard.to_u8(),
        },
    ]);
    if owner_linked {
        effects.push(EffectRequest::AddMark {
            player: players[0],
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        });
    }
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner: players[0],
        linked: players[1],
        empty_linked: players[2],
        unlinked: players[3],
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_fenming_and_complete_chendong_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENMING));
    assert_eq!(
        content_compiler::compile_source_named("fenming-test", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        189
    );

    let xianding = content_schema::ContentPackage::from_json_bytes(XIANDING_PACKAGE).unwrap();
    assert!(xianding.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    assert_eq!(
        xianding
            .character_index
            .iter()
            .find(|character| character.id == CHENDONG)
            .unwrap()
            .skills,
        [DCDUANXIE, FENMING]
    );

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&xianding).unwrap();
}

#[test]
fn linked_owner_discards_one_card_from_each_eligible_linked_player_in_seat_order() {
    let mut setup = setup(true);
    let owner_card = inject(&mut setup.state, setup.owner, ZoneKind::Hand, 189_001);
    let target_card = inject(&mut setup.state, setup.linked, ZoneKind::Equip, 189_002);
    let unlinked_card = inject(&mut setup.state, setup.unlinked, ZoneKind::Hand, 189_003);

    let (state, offered) = submit(setup.state, setup.owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, owner_choice) = resolve(state, setup.owner, vec![1], 2);
    let choice = owner_choice.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [owner_card.raw()]
    );

    let (state, target_choice) = resolve(state, setup.owner, vec![owner_card.raw()], 3);
    let choice = target_choice.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_card.raw()]
    );

    let (state, discarded) = resolve(state, setup.owner, vec![target_card.raw()], 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(owner_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(target_card).unwrap().zone, discard);
    assert_eq!(
        state.cards.get(unlinked_card).unwrap().owner,
        setup.unlinked
    );
    assert!(state.hand_cards(setup.empty_linked).is_empty());
}

#[test]
fn unlinked_owner_is_not_offered_fenming() {
    let mut setup = setup(false);
    let target_card = inject(&mut setup.state, setup.linked, ZoneKind::Hand, 189_010);
    let (state, result) = submit(setup.state, setup.owner, CommandKind::EndPhase, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(target_card).unwrap().owner, setup.linked);
}
