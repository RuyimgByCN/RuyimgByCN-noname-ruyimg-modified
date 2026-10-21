//! Golden coverage for REFRESH 界庞德「鞬出」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::sha_use_limit_for;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const OL_PANGDE: u32 = 3_770_183_463;
const MASHU: u32 = 2;
const REJIANCHU: u32 = 3_270_398_373;
const REJIANCHU2: u32 = 3_848_225_310;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/refresh/rejianchu.skill"),
    include_str!("../../../../content/original/ported/refresh/rejianchu2.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("rejianchu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_730, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
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
        skill_def: SkillDefId::new(REJIANCHU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
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

fn play_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
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

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == REJIANCHU2).then_some(*count))
        .unwrap_or(0)
}

#[test]
fn package_publishes_complete_ol_pangde_with_abi313_support() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 313);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_PANGDE
            && character.name_key == "ol_pangde"
            && character.max_hp == 4
            && character.skills == [MASHU, REJIANCHU]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIANCHU)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "card_targeted",
            "owner_source_sha_target_alive_has_discardable_hand_equip",
            true
        )
    );
}

#[test]
fn basic_card_gives_the_target_the_physical_sha_and_keeps_normal_response() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 312_001);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 312_002);
    let tao = inject(&mut tx, target, ZoneKind::Hand, CardKind::Tao, 312_003);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (offered, result) = play_sha(tx.commit().state, owner, target, sha, 1);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (selecting, result) = resolve(offered, owner, vec![1], 2);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (responding, result) = resolve(selecting, owner, vec![tao.raw()], 3);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    assert!(responding.hand_cards(target).contains(&sha));
    assert_eq!(mark_count(&responding, owner), 0);
    assert!(skill_id(&responding, owner, REJIANCHU2).is_none());
    let (resolved, result) = resolve(responding, target, vec![shan.raw()], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp);
}

#[test]
fn nonbasic_cards_stack_sha_uses_then_the_turn_end_clears_the_bonus() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let first_sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 312_011);
    let second_sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 312_012);
    let first_trick = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        312_013,
    );
    let second_trick = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::TrickGuohe,
        312_014,
    );
    let hp = tx.state().players.get(target).unwrap().hp;

    let (offered, _) = play_sha(tx.commit().state, owner, target, first_sha, 11);
    let (selecting, _) = resolve(offered, owner, vec![1], 12);
    let (resolved, result) = resolve(selecting, owner, vec![first_trick.raw()], 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(mark_count(&resolved, owner), 1);
    assert_eq!(sha_use_limit_for(&resolved, owner), 2);

    let (offered, _) = play_sha(resolved, owner, target, second_sha, 14);
    let (selecting, _) = resolve(offered, owner, vec![1], 15);
    let (resolved, result) = resolve(selecting, owner, vec![second_trick.raw()], 16);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 2);
    assert_eq!(mark_count(&resolved, owner), 2);
    assert_eq!(sha_use_limit_for(&resolved, owner), 3);

    let mut tx = Transaction::begin(resolved);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 1,
            damaged_players: 1,
            card_targeted_other: true,
            current_dealt_damage_other: true,
            phase_targeted_players: 1,
            phase_hp_changed_players: 1,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(skill_id(tx.state(), owner, REJIANCHU2).is_none());
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(sha_use_limit_for(tx.state(), owner), 1);
}
