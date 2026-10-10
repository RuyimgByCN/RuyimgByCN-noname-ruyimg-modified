//! Golden coverage for REFRESH 公孙瓒「趫猛」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_GONGSUNZAN: u32 = 2_326_980_049;
const DCYICONG: u32 = 3_742_694_356;
const DCQIAOMENG: u32 = 3_871_427_916;
const DCQIAOMENG_EFFECT: u32 = 2_662_675_115;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/dcqiaomeng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("dcqiaomeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730_313, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: seats[0],
            skill_def: SkillDefId::new(DCQIAOMENG),
        },
        EffectRequest::SetTurnState {
            current: seats[0],
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, seats[0], seats[1], seats[2])
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
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

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state,
        owner,
        CommandKind::PlayCard {
            card,
            targets,
            card_targets: vec![],
        },
        key,
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

fn discard_contains(state: &GameState, card: CardId) -> bool {
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    state.zones.get(discard).unwrap().cards.contains(&card)
}

fn choose_target_and_card(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    selected: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let (state, result) = resolve(state, owner, vec![1], key);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, result) = resolve(state, owner, vec![target.raw()], key + 1);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    resolve(state, owner, vec![selected.raw()], key + 2)
}

#[test]
fn package_publishes_dc_gongsunzan_and_exact_abi313_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 313);
    assert!(package.character_index.iter().any(|character| {
        character.id == DC_GONGSUNZAN
            && character.name_key == "dc_gongsunzan"
            && character.max_hp == 4
            && character.skills == [DCYICONG, DCQIAOMENG]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCQIAOMENG)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "card_targeted",
            "owner_source_first_target_black_card_any_target_other_discardable_hand_equip",
            true
        )
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DCQIAOMENG_EFFECT
            && modifier.target == "card_use_unrespondable_for_skill_card_policy"
    }));
}

#[test]
fn equipment_is_gained_and_discarded_trick_makes_the_sha_unrespondable() {
    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        313_001,
    );
    let weapon = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Diamond,
        313_002,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (offered, result) = play(tx.commit().state, owner, sha, vec![target], 1);
    assert_eq!(
        result.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::Confirm),
        "{result:?}"
    );
    let (resolved, result) = choose_target_and_card(offered, owner, target, weapon, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(resolved.hand_cards(owner).contains(&weapon));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 1);

    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        313_011,
    );
    let _shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        313_012,
    );
    let trick = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::TrickGuohe,
        Suit::Diamond,
        313_013,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (offered, _) = play(tx.commit().state, owner, sha, vec![target], 11);
    let (resolved, result) = choose_target_and_card(offered, owner, target, trick, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(discard_contains(&resolved, trick));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn basic_keeps_normal_response_red_is_rejected_and_all_targets_are_available_once() {
    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        313_021,
    );
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        313_022,
    );
    let tao = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        313_023,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (offered, _) = play(tx.commit().state, owner, sha, vec![target], 21);
    let (responding, result) = choose_target_and_card(offered, owner, target, tao, 22);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    let (resolved, result) = resolve(responding, target, vec![shan.raw()], 25);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(discard_contains(&resolved, tao));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp);

    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let red_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        313_031,
    );
    inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Club,
        313_032,
    );
    let (_, result) = play(tx.commit().state, owner, red_sha, vec![target], 31);
    assert!(result.choice.is_none());

    let (state, owner, first, second) = setup();
    let mut tx = Transaction::begin(state);
    let trick = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickTiaojiyanmei,
        Suit::Club,
        313_041,
    );
    let first_card = inject(
        &mut tx,
        first,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        313_042,
    );
    let second_card = inject(
        &mut tx,
        second,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Diamond,
        313_043,
    );
    inject(
        &mut tx,
        second,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        313_044,
    );
    let (offered, result) = play(tx.commit().state, owner, trick, vec![first, second], 41);
    assert_eq!(
        result.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::Confirm),
        "{result:?}"
    );
    let (selecting_target, result) = resolve(offered, owner, vec![1], 42);
    let choice = result.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == first.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == second.raw()));
    let (selecting_card, result) = resolve(selecting_target, owner, vec![second.raw()], 43);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (resolved, result) = resolve(selecting_card, owner, vec![second_card.raw()], 44);
    assert!(discard_contains(&resolved, second_card));
    assert!(resolved.hand_cards(first).contains(&first_card));
    assert!(!result
        .choice
        .as_ref()
        .is_some_and(|choice| choice.observer == owner && choice.kind == ChoiceKind::Confirm));
}
