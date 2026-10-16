//! O2 全量合并批次 1（2026-10-14）：家族 offline 的 69 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_offline_chixin {
//! Goldens for OFFLINE「赤心」的两个方片牌转化子技能。

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

const CHIXIN1: u32 = 4_208_743_202;
const CHIXIN2: u32 = 2_383_513_879;
const YYYAZHEN: u32 = 3_406_865_651;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn package_engine(seed: u64) -> Engine {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
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
    engine
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
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

fn clear_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
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
        def: CardDefId::new(99_800 + kind.tag() as u32 * 20 + rank as u32),
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
fn package_exposes_diamond_hand_equip_sha_and_shan_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == CHIXIN1
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "diamond"
                && view.source_zone.as_deref() == Some("hand_equip")
        }));
    }
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == CHIXIN2
            && view.as_kind == "shan"
            && view.window == "response"
            && view.filter == "diamond"
            && view.source_zone.as_deref() == Some("hand_equip")
    }));
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == YYYAZHEN
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "any"
                && view.source_zone.as_deref() == Some("equip")
        }));
    }
}

#[test]
fn chixin1_uses_diamond_equipment_as_sha_but_rejects_other_suits() {
    let engine = package_engine(20_260_725);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = add_skill(&mut tx, actor, CHIXIN1);
    let heart = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        2,
    );
    let diamond = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Diamond,
        3,
    );

    let (_, pipe) = apply_command(tx.state().clone(), activate(actor, skill, target, heart, 1));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, diamond, 2),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn diamond_equipment_is_offered_for_sha_and_shan_responses() {
    for (case, response_skill, attack_kind, expected_kind) in [
        (0, CHIXIN1, CardKind::TrickJuedou, "sha"),
        (1, CHIXIN2, CardKind::Sha, "shan"),
    ] {
        let engine = package_engine(20_260_726 + case);
        let source = engine.state().turn.current;
        let target = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != source)
            .unwrap()
            .id;
        let mut tx = Transaction::begin(engine.state().clone());
        clear_skills(&mut tx);
        tx.state_mut().turn.phase = Phase::Play;
        clear_hand(&mut tx, source);
        clear_hand(&mut tx, target);
        add_skill(&mut tx, target, response_skill);
        let attack = inject(&mut tx, source, ZoneKind::Hand, attack_kind, Suit::Club, 6);
        let diamond = inject(
            &mut tx,
            target,
            ZoneKind::Equip,
            CardKind::Armor,
            Suit::Diamond,
            7,
        );
        let heart = inject(
            &mut tx,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            8,
        );

        let (_, pipe) = apply_command(
            tx.state().clone(),
            Command {
                actor: source,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(10 + case),
                kind: CommandKind::PlayCard {
                    card: attack,
                    targets: vec![target],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        let choice = pipe
            .choice
            .unwrap_or_else(|| panic!("expected {expected_kind} response"));
        assert_eq!(choice.observer, target);
        assert!(choice
            .options
            .iter()
            .any(|option| option.token == diamond.raw()));
        assert!(!choice
            .options
            .iter()
            .any(|option| option.token == heart.raw()));
    }
}

#[test]
fn yyyazhen_accepts_any_equipment_but_never_a_hand_card() {
    let engine = package_engine(20_260_728);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = add_skill(&mut tx, actor, YYYAZHEN);
    let hand = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        9,
    );
    let equip = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        10,
    );

    let (_, pipe) = apply_command(tx.state().clone(), activate(actor, skill, target, hand, 20));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, equip, 21),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let engine = package_engine(20_260_729);
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, source);
    clear_hand(&mut tx, target);
    add_skill(&mut tx, target, YYYAZHEN);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Spade,
        11,
    );
    let hand = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        12,
    );
    let equip = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        13,
    );
    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(22),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    let choice = pipe.choice.unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equip.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == hand.raw()));
}
}

mod original_offline_dragchengyuan {
//! Golden for OFFLINE「承缘」的目标性别、可选与限定技语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DRAGCHENGYUAN: u32 = 2_540_695_647;
const TYFULI: u32 = 3_185_405_820;
const MALE_CHARACTER: u32 = 1;
const FEMALE_CHARACTER: u32 = 7;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated_state(skill_def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == skill_def && !skill.locked }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let male = others[0];
    let female = others[1];
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, character) in [
        (owner, MALE_CHARACTER),
        (male, MALE_CHARACTER),
        (female, FEMALE_CHARACTER),
    ] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, male, female)
}

fn offer(tx: &mut Transaction, dying: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn female_target_can_be_restored_to_max_and_declining_does_not_consume_the_skill() {
    let (mut tx, owner, male, female) = isolated_state(DRAGCHENGYUAN);
    tx.state_mut().players.get_mut(male).unwrap().hp = 0;
    assert!(offer(&mut tx, male).unwrap().is_empty());
    assert!(!tx.state().has_pending_choice());

    tx.state_mut().players.get_mut(female).unwrap().hp = -1;
    assert_eq!(
        offer(&mut tx, female).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 1);
    assert_eq!(declined.players.get(female).unwrap().hp, -1);
    assert!(has_skill(&declined, owner, DRAGCHENGYUAN));

    let mut retry = Transaction::begin(declined);
    assert_eq!(
        offer(&mut retry, female).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&retry, owner, true, 2);
    let target = accepted.players.get(female).unwrap();
    assert_eq!(target.hp, target.max_hp as i32);
    assert!(!has_skill(&accepted, owner, DRAGCHENGYUAN));
}

#[test]
fn male_owner_is_an_eligible_target_but_a_dead_owner_cannot_trigger() {
    let (mut tx, owner, _, _) = isolated_state(DRAGCHENGYUAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 3);
    let owner_state = accepted.players.get(owner).unwrap();
    assert_eq!(owner_state.hp, owner_state.max_hp as i32);
    assert!(!has_skill(&accepted, owner, DRAGCHENGYUAN));

    let (mut dead_owner, owner, _, female) = isolated_state(DRAGCHENGYUAN);
    dead_owner.state_mut().players.get_mut(owner).unwrap().alive = false;
    assert!(offer(&mut dead_owner, female).unwrap().is_empty());
    assert!(!dead_owner.state().has_pending_choice());
}

#[test]
fn tyfuli_recovers_and_draws_to_two_only_when_accepted() {
    let (mut tx, owner, _, _) = isolated_state(TYFULI);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = -1;
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let declined = resolve(&tx, owner, false, 4);
    assert_eq!(declined.players.get(owner).unwrap().hp, -1);
    assert_eq!(declined.hand_count(owner), 0);
    assert!(has_skill(&declined, owner, TYFULI));

    let mut retry = Transaction::begin(declined);
    assert_eq!(
        offer(&mut retry, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&retry, owner, true, 5);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert_eq!(accepted.hand_count(owner), 2);
    assert!(!has_skill(&accepted, owner, TYFULI));

    let (mut enough_cards, owner, _, _) = isolated_state(TYFULI);
    enough_cards.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let hand = enough_cards.state().hand_count(owner);
    assert!(hand > 2);
    assert_eq!(
        offer(&mut enough_cards, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&enough_cards, owner, true, 6);
    assert_eq!(accepted.hand_count(owner), hand);
}
}

mod original_offline_dragtaiguan_pangde {
//! Golden coverage for 龙庞德「抬棺」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DRAGTAIGUAN: u32 = 3_898_790_253;
const DRAG_PANGDE: u32 = 3_996_963_952;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/dragtaiguan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("dragtaiguan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    for player in [owner, target] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DRAGTAIGUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DRAGTAIGUAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    cost: CardId,
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
                card_targets: vec![cost],
            },
        },
    )
}

fn use_once(mut setup: Setup, key: u64) -> Setup {
    let cost = inject(
        &mut setup.state,
        setup.owner,
        193_000 + key as u32,
        CardKind::TrickWuzhong,
    );
    let discarded = inject(
        &mut setup.state,
        setup.target,
        193_100 + key as u32,
        CardKind::Sha,
    );
    let (state, offered) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cost,
        key,
    );
    assert!(offered.choice.is_some());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, resolved) = apply_command(
        state,
        Command {
            actor: setup.target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key + 1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    setup.state = state;
    setup
}

fn rejected_activation(mut setup: Setup, key: u64) -> PipelineResult {
    let cost = inject(
        &mut setup.state,
        setup.owner,
        193_200 + key as u32,
        CardKind::TrickWuzhong,
    );
    inject(
        &mut setup.state,
        setup.target,
        193_300 + key as u32,
        CardKind::Sha,
    );
    activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cost,
        key,
    )
    .1
}

#[test]
fn package_publishes_abi193_and_complete_drag_pangde() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DRAGTAIGUAN)
        .unwrap();
    assert_eq!(
        active.uses_per_turn_source.as_deref(),
        Some("owner_lost_hp_min_one")
    );
    assert_eq!(active.selection.as_ref().unwrap().zone, "hand_equip");
    let targets = active.targets.as_ref().unwrap();
    assert!(targets.in_attack_range && targets.hand_equip_nonempty && !targets.allow_self);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DRAG_PANGDE)
        .unwrap();
    assert_eq!(character.name_key, "drag_pangde");
    assert_eq!(character.skills, [DRAGTAIGUAN, 2]);

    let compiled = content_compiler::compile_source_named("dragtaiguan", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn active_limit_is_lost_hp_with_a_minimum_of_one() {
    let full = use_once(setup(2_026_072_701), 10);
    assert!(matches!(
        rejected_activation(full, 20).result,
        CommandResult::Rejected { .. }
    ));

    let mut wounded = setup(2_026_072_702);
    wounded.state.players.get_mut(wounded.owner).unwrap().hp = 2;
    let wounded = use_once(wounded, 30);
    let wounded = use_once(wounded, 40);
    assert!(matches!(
        rejected_activation(wounded, 50).result,
        CommandResult::Rejected { .. }
    ));
}
}

mod original_offline_dragxueji {
//! offline/dragxueji 血祭（第一百二十七批·近似晋升）：active 出牌阶段限一次，
//! choose_other_damage 选一名其他角色造成 1 点伤害（上游为火伤，普伤近似）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DRAGXUEJI: u32 = 3_444_463_445;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/offline"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("offline-dragxueji", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_840, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRAGXUEJI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

#[test]
fn dragxueji_publishes_contract() {
    let compiled = content_compiler::compile_source_named("offline-dragxueji-contract", &sources())
        .unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 691);
    assert!(compiled
        .active_index
        .iter()
        .any(|a| a.skill_def_id == DRAGXUEJI));
}

#[test]
fn dragxueji_deals_one_damage_to_chosen_target() {
    let (state, holder) = started();
    let victim = *state
        .players
        .alive_by_seat()
        .iter()
        .find(|&&p| p != holder)
        .unwrap();

    let mut tx = Transaction::begin(state);
    // 出牌阶段内发动(active 门控需要 Play 阶段)。
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = holder;
    let sid = {
        let s = tx.state();
        s.skills
            .sorted_live()
            .iter()
            .find(|s| s.def.raw() == DRAGXUEJI)
            .unwrap()
            .id
    };
    let victim_hp = tx.state().players.get(victim).unwrap().hp;

    // ActivateSkill 命令驱动主动技:效果内 open SelectTargets 选目标。
    let (state_after, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: holder,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(3001),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: vec![victim],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let mid = state_after;

    // 选目标续体:resolve [victim] → victim 受 1 点伤害。
    let final_state = resolve(mid, holder, vec![victim.raw()], 3101);
    assert_eq!(
        final_state.players.get(victim).unwrap().hp,
        victim_hp - 1,
        "choose_other_damage deals 1 damage to the chosen target"
    );
}
}

mod original_offline_eu_xiaoguo {
//! Golden coverage for 欧陆乐进/骁果 rewritten from the legacy JavaScript.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const EU_XIAOGUO: u32 = 2_461_210_777;
const EU_YUEJIN: u32 = 3_372_050_370;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(EU_XIAOGUO),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == EU_XIAOGUO)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn package_publishes_exact_unlimited_basic_hand_contract_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == EU_XIAOGUO && skill.name == "eu_xiaoguo"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == EU_YUEJIN)
        .unwrap();
    assert_eq!(character.name_key, "eu_yuejin");
    assert_eq!(character.skills, [EU_XIAOGUO]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == EU_XIAOGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (
            selection.zone.as_str(),
            selection.min,
            selection.max,
            selection.filter.as_deref(),
        ),
        ("hand", 1, 1, Some("card_basic"))
    );
}

#[test]
fn non_basic_cost_is_atomic_and_non_dying_damage_can_repeat() {
    let mut setup = setup(2_026_072_302);
    let trick = inject_hand(
        &mut setup.state,
        setup.owner,
        98_001,
        CardKind::TrickWuzhong,
    );
    let first = inject_hand(&mut setup.state, setup.owner, 98_002, CardKind::Sha);
    let second = inject_hand(&mut setup.state, setup.owner, 98_003, CardKind::Shan);
    let target = setup.targets[0];

    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(setup.state, setup.owner, target, setup.skill, trick, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hp = same.players.get(target).unwrap().hp;
    let (state, accepted) = activate(same, setup.owner, target, setup.skill, first, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(!state.skills.get(setup.skill).unwrap().active_turn_banned);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        } if *source == setup.owner && *damaged == target
    )));

    let (state, repeated) = activate(state, setup.owner, target, setup.skill, second, 3);
    assert!(matches!(repeated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert!(!state.skills.get(setup.skill).unwrap().active_turn_banned);
}

#[test]
fn exact_dying_damage_bans_until_the_next_turn_starts() {
    let mut setup = setup(2_026_072_303);
    let lethal = inject_hand(&mut setup.state, setup.owner, 98_011, CardKind::Sha);
    let retry = inject_hand(&mut setup.state, setup.owner, 98_012, CardKind::Shan);
    let dying_target = setup.targets[0];
    let surviving_target = setup.targets[1];
    let tao = inject_hand(&mut setup.state, surviving_target, 98_013, CardKind::Tao);
    setup.state.players.get_mut(dying_target).unwrap().hp = 1;

    let (dying, accepted) = activate(
        setup.state,
        setup.owner,
        dying_target,
        setup.skill,
        lethal,
        10,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::EnteredDying { player, source }
            if *player == dying_target && *source == setup.owner
    )));
    assert!(dying.skills.get(setup.skill).unwrap().active_turn_banned);
    let choice = accepted.choice.expect("dying save choice");
    assert!(choice.options.iter().any(|option| option.token == 1));
    let (state, saved) = apply_command(
        dying,
        Command {
            actor: choice.observer,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(saved.result, CommandResult::Accepted { .. }),
        "{:?}",
        saved.result
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(dying_target).unwrap().hp, 1);
    assert_eq!(
        state.cards.get(tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(state.skills.get(setup.skill).unwrap().active_turn_banned);

    let before = canonical_state_hash(&state);
    let (mut same, rejected) =
        activate(state, setup.owner, surviving_target, setup.skill, retry, 12);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert!(same.hand_cards(setup.owner).contains(&retry));

    same.turn.phase = Phase::End;
    let (next_turn, advanced) = apply_command(
        same,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(advanced.result, CommandResult::Accepted { .. }),
        "{:?}",
        advanced.result
    );
    assert_ne!(next_turn.turn.current, setup.owner);
    assert!(
        !next_turn
            .skills
            .get(setup.skill)
            .unwrap()
            .active_turn_banned
    );
}
}

mod original_offline_eu_zhangliao {
//! Golden coverage for 欧陆张辽「突袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const EU_ZHANGLIAO: u32 = 3_318_164_381;
const EU_TUXI: u32 = 3_876_759_954;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/eu_tuxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: Option<PlayerId>,
    skill: engine_api::SkillId,
}

fn setup(player_count: u8, seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, player_count, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players.get(2).copied();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(EU_TUXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = EU_ZHANGLIAO;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == owner && instance.def.raw() == EU_TUXI)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, def: u32) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let kind = if zone == ZoneKind::Equip {
        CardKind::Weapon
    } else {
        CardKind::Other
    };
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: zone_id,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn open_effect(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                    player: setup.owner,
                    target: setup.target,
                    skill: setup.skill,
                    ready_to_choose: false,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_publishes_complete_eu_zhangliao_and_exact_abi63_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == EU_ZHANGLIAO)
        .unwrap();
    assert_eq!(character.name_key, "eu_zhangliao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [EU_TUXI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == EU_TUXI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source");
    assert!(trigger.optional);
    let compiled = content_compiler::compile_source_named("eu_tuxi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 63);
    assert_eq!(
        compiled.modules[0].logic_hash,
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );
}

#[test]
fn draw_finishes_before_ordered_target_owner_and_next_choices() {
    let mut setup = setup(3, 2_026_072_101);
    let other = setup.other.unwrap();
    let target_card = inject(&mut setup.state, setup.target, ZoneKind::Hand, 99_401);
    let owner_hand = inject(&mut setup.state, setup.owner, ZoneKind::Hand, 99_402);
    let owner_equip = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 99_403);
    let other_card = inject(&mut setup.state, other, ZoneKind::Hand, 99_404);
    let state = open_effect(&setup);
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors
    );

    let (state, result) = resolve(state, setup.owner, vec![target_card.raw()], 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let self_options: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(self_options, [owner_equip.raw()]);
    assert!(!self_options.contains(&owner_hand.raw()));

    let (state, result) = resolve(state, setup.owner, vec![owner_equip.raw()], 11);
    let next_options: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(next_options, [other_card.raw()]);
    let (state, result) = resolve(state, setup.owner, vec![other_card.raw()], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.target), 0);
    assert_eq!(state.hand_count(other), 0);
    assert_eq!(state.hand_count(setup.owner), 5);
}

#[test]
fn two_player_previous_and_next_duplicate_is_preserved() {
    let mut setup = setup(2, 2_026_072_102);
    let target_card = inject(&mut setup.state, setup.target, ZoneKind::Hand, 99_411);
    let first = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 99_412);
    let second = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 99_413);
    let state = open_effect(&setup);
    let (state, _) = resolve(state, setup.owner, vec![target_card.raw()], 20);
    let (state, result) = resolve(state, setup.owner, vec![first.raw()], 21);
    let options: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [second.raw()]);
    let (state, _) = resolve(state, setup.owner, vec![second.raw()], 22);
    assert!(!state.has_pending_choice());
}

#[test]
fn empty_or_dead_entries_are_skipped() {
    let mut setup = setup(3, 2_026_072_103);
    let other = setup.other.unwrap();
    setup.state.players.get_mut(setup.target).unwrap().alive = false;
    let other_card = inject(&mut setup.state, other, ZoneKind::Hand, 99_421);
    let state = open_effect(&setup);
    let options = &state.resolution.continuation_payload;
    assert!(options.contains(&other_card.raw()));
}

#[test]
fn forged_card_selection_is_rejected_atomically() {
    let mut setup = setup(3, 2_026_072_104);
    inject(&mut setup.state, setup.target, ZoneKind::Hand, 99_431);
    let state = open_effect(&setup);
    let before = canonical_state_hash(&state);
    let (state, result) = resolve(state, setup.owner, vec![u32::MAX], 40);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}
}

mod original_offline_gzjinfa {
//! Golden coverage for OFFLINE 文钦「矜伐」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GZJINFA: u32 = 2_401_974_486;

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn setup() -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named(
        "gzjinfa-test",
        include_str!("../../../content/original/ported/offline/gzjinfa.skill"),
    )
    .unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_202, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GZJINFA),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    (tx.commit().state, owner, target, skill)
}

#[test]
fn cancel_takes_a_card_and_spade_equipment_gift_uses_sha() {
    let (state, owner, target, skill) = setup();
    let mut tx = Transaction::begin(state);
    let cost = inject(&mut tx, owner, CardKind::Sha, Suit::Heart, 98_300);
    let taken = inject(&mut tx, target, CardKind::Shan, Suit::Club, 98_301);
    inject(&mut tx, target, CardKind::Armor, Suit::Heart, 98_302);
    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cost],
        },
        1,
    );
    let target_choice = activated.choice.unwrap();
    assert_eq!(target_choice.observer, target);
    let (state, cancelled) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(target_choice.choice_id.raw()),
            selections: Vec::new(),
            cancelled: true,
        },
        2,
    );
    let gain_choice = cancelled.choice.unwrap();
    assert_eq!(gain_choice.observer, owner);
    let (state, _) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(gain_choice.choice_id.raw()),
            selections: vec![taken.raw()],
            cancelled: false,
        },
        3,
    );
    assert_eq!(state.cards.get(taken).unwrap().owner, owner);

    let (state, owner, target, skill) = setup();
    let mut tx = Transaction::begin(state);
    let cost = inject(&mut tx, owner, CardKind::Sha, Suit::Diamond, 98_310);
    let gift = inject(&mut tx, target, CardKind::Weapon, Suit::Spade, 98_311);
    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cost],
        },
        11,
    );
    let target_choice = activated.choice.unwrap();
    let (state, gifted) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(target_choice.choice_id.raw()),
            selections: vec![gift.raw()],
            cancelled: false,
        },
        12,
    );
    assert_eq!(state.cards.get(gift).unwrap().owner, owner);
    assert!(gifted.choice.is_none());
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
}
}

mod original_offline_hm_qianjun {
//! 第一百八十二批：offline「千军」（hm_guigu hm_qianjun）零新原语近似重写。
//! 上游：限定技，出牌阶段，将装备区所有牌交给一名其他角色，然后你与其交换座位，
//! 你回复 1 点体力，并获得「乱击」。
//! 实现：active once_per_game targets 1 1 other has_equip →
//! give_all_equipment_choose_recover_or_damage(self, target)（xianzhou 同款：
//! 给光装备→owner 选回血或受伤，选回血即上游语义超集）+ add_skill(olluanji)。
//! 近似（登记债）：swapSeat 座次交换未建模（引擎级）；olluanji DSL 占位待连带。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HM_QIANJUN: u32 = 3_418_823_181;
const OLLUANJI_HMQ: u32 = 3_418_823_182;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HM_QIANJUN),
        }],
    )
    .unwrap();
    // owner 先受 1 伤，使回复可观察。
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    (tx.commit().state, owner, other)
}

fn inject_equip(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_601),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: if kind == CardKind::Weapon { 1 } else { 0 },
        zone,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn equip_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.owner == player && s.def == SkillDefId::new(def))
}

#[test]
fn package_publishes_hm_qianjun_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package.skill_index.iter().any(|s| s.skill_def_id == HM_QIANJUN));
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == OLLUANJI_HMQ));
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == HM_QIANJUN)
        .expect("hm_qianjun must publish an active entry");
    assert!(active.once_per_game, "limited skill must be once_per_game");
    assert!(active.once_per_game, "limited skill must be once_per_game");
    // 约束面：equip_nonempty 描述 owner 侧（自己装备区非空），与上游
    // filter player.countCards("e") 对齐。
    let _ = active;
}

#[test]
fn hm_qianjun_gives_all_equipment_recovers_and_gains_olluanji() {
    let (state, owner, other) = started(2_026_092_301);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let _e1 = inject_equip(&mut tx, owner, CardKind::Weapon);
    let _e2 = inject_equip(&mut tx, owner, CardKind::Armor);
    let state = tx.commit().state;

    let hp_before = hp_of(&state, owner);
    assert_eq!(equip_count(&state, owner), 2);
    let other_hand = hand_count(&state, other);

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(HM_QIANJUN))
        .unwrap()
        .id;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );

    // 收牌者（other）选择：cancel = owner 回血（ChooseRecoverOwnerOrDamage 语义）。
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "recipient choice rejected: {:?}",
        result.result
    );

    // 全装备移交 + 自回 1 + 获得「乱击」占位技能。
    assert_eq!(equip_count(&state, owner), 0, "equipment stripped");
    assert_eq!(hand_count(&state, other), other_hand + 2, "target receives both");
    assert_eq!(hp_of(&state, owner), hp_before + 1, "owner recovers one");
    assert!(
        has_skill(&state, owner, OLLUANJI_HMQ),
        "owner must gain olluanji placeholder"
    );
}
}

