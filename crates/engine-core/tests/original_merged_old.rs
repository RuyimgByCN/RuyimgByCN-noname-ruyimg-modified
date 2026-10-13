//! O2 全量合并批次 1（2026-10-14）：家族 old 的 20 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_old_chegu {
//! Golden coverage for「彻骨」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHEGU: u32 = 4_294_592_545;
const SOURCE: &str = include_str!("../../../content/original/ported/old/chegu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_271 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, result)
}

#[test]
fn cancels_the_original_sha_and_uniform_discards_raise_this_turns_limit() {
    let package = content_compiler::compile_source_named("chegu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_914, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let sha_target = players[1];
    let discard_target = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CHEGU),
    });
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 7);
    let red_basic = inject(&mut tx, discard_target, CardKind::Tao, Suit::Heart, 3);
    let red_equip = inject(&mut tx, discard_target, CardKind::Weapon, Suit::Diamond, 4);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let target_hp = tx.state().players.get(sha_target).unwrap().hp;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let state = tx.commit().state;

    let revision = state.revision;
    let (state, played) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![sha_target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert!(played.choice.is_some());
    let (state, _) = resolve(state, owner, vec![1], 2);
    let (state, _) = resolve(state, owner, vec![1], 3);
    let (state, _) = resolve(state, owner, vec![discard_target.raw()], 4);
    let (state, _) = resolve(state, owner, vec![red_basic.raw(), red_equip.raw()], 5);

    assert_eq!(state.players.get(sha_target).unwrap().hp, target_hp);
    assert_eq!(state.cards.get(red_basic).unwrap().zone, discard);
    assert_eq!(state.cards.get(red_equip).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    let extra1 = inject(&mut tx, discard_target, CardKind::Shan, Suit::Club, 5);
    let extra2 = inject(&mut tx, discard_target, CardKind::Tao, Suit::Heart, 6);
    let extra3 = inject(&mut tx, discard_target, CardKind::Weapon, Suit::Spade, 8);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveChegu {
                player: owner,
                card: sha,
                targets: vec![discard_target],
                cards: Vec::new(),
                branch: 1,
                index: 0,
                stage: 2,
                skill: SkillId::new(skill.raw()),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let choice = tx.commit().output.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (1, 3));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra1.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra2.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra3.raw()));
}

#[test]
fn package_publishes_chegu_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHEGU && skill.name == "chegu"));
    assert!(package.character_index.iter().any(|character| {
        character.id == 2_763_282_459 && character.name_key == "two_yj_hanbing"
    }));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_gaoshun_xianzhen {
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
}

mod original_old_guhuo {
//! Golden coverage for old「蛊惑」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLD_GUHUO: u32 = 3_484_005_747;
const SOURCE: &str = include_str!("../../../content/original/ported/old/old_guhuo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    challenger: PlayerId,
    target: PlayerId,
    source: CardId,
}

fn setup(kind: CardKind, suit: Suit) -> Setup {
    let package = content_compiler::compile_source_named("old-guhuo-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 273);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_931, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let challenger = players[1];
    let target = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_GUHUO),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let source = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_273),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(source);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        challenger,
        target,
        source,
    }
}

