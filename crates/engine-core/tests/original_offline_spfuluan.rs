//! Golden coverage for SP王元姬「扶乱」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPFULUAN: u32 = 2_671_318_114;
const SPFULUAN2: u32 = 4_252_815_092;
const SPSHUDE: u32 = 2_834_305_041;
const SP_WANGYUANJI: u32 = 3_377_943_959;
const SUIT_OVERRIDE: u32 = 4_200_003_040;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/spfuluan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    far_target: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let suit_package = content_compiler::compile_source_named(
        "spfuluan-effective-suit-test",
        "skill suit_override(4200003040) {
            modifier card_suit(diamond) override heart;
        }",
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&suit_package).unwrap();

    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SPFULUAN),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SUIT_OVERRIDE),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SPFULUAN)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: others[0],
        far_target: others[1],
        skill,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn three_effective_hearts(state: &mut GameState, owner: PlayerId) -> [CardId; 3] {
    [
        inject(
            state,
            owner,
            ZoneKind::Hand,
            304_001,
            CardKind::Shan,
            Suit::Heart,
        ),
        inject(
            state,
            owner,
            ZoneKind::Hand,
            304_002,
            CardKind::Tao,
            Suit::Heart,
        ),
        inject(
            state,
            owner,
            ZoneKind::Equip,
            304_003,
            CardKind::Armor,
            Suit::Diamond,
        ),
    ]
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: cards,
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_abi304_contract_modifier_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 838);
    assert_eq!(package.character_index.len(), 382);
    assert_eq!(package.modules.len(), 724);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 128);

    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SPFULUAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(targets.allow_self && targets.in_attack_range);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 3, 3)
    );
    assert!(selection.allow_cancel && selection.same_suit);

    let modifier = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == SPFULUAN2)
        .unwrap();
    assert_eq!(modifier.target, "card_use_cancel");
    assert_eq!(modifier.selector.as_deref(), Some("sha"));

    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_WANGYUANJI)
        .unwrap();
    assert_eq!(character.name_key, "sp_wangyuanji");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [SPFULUAN, SPSHUDE]);

    let compiled = content_compiler::compile_source_named("spfuluan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 304);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn effective_same_suit_hand_equip_cost_turns_target_and_installs_temporary_ban() {
    let mut setup = setup(20_260_730_1);
    let cards = three_effective_hearts(&mut setup.state, setup.owner);
    assert_eq!(
        engine_core::query::effective_card_suit(
            &setup.state,
            setup.owner,
            setup.state.cards.get(cards[2]).unwrap()
        ),
        Suit::Heart
    );

    let (state, result) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cards.to_vec(),
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
    assert!(state.players.get(setup.target).unwrap().turned_over);
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 1);
    let temporary = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == SPFULUAN2)
        .unwrap();
    assert_eq!(temporary.temporary_until_turn_end_of, setup.owner);
}

#[test]
fn mixed_suits_and_out_of_range_targets_are_rejected_atomically() {
    let mut mixed_setup = setup(20_260_730_2);
    let cards = three_effective_hearts(&mut mixed_setup.state, mixed_setup.owner);
    let spade = inject(
        &mut mixed_setup.state,
        mixed_setup.owner,
        ZoneKind::Hand,
        304_004,
        CardKind::Sha,
        Suit::Spade,
    );
    let before = canonical_state_hash(&mixed_setup.state);
    let (same, mixed) = activate(
        mixed_setup.state.clone(),
        mixed_setup.owner,
        mixed_setup.target,
        mixed_setup.skill,
        vec![cards[0], cards[1], spade],
        1,
    );
    assert!(matches!(mixed.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (same, far) = activate(
        mixed_setup.state,
        mixed_setup.owner,
        mixed_setup.far_target,
        mixed_setup.skill,
        cards.to_vec(),
        2,
    );
    assert!(matches!(far.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut setup = setup(20_260_730_7);
    let cards: Vec<_> = (0..3)
        .map(|index| {
            inject(
                &mut setup.state,
                setup.owner,
                ZoneKind::Hand,
                304_010 + index,
                CardKind::Sha,
                Suit::None,
            )
        })
        .collect();
    let before = canonical_state_hash(&setup.state);
    let (same, no_suit) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cards,
        3,
    );
    assert!(matches!(no_suit.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn selection_can_cancel_without_cards_or_use_count_and_self_target_is_legal() {
    let mut cancel_setup = setup(20_260_730_3);
    let cards = three_effective_hearts(&mut cancel_setup.state, cancel_setup.owner);
    let original_zones: Vec<_> = cards
        .iter()
        .map(|card| cancel_setup.state.cards.get(*card).unwrap().zone)
        .collect();
    let (pending, offered) = activate(
        cancel_setup.state,
        cancel_setup.owner,
        cancel_setup.target,
        cancel_setup.skill,
        Vec::new(),
        1,
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (3, 3));
    assert!(choice.allow_cancel);
    let (cancelled, result) = resolve(pending, cancel_setup.owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        cancelled.skills.get(cancel_setup.skill).unwrap().use_count,
        0
    );
    assert!(
        !cancelled
            .players
            .get(cancel_setup.target)
            .unwrap()
            .turned_over
    );
    assert!(cards
        .iter()
        .zip(original_zones)
        .all(|(card, zone)| { cancelled.cards.get(*card).map(|card| card.zone) == Some(zone) }));

    let mut setup = setup(20_260_730_4);
    let cards = three_effective_hearts(&mut setup.state, setup.owner);
    let (state, result) = activate(
        setup.state,
        setup.owner,
        setup.owner,
        setup.skill,
        cards.to_vec(),
        3,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.players.get(setup.owner).unwrap().turned_over);
}

#[test]
fn active_sha_and_ignore_count_virtual_sha_are_blocked_until_turn_end() {
    let mut setup = setup(20_260_730_5);
    let cards = three_effective_hearts(&mut setup.state, setup.owner);
    let (mut state, result) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cards.to_vec(),
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let sha = inject(
        &mut state,
        setup.owner,
        ZoneKind::Hand,
        304_005,
        CardKind::Sha,
        Suit::Club,
    );
    let before = canonical_state_hash(&state);
    let revision = state.revision;
    let (same, rejected) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut tx = Transaction::begin(same);
    let hand = tx.state().hand_count(setup.owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChooseUseVirtualShaOrDraw {
            player: setup.owner,
            skill: setup.skill,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(setup.owner), hand + 1);

    let mut state = tx.commit().state;
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, ended) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == setup.owner && skill.def.raw() == SPFULUAN2));
}

#[test]
fn dead_or_dying_players_cannot_activate_true_active_skills() {
    let mut setup = setup(20_260_730_6);
    let cards = three_effective_hearts(&mut setup.state, setup.owner);
    setup.state.players.get_mut(setup.owner).unwrap().dying = true;
    let (same, dying) = activate(
        setup.state.clone(),
        setup.owner,
        setup.target,
        setup.skill,
        cards.to_vec(),
        1,
    );
    assert!(matches!(dying.result, CommandResult::Rejected { .. }));
    assert_eq!(same.skills.get(setup.skill).unwrap().use_count, 0);

    setup.state.players.get_mut(setup.owner).unwrap().dying = false;
    setup.state.players.get_mut(setup.owner).unwrap().alive = false;
    let (same, dead) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cards.to_vec(),
        2,
    );
    assert!(matches!(dead.result, CommandResult::Rejected { .. }));
    assert_eq!(same.skills.get(setup.skill).unwrap().use_count, 0);
}