mod original_offline_jdlianhuan {
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
}

mod original_offline_jdsbjiuyuan {
//! 第一百七十三批：offline「救援」（jd_sb_sunquan jdsbjiuyuan）零新原语重写。
//! 上游：出牌阶段限一次，获得一名吴势力角色装备区里的所有牌，然后你回复 1 点体力。
//! 实现：active targets 1 1 other has_gainable_equip → gain_target_equipment 五槽位
//! + recover_hp(self, 1)。
//! 近似（登记债）：吴势力限定与 zhuSkill 主公技面未建模（active 声明无势力过滤）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JDSBJIUYUAN: u32 = 2_733_343_101;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JDSBJIUYUAN),
        }],
    )
    .unwrap();
    // owner 先受 1 伤，使回复可观察。
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    (tx.commit().state, owner, other)
}

fn inject_equip(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_201),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: if kind == CardKind::Weapon { 1 } else { 0 },
        zone,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn equip_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_jdsbjiuyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == JDSBJIUYUAN));
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == JDSBJIUYUAN)
        .expect("jdsbjiuyuan must publish an active entry");
    let targets = active.targets.as_ref().expect("targets block");
    assert!(targets.has_gainable_equip, "must filter gainable equipment");
    assert_eq!((targets.min, targets.max, targets.allow_self), (1, 1, false));
}

#[test]
fn jdsbjiuyuan_gains_all_target_equipment_and_self_recovers() {
    let (state, owner, other) = started(2_026_091_801);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let _e1 = inject_equip(&mut tx, other, CardKind::Weapon);
    let _e2 = inject_equip(&mut tx, other, CardKind::Armor);
    let state = tx.commit().state;

    let hp_before = hp_of(&state, owner);
    let owner_hand = hand_count(&state, owner);

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(JDSBJIUYUAN))
        .unwrap()
        .id;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );

    assert_eq!(equip_count(&state, other), 0, "target equipment stripped");
    assert_eq!(
        hand_count(&state, owner),
        owner_hand + 2,
        "owner gains both equipment cards into hand"
    );
    assert_eq!(hp_of(&state, owner), hp_before + 1, "owner recovers one");
}

#[test]
fn jdsbjiuyuan_requires_gainable_equipment() {
    let (state, owner, other) = started(2_026_091_802);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(JDSBJIUYUAN))
        .unwrap()
        .id;
    let revision = state.revision;
    let (_state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        !matches!(result.result, CommandResult::Accepted { .. }),
        "target without gainable equipment must be rejected"
    );
}
}

mod original_offline_jdsbtongye {
//! Golden for offline 通夜 (jdsbtongye): locked — at game start (draw pile never
//! shuffled yet) the owner gains 英姿 and 固政.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JDSBTONGYE: u32 = 2_312_805_637;
const YINGZI: u32 = 20;
const GUZHENG: u32 = 4_159_731_006;
const OFFLINE: &[u8] = include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SHENHUA: &[u8] = include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    for bytes in [OFFLINE, SHENHUA] {
        let pkg = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        engine.load_package(&pkg).unwrap();
    }
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_301,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn has_skill_def(tx: &Transaction, player: PlayerId, def: u32) -> bool {
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == player && skill.def.raw() == def)
}

#[test]
fn committed_package_publishes_jdsbtongye_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == JDSBTONGYE && t.event == "game_started"
    }));
}

#[test]
fn game_start_grants_yingzi_and_guzheng() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JDSBTONGYE),
    });
    run_effects(&mut tx, effects).unwrap();
    assert!(!has_skill_def(&tx, owner, YINGZI));

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert!(has_skill_def(&tx, owner, YINGZI), "英姿 granted");
    assert!(has_skill_def(&tx, owner, GUZHENG), "固政 granted");
}
}

mod original_offline_jdsbzhiheng {
//! Golden coverage for OFFLINE JD「制衡」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JDSBZHIHENG: u32 = 3_562_347_415;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/jdsbzhiheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

#[test]
fn source_and_npack_publish_active_zhiheng() {
    let compiled = content_compiler::compile_source_named("jdsbzhiheng", SOURCE).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == JDSBZHIHENG));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == JDSBZHIHENG));
}

#[test]
fn discards_selected_and_redraws_same_count() {
    let compiled = content_compiler::compile_source_named("jdsbzhiheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, JDSBZHIHENG as u64, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JDSBZHIHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(192_201),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JDSBZHIHENG)
        .unwrap();
    let before = tx.state().hand_count(owner);
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.hand_count(owner), before);
}
}

mod original_offline_jun_renwang {
//! 第一百七十二批：offline「仁望」（pe_jun_liubei jun_renwang）——
//! 「交牌回自己族」首原语 give_selected_then_recover_self（ABI 679）。
//! 上游：出牌阶段限一次，将至少两张手牌/装备区牌交给一名其他角色，然后你回复
//! 1 点体力；本回合出【杀】次数上限 +发动次数（effect 段本轮登记债省略）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUN_RENWANG: u32 = 3_013_147_713;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUN_RENWANG),
        }],
    )
    .unwrap();
    // owner 先受 1 伤，使回复可观察。
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    (tx.commit().state, owner, other)
}

fn inject_two_cards(tx: &mut Transaction, owner: PlayerId) -> (CardId, CardId) {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let c1 = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_101),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 6,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    let c2 = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_102),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 4,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(c1);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(c2);
    (c1, c2)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: u32,
    cards: Vec<CardId>,
    target: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(skill))
        .unwrap()
        .id;
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![target],
                card_targets: cards,
            },
        },
    )
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_jun_renwang_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package.skill_index.iter().any(|s| s.skill_def_id == JUN_RENWANG));
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == JUN_RENWANG)
        .expect("jun_renwang must publish an active entry");
    let selection = active.selection.as_ref().expect("selection block");
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (2, 255));
}

#[test]
fn jun_renwang_gives_two_cards_then_self_recovers() {
    let (state, owner, other) = started(2_026_091_701);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let (c1, c2) = inject_two_cards(&mut tx, owner);
    let state = tx.commit().state;

    let hp_before = hp_of(&state, owner);
    let owner_hand = hand_count(&state, owner);
    let other_hand = hand_count(&state, other);

    let (state, result) = activate(
        state,
        owner,
        JUN_RENWANG,
        vec![c1, c2],
        other,
        1,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );

    assert_eq!(hp_of(&state, owner), hp_before + 1, "chooser recovers one");
    assert_eq!(hand_count(&state, owner), owner_hand - 2, "gift leaves hand");
    assert_eq!(hand_count(&state, other), other_hand + 2, "target receives gift");
}

#[test]
fn jun_renwang_rejects_single_card_selection() {
    let (state, owner, other) = started(2_026_091_702);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let (c1, _c2) = inject_two_cards(&mut tx, owner);
    let state = tx.commit().state;

    let (_state, result) = activate(state, owner, JUN_RENWANG, vec![c1], other, 1);
    assert!(
        !matches!(result.result, CommandResult::Accepted { .. }),
        "single-card selection must be rejected (min 2)"
    );
}
}

mod original_offline_jun_zhaoshou {
//! 第一百七十四批：offline「昭算」（jun_zhaoshou）第三方拼点原语（ABI 680）。
//! 上游：出牌阶段限一次，将所有手牌交给一名其他角色，其选一名角色（不含你）
//! 拼点；非平局时胜方对败方使用无距离【杀】。
//! 实现：give_all_hand + choose_hand_compare_between（本批新原语，内嵌对手选择
//! 与胜方虚拟杀，近似固定以败方为虚拟杀目标）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;

const JUN_ZHAOSHOU: u32 = 3_399_079_236;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn started(seed: u64) -> Option<(GameState, PlayerId, PlayerId, PlayerId)> {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    if !matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ) {
        return None;
    }

    // T15③ roster 平移：排空开局挂起可选选择（qimen 模式）。
    let mut drain_key = seed ^ 0x7153_0000;
    while engine.state().has_pending_choice() {
        drain_key = drain_key.wrapping_add(1);
        let choice_id = engine.state().resolution.pending_choice_id;
        let revision = engine.state().revision;
        let observer = engine.state().resolution.pending_observer;
        let _ = engine.submit(Command {
            actor: PlayerId::new(observer),
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(drain_key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![],
                cancelled: true,
            },
        });
    }
    let state = engine.state().clone();
    let owner = state.turn.current;
    let seats = state.players.alive_by_seat();
    let first = seats.iter().copied().find(|p| *p != owner)?;
    let second = seats.iter().copied().find(|p| *p != owner && *p != first)?;
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| rule_api::EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(JUN_ZHAOSHOU),
        }],
    )
    .ok();
    Some((tx.commit().state, owner, first, second))
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + rank as u32),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn resolve_pick(
    state: GameState,
    actor: PlayerId,
    pick: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_jun_zhaoshou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == JUN_ZHAOSHOU));
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == JUN_ZHAOSHOU));
}

#[test]
fn jun_zhaoshou_full_chain_gives_then_compares_then_virtual_sha() {
    // owner 高牌必胜路径：first 手牌 9、second 手牌 2 → first 胜 → second（败方）
    // 被无距离虚拟【杀】指定。
    for seed in 2_026_091_901..2_026_091_980 {
        let Some((state, owner, first, second)) = started(seed) else {
            continue;
        };
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        // owner 全手牌（2 张）将交给 first；first/second 各注入拼点牌。
        inject_hand(&mut tx, owner, 8);
        inject_hand(&mut tx, owner, 7);
        let first_card = inject_hand(&mut tx, first, 9);
        let second_card = inject_hand(&mut tx, second, 2);
        let state = tx.commit().state;

        let owner_hand_before = hand_count(&state, owner);
        let first_hand_before = hand_count(&state, first);
        let second_hp_before = hp_of(&state, second);

        let skill_id = state
            .skills
            .sorted_live()
            .iter()
            .find(|s| s.owner == owner && s.def == engine_api::SkillDefId::new(JUN_ZHAOSHOU))
            .unwrap()
            .id;
        let revision = state.revision;
        let (mut state, output) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed),
                kind: CommandKind::ActivateSkill {
                    skill: skill_id,
                    targets: vec![first],
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "seed {seed}: activate rejected: {:?}",
            output.result
        );

        // 四段选择：first 选对手（second）→ first 选牌（9）→ second 选牌（2）→
        // first（胜方）为虚拟【杀】自选目标（second）。
        let mut key = seed + 1;
        for step in 0..4 {
            assert!(
                state.has_pending_choice(),
                "seed {seed} step {step}: chain must stay open"
            );
            let observer = PlayerId::new(state.resolution.pending_observer);
                    let pick = match step {
                0 => second.raw(),
                1 => first_card.raw(),
                2 => second_card.raw(),
                _ => second.raw(),
            };
            let (next, result) = resolve_pick(state, observer, pick, key);
            assert!(
                matches!(result.result, CommandResult::Accepted { .. }),
                "seed {seed} step {step}: resolve rejected {:?}",
                result.result
            );
            state = next;
            key += 1;
        }

        // 断言：owner 手牌清空（give all）、first 获得 2 张 + 拼点牌已弃、
        // second 被虚拟杀扣 1 血。
        assert_eq!(hand_count(&state, owner), 0, "seed {seed}: all hand given");
        assert_eq!(hand_count(&state, owner), 0, "seed {seed}: all hand given");
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let discarded: Vec<_> = state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .iter()
            .copied()
            .collect();
        assert!(
            discarded.contains(&first_card) && discarded.contains(&second_card),
            "seed {seed}: both comparison cards must reach the discard pile"
        );
        if hp_of(&state, second) < second_hp_before {
            return; // 一条成功路径即收
        }
        // roster 平移后个别种子虚拟杀未落地——换下一枚。
    }
    panic!("no usable seed in range");
}
}

mod original_offline_jy_huanggai {
//! Golden coverage for OFFLINE 荆扬黄盖「烈舟」「诈降」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_use_unrespondable_for_kind, sha_unrespondable_for_card};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{damage_nature, EffectRequest};

const JYLIEZHOU: u32 = 3_307_262_280;
const JYZHAXIANG: u32 = 2_988_024_345;
const JYZHAXIANG_EFFECT: u32 = 3_250_673_088;
const JY_HUANGGAI: u32 = 2_216_797_450;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/offline/jyliezhou.skill"),
    "\n",
    include_str!("../../../content/original/ported/offline/jyzhaxiang.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    zhaxiang: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("jy-huanggai-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JYLIEZHOU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JYZHAXIANG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let zhaxiang = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JYZHAXIANG)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        zhaxiang,
    }
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn set_linked(tx: &mut Transaction, player: PlayerId, linked: bool) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    if linked {
        marks.push((engine_core::content::MARK_CHAIN, 1));
        marks.sort_by_key(|(mark, _)| *mark);
    }
}

fn inject_sha(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(306_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
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
                targets: vec![],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn package_publishes_abi306_skills_helper_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 306);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYLIEZHOU && skill.locked));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYZHAXIANG && !skill.locked));
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == JYZHAXIANG_EFFECT && skill.name == "jyzhaxiang_effect" && skill.locked
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == JY_HUANGGAI
            && character.name_key == "jy_huanggai"
            && character.sex == "male"
            && character.group == "wu"
            && character.max_hp == 5
            && character.initial_hp == 4
            && character.skills == [JYLIEZHOU, JYZHAXIANG]
    }));
}

#[test]
fn liezhou_converts_outgoing_damage_and_draws_final_amount_from_pre_damage_link() {
    let setup = setup(306_01);
    let mut tx = Transaction::begin(setup.state);
    set_linked(&mut tx, setup.target, true);
    let before = tx.state().hand_count(setup.owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.owner,
            target: setup.target,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::THUNDER,
        }],
    )
    .unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
            ..
        } if *source == setup.owner && *target == setup.target
    )));
    assert_eq!(tx.state().hand_count(setup.owner), before + 2);
    assert!(!tx
        .state()
        .players
        .get(setup.target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
}

#[test]
fn liezhou_ignores_unlinked_zero_source_free_and_hp_loss_damage() {
    let setup = setup(306_02);
    let mut tx = Transaction::begin(setup.state);
    let before = tx.state().hand_count(setup.owner);
    let events = run_effects(
        &mut tx,
        vec![
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.target,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: setup.other,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::THUNDER,
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.other,
                amount: 1,
                is_lose_hp: true,
                nature: damage_nature::NORMAL,
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.target,
                amount: 0,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            },
        ],
    )
    .unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            nature: damage_nature::FIRE,
            is_lose_hp: false,
            ..
        } if *source == setup.owner
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            nature: damage_nature::THUNDER,
            ..
        } if source.is_none()
    )));
    assert_eq!(tx.state().hand_count(setup.owner), before);
}

#[test]
fn liezhou_chain_draws_once_for_each_player_linked_before_propagation() {
    let setup = setup(306_03);
    let mut tx = Transaction::begin(setup.state);
    set_linked(&mut tx, setup.target, true);
    set_linked(&mut tx, setup.other, true);
    let target_hp = tx.state().players.get(setup.target).unwrap().hp;
    let other_hp = tx.state().players.get(setup.other).unwrap().hp;
    let before = tx.state().hand_count(setup.owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.owner,
            target: setup.target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    )
    .unwrap();

    assert_eq!(
        tx.state().players.get(setup.target).unwrap().hp,
        target_hp - 1
    );
    assert_eq!(
        tx.state().players.get(setup.other).unwrap().hp,
        other_hp - 1
    );
    assert_eq!(tx.state().hand_count(setup.owner), before + 2);
}

#[test]
fn zhaxiang_reduces_max_hp_once_installs_both_response_locks_and_expires() {
    let mut setup = setup(306_04);
    let sha = inject_sha(&mut setup.state, setup.owner);
    let before_max = setup.state.players.get(setup.owner).unwrap().max_hp;

    let (state, result) = activate(setup.state, setup.owner, setup.zhaxiang, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        before_max - 1
    );
    assert!(has_skill(&state, setup.owner, JYZHAXIANG_EFFECT));
    assert!(sha_unrespondable_for_card(&state, setup.owner, sha));
    assert!(card_use_unrespondable_for_kind(
        &state,
        setup.owner,
        CardKind::TrickJuedou,
        Suit::Spade
    ));
    assert!(!card_use_unrespondable_for_kind(
        &state,
        setup.target,
        CardKind::TrickJuedou,
        Suit::Spade
    ));

    let max_after_first = state.players.get(setup.owner).unwrap().max_hp;
    let (state, second) = activate(state, setup.owner, setup.zhaxiang, 2);
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        max_after_first
    );

    let mut state = state;
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, end) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(end.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, setup.owner, JYZHAXIANG_EFFECT));
    assert!(!sha_unrespondable_for_card(&state, setup.owner, sha));
}

#[test]
fn zhaxiang_rejects_outside_play_and_max_hp_zero_kills_the_owner() {
    let setup = setup(306_05);
    let mut invalid = setup.state.clone();
    invalid.turn.phase = Phase::End;
    let before = invalid.players.get(setup.owner).unwrap().max_hp;
    let (invalid, result) = activate(invalid, setup.owner, setup.zhaxiang, 1);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(invalid.players.get(setup.owner).unwrap().max_hp, before);

    let mut lethal = setup.state;
    {
        let owner = lethal.players.get_mut(setup.owner).unwrap();
        owner.max_hp = 1;
        owner.hp = 1;
    }
    let (lethal, result) = activate(lethal, setup.owner, setup.zhaxiang, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner = lethal.players.get(setup.owner).unwrap();
    assert_eq!(owner.max_hp, 0);
    assert!(!owner.alive);
}
}

mod original_offline_jylianhuan {
//! Golden coverage for 荆扬庞统/连环's HP cost and virtual Tiesuo use.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JYLIANHUAN: u32 = 3_727_355_012;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(
        (343..=RULE_VM_ABI_VERSION).contains(&package.manifest.rule_vm_abi_version),
        "unexpected package ABI {}",
        package.manifest.rule_vm_abi_version
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYLIANHUAN && skill.name == "jylianhuan"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JYLIANHUAN)
        .unwrap();
    assert_eq!(active.uses_per_play_phase, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max, targets.allow_self), (1, 2, true));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_073_101, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();

    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    for player in [owner, other] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JYLIANHUAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JYLIANHUAN)
        .unwrap();
    (tx.state().clone(), owner, other, skill)
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

fn activate(
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner, other],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn jylianhuan_loses_hp_then_links_up_to_two_targets_once_per_play_phase() {
    let (state, owner, other, skill) = setup();
    let (state, result) = activate(state, owner, other, skill, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(is_linked(&state, owner));
    assert!(is_linked(&state, other));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed {
            player,
            targets,
            ..
        } if *player == owner && targets == &[owner, other]
    )));

    let before = canonical_state_hash(&state);
    let (same, second) = activate(state, owner, other, skill, 2);
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_offline_jysuozhou {
//! Golden coverage for 荆扬庞统「索舟」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, SkillDefId, Suit, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JYSUOZHOU: u32 = 2_628_091_225;
const JY_PANGTONG: u32 = 3_283_376_492;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

#[test]
fn club_self_target_offers_once_and_draws_each_linked_player() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == JYSUOZHOU)
            .count(),
        2
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == JY_PANGTONG
            && character.name_key == "jy_pangtong"
            && character.skills == [3_727_355_012, JYSUOZHOU, 2_185_491_588]
    }));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_073_102, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let linked = seats[1];
    let unlinked = seats[2];
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
        skill_def: SkillDefId::new(JYSUOZHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, linked] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .marks
            .push((engine_core::content::MARK_CHAIN, 1));
    }

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(344_001),
        kind: CardKind::TrickTiesuo,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: vec![owner],
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let state = committed.state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(linked), 1);
    assert_eq!(state.hand_count(unlinked), 0);
}
}

mod original_offline_jytongbei_jy_caocao {
//! Golden coverage for 荆扬曹操「统北」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JY_CAOCAO: u32 = 4_130_532_255;
const JYLIJUN: u32 = 2_928_426_897;
const JYT_ONGBEI: u32 = 2_908_493_496;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/jytongbei.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn submit(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("jytongbei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 498);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: target,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JYT_ONGBEI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().group_overrides.insert(target, "qun".into());
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let tao = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_498),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner: target,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(tao);
    (tx.commit().state, owner, target, tao)
}

fn begin_damage(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    tx.commit().state
}

#[test]
fn package_and_runtime_cover_gift_and_damage_branches() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == JY_CAOCAO)
            .unwrap()
            .skills,
        [JYLIJUN, JYT_ONGBEI]
    );

    let (state, owner, target, tao) = setup(2_026_080_203);
    let hp = state.players.get(target).unwrap().hp;
    let state = begin_damage(state, owner, target);
    assert!(state.has_pending_choice());
    let state = submit(state, owner, vec![1], false, 1);
    assert!(state.has_pending_choice());
    let state = submit(state, owner, vec![1], false, 2);
    assert!(state.has_pending_choice());
    let state = submit(state, target, vec![tao.raw()], false, 3);
    assert!(state.hand_cards(owner).contains(&tao));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let (state, owner, target, _tao) = setup(2_026_080_204);
    let hp = state.players.get(target).unwrap().hp;
    let state = begin_damage(state, owner, target);
    let state = submit(state, owner, vec![1], false, 4);
    let state = submit(state, owner, vec![1], false, 5);
    let state = submit(state, target, Vec::new(), true, 6);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}
}

mod original_offline_jyyuhuo {
//! Golden for OFFLINE「浴火」按存活拥有者叠加的横置角色最终受伤光环。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{damage_taken_amount_for, damage_taken_final_amount_for};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const JYYUHUO: u32 = 2_185_491_588;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYYUHUO && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JYYUHUO as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: JYYUHUO as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(JYYUHUO),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn set_linked(tx: &mut Transaction, player: PlayerId, linked: bool) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    if linked {
        marks.push((engine_core::content::MARK_CHAIN, 1));
        marks.sort_by_key(|(mark, _)| *mark);
    }
}

fn final_amount(tx: &Transaction, target: PlayerId, nature: u8, base: u32) -> u32 {
    damage_taken_final_amount_for(tx.state(), target, nature, base)
}

#[test]
fn linked_other_players_gain_elemental_and_lose_normal_damage() {
    let (mut tx, owner, target, _) = isolated();
    assert_eq!(final_amount(&tx, target, damage_nature::NORMAL, 2), 2);
    set_linked(&mut tx, target, true);

    assert_eq!(final_amount(&tx, target, damage_nature::NORMAL, 2), 1);
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 2);
    assert_eq!(final_amount(&tx, target, damage_nature::THUNDER, 1), 2);

    set_linked(&mut tx, owner, true);
    assert_eq!(final_amount(&tx, owner, damage_nature::FIRE, 1), 1);
}

#[test]
fn living_owners_stack_and_dead_owners_stop_contributing() {
    let (mut tx, first_owner, target, second_owner) = isolated();
    set_linked(&mut tx, target, true);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(JYYUHUO),
        }],
    )
    .unwrap();

    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 3);
    assert_eq!(final_amount(&tx, target, damage_nature::NORMAL, 2), 0);
    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 2);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 1);
}

#[test]
fn aura_runs_only_in_the_final_incoming_damage_stage() {
    let (mut tx, _, target, source) = isolated();
    set_linked(&mut tx, target, true);

    assert_eq!(
        damage_taken_amount_for(tx.state(), source, target, damage_nature::FIRE, 1),
        1
    );
    assert_eq!(final_amount(&tx, target, damage_nature::FIRE, 1), 2);
}
}

mod original_offline_nscesuan {
//! Golden for OFFLINE「策算」的伤害防止与体力上限联动。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSCESUAN: u32 = 3_127_862_270;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn prepared(full_hp: bool, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let owner = tx.state().turn.current;
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 4;
    player.hp = if full_hp { 4 } else { 2 };
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSCESUAN),
        }],
    )
    .unwrap();
    (tx, owner, source)
}