fn choose(state: GameState, token: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn declare_sha(setup: Setup, decisions: [u32; 2]) -> GameState {
    let skill = setup.state.players.get(setup.owner).unwrap().skills[0];
    let revision = setup.state.revision;
    let (state, result) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Sha.tag(),
                targets: vec![setup.target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = choose(state, setup.source.raw(), 2);
    let state = choose(state, decisions[0], 3);
    choose(state, decisions[1], 4)
}

#[test]
fn challenge_truth_fake_and_no_challenge_follow_legacy_resolution() {
    let truthful_heart = setup(CardKind::Sha, Suit::Heart);
    let target = truthful_heart.target;
    let challenger = truthful_heart.challenger;
    let target_hp = truthful_heart.state.players.get(target).unwrap().hp;
    let challenger_hp = truthful_heart.state.players.get(challenger).unwrap().hp;
    let state = declare_sha(truthful_heart, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.players.get(challenger).unwrap().hp, challenger_hp - 1);

    let truthful_spade = setup(CardKind::Sha, Suit::Spade);
    let target = truthful_spade.target;
    let source = truthful_spade.source;
    let target_hp = truthful_spade.state.players.get(target).unwrap().hp;
    let state = declare_sha(truthful_spade, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(source).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );

    let fake = setup(CardKind::Tao, Suit::Heart);
    let target = fake.target;
    let challenger = fake.challenger;
    let target_hp = fake.state.players.get(target).unwrap().hp;
    let hand_before = fake.state.hand_count(challenger);
    let state = declare_sha(fake, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    assert_eq!(state.hand_count(challenger), hand_before + 1);

    let unchallenged_fake = setup(CardKind::Tao, Suit::Spade);
    let target = unchallenged_fake.target;
    let target_hp = unchallenged_fake.state.players.get(target).unwrap().hp;
    let state = declare_sha(unchallenged_fake, [0, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn package_publishes_old_guhuo_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLD_GUHUO && skill.name == "old_guhuo"));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == 3_730_153_489 && character.name_key == "yuji"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_jianrou {
//! Golden coverage for old「剑柔」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIANROU: u32 = 4_238_996_080;
const SOURCE: &str = include_str!("../../../content/original/ported/old/jianrou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jianrou-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_811, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIANROU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    Setup {
        state: tx.commit().state,
        owner,
        source,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn prevent_with(mut setup: Setup, first: (CardKind, Suit), second: (CardKind, Suit)) -> GameState {
    let a = inject(&mut setup.state, setup.owner, 99_801, first.0, first.1);
    let b = inject(&mut setup.state, setup.owner, 99_802, second.0, second.1);
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve(tx.commit().state, setup.owner, vec![1], 1);
    let state = resolve(state, setup.owner, vec![a.raw(), b.raw()], 2);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
    state
}

#[test]
fn prevents_damage_and_only_refunds_round_use_for_same_color_or_type() {
    let refunded = setup();
    let skill = refunded.skill;
    let hand = refunded.state.hand_count(refunded.owner);
    let state = prevent_with(
        refunded,
        (CardKind::Sha, Suit::Spade),
        (CardKind::Tao, Suit::Heart),
    );
    assert_eq!(
        state.hand_count(state.skills.get(skill).unwrap().owner),
        hand + 1
    );
    assert_eq!(state.skills.get(skill).unwrap().round_fire_round, 0);

    let setup = setup();
    let round = setup.state.turn.round;
    let skill = setup.skill;
    let state = prevent_with(
        setup,
        (CardKind::Sha, Suit::Heart),
        (CardKind::Weapon, Suit::Spade),
    );
    assert_eq!(state.skills.get(skill).unwrap().round_fire_round, round);
}

#[test]
fn package_publishes_jianrou_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIANROU && skill.name == "jianrou"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_junkguixin {
//! Golden coverage for old「归心」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUNKGUIXIN: u32 = 4_267_439_357;
const LORD_SKILL: u32 = 90_272;
const SOURCE: &str = include_str!("../../../content/original/ported/old/junkguixin.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let mut package = content_compiler::compile_source_named(
        "junkguixin-test",
        &format!("{SOURCE}\nskill test_lord({LORD_SKILL}) locked lord_only {{}}"),
    )
    .unwrap();
    package.character_index.push(PackageCharacterEntry {
        id: 90_272,
        name_key: "unused_lord".into(),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![LORD_SKILL],
        sex: "male".into(),
        group: "wei".into(),
        is_lord: true,
    clans: vec![],
        tags: Vec::new(),
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: Vec::new(),
    });
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_930, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUNKGUIXIN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .character_def = 0;
    }
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveJunkguixin {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                group: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    (tx.commit().state, owner, target)
}

fn choose(state: GameState, owner: PlayerId, token: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn gains_an_unused_lord_skill_or_changes_another_players_faction() {
    let (state, owner, _) = setup();
    let state = choose(state, owner, 1, 1);
    let state = choose(state, owner, LORD_SKILL, 2);
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == LORD_SKILL)
        }));

    let (state, owner, target) = setup();
    let before = canonical_state_hash(&state);
    let state = choose(state, owner, 2, 3);
    let state = choose(state, owner, target.raw(), 4);
    let state = choose(state, owner, 4, 5);
    assert_eq!(state.player_group(target), Some("qun"));
    assert_ne!(canonical_state_hash(&state), before);
}

#[test]
fn package_publishes_junkguixin_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUNKGUIXIN && skill.name == "junkguixin"));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == 2_847_507_392 && character.name_key == "old_caocao"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_liubiao_oldzishou {
//! Golden coverage for 旧刘表「旧自守」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLD_LIUBIAO: u32 = 3_231_055_413;
const OLDZISHOU: u32 = 2_477_599_277;
const ZONGSHI: u32 = 2_639_072_445;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldzishou.skill");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&old).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&yijiang).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDZISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.character_def = OLD_LIUBIAO;
    participant.max_hp = 4;
    participant.hp = 2;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
    }
}

fn submit(
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
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![u32::from(accept)],
            cancelled: false,
        },
        key,
    )
}

fn enter_draw(state: GameState, owner: PlayerId, accept: bool) -> (GameState, PipelineResult) {
    let (state, offered) = submit(state, owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    resolve(state, owner, accept, 2)
}

#[test]
fn package_publishes_exact_abi100_skill_and_character_closure() {
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(old.manifest.rule_vm_abi_version, 275);
    assert!(old.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_LIUBIAO)
        .unwrap();
    assert_eq!(character.name_key, "old_liubiao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLDZISHOU, ZONGSHI]);

    let compiled = content_compiler::compile_source_named("oldzishou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 100);
    for expected in compiled.modules {
        let actual = old
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn declining_oldzishou_draws_normally_and_enters_play() {
    let setup = setup(2_026_074_201);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);

    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.skipped_phase_count(), 0);
}

#[test]
fn accepting_oldzishou_adds_lost_hp_and_skips_the_upcoming_play_phase() {
    let setup = setup(2_026_074_202);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 4);
    assert_eq!(state.turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS), 0);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);

    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
    assert_eq!(state.turn.skipped_phase_count(), 1);
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Play,
        } if *current == setup.owner
    )));
}
}

mod original_old_new_reyaowu {
//! Golden tests for 旧华雄「耀武」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/new_reyaowu.skill"),
    include_str!("../../../content/original/ported/refresh/reyaowu.skill")
);
const OL_HUAXIONG: u32 = 2_161_198_899;
const NEW_REYAOWU: u32 = 2_159_889_222;
const REYAOWU: u32 = 2_805_206_003;

