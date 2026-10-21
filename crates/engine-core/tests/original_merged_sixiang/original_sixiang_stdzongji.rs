//! Golden coverage for SIXIANG「纵计」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZONGJI: u32 = 3_394_363_597;
const STD_WANGYUN: u32 = 3_976_039_480;
const SIXIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdzongji.skill");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzongji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_724, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let target = seats[2];
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
        skill_def: SkillDefId::new(STDZONGJI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, source, target)
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
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

fn assert_discarded(state: &GameState, card: CardId) {
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.get(card).unwrap();
    assert_eq!(card.zone, discard);
    assert_eq!(card.owner, PlayerId::NONE);
}

#[test]
fn package_publishes_exact_skill_and_closed_character() {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDZONGJI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WANGYUN)
        .unwrap();
    assert_eq!(character.name_key, "std_wangyun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [2_318_160_114, STDZONGJI]);
}

#[test]
fn accepted_sha_damage_discards_target_then_source_cards() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, source, ZoneKind::Hand, CardKind::Sha, 126_000);
    let source_card = inject(
        &mut tx,
        source,
        ZoneKind::Equip,
        CardKind::HorseOff,
        126_001,
    );
    let target_card = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 126_002);

    let (offered, result) = play(tx.commit().state, source, sha, vec![target], 1);
    let confirm = result.choice.as_ref().unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(confirm.observer, owner);

    let (target_choice_state, result) = resolve(offered, owner, vec![1], false, 2);
    let target_choice = result.choice.as_ref().unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(target_choice.observer, owner);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_card.raw()]
    );

    let (source_choice_state, result) = resolve(
        target_choice_state,
        owner,
        vec![target_card.raw()],
        false,
        3,
    );
    let source_choice = result.choice.as_ref().unwrap();
    assert_eq!(source_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(source_choice.observer, owner);
    assert_eq!(
        source_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [source_card.raw()]
    );

    let (resolved, result) = resolve(
        source_choice_state,
        owner,
        vec![source_card.raw()],
        false,
        4,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(target).unwrap().hp, 3);
    assert_discarded(&resolved, target_card);
    assert_discarded(&resolved, source_card);
}

#[test]
fn duel_damage_uses_the_same_target_then_source_sequence() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        126_005,
    );
    let source_card = inject(&mut tx, source, ZoneKind::Hand, CardKind::Other, 126_006);
    let target_card = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 126_007);
    let (offered, result) = play(tx.commit().state, source, duel, vec![target], 5);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);

    let (state, _) = resolve(offered, owner, vec![1], false, 6);
    let (state, _) = resolve(state, owner, vec![target_card.raw()], false, 7);
    let (resolved, result) = resolve(state, owner, vec![source_card.raw()], false, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(target).unwrap().hp, 3);
    assert_discarded(&resolved, target_card);
    assert_discarded(&resolved, source_card);
}

#[test]
fn decline_preserves_both_participants_cards() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, source, ZoneKind::Hand, CardKind::Sha, 126_010);
    let source_card = inject(&mut tx, source, ZoneKind::Hand, CardKind::Other, 126_011);
    let target_card = inject(&mut tx, target, ZoneKind::Equip, CardKind::Armor, 126_012);
    let (offered, result) = play(tx.commit().state, source, sha, vec![target], 10);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);

    let (declined, result) = resolve(offered, owner, vec![], true, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(declined.cards.get(source_card).unwrap().owner, source);
    assert_eq!(declined.cards.get(target_card).unwrap().owner, target);
}

#[test]
fn non_sha_or_duel_damage_and_empty_participants_do_not_prompt() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let nanman = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickNanman,
        126_020,
    );
    inject(&mut tx, source, ZoneKind::Hand, CardKind::Other, 126_021);
    inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 126_022);
    let (_, result) = play(tx.commit().state, source, nanman, vec![], 20);
    assert!(
        result
            .choice
            .as_ref()
            .is_none_or(|choice| choice.observer != owner),
        "ordinary trick damage must not offer stdzongji"
    );

    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, source, ZoneKind::Hand, CardKind::Sha, 126_023);
    let (_, result) = play(tx.commit().state, source, sha, vec![target], 21);
    assert!(
        result
            .choice
            .as_ref()
            .is_none_or(|choice| choice.observer != owner),
        "empty source and target must not offer stdzongji"
    );
}