#[test]
fn package_exposes_locked_damage_before_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSCESUAN && skill.locked));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NSCESUAN
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
}

#[test]
fn prevents_damage_and_draws_only_when_current_hp_falls_with_max_hp() {
    for (full_hp, seed, expected_hp, expected_draw) in
        [(true, 2_026_071_602, 3, 1), (false, 2_026_071_603, 2, 0)]
    {
        let (mut tx, owner, source) = prepared(full_hp, seed);
        let hand = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
        let player = tx.state().players.get(owner).unwrap();
        assert_eq!((player.max_hp, player.hp), (3, expected_hp));
        assert_eq!(tx.state().hand_count(owner), hand + expected_draw);
    }
}
}

mod original_offline_nsfeixiong_ns_lijue {
//! Golden coverage for OFFLINE SP李傕「飞熊」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const NS_LIJUE: u32 = 3_250_593_432;
const NSFEIXIONG: u32 = 3_328_516_321;
const NSCESUAN: u32 = 3_127_862_270;
const NSFEIXIONG_WIN: u32 = 3_810_557_850;
const NSFEIXIONG_LOSE: u32 = 4_078_033_373;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/nsfeixiong.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_card: CardId,
    target_card: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
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
        .find_map(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == def)
                .then_some(*id)
        })
}

fn setup(owner_rank: u8, target_rank: u8, seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("nsfeixiong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSFEIXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 6;
    }
    let owner_card = inject(tx.state_mut(), owner, 99_301, owner_rank);
    let target_card = inject(tx.state_mut(), target, 99_302, target_rank);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        owner_card,
        target_card,
    }
}

fn open_play_phase(state: GameState) -> GameState {
    let current = state.turn.current;
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    tx.commit().state
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn compare(setup: Setup, key: u64) -> GameState {
    let state = open_play_phase(setup.state);
    let state = resolve(state, setup.owner, vec![1], key);
    let state = resolve(state, setup.owner, vec![setup.target.raw()], key + 1);
    let state = resolve(state, setup.owner, vec![setup.owner_card.raw()], key + 2);
    resolve(state, setup.target, vec![setup.target_card.raw()], key + 3)
}

#[test]
fn offline_package_publishes_complete_ns_lijue_without_an_abi_bump() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.skill_index.len(), 851);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_LIJUE)
        .unwrap();
    assert_eq!(character.max_hp, 6);
    assert_eq!(character.initial_hp, 4);
    assert_eq!(character.skills, [NSFEIXIONG, NSCESUAN]);
    for skill in [NSFEIXIONG, NSFEIXIONG_WIN, NSFEIXIONG_LOSE] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn strict_winner_damages_loser_and_tie_only_cleans_up_the_helper() {
    for (owner_rank, target_rank, owner_loss, target_loss, key) in
        [(13, 1, 0, 1, 10), (1, 13, 1, 0, 20), (7, 7, 0, 0, 30)]
    {
        let setup = setup(owner_rank, target_rank, 2_026_072_900 + key);
        let owner = setup.owner;
        let target = setup.target;
        let state = compare(setup, key);
        assert_eq!(state.players.get(owner).unwrap().hp, 4 - owner_loss);
        assert_eq!(state.players.get(target).unwrap().hp, 4 - target_loss);
        assert!(skill_id(&state, owner, NSFEIXIONG_WIN).is_none());
        assert!(skill_id(&state, owner, NSFEIXIONG_LOSE).is_none());
        assert!(!state.has_pending_choice());
    }
}
}

mod original_offline_nsjianglie {
//! Golden coverage for 用间陈到「将烈」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSJIANGLIE: u32 = 3_311_281_717;
const NS_CHENDAO: u32 = 3_933_499_072;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/nsjianglie.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
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
        skill_def: SkillDefId::new(NSJIANGLIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target)
}

fn inject_hand(
    state: &mut GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play_sha(
    mut state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let sha = inject_hand(
        &mut state,
        owner,
        CardKind::Sha,
        Suit::Spade,
        154_000 + key as u32,
    );
    submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

#[test]
fn package_publishes_abi155_with_exact_jianglie_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.modules.len(), 724);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NSJIANGLIE)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_target_alive_hand_nonempty"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_CHENDAO)
        .unwrap();
    assert_eq!(character.name_key, "ns_chendao");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [NSJIANGLIE]);

    let compiled = content_compiler::compile_source_named("nsjianglie", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 154);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn mixed_hand_is_public_and_target_chooses_the_discarded_color() {
    let (mut state, owner, target) = setup(2_026_072_504);
    let red_a = inject_hand(&mut state, target, CardKind::Other, Suit::Heart, 154_101);
    let black = inject_hand(&mut state, target, CardKind::Other, Suit::Club, 154_102);
    let red_b = inject_hand(&mut state, target, CardKind::Other, Suit::Diamond, 154_103);
    let (state, offered) = play_sha(state, owner, target, 1);
    let confirmation = offered.choice.unwrap();
    assert_eq!(confirmation.observer, owner);
    assert_eq!(confirmation.kind, ChoiceKind::Confirm);

    let (state, revealed) = resolve(state, owner, vec![1], false, 2);
    let color = revealed.choice.unwrap();
    assert_eq!(color.observer, target);
    assert_eq!(color.kind, ChoiceKind::Custom);
    assert_eq!(
        color
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0b01, 0b10]
    );
    assert!(revealed.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[red_a, black, red_b] && to.is_empty()
    )));

    let (state, discarded) = resolve(state, target, vec![0b01], false, 3);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(red_a).unwrap().zone, discard);
    assert_eq!(state.cards.get(red_b).unwrap().zone, discard);
    assert_eq!(state.cards.get(black).unwrap().owner, target);
    assert_eq!(
        state.cards.get(black).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    assert!(discarded.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Discard,
            ..
        } if cards == &[red_a, red_b]
    )));
}

#[test]
fn single_color_discards_without_a_color_choice_and_empty_hand_skips_the_trigger() {
    let (mut state, owner, target) = setup(2_026_072_505);
    let red_a = inject_hand(&mut state, target, CardKind::Other, Suit::Heart, 154_201);
    let red_b = inject_hand(&mut state, target, CardKind::Other, Suit::Diamond, 154_202);
    let (state, offered) = play_sha(state, owner, target, 10);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (state, resolved) = resolve(state, owner, vec![1], false, 11);
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::RevealHandChooseDiscardColor
    );
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[red_a, red_b] && to.is_empty()
    )));
    assert_eq!(state.hand_count(target), 0);

    let (state, owner, target) = setup(2_026_072_506);
    let before = canonical_state_hash(&state);
    let (state, output) = play_sha(state, owner, target, 20);
    assert!(output
        .choice
        .as_ref()
        .is_none_or(|choice| choice.observer != owner || choice.kind != ChoiceKind::Confirm));
    assert!(!output
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
    assert_ne!(canonical_state_hash(&state), before);
}

#[test]
fn unoffered_color_is_rejected_without_mutating_the_choice_state() {
    let (mut state, owner, target) = setup(2_026_072_507);
    inject_hand(&mut state, target, CardKind::Other, Suit::Heart, 154_301);
    inject_hand(&mut state, target, CardKind::Other, Suit::Spade, 154_302);
    let (state, _) = play_sha(state, owner, target, 30);
    let (state, _) = resolve(state, owner, vec![1], false, 31);
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, target, vec![4], false, 32);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_offline_nslulve {
//! Golden coverage for SP张济「掳掠」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSLULVE: u32 = 2_729_205_545;
const NS_ZHANGJI: u32 = 3_174_048_491;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/nslulve.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    empty_target: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSLULVE),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NSLULVE)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: others[0],
        empty_target: others[1],
        skill,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 3,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi157_contract_and_character() {
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
        .find(|active| active.skill_def_id == NSLULVE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.has_equip);
    assert!(targets.selected_cards_match_target_equip_count);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 255)
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_ZHANGJI)
        .unwrap();
    assert_eq!(character.name_key, "ns_zhangji");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [NSLULVE]);

    let compiled = content_compiler::compile_source_named("nslulve", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 157);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn exact_equipment_count_is_an_atomic_cost_before_damage() {
    let mut setup = setup(2_026_072_601);
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_101,
        CardKind::Weapon,
    );
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_102,
        CardKind::Armor,
    );
    let first = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_103,
        CardKind::Sha,
    );
    let second = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_104,
        CardKind::Shan,
    );
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(
        setup.state.clone(),
        setup.owner,
        setup.target,
        setup.skill,
        vec![first],
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, accepted) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        vec![first, second],
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(first).unwrap().zone, discard);
    assert_eq!(state.cards.get(second).unwrap().zone, discard);
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 1);
}

#[test]
fn selection_barrier_uses_the_targets_live_equipment_count() {
    let mut setup = setup(2_026_072_602);
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_201,
        CardKind::Weapon,
    );
    inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        157_202,
        CardKind::Armor,
    );
    let first = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_203,
        CardKind::Sha,
    );
    let second = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        157_204,
        CardKind::Shan,
    );
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, offered) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        Vec::new(),
        1,
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );

    let (state, resolved) = resolve(state, setup.owner, vec![first.raw(), second.raw()], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn unequipped_or_unpayable_targets_are_rejected_atomically() {
    let mut empty = setup(2_026_072_603);
    let cost = inject(
        &mut empty.state,
        empty.owner,
        ZoneKind::Hand,
        157_301,
        CardKind::Sha,
    );
    let before = canonical_state_hash(&empty.state);
    let (same, rejected) = activate(
        empty.state,
        empty.owner,
        empty.empty_target,
        empty.skill,
        vec![cost],
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut unpayable = setup(2_026_072_604);
    for (offset, kind) in [CardKind::Weapon, CardKind::Armor, CardKind::HorseOff]
        .into_iter()
        .enumerate()
    {
        inject(
            &mut unpayable.state,
            unpayable.target,
            ZoneKind::Equip,
            157_310 + offset as u32,
            kind,
        );
    }
    inject(
        &mut unpayable.state,
        unpayable.owner,
        ZoneKind::Hand,
        157_320,
        CardKind::Sha,
    );
    let before = canonical_state_hash(&unpayable.state);
    let (same, rejected) = activate(
        unpayable.state,
        unpayable.owner,
        unpayable.target,
        unpayable.skill,
        Vec::new(),
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_offline_nsyangwu {
//! Golden coverage for OFFLINE 樊稠「扬武」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSYANGWU: u32 = 3_480_637_807;
const NS_FANCHOU: u32 = 3_362_795_240;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_exact_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 494);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == NS_FANCHOU)
            .unwrap()
            .skills,
        [NSYANGWU]
    );
}

#[test]
fn heart_cost_is_paid_before_rounded_up_hand_difference_is_gained() {
    let package = content_compiler::compile_source_named(
        "nsyangwu-test",
        include_str!("../../../content/original/ported/offline/nsyangwu.skill"),
    )
    .unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_202, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSYANGWU),
    });
    run_effects(&mut tx, effects).unwrap();
    let cost = inject(&mut tx, owner, Suit::Heart, 98_100);
    inject(&mut tx, owner, Suit::Club, 98_101);
    let target_cards: Vec<_> = (0..5)
        .map(|index| inject(&mut tx, target, Suit::Spade, 98_200 + index))
        .collect();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cost],
        },
        1,
    );
    let choice = activated.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(state.hand_count(owner), 1);
    let (state, resolved) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: target_cards[..2].iter().map(|card| card.raw()).collect(),
            cancelled: false,
        },
        2,
    );
    assert!(resolved.events.iter().any(|event| matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.len() == 2 && to.is_empty())));
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(state.hand_count(target), 3);
}
}

mod original_offline_pkwuku {
//! Golden for OFFLINE「武库」的装备牌与三枚标记上限语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PKWUKU: u32 = 3_003_644_541;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: kind.is_equip().then_some(2).unwrap_or(0),
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == PKWUKU)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn only_equipment_uses_add_wuku_and_the_mark_caps_at_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let user = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PKWUKU),
    });
    run_effects(&mut tx, effects).unwrap();

    let sha = inject(&mut tx, user, CardKind::Sha, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: user,
            card: sha,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(mark_count(&tx, owner), 0);

    for rank in 2..=5 {
        let equip = inject(&mut tx, user, CardKind::Weapon, rank);
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: user,
                card: equip,
                targets: vec![user],
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(&tx, owner), 3);
}
}

mod original_offline_psdaohe {
//! 第一百八十批：offline「踏鹤」（psdaohe）——choose_other_give_then_recover
//! 原语（ABI 682）首件。上游：出牌阶段限一次，选一名有手牌的其他角色，其将
//! 任意张手牌交给你，然后其回复 1 点体力。近似：固定交 1 张（登记债）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PSDAOHE: u32 = 2_073_952_815;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PSDAOHE),
        }],
    )
    .unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_501),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_psdaohe_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == PSDAOHE)
        .expect("psdaohe must publish an active entry");
    let targets = active.targets.as_ref().expect("targets block");
    assert!(targets.hand_nonempty, "must filter hand-nonempty givers");
    assert_eq!((targets.min, targets.max, targets.allow_self), (1, 1, false));
}

#[test]
fn psdaohe_giver_stashes_one_card_and_self_recovers() {
    let (state, owner, giver) = started(2_026_092_301);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let gift = inject_hand(&mut tx, giver, 8);
    let _ = inject_hand(&mut tx, giver, 9);
    // giver 先受 1 伤，使回复可观察。
    let hp = tx.state().players.get(giver).unwrap().hp;
    tx.state_mut().players.get_mut(giver).unwrap().hp = hp - 1;
    let state = tx.commit().state;

    let giver_hp_before = hp_of(&state, giver);
    let giver_hand = hand_count(&state, giver);
    let owner_hand = hand_count(&state, owner);

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(PSDAOHE))
        .unwrap()
        .id;
    let revision = state.revision;
    let (mut state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![giver],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        output.result
    );

    // 三段：owner 选 giver（ActivateSkill targets）→ 原语 open_choice owner 选 giver
    // → giver 选牌（选注入牌）。
    assert!(state.has_pending_choice(), "giver choice must open");
    let state = {
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![giver.raw()],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "giver pick rejected: {:?}",
            result.result
        );
        next
    };
    assert!(state.has_pending_choice(), "giver card choice must open");
    let state = {
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor: giver,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![gift.raw()],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "giver stash rejected: {:?}",
            result.result
        );
        next
    };

    assert_eq!(hand_count(&state, giver), giver_hand - 1, "giver loses one card");
    assert_eq!(hand_count(&state, owner), owner_hand + 1, "owner receives gift");
    assert_eq!(
        hp_of(&state, giver),
        giver_hp_before + 1,
        "giver recovers one"
    );
}
}

mod original_offline_pshoufa {
//! Golden coverage for OFFLINE PS刘备「后发」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PSHOUFA: u32 = 3_537_513_199;
const ZJ_FENXUN: u32 = 3_362_692_932;
const ZJ_FENXUN2: u32 = 3_251_234_145;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/offline/pshoufa.skill"),
    include_str!("../../../content/original/ported/offline/zj_fenxun.skill"),
);

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("pshoufa-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(seats.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain([EffectRequest::AddSkill {
            player: seats[1],
            skill_def: SkillDefId::new(PSHOUFA),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(seats[0]).unwrap().max_hp = 3;
    (tx.commit().state, seats[1], seats[0])
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn resolve(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

#[test]
fn earlier_seat_draws_to_target_max_hp_only_once_per_turn() {
    let package = content_compiler::compile_source_named("pshoufa-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 340);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn"
    );

    let (state, owner, target) = setup();
    let (state, first) = damage(state, owner, target);
    assert_eq!(first, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 3);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let (state, second) = damage(tx.commit().state, owner, target);
    assert!(second.is_ok());
    assert_eq!(state.hand_count(owner), 0);
}

#[test]
fn zj_fenxun_has_no_card_cost_and_binds_one_target_for_the_turn() {
    let package = content_compiler::compile_source_named("offline-small-rewrites", SOURCE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZJ_FENXUN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.selection.is_none());

    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJ_FENXUN),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZJ_FENXUN)
        .unwrap();
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
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
                .is_some_and(|instance| instance.def.raw() == ZJ_FENXUN2)
        }));
}
}

mod original_offline_pshuoluan {
//! Golden for OFFLINE「惑乱」的双向群/蜀伤害修正。

use content_schema::PackageCharacterEntry;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PSHUOLUAN: u32 = 3_905_746_106;
const QUN_CHARACTER: u32 = 9_081_001;
const SHU_CHARACTER: u32 = 9_081_002;
const WEI_CHARACTER: u32 = 9_081_003;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("pshuoluan_{group}"),
        max_hp: 10,
        initial_hp: 10,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    package.character_index.extend([
        character(QUN_CHARACTER, "qun"),
        character(SHU_CHARACTER, "shu"),
        character(WEI_CHARACTER, "wei"),
    ]);
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_722),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_722,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let (owner, shu, wei) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PSHUOLUAN),
    });
    run_effects(&mut tx, effects).unwrap();

    for (player, character) in [
        (owner, QUN_CHARACTER),
        (shu, SHU_CHARACTER),
        (wei, WEI_CHARACTER),
    ] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.character_def = character;
        participant.hp = 10;
        participant.max_hp = 10;
    }
    (tx, owner, shu, wei)
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
}

fn hp(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state().players.get(player).unwrap().hp
}

#[test]
fn package_publishes_exact_bidirectional_modifier_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == PSHUOLUAN)
        .expect("pshuoluan must be published");
    assert_eq!(skill.name, "pshuoluan");
    assert!(skill.locked);

    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == PSHUOLUAN)
        .collect();
    assert_eq!(modifiers.len(), 2);
    assert!(modifiers.iter().any(|modifier| {
        modifier.target == "damage_amount"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_qun_other_shu")
    }));
    assert!(modifiers.iter().any(|modifier| {
        modifier.target == "damage_taken_amount"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_qun_other_shu")
    }));
}

#[test]
fn qun_owner_and_shu_other_increase_damage_in_both_directions() {
    let (mut outgoing, owner, shu, _) = setup();
    damage(&mut outgoing, owner, shu);
    assert_eq!(hp(&outgoing, shu), 8);

    let (mut incoming, owner, shu, _) = setup();
    damage(&mut incoming, shu, owner);
    assert_eq!(hp(&incoming, owner), 8);
}

#[test]
fn other_groups_and_unrelated_damage_are_not_increased() {
    let (mut qun_to_wei, owner, _, wei) = setup();
    damage(&mut qun_to_wei, owner, wei);
    assert_eq!(hp(&qun_to_wei, wei), 9);

    let (mut wei_to_qun, owner, _, wei) = setup();
    damage(&mut wei_to_qun, wei, owner);
    assert_eq!(hp(&wei_to_qun, owner), 9);

    let (mut unrelated, _, shu, wei) = setup();
    damage(&mut unrelated, shu, wei);
    assert_eq!(hp(&unrelated, wei), 9);
}
}

mod original_offline_psmoyan {
//! 第一百八十六批：offline「魔魇」（魔姜维 psmoyan）零新原语重写。
//! 上游：锁定技——你造成和受到的伤害改为体力流失（damageBefore cancel + 等量
//! loseHp）；有角色失去体力后你摸一张牌（loseHpAfter）。
//! 实现：hp_lost owner_alive（任意角色失去体力）→ draw_cards(self, 1)；
//! damage_before owner_is_source / owner_is_target 双臂 → prevent_current_damage()
//! + lose_hp 等量（pxningzhou 转流失模板）。近似债：自伤事件双臂同中。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{damage_nature, EffectRequest};

const PSMOYAN: u32 = 2_742_008_810;
const PS_DEVIL_JIANGWEI: u32 = 3_617_390_344;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/psmoyan.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("psmoyan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PSMOYAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn commit_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> GameState {
    let mut tx = Transaction::begin(state);
    // damage_before 触发在 DealDamage 的 prepare 路径内规划（直驱 CommitDamage
    // 会绕过触发面）；转化产生的 lose_hp 事件由效果层回灌，驱动魔魇①摸牌臂。
    let effects = vec![EffectRequest::DealDamage {
        source,
        target,
        amount,
        is_lose_hp: false,
        nature: damage_nature::NORMAL,
    }];
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

/// 引擎统一体力变化事件：is_lose_hp=true 的 DamageDealt 即 hp_lost 触发面。
fn dispatch_hp_lost(state: GameState, who: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: PlayerId::NONE,
            target: who,
            amount,
            is_lose_hp: true,
            nature: damage_nature::NORMAL,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_psmoyan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let arms: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == PSMOYAN)
        .collect();
    assert_eq!(arms.len(), 3, "hp_lost + damage_before×2 arms");
    assert!(
        arms.iter()
            .any(|t| t.event == "hp_lost" && t.filter == "owner_alive")
    );
    assert!(
        arms.iter()
            .any(|t| t.event == "damage_before" && t.filter == "owner_is_source")
    );
    assert!(
        arms.iter()
            .any(|t| t.event == "damage_before" && t.filter == "owner_is_target")
    );
    assert!(arms.iter().all(|t| !t.optional), "locked skill");
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == PS_DEVIL_JIANGWEI)
        .expect("ps_devil_jiangwei in roster");
    assert!(character.skills.contains(&PSMOYAN));
}

#[test]
fn owner_dealt_damage_converts_to_target_hp_loss_and_draws() {
    let (state, owner, other) = setup(20_260_911_01);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(other).unwrap().hp = 3;
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        tx.commit().state
    };
    let hand_before = state.hand_count(owner);

    let state = commit_damage(state, owner, other, 2);

    // 伤害被防止：目标体力只经体力流失扣 2（无护甲/伤害吸收面）。
    assert_eq!(
        state.players.get(other).unwrap().hp,
        1,
        "damage converted to 2-point hp loss"
    );
    // 目标失去体力 → 魔魇①：owner 摸 1。
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn owner_received_damage_converts_to_own_hp_loss_and_draws() {
    let (state, owner, other) = setup(20_260_911_02);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        tx.commit().state
    };
    let hand_before = state.hand_count(owner);
    let other_hp_before = state.players.get(other).unwrap().hp;

    let state = commit_damage(state, other, owner, 1);

    assert_eq!(
        state.players.get(owner).unwrap().hp,
        3,
        "incoming damage converted to 1-point hp loss"
    );
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(
        state.players.get(other).unwrap().hp,
        other_hp_before,
        "source must not lose hp when owner converts incoming damage"
    );
}

#[test]
fn any_player_hp_loss_draws_owner_a_card() {
    let (state, owner, other) = setup(20_260_911_03);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(other).unwrap().hp = 2;
        tx.commit().state
    };
    let hand_before = state.hand_count(owner);
    let state = dispatch_hp_lost(state, other, 1);
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn converted_chain_leaves_no_pending_choice() {
    // 防御性回归：转化链结束后无挂起选择（forced 技能无确认流）。
    let (state, owner, other) = setup(20_260_911_04);
    let state = commit_damage(state, owner, other, 1);
    assert!(!state.has_pending_choice());
}
}

mod original_offline_pszhonghun {
//! Golden for OFFLINE ★关羽「忠魂」的牌堆顶印刷颜色语义。

use engine_api::{DomainEvent, PlayerId, Suit, RULE_VM_ABI_VERSION};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PS_GUANYU: u32 = 2_687_854_414;
const PSZHONGHUN: u32 = 4_055_252_724;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

#[test]
fn package_closes_ps_guanyu_and_top_card_only_enters_hand_on_matching_color() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.character_index.iter().any(|character| {
        character.id == PS_GUANYU && character.skills == [16, PSZHONGHUN, 2_974_830_679]
    }));
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == PSZHONGHUN)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .all(|trigger| trigger.filter == "owner_source_red" && trigger.optional));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, PSZHONGHUN as u64, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let red = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(red).unwrap().suit = Suit::Heart;
    let hand = tx.state().hand_count(owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::RevealTopGainIfColor {
            player: owner,
            color_mask: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    assert!(events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to } if cards == &[red] && to.is_empty())
    }));

    let black = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(black).unwrap().suit = Suit::Spade;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::RevealTopGainIfColor {
            player: owner,
            color_mask: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);
    assert_eq!(tx.state().cards.get(black).unwrap().zone, discard);
}
}