fn loaded(seed: u64, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, target] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn inject_card(tx: &mut Transaction, source: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(69_000 + suit as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_sha(tx: &mut Transaction, source: PlayerId, suit: Suit) -> CardId {
    inject_card(tx, source, CardKind::Sha, suit)
}

fn play(source: PlayerId, target: PlayerId, card: CardId, key: u64) -> Command {
    Command {
        actor: source,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: vec![],
        },
    }
}

#[test]
fn committed_package_contains_old_huaxiong_and_both_damage_branches() {
    let package = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_HUAXIONG)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (0, 6));
    assert_eq!(character.skills, vec![NEW_REYAOWU]);
    let skills = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(skills
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NEW_REYAOWU && skill.locked));
    let filters: Vec<_> = skills
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == NEW_REYAOWU)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        [
            "owner_is_target_red_sha_source_alive",
            "owner_is_target_nonred_sha"
        ]
    );
    let filters: Vec<_> = skills
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == REYAOWU)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        [
            "owner_is_target_red_card_source_alive",
            "owner_is_target_nonred_card"
        ]
    );
}

#[test]
fn red_sha_pauses_before_damage_for_the_source_to_recover_or_draw() {
    let (mut tx, source, target) = loaded(20_260_716_1, NEW_REYAOWU);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    let source_hp = tx.state().players.get(source).unwrap().hp;
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let sha = inject_sha(&mut tx, source, Suit::Heart);

    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 1));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, source);
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);

    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![2],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, source_hp + 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn nonred_sha_draws_for_the_target_without_opening_a_choice() {
    let (mut tx, source, target) = loaded(20_260_716_2, NEW_REYAOWU);
    let hp = tx.state().players.get(target).unwrap().hp;
    let hand = tx.state().hand_count(target);
    let sha = inject_sha(&mut tx, source, Suit::Spade);
    let (state, pipeline) = apply_command(tx.state().clone(), play(source, target, sha, 3));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), hand + 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn old_reyaowu_uses_any_linked_card_and_ignores_unlinked_damage() {
    let (mut red, source, target) = loaded(20_260_716_4, REYAOWU);
    let card = inject_card(&mut red, source, CardKind::TrickHuogong, Suit::Heart);
    let source_hand = red.state().hand_count(source);
    let target_hp = red.state().players.get(target).unwrap().hp;
    run_effects(
        &mut red,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(red.state().hand_count(source), source_hand + 1);
    assert_eq!(red.state().players.get(target).unwrap().hp, target_hp - 1);

    let (mut black, source, target) = loaded(20_260_716_5, REYAOWU);
    let card = inject_card(&mut black, source, CardKind::TrickHuogong, Suit::Club);
    let target_hand = black.state().hand_count(target);
    run_effects(
        &mut black,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(black.state().hand_count(target), target_hand + 1);

    let (mut unlinked, source, target) = loaded(20_260_716_6, REYAOWU);
    let source_hand = unlinked.state().hand_count(source);
    let target_hand = unlinked.state().hand_count(target);
    run_effects(
        &mut unlinked,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(unlinked.state().hand_count(source), source_hand);
    assert_eq!(unlinked.state().hand_count(target), target_hand);
}

#[test]
fn elemental_chain_keeps_the_sha_card_provenance() {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_716_3,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let source_index = seats.iter().position(|player| *player == source).unwrap();
    let first = seats[(source_index + 1) % 3];
    let owner = seats[(source_index + 2) % 3];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    for player in [source, first, owner] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NEW_REYAOWU),
            },
            EffectRequest::AddMark {
                player: first,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: source,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject_sha(&mut tx, source, Suit::Diamond);
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::ShaFire;
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    let (state, pipeline) = apply_command(tx.state().clone(), play(source, first, sha, 4));
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, source);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);

    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}
}

mod original_old_old_jijun {
//! Golden coverage for old「集军」rewritten from legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    MatchOutcome, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLD_JIJUN: u32 = 2_612_231_702;
const OLD_FANGTONG: u32 = 2_714_288_558;
const SOURCE: &str = include_str!("../../../content/original/ported/old/old_jijun.skill");
const PAIR_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/old/old_jijun.skill"),
    include_str!("../../../content/original/ported/old/old_fangtong.skill"),
);
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

#[test]
fn selected_hand_cards_accumulate_in_the_skill_pile() {
    let package = content_compiler::compile_source_named("old-jijun-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_805, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_JIJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let cards = tx.state().hand_cards(owner);
    let state = tx.commit().state;

    let revision = state.revision;
    let (state, first) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards[..2].to_vec(),
            },
        },
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, OLD_JIJUN), cards[..2]);
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == OLD_JIJUN)
        .unwrap();
    assert_eq!(
        state.zones.get(pile.zone).unwrap().kind,
        ZoneKind::Expansion
    );

    let revision = state.revision;
    let (state, second) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![cards[2]],
            },
        },
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, OLD_JIJUN).len(), 3);
}

#[test]
fn package_publishes_old_jijun_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    for (def, name) in [(OLD_JIJUN, "old_jijun"), (OLD_FANGTONG, "old_fangtong")] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def && skill.name == name));
    }
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn thirty_six_fang_end_the_match_for_the_owners_camp() {
    let package = content_compiler::compile_source_named("old-fangtong-test", PAIR_SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_806, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for skill_def in [OLD_JIJUN, OLD_FANGTONG] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let jijun = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLD_JIJUN)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let need = 36usize.saturating_sub(tx.state().hand_count(owner) as usize);
    let cards = tx.state().zones.get(draw).unwrap().cards[..need].to_vec();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    let cards = tx.state().hand_cards(owner);
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, stored) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill: jijun,
                targets: vec![],
                card_targets: cards,
            },
        },
    );
    assert!(matches!(stored.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, OLD_JIJUN).len(), 36);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let events = run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().outcome, MatchOutcome::LoyalistWin);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::MatchEnded {
            outcome: MatchOutcome::LoyalistWin
        }
    )));
}
}

