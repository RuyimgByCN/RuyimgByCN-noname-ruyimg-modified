//! Golden coverage for 雁翎典韦「强袭」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YLYGQIANGXI: u32 = 2_647_374_576;
const YLYG_DIANWEI: u32 = 2_378_588_587;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/ylygqiangxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    middle: PlayerId,
    second: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, first, middle, second) = (seats[0], seats[1], seats[2], seats[3]);
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YLYGQIANGXI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YLYGQIANGXI)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first,
        middle,
        second,
        skill,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
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

fn begin_compare(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    owner_card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let (state, activated) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let (state, selected) = resolve(state, owner, vec![owner_card.raw()], key + 1);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCardSecondFailureSeatDamage
    );
    (state, selected)
}

#[test]
fn package_publishes_exact_abi156_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == YLYGQIANGXI)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.can_compare);
    assert!(targets.unused_by_skill_this_turn);
    assert_eq!(active.selection.as_ref().unwrap().zone, "hand");
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YLYG_DIANWEI)
        .unwrap();
    assert_eq!(character.name_key, "ylyg_dianwei");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, [YLYGQIANGXI]);

    let compiled = content_compiler::compile_source_named("ylygqiangxi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 156);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn strict_win_can_draw_or_discard_one_target_card() {
    let mut draw = setup(2_026_072_511);
    let owner_card = inject_hand(&mut draw.state, draw.owner, 156_101, 13);
    let target_card = inject_hand(&mut draw.state, draw.first, 156_102, 1);
    let (state, _) = begin_compare(
        draw.state, draw.owner, draw.first, draw.skill, owner_card, 1,
    );
    let (state, compared) = resolve(state, draw.first, vec![target_card.raw()], 3);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(draw.owner), 1);

    let mut discard = setup(2_026_072_512);
    let owner_card = inject_hand(&mut discard.state, discard.owner, 156_111, 13);
    let target_card = inject_hand(&mut discard.state, discard.first, 156_112, 1);
    let extra = inject_hand(&mut discard.state, discard.first, 156_113, 7);
    let (state, _) = begin_compare(
        discard.state,
        discard.owner,
        discard.first,
        discard.skill,
        owner_card,
        10,
    );
    let (state, compared) = resolve(state, discard.first, vec![target_card.raw()], 12);
    let choice = compared.choice.unwrap();
    assert_eq!(choice.observer, discard.owner);
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrDiscardTargetCard
    );
    let (state, chose_discard) = resolve(state, discard.owner, vec![2], 13);
    assert!(matches!(
        chose_discard.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let (state, discarded) = resolve(state, discard.owner, vec![extra.raw()], 14);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(discard.first), 0);
}

#[test]
fn second_non_win_loses_hp_chooses_equal_path_and_bans_the_skill() {
    let mut setup = setup(2_026_072_513);
    let owner_first = inject_hand(&mut setup.state, setup.owner, 156_201, 1);
    let owner_second = inject_hand(&mut setup.state, setup.owner, 156_202, 2);
    let first_card = inject_hand(&mut setup.state, setup.first, 156_203, 13);
    let first_again = inject_hand(&mut setup.state, setup.first, 156_204, 12);
    let second_card = inject_hand(&mut setup.state, setup.second, 156_205, 13);

    let (state, _) = begin_compare(
        setup.state,
        setup.owner,
        setup.first,
        setup.skill,
        owner_first,
        20,
    );
    let (state, first_result) = resolve(state, setup.first, vec![first_card.raw()], 22);
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!state.has_pending_choice());

    let before_duplicate = canonical_state_hash(&state);
    let (state, duplicate) = command(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.first],
            card_targets: Vec::new(),
        },
        23,
    );
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before_duplicate);
    assert_eq!(state.cards.get(first_again).unwrap().owner, setup.first);

    let (state, _) = begin_compare(
        state,
        setup.owner,
        setup.second,
        setup.skill,
        owner_second,
        24,
    );
    let (state, second_result) = resolve(state, setup.second, vec![second_card.raw()], 26);
    assert!(matches!(
        second_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 9);
    assert!(state.skills.get(setup.skill).unwrap().active_turn_banned);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseSeatPathDamage
    );
    let (state, damaged) = resolve(state, setup.owner, vec![2], 27);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    let order: Vec<_> = damaged
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::DamageDealt {
                source,
                target,
                amount: 1,
                ..
            } if *source == setup.owner => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(order, [setup.second, setup.middle, setup.first]);
    assert_eq!(state.players.get(setup.second).unwrap().hp, 9);
    assert_eq!(state.players.get(setup.middle).unwrap().hp, 9);
    assert_eq!(state.players.get(setup.first).unwrap().hp, 9);
}

#[test]
fn malformed_duplicate_target_activation_is_atomic() {
    let mut setup = setup(2_026_072_514);
    let owner_card = inject_hand(&mut setup.state, setup.owner, 156_301, 7);
    inject_hand(&mut setup.state, setup.first, 156_302, 7);
    let before = canonical_state_hash(&setup.state);
    let (state, rejected) = command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.first, setup.first],
            card_targets: vec![owner_card],
        },
        30,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}
