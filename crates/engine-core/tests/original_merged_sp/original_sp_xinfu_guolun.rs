//! Golden coverage for SP 庞统「过论」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::project_observer;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_GUOLUN: u32 = 2_723_622_416;
const XINFU_SONGSANG: u32 = 2_272_181_771;
const RE_JSP_PANGTONG: u32 = 2_348_829_214;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    owner_hand: CardId,
    target_cards: Vec<CardId>,
}

fn inject(state: &mut GameState, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(294_000 + u32::from(rank)),
        kind: if zone_kind == ZoneKind::Equip {
            CardKind::Weapon
        } else {
            CardKind::Other
        },
        suit: Suit::Club,
        rank,
        weapon_range: u8::from(zone_kind == ZoneKind::Equip) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(owner_rank: u8, target_ranks: &[u8]) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_073_001, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(XINFU_GUOLUN),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == XINFU_GUOLUN)
        .unwrap();
    let mut state = tx.commit().state;
    let owner_hand = inject(&mut state, owner, ZoneKind::Hand, owner_rank);
    let target_cards = target_ranks
        .iter()
        .map(|rank| inject(&mut state, target, ZoneKind::Hand, *rank))
        .collect();
    Setup {
        state,
        owner,
        target,
        skill,
        owner_hand,
        target_cards,
    }
}

fn command(
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

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn knows_face(state: &GameState, observer: PlayerId, card: CardId) -> bool {
    project_observer(state, ObserverKind::Player(observer))
        .known_cards
        .iter()
        .any(|known| known.id == card && known.def.is_some())
}

#[test]
fn current_sp_package_keeps_complete_re_jsp_pangtong() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(package.character_index.len(), 217);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XINFU_GUOLUN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.hand_nonempty);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_JSP_PANGTONG)
        .unwrap();
    assert_eq!(character.name_key, "re_jsp_pangtong");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [XINFU_GUOLUN, XINFU_SONGSANG]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn target_hand_is_hidden_then_only_selected_face_is_projected_and_cancel_consumes_use() {
    let setup = setup(9, &[2, 11]);
    let owner = setup.owner;
    let target = setup.target;
    let skill = setup.skill;
    let owner_hand = setup.owner_hand;
    let selected = setup.target_cards[0];
    let hidden = setup.target_cards[1];
    let owner_count = setup.state.hand_count(owner);
    let target_count = setup.state.hand_count(target);
    assert!(!knows_face(&setup.state, owner, selected));
    assert!(!knows_face(&setup.state, owner, hidden));

    let (state, offered) = activate(setup, 1);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let first = offered.choice.unwrap();
    assert_eq!(first.observer, owner);
    assert_eq!(first.kind, ChoiceKind::SelectCards);
    assert!(!first.allow_cancel);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveXinfuGuolun
    );
    assert!(!knows_face(&state, owner, selected));
    assert!(!knows_face(&state, owner, hidden));

    let (state, revealed) = resolve(state, owner, vec![selected.raw()], false, 2);
    assert!(matches!(revealed.result, CommandResult::Accepted { .. }));
    let second = revealed.choice.unwrap();
    assert!(second.allow_cancel);
    assert!(second
        .options
        .iter()
        .any(|option| option.token == owner_hand.raw()));
    assert!(knows_face(&state, owner, selected));
    assert!(!knows_face(&state, owner, hidden));

    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 3);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_count);
    assert_eq!(state.hand_count(target), target_count);
    assert_eq!(state.cards.get(selected).unwrap().owner, target);
    assert_eq!(state.cards.get(owner_hand).unwrap().owner, owner);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);

    let (_, repeated) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        4,
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn lower_authoritative_number_equipment_is_exchanged_then_its_owner_draws() {
    let mut setup = setup(12, &[8]);
    let equipment = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 3);
    let owner = setup.owner;
    let target = setup.target;
    let target_card = setup.target_cards[0];
    let owner_count = setup.state.hand_count(owner);
    let target_count = setup.state.hand_count(target);

    let (state, _) = activate(setup, 10);
    let (state, _) = resolve(state, owner, vec![target_card.raw()], false, 11);
    let (state, exchanged) = resolve(state, owner, vec![equipment.raw()], false, 12);
    assert!(matches!(exchanged.result, CommandResult::Accepted { .. }));
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().owner, target);
    assert_eq!(state.cards.get(equipment).unwrap().zone, target_hand);
    assert_eq!(state.cards.get(target_card).unwrap().owner, owner);
    assert_eq!(state.cards.get(target_card).unwrap().zone, owner_hand);
    assert_eq!(state.hand_count(owner), owner_count + 2);
    assert_eq!(state.hand_count(target), target_count);
}

#[test]
fn equal_authoritative_numbers_exchange_without_drawing() {
    let setup = setup(7, &[7]);
    let owner = setup.owner;
    let target = setup.target;
    let owner_card = setup.owner_hand;
    let target_card = setup.target_cards[0];
    let owner_count = setup.state.hand_count(owner);
    let target_count = setup.state.hand_count(target);

    let (state, _) = activate(setup, 20);
    let (state, _) = resolve(state, owner, vec![target_card.raw()], false, 21);
    let (state, exchanged) = resolve(state, owner, vec![owner_card.raw()], false, 22);
    assert!(matches!(exchanged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(owner_card).unwrap().owner, target);
    assert_eq!(state.cards.get(target_card).unwrap().owner, owner);
    assert_eq!(state.hand_count(owner), owner_count);
    assert_eq!(state.hand_count(target), target_count);
}