mod original_old_old_zuilun {
//! Golden coverage for 旧诸葛瞻「罪论」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLD_ZUILUN: u32 = 2_251_707_589;
const OLD_ZHUGEZHAN: u32 = 3_427_958_194;
const SOURCE: &str = include_str!("../../../content/original/ported/old/old_zuilun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    hand_card: CardId,
    equip_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("old-zuilun-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_703, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let target = tx.state().players.alive_by_seat()[1];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in [owner, target] {
        for kind in [ZoneKind::Hand, ZoneKind::Equip] {
            let zone = tx.state().zones.find(kind, player).unwrap();
            let cards = tx.state().zones.get(zone).unwrap().cards.clone();
            if !cards.is_empty() {
                effects.push(EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(zone),
                });
            }
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_ZUILUN),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLD_ZUILUN)
        .unwrap();
    let hand_card = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 1);
    let equip_card = inject(&mut tx, target, ZoneKind::Equip, CardKind::Armor, 2);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
        hand_card,
        equip_card,
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(setup: &Setup, state: GameState, key: u64) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_rewritten_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLD_ZUILUN && skill.name == "old_zuilun"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OLD_ZHUGEZHAN)
        .unwrap();
    assert_eq!(character.skills, [OLD_ZUILUN, 3_976_662_486]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn gains_from_hand_then_only_equipment_and_target_draws_after_each_gain() {
    let setup = setup();
    let owner_hand_before = setup.state.hand_count(setup.owner);
    let target_hand_before = setup.state.hand_count(setup.target);
    let (state, activated) = activate(&setup, setup.state.clone(), 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw
    );
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == setup.hand_card.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == setup.equip_card.raw()));

    let (state, gained_hand) = resolve(state, setup.owner, setup.hand_card, 2);
    assert!(matches!(gained_hand.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), owner_hand_before + 1);
    assert_eq!(state.hand_count(setup.target), target_hand_before);
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        1
    );

    let (state, activated) = activate(&setup, state, 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, setup.equip_card.raw());
    let (state, gained_equip) = resolve(state, setup.owner, setup.equip_card, 4);
    assert!(matches!(
        gained_equip.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_category_mask,
        3
    );

    let (_, third) = activate(&setup, state, 5);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
}
}

mod original_old_oldanxu {
//! Golden coverage for old「安恤」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDANXU: u32 = 2_342_781_658;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldanxu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    giver: PlayerId,
    receiver: PlayerId,
    skill: SkillId,
    card: CardId,
}

fn setup(suit: Suit) -> Setup {
    let package = content_compiler::compile_source_named("oldanxu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_804, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let &[owner, giver, receiver] = players.as_slice() else {
        unreachable!()
    };
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().zones.get(hand).unwrap().cards.clone();
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDANXU),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLDANXU)
        .unwrap();
    let card = inject(&mut tx, giver, suit, 7);
    inject(&mut tx, giver, Suit::Heart, 8);
    Setup {
        state: tx.commit().state,
        owner,
        giver,
        receiver,
        skill,
        card,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + u32::from(rank)),
        kind: CardKind::Shan,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_uses_extended_effect_tag_and_publishes_oldanxu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDANXU && skill.name == "oldanxu"));
    let module = package
        .modules
        .iter()
        .find(|module| module.name.starts_with("oldanxu_active"))
        .unwrap();
    assert!(module.code.windows(3).any(|bytes| bytes == [0x5A, 0, 1]));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn lower_hand_target_gains_card_and_non_spade_draws_for_owner() {
    let setup = setup(Suit::Club);
    let owner_before = setup.state.hand_count(setup.owner);
    let revision = setup.state.revision;
    let (state, activated) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOldAnxu
    );
    let state = choose(state, setup.owner, setup.giver.raw(), 2);
    let state = choose(state, setup.owner, setup.receiver.raw(), 3);
    let state = choose(state, setup.receiver, setup.card.raw(), 4);
    assert_eq!(state.cards.get(setup.card).unwrap().owner, setup.receiver);
    assert_eq!(state.hand_count(setup.giver), 1);
    assert_eq!(state.hand_count(setup.receiver), 1);
    assert_eq!(state.hand_count(setup.owner), owner_before + 1);
}
}

mod original_old_oldgongji {
//! Golden for 旧韩当「弓骑」的装备牌转无距离【杀】与响应语义。

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLDGONGJI: u32 = 4_171_347_410;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn package_engine(seed: u64, player_count: u8) -> Engine {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn remove_all_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDGONGJI),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLDGONGJI)
        .unwrap()
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(97_500 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: SkillId,
    target: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
    }
}

#[test]
fn package_exposes_equipment_sha_use_response_and_distance_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 5);
    for window in ["play_phase", "response"] {
        let view = package
            .view_as_index
            .iter()
            .find(|view| view.skill_def_id == OLDGONGJI && view.window == window)
            .unwrap();
        assert_eq!(view.as_kind, "sha");
        assert_eq!(view.filter, "card_equip");
        assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
        assert_eq!(view.ignore_distance, window == "play_phase");
    }
}

#[test]
fn hand_equipment_reaches_distance_two_but_non_equipment_is_rejected() {
    let engine = package_engine(20_260_716, 4);
    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    remove_all_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, far);
    let skill = add_skill(&mut tx, actor);
    let non_equip = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        2,
    );
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        3,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, far, non_equip, 1),
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(far).unwrap().hp;
    let (state, pipe) = apply_command(tx.state().clone(), activate(actor, skill, far, source, 2));
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(far).unwrap().hp, hp - 1);
}

