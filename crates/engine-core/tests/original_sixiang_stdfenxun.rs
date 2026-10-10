//! Golden coverage for 标丁奉「奋迅」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, RejectReason, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::query::card_target_within_distance;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const STD_DINGFENG: u32 = 3_454_509_574;
const STDDUANBING: u32 = 2_170_694_873;
const STDFENXUN: u32 = 2_579_637_455;
const STDFENXUN_EFFECT: u32 = 3_372_226_216;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdfenxun.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    bound: PlayerId,
    far: PlayerId,
    skill: engine_api::SkillId,
    armor: CardId,
    weapon: CardId,
    sha: CardId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(include_armor: bool) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, 2_026_072_407, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let owner = tx.state().turn.current;
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
            player: owner,
            skill_def: SkillDefId::new(STDFENXUN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let armor = if include_armor {
        inject(
            tx.state_mut(),
            owner,
            130_001,
            CardKind::Armor,
            Suit::Heart,
            2,
        )
    } else {
        CardId::NONE
    };
    let weapon = inject(
        tx.state_mut(),
        owner,
        130_002,
        CardKind::Weapon,
        Suit::Club,
        5,
    );
    let sha = inject(
        tx.state_mut(),
        owner,
        130_003,
        CardKind::Sha,
        Suit::Spade,
        7,
    );
    let distant: Vec<_> = players
        .into_iter()
        .filter(|player| {
            *player != owner && !card_target_within_distance(tx.state(), owner, sha, *player, 1)
        })
        .collect();
    assert!(distant.len() >= 2);
    let skill = skill_id(tx.state(), owner, STDFENXUN).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        bound: distant[0],
        far: distant[1],
        skill,
        armor,
        weapon,
        sha,
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
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

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_abi131_skills_and_complete_std_dingfeng() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == STDFENXUN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        (
            active.targets.as_ref().unwrap().min,
            active.targets.as_ref().unwrap().max
        ),
        (1, 1)
    );
    assert!(!active.targets.as_ref().unwrap().allow_self);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (1, 1));
    assert_eq!(selection.filter.as_deref(), Some("card_armor"));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == STDFENXUN_EFFECT
            && modifier.target == "card_distance_unlimited_for_skill_bound_target"
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDFENXUN_EFFECT
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DINGFENG)
        .unwrap();
    assert_eq!(character.name_key, "std_dingfeng");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDDUANBING, STDFENXUN]);

    let compiled = content_compiler::compile_source_named("stdfenxun", SOURCE).unwrap();
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn armor_cost_binds_only_the_selected_target_and_expires_at_turn_end() {
    let setup = setup(true);
    let (offered, activation) = command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    assert_eq!(
        offered.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let options: Vec<_> = activation
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [setup.armor.raw()]);

    let before_forgery = canonical_state_hash(&offered);
    let (unchanged, forged) = resolve(offered.clone(), setup.owner, vec![setup.weapon.raw()], 2);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forgery);

    let (resolved, result) = resolve(offered, setup.owner, vec![setup.armor.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!resolved.has_pending_choice());
    let discard = resolved
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(resolved.cards.get(setup.armor).unwrap().zone, discard);
    let helper = skill_id(&resolved, setup.owner, STDFENXUN_EFFECT).unwrap();
    let mut bound_state = b"BND1".to_vec();
    bound_state.extend_from_slice(&setup.bound.raw().to_le_bytes());
    assert_eq!(resolved.skills.get(helper).unwrap().state_blob, bound_state);
    assert!(card_target_within_distance(
        &resolved,
        setup.owner,
        setup.sha,
        setup.bound,
        1
    ));
    assert!(!card_target_within_distance(
        &resolved,
        setup.owner,
        setup.sha,
        setup.far,
        1
    ));

    let (_, repeated) = command(
        resolved.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.far],
            card_targets: vec![setup.weapon],
        },
        4,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active skill use limit reached"
    ));

    let expired = end_owner_turn(resolved, setup.owner);
    assert!(skill_id(&expired, setup.owner, STDFENXUN_EFFECT).is_none());
    assert!(!card_target_within_distance(
        &expired,
        setup.owner,
        setup.sha,
        setup.bound,
        1
    ));
}

#[test]
fn self_target_and_activation_without_armor_are_rejected_atomically() {
    let with_armor = setup(true);
    let before_self = canonical_state_hash(&with_armor.state);
    let (same, self_target) = command(
        with_armor.state,
        with_armor.owner,
        CommandKind::ActivateSkill {
            skill: with_armor.skill,
            targets: vec![with_armor.owner],
            card_targets: vec![with_armor.armor],
        },
        10,
    );
    assert!(matches!(self_target.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_self);

    let without_armor = setup(false);
    let before_cost = canonical_state_hash(&without_armor.state);
    let (same, no_cost) = command(
        without_armor.state,
        without_armor.owner,
        CommandKind::ActivateSkill {
            skill: without_armor.skill,
            targets: vec![without_armor.bound],
            card_targets: Vec::new(),
        },
        11,
    );
    assert!(matches!(no_cost.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before_cost);
}