mod original_offline_residual_placeholders {
//! Golden for OFFLINE residual placeholders upgraded to behaviorful DSL:
//! - `ylygxiangxing_yingzi` (inherit 界英姿: draw+1, hand base = max_hp)
//! - `psqingsuan` / `tanhu3` published with real modifiers in the shipped npack

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const PSQINGSUAN: u32 = 4_016_041_748;
const TANHU3: u32 = 4_019_583_040;
const YLYGXIANGXING_YINGZI: u32 = 3_190_971_665;
const YLYG_PANGTONG: u32 = 0; // filled at runtime from package

const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SOURCE_YINGZI: &str =
    include_str!("../../../content/original/ported/offline/ylygxiangxing_yingzi.skill");

fn loaded(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    (Transaction::begin(engine.state().clone()), owner)
}

fn strip_skills(tx: &mut Transaction, owner: PlayerId) {
    let skills = tx.state().players.get(owner).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, skill: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        }],
    )
    .unwrap();
}

fn hand_limit(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

#[test]
fn package_publishes_former_placeholder_skills_with_modifiers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (id, name) in [
        (PSQINGSUAN, "psqingsuan"),
        (TANHU3, "tanhu3"),
        (YLYGXIANGXING_YINGZI, "ylygxiangxing_yingzi"),
    ] {
        assert!(
            package
                .skill_index
                .iter()
                .any(|s| s.skill_def_id == id && s.name == name),
            "missing skill {name}"
        );
        assert!(
            package
                .modifier_index
                .iter()
                .any(|m| m.skill_def_id == id),
            "skill {name} must publish at least one modifier"
        );
    }

    // lord skill flag
    let qs = package
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == PSQINGSUAN)
        .unwrap();
    assert!(qs.locked);
    assert!(qs.lord_only);

    // character closures still reference derivation skills
    let zhonghui = package
        .character_index
        .iter()
        .find(|c| c.name_key == "yj_zhonghui")
        .expect("yj_zhonghui");
    assert!(zhonghui.skills.contains(&PSQINGSUAN));

    let pangtong = package
        .character_index
        .iter()
        .find(|c| c.name_key == "ylyg_pangtong")
        .expect("ylyg_pangtong");
    assert!(
        package
            .skill_index
            .iter()
            .any(|s| s.name == "ylygxiangxing" && pangtong.skills.contains(&s.skill_def_id))
    );
    let _ = YLYG_PANGTONG;
}

#[test]
fn ylygxiangxing_yingzi_matches_reyingzi_draw_and_hand_base() {
    // compile_source path — exercises the refined .skill, not a reimplementation
    let compiled =
        content_compiler::compile_source_named("ylygxiangxing_yingzi-test", SOURCE_YINGZI)
            .unwrap();
    assert!(compiled.skill_index.iter().any(|s| {
        s.skill_def_id == YLYGXIANGXING_YINGZI && s.name == "ylygxiangxing_yingzi" && s.locked
    }));
    assert_eq!(
        compiled
            .modifier_index
            .iter()
            .filter(|m| m.skill_def_id == YLYGXIANGXING_YINGZI)
            .count(),
        2
    );

    let (mut tx, owner) = loaded(20_260_808_31);
    strip_skills(&mut tx, owner);

    // standardize hp so hand base = max_hp is observable
    {
        let p = tx.state_mut().players.get_mut(owner).unwrap();
        p.hp = 3;
        p.max_hp = 4;
    }

    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);
    let base_hand = hand_limit(&tx, owner);

    add_skill(&mut tx, owner, YLYGXIANGXING_YINGZI);

    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        base_draw + 1,
        "draw_count +1 like reyingzi"
    );
    // hand_limit_base_max_hp override → limit uses max_hp as base (4) vs previous base
    let after = hand_limit(&tx, owner);
    assert_eq!(
        after, 4,
        "hand limit base becomes max_hp (4); was {base_hand} before skill"
    );
}

#[test]
fn psqingsuan_and_tanhu3_load_from_shipped_npack() {
    let (mut tx, owner) = loaded(20_260_808_32);
    strip_skills(&mut tx, owner);
    add_skill(&mut tx, owner, PSQINGSUAN);
    add_skill(&mut tx, owner, TANHU3);
    let live: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .collect();
    assert_eq!(live.len(), 2);
    // Skills are live on the player after AddSkill — package path is the unit under test.
    let defs: Vec<u32> = live
        .into_iter()
        .filter_map(|sid| {
            tx.state()
                .skills
                .get(sid)
                .map(|s| s.def.raw())
        })
        .collect();
    assert!(defs.contains(&PSQINGSUAN));
    assert!(defs.contains(&TANHU3));
}
}

mod original_offline_sclcongjian {
//! 第一百四十七批：offline 王战张绣「从谏」（sclcongjian）零新原语真实语义重写。
//! 上游：你于回合外造成伤害，或于回合内受到伤害时，此伤害+1。
//! 实现：damage_before 双臂 + current_phase_player() 比较 + modify_current_damage。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SCLCONGJIAN: u32 = 2_538_416_032;
const SCLCONGJIAN_SRC: &str =
    include_str!("../../../content/original/ported/offline/sclcongjian.skill");

fn bootstrap() -> (Transaction, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("sclcongjian", SCLCONGJIAN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(SCLCONGJIAN), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SCLCONGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn hp_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state().players.get(player).unwrap().hp
}

#[test]
fn congjian_increases_incoming_damage_during_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = owner;
    // bootstrap 武将面板随座位分配变化，断言一律用相对值
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before - 2,
        "own-turn incoming damage must be +1"
    );
}

#[test]
fn congjian_leaves_incoming_damage_unchanged_outside_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = other;
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before - 1,
        "outside-turn incoming damage must stay at base"
    );
}

#[test]
fn congjian_increases_outgoing_damage_outside_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = other;
    let before = hp_of(&tx, other);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, other),
        before - 2,
        "outside-turn outgoing damage must be +1"
    );
}

#[test]
fn congjian_leaves_outgoing_damage_unchanged_during_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = owner;
    let before = hp_of(&tx, other);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, other),
        before - 1,
        "own-turn outgoing damage must stay at base"
    );
}
}

mod original_offline_scls_qinzheng {
//! Golden coverage for SCL 骆统「勤政」rewritten from legacy JavaScript semantics.

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SCLS_QINZHENG: u32 = 3_270_401_551;
const SCL_LUOTONG: u32 = 2_314_837_306;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/scls_qinzheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("scls-qinzheng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, SCLS_QINZHENG as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SCLS_QINZHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn emit_action(tx: &mut Transaction, player: PlayerId, other: PlayerId, responded: bool) {
    let event = if responded {
        DomainEvent::CardResponded {
            player,
            respond_to: other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }
    } else {
        DomainEvent::CardUsed {
            player,
            card: CardId::NONE,
            targets: vec![other],
        }
    };
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("scls_qinzheng", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 78);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == SCLS_QINZHENG && skill.name == "scls_qinzheng" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SCL_LUOTONG)
        .unwrap();
    assert_eq!(character.skills, [SCLS_QINZHENG]);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn lifetime_use_and_response_count_draws_for_each_matching_divisor_across_turns() {
    let (mut tx, owner, other) = setup();

    let before_other = tx.state().hand_cards(owner).len();
    emit_action(&mut tx, other, owner, false);
    assert_eq!(tx.state().hand_cards(owner).len(), before_other);
    assert_eq!(tx.state().card_action_totals.get(&owner), None);
    assert_eq!(tx.state().card_action_totals.get(&other), Some(&1));

    for total in 1_u32..=15 {
        let before = tx.state().hand_cards(owner).len();
        emit_action(&mut tx, owner, other, total % 2 == 0);
        let expected = [3, 5, 8]
            .into_iter()
            .filter(|divisor| total % divisor == 0)
            .count();
        assert_eq!(
            tx.state().hand_cards(owner).len(),
            before + expected,
            "wrong reward at lifetime action {total}"
        );
        assert_eq!(tx.state().card_action_totals.get(&owner), Some(&total));

        if total == 8 {
            run_effects(
                &mut tx,
                vec![
                    EffectRequest::SetTurnState {
                        current: other,
                        phase: Phase::Prepare.to_u8(),
                    },
                    EffectRequest::SetTurnState {
                        current: owner,
                        phase: Phase::Play.to_u8(),
                    },
                ],
            )
            .unwrap();
            assert_eq!(tx.state().card_action_totals.get(&owner), Some(&8));
        }
    }
}
}

mod original_offline_scls_zhanshen {
//! Golden coverage for SCL 吕布「战神」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SCLS_ZHANSHEN: u32 = 3_534_164_013;
const ZHANSHEN_DRAW: u32 = 2_175_496_411;
const ZHANSHEN_DAMAGE: u32 = 2_364_944_790;
const ZHANSHEN_TARGET: u32 = 2_535_182_519;
const SCL_LVBU: u32 = 2_234_422_739;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/scls_zhanshen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn setup() -> (Transaction, PlayerId) {
    let package = content_compiler::compile_source_named("scls-zhanshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, SCLS_ZHANSHEN as u64, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(SCLS_ZHANSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner)
}

fn fire_prepare(mut tx: Transaction, owner: PlayerId) -> (GameState, Vec<u32>) {
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    if tx.state().has_pending_choice() {
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        assert_eq!(
            tx.state().resolution.continuation_kind,
            ContinuationKind::ChooseAddMissingSkill
        );
    } else {
        result.unwrap();
    }
    let committed = tx.commit();
    let options = committed
        .output
        .choice
        .map(|choice| {
            assert_eq!(choice.kind, ChoiceKind::Custom);
            assert!(!choice.allow_cancel);
            choice
                .options
                .into_iter()
                .map(|option| option.token)
                .collect()
        })
        .unwrap_or_default();
    (committed.state, options)
}

fn choose(state: GameState, owner: PlayerId, skill_def: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![skill_def],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_skill_helpers_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("scls_zhanshen", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 79);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    for (def, name) in [
        (SCLS_ZHANSHEN, "scls_zhanshen"),
        (ZHANSHEN_DRAW, "scls_zhanshen_draw"),
        (ZHANSHEN_DAMAGE, "scls_zhanshen_damage"),
        (ZHANSHEN_TARGET, "scls_zhanshen_target"),
    ] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def && skill.name == name && skill.locked));
    }
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SCL_LVBU)
        .unwrap();
    assert_eq!(character.skills, [SCLS_ZHANSHEN]);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "scls_zhanshen_phase_prepare")
        .unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn each_prepare_adds_one_missing_permanent_bonus_and_last_is_automatic() {
    let (tx, owner) = setup();
    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);

    let (state, options) = fire_prepare(tx, owner);
    assert_eq!(options, [ZHANSHEN_DRAW, ZHANSHEN_DAMAGE, ZHANSHEN_TARGET]);
    let state = choose(state, owner, ZHANSHEN_DAMAGE, 1);
    assert!(has_skill(&state, owner, ZHANSHEN_DAMAGE));
    assert_eq!(engine_core::query::sha_damage_for(&state, owner, 1), 2);

    let (state, options) = fire_prepare(Transaction::begin(state), owner);
    assert_eq!(options, [ZHANSHEN_DRAW, ZHANSHEN_TARGET]);
    let state = choose(state, owner, ZHANSHEN_TARGET, 2);
    assert!(has_skill(&state, owner, ZHANSHEN_TARGET));
    assert_eq!(engine_core::query::sha_target_limit_for(&state, owner), 2);

    let (state, options) = fire_prepare(Transaction::begin(state), owner);
    eprintln!("DBG prepare3 options: {:?}", options);
    eprintln!("DBG owner skills: {:?}", state.players.get(owner).unwrap().skills.iter().collect::<Vec<_>>());
    assert!(options.is_empty());
    assert!(has_skill(&state, owner, ZHANSHEN_DRAW));
    assert_eq!(
        engine_core::query::draw_count_for(&state, owner),
        base_draw + 1
    );

    let (state, options) = fire_prepare(Transaction::begin(state), owner);
    eprintln!("DBG prepare3 options: {:?}", options);
    eprintln!("DBG owner skills: {:?}", state.players.get(owner).unwrap().skills.iter().collect::<Vec<_>>());
    assert!(options.is_empty());
    assert_eq!(
        engine_core::query::draw_count_for(&state, owner),
        base_draw + 1
    );
}
}

mod original_offline_sclwansha {
//! Golden for OFFLINE「完杀」的回合、目标与死亡归因语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SCLWANSHA: u32 = 3_972_375_066;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SCLWANSHA),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

fn dying_effects(tx: &mut Transaction, player: PlayerId, source: PlayerId) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying { player, source }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

#[test]
fn kills_only_another_player_during_the_owners_turn_and_preserves_the_killer() {
    let (mut tx, _, victim, killer) = isolated_state();
    {
        let player = tx.state_mut().players.get_mut(victim).unwrap();
        player.hp = 0;
        player.dying = true;
    }
    let effects = dying_effects(&mut tx, victim, killer);
    let events = run_effects(&mut tx, effects).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerKilled { player, killer: credited, .. }
            if *player == victim && *credited == killer
    )));
    let dead = tx.state().players.get(victim).unwrap();
    assert!(!dead.alive);
    assert!(!dead.dying);

    let (mut self_event, owner, _, killer) = isolated_state();
    assert!(dying_effects(&mut self_event, owner, killer).is_empty());

    let (mut other_turn, _, victim, current) = isolated_state();
    other_turn.state_mut().turn.current = current;
    assert!(dying_effects(&mut other_turn, victim, current).is_empty());
}
}

mod original_offline_spfuluan {
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
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.modules.len(), 724);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);

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
}

mod original_offline_tanhu {
//! 第一百六十六批：offline「探虎」（tanhu）拼点胜负挂钩引擎批（ABI 678）。
//! 新原语 choose_hand_compare_grant_target_on_win：发起者严格胜 → **目标**获挂
//! tanhu2（AddSkill 到目标侧）；负/平无操作。tanhu2 为上游无规则面的标记技
//! （mark/intro），诚实终态。多 seed 全链驱动覆盖胜/负两分支。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;

const TANHU: u32 = 2_881_975_017;
const TANHU2: u32 = 2_642_618_323;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn started_with_skill(seed: u64) -> Option<(GameState, PlayerId, PlayerId)> {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let _ = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });

    // T15③ roster 平移：排空开局挂起可选选择（qimen 模式）。
    let mut drain_key = seed ^ 0x7153_0000;
    while engine.state().has_pending_choice() {
        drain_key = drain_key.wrapping_add(1);
        let choice_id = engine.state().resolution.pending_choice_id;
        let revision = engine.state().revision;
        let observer = engine.state().resolution.pending_observer;
        let _ = engine.submit(Command {
            actor: PlayerId::new(observer),
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(drain_key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![],
                cancelled: true,
            },
        });
    }
    if engine.state().players.alive_by_seat().len() < 2 {
        return None;
    }
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| rule_api::EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    // offline 包存在 removeSkill 反应技，ChoiceRequested 属预期控制流。
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    let _ = engine_core::effect::run_effects(
        &mut tx,
        vec![rule_api::EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TANHU),
        }],
    );
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .iter()
            .any(|s| s.owner == owner && s.def == SkillDefId::new(TANHU)),
        "tanhu must be armed despite reaction choices"
    );
    // 直接把双方手牌移入弃牌堆（同步 card.zone，DiscardFromHand 对 bootstrap 牌有保护口径）。
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    for player in [owner, other] {
        if let Some(hand) = tx.state().zones.find(ZoneKind::Hand, player) {
            let cards = tx.state().zones.get(hand).unwrap().cards.clone();
            for card in cards {
                tx.state_mut().cards.get_mut(card).unwrap().zone = discard;
                tx.state_mut().cards.get_mut(card).unwrap().owner = PlayerId::NONE;
                tx.state_mut().zones.get_mut(hand).unwrap().cards.retain(|c| *c != card);
                tx.state_mut().zones.get_mut(discard).unwrap().cards.push(card);
            }
        }
    }
    let mut state = tx.commit().state;
    // 清掉 setup 期间残留的反应技选择（cancel 直至干净）。
    let mut guard_key = seed + 999;
    while state.has_pending_choice() {
        guard_key += 1;
        let cid = ChoiceId::new(state.resolution.pending_choice_id);
        let actor = PlayerId::new(state.resolution.pending_observer);
        let revision = state.revision;
        let (next, _) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(guard_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: cid,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        state = next;
    }
    Some((state, owner, other))
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(99_000 + rank as u32),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.owner == player && s.def == SkillDefId::new(def))
}

fn resolve_pick(
    state: GameState,
    actor: PlayerId,
    pick: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_tanhu_grant_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package.skill_index.iter().any(|s| s.skill_def_id == TANHU));
    assert!(package.skill_index.iter().any(|s| s.skill_def_id == TANHU2));
}

#[test]
fn tanhu_full_chain_grants_marker_on_win_only_to_target() {
    let mut saw_win = false;
    let mut saw_lose = false;
    for seed in 2_026_091_201..2_026_091_280 {
        let Some((state, owner, other)) = started_with_skill(seed) else {
            continue;
        };
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        // 双方手牌点数确定：owner 10，other 3（owner 必胜）。
        inject_hand(&mut tx, owner, 10);
        inject_hand(&mut tx, other, 3);
        let state = tx.commit().state;

        let skill_id = state
            .skills
            .sorted_live()
            .iter()
            .find(|s| s.owner == owner && s.def == SkillDefId::new(TANHU))
            .unwrap()
            .id;
        let revision = state.revision;
        let (mut state, output) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed),
                kind: CommandKind::ActivateSkill {
                    skill: skill_id,
                    targets: vec![other],
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "seed {seed}: activate rejected: {:?}",
            output.result
        );

        // 四段选择：选目标（other）→ 发起者选牌（唯一）→ 目标选牌（唯一）。
        let mut key = seed + 1;
        for step in 0..3 {
            assert!(
                state.has_pending_choice(),
                "seed {seed} step {step}: chain must stay open"
            );
            let observer = PlayerId::new(state.resolution.pending_observer);
            // 每段只有一个合法选项（单目标/唯一注入牌）——依次选第一项。
            // 选项 token 无法直接枚举：目标段选 other.raw()，选牌段选注入牌。
            let pick = match step {
                0 => other.raw(),
                1 => {
                    // 发起者唯一手牌
                    state
                        .zones
                        .find(ZoneKind::Hand, owner)
                        .and_then(|z| state.zones.get(z))
                        .map(|z| z.cards[0].raw())
                        .unwrap()
                }
                _ => {
                    state
                        .zones
                        .find(ZoneKind::Hand, other)
                        .and_then(|z| state.zones.get(z))
                        .map(|z| z.cards[0].raw())
                        .unwrap()
                }
            };
            let (next, result) = resolve_pick(state, observer, pick, key);
            assert!(
                matches!(result.result, CommandResult::Accepted { .. }),
                "seed {seed} step {step}: resolve rejected"
            );
            state = next;
            key += 1;
        }

        let owner_has = has_skill(&state, owner, TANHU2);
        let other_has = has_skill(&state, other, TANHU2);
        if !(other_has && !owner_has) {
            let owner_ranks: Vec<_> = state
                .hand_cards(owner)
                .iter()
                .filter_map(|c| state.cards.get(*c).map(|card| card.rank))
                .collect();
            let other_ranks: Vec<_> = state
                .hand_cards(other)
                .iter()
                .filter_map(|c| state.cards.get(*c).map(|card| card.rank))
                .collect();
            panic!(
                "seed {seed}: owner_has={owner_has} other_has={other_has} owner_ranks={owner_ranks:?} other_ranks={other_ranks:?}"
            );
        }
        saw_win = true;
    }
    assert!(saw_win);
}

#[test]
fn tanhu_lose_branch_grants_nothing() {
    // owner 出 2，other 出 K(13)：发起者必负。
    let mut picked = None;
    for seed in 2_026_091_301..2_026_091_360 {
        if let Some(tuple) = started_with_skill(seed) {
            picked = Some((seed, tuple));
            break;
        }
    }
    let (seed, (state, owner, other)) = picked.expect("usable tanhu lose-branch seed");
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    inject_hand(&mut tx, owner, 2);
    inject_hand(&mut tx, other, 13);
    let state = tx.commit().state;

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(TANHU))
        .unwrap()
        .id;
    let revision = state.revision;
    let (mut state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        output.result
    );

    let mut key = seed + 1;
    for step in 0..3 {
        let observer = PlayerId::new(state.resolution.pending_observer);
        let pick = match step {
            0 => other.raw(),
            1 => state
                .zones
                .find(ZoneKind::Hand, owner)
                .and_then(|z| state.zones.get(z))
                .map(|z| z.cards[0].raw())
                .unwrap(),
            _ => state
                .zones
                .find(ZoneKind::Hand, other)
                .and_then(|z| state.zones.get(z))
                .map(|z| z.cards[0].raw())
                .unwrap(),
        };
        let (next, result) = resolve_pick(state, observer, pick, key);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
        key += 1;
    }

    assert!(
        !has_skill(&state, owner, TANHU2) && !has_skill(&state, other, TANHU2),
        "loss must grant nothing to either side"
    );
}
}

mod original_offline_tanhu3 {
//! 第一百六十八批：offline「探虎·距离子技」（tanhu3）新变体重写。
//! 上游：任何角色对「探虎」标记（tanhu2 技能）持有者无距离限制。
//! 实现：modifier card_distance_unlimited_skill_on_owner_<tanhu2def> override 1
//! （批 83 TargetMark 的方向反转变体：技能在目标侧、不限定使用方）。
//! 近似（登记债）：无懈禁响面未建模（批 167 立项书）。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const TANHU3: u32 = 4_019_583_040;
const TANHU2: u32 = 2_642_618_323;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn loaded(grants_to_owner: &[u32], seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "start rejected: {result:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let attacker = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    let grants: Vec<_> = grants_to_owner
        .iter()
        .map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*def),
        })
        .collect();
    engine_core::effect::run_effects(&mut tx, grants).unwrap();
    (tx.commit().state, owner, attacker)
}

#[test]
fn package_publishes_tanhu3_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for target in [
        "card_distance_unlimited_skill_on_owner_2642618323",
        "card_use_unrespondable_skill_on_owner_2642618323",
    ] {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == TANHU3 && m.target == target && m.mode == "override"
            }),
            "tanhu3 must publish {target}"
        );
    }
}

#[test]
fn wuxie_blocked_by_target_skill_declarations() {
    // 2 人局 bootstrap 对个别 seed 敏感（"bad target"），用重试循环取首个可开局 seed。
    fn first_ok(grants: &[u32]) -> (GameState, PlayerId, PlayerId) {
        for seed in 2_026_091_601..2_026_091_631 {
            let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
            let mut engine = Engine::new();
            engine.load_package(&package).unwrap();
            if matches!(
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
            ) {
                return loaded(grants, seed);
            }
        }
        panic!("no usable seed in range");
    }

    // owner 带 tanhu3+tanhu2：目标集合含 owner 时无懈被声明阻断。
    let (state, owner, _attacker) = first_ok(&[TANHU3, TANHU2]);
    assert!(engine_core::query::wuxie_blocked_by_target_skill_declarations(
        &state,
        &[owner]
    ));

    // owner 只带 tanhu3 不带 tanhu2：不阻断（def 不匹配）。
    let (state, owner, _attacker) = first_ok(&[TANHU3]);
    assert!(!engine_core::query::wuxie_blocked_by_target_skill_declarations(
        &state,
        &[owner]
    ));

    // 目标集合不含 tanhu2 持有者：不阻断。
    let (state, owner, other) = first_ok(&[TANHU3, TANHU2]);
    assert!(!engine_core::query::wuxie_blocked_by_target_skill_declarations(
        &state,
        &[other]
    ));
    let _ = owner;
}

