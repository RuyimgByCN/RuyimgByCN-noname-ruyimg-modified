//! Golden coverage for 旧高顺「陷阵」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, RejectReason, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    card_target_within_distance, sha_use_limit_for, sha_use_unlimited_for_target,
};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const OLD_GAOSHUN: u32 = 4_065_281_135;
const XIANZHEN: u32 = 3_382_226_057;
const XIANZHEN_WIN: u32 = 3_706_320_139;
const XIANZHEN_LOSE: u32 = 2_499_130_347;
const JINJIU: u32 = 2_885_443_165;
const TEST_ARMOR: u32 = 1_990_000_002;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/yijiang/xianzhen.skill"),
    include_str!("../../../content/original/ported/yijiang/xianzhen2.skill"),
    include_str!("../../../content/original/ported/yijiang/xianzhen3.skill")
);
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const ARMOR_SOURCE: &str = r#"
skill xianzhen_test_armor(1990000002) {
  modifier armor_damage_taken_amount(fire) add 1;
}
"#;

struct Setup {
    state: GameState,
    owner: PlayerId,
    near: PlayerId,
    bound: PlayerId,
    far: PlayerId,
    owner_compare: CardId,
    target_compare: CardId,
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

fn setup(seed: u64, owner_rank: u8, target_rank: Option<u8>) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let armor = content_compiler::compile_source_named("xianzhen-armor", ARMOR_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&armor).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, near, bound, far) = (seats[0], seats[1], seats[2], seats[3]);
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
            skill_def: SkillDefId::new(XIANZHEN),
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
    let owner_compare = inject(
        tx.state_mut(),
        owner,
        99_401,
        CardKind::Other,
        Suit::Spade,
        owner_rank,
    );
    let target_compare = target_rank
        .map(|rank| {
            inject(
                tx.state_mut(),
                bound,
                99_402,
                CardKind::Other,
                Suit::Heart,
                rank,
            )
        })
        .unwrap_or(CardId::NONE);
    Setup {
        state: tx.commit().state,
        owner,
        near,
        bound,
        far,
        owner_compare,
        target_compare,
    }
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

fn compare(setup: &Setup, key: u64) -> GameState {
    let active = skill_id(&setup.state, setup.owner, XIANZHEN).unwrap();
    let (state, choose_owner_card) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.bound],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(
        choose_owner_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let (state, choose_target_card) =
        resolve(state, setup.owner, vec![setup.owner_compare.raw()], key + 1);
    assert!(matches!(
        choose_target_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let (state, compared) = resolve(
        state,
        setup.bound,
        vec![setup.target_compare.raw()],
        key + 2,
    );
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    state
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
fn packages_publish_complete_old_gaoshun_and_exact_xianzhen_modules() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(yijiang.skill_index.len(), 281);
    let active = yijiang
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIANZHEN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().unwrap().can_compare);
    for helper in [XIANZHEN_WIN, XIANZHEN_LOSE] {
        assert!(yijiang.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == helper
                && trigger.event == "turn_ended"
                && trigger.filter == "owner_is_target"
        }));
    }
    let compiled = content_compiler::compile_source_named("xianzhen", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
    for expected in &compiled.modules {
        let actual = yijiang
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(old.character_index.len(), 78);
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_GAOSHUN)
        .unwrap();
    assert_eq!(character.name_key, "old_gaoshun");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [XIANZHEN, JINJIU]);
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
}

#[test]
fn strict_win_binds_only_the_compared_target_and_expires_at_turn_end() {
    let setup = setup(2_026_072_301, 13, Some(1));
    let mut state = compare(&setup, 1);
    assert!(skill_id(&state, setup.owner, XIANZHEN_WIN).is_some());
    assert!(skill_id(&state, setup.owner, XIANZHEN_LOSE).is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.owner_compare).unwrap().zone, discard);
    assert_eq!(state.cards.get(setup.target_compare).unwrap().zone, discard);

    let sha = inject(
        &mut state,
        setup.owner,
        99_403,
        CardKind::Sha,
        Suit::Club,
        7,
    );
    assert!(card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.bound,
        1
    ));
    assert!(!card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.far,
        1
    ));
    assert!(sha_use_unlimited_for_target(
        &state,
        setup.owner,
        setup.bound
    ));
    assert!(!sha_use_unlimited_for_target(
        &state,
        setup.owner,
        setup.near
    ));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: setup.bound,
                skill_def: SkillDefId::new(TEST_ARMOR),
            },
            EffectRequest::AddSkill {
                player: setup.far,
                skill_def: SkillDefId::new(TEST_ARMOR),
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.bound,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.far,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.bound).unwrap().hp, 9);
    assert_eq!(tx.state().players.get(setup.far).unwrap().hp, 8);
    let state = end_owner_turn(tx.commit().state, setup.owner);
    assert!(skill_id(&state, setup.owner, XIANZHEN_WIN).is_none());
    assert!(!card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.bound,
        1
    ));
    assert!(!sha_use_unlimited_for_target(
        &state,
        setup.owner,
        setup.bound
    ));
}

#[test]
fn tie_and_loss_both_forbid_sha_until_turn_end() {
    for (offset, owner_rank, target_rank) in [(0, 7, 7), (1, 1, 13)] {
        let setup = setup(2_026_072_310 + offset, owner_rank, Some(target_rank));
        let state = compare(&setup, 20 + offset * 10);
        assert!(skill_id(&state, setup.owner, XIANZHEN_WIN).is_none());
        assert!(skill_id(&state, setup.owner, XIANZHEN_LOSE).is_some());
        assert_eq!(sha_use_limit_for(&state, setup.owner), 0);
        let state = end_owner_turn(state, setup.owner);
        assert!(skill_id(&state, setup.owner, XIANZHEN_LOSE).is_none());
        assert_eq!(sha_use_limit_for(&state, setup.owner), 1);
    }
}

#[test]
fn comparison_rejects_empty_hand_target_and_second_use_in_the_same_turn() {
    let empty = setup(2_026_072_320, 10, None);
    let active = skill_id(&empty.state, empty.owner, XIANZHEN).unwrap();
    let (_, rejected) = command(
        empty.state,
        empty.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![empty.bound],
            card_targets: vec![empty.owner_compare],
        },
        50,
    );
    assert!(matches!(
        rejected.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active target has no hand card to compare"
    ));

    let setup = setup(2_026_072_321, 13, Some(1));
    let mut state = compare(&setup, 60);
    let owner_card = inject(
        &mut state,
        setup.owner,
        99_404,
        CardKind::Other,
        Suit::Diamond,
        8,
    );
    inject(
        &mut state,
        setup.bound,
        99_405,
        CardKind::Other,
        Suit::Club,
        9,
    );
    let active = skill_id(&state, setup.owner, XIANZHEN).unwrap();
    let (_, repeated) = command(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.bound],
            card_targets: vec![owner_card],
        },
        70,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active skill use limit reached"
    ));
}