#[test]
fn duel_offers_equipment_zone_card_as_response_sha() {
    let engine = package_engine(20_260_717, 2);
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    remove_all_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, source);
    clear_hand(&mut tx, target);
    add_skill(&mut tx, target);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Diamond,
        7,
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        8,
    );
    let non_equip = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        9,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("弓骑应提供装备牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == non_equip.raw()));
}
}

mod original_old_oldhuisheng {
//! Golden coverage for old「贿生」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDHUISHENG: u32 = 2_369_548_046;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldhuisheng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("oldhuisheng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_812, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDHUISHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        source,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn open(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    resolve(tx.commit().state, setup.owner, vec![1], false, 1)
}

#[test]
fn source_can_gain_to_prevent_and_is_then_recorded_for_the_match() {
    let setup = setup();
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let source_hand = setup.state.hand_count(setup.source);
    let offered = open(&setup);
    let cards = setup.state.hand_cards(setup.owner)[..2].to_vec();
    let state = resolve(
        offered,
        setup.owner,
        cards.iter().map(|card| card.raw()).collect(),
        false,
        2,
    );
    let state = resolve(state, setup.source, vec![cards[0].raw()], false, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(setup.source), source_hand + 1);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.source,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, hp - 1);
}

#[test]
fn source_can_discard_equal_count_and_damage_continues() {
    let setup = setup();
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let owner_card = setup.state.hand_cards(setup.owner)[0];
    let source_card = setup.state.hand_cards(setup.source)[0];
    let state = resolve(open(&setup), setup.owner, vec![owner_card.raw()], false, 10);
    let state = resolve(state, setup.source, Vec::new(), true, 11);
    let state = resolve(state, setup.source, vec![source_card.raw()], false, 12);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp - 1);
}

#[test]
fn package_publishes_oldhuisheng_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDHUISHENG && skill.name == "oldhuisheng"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_oldjiefan {
//! Golden coverage for old「解烦」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDGONGJI: u32 = 4_171_347_410;
const OLDJIEFAN: u32 = 3_185_096_783;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/old/oldgongji.skill"),
    include_str!("../../../content/original/ported/old/oldjiefan.skill")
);
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn resolve(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn oldgongji_equipment_sha_uses_normal_response_chain_then_saves_the_bound_dying_player() {
    let package = content_compiler::compile_source_named("oldjiefan-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_913, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let current = players[1];
    let dying = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let current_hand = tx.state().zones.find(ZoneKind::Hand, current).unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::MoveCards {
        cards: tx.state().hand_cards(current),
        to: discard,
        from: Some(current_hand),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDGONGJI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDJIEFAN),
    });
    run_effects(&mut tx, effects).unwrap();

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_270),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 2,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(equipment);
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(dying).unwrap().hp = 1;
    let current_hp = tx.state().players.get(current).unwrap().hp;

    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: current,
                target: dying,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve(tx.commit().state, owner, 1, 1);
    assert_eq!(
        state.resolution.pending_observer,
        owner.raw(),
        "accepting 解烦 must ask for its Sha source"
    );
    let state = resolve(state, owner, equipment.raw(), 2);

    assert_eq!(state.players.get(current).unwrap().hp, current_hp);
    assert_eq!(state.players.get(dying).unwrap().hp, 1);
    assert!(!state.players.get(dying).unwrap().dying);
    assert_eq!(
        state.cards.get(equipment).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn package_publishes_oldjiefan_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDJIEFAN && skill.name == "oldjiefan"));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == 3_688_822_524 && character.name_key == "old_handang"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_oldmingjian {
//! Golden coverage for old「明鉴」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDMINGJIAN: u32 = 2_370_380_501;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldmingjian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn gives_all_hand_then_target_gets_only_play_phase_even_turned_over() {
    let package = content_compiler::compile_source_named("oldmingjian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_809, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let successor = players[1];
    let target = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDMINGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(target).unwrap().turned_over = true;
    let owner_hand = tx.state().hand_count(owner);
    let target_hand = tx.state().hand_count(target);
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveOldMingjian {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![target.raw()],
            cancelled: false,
        },
        1,
    );

    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(target), target_hand + owner_hand);
    assert_eq!(state.turn.current, target);
    assert_eq!(state.turn.phase, Phase::Play);
    assert!(state.turn.extra_play_phase_active);
    assert!(state.players.get(target).unwrap().turned_over);

    let state = submit(state, target, CommandKind::EndPhase, 2);
    assert_eq!(state.turn.current, successor);
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert!(!state.turn.extra_play_phase_active);
    assert!(state.players.get(target).unwrap().turned_over);
}