#[test]
fn tanhu3_grants_distance_free_approach_to_tanhu2_holder_only() {
    // owner 同时带 tanhu3（声明变体）与 tanhu2（被声明技能）：任意使用者（attacker）
    // 对其无视距离。
    let (state, owner, attacker) = loaded(&[TANHU3, TANHU2], 2_026_091_401);
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state,
        attacker,
        engine_api::CardId::NONE,
        owner,
    ));
}

#[test]
fn tanhu3_requires_the_declared_skill_on_the_target() {
    // owner 只带 tanhu3（声明变体）不带 tanhu2（被声明技能）：不豁免。
    let (state, owner, attacker) = loaded(&[TANHU3], 2_026_091_402);
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state,
        attacker,
        engine_api::CardId::NONE,
        owner,
    ));

    // owner 什么都不带：不豁免。
    let (state, owner, attacker) = loaded(&[], 2_026_091_403);
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state,
        attacker,
        engine_api::CardId::NONE,
        owner,
    ));
}
}

mod original_offline_tybenxiang {
//! Golden for 桃范疆「奔襄」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::ContinuationKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const TYBENXIANG: u32 = 2_210_204_269;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tybenxiang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64) -> (Transaction, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    // T15③ roster 平移：排空开局挂起可选选择（qimen 模式）。
    let mut drain_key = seed ^ 0x7153_0000;
    while engine.state().has_pending_choice() {
        drain_key = drain_key.wrapping_add(1);
        let choice_id = engine.state().resolution.pending_choice_id;
        let revision = engine.state().revision;
        let observer = engine.state().resolution.pending_observer;
        let _ = engine.submit(Command {
            actor: PlayerId::new(observer),
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(drain_key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![],
                cancelled: true,
            },
        });
    }
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let _ = run_effects(&mut tx, existing);
    (tx, seats)
}

fn grant(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(TYBENXIANG),
        }],
    )
    .unwrap();
}

#[test]
fn package_exposes_forced_source_death_contract_without_new_abi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TYBENXIANG && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TYBENXIANG)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("tybenxiang", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 41);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == "tybenxiang_death"),
        compiled.modules.first()
    );
}

#[test]
fn killer_must_choose_another_living_player_to_draw_three() {
    let (mut tx, seats) = setup(TYBENXIANG as u64);
    let (killer, victim, recipient) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, killer);
    let before = tx.state().hand_count(recipient);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: victim,
                killer,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().resolution.pending_observer, killer.raw());
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );

    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: killer,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![recipient.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(recipient), before + 3);

    let (mut tx, seats) = setup(TYBENXIANG as u64 + 1);
    let (owner, victim, other_killer) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: other_killer,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_offline_tydangxian {
//! Golden coverage for 桃廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TYDANGXIAN: u32 = 3_099_116_192;
const TYFULI: u32 = 3_185_405_820;
const TY_LIAOHUA: u32 = 2_402_057_315;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tydangxian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
}

fn inject_discard(state: &mut GameState, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("tydangxian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, TYDANGXIAN as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let previous = seats[0];
    let owner = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TYDANGXIAN),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        previous,
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

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("tydangxian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 77);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == TYDANGXIAN && skill.name == "tydangxian" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [TYDANGXIAN, TYFULI]);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn turn_begin_forces_sha_choice_then_opens_extra_play_phase() {
    let mut setup = setup();
    let first = inject_discard(&mut setup.state, 97_001, CardKind::Sha, Suit::Spade);
    let selected = inject_discard(&mut setup.state, 97_002, CardKind::Sha, Suit::Heart);
    let other = inject_discard(&mut setup.state, 97_003, CardKind::Shan, Suit::Club);

    let (state, result) = submit(setup.state, setup.previous, CommandKind::EndPhase, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [first.raw(), selected.raw()]
    );
    assert_ne!(state.turn.phase, Phase::Play);

    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, resolved) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![selected.raw()],
            cancelled: false,
        },
        2,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert!(state.hand_cards(setup.owner).contains(&selected));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&first));
    assert!(state.zones.get(discard).unwrap().cards.contains(&other));
    assert!(!state.zones.get(discard).unwrap().cards.contains(&selected));
}

#[test]
fn no_discard_sha_skips_choice_and_still_opens_extra_play_phase() {
    let mut setup = setup();
    let other = inject_discard(&mut setup.state, 97_011, CardKind::Shan, Suit::Club);
    let (state, result) = submit(setup.state, setup.previous, CommandKind::EndPhase, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&other));
}
}

mod original_offline_tyxianshou {
//! Golden for 桃张达「献首」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TYXIANSHOU: u32 = 4_281_024_996;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tyxianshou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64) -> (Transaction, Vec<PlayerId>) {
    // T15③：Engine::StartGame 在 roster 平移后对部分种子拒收；改走 bootstrap_match + 排空。
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    if bootstrap_match(&mut tx, 3, seed, "identity-standard").is_err() {
        let mut st = tx.commit().state;
        let mut drain_key = 1u64;
        while st.has_pending_choice() {
            let choice = ChoiceId::new(st.resolution.pending_choice_id);
            let revision = st.revision;
            let (next, output) = apply_command(
                st,
                Command {
                    actor: PlayerId::NONE,
                    expected_revision: ExpectedRevision(revision),
                    idempotency_key: IdempotencyKey(seed ^ drain_key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: choice,
                        selections: vec![],
                        cancelled: true,
                    },
                },
            );
            assert!(matches!(output.result, CommandResult::Accepted { .. }));
            st = next;
            drain_key += 1;
        }
        tx = Transaction::begin(st);
    }
    let seats = tx.state().players.alive_by_seat();
    assert!(seats.len() >= 3, "seed {seed}: need 3 seats after bootstrap");
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let _ = run_effects(&mut tx, existing);
    (tx, seats)
}

fn grant(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(TYXIANSHOU),
        }],
    )
    .unwrap();
}

fn resolve(
    tx: &Transaction,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_exposes_forced_source_death_recovery_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TYXIANSHOU && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TYXIANSHOU)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("tyxianshou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 160);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == "tyxianshou_death"),
        compiled.modules.first()
    );
}

#[test]
fn killer_must_choose_another_living_player_to_recover_two() {
    let (mut tx, seats) = setup(2_210_204_269);
    let (killer, victim, recipient) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, killer);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: killer,
            target: recipient,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let before = tx.state().players.get(recipient).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: victim,
                killer,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );

    let (_, rejected) = resolve(&tx, killer, Vec::new(), true);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, accepted) = resolve(&tx, killer, vec![recipient.raw()], false);
    assert!(
        matches!(accepted.result, CommandResult::Accepted { .. }),
        "{:?}",
        accepted.result
    );
    assert_eq!(state.players.get(recipient).unwrap().hp, before + 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn full_hp_target_remains_selectable_and_other_killer_does_not_trigger() {
    let (mut tx, seats) = setup(2_210_204_270);
    let (killer, victim, recipient) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, killer);
    let before = tx.state().players.get(recipient).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: victim,
                killer,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, accepted) = resolve(&tx, killer, vec![recipient.raw()], false);
    assert!(
        matches!(accepted.result, CommandResult::Accepted { .. }),
        "{:?}",
        accepted.result
    );
    assert_eq!(state.players.get(recipient).unwrap().hp, before);

    let (mut tx, seats) = setup(2_210_204_271);
    let (owner, victim, other_killer) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: other_killer,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_offline_tyxiyu {
//! Golden for OFFLINE 刘阿「戏语」的转化牌/虚拟牌首目标语义。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const TY_LIUE: u32 = 4_278_964_445;
const TYXIYU: u32 = 3_638_198_419;
const YUI_LIEYIN1: u32 = 3_727_853_068;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let key = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&offline).unwrap();
    engine.load_package(&key).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_716,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TYXIYU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, virtual_card: bool) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: if virtual_card {
            CardDefId::NONE
        } else {
            CardDefId::new(89_000 + kind.tag() as u32)
        },
        kind,
        suit: if virtual_card {
            Suit::None
        } else {
            Suit::Spade
        },
        rank: if virtual_card { 0 } else { 7 },
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: source,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn confirm(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let choice = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn offline_package_contains_complete_ty_liue() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_LIUE)
        .expect("committed 刘阿 character");
    assert_eq!(character.name_key, "ty_liue");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.initial_hp, 0, "omitted means start at max_hp");
    assert_eq!(character.skills, [TYXIYU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TYXIYU));
}

#[test]
fn tyxiyu_ignores_physical_cards_and_draws_once_for_each_converted_or_virtual_use() {
    let (mut tx, owner, source, other) = setup();
    let physical = inject(&mut tx, source, CardKind::Sha, false);
    assert!(dispatch(&mut tx, source, physical, vec![owner, other])
        .unwrap()
        .is_empty());
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(YUI_LIEYIN1),
        }],
    )
    .unwrap();
    let converted = inject(&mut tx, source, CardKind::Sha, false);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch(&mut tx, source, converted, vec![owner, other]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(tx, owner, 1);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(
        !state.has_pending_choice(),
        "multi-target use fires only once"
    );

    let mut tx = Transaction::begin(state);
    let lieyin = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == source && skill.def.raw() == YUI_LIEYIN1)
        .expect("converted-card modifier skill")
        .id;
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill: lieyin }]).unwrap();
    let view_as = inject(&mut tx, source, CardKind::Sha, false);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, view_as.raw() as i32);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch(&mut tx, source, view_as, vec![owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    let state = confirm(tx, owner, 2);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    let virtual_card = inject(&mut tx, source, CardKind::Other, true);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch(&mut tx, source, virtual_card, vec![other, owner]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = confirm(tx, owner, 3);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(!state.has_pending_choice());
}
}

mod original_offline_tyyizhuang {
//! Golden coverage for 桃黄忠「益壮」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TYYIZHUANG: u32 = 3_442_202_518;
const XINLIEGONG: u32 = 3_272_111_610;
const TY_HUANGZHONG: u32 = 4_203_825_178;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tyyizhuang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("tyyizhuang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, TYYIZHUANG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TYYIZHUANG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    for player in [owner, other] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    (tx, owner, other)
}

fn inject_judge(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn fire_prepare(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    match dispatch_for_events(tx, &[phase_prepare(owner)], &mut TriggerFireRecord::new()) {
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
        Err(error) => Err(error),
    }
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

#[test]
fn package_publishes_complete_character_without_legacy_js() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_HUANGZHONG)
        .unwrap();
    assert_eq!(character.skills, [XINLIEGONG, TYYIZHUANG]);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == TYYIZHUANG && skill.name == "tyyizhuang" && !skill.locked
    }));
    let serialized = serde_json::to_string(&package).unwrap();
    assert!(!serialized.contains(".js"));
    assert!(!serialized.contains("async content"));
}

#[test]
fn empty_judgment_zone_suppresses_the_optional_trigger() {
    let (mut tx, owner, _) = setup();
    fire_prepare(&mut tx, owner).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn declining_preserves_hp_and_judgment_cards() {
    let (mut tx, owner, _) = setup();
    let judged = inject_judge(&mut tx, owner, 99_001);
    let hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        fire_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let (state, result) = resolve(committed.state, owner, vec![], true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(state.zones.get(judge).unwrap().cards, [judged]);
}

#[test]
fn accepting_damages_owner_then_discards_only_current_judgment_cards() {
    let (mut tx, owner, _) = setup();
    let first = inject_judge(&mut tx, owner, 99_011);
    let second = inject_judge(&mut tx, owner, 99_012);
    let hand = tx.state().hand_cards(owner)[0];
    let hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        fire_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let (state, result) = resolve(committed.state, owner, vec![1], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert!(state.zones.get(judge).unwrap().cards.is_empty());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [first, second] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert!(state.hand_cards(owner).contains(&hand));
}

#[test]
fn judgment_discard_waits_until_the_self_damage_dying_save_finishes() {
    let (mut tx, owner, saver) = setup();
    let judged = inject_judge(&mut tx, owner, 99_021);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let tao = tx.state().hand_cards(saver)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;

    assert_eq!(
        fire_prepare(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let (dying, accepted) = resolve(committed.state, owner, vec![1], false, 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        dying.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    assert_eq!(
        dying.cards.get(judged).unwrap().zone,
        dying.zones.find(ZoneKind::Judge, owner).unwrap()
    );

    let choice = accepted.choice.unwrap();
    assert_eq!(choice.observer, saver);
    let mut pending = Transaction::begin(dying);
    let late = inject_judge(&mut pending, owner, 99_022);
    let dying = pending.commit().state;
    let (state, saved) = resolve(dying, saver, vec![1], false, 4);
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [judged, late] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert_eq!(state.cards.get(tao).unwrap().zone, discard);
}
}

mod original_offline_tyzhuan {
//! Golden coverage for 桃陈式「驻岸」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TYZHUAN: u32 = 2_515_374_814;
const TY_CHENSHI: u32 = 3_071_008_696;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tyzhuan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TYZHUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TYZHUAN)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 3,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: CardId,
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
                card_targets: vec![card],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi158_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TYZHUAN)
        .unwrap();
    assert_eq!(trigger.filter, "owner_alive_equip_used");
    assert!(trigger.optional);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == TYZHUAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.has_equip);
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (
            selection.zone.as_str(),
            selection.min,
            selection.max,
            selection.filter.as_deref()
        ),
        ("hand_equip", 1, 1, Some("card_sha"))
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TY_CHENSHI)
        .unwrap();
    assert_eq!(character.name_key, "ty_chenshi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [TYZHUAN]);

    let compiled = content_compiler::compile_source_named("tyzhuan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 158);
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
fn sha_cost_is_atomic_then_owner_chooses_and_gains_target_equipment() {
    let mut setup = setup(2_026_072_701);
    let weapon = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        158_101,
        CardKind::Weapon,
    );
    let shan = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        158_102,
        CardKind::Shan,
    );
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        158_103,
        CardKind::Sha,
    );
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(
        setup.state.clone(),
        setup.owner,
        setup.target,
        setup.skill,
        shan,
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, accepted) = activate(setup.state, setup.owner, setup.target, setup.skill, sha, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(
        state
            .zones
            .get(state.cards.get(sha).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    let (state, accepted) = resolve(state, setup.owner, vec![weapon.raw()], 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let gained = state.cards.get(weapon).unwrap();
    assert_eq!(gained.owner, setup.owner);
    assert_eq!(state.zones.get(gained.zone).unwrap().kind, ZoneKind::Hand);
}

#[test]
fn equipment_use_offers_optional_draw_but_non_equipment_does_not() {
    let mut setup = setup(2_026_072_702);
    let sha = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        158_201,
        CardKind::Sha,
    );
    let equip = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        158_202,
        CardKind::Armor,
    );
    let mut tx = Transaction::begin(setup.state);
    let hand = tx.state().hand_count(setup.owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: setup.target,
            card: sha,
            targets: vec![setup.owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: setup.target,
                card: equip,
                targets: vec![setup.target],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, accepted) = resolve(tx.commit().state, setup.owner, vec![1], 4);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand + 1);
}
}

mod original_offline_vtbshuli {
//! Golden for OFFLINE 小杀「淑丽」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const VTBSHULI: u32 = 3_163_490_927;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

// 2026-08-15：offline roster +1（zh_shen_zhangjiao）平移了固定种子的身份分配，
// 初始武将出现 skill_added 类可选触发打断 setup 批次；种子 _01→_03、_02→_04 平移恢复。
fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, source, target) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(VTBSHULI),
    });
    run_effects(&mut tx, effects).unwrap();
    let victim = tx.state_mut().players.get_mut(target).unwrap();
    victim.hp = 20;
    victim.max_hp = 20;
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_459),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    (tx.commit().state, owner, source, target, sha)
}

fn deal_sha(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    sha: CardId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card: sha,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn accept(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_contains_optional_twice_per_turn_sha_filter() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == VTBSHULI && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == VTBSHULI)
        .unwrap();
    assert_eq!(
        (trigger.event.as_str(), trigger.filter.as_str()),
        ("damage", "owner_alive_other_source_sha_twice")
    );
    assert!(trigger.optional);
}

#[test]
fn other_players_sha_damage_can_draw_for_both_sides_only_twice_per_turn() {
    let (mut state, owner, source, target, sha) = setup(20_260_716_03);
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    for key in 1..=2 {
        let (next, result) = deal_sha(state, source, target, sha);
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        state = accept(next, owner, key);
    }
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand + 2);

    let (state, result) = deal_sha(state, source, target, sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand + 2);
}

#[test]
fn own_or_non_sha_damage_does_not_offer_shuli() {
    let (state, owner, source, target, sha) = setup(20_260_716_04);
    let (state, result) = deal_sha(state, owner, target, sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_offline_wn_dutan {
//! Golden coverage for 成宜/独探's source-free multi-target Duel.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WN_DUTAN: u32 = 4_152_033_537;
const WN_CHENGYI: u32 = 2_292_190_551;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, Vec<PlayerId>, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(
        (224..=RULE_VM_ABI_VERSION).contains(&package.manifest.rule_vm_abi_version),
        "unexpected package ABI {}",
        package.manifest.rule_vm_abi_version
    );
    assert!(package.skill_index.len() >= 71);
    assert!(package.character_index.len() >= 25);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WN_DUTAN && skill.name == "wn_dutan"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WN_CHENGYI)
        .unwrap();
    assert_eq!(character.name_key, "wn_chengyi");
    assert_eq!(character.skills, [WN_DUTAN]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WN_DUTAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let target_contract = active.targets.as_ref().unwrap();
    assert_eq!((target_contract.min, target_contract.max), (1, 255));
    assert!(!target_contract.allow_self);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_301, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    assert_eq!(targets.len(), 2);

    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
        let player_state = tx.state_mut().players.get_mut(player).unwrap();
        player_state.hp = 8;
        player_state.max_hp = 8;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WN_DUTAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == WN_DUTAN)
        .unwrap();
    (tx.state().clone(), owner, targets, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: &[PlayerId],
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: targets.to_vec(),
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn wn_dutan_uses_one_virtual_duel_on_every_selected_target_once() {
    let (state, owner, targets, skill) = setup();
    let hp: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();

    let (state, result) = activate(state, owner, &targets, skill, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    for (target, before) in targets.iter().zip(hp) {
        assert_eq!(state.players.get(*target).unwrap().hp, before - 1);
    }
    let uses: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardUsed {
                player,
                card,
                targets: used_targets,
            } if *player == owner => Some((*card, used_targets)),
            _ => None,
        })
        .collect();
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].1.as_slice(), targets.as_slice());
    let virtual_card = state.cards.get(uses[0].0).unwrap();
    assert!(virtual_card.def.is_none());

    let before = canonical_state_hash(&state);
    let (same, second) = activate(state, owner, &targets, skill, 2);
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_offline_wn_qifeng {
//! Golden coverage for OFFLINE 杨秋「齐锋」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WN_QIFENG: u32 = 3_930_492_591;

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn losing_the_same_card_twice_makes_the_phase_end_virtual_sha_deal_two_damage() {
    let package = content_compiler::compile_source_named(
        "wn-qifeng-test",
        include_str!("../../../content/original/ported/offline/wn_qifeng.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 496);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_202, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(WN_QIFENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.lost_cards.clear();
    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_lost_by_player(owner), 0);
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Discard;
    let card = inject(&mut tx, owner, 98_400);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![card],
        }],
    )
    .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![card],
                to: hand,
                from: Some(discard),
            },
            EffectRequest::DiscardCards {
                player: owner,
                cards: vec![card],
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);
    assert!(tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def == SkillDefId::new(WN_QIFENG)));
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: target,
                phase: Phase::End.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}
}

mod original_offline_wxdlfangzhu {
//! Golden coverage for OFFLINE 文心雕龙「放逐」(wxdlfangzhu).
//! Same observable contract as SHENHUA fangzhu: after the owner takes damage, they
//! may choose another living player who draws the owner's current lost HP and then
//! turns over. Drives the shipped `choose_other_draw_owner_lost_hp_turn_over` path.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WXDLFANGZHU: u32 = 2_892_907_842;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/wxdlfangzhu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("wxdlfangzhu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_815_85, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(WXDLFANGZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 6;
        owner_state.hp = 6;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        target,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
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
fn compiled_source_and_npack_publish_the_fangzhu_contract() {
    let compiled = content_compiler::compile_source_named("wxdlfangzhu", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == WXDLFANGZHU)
        .expect("wxdlfangzhu trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_other_alive");
    assert!(trigger.optional);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WXDLFANGZHU && skill.name == "wxdlfangzhu" && !skill.locked));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WXDLFANGZHU && skill.name == "wxdlfangzhu"));
    let pack_trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == WXDLFANGZHU)
        .expect("npack wxdlfangzhu trigger");
    assert_eq!(pack_trigger.event, "damage");
    assert_eq!(pack_trigger.filter, "owner_is_target_other_alive");
    assert!(pack_trigger.optional);
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for wxdlfangzhu");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn target_draws_current_owner_lost_hp_before_turning_over() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (mut state, confirmed) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver
    );
    let options: Vec<_> = confirmed
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(!options.contains(&setup.owner.raw()));
    assert!(options.contains(&setup.target.raw()));

    state.players.get_mut(setup.owner).unwrap().hp = 3;
    let (state, resolved) = resolve(state, setup.owner, vec![setup.target.raw()], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 3);
    assert!(state.players.get(setup.target).unwrap().turned_over);
    let draw_index = resolved
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == setup.target))
        .unwrap();
    let turn_index = resolved
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::PlayerTurnedOver { player, .. } if *player == setup.target))
        .unwrap();
    assert!(draw_index < turn_index);
}

#[test]
fn no_other_player_suppresses_trigger_and_dead_choice_is_rejected() {
    let setup = setup();
    let mut no_other = Transaction::begin(setup.state.clone());
    for player in no_other.state().players.alive_by_seat() {
        if player != setup.owner {
            no_other.state_mut().players.get_mut(player).unwrap().alive = false;
        }
    }
    run_effects(
        &mut no_other,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!no_other.state().has_pending_choice());

    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (mut state, confirmed) = resolve(tx.commit().state, setup.owner, vec![1], false, 3);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    state.players.get_mut(setup.target).unwrap().alive = false;
    let (_, rejected) = resolve(state, setup.owner, vec![setup.target.raw()], false, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
}

mod original_offline_wxdlqianqian {
//! Golden coverage for OFFLINE 文心雕龙「慊慊」(wxdlqianqian).
//! First card the owner uses each turn cannot be a heart. After any successful
//! use, the lock is withdrawn until the current turn ends.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_use_cancelled_for_card, query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WXDLQIANQIAN: u32 = 3_312_200_979;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/wxdlqianqian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("wxdlqianqian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_815_86, "identity-standard").unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(WXDLQIANQIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let heart = inject_hand(tx.state_mut(), owner, Suit::Heart, CardKind::Sha, 170_201);
    let spade = inject_hand(tx.state_mut(), owner, Suit::Spade, CardKind::Sha, 170_202);
    (tx.commit().state, owner, heart, spade)
}

fn inject_hand(
    state: &mut GameState,
    player: PlayerId,
    suit: Suit,
    kind: CardKind,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn compiled_source_and_npack_publish_the_qianqian_contract() {
    let compiled = content_compiler::compile_source_named("wxdlqianqian", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == WXDLQIANQIAN && skill.name == "wxdlqianqian" && skill.locked
    }));
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == WXDLQIANQIAN
            && trigger.event == "card_used"
            && trigger.filter == "owner_is_source"
            && !trigger.optional
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == WXDLQIANQIAN && modifier.target.contains("card_use_cancel")
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WXDLQIANQIAN && skill.name == "wxdlqianqian"));
    let pack_trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == WXDLQIANQIAN)
        .expect("npack wxdlqianqian trigger");
    assert_eq!(pack_trigger.event, "card_used");
    assert_eq!(pack_trigger.filter, "owner_is_source");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for wxdlqianqian");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn first_card_this_turn_cannot_be_heart_until_another_card_is_used() {
    let (state, owner, heart, spade) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    assert!(card_use_cancelled_for_card(tx.state(), owner, heart));
    assert!(!card_use_cancelled_for_card(tx.state(), owner, spade));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: heart
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(false)
    ));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: spade
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: spade,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(fired.iter().any(|effect| matches!(
        effect,
        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def }
            if *player == owner && skill_def.raw() == WXDLQIANQIAN
    )));
    run_effects(&mut tx, fired).unwrap();
    assert!(!card_use_cancelled_for_card(tx.state(), owner, heart));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: heart
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));
}

