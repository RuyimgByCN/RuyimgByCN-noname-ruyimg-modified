//! Golden coverage for 九鼎庞统「连环」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JDLIANHUAN: u32 = 3_773_194_665;
const JD_SB_PANGTONG: u32 = 3_671_358_880;
const OLDNIEPAN: u32 = 3_136_864_229;
const PROTECTED_EQUIP: u32 = 4_200_000_001;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/offline/jdlianhuan.skill");

fn setup(player_count: u8, seed: u64) -> (GameState, Vec<PlayerId>, SkillId) {
    let source = format!(
        "{SOURCE}\nskill protected_equip({PROTECTED_EQUIP}) {{\n  modifier card_discard_protected(equip) override 1;\n}}\n"
    );
    let package = content_compiler::compile_source_named("jdlianhuan-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, seed, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
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
        player: seats[0],
        skill_def: SkillDefId::new(JDLIANHUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = seats[0];
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(seats[0])
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == JDLIANHUAN)
        })
        .unwrap();
    (tx.commit().state, seats, skill)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(305_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![card],
            },
        },
    )
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
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

fn is_linked(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

fn assert_accepted(result: &PipelineResult) {
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
}

#[test]
fn package_publishes_character_skill_dependency_and_abi305() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 305);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official" && dependency.package_id == "shenhua"
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == JDLIANHUAN
            && view.as_kind == "tiesuo"
            && view.window == "play_phase"
            && view.filter == "club"
            && view
                .source_zone
                .as_deref()
                .is_none_or(|zone| zone == "hand")
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JDLIANHUAN
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty"
            && !trigger.optional
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == JD_SB_PANGTONG
            && character.name_key == "jd_sb_pangtong"
            && character.max_hp == 3
            && character.skills == [JDLIANHUAN, OLDNIEPAN]
    }));
}

#[test]
fn only_club_hand_cards_convert_and_zero_target_recasts() {
    let (state, seats, skill) = setup(2, 20_260_730);
    let owner = seats[0];
    let mut tx = Transaction::begin(state);
    let wrong_hand = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        3,
    );
    let club_equip = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        4,
    );
    let wrong_equip = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        5,
    );
    let club_hand = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        6,
    );
    let state = tx.commit().state;

    for (key, card) in [(1, wrong_hand), (2, club_equip), (3, wrong_equip)] {
        let (next, result) = activate(state.clone(), owner, skill, card, vec![], key);
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
        assert_eq!(
            engine_core::canonical_state_hash(&next),
            engine_core::canonical_state_hash(&state)
        );
    }

    let before = state.hand_count(owner);
    let (state, result) = activate(state, owner, skill, club_hand, vec![], 4);
    assert_accepted(&result);
    assert!(!state.hand_cards(owner).contains(&club_hand));
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn converted_tiesuo_opens_direct_cancelable_discard_then_resolves() {
    let (state, seats, skill) = setup(2, 20_260_731);
    let (owner, target) = (seats[0], seats[1]);
    let mut tx = Transaction::begin(state);
    let source = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        7,
    );
    let target_card = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Spade,
        8,
    );

    let (state, result) = activate(tx.commit().state, owner, skill, source, vec![target], 10);
    assert_accepted(&result);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == target_card.raw()));

    let (state, result) = resolve(state, owner, vec![], true, 11);
    assert_accepted(&result);
    assert!(!state.has_pending_choice());
    assert!(state.hand_cards(target).contains(&target_card));
    assert!(is_linked(&state, target));
}

#[test]
fn real_tiesuo_discards_before_a_later_wuxie_nullifies_it() {
    let (state, seats, _) = setup(3, 20_260_732);
    let (owner, target, responder) = (seats[0], seats[1], seats[2]);
    let mut tx = Transaction::begin(state);
    let tiesuo = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickTiesuo,
        Suit::Spade,
        9,
    );
    let discard = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        10,
    );
    let wuxie = inject(
        &mut tx,
        responder,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        Suit::Club,
        11,
    );

    let (state, result) = play(tx.commit().state, owner, tiesuo, vec![target], 20);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);

    let (state, result) = resolve(state, owner, vec![discard.raw()], false, 21);
    assert_accepted(&result);
    assert_eq!(result.choice.as_ref().unwrap().observer, responder);
    assert!(result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == wuxie.raw()));
    let discard_pile = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(discard).unwrap().zone, discard_pile);

    let (state, result) = resolve(state, responder, vec![wuxie.raw()], false, 22);
    assert_accepted(&result);
    assert!(!state.has_pending_choice());
    assert!(!is_linked(&state, target));
    assert_eq!(state.cards.get(discard).unwrap().zone, discard_pile);
}

#[test]
fn linked_and_empty_targets_do_not_offer_discard() {
    let (state, seats, _) = setup(3, 20_260_733);
    let (owner, linked, empty) = (seats[0], seats[1], seats[2]);
    let mut tx = Transaction::begin(state);
    let tiesuo = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickTiesuo,
        Suit::Spade,
        12,
    );
    inject(
        &mut tx,
        linked,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        13,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: linked,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();

    let (state, result) = play(tx.commit().state, owner, tiesuo, vec![linked, empty], 30);
    assert_accepted(&result);
    assert!(result.choice.is_none());
    assert!(!is_linked(&state, linked));
    assert!(is_linked(&state, empty));
}

#[test]
fn multi_target_choices_follow_declared_order_and_empty_legal_scope_is_noop() {
    let (state, seats, _) = setup(3, 20_260_734);
    let (owner, first, second) = (seats[0], seats[2], seats[1]);
    let mut tx = Transaction::begin(state);
    let tiesuo = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickTiesuo,
        Suit::Spade,
        1,
    );
    let first_card = inject(
        &mut tx,
        first,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        2,
    );
    let second_card = inject(
        &mut tx,
        second,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        3,
    );

    let (state, result) = play(tx.commit().state, owner, tiesuo, vec![first, second], 40);
    let choice = result.choice.as_ref().unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == first_card.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == second_card.raw()));

    let (state, result) = resolve(state, owner, vec![], true, 41);
    let choice = result.choice.as_ref().unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == second_card.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == first_card.raw()));
    let (state, result) = resolve(state, owner, vec![], true, 42);
    assert_accepted(&result);
    assert!(is_linked(&state, first) && is_linked(&state, second));

    let (state, seats, _) = setup(2, 20_260_735);
    let (owner, target) = (seats[0], seats[1]);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(PROTECTED_EQUIP),
        }],
    )
    .unwrap();
    let protected = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        4,
    );
    let tiesuo = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickTiesuo,
        Suit::Spade,
        5,
    );
    let (state, result) = play(tx.commit().state, owner, tiesuo, vec![target], 43);
    assert_accepted(&result);
    assert!(result.choice.is_none());
    assert!(is_linked(&state, target));
    assert_eq!(
        state.cards.get(protected).unwrap().zone,
        state.zones.find(ZoneKind::Equip, target).unwrap()
    );
}