#[test]
fn package_publishes_oldmingjian_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDMINGJIAN && skill.name == "oldmingjian"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_oldqinqing {
//! Golden coverage for old「寝情」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDQINQING: u32 = 2_296_564_419;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldqinqing.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    lord: PlayerId,
    skill: SkillId,
    discard_card: engine_api::CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("oldqinqing-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_807, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let lord = players
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let owner = players
        .iter()
        .copied()
        .find(|player| *player != lord)
        .unwrap();
    let target = players
        .iter()
        .copied()
        .find(|player| *player != lord && *player != owner)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().zones.get(hand).unwrap().cards.clone();
        effects.push(EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDQINQING),
    });
    run_effects(&mut tx, effects).unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().zones.get(draw).unwrap().cards[..3].to_vec();
    let lord_hand = tx.state().zones.find(ZoneKind::Hand, lord).unwrap();
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: cards[..1].to_vec(),
                to: lord_hand,
                from: Some(draw),
            },
            EffectRequest::MoveCards {
                cards: cards[1..].to_vec(),
                to: target_hand,
                from: Some(draw),
            },
        ],
    )
    .unwrap();
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    Setup {
        state: tx.commit().state,
        owner,
        target,
        lord,
        skill,
        discard_card: cards[1],
    }
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn discards_then_target_draws_then_owner_draws_on_hand_advantage() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveOldQinqing {
                player: setup.owner,
                target: PlayerId::NONE,
                lord: PlayerId::NONE,
                stage: 0,
                skill: setup.skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = choose(tx.commit().state, setup.owner, setup.target.raw(), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let state = choose(state, setup.owner, setup.discard_card.raw(), 2);
    assert_eq!(state.hand_count(setup.lord), 1);
    assert_eq!(state.hand_count(setup.target), 2);
    assert_eq!(state.hand_count(setup.owner), 1);
}

#[test]
fn package_publishes_oldqinqing_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDQINQING && skill.name == "oldqinqing"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_oldxuanfeng {
//! Golden coverage for old「旋风」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDXUANFENG: u32 = 2_240_977_101;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldxuanfeng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    target_hp: i32,
}

fn setup(clear_target_hand: bool) -> Setup {
    let package = content_compiler::compile_source_named("oldxuanfeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_810, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDXUANFENG),
    });
    if clear_target_hand {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(target),
            to: tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap(),
            from: tx.state().zones.find(ZoneKind::Hand, target),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let target_hp = tx.state().players.get(target).unwrap().hp;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
        target_hp,
    }
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn begin(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveOldXuanfeng {
                player: setup.owner,
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                stage: 0,
                branch: 0,
                skill: setup.skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    choose(tx.commit().state, setup.owner, setup.target.raw(), 1)
}

#[test]
fn distance_one_target_can_take_direct_damage_or_an_uncounted_virtual_sha() {
    let direct = setup(false);
    let state = choose(begin(&direct), direct.owner, 2, 2);
    assert_eq!(
        state.players.get(direct.target).unwrap().hp,
        direct.target_hp - 1
    );

    let sha = setup(true);
    let state = choose(begin(&sha), sha.owner, 1, 3);
    assert_eq!(state.players.get(sha.target).unwrap().hp, sha.target_hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
}

#[test]
fn package_publishes_oldxuanfeng_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDXUANFENG && skill.name == "oldxuanfeng"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_tianxiang {
//! Golden coverage for 天香（old/shenhua 小乔）伤害重定向（ABI 456）.
//!
//! 旧小乔受到伤害时：可选确认 → 弃一张红桃手牌 → 选择其他角色 → 原伤害取消并以
//! 同源/同量/同属性/同牌重放给新目标；新目标获得「天香·摸牌」并在伤害结算后按
//! 已损失体力值摸牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TIANXIANG: u32 = 3_182_807_327;
const TIANXIANG2: u32 = 4_111_110_111;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/tianxiang.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    if !matches!(result.result, CommandResult::Accepted { .. }) {
        panic!("choice rejected: {:?}", result);
    }
    next
}

#[test]
fn redirects_damage_to_chosen_target_and_grants_draw_skill() {
    let package = content_compiler::compile_source_named("tianxiang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 456);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, victim, source) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TIANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();

    // Give the owner a discardable heart hand card.
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .expect("owner hand zone");
    let heart = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(heart);
    assert!(tx.state().hand_count(owner) >= 1);

    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let victim_hand = tx.state().hand_count(victim);

    // Fire damage 2 to the owner: optional confirm → heart card → redirect target.
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let state = drive_choice(state, owner, 1); // optional confirm
    let state = drive_choice(state, owner, heart.raw()); // heart card cost
    let state = drive_choice(state, owner, victim.raw()); // redirect target

    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp,
        "owner damage redirected"
    );
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp - 2,
        "victim takes redirected damage"
    );
    assert_eq!(
        state.hand_count(victim),
        victim_hand + 2,
        "victim draws exactly lost-HP cards after the damage"
    );
    assert!(
        !state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == victim && skill.def.raw() == TIANXIANG2),
        "tianxiang2 fires once and removes itself"
    );
    assert!(!state.has_pending_choice());
}
}