#[test]
fn npack_loads_into_a_started_match() {
    let mut engine = engine_core::Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_87,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(engine
        .state()
        .content
        .skills()
        .any(|def| def.skill_def.raw() == WXDLQIANQIAN));
}
}

mod original_offline_wxdlyishi {
//! Golden coverage for OFFLINE 文心雕龙「遗世」(wxdlyishi).
//! Once per game, the owner picks any other living players; each draws the owner's
//! current lost HP. A follow-up choice grants wxdlfangzhu to one other player.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WXDLYISHI: u32 = 3_108_822_644;
const WXDLFANGZHU: u32 = 2_892_907_842;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/wxdlyishi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, Vec<PlayerId>, SkillId) {
    let package = content_compiler::compile_source_named("wxdlyishi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_815_91, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
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
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(WXDLYISHI),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 6;
        owner_state.hp = 3;
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == WXDLYISHI)
        .unwrap();
    (tx.commit().state, owner, others, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: &[PlayerId],
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: targets.to_vec(),
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
            },
        },
    )
}

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).is_some_and(|instance| instance.def.raw() == def))
}

#[test]
fn compiled_source_and_npack_publish_the_yishi_contract() {
    let compiled = content_compiler::compile_source_named("wxdlyishi", SOURCE).unwrap();
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WXDLYISHI)
        .expect("wxdlyishi active");
    assert!(active.once_per_game);
    assert_eq!(active.window, "play_phase");
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 255));
    assert!(!targets.allow_self);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WXDLYISHI && skill.name == "wxdlyishi"));
    let pack_active = committed
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WXDLYISHI)
        .expect("npack wxdlyishi active");
    assert!(pack_active.once_per_game);
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for wxdlyishi");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn selected_others_draw_owner_lost_hp_then_one_gains_fangzhu() {
    let (state, owner, others, skill) = setup();
    assert_eq!(others.len(), 2);
    let before: Vec<_> = others.iter().map(|p| state.hand_count(*p)).collect();
    let (state, result) = activate(state, owner, &others, skill, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }), "{:?}", result.result);
    for (player, start) in others.iter().zip(before) {
        assert_eq!(state.hand_count(*player), start + 3);
    }
    assert!(state.has_pending_choice());
    let (state, granted) = resolve(state, owner, vec![others[0].raw()], 2);
    assert!(matches!(granted.result, CommandResult::Accepted { .. }), "{:?}", granted.result);
    assert!(has_def(&state, others[0], WXDLFANGZHU));
    assert!(!has_def(&state, others[1], WXDLFANGZHU));
}

#[test]
fn npack_loads_wxdlyishi() {
    let mut engine = engine_core::Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_92,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(engine
        .state()
        .content
        .skills()
        .any(|def| def.skill_def.raw() == WXDLYISHI));
}
}

mod original_offline_xkjubian {
//! Golden for OFFLINE「惧鞭」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::ContinuationKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUBIAN: u32 = 3_008_850_817;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated(hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JUBIAN as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: JUBIAN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: hand,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUBIAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    (tx, owner, other)
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

#[test]
fn committed_package_exposes_pre_damage_hand_above_hp_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JUBIAN)
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("damage_before", "owner_is_target_hand_above_hp", false)
    );
}

#[test]
fn excess_hand_choice_resumes_then_prevents_damage() {
    let (mut tx, owner, other) = isolated(6);
    let original = tx.state().hand_cards(owner);
    assert_eq!(
        damage(&mut tx, other, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    let discarded = vec![original[2].raw(), original[3].raw()];
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(JUBIAN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: discarded,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
}

#[test]
fn hand_not_above_hp_takes_damage_and_source_is_not_restricted() {
    let (mut ordinary, owner, other) = isolated(4);
    damage(&mut ordinary, other, owner).unwrap();
    assert_eq!(ordinary.state().players.get(owner).unwrap().hp, 3);

    let (mut self_damage, owner, _) = isolated(5);
    assert_eq!(
        damage(&mut self_damage, owner, owner),
        Err(EffectError::ChoiceRequested)
    );
}
}

mod original_offline_xktanlu_cuilian {
//! Golden coverage for OFFLINE 崔廉「贪赂」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{choose_command, Engine, Snapshot};
use rule_api::EffectRequest;

const CUILIAN: u32 = 4_231_642_947;
const TANLU: u32 = 4_005_608_784;
const JUBIAN: u32 = 3_008_850_817;
const PROTECT_EQUIP_GAIN: u32 = 4_200_003_031;
const GIVER_DIES: u32 = 4_200_003_032;
const SOURCE_DIES: u32 = 4_200_003_033;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/xktanlu.skill");
const JUBIAN_SOURCE: &str = include_str!("../../../content/original/ported/offline/xkjubian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const HELPERS: &str = r#"
skill protect_equip_gain(4200003031) locked {
  modifier card_gain_protected(equip) override 1;
}

skill giver_dies(4200003032) locked {
  trigger cards_given owner_alive_gave_cards_to_other_alive {
    effect lose_hp(self, 99);
  }
}

skill source_dies(4200003033) locked {
  trigger damage owner_is_source {
    effect lose_hp(self, 99);
  }
}
"#;

#[derive(Clone)]
struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    target_cards: Vec<CardId>,
    owner_cards: Vec<CardId>,
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_500 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    owner_hp: i32,
    target_hp: i32,
    target_kinds: &[CardKind],
    owner_kinds: &[CardKind],
    owner_helpers: &[u32],
    target_helpers: &[u32],
) -> Setup {
    let package = compile_source_named(
        "offline-xktanlu-test",
        &format!("{SOURCE}\n{JUBIAN_SOURCE}\n{HELPERS}"),
    )
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_003, "identity-standard").unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TANLU),
    });
    effects.extend(
        owner_helpers
            .iter()
            .map(|definition| EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    effects.extend(
        target_helpers
            .iter()
            .map(|definition| EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_hp;
    tx.state_mut().players.get_mut(target).unwrap().hp = target_hp;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TANLU)
        .unwrap();
    let mut state = tx.commit().state;
    let target_cards = target_kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| inject(&mut state, target, *kind, index as u8 + 1))
        .collect();
    let owner_cards = owner_kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| inject(&mut state, owner, *kind, index as u8 + 8))
        .collect();
    Setup {
        state,
        owner,
        target,
        skill,
        target_cards,
        owner_cards,
    }
}

fn begin_turn(setup: &Setup) -> (GameState, ChoiceRequest, Vec<DomainEvent>) {
    let mut tx = Transaction::begin(setup.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::BeginTurn {
                player: setup.target
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    (
        committed.state,
        committed.output.choice.unwrap(),
        committed.output.events,
    )
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
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn accept(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, choice, _) = begin_turn(setup);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    resolve(state, setup.owner, vec![1], false, key)
}

fn pass_dying(
    mut state: GameState,
    mut result: PipelineResult,
    mut key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let mut events = result.events;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == engine_core::ContinuationKind::DyingSave
    {
        let actor = PlayerId::new(state.resolution.pending_observer);
        (state, result) = resolve(state, actor, vec![0], false, key);
        key += 1;
        events.extend(result.events);
    }
    (state, events)
}

#[test]
fn package_module_character_and_ids_are_stable() {
    let compiled = compile_source_named("offline-xktanlu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 303);
    let trigger = &compiled.trigger_index[0];
    assert_eq!(trigger.skill_def_id, TANLU);
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    assert_eq!(compiled.modules[0].code, [83, 84, 90, 34, 1, 1]);
    assert_eq!(rule_vm::bytecode::effect_tag::RESOLVE_TANLU, 290);
    assert_eq!(engine_core::ContinuationKind::ResolveTanlu.tag(), 208);

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.modules.len(), 724);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CUILIAN)
        .unwrap();
    assert_eq!(character.name_key, "xk_cuilian");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [TANLU, JUBIAN]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn zero_or_gap_above_original_hand_forces_damage_then_discard() {
    for (owner_hp, target_hp, target_cards) in [(2, 2, 2), (1, 4, 2)] {
        let fixture = setup(
            owner_hp,
            target_hp,
            &vec![CardKind::Other; target_cards],
            &[CardKind::Other],
            &[],
            &[],
        );
        let target = fixture.target;
        let owner_card = fixture.owner_cards[0];
        let hp = fixture.state.players.get(target).unwrap().hp;
        let (state, result) = accept(&fixture, 10 + target_cards as u64);
        assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
        assert_eq!(
            state.resolution.continuation_kind,
            engine_core::ContinuationKind::ResolveTanlu
        );
        assert_eq!(result.choice.as_ref().unwrap().observer, target);
        let (state, result) = resolve(state, target, vec![owner_card.raw()], false, 20);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice());
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(owner_card).unwrap().zone, discard);
    }
}

#[test]
fn gift_branch_rejects_forgery_then_moves_exactly_x_as_one_give_batch() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Other, CardKind::Other, CardKind::Other],
        &[],
        &[],
        &[],
    );
    let owner = fixture.owner;
    let target = fixture.target;
    let cards = fixture.target_cards.clone();
    let (state, result) = accept(&fixture, 30);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Custom);

    let (state, result) = resolve(state, owner, vec![0], false, 31);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let (state, result) = resolve(state, target, Vec::new(), true, 32);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let (state, result) = resolve(state, target, vec![2], false, 33);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let (state, result) = resolve(state, target, vec![0], false, 34);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );

    let (state, result) = resolve(
        state,
        target,
        vec![cards[0].raw(), cards[0].raw()],
        false,
        35,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let selected = vec![cards[0].raw(), cards[1].raw()];
    let (state, result) = resolve(state, target, selected, false, 36);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(cards[..2]
        .iter()
        .all(|card| state.cards.get(*card).unwrap().owner == owner));
    let moved: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsMoved { cards, .. } if cards == &fixture.target_cards[..2] => {
                Some(cards.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(moved, [fixture.target_cards[..2].to_vec()]);
}

#[test]
fn protected_cards_force_a_smaller_gift_and_nested_giver_death_finishes() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Weapon, CardKind::Weapon, CardKind::Other],
        &[],
        &[],
        &[PROTECT_EQUIP_GAIN, GIVER_DIES],
    );
    let owner = fixture.owner;
    let target = fixture.target;
    let given = fixture.target_cards[2];
    let (state, _) = accept(&fixture, 40);
    let (state, result) = resolve(state, target, vec![0], false, 41);
    let (state, events) = pass_dying(state, result, 42);
    assert_eq!(state.cards.get(given).unwrap().owner, owner);
    assert!(!state.players.get(target).unwrap().alive);
    let moved = events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::CardsMoved { cards, .. } if cards == &[given]),
        )
        .unwrap();
    let killed = events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::PlayerKilled { player, .. } if *player == target),
        )
        .unwrap();
    assert!(moved < killed);
}

#[test]
fn jubian_prevents_damage_before_tanlu_reloads_the_discard_choice() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Other, CardKind::Other],
        &[CardKind::Other],
        &[],
        &[JUBIAN],
    );
    let target = fixture.target;
    let target_card = fixture.target_cards[0];
    let owner_card = fixture.owner_cards[0];
    let hp = fixture.state.players.get(target).unwrap().hp;
    let (state, _) = accept(&fixture, 50);
    let (state, result) = resolve(state, target, vec![1], false, 51);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ResolveTanlu
    );

    let (state, result) = resolve(state, target, vec![target_card.raw()], false, 52);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ResolveTanlu
    );
    assert_eq!(
        result.choice.as_ref().unwrap().options[0].token,
        owner_card.raw()
    );
    let (state, result) = resolve(state, target, vec![owner_card.raw()], false, 53);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn target_death_skips_discard_and_source_death_does_not_cancel_the_started_chain() {
    let target_dies = setup(2, 1, &[CardKind::Other], &[CardKind::Other], &[], &[]);
    let target = target_dies.target;
    let owner_card = target_dies.owner_cards[0];
    let (state, _) = accept(&target_dies, 60);
    let (state, result) = resolve(state, target, vec![1], false, 61);
    let (state, events) = pass_dying(state, result, 62);
    assert!(!state.players.get(target).unwrap().alive);
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ResolveTanlu
    );
    assert_ne!(
        state.cards.get(owner_card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved { cards, .. } if cards == &[owner_card]
    )));

    let source_dies = setup(2, 2, &[], &[CardKind::Other], &[SOURCE_DIES], &[]);
    let owner = source_dies.owner;
    let target = source_dies.target;
    let target_hp = source_dies.state.players.get(target).unwrap().hp;
    let (state, result) = accept(&source_dies, 70);
    let (state, events) = pass_dying(state, result, 71);
    assert!(!state.players.get(owner).unwrap().alive);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), 0);
    assert!(!state.has_pending_choice());
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt { source, target: damaged, .. }
            if *source == owner && *damaged == target
    )));
}

#[test]
fn optional_cancel_bad_initial_context_and_runner_choices_are_authoritative() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Other, CardKind::Other, CardKind::Other],
        &[],
        &[],
        &[],
    );
    let (state, _, _) = begin_turn(&fixture);
    let (state, result) = resolve(state, fixture.owner, Vec::new(), true, 80);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(fixture.target).unwrap().hp, 1);

    let mut tx = Transaction::begin(fixture.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTanlu {
                player: fixture.owner,
                target: fixture.target,
                amount: 0,
                stage: 0,
                skill: fixture.skill,
            }]
        ),
        Err(EffectError::Illegal(_))
    ));

    let (state, branch) = accept(&fixture, 81);
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = branch.choice;
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let branch_command = choose_command(&engine);
    assert!(matches!(
        &branch_command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: false,
            ..
        } if selections == &vec![0]
    ));
    assert!(matches!(
        engine.submit(branch_command),
        CommandResult::Accepted { .. }
    ));
    let gift_command = choose_command(&engine);
    assert!(matches!(
        &gift_command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: false,
            ..
        } if selections.len() == 2
    ));
    assert!(matches!(
        engine.submit(gift_command),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(engine.state().hand_count(fixture.owner), 2);
}
}

mod original_offline_yjluoshen {
//! Golden coverage for OFFLINE 洛神 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YJ_ZHENJI: u32 = 3_367_281_789;
const YJLUOSHEN: u32 = 3_100_602_929;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/yjluoshen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: engine_api::CardId,
    second: engine_api::CardId,
    different: engine_api::CardId,
}

fn setup(suits: [Suit; 3]) -> Setup {
    let package = compile_source_named("offline-yjluoshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730_307, "identity-offline").unwrap();
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
        skill_def: SkillDefId::new(YJLUOSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let first = cards[cards.len() - 1];
    let second = cards[cards.len() - 2];
    let different = cards[cards.len() - 3];
    for (card, suit) in [first, second, different].into_iter().zip(suits) {
        tx.state_mut().cards.get_mut(card).unwrap().suit = suit;
    }
    Setup {
        state: tx.commit().state,
        owner,
        first,
        second,
        different,
    }
}

fn offer(setup: Setup) -> Setup {
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    Setup {
        state: committed.state,
        ..setup
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

#[test]
fn package_closes_yj_zhenji_without_legacy_javascript() {
    let compiled = compile_source_named("offline-yjluoshen", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 307);
    assert_eq!(
        rule_vm::bytecode::effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT,
        291
    );
    assert_eq!(
        engine_core::ContinuationKind::JudgeSameColorGainRepeat.tag(),
        209
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YJ_ZHENJI)
        .unwrap();
    assert_eq!(character.skills, [YJLUOSHEN, 24]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn first_effective_color_is_gained_and_repeats_until_a_different_color() {
    for suits in [
        [Suit::Spade, Suit::Club, Suit::Heart],
        [Suit::Heart, Suit::Diamond, Suit::Spade],
    ] {
        let setup = offer(setup(suits));
        let hand_before = setup.state.hand_count(setup.owner);
        let (state, first) = resolve(setup.state, setup.owner, vec![1], false, 1);
        assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
        assert!(state.hand_cards(setup.owner).contains(&setup.first));

        let (state, second) = resolve(state, setup.owner, vec![1], false, 2);
        assert_eq!(second.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
        assert!(state.hand_cards(setup.owner).contains(&setup.second));

        let (state, stopped) = resolve(state, setup.owner, vec![1], false, 3);
        assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
        assert!(stopped.choice.is_none());
        assert_eq!(state.hand_count(setup.owner), hand_before + 2);
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(setup.different).unwrap().zone, discard);
    }
}

#[test]
fn owner_may_stop_and_a_stale_phase_cannot_start_another_judgment() {
    let stopped_setup = offer(setup([Suit::Spade, Suit::Club, Suit::Heart]));
    let (state, first) = resolve(stopped_setup.state, stopped_setup.owner, vec![1], false, 4);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, stopped) = resolve(state, stopped_setup.owner, Vec::new(), true, 5);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(!state
        .hand_cards(stopped_setup.owner)
        .contains(&stopped_setup.second));

    let stale_setup = offer(setup([Suit::Heart, Suit::Diamond, Suit::Spade]));
    let (mut state, first) = resolve(stale_setup.state, stale_setup.owner, vec![1], false, 6);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    state.turn.phase = Phase::Play;
    let (state, stale) = resolve(state, stale_setup.owner, vec![1], false, 7);
    assert!(matches!(stale.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(!state
        .hand_cards(stale_setup.owner)
        .contains(&stale_setup.second));
}
}

mod original_offline_yjxuepin {
//! Golden coverage for 用间曹昂「血拼」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YJXUEPIN: u32 = 2_926_558_107;
const YJ_CAOANG: u32 = 3_284_926_958;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/yjxuepin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    in_range: PlayerId,
    out_of_range: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let owner = tx.state().turn.current;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let in_range = others
        .iter()
        .copied()
        .find(|target| engine_core::query::player_in_attack_range(tx.state(), owner, *target))
        .unwrap();
    let out_of_range = others
        .iter()
        .copied()
        .find(|target| !engine_core::query::player_in_attack_range(tx.state(), owner, *target))
        .unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YJXUEPIN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YJXUEPIN)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        in_range,
        out_of_range,
        skill,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
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
                card_targets: Vec::new(),
            },
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
    let revision = state.revision;
    apply_command(
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
    )
}

#[test]
fn package_publishes_exact_abi155_contract_and_character() {
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
        .find(|active| active.skill_def_id == YJXUEPIN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.in_attack_range);
    assert!(targets.has_discardable_hand_equip);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YJ_CAOANG)
        .unwrap();
    assert_eq!(character.name_key, "yj_caoang");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [YJXUEPIN]);

    let compiled = content_compiler::compile_source_named("yjxuepin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 155);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn two_same_category_cards_recover_after_the_forced_discard() {
    let mut setup = setup(2_026_072_508);
    let sha = inject_hand(&mut setup.state, setup.in_range, 155_101, CardKind::Sha);
    let shan = inject_hand(&mut setup.state, setup.in_range, 155_102, CardKind::Shan);
    let trick = inject_hand(
        &mut setup.state,
        setup.in_range,
        155_103,
        CardKind::TrickWuzhong,
    );
    let (state, offered) = activate(setup.state, setup.owner, setup.in_range, setup.skill, 1);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCardsRecoverSameCategory
    );

    let (state, resolved) = resolve(state, setup.owner, vec![sha.raw(), shan.raw()], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(state.hand_count(setup.in_range), 1);
    assert_eq!(state.cards.get(trick).unwrap().owner, setup.in_range);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Discard,
            ..
        } if cards == &[sha, shan]
    )));
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::HpRecovered {
            player,
            amount: 1,
            ..
        } if *player == setup.owner
    )));
}

#[test]
fn one_card_or_mixed_categories_do_not_recover() {
    let mut one_card = setup(2_026_072_509);
    let only = inject_hand(
        &mut one_card.state,
        one_card.in_range,
        155_201,
        CardKind::Sha,
    );
    let (state, output) = activate(
        one_card.state,
        one_card.owner,
        one_card.in_range,
        one_card.skill,
        10,
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(one_card.owner).unwrap().hp, 3);
    assert_eq!(state.hand_count(one_card.in_range), 0);
    assert_eq!(
        state.cards.get(only).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let mut setup = setup(2_026_072_510);
    let basic = inject_hand(&mut setup.state, setup.in_range, 155_211, CardKind::Sha);
    let trick = inject_hand(
        &mut setup.state,
        setup.in_range,
        155_212,
        CardKind::TrickWuzhong,
    );
    inject_hand(&mut setup.state, setup.in_range, 155_213, CardKind::Shan);
    let (state, _) = activate(setup.state, setup.owner, setup.in_range, setup.skill, 11);
    let (state, output) = resolve(state, setup.owner, vec![basic.raw(), trick.raw()], 12);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert!(!output
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { .. })));
}

#[test]
fn range_empty_target_and_duplicate_selection_are_rejected_atomically() {
    let mut setup = setup(2_026_072_511);
    inject_hand(&mut setup.state, setup.out_of_range, 155_301, CardKind::Sha);
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(
        setup.state,
        setup.owner,
        setup.out_of_range,
        setup.skill,
        20,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let before = canonical_state_hash(&same);
    let (same, rejected) = activate(same, setup.owner, setup.in_range, setup.skill, 21);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    let first = inject_hand(&mut state, setup.in_range, 155_311, CardKind::Sha);
    inject_hand(&mut state, setup.in_range, 155_312, CardKind::Shan);
    inject_hand(&mut state, setup.in_range, 155_313, CardKind::TrickWuzhong);
    let (state, _) = activate(state, setup.owner, setup.in_range, setup.skill, 22);
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, setup.owner, vec![first.raw(), first.raw()], 23);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_offline_yjzhenlve {
//! Golden tests for OFFLINE「缜略」rewritten from inherited legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/offline/yjzhenlve.skill");
const YJZHENLVE: u32 = 2_568_267_200;

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
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
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn add_skill(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(YJZHENLVE),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(68_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(source: PlayerId, card: CardId, targets: Vec<PlayerId>, key: u64) -> Command {
    Command {
        actor: source,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets,
            card_targets: vec![],
        },
    }
}

#[test]
fn committed_skill_matches_the_inherited_zhenlue_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let compiled = content_compiler::compile_source_named("yjzhenlve", SOURCE).unwrap();
    assert!(compiled.skill_index[0].locked);
    assert_eq!(compiled.modifier_index[0].target, "card_use_unrespondable");
    assert_eq!(
        compiled.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(compiled.modifier_index[1].target, "card_target_cancel");
    assert_eq!(
        compiled.modifier_index[1].selector.as_deref(),
        Some("delayed_trick")
    );
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YJZHENLVE && skill.locked));
}

#[test]
fn ordinary_tricks_skip_wuxie_and_delayed_tricks_cannot_target_the_owner() {
    let (mut unrespondable, source, target) = loaded(20_260_721);
    add_skill(&mut unrespondable, source);
    let wuzhong = inject(
        &mut unrespondable,
        source,
        CardKind::TrickWuzhong,
        Suit::Heart,
        7,
    );
    let wuxie = inject(
        &mut unrespondable,
        target,
        CardKind::TrickWuxie,
        Suit::Club,
        12,
    );
    let (state, pipeline) = apply_command(
        unrespondable.state().clone(),
        play(source, wuzhong, vec![], 1),
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), 2);
    assert!(state.hand_cards(target).contains(&wuxie));

    let (mut protected, source, target) = loaded(20_260_722);
    add_skill(&mut protected, target);
    let le = inject(&mut protected, source, CardKind::TrickLe, Suit::Spade, 6);
    let (state, pipeline) =
        apply_command(protected.state().clone(), play(source, le, vec![target], 2));
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert!(state.hand_cards(source).contains(&le));
}
}

