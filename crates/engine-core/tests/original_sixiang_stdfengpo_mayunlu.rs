//! Golden for SIXIANG Ma Yunlu's 标凤魄 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDFENGPO: u32 = 2_400_535_495;
const MASHU: u32 = 2;
const STD_MAYUNLU: u32 = 3_313_632_024;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdfengpo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdfengpo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
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
            player: owner,
            skill_def: SkillDefId::new(STDFENGPO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 20;
        participant.hp = 15;
    }
    (tx.commit().state, owner, target, other)
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

fn begin_sha_damage(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    flags: u8,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: flags,
        }],
    );
    (tx.commit().state, result)
}

fn accept(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    resolve(state, owner, vec![1], false, key)
}

#[test]
fn package_publishes_stdfengpo_and_std_mayunlu_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert_eq!(package.trigger_index.len(), 124);
    assert_eq!(package.modules.len(), 147);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == STDFENGPO)
        .unwrap();
    assert_eq!(skill.name, "stdfengpo");
    assert!(!skill.locked);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDFENGPO)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_damage_before_endpoint_discardable_hand_equip"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_MAYUNLU)
        .unwrap();
    assert_eq!(character.skills, [STDFENGPO, MASHU]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn diamond_from_either_endpoint_adds_one_after_discard() {
    for (seed, from_owner) in [(2_026_072_710, false), (2_026_072_711, true)] {
        let (mut state, owner, target, _) = setup(seed);
        let card_owner = if from_owner { owner } else { target };
        let card = inject(
            &mut state,
            card_owner,
            if from_owner {
                ZoneKind::Equip
            } else {
                ZoneKind::Hand
            },
            Suit::Diamond,
            230_000 + u32::from(from_owner),
        );
        let hp_before = state.players.get(target).unwrap().hp;
        let (state, result) = begin_sha_damage(state, owner, target, 0);
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        let (state, accepted) = accept(state, owner, 1);
        let choice = accepted.choice.unwrap();
        assert_eq!(choice.kind, ChoiceKind::SelectCards);
        assert_eq!(choice.observer, owner);
        assert!(choice
            .options
            .iter()
            .any(|option| option.token == card.raw()));
        let (state, result) = resolve(state, owner, vec![card.raw()], false, 2);
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        assert_eq!(state.players.get(target).unwrap().hp, hp_before - 2);
        assert_eq!(
            state.cards.get(card).unwrap().zone,
            state
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap()
        );
    }
}

#[test]
fn nonmatching_suit_discards_without_increasing_damage() {
    let (mut state, owner, target, _) = setup(2_026_072_712);
    let card = inject(&mut state, target, ZoneKind::Hand, Suit::Club, 230_010);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, result) = begin_sha_damage(state, owner, target, 0);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = accept(state, owner, 3);
    let (state, result) = resolve(state, owner, vec![card.raw()], false, 4);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
}

#[test]
fn chain_sha_is_included_and_empty_endpoints_do_not_prompt() {
    let (mut state, owner, target, _) = setup(2_026_072_713);
    let card = inject(&mut state, owner, ZoneKind::Hand, Suit::Diamond, 230_020);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, result) = begin_sha_damage(
        state,
        owner,
        target,
        rule_api::damage_card_flag::CHAIN_PROPAGATED,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = accept(state, owner, 5);
    let (state, result) = resolve(state, owner, vec![card.raw()], false, 6);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 2);

    let (state, owner, target, _) = setup(2_026_072_714);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, result) = begin_sha_damage(state, owner, target, 0);
    assert!(result.is_ok());
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::None);
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
}