mod original_old_weiyan_kuanggu {
//! Golden for 旧包魏延「狂骨」：近距离造成伤害后回复等量体力。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const WEIYAN: u32 = 2_400_833_780;
const KUANGGU: u32 = 3_700_078_390;
const JUEYAN_WEAPON: u32 = 3_194_432_315;
const JUEYAN_ARMOR: u32 = 4_232_768_410;
const JIEZI: u32 = 2_177_948_990;
const XUHUANG: u32 = 4_181_254_980;
const GZDUANLIANG: u32 = 2_148_140_249;
const SHENHUA_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

#[test]
fn wounded_source_recovers_only_within_distance_one() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    assert!(shenhua
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == KUANGGU && skill.locked));

    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    assert!(old.character_index.iter().any(|character| {
        character.id == WEIYAN && character.max_hp == 4 && character.skills == [KUANGGU]
    }));
    assert!(old.character_index.iter().any(|character| {
        character.id == XUHUANG && character.max_hp == 4 && character.skills == [GZDUANLIANG]
    }));

    let mut engine = Engine::new();
    // Character and skill packages are independent; either load order is valid.
    engine.load_package(&old).unwrap();
    engine.load_package(&shenhua).unwrap();
    assert!(matches!(
        engine.submit(start(4)),
        CommandResult::Accepted { .. }
    ));

    let seats = engine.state().players.alive_by_seat();
    assert_eq!(seats.len(), 4);
    // Pin the subject explicitly so adding another fully migrated old-pack character does not
    // make this skill Golden depend on character-pool sampling.
    let source_index = 0;
    let source = seats[source_index];
    let near = seats[(source_index + 1) % 4];
    let far = seats[(source_index + 2) % 4];
    let other_near = seats[(source_index + 3) % 4];
    let mut tx = Transaction::begin(engine.state().clone());
    for player in &seats {
        let existing = tx.state().players.get(*player).unwrap().skills.clone();
        for skill in existing {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = WEIYAN;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(KUANGGU),
        }],
    )
    .unwrap();
    let max_hp = tx.state().players.get(source).unwrap().max_hp as u32;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: max_hp - 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, 1);

    let budget = tx.state().resolution.trigger_budget;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: near,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, 3);
    assert_eq!(tx.state().resolution.trigger_budget, budget - 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: 2,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    let budget = tx.state().resolution.trigger_budget;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: far,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, 1);
    assert_eq!(tx.state().resolution.trigger_budget, budget);

    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source,
            player: source,
            amount: max_hp,
        }],
    )
    .unwrap();
    let budget = tx.state().resolution.trigger_budget;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: other_near,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, max_hp as i32);
    assert_eq!(tx.state().resolution.trigger_budget, budget);
}

#[test]
fn jueyan_subskills_add_three_sha_and_three_hand_limit() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&shenhua).unwrap();
    assert!(matches!(
        engine.submit(start(5)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let base_hand = match engine_core::query::query(
        tx.state(),
        &rule_api::QueryRequest::HandLimit { player: owner },
        &mut engine_core::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUEYAN_WEAPON),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUEYAN_ARMOR),
            },
        ],
    )
    .unwrap();
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 4);
    assert!(matches!(
        engine_core::query::query(
            tx.state(),
            &rule_api::QueryRequest::HandLimit { player: owner },
            &mut engine_core::query::QueryContext::new(),
        ),
        rule_api::QueryResponse::Int(value) if value == base_hand + 3
    ));
}

#[test]
fn jiezi_draws_only_when_another_players_draw_phase_is_skipped() {
    let package = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(221)),
        CommandResult::Accepted { .. }
    ));
    let current = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != current)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIEZI),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
    let before = tx.state().hand_count(owner);
    let state = tx.commit().state;
    let revision = state.revision;
    let (after, result) = apply_command(
        state,
        Command {
            actor: current,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(221),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after.hand_count(owner), before + 1);
    assert_eq!(after.turn.phase, Phase::Play);

    let mut self_skip = Transaction::begin(engine.state().clone());
    run_effects(
        &mut self_skip,
        vec![EffectRequest::AddSkill {
            player: current,
            skill_def: SkillDefId::new(JIEZI),
        }],
    )
    .unwrap();
    self_skip.state_mut().turn.phase = Phase::Judge;
    self_skip.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
    let before = self_skip.state().hand_count(current);
    let state = self_skip.commit().state;
    let revision = state.revision;
    let (after, result) = apply_command(
        state,
        Command {
            actor: current,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(222),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after.hand_count(current), before);
}
}

mod original_old_yao_yuanshu {
//! Golden coverage for 爻疑/谶纬 rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::projection::project_observer;
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YAO_YAOYI: u32 = 2_523_033_082;
const YAO_CHENWEI: u32 = 2_387_616_429;
const YAO_CHARACTER: u32 = 3_530_826_120;
const YAOYI_SOURCE: &str = include_str!("../../../content/original/ported/old/yao_yaoyi.skill");
const CHENWEI_SOURCE: &str = include_str!("../../../content/original/ported/old/yao_chenwei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    yaoyi: SkillId,
    chenwei: SkillId,
}

fn setup() -> Setup {
    let mut package = content_compiler::compile_source_named(
        "yao-yuanshu-test",
        &format!("{YAOYI_SOURCE}\n{CHENWEI_SOURCE}"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    package.character_index.push(PackageCharacterEntry {
        id: YAO_CHARACTER,
        name_key: "x_yao_yuanshu".into(),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![YAO_YAOYI, YAO_CHENWEI],
        sex: "male".into(),
        group: "qun".into(),
        is_lord: false,
    clans: vec![],
        tags: Vec::new(),
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: Vec::new(),
    });
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_932, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YAO_YAOYI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YAO_CHENWEI),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let initial_cards = tx.state().hand_cards(owner);
    tx.state_mut()
        .face_down_hand_cards
        .extend(initial_cards.into_iter().take(4));
    let skill = |def| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let yaoyi = skill(YAO_YAOYI);
    let chenwei = skill(YAO_CHENWEI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        other,
        yaoyi,
        chenwei,
    }
}

fn choose(state: GameState, token: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice key {key}: {:?}",
        result.result
    );
    state
}

fn use_as(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    kind: CardKind,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: kind.tag(),
                targets,
                equip_slot: 0,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "use key {key}: {:?}",
        result.result
    );
    state
}

fn pass_wuxie(mut state: GameState, mut key: u64) -> GameState {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        key += 1;
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }
    state
}