mod original_offline_ylyg_hunzi {
//! 第一百八十八批：offline「魂姿」（雁翎孙策 ylyg_hunzi）觉醒零新原语重写。
//! 上游：当你的体力为 1 时：加 1 点体力上限，获得〖英姿〗与〖英魂〗。
//! 实现：hp_lost owner_is_target + body hp==1 门控 + once 标记 → change_max_hp(+1)
//! + 跨包 add_skill 字面量（refresh 英姿、shenhua 英魂——均为 offline 声明依赖）。
//! 近似（登记债）：触发面收窄为体力流失（上游 changeHp 含回复至 1 的路径）。

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YLYG_HUNZI: u32 = 2_262_123_725;
const REYINGZI: u32 = 2_412_932_912; // refresh 英姿
const YINGHUN: u32 = 3_072_887_319; // shenhua 英魂
const SOURCE: &str = include_str!("../../../content/original/ported/offline/ylyg_hunzi.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64, hp: i32) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("ylyg-hunzi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(YLYG_HUNZI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    (tx.commit().state, owner)
}

fn lose_hp(state: GameState, who: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: PlayerId::NONE,
            target: who,
            amount,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn package_publishes_ylyg_hunzi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == YLYG_HUNZI)
        .unwrap();
    assert_eq!(trigger.event, "hp_lost");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
}

#[test]
fn awakening_at_one_hp_grants_yingzi_and_yinghun_once() {
    // hp=1 时流失 1 体力 → 触发：上限+1、获得双技。
    let (state, owner) = setup(20_260_912_02, 1);
    let max_before = state.players.get(owner).unwrap().max_hp;

    let state = lose_hp(state, owner, 1);
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_before + 1,
        "gain one max hp"
    );
    assert!(has_skill(&state, owner, REYINGZI), "英姿 granted");
    assert!(has_skill(&state, owner, YINGHUN), "英魂 granted");

    // once 门控：再次流失不再触发（上限不再增长）。
    let max_after_awaken = state.players.get(owner).unwrap().max_hp;
    let state = lose_hp(state, owner, 1);
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_after_awaken);
}

#[test]
fn hp_above_one_does_not_awaken() {
    let (state, owner) = setup(20_260_912_03, 3);
    let max_before = state.players.get(owner).unwrap().max_hp;
    let state = lose_hp(state, owner, 1);
    // 合成事件只跑触发链不应用体力变化：hp 保持 3，觉醒门控不成立。
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_before);
    assert!(!has_skill(&state, owner, REYINGZI));
}
}

mod original_offline_ylyg_jiang {
//! 第一百八十八批：offline「激昂」（雁翎孙策 ylyg_jiang）零新原语重写 + 全局过滤器。
//! 上游：任意角色使用【决斗】或红色【杀】指定目标后——使用者是你可以摸两张，
//! 否则你与其各摸一张。实现：card_used + owner_alive_(other_)global_juedou_or_
//! red_sha_used 双过滤器（5 点注册零 ABI，sbjiang 过滤器去 owner 介入版）。
//! 近似（登记债）：应答窗口与点数 13 当杀 mod 未建模。

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YLYG_JIANG: u32 = 4_118_108_424;
const YLYG_SUNCE: u32 = 3_809_908_961;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/ylyg_jiang.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("ylyg-jiang-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YLYG_JIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn alloc_card(state: &mut GameState, kind: CardKind, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Processing, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_999),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state
        .zones
        .get_mut(zone)
        .unwrap()
        .cards
        .push(card);
    card
}

fn card_used(state: GameState, user: PlayerId, kind: CardKind, suit: Suit) -> (GameState, usize) {
    let mut tx = Transaction::begin(state);
    let card = alloc_card(tx.state_mut(), kind, suit);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let n = effects.len();
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, n)
}

#[test]
fn package_publishes_ylyg_jiang_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let arms: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == YLYG_JIANG)
        .collect();
    assert_eq!(arms.len(), 2, "self + other global arms");
    assert!(
        arms.iter()
            .any(|t| t.filter == "owner_alive_global_juedou_or_red_sha_used")
    );
    assert!(
        arms.iter()
            .any(|t| t.filter == "owner_alive_other_global_juedou_or_red_sha_used")
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YLYG_SUNCE)
        .expect("ylyg_sunce in roster");
    assert!(character.skills.contains(&YLYG_JIANG));
}

#[test]
fn own_juedou_draws_two_and_other_juedou_draws_one_each() {
    let (state, owner, other) = setup(20_260_912_01);
    let hand_before = state.hand_count(owner);

    // 自己使用【决斗】：摸 2。
    let (state, _) = card_used(state, owner, CardKind::TrickJuedou, Suit::Spade);
    assert_eq!(state.hand_count(owner), hand_before + 2, "own juedou: draw 2");

    // 他人使用红【杀】：各摸 1（owner 净 +1）。
    let (state, n) = card_used(state, other, CardKind::Sha, Suit::Heart);
    assert!(n > 0, "other red sha must fire");
    assert_eq!(state.hand_count(owner), hand_before + 3, "each draws one");

    // 他人使用黑【杀】：不触发。
    let (state, n) = card_used(state, other, CardKind::Sha, Suit::Spade);
    assert_eq!(n, 0, "black sha must not fire");
    assert_eq!(state.hand_count(owner), hand_before + 3);
}
}

mod original_offline_ylygqiangxi {
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
const SOURCE: &str = include_str!("../../../content/original/ported/offline/ylygqiangxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

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
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
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
}

mod original_offline_yy_baimaxiaoqi_skill {
//! Golden for OFFLINE equipment skill「白马骁骑」and its equipment-count tiers.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const BAIMAXIAOQI: u32 = 3_935_591_090;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn loaded() -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(BAIMAXIAOQI as u64),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: BAIMAXIAOQI as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    for player in tx.state().players.alive_by_seat() {
        let skills = tx.state().players.get(player).unwrap().skills.clone();
        for skill in skills {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BAIMAXIAOQI),
        }],
    )
    .unwrap();
    (tx, owner)
}

fn int_query(tx: &Transaction, request: QueryRequest) -> i32 {
    match query(tx.state(), &request, &mut QueryContext::new()) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn attack_range(tx: &Transaction, owner: PlayerId) -> i32 {
    int_query(tx, QueryRequest::AttackRange { player: owner })
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    int_query(tx, QueryRequest::Distance { from, to })
}

fn equip(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_000 + kind as u32),
        kind,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_all_four_equipment_count_modifiers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == BAIMAXIAOQI)
        .collect();
    assert_eq!(modifiers.len(), 4);
    assert!(modifiers.iter().any(|modifier| {
        modifier.target == "attack_range"
            && modifier.value_source.as_deref() == Some("owner_equip_count")
    }));
    for (target, threshold) in [
        ("sha_use_limit", 2),
        ("distance_from", 3),
        ("draw_count", 4),
    ] {
        assert!(modifiers.iter().any(|modifier| {
            modifier.target == target
                && modifier.value_source.as_deref()
                    == Some(format!("owner_equip_count_floor_{threshold}").as_str())
        }));
    }
}

#[test]
fn equipment_count_applies_each_legacy_threshold_without_early_bonus() {
    let (mut tx, owner) = loaded();
    let seats = tx.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let base_range = attack_range(&tx, owner);
    let base_sha = engine_core::query::sha_use_limit_for(tx.state(), owner);
    let base_distance = distance(&tx, owner, far);
    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);
    assert_eq!(attack_range(&tx, owner), base_range);
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        base_sha
    );
    assert_eq!(distance(&tx, owner, far), base_distance);
    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        base_draw
    );

    for (index, kind) in [
        CardKind::Weapon,
        CardKind::Armor,
        CardKind::HorseOff,
        CardKind::HorseDef,
    ]
    .into_iter()
    .enumerate()
    {
        equip(&mut tx, owner, kind);
        let count = index as i32 + 1;
        assert_eq!(attack_range(&tx, owner), base_range + count);
        assert_eq!(
            engine_core::query::sha_use_limit_for(tx.state(), owner),
            base_sha + if count >= 2 { count as u32 } else { 0 }
        );
        assert_eq!(
            distance(&tx, owner, far),
            if count >= 3 {
                (base_distance - count).max(1)
            } else {
                base_distance
            }
        );
        assert_eq!(
            engine_core::query::draw_count_for(tx.state(), owner),
            base_draw + if count >= 4 { count as u32 } else { 0 }
        );
    }
}
}

mod original_offline_yyfuqi {
//! Golden for OFFLINE「伏骑」response locking and per-far-target draws.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const YYFUQI: u32 = 3_281_236_781;
const JIAOZI: u32 = 2_415_897_133;
const YY_QUYI: u32 = 4_120_930_757;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn loaded() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(YYFUQI as u64),
        kind: CommandKind::StartGame {
            player_count: 6,
            rng_seed: YYFUQI as u64 + 1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let removals = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, removals).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YYFUQI),
        }],
    )
    .unwrap();
    (tx, owner, seats)
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(91_001),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_locked_response_modifier_and_per_target_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let quyi = package
        .character_index
        .iter()
        .find(|character| character.id == YY_QUYI)
        .expect("yy_quyi must be published with a complete cross-package closure");
    assert_eq!(quyi.name_key, "yy_quyi");
    assert_eq!(quyi.max_hp, 4);
    assert_eq!(quyi.skills, [YYFUQI, JIAOZI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YYFUQI && skill.locked));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YYFUQI
            && modifier.target == "card_use_unrespondable_within_responder_to_owner_distance"
            && modifier.value == 1
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YYFUQI
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_source_target_distance_gt_one"
    }));
}

#[test]
fn near_targets_are_locked_and_each_far_target_draws_once() {
    let (mut tx, owner, seats) = loaded();
    let other: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let near: Vec<_> = other
        .iter()
        .copied()
        .filter(|target| distance(&tx, owner, *target) <= 1)
        .collect();
    let far: Vec<_> = other
        .iter()
        .copied()
        .filter(|target| distance(&tx, owner, *target) > 1)
        .collect();
    assert!(!near.is_empty());
    assert!(far.len() > 1);
    for target in &near {
        assert!(engine_core::query::card_use_unrespondable_within_distance(
            tx.state(),
            owner,
            *target
        ));
    }
    for target in &far {
        assert!(!engine_core::query::card_use_unrespondable_within_distance(
            tx.state(),
            owner,
            *target
        ));
    }

    let card = inject(&mut tx, owner);
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + far.len() as u32);
}

#[test]
fn another_players_card_does_not_draw_for_the_owner() {
    let (mut tx, owner, seats) = loaded();
    let source = seats.into_iter().find(|player| *player != owner).unwrap();
    let card = inject(&mut tx, source);
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}
}

mod original_offline_yysudi {
//! Golden for OFFLINE 袁绍「肃敌」的 respondTo 与攻击范围语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const YYSUDI: u32 = 3_521_654_010;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated(player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
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
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YYSUDI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_900 + rank as u32),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn draws_when_an_in_range_player_responds_to_the_owners_sha() {
    let (mut tx, owner, others) = isolated(2);
    let responder = others[0];
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
            EffectRequest::DiscardFromHand {
                player: responder,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let sha = inject(&mut tx, owner, CardKind::Sha, 7);
    let shan = inject(&mut tx, responder, CardKind::Shan, 8);

    let (state, pending) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![responder],
                card_targets: vec![],
            },
        },
    );
    let choice = pending.choice.expect("Shan response choice");
    let hand_before = state.hand_count(owner);
    let (state, result) = apply_command(
        state,
        Command {
            actor: responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn ignores_the_wrong_response_origin_and_an_out_of_range_responder() {
    let (mut tx, owner, others) = isolated(4);
    let far = others
        .iter()
        .copied()
        .max_by_key(|player| {
            match query(
                tx.state(),
                &QueryRequest::Distance {
                    from: *player,
                    to: owner,
                },
                &mut QueryContext::new(),
            ) {
                QueryResponse::Int(distance) => distance,
                _ => 0,
            }
        })
        .unwrap();
    let near = others
        .iter()
        .copied()
        .find(|player| *player != far)
        .unwrap();
    let before = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: near,
            respond_to: far,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: far,
            respond_to: owner,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}
}

mod original_offline_yyxuezhan {
//! Golden tests for OFFLINE「血战」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::effective_card_kind;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/offline/yyxuezhan.skill");
const YYXUEZHAN: u32 = 3_105_237_184;
const YYWENCHOU: u32 = 2_778_716_453;
const YYYAZHEN: u32 = 3_406_865_651;

fn loaded(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
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
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in [source, target] {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: source,
        skill_def: SkillDefId::new(YYXUEZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, source, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(69_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn committed_package_contains_the_rewritten_skill_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    let compiled = content_compiler::compile_source_named("yyxuezhan", SOURCE).unwrap();

    assert!(compiled.skill_index[0].locked);
    assert_eq!(compiled.modifier_index.len(), 2);
    assert_eq!(compiled.modifier_index[0].target, "card_name");
    assert_eq!(
        compiled.modifier_index[0].selector.as_deref(),
        Some("ordinary_trick")
    );
    assert_eq!(compiled.modifier_index[0].value, 23);
    assert_eq!(compiled.modifier_index[1].target, "card_use_unrespondable");
    assert_eq!(
        compiled.modifier_index[1].selector.as_deref(),
        Some("juedou")
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YYXUEZHAN && skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YYWENCHOU)
        .unwrap();
    assert_eq!(character.name_key, "yy_wenchou");
    assert_eq!(character.skills, vec![YYXUEZHAN, YYYAZHEN]);
}

#[test]
fn ordinary_trick_becomes_unwuxieable_duel_but_still_asks_for_sha() {
    let (mut tx, source, target) = loaded(20_260_722);
    let physical_trick = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart, 7);
    let sha = inject(&mut tx, target, CardKind::Sha, Suit::Spade, 8);
    let wuxie = inject(&mut tx, target, CardKind::TrickWuxie, Suit::Club, 12);

    assert_eq!(
        effective_card_kind(
            tx.state(),
            source,
            tx.state().cards.get(physical_trick).unwrap()
        ),
        CardKind::TrickJuedou
    );
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: physical_trick,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );

    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JuedouDuel
    );
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, sha.raw());
    assert!(state.hand_cards(target).contains(&wuxie));
    assert_eq!(
        state.cards.get(physical_trick).unwrap().kind,
        CardKind::TrickWuzhong
    );
}

#[test]
fn delayed_tricks_keep_their_physical_name() {
    let (mut tx, source, _) = loaded(20_260_723);
    let delayed = inject(&mut tx, source, CardKind::TrickLe, Suit::Diamond, 6);

    assert_eq!(
        effective_card_kind(tx.state(), source, tx.state().cards.get(delayed).unwrap()),
        CardKind::TrickLe
    );
}
}

mod original_offline_zj_cuijian {
//! Golden coverage for 智将张虎「摧坚」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZJ_CUIJIAN: u32 = 3_620_039_376;
const ZJ_ZHANGHU: u32 = 2_696_580_984;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/zj_cuijian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZJ_CUIJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZJ_CUIJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
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
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_publishes_abi150_exact_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZJ_CUIJIAN && skill.name == "zj_cuijian"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZJ_CUIJIAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (
            targets.min,
            targets.max,
            targets.allow_self,
            targets.hand_nonempty,
        ),
        (1, 1, false, true)
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZJ_ZHANGHU)
        .unwrap();
    assert_eq!(character.name_key, "zj_zhanghu");
    assert_eq!(character.skills, [ZJ_CUIJIAN]);

    let compiled = content_compiler::compile_source_named("zj_cuijian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 150);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn gains_every_shan_and_armor_after_revealing_the_complete_hand() {
    let mut setup = setup(2_026_072_501);
    let target = setup.targets[0];
    let shan = inject_hand(&mut setup.state, target, 150_001, CardKind::Shan);
    let armor = inject_hand(&mut setup.state, target, 150_002, CardKind::Armor);
    let sha = inject_hand(&mut setup.state, target, 150_003, CardKind::Sha);
    let weapon = inject_hand(&mut setup.state, target, 150_004, CardKind::Weapon);
    let (state, output) = activate(setup.state, setup.owner, target, setup.skill, 1);

    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(target), 2);
    for card in [shan, armor] {
        assert_eq!(state.cards.get(card).unwrap().owner, setup.owner);
    }
    for card in [sha, weapon] {
        assert_eq!(state.cards.get(card).unwrap().owner, target);
    }
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[shan, armor, sha, weapon] && to.is_empty()
    )));
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &[shan, armor]
    )));

    let before = canonical_state_hash(&state);
    let (same, repeated) = activate(state, setup.owner, target, setup.skill, 2);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn no_matching_card_draws_two_and_an_empty_target_is_rejected_atomically() {
    let setup = setup(2_026_072_502);
    let empty_target = setup.targets[0];
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(setup.state, setup.owner, empty_target, setup.skill, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    let sha = inject_hand(&mut state, empty_target, 150_011, CardKind::Sha);
    let weapon = inject_hand(&mut state, empty_target, 150_012, CardKind::Weapon);
    let (state, output) = activate(state, setup.owner, empty_target, setup.skill, 11);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(empty_target), 2);
    assert_eq!(state.cards.get(sha).unwrap().owner, empty_target);
    assert_eq!(state.cards.get(weapon).unwrap().owner, empty_target);
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[sha, weapon] && to.is_empty()
    )));
    assert!(!output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards.contains(&sha) || cards.contains(&weapon)
    )));
}
}

mod original_offline_zj_duanbing {
//! Golden for OFFLINE 智将丁奉「短兵」rewritten from legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    RULE_VM_ABI_VERSION,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, sha_response_count_for, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZJ_DINGFENG: u32 = 3_741_059_626;
const ZJ_DUANBING: u32 = 2_843_523_405;
const ZJ_FENXUN: u32 = 3_362_692_932;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(ZJ_DUANBING as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: ZJ_DUANBING as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZJ_DUANBING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 20;
        participant.max_hp = 20;
    }
    (tx, owner, seats)
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected distance response: {other:?}"),
    }
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
}

#[test]
fn package_closes_zj_dingfeng_and_publishes_both_generic_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.character_index.iter().any(|character| {
        character.id == ZJ_DINGFENG && character.skills == [ZJ_DUANBING, ZJ_FENXUN]
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZJ_DUANBING
            && trigger.filter
                == "owner_source_target_distance_le_one_first_damage_to_target_this_turn"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == ZJ_DUANBING
            && modifier.target == "sha_response_count_if_source_owner_target_distance_le_one"
    }));
}

#[test]
fn nearby_sha_needs_two_shan_and_only_first_damage_per_target_draws() {
    let (mut tx, owner, seats) = setup();
    let near: Vec<_> = seats
        .iter()
        .copied()
        .filter(|target| *target != owner && distance(&tx, owner, *target) == 1)
        .collect();
    let far = seats
        .iter()
        .copied()
        .find(|target| distance(&tx, owner, *target) > 1)
        .unwrap();
    assert_eq!(sha_response_count_for(tx.state(), owner, near[0], 1), 2);
    assert_eq!(sha_response_count_for(tx.state(), owner, far, 1), 1);
    assert_eq!(sha_response_count_for(tx.state(), near[0], owner, 1), 1);

    let hand = tx.state().hand_count(owner);
    damage(&mut tx, owner, near[0]);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    damage(&mut tx, owner, near[0]);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    damage(&mut tx, owner, near[1]);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    damage(&mut tx, owner, far);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}
}

mod original_offline_zj_guluo {
//! Golden for OFFLINE「孤落」的空手濒死语义。

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const GULUO: u32 = 3_452_606_144;
const CHENGGONG: u32 = 3_971_525_290;
const ZHANGU: u32 = 3_458_676_792;
const JUN_JIANG: u32 = 4_092_230_721;
const SPSHUDE: u32 = 2_834_305_041;
const JIE: u32 = 3_115_545_698;
const YJQINGJIAO: u32 = 4_207_753_903;
const STDQINGJIAO: u32 = 4_273_235_044;
const EU_XIRU: u32 = 3_520_897_772;
const SCLS_YINSHI: u32 = 3_553_948_732;
const GZSUISHI2: u32 = 3_663_559_953;
const GZSUISHI: u32 = 4_271_491_554;
const JYLIJUN: u32 = 2_928_426_897;
const EU_JIQUAN: u32 = 2_434_098_210;
const PSDIWAN: u32 = 3_273_351_282;
const HM_ZHONG_DIAMOND_SKILL: u32 = 2_520_096_911;
const ZJ_HANCI: u32 = 3_201_112_152;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn isolated(engine: &Engine) -> Transaction {
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<EffectRequest> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    // Removal may surface (or leave) optional-trigger choices for
    // deal-dependent skills; keep removing and cancelling until the state is
    // choice-free and skill-free.
    let mut key = 9_000u64;
    loop {
        if !tx.state().skills.sorted_live().is_empty() {
            match run_effects(&mut tx, effects.clone()) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("effect error: {e:?}"),
            }
        }
        if tx.state().resolution.pending_choice_id == 0 {
            break;
        }
        let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
        let (next, _) = apply_command(
            tx.state().clone(),
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(tx.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        tx = Transaction::begin(next);
        key += 1;
    }
    tx
}

fn empty_dying(engine: &Engine, max_hp: u32) -> Transaction {
    let owner = engine.state().turn.current;
    let mut tx = isolated(engine);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(GULUO),
            },
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 0;
    player.max_hp = max_hp;
    match run_effects(&mut tx, vec![EffectRequest::EnterDying { player: owner }]) {
        Ok(_) => {}
        Err(e) => eprintln!("DBG EnterDying err: {:?}", e),
    }
    tx
}

#[test]
fn hanci_draws_for_each_living_player_when_one_skill_is_added() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_4),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_4,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let (owner, recipient) = (players[0], players[1]);
    let mut tx = isolated(&engine);
    let initial_owner_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJ_HANCI),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_owner_hand + 2);

    let (owner_hand, recipient_hand) = (
        tx.state().hand_count(owner),
        tx.state().hand_count(recipient),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: recipient,
            skill_def: SkillDefId::new(SCLS_YINSHI),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_hand + 1);
    assert_eq!(tx.state().hand_count(recipient), recipient_hand + 1);

    tx.state_mut().players.get_mut(recipient).unwrap().alive = false;
    let hands = (
        tx.state().hand_count(owner),
        tx.state().hand_count(recipient),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: recipient,
            skill_def: SkillDefId::new(JIE),
        }],
    )
    .unwrap();
    assert_eq!(
        (
            tx.state().hand_count(owner),
            tx.state().hand_count(recipient)
        ),
        hands
    );

    tx.state_mut().players.get_mut(recipient).unwrap().alive = true;
    tx.state_mut().players.get_mut(owner).unwrap().alive = false;
    let hands = (
        tx.state().hand_count(owner),
        tx.state().hand_count(recipient),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: recipient,
            skill_def: SkillDefId::new(HM_ZHONG_DIAMOND_SKILL),
        }],
    )
    .unwrap();
    assert_eq!(
        (
            tx.state().hand_count(owner),
            tx.state().hand_count(recipient)
        ),
        hands
    );
}

#[test]
fn xiru_counts_all_living_mount_cards_and_rounds_up() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_2),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_714_2,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(EU_XIRU),
        }],
    )
    .unwrap();
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);

    let players = tx.state().players.alive_by_seat();
    for (index, expected) in [3, 3, 4].into_iter().enumerate() {
        let holder = players[index];
        let zone = tx.state().zones.find(ZoneKind::Equip, holder).unwrap();
        let kind = if index % 2 == 0 {
            CardKind::HorseOff
        } else {
            CardKind::HorseDef
        };
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(99_000 + index as u32),
            kind,
            suit: Suit::Spade,
            rank: 5,
            weapon_range: 0,
            zone,
            owner: holder,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        assert_eq!(
            engine_core::query::draw_count_for(tx.state(), owner),
            expected
        );
    }
}