#[test]
fn initial_cards_are_hidden_and_each_declared_name_is_once_per_turn() {
    let mut setup = setup();
    let owner_hand = setup.state.hand_cards(setup.owner);
    let tagged: Vec<_> = owner_hand
        .iter()
        .copied()
        .filter(|card| setup.state.face_down_hand_cards.contains(card))
        .collect();
    assert_eq!(tagged.len(), 4);
    let snapshot = project_observer(&setup.state, ObserverKind::Player(setup.owner));
    assert!(tagged
        .iter()
        .all(|card| snapshot.known_cards.iter().all(|view| view.id != *card)));

    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveSkill {
            skill: setup.chenwei,
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let source = tagged[0];
    let state = use_as(
        setup.state,
        setup.owner,
        setup.yaoyi,
        CardKind::TrickWuzhong,
        Vec::new(),
        1,
    );
    let state = pass_wuxie(choose(state, source.raw(), 2), 100);
    assert!(!state.face_down_hand_cards.contains(&source));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(source).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    let revision = state.revision;
    let (_, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::UseSkillCard {
                skill: setup.yaoyi,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: Vec::new(),
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn chenwei_alternates_toggle_and_gain_face_down_branches() {
    let setup = setup();
    let tagged = setup
        .state
        .hand_cards(setup.owner)
        .into_iter()
        .filter(|card| setup.state.face_down_hand_cards.contains(card))
        .collect::<Vec<_>>();
    let untagged = setup
        .state
        .hand_cards(setup.owner)
        .into_iter()
        .find(|card| !setup.state.face_down_hand_cards.contains(card))
        .unwrap();
    let state = use_as(
        setup.state,
        setup.owner,
        setup.yaoyi,
        CardKind::TrickWuzhong,
        Vec::new(),
        10,
    );
    let state = choose(state, tagged[0].raw(), 11);
    let state = choose(state, 1, 12);
    let state = choose(state, setup.other.raw(), 13);
    let state = pass_wuxie(choose(state, untagged.raw(), 14), 200);
    assert!(state.face_down_hand_cards.contains(&untagged));
    assert_eq!(state.skills.get(setup.chenwei).unwrap().view_as_branch, 1);

    let target_card = state.hand_cards(setup.other)[0];
    let source = state
        .hand_cards(setup.owner)
        .into_iter()
        .find(|card| state.face_down_hand_cards.contains(card))
        .unwrap();
    let state = use_as(
        state,
        setup.owner,
        setup.yaoyi,
        CardKind::Jiu,
        Vec::new(),
        15,
    );
    let state = choose(state, source.raw(), 16);
    let state = choose(state, 1, 17);
    let state = choose(state, setup.other.raw(), 18);
    let state = choose(state, target_card.raw(), 19);
    assert_eq!(state.cards.get(target_card).unwrap().owner, setup.owner);
    assert!(state.face_down_hand_cards.contains(&target_card));
    assert_eq!(state.skills.get(setup.chenwei).unwrap().view_as_branch, 0);
}

#[test]
fn package_publishes_yao_pair_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YAO_YAOYI));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YAO_CHENWEI));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == YAO_CHARACTER));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
}

mod original_old_yj_y_xunxu {
//! Golden for OLD「勘律荀勖」character closure + 勘律 damage-draw path.
//!
//! Drives shipped npack + `.skill` source (not a re-implementation).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage, EffectError};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const KANLV: u32 = 2_212_101_609;
const YJSHENWEI: u32 = 3_627_315_775;
const YJ_Y_XUNXU: u32 = 3_077_663_284;

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const KANLV_SOURCE: &str = include_str!("../../../content/original/ported/old/kanlv.skill");

fn isolated(skill_def: u32, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, setup).unwrap();
    (tx, owner, other)
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_yj_y_xunxu_closure_with_both_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == KANLV && s.name == "kanlv"));
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == YJSHENWEI && s.name == "yjshenwei"));
    assert!(package
        .view_as_index
        .iter()
        .any(|v| v.skill_def_id == YJSHENWEI));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == KANLV && t.event == "damage" && t.filter == "owner_is_target" && t.optional
    }));

    let char = package
        .character_index
        .iter()
        .find(|c| c.name_key == "yj_y_xunxu" && c.id == YJ_Y_XUNXU)
        .expect("yj_y_xunxu");
    assert_eq!(char.skills, vec![KANLV, YJSHENWEI]);
    assert_eq!(
        char.portrait_asset.as_deref(),
        Some("character_yj_y_xunxu.jpg")
    );
    assert!(package
        .asset_paths
        .contains_key("character_yj_y_xunxu.jpg"));
}

#[test]
fn kanlv_source_and_npack_agree_on_optional_damage_draw() {
    let compiled = content_compiler::compile_source_named("kanlv-test", KANLV_SOURCE).unwrap();
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == KANLV && t.event == "damage" && t.filter == "owner_is_target" && t.optional
    }));

    let (mut tx, owner, other) = isolated(KANLV, 20_260_808_41);
    let before = tx.state().hand_count(owner);

    assert_eq!(
        schedule_damage(&mut tx, other, owner, 1).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 41);
    assert_eq!(state.hand_count(owner), before + 1);
    let p = state.players.get(owner).unwrap();
    assert_eq!(p.hp, p.max_hp as i32 - 1);
}

#[test]
fn yjshenwei_loads_as_view_as_from_npack() {
    let (tx, owner, _) = isolated(YJSHENWEI, 20_260_808_42);
    assert!(tx.state().players.get(owner).unwrap().skills.iter().any(|sid| {
        tx.state()
            .skills
            .get(*sid)
            .map(|s| s.def.raw() == YJSHENWEI)
            .unwrap_or(false)
    }));
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == YJSHENWEI)
        .expect("view_as");
    assert_eq!(view.as_kind.as_str(), "tao");
    assert!(view.same_suit);
    assert_eq!(view.max_cards, 2);
}
}