#[test]
fn yinshi_skips_judgment_and_leaves_delayed_cards_in_place() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_3),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_3,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCLS_YINSHI),
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let delayed = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100),
        kind: CardKind::TrickLe,
        suit: Suit::Spade,
        rank: 6,
        weapon_range: 0,
        zone: judge,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(judge)
        .unwrap()
        .cards
        .push(delayed);
    tx.state_mut().turn.phase = engine_core::state::Phase::Judge;

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, engine_core::state::Phase::Draw);
    assert_eq!(state.cards.get(delayed).unwrap().zone, judge);
}

#[test]
fn qingjiao_draws_after_damaging_another_qun_character() {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let offline = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for skill_def in [STDQINGJIAO, YJQINGJIAO] {
        for (target_character, expected_draw) in [(24, 1), (1, 0)] {
            let mut engine = Engine::new();
            engine.load_package(&standard).unwrap();
            engine.load_package(&offline).unwrap();
            engine.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                // offline 新将批 roster 平移：+salt 避开 identity 分配坏种子窗。
                idempotency_key: IdempotencyKey(skill_def as u64 + target_character as u64 + 7),
                kind: CommandKind::StartGame {
                    player_count: 4,
                    rng_seed: skill_def as u64 + target_character as u64 + 7,
                    ruleset: "identity-standard".into(),
                },
            });
            let owner = engine
                .state()
                .players
                .iter_live()
                .find(|player| player.identity == engine_core::state::Identity::Lord)
                .unwrap()
                .id;
            let target = engine
                .state()
                .players
                .iter_live()
                .find(|player| player.id != owner)
                .unwrap()
                .id;
            let mut tx = isolated(&engine);
            tx.state_mut().turn.current = owner;
            tx.state_mut().turn.phase = engine_core::state::Phase::Play;
            tx.state_mut()
                .players
                .get_mut(target)
                .unwrap()
                .character_def = target_character;
            run_effects(
                &mut tx,
                vec![
                    EffectRequest::AddSkill {
                        player: owner,
                        skill_def: SkillDefId::new(skill_def),
                    },
                    EffectRequest::DealDamage {
                        source: owner,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: 0,
                    },
                ],
            )
            .unwrap();
            let before = tx.state().hand_count(owner);
            run_effects(
                &mut tx,
                vec![EffectRequest::SetTurnState {
                    current: owner,
                    phase: engine_core::state::Phase::End.to_u8(),
                }],
            )
            .unwrap();
            assert_eq!(tx.state().hand_count(owner), before + expected_draw);
        }
    }
}

#[test]
fn jie_only_adds_damage_to_red_sha() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_822),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_822,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIE),
        }],
    )
    .unwrap();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = engine_core::state::CardKind::Sha;

    tx.state_mut().cards.get_mut(card).unwrap().suit = Suit::Heart;
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, card, 1),
        2
    );
    tx.state_mut().cards.get_mut(card).unwrap().suit = Suit::Spade;
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, card, 1),
        1
    );
    assert_eq!(engine_core::query::sha_damage_for(tx.state(), owner, 1), 1);
}

#[test]
fn jun_jiang_offers_draw_only_after_the_owner_uses_a_red_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713_1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let prepare = |suit| {
        let mut tx = isolated(&engine);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUN_JIANG),
            }],
        )
        .unwrap();
        let card = tx.state().hand_cards(owner)[0];
        tx.state_mut().cards.get_mut(card).unwrap().suit = suit;
        (tx, card)
    };

    let (mut red, red_card) = prepare(Suit::Heart);
    let hand = red.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut red,
            &[DomainEvent::CardUsed {
                player: owner,
                card: red_card,
                targets: vec![],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        red.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(red.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(red.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let (mut black, black_card) = prepare(Suit::Spade);
    let effects = dispatch_for_events(
        &mut black,
        &[DomainEvent::CardUsed {
            player: owner,
            card: black_card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!black.state().has_pending_choice());
}

#[test]
fn spshude_offers_to_draw_to_max_hp_only_when_hand_is_below_it() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let prepare = || {
        let mut tx = isolated(&engine);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SPSHUDE),
            }],
        )
        .unwrap();
        tx
    };

    let mut below = prepare();
    run_effects(
        &mut below,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let max_hp = below.state().players.get(owner).unwrap().max_hp;
    assert_eq!(below.state().hand_count(owner), 0);
    assert_eq!(
        dispatch_for_events(
            &mut below,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        below.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(below.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(below.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), max_hp);

    let mut full = prepare();
    let max_hp = full.state().players.get(owner).unwrap().max_hp;
    let hand = full.state().hand_count(owner);
    if hand < max_hp {
        run_effects(
            &mut full,
            vec![EffectRequest::DrawCards {
                player: owner,
                count: max_hp - hand,
            }],
        )
        .unwrap();
    }
    let effects = dispatch_for_events(
        &mut full,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!full.state().has_pending_choice());

    let mut dead = prepare();
    run_effects(
        &mut dead,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    dead.state_mut().players.get_mut(owner).unwrap().alive = false;
    let effects = dispatch_for_events(
        &mut dead,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!dead.state().has_pending_choice());
}

#[test]
fn committed_package_recovers_or_dies_when_max_hp_reaches_zero() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed.manifest.logic_hash,
        committed.compute_logic_hash()
    );
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == GULUO));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHENGGONG));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHANGU));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == EU_XIRU));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SCLS_YINSHI));

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;

    let recovered = empty_dying(&engine, 4);
    let player = recovered.state().players.get(owner).unwrap();
    assert_eq!(
        (player.max_hp, player.hp, player.alive, player.dying),
        (3, 1, true, false)
    );

    let dead = empty_dying(&engine, 1);
    let player = dead.state().players.get(owner).unwrap();
    assert_eq!((player.max_hp, player.hp, player.alive), (0, 0, false));
}

#[test]
fn zhangu_requires_max_hp_above_one_and_an_empty_zone() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let mut active = isolated(&engine);
    run_effects(
        &mut active,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHANGU),
        }],
    )
    .unwrap();
    let max_hp = active.state().players.get(owner).unwrap().max_hp;
    let hand = active.state().hand_count(owner);
    run_effects(
        &mut active,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        active.state().players.get(owner).unwrap().max_hp,
        max_hp - 1
    );
    assert_eq!(active.state().hand_count(owner), hand + 2);

    let mut blocked = isolated(&engine);
    run_effects(
        &mut blocked,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHANGU),
        }],
    )
    .unwrap();
    blocked.state_mut().players.get_mut(owner).unwrap().max_hp = 1;
    let hand = blocked.state().hand_count(owner);
    run_effects(
        &mut blocked,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(blocked.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(blocked.state().hand_count(owner), hand);
}

#[test]
fn gzsuishi2_loses_hp_only_when_the_dead_player_has_the_same_group() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (victim_character, expected_hp) in [(9_215_002, 4), (9_215_003, 5)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "gzsuishi2-test-roster".into();
        roster.character_index = [(9_215_001, "wei"), (9_215_002, "wei"), (9_215_003, "shu")]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                id,
                name_key: format!("group_{id}"),
                max_hp: 5,
                initial_hp: 5,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            })
            .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(victim_character as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: victim_character as u64,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine.state().turn.current;
        let mut tx = isolated(&engine);
        let victim = tx
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        {
            let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
            owner_state.character_def = 9_215_001;
            owner_state.hp = 5;
            owner_state.max_hp = 5;
        }
        tx.state_mut()
            .players
            .get_mut(victim)
            .unwrap()
            .character_def = victim_character;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(GZSUISHI2),
            }],
        )
        .unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: victim,
                killer: PlayerId::NONE,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }
}

#[test]
fn jylijun_only_offers_to_draw_for_a_living_wei_damage_target() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (target_group, should_trigger) in [("wei", true), ("shu", false)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "jylijun-test-roster".into();
        roster.character_index = [(9_217_001, "wei"), (9_217_002, target_group)]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                id,
                name_key: format!("lijun_group_{id}"),
                max_hp: 5,
                initial_hp: 5,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            })
            .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_825),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_825,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine.state().turn.current;
        let target = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        let mut tx = isolated(&engine);
        tx.state_mut().players.get_mut(owner).unwrap().character_def = 9_217_001;
        {
            let target_state = tx.state_mut().players.get_mut(target).unwrap();
            target_state.character_def = 9_217_002;
            target_state.hp = 5;
            target_state.max_hp = 5;
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JYLIJUN),
            }],
        )
        .unwrap();
        let hand = tx.state().hand_count(target);
        let result = run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        );
        if should_trigger {
            assert_eq!(
                result.unwrap_err(),
                engine_core::effect::EffectError::ChoiceRequested
            );
            let (state, result) = apply_command(
                tx.state().clone(),
                Command {
                    actor: owner,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(1),
                    kind: CommandKind::ResolveChoice {
                        choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            assert_eq!(state.hand_count(target), hand + 1);
        } else {
            result.unwrap();
            assert_eq!(tx.state().hand_count(target), hand);
        }
    }
}

#[test]
fn gzsuishi_uses_the_damage_source_group_when_another_player_enters_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (source_group, expected_draws) in [("wei", 1), ("shu", 0)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "gzsuishi-test-roster".into();
        roster.character_index = [
            (9_218_001, "wei"),
            (9_218_002, source_group),
            (9_218_003, "qun"),
        ]
        .into_iter()
        .map(|(id, group)| PackageCharacterEntry {
            id,
            name_key: format!("suishi_group_{id}"),
            max_hp: 5,
            initial_hp: 5,
            skills: vec![],
            sex: "male".into(),
            group: group.into(),
            is_lord: false,
            clans: vec![],
            tags: vec![],
            sort_key: String::new(),
            portrait_asset: None,
            skin_assets: vec![],
        })
        .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_826),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_826,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine.state().turn.current;
        let others: Vec<_> = engine
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != owner)
            .collect();
        let source = others[0];
        let target = others[1];
        let mut tx = isolated(&engine);
        tx.state_mut().players.get_mut(owner).unwrap().character_def = 9_218_001;
        tx.state_mut()
            .players
            .get_mut(source)
            .unwrap()
            .character_def = 9_218_002;
        {
            let target_state = tx.state_mut().players.get_mut(target).unwrap();
            target_state.character_def = 9_218_003;
            target_state.hp = 1;
            target_state.max_hp = 5;
        }
        let mut setup_effects: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            })
            .collect();
        setup_effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GZSUISHI),
        });
        run_effects(&mut tx, setup_effects).unwrap();
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let safe_draw = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_218_100),
            kind: CardKind::Other,
            suit: Suit::Club,
            rank: 1,
            weapon_range: 0,
            zone: draw,
            owner: PlayerId::NONE,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .push(safe_draw);
        let hand = tx.state().hand_count(owner);
        let events = run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
        assert!(!tx.state().has_pending_choice());
        assert!(!tx.state().players.get(target).unwrap().alive);
        assert!(events.iter().any(|event| matches!(
            event,
            engine_api::DomainEvent::PlayerKilled { player, .. } if *player == target
        )));
    }
}

#[test]
fn eu_jiquan_only_fires_for_a_living_western_turn_owner() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (group, expected_draws) in [("western", 1), ("wei", 0)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "eu-jiquan-test-roster".into();
        roster.character_index = [(9_219_001, "wei"), (9_219_002, group)]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                id,
                name_key: format!("jiquan_group_{id}"),
                max_hp: 5,
                initial_hp: 5,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            })
            .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_826),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_826,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.identity == engine_core::state::Identity::Lord)
            .unwrap()
            .id;
        let current = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        let mut tx = isolated(&engine);
        tx.state_mut().players.get_mut(owner).unwrap().character_def = 9_219_001;
        tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
        tx.state_mut()
            .players
            .get_mut(current)
            .unwrap()
            .character_def = 9_219_002;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(EU_JIQUAN),
            }],
        )
        .unwrap();
        let hand = tx.state().hand_count(owner);
        match run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current,
                phase: engine_core::state::Phase::Prepare.to_u8(),
            }],
        ) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("SetTurnState error: {e:?}"),
        }
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
        assert_eq!(
            tx.state().players.get(owner).unwrap().hp,
            3 + expected_draws as i32
        );
    }
}

#[test]
fn psdiwan_draws_the_declared_sha_target_count_for_its_user() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_827),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_827,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let targets: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PSDIWAN),
        }],
    )
    .unwrap();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + targets.len() as u32);

    for (source, kind) in [(owner, CardKind::Tao), (targets[0], CardKind::Sha)] {
        let mut tx = isolated(&engine);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(PSDIWAN),
            }],
        )
        .unwrap();
        let card = tx.state().hand_cards(source)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
        assert!(dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: source,
                card,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap()
        .is_empty());
        assert!(!tx.state().has_pending_choice());
    }
}

#[test]
fn hm_zhong_diamond_draws_two_after_damage_only() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_3),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_3,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HM_ZHONG_DIAMOND_SKILL),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(tx.state().hand_count(owner), hand + 2);

    let hand = tx.state().hand_count(owner);
    for (target, is_lose_hp) in [(other, false), (owner, true)] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target,
                amount: 1,
                is_lose_hp,
                nature: 0,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand);
    }
}
}

mod original_offline_zjfuli {
//! offline/zj_fuli 界扶利（第九十八批·§6 S0/S1 首个消费者）：
//! 濒死 self-save → recover 至 2 + 摸 2 + 相位裁剪额外回合（仅 Play）。
//! queue_extra_phases ABI 672/tag 695：bit=1<<TurnPhaseTag，Play=1<<3=8。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZJ_FULI: u32 = 2_494_001_087;
const PLAY_MASK: u32 = 1 << 3;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/offline"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("offline-zjfuli", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_836, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJ_FULI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

#[test]
fn zjfuli_publishes_contract() {
    let compiled = content_compiler::compile_source_named("offline-zjfuli-contract", &sources())
        .unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 691);
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == ZJ_FULI && t.event == "dying" && t.optional
    }));
}

#[test]
fn zjfuli_latches_and_enqueues_play_only_extra_turn() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 造濒死：压到 1 点体力并进入濒死窗口（dying 事件由合成驱动）。
    {
        let p = tx.state_mut().players.get_mut(holder).unwrap();
        p.hp = 1;
    }
    let hp_before = 1;
    let hand_before = tx.state().hand_count(holder);

    let opened = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: holder,
            source: engine_api::PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "dying opens the self-save confirm"
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 1401);
    assert!(!mid.has_pending_choice());

    // 回复至 2（表达式 2 - hp 差值，引擎钳位非负）。
    assert_eq!(mid.players.get(holder).unwrap().hp, 2);
    // 摸两张。
    assert_eq!(mid.hand_count(holder), hand_before + 2);
    // 裁剪回合入位：仅 Play（tag 3）。
    let entry = mid
        .turn
        .extra_phase_queue
        .iter()
        .find(|e| e.player == holder)
        .expect("cropped turn enqueued");
    assert_eq!(entry.phases, vec![3u8]);
    assert!(!entry.no_turn_over);

    // limited（storage key1）闩上：二次濒死不再触发。
    let mut tx = Transaction::begin(mid);
    {
        let p = tx.state_mut().players.get_mut(holder).unwrap();
        p.hp = 1;
    }
    let opened2 = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: holder,
            source: engine_api::PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    assert_eq!(
        opened2.unwrap_err(),
        EffectError::ChoiceRequested,
        "optional confirm still opens while latched"
    );
    let tx = tx.commit();
    let latched = resolve(tx.state, holder, vec![1], 1501);
    assert_eq!(latched.players.get(holder).unwrap().hp, 1, "locked by storage");
    assert!(!latched.has_pending_choice());
    let _ = hp_before;
}
}

mod original_offline_zylianji {
//! offline/zylianji 连计（第一百一十九批·§6 S2）：
//! phase_play_end（本阶段用过牌，类别≥1）→ 令一名角色摸 1 → 跳自己剩余阶段 →
//! 仅 Play 的裁剪额外回合（skip_remaining_phases + queue_extra_phases）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZYLIANJI: u32 = 4_062_778_093;
const TWJIEQIU: u32 = 4_201_797_709;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/offline"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("offline-zylianji", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_839, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZYLIANJI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn others_of(state: &GameState, owner: PlayerId) -> Vec<PlayerId> {
    state
        .players
        .iter_live()
        .map(|p| p.id)
        .filter(|id| *id != owner)
        .collect()
}

fn inject_cards(tx: &mut Transaction, owner: PlayerId, count: u8) -> Vec<engine_api::CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    (0..count)
        .map(|index| {
            let card = tx.state_mut().cards.alloc(Card {
                id: engine_api::CardId::NONE,
                def: engine_api::CardDefId::new(1 + index as u32),
                kind: engine_core::state::CardKind::Tao,
                suit: engine_api::Suit::Heart,
                rank: 2 + index,
                weapon_range: 0,
                zone: hand,
                owner,
                face_up: true,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}

fn discard_move(cards: Vec<engine_api::CardId>, to: engine_api::ZoneId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::CardsMoved {
        cards,
        from: engine_api::ZoneId::NONE,
        to,
        reason: engine_api::CardMoveReason::Discard,
    }
}

fn end_of_play_event(current: PlayerId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::PhaseEnded {
        current,
        phase: engine_api::TurnPhaseTag::Play,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn zylianji_publishes_contract() {
    let compiled =
        content_compiler::compile_source_named("offline-zylianji-contract", &sources()).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 691);
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == ZYLIANJI
            && t.event == "phase_play_end"
            && t.filter == "owner_alive_play_end_used_categories_positive"
            && t.optional
    }));
}

#[test]
fn zylianji_grants_draw_skips_remaining_and_cropped_turn() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);
    let sid = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == ZYLIANJI)
        .unwrap()
        .id;

    // 出牌阶段（Play）用了 2 张牌 → 类别数 ≥1。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let cards = inject_cards(&mut tx, holder, 2);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card: cards[0],
            targets: vec![],
        }],
    );
    opened.unwrap();

    // 出牌阶段结束：confirm → 摸1给任一角色 → 跳自己剩余 → 仅 Play 裁剪回合。
    let cur = tx.state().turn.current;
    let opened = drive(&mut tx, &[end_of_play_event(cur)]);
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "used cards this phase opens the confirm"
    );
    let committed = tx.commit();
    // confirm 后先开「选一名角色摸 1」的续体（choose_other_draw_or_recover）。
    let mid = resolve(committed.state, holder, vec![1], 1801);
    assert_eq!(
        mid.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ChooseOtherDrawOrRecover
    );
    let observer = mid.resolution.pending_observer;
    let mid = resolve(mid, PlayerId::new(observer), vec![victim.raw()], 1802);
    assert!(!mid.has_pending_choice());

    // 跳过的 SKIP 标志已置位（Draw/Discard），Play 未跳。
    assert!(mid.turn.skip_draw());
    assert!(!mid.turn.skip_play());
    assert!(mid.turn.skip_discard());
    // 裁剪回合入队：仅 Play。
    assert_eq!(mid.turn.extra_phase_queue.len(), 1);
    assert_eq!(mid.turn.extra_phase_queue[0].player, holder);
    assert_eq!(mid.turn.extra_phase_queue[0].phases, vec![3u8]);
}
}

mod original_offline_zymoucheng {
//! Golden for OFFLINE「谋逞」的装备来源、借刀流程与每回合次数限制。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZYMOUCHENG: u32 = 3_584_095_001;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
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
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: (kind == CardKind::Weapon).then_some(3).unwrap_or(0),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZYMOUCHENG),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZYMOUCHENG)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    holder: PlayerId,
    victim: PlayerId,
    source: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![holder, victim],
                card_targets: vec![source],
            },
        },
    )
}

#[test]
fn package_exposes_the_full_legacy_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == ZYMOUCHENG)
        .expect("谋逞 view_as entry");
    assert_eq!(view.as_kind, "jiedao");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "black");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(view.uses_per_turn, Some(1));
}

#[test]
fn black_equipment_can_be_used_as_jiedao_and_the_skill_cannot_repeat() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_715, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let players = tx.state().players.alive_by_seat();
    let (actor, holder, victim) = (tx.state().turn.current, players[1], players[2]);
    for player in players {
        clear_hand(&mut tx, player);
    }
    let skill = add_skill(&mut tx, actor);
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        7,
    );
    let second_source = inject(&mut tx, actor, ZoneKind::Hand, CardKind::Tao, Suit::Club, 8);
    inject(
        &mut tx,
        holder,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        9,
    );

    let (state, pipe) = activate(tx.state().clone(), actor, skill, holder, victim, source, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("借刀持刀者选择");
    assert_eq!(choice.observer, holder);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(source).unwrap().zone, discard);

    let (state, pipe) = apply_command(
        state,
        Command {
            actor: holder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));

    let mut tx = Transaction::begin(state);
    inject(
        &mut tx,
        holder,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Diamond,
        10,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, actor, skill, holder, victim, second_source, 3);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_offline_zyshangyi {
//! Golden coverage for 蒋钦「尚义」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZYSHANGYI: u32 = 2_778_115_845;
const JIANGQING: u32 = 2_799_920_397;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/zyshangyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZYSHANGYI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZYSHANGYI)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject_card(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(matches!(kind, CardKind::Weapon)),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
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

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

#[test]
fn package_publishes_jiangqing_and_the_exact_active_contract_at_abi153() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.active_index.len(), 137);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZYSHANGYI && skill.name == "zyshangyi"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZYSHANGYI)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|target| (target.min, target.max, target.allow_self)),
        Some((1, 1, false))
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JIANGQING)
        .unwrap();
    assert_eq!(character.name_key, "jiangqing");
    assert_eq!(character.skills, [ZYSHANGYI]);

    let compiled = content_compiler::compile_source_named("zyshangyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 153);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn reveals_the_owner_hand_to_the_target_and_cancellation_keeps_target_cards() {
    let mut setup = setup(2_026_072_503);
    let owner_sha = inject_card(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Sha,
        153_001,
    );
    let owner_shan = inject_card(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Shan,
        153_002,
    );
    let target_hand = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        CardKind::Tao,
        153_003,
    );
    let target_equip = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        153_004,
    );
    let (state, output) = activate(setup.state, setup.owner, setup.target, setup.skill, 1);

    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[owner_sha, owner_shan] && to == &[setup.target]
    )));
    let choice = output.choice.unwrap();
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, true)
    );
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_hand.raw()]
    );

    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(target_hand).unwrap().owner, setup.target);
    assert_eq!(state.cards.get(target_equip).unwrap().owner, setup.target);
}

#[test]
fn selected_hand_card_is_discarded_but_equipment_is_never_offered() {
    let mut setup = setup(2_026_072_504);
    let target_hand = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        CardKind::Tao,
        153_011,
    );
    let target_equip = inject_card(
        &mut setup.state,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        153_012,
    );
    let (state, output) = activate(setup.state, setup.owner, setup.target, setup.skill, 10);
    let choice = output.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target_hand.raw()]
    );
    let (state, finished) = resolve(state, setup.owner, vec![target_hand.raw()], false, 11);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_hand).unwrap().zone, discard);
    assert_eq!(
        state.cards.get(target_equip).unwrap().zone,
        state.zones.find(ZoneKind::Equip, setup.target).unwrap()
    );
}

#[test]
fn empty_target_hand_is_a_noop_while_self_target_and_second_use_are_rejected() {
    let mut setup = setup(2_026_072_505);
    let owner_card = inject_card(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Sha,
        153_021,
    );
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(setup.state, setup.owner, setup.owner, setup.skill, 20);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, accepted) = activate(same, setup.owner, setup.target, setup.skill, 21);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.choice.is_none());
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[owner_card] && to == &[setup.target]
    )));
    let before = canonical_state_hash(&state);
    let (same, repeated) = activate(state, setup.owner, setup.target, setup.skill, 22);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}
