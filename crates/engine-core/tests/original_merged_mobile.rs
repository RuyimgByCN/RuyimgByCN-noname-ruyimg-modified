//! O2 全量合并批次 1（2026-10-14）：家族 mobile 的 41 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_mobile_beizhu_dingyuan {
//! Golden coverage for MOBILE 丁原「备诛」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, ObserverKind, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const DINGYUAN: u32 = 2_181_570_673;
const BEIZHU: u32 = 2_346_585_902;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/beizhu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("beizhu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
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
        EffectRequest::MoveCards {
            cards: draw_cards,
            to: removed,
            from: Some(draw),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BEIZHU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == BEIZHU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
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
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(setup: Setup, target: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let revision = setup.state.revision;
    apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
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
fn package_publishes_abi324_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 324);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == BEIZHU && skill.name == "beizhu"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == BEIZHU)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().is_some_and(|targets| {
        (targets.min, targets.max, targets.allow_self) == (1, 1, false)
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DINGYUAN)
        .unwrap();
    assert_eq!(character.name_key, "dingyuan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [BEIZHU]);
}

#[test]
fn target_contract_rejects_self_and_empty_hand_atomically() {
    for self_target in [true, false] {
        let setup = setup(2_026_073_201 + u64::from(self_target));
        let target = if self_target {
            setup.owner
        } else {
            setup.target
        };
        let before = canonical_state_hash(&setup.state);
        let (state, result) = activate(setup, target, 1);
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
        assert_eq!(canonical_state_hash(&state), before);
    }
}

#[test]
fn no_sha_branch_privately_reveals_discards_and_can_cancel_or_grant_draw_pile_sha() {
    let mut cancel = setup(2_026_073_203);
    let target = cancel.target;
    let owner = cancel.owner;
    let other = cancel.other;
    let hidden = inject(
        &mut cancel.state,
        target,
        ZoneKind::Hand,
        99_201,
        CardKind::Other,
    );
    let draw_sha = inject(
        &mut cancel.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_202,
        CardKind::Sha,
    );
    let (state, activated) = activate(cancel, target, 2);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [hidden.raw()]
    );
    let owner_view = project_observer(&state, ObserverKind::Player(owner));
    assert!(owner_view
        .known_cards
        .iter()
        .any(|card| card.id == hidden && card.def.is_some()));
    let other_view = project_observer(&state, ObserverKind::Player(other));
    assert!(other_view
        .known_cards
        .iter()
        .all(|card| card.id != hidden || card.def.is_none()));

    let (state, discarded) = resolve(state, owner, vec![hidden.raw()], false, 3);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let grant = discarded.choice.unwrap();
    assert_eq!(grant.kind, ChoiceKind::Custom);
    assert!(grant.allow_cancel);
    let (state, result) = resolve(state, owner, Vec::new(), true, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(draw_sha).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap()
    );

    let mut grant = setup(2_026_073_204);
    let target = grant.target;
    let owner = grant.owner;
    let hidden = inject(
        &mut grant.state,
        target,
        ZoneKind::Hand,
        99_203,
        CardKind::Other,
    );
    let draw_sha = inject(
        &mut grant.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_204,
        CardKind::ShaThunder,
    );
    let (state, _) = activate(grant, target, 5);
    let (state, _) = resolve(state, owner, vec![hidden.raw()], false, 6);
    let (state, result) = resolve(state, owner, vec![1], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    assert_eq!(state.cards.get(draw_sha).unwrap().zone, target_hand);
    assert_eq!(state.cards.get(draw_sha).unwrap().owner, target);
}

#[test]
fn all_snapshotted_sha_are_forced_and_direct_damage_draws_per_point() {
    let mut setup = setup(2_026_073_205);
    let owner = setup.owner;
    let target = setup.target;
    for def in [99_205, 99_206] {
        inject(&mut setup.state, target, ZoneKind::Hand, def, CardKind::Sha);
    }
    for def in [99_207, 99_208] {
        inject(
            &mut setup.state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            def,
            CardKind::Other,
        );
    }
    let hp = setup.state.players.get(owner).unwrap().hp;
    let (state, result) = activate(setup, target, 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn queued_sha_skips_stale_cards_and_survives_main_skill_removal() {
    let mut setup = setup(2_026_073_206);
    let owner = setup.owner;
    let target = setup.target;
    let stale = inject(
        &mut setup.state,
        target,
        ZoneKind::Hand,
        99_211,
        CardKind::Other,
    );
    let sha = inject(
        &mut setup.state,
        target,
        ZoneKind::Hand,
        99_212,
        CardKind::Sha,
    );
    inject(
        &mut setup.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_213,
        CardKind::Other,
    );
    let hp = setup.state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: setup.skill },
            EffectRequest::ResolveBeizhu {
                player: owner,
                target,
                stage: 2,
                cards: vec![stale, sha],
                skill: setup.skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), 1);
    assert_eq!(tx.state().hand_cards(target), [stale]);
}

#[test]
fn elemental_chain_damage_does_not_draw_for_the_propagated_target() {
    let mut setup = setup(2_026_073_207);
    let owner = setup.owner;
    let target = setup.target;
    let other = setup.other;
    inject(
        &mut setup.state,
        target,
        ZoneKind::Hand,
        99_209,
        CardKind::ShaFire,
    );
    inject(
        &mut setup.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        99_210,
        CardKind::Other,
    );
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: other,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let owner_hp = setup.state.players.get(owner).unwrap().hp;
    let other_hp = setup.state.players.get(other).unwrap().hp;
    let (state, result) = activate(setup, target, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(other), 0);
}
}

mod original_mobile_chengzhao_dongcheng {
//! Golden coverage for MOBILE 董承「承诏」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHENGZHAO: u32 = 3_452_131_931;
const CHENGZHAO_WIN: u32 = 2_503_452_290;
const CHENGZHAO_LOSE: u32 = 4_247_003_544;
const DONGCHENG: u32 = 3_178_823_789;
const TENGJIA4: u32 = 4_041_276_943;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/chengzhao.skill");
const ARMOR: &str = include_str!("../../../content/original/ported/mobile/rw_tengjia4.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_card: CardId,
    target_card: CardId,
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

fn setup(owner_rank: u8, target_rank: u8, gains: u32, seed: u64) -> Setup {
    let chengzhao = content_compiler::compile_source_named("chengzhao-test", SOURCE).unwrap();
    let armor = content_compiler::compile_source_named("chengzhao-armor-test", ARMOR).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&chengzhao).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&armor).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ending = players[1];
    let target = players[2];
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
            skill_def: SkillDefId::new(CHENGZHAO),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(TENGJIA4),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = ending;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_gained_by_player(owner), gains as i32);
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    let owner_card = inject(tx.state_mut(), owner, 99_401, owner_rank);
    let target_card = inject(tx.state_mut(), target, 99_402, target_rank);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        owner_card,
        target_card,
    }
}

fn trigger_end_phase(state: GameState) -> GameState {
    let current = state.turn.current;
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => {
            let result = run_effects(&mut tx, effects);
            assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
        }
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice {key}: {:?}",
        result.result,
    );
    state
}

fn compare(setup: Setup, key: u64) -> GameState {
    let state = trigger_end_phase(setup.state);
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(state, setup.owner, vec![1], key);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseHandCompareTarget
    );
    let state = resolve(state, setup.owner, vec![setup.target.raw()], key + 1);
    let state = resolve(state, setup.owner, vec![setup.owner_card.raw()], key + 2);
    resolve(state, setup.target, vec![setup.target_card.raw()], key + 3)
}

#[test]
fn package_publishes_dongcheng_and_all_chengzhao_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 325);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == DONGCHENG)
            .unwrap()
            .skills,
        [CHENGZHAO]
    );
    for skill in [CHENGZHAO, CHENGZHAO_WIN, CHENGZHAO_LOSE] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn trigger_requires_two_gained_cards_and_both_comparison_hands() {
    let setup = setup(13, 1, 1, 2_026_073_001);
    let state = trigger_end_phase(setup.state);
    assert!(!state.has_pending_choice());
}

#[test]
fn strict_win_uses_distance_unlimited_armor_ignoring_sha_and_cleans_helpers() {
    let setup = setup(13, 1, 2, 2_026_073_002);
    let owner = setup.owner;
    let target = setup.target;
    let state = compare(setup, 10);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert!(skill_id(&state, owner, CHENGZHAO_WIN).is_none());
    assert!(skill_id(&state, owner, CHENGZHAO_LOSE).is_none());
    assert!(!state.has_pending_choice());
}

#[test]
fn tie_or_loss_does_not_use_sha_and_cleans_helpers() {
    for (owner_rank, target_rank, key) in [(1, 13, 20), (7, 7, 30)] {
        let setup = setup(owner_rank, target_rank, 2, 2_026_073_000 + key);
        let owner = setup.owner;
        let target = setup.target;
        let state = compare(setup, key);
        assert_eq!(state.players.get(target).unwrap().hp, 4);
        assert!(skill_id(&state, owner, CHENGZHAO_WIN).is_none());
        assert!(skill_id(&state, owner, CHENGZHAO_LOSE).is_none());
        assert!(!state.has_pending_choice());
    }
}
}

mod original_mobile_daoji_hucheer {
//! Golden coverage for MOBILE 胡车儿「盗戟」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, EquipSlot, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DAOJI: u32 = 4_263_252_675;
const HUCHEER: u32 = 3_031_184_045;
const GAIN_GUARD: u32 = 4_263_252_676;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/daoji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
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
        suit: Suit::Club,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, gain_guard: bool) -> Setup {
    let source = if gain_guard {
        format!(
            "{SOURCE}\nskill gain_guard({GAIN_GUARD}) locked {{\n  modifier card_gain_protected(equip) override 1;\n}}\n"
        )
    } else {
        SOURCE.into()
    };
    let package = content_compiler::compile_source_named("daoji-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(DAOJI),
    });
    if gain_guard {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(GAIN_GUARD),
        });
    }
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
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == DAOJI)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, CommandResult) {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    (state, result.result)
}

fn activate(setup: &Setup, cost: CardId, key: u64) -> (GameState, CommandResult) {
    submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: vec![cost],
        },
        key,
    )
}

fn choose_equipment(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, CommandResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![card.raw()],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_hucheer_and_daoji() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 327);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUCHEER)
            .unwrap()
            .skills,
        [DAOJI]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DAOJI)
        .unwrap();
    assert!(active.targets.as_ref().unwrap().has_gainable_equip);
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_nonbasic")
    );
}

#[test]
fn weapon_is_gained_equipped_then_damages_original_owner() {
    let setup = setup(2_026_073_201, false);
    let mut tx = Transaction::begin(setup.state);
    let cost = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_601,
    );
    let weapon = inject(
        &mut tx,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        99_602,
    );
    let hp = tx.state().players.get(setup.target).unwrap().hp;
    let setup = Setup {
        state: tx.commit().state,
        ..setup
    };
    let (state, result) = activate(&setup, cost, 1);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveDaoji
    );
    let (state, result) = choose_equipment(state, setup.owner, weapon, 2);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let equip = state.zones.find(ZoneKind::Equip, setup.owner).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().owner, setup.owner);
    assert_eq!(state.cards.get(weapon).unwrap().zone, equip);
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
}

#[test]
fn nonweapon_causes_no_damage_and_disabled_weapon_slot_still_takes_damage() {
    for (kind, disable_slot, expected_in_hand, expected_damage, seed) in [
        (CardKind::Armor, false, false, 0, 2_026_073_202),
        (CardKind::Weapon, true, true, 1, 2_026_073_203),
    ] {
        let setup = setup(seed, false);
        let mut tx = Transaction::begin(setup.state);
        if disable_slot {
            tx.state_mut()
                .players
                .get_mut(setup.owner)
                .unwrap()
                .disabled_equip_slots[EquipSlot::Weapon.index()] = 1;
        }
        let cost = inject(
            &mut tx,
            setup.owner,
            ZoneKind::Hand,
            CardKind::Other,
            99_610,
        );
        let equipment = inject(&mut tx, setup.target, ZoneKind::Equip, kind, 99_611);
        let hp = tx.state().players.get(setup.target).unwrap().hp;
        let setup = Setup {
            state: tx.commit().state,
            ..setup
        };
        let (state, result) = activate(&setup, cost, u64::from(kind.tag()));
        assert!(matches!(result, CommandResult::Accepted { .. }));
        let (state, result) =
            choose_equipment(state, setup.owner, equipment, u64::from(kind.tag()) + 10);
        assert!(matches!(result, CommandResult::Accepted { .. }));
        let hand = state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
        assert_eq!(
            state.cards.get(equipment).unwrap().zone == hand,
            expected_in_hand
        );
        assert_eq!(
            state.players.get(setup.target).unwrap().hp,
            hp - expected_damage
        );
    }
}

#[test]
fn basic_cost_and_gain_protected_equipment_are_rejected() {
    {
        let setup = setup(2_026_073_204, false);
        let mut tx = Transaction::begin(setup.state);
        let basic = inject(&mut tx, setup.owner, ZoneKind::Hand, CardKind::Sha, 99_620);
        inject(
            &mut tx,
            setup.target,
            ZoneKind::Equip,
            CardKind::Armor,
            99_621,
        );
        let setup = Setup {
            state: tx.commit().state,
            ..setup
        };
        let (_, result) = activate(&setup, basic, 21);
        assert!(matches!(result, CommandResult::Rejected { .. }));
    }

    let setup = setup(2_026_073_205, true);
    let mut tx = Transaction::begin(setup.state);
    let cost = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_622,
    );
    inject(
        &mut tx,
        setup.target,
        ZoneKind::Equip,
        CardKind::Armor,
        99_623,
    );
    let setup = Setup {
        state: tx.commit().state,
        ..setup
    };
    let (_, result) = activate(&setup, cost, 22);
    assert!(matches!(result, CommandResult::Rejected { .. }));
}
}

mod original_mobile_jiaohua_liwei {
//! Golden coverage for MOBILE 李遗「教化」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIAOHUA: u32 = 4_264_685_853;
const LIWEI: u32 = 3_400_062_521;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/jiaohua.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: engine_api::SkillId,
}

fn inject(tx: &mut Transaction, zone_kind: ZoneKind, kind: CardKind, def: u32) -> CardId {
    let owner = PlayerId::NONE;
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jiaohua-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_073_301, "identity-standard").unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIAOHUA),
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
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == JIAOHUA)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
    }
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

fn activate(state: GameState, setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    submit(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![selection],
            cancelled: false,
        },
        key,
    )
}

fn tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_liwei_and_jiaohua() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 328);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LIWEI)
            .unwrap()
            .skills,
        [JIAOHUA]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JIAOHUA)
        .unwrap();
    assert_eq!(active.uses_per_play_phase, Some(2));
    assert!(active.targets.is_none());
    assert!(active.selection.is_none());
}

#[test]
fn category_cycle_persists_gains_from_draw_only_and_resets_after_all_three() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state.clone());
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
    let existing = tx.state().zones.get(draw).unwrap().cards.clone();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: existing,
            to: discard,
            from: Some(draw),
        }],
    )
    .unwrap();
    let lower_basic = inject(&mut tx, ZoneKind::DrawPile, CardKind::Sha, 99_701);
    inject(&mut tx, ZoneKind::DrawPile, CardKind::TrickWuzhong, 99_702);
    let top_basic = inject(&mut tx, ZoneKind::DrawPile, CardKind::Shan, 99_703);
    let discard_equipment = inject(&mut tx, ZoneKind::DiscardPile, CardKind::Armor, 99_704);
    let mut state = tx.commit().state;

    let (next, offered) = activate(state, &setup, 1);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(tokens(&offered), [1, 2, 4]);
    assert_eq!(
        next.resolution.continuation_kind,
        ContinuationKind::ResolveJiaohua
    );
    let (next, targets) = choose(next, setup.owner, 1, 2);
    assert_eq!(
        tokens(&targets),
        next.players
            .alive_by_seat()
            .iter()
            .map(|p| p.raw())
            .collect::<Vec<_>>()
    );
    let (next, gained) = choose(next, setup.owner, setup.other.raw(), 3);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    let other_hand = next.zones.find(ZoneKind::Hand, setup.other).unwrap();
    assert_eq!(next.cards.get(top_basic).unwrap().zone, other_hand);
    assert_ne!(next.cards.get(lower_basic).unwrap().zone, other_hand);

    let (next, offered) = activate(next, &setup, 4);
    assert_eq!(tokens(&offered), [2, 4]);
    let (next, targets) = choose(next, setup.owner, 4, 5);
    assert!(tokens(&targets).contains(&setup.owner.raw()));
    let owner_hand_before = next.hand_count(setup.owner);
    let (next, empty_gain) = choose(next, setup.owner, setup.owner.raw(), 6);
    assert!(matches!(empty_gain.result, CommandResult::Accepted { .. }));
    assert_eq!(next.hand_count(setup.owner), owner_hand_before);
    assert_eq!(
        next.cards.get(discard_equipment).unwrap().zone,
        next.zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (next, rejected) = activate(next, &setup, 7);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    state = next;
    state.skills.reset_play_phase_use_counts();

    let (next, offered) = activate(state, &setup, 8);
    assert_eq!(tokens(&offered), [2]);
    let (next, _) = choose(next, setup.owner, 2, 9);
    let (mut next, gained) = choose(next, setup.owner, setup.owner.raw(), 10);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    next.skills.reset_play_phase_use_counts();
    let (_, offered) = activate(next, &setup, 11);
    assert_eq!(tokens(&offered), [1, 2, 4]);
}
}

mod original_mobile_jiebing_yangfu {
//! Golden coverage for MOBILE 杨阜「借兵」(ABI 499).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIEBING: u32 = 3_832_378_886;
const HANNAN: u32 = 3_756_118_415;
const YANGFU: u32 = 2_919_026_286;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/jiebing.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn damage_excludes_source_and_equips_random_gain() {
    let packaged = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        packaged
            .character_index
            .iter()
            .find(|character| character.id == YANGFU)
            .unwrap()
            .skills,
        [JIEBING, HANNAN]
    );

    let package = content_compiler::compile_source_named("jiebing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 499);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, source, donor) = (seats[0], seats[1], seats[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIEBING),
    });
    run_effects(&mut tx, effects).unwrap();

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for kind in [ZoneKind::Hand, ZoneKind::Equip] {
        let zone = tx.state().zones.find(kind, donor).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        run_effects(
            &mut tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(zone),
                })
                .collect(),
        )
        .unwrap();
    }
    let donor_equip = tx.state().zones.find(ZoneKind::Equip, donor).unwrap();
    let (weapon, from) = tx
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .find(|card| card.kind == CardKind::Weapon)
        .map(|card| (card.id, card.zone))
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: donor_equip,
            from: Some(from),
        }],
    )
    .unwrap();

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    assert_eq!(tx.state().resolution.continuation_payload[2], source.raw());
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![donor.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert_eq!(
        state.cards.get(weapon).unwrap().owner,
        owner,
        "{:?}",
        state.cards.get(weapon).unwrap()
    );
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_equip);
}
}

mod original_mobile_jujun_bixian {
//! Golden for MOBILE「据峻 / 壁险」的阶段历史与无实体普通用牌流程。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUJUN: u32 = 3_855_454_532;
const BIXIAN: u32 = 4_274_044_152;
const O_CHENLAN: u32 = 2_348_532_465;
const O_MEICHENG: u32 = 2_493_526_902;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64, skill: u32) -> (Transaction, PlayerId, PlayerId) {
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
    let owner = engine.state().turn.current;
    let other = engine
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
        .map(|instance| EffectRequest::RemoveSkill { skill: instance.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        },
        EffectRequest::SetTurnState {
            current: other,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn target_owner(tx: &mut Transaction, user: PlayerId, owner: PlayerId, rank: u8) {
    let hand = tx.state().zones.find(ZoneKind::Hand, user).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner: user,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        tx,
        vec![EffectRequest::UseCard {
            player: user,
            card,
            targets: vec![owner],
        }],
    )
    .unwrap();
}

fn bit(player: PlayerId) -> i32 {
    1_i32 << (player.raw() - 1)
}

#[test]
fn package_declares_current_abi_skills_and_closed_characters() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    for (skill, event_filter) in [
        (
            JUJUN,
            "owner_alive_phase_targeted_hp_changed_can_use_wanjian",
        ),
        (
            BIXIAN,
            "owner_alive_phase_targeted_hp_unchanged_can_use_juedou",
        ),
    ] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill && entry.locked));
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == skill
                && trigger.event == "phase_any_end"
                && trigger.filter == event_filter
                && !trigger.optional
        }));
    }
    for (character, skill) in [(O_CHENLAN, JUJUN), (O_MEICHENG, BIXIAN)] {
        let character = package
            .character_index
            .iter()
            .find(|entry| entry.id == character)
            .unwrap();
        assert_eq!(character.group, "qun");
        assert_eq!(character.max_hp, 4);
        assert_eq!(character.skills, vec![skill]);
    }
}

#[test]
fn targeted_without_hp_change_forces_bixian_duel_target_choice() {
    let (mut tx, owner, other) = setup(20_260_723, BIXIAN);
    target_owner(&mut tx, other, owner, 3);
    assert_eq!(
        tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS) & bit(owner),
        bit(owner)
    );
    assert_eq!(
        tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) & bit(owner),
        0
    );

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Discard.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseUseVirtualCard
    );
    assert_eq!(tx.state().resolution.continuation_payload[2], 23);
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS), 0);
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS), 0);
}

#[test]
fn max_hp_change_without_actual_hp_change_still_forces_bixian() {
    let (mut tx, owner, other) = setup(20_260_726, BIXIAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    target_owner(&mut tx, other, owner, 5);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: owner,
            delta: -1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(
        tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) & bit(owner),
        0
    );

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Discard.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseUseVirtualCard
    );
}

#[test]
fn targeted_with_hp_change_forces_jujun_wanjian() {
    let (mut tx, owner, other) = setup(20_260_724, JUJUN);
    target_owner(&mut tx, other, owner, 4);
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
        tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) & bit(owner),
        bit(owner)
    );

    match run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: Phase::Discard.to_u8(),
        }],
    ) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("unexpected Wanjian resolution error: {error}"),
    }
    assert!(tx.state().cards.sorted_live().into_iter().any(|card| {
        card.def.is_none() && card.kind == CardKind::TrickWanjian && card.suit == Suit::None
    }));
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS), 0);
    assert_eq!(tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS), 0);
}

#[test]
fn hp_change_without_becoming_a_card_target_fires_neither_skill() {
    let (mut tx, owner, other) = setup(20_260_725, JUJUN);
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
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    assert!(!tx
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .any(|card| card.def.is_none() && card.kind == CardKind::TrickWanjian));
}
}

mod original_mobile_laishou {
//! Golden coverage for MOBILE 来敏「来寿」rewritten from legacy JavaScript semantics.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LAISHOU: u32 = 2_235_144_457;
const LAIMIN: u32 = 4_094_388_327;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/laishou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("laishou-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_073_302, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(LAISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 3;
    player.max_hp = 3;
    (tx, owner, source)
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, amount: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
}

#[test]
fn package_publishes_laimin_after_luanqun_closes_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LAISHOU && skill.locked));
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LAIMIN)
            .unwrap()
            .skills,
        [LAISHOU, 3_119_764_480]
    );
}

#[test]
fn lethal_damage_grows_max_hp_and_is_prevented_then_prepare_kills_at_nine() {
    let (mut tx, owner, source) = setup();
    damage(&mut tx, source, owner, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 3);

    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 3;
    player.max_hp = 3;
    damage(&mut tx, source, owner, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);

    damage(&mut tx, source, owner, 4);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 10);

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(!tx.state().players.get(owner).unwrap().alive);
}
}

mod original_mobile_luanqun_laimin {
//! Golden coverage for MOBILE 来敏「乱群」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LUANQUN: u32 = 3_119_764_480;
const LUANQUN_EFFECT: u32 = 3_973_186_966;
const LUANQUN_DIRECT_HIT: u32 = 3_537_894_821;
const LAISHOU: u32 = 2_235_144_457;
const LAIMIN: u32 = 4_094_388_327;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    mismatch: PlayerId,
    matching: PlayerId,
    skill: engine_api::SkillId,
    owner_card: CardId,
    mismatch_card: CardId,
    mismatch_second_sha: CardId,
    matching_card: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn setup() -> Setup {
    let source = [
        include_str!("../../../content/original/ported/mobile/luanqun.skill"),
        include_str!("../../../content/original/ported/mobile/luanqun_effect.skill"),
        include_str!("../../../content/original/ported/mobile/luanqun_directHit.skill"),
    ]
    .join("\n");
    let package = content_compiler::compile_source_named("luanqun-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_303, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mismatch = players[1];
    let matching = players[2];
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
        skill_def: SkillDefId::new(LUANQUN),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_card = inject(&mut tx, owner, CardKind::Shan, Suit::Heart, 99_801);
    let mismatch_card = inject(&mut tx, mismatch, CardKind::Sha, Suit::Club, 99_802);
    let mismatch_second_sha = inject(&mut tx, mismatch, CardKind::Sha, Suit::Spade, 99_803);
    let matching_card = inject(&mut tx, matching, CardKind::Tao, Suit::Diamond, 99_804);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == LUANQUN)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        mismatch,
        matching,
        skill,
        owner_card,
        mismatch_card,
        mismatch_second_sha,
        matching_card,
    }
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

fn choose(
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

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_laimin_and_both_hidden_helpers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 329);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LAIMIN)
            .unwrap()
            .skills,
        [LAISHOU, LUANQUN]
    );
    for skill in [LUANQUN, LUANQUN_EFFECT, LUANQUN_DIRECT_HIT] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn selections_stay_private_then_mismatch_restricts_first_sha_and_direct_hits_owner() {
    let setup = setup();
    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(
        activated
            .choice
            .as_ref()
            .unwrap_or_else(|| panic!("activation did not open a choice: {activated:?}"))
            .observer,
        setup.owner
    );
    assert!(!activated
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, first) = choose(state, setup.owner, vec![setup.owner_card.raw()], false, 2);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.mismatch);
    assert!(!first
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, second) = choose(
        state,
        setup.mismatch,
        vec![setup.mismatch_card.raw()],
        false,
        3,
    );
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.matching);
    assert!(!second
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, revealed) = choose(
        state,
        setup.matching,
        vec![setup.matching_card.raw()],
        false,
        4,
    );
    assert!(revealed.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::CardsRevealed { cards, to }
                if cards == &[setup.owner_card, setup.mismatch_card, setup.matching_card]
                    && to.is_empty()
        )
    }));
    assert_eq!(revealed.choice.as_ref().unwrap().observer, setup.owner);
    assert_eq!(
        revealed.choice.as_ref().unwrap().options[0].token,
        setup.matching_card.raw()
    );
    assert!(has_skill(&state, setup.mismatch, LUANQUN_EFFECT));
    assert!(has_skill(&state, setup.mismatch, LUANQUN_DIRECT_HIT));
    assert!(!has_skill(&state, setup.matching, LUANQUN_EFFECT));

    let (mut state, gained) = choose(
        state,
        setup.owner,
        vec![setup.matching_card.raw()],
        false,
        5,
    );
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(setup.matching_card).unwrap().owner,
        setup.owner
    );

    state.turn.current = setup.mismatch;
    state.turn.phase = Phase::Play;
    let (next, rejected) = submit(
        state,
        setup.mismatch,
        CommandKind::PlayCard {
            card: setup.mismatch_card,
            targets: vec![setup.matching],
            card_targets: Vec::new(),
        },
        6,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let owner_hp = next.players.get(setup.owner).unwrap().hp;
    let (mut next, direct_hit) = submit(
        next,
        setup.mismatch,
        CommandKind::PlayCard {
            card: setup.mismatch_card,
            targets: vec![setup.owner],
            card_targets: Vec::new(),
        },
        7,
    );
    assert!(matches!(direct_hit.result, CommandResult::Accepted { .. }));
    assert!(direct_hit.choice.is_none());
    assert_eq!(next.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert!(!has_skill(&next, setup.mismatch, LUANQUN_EFFECT));
    assert!(has_skill(&next, setup.mismatch, LUANQUN_DIRECT_HIT));

    next.turn.set_flag(turn_flag::SHA_PLAYED, 0);
    next.turn.set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let (mut next, unrestricted) = submit(
        next,
        setup.mismatch,
        CommandKind::PlayCard {
            card: setup.mismatch_second_sha,
            targets: vec![setup.matching],
            card_targets: Vec::new(),
        },
        8,
    );
    assert!(matches!(
        unrestricted.result,
        CommandResult::Accepted { .. }
    ));

    next.turn.phase = Phase::End;
    let (next, ended) = submit(next, setup.mismatch, CommandKind::EndPhase, 9);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&next, setup.mismatch, LUANQUN_DIRECT_HIT));
}
}

mod original_mobile_ly_piliche {
//! Golden coverage for MOBILE equipment skill「霹雳车」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LY_PILICHE: u32 = 2_383_388_842;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/ly_piliche.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LY_PILICHE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target)
}

fn add_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Diamond,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
}

fn confirm(
    state: GameState,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_contains_equipment_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LY_PILICHE && skill.name == "ly_piliche"));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LY_PILICHE
            && trigger.event == "damage"
            && trigger.filter == "owner_source_other_target_alive_has_discardable_hand_equip"
            && trigger.optional
    }));

    let compiled = content_compiler::compile_source_named("ly_piliche", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn accepting_discards_all_equipment_and_never_discards_target_hand() {
    let (state, owner, target) = setup(2_026_072_406);
    let mut tx = Transaction::begin(state);
    let weapon = add_card(
        &mut tx,
        target,
        ZoneKind::Equip,
        LY_PILICHE + 1,
        CardKind::Weapon,
    );
    let armor = add_card(
        &mut tx,
        target,
        ZoneKind::Equip,
        LY_PILICHE + 2,
        CardKind::Armor,
    );
    let hand = add_card(
        &mut tx,
        target,
        ZoneKind::Hand,
        LY_PILICHE + 3,
        CardKind::TrickWuzhong,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let (state, result) = damage(tx.commit().state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = confirm(state, owner, true, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    assert_eq!(state.cards.get(armor).unwrap().zone, discard);
    assert_eq!(state.cards.get(hand).unwrap().zone, target_hand);
    assert_eq!(state.cards.get(hand).unwrap().owner, target);
}

#[test]
fn hand_only_target_can_confirm_but_effect_is_a_noop() {
    let (state, owner, target) = setup(2_026_072_407);
    let mut tx = Transaction::begin(state);
    let hand = add_card(
        &mut tx,
        target,
        ZoneKind::Hand,
        LY_PILICHE + 4,
        CardKind::TrickWuzhong,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let (state, result) = damage(tx.commit().state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = confirm(state, owner, true, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(hand).unwrap().zone, target_hand);
}

#[test]
fn self_damage_and_empty_other_target_do_not_offer_the_skill() {
    let (state, owner, target) = setup(2_026_072_408);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let (state, other_result) = damage(tx.commit().state, owner, target);
    assert!(other_result.is_ok());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    add_card(
        &mut tx,
        owner,
        ZoneKind::Equip,
        LY_PILICHE + 5,
        CardKind::Armor,
    );
    let (state, self_result) = damage(tx.commit().state, owner, owner);
    assert!(self_result.is_ok());
    assert!(!state.has_pending_choice());
}
}

mod original_mobile_mbanda {
//! Golden coverage for mobile 吴珂「谙达」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const MBANDA: u32 = 2_849_838_127;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(cards: &[(Suit, ZoneKind)]) -> (Transaction, PlayerId, PlayerId, PlayerId, Vec<CardId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 2_026_073_101,
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
    let source = others[0];
    let target = others[1];
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
        skill_def: SkillDefId::new(MBANDA),
    });
    run_effects(&mut tx, effects).unwrap();
    clear_hand_and_equip(&mut tx, source);
    let cards = cards
        .iter()
        .enumerate()
        .map(|(index, (suit, zone))| inject(&mut tx, source, *suit, *zone, index as u32))
        .collect();
    {
        let participant = tx.state_mut().players.get_mut(target).unwrap();
        participant.hp = 0;
        participant.dying = true;
    }
    (tx, owner, source, target, cards)
}

fn clear_hand_and_equip(tx: &mut Transaction, player: PlayerId) {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(tx.state().zones.get(equip).unwrap().cards.iter().copied());
    }
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    suit: Suit,
    zone_kind: ZoneKind,
    offset: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(330_001 + offset),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn offer(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: target,
            source,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    (state, output.events)
}

fn mbanda_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == MBANDA)
        .unwrap()
        .id
}

#[test]
fn package_publishes_abi330_mbanda_trigger() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 440);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBANDA && skill.name == "mbanda" && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBANDA)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "dying",
            "owner_alive_source_alive_event_player_dying_once_per_round",
            true
        )
    );
}

#[test]
fn source_gives_different_colors_across_hand_and_equip_without_recovery() {
    let (mut tx, owner, source, target, cards) = setup(&[
        (Suit::Heart, ZoneKind::Hand),
        (Suit::Spade, ZoneKind::Equip),
    ]);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(offered.resolution.pending_observer, source.raw());
    let (given, events) = resolve(
        offered,
        source,
        cards.iter().map(|card| card.raw()).collect(),
        false,
        2,
    );
    assert!(cards
        .iter()
        .all(|card| given.cards.get(*card).unwrap().owner == target));
    assert_eq!(given.players.get(target).unwrap().hp, 0);
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { .. })));
    let skill = given.skills.get(mbanda_instance(&given, owner)).unwrap();
    assert_eq!(skill.round_fire_round, given.turn.round);

    let mut spent = Transaction::begin(given);
    assert!(offer(&mut spent, source, target).unwrap().is_empty());
}

#[test]
fn no_different_color_pair_recovers_immediately_after_activation() {
    let (mut tx, owner, source, target, _) = setup(&[
        (Suit::Heart, ZoneKind::Hand),
        (Suit::Diamond, ZoneKind::Equip),
    ]);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (recovered, events) = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert!(!recovered.has_pending_choice());
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)
    ));
}

#[test]
fn source_decline_recovers_and_same_color_pair_is_rejected() {
    let (mut tx, owner, source, target, cards) = setup(&[
        (Suit::Heart, ZoneKind::Hand),
        (Suit::Diamond, ZoneKind::Hand),
        (Suit::Spade, ZoneKind::Equip),
    ]);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 4);
    let choice_id = ChoiceId::new(offered.resolution.pending_choice_id);
    let revision = offered.revision;
    let (unchanged, output) = apply_command(
        offered,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![cards[0].raw(), cards[1].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Rejected { .. }));
    assert!(unchanged.has_pending_choice());

    let (recovered, _) = resolve(unchanged, source, Vec::new(), true, 6);
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
}
}

mod original_mobile_mbanxian {
//! Golden coverage for mobile 曹性「暗弦」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBANXIAN: u32 = 2_708_680_147;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_073_103,
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
        skill_def: SkillDefId::new(MBANXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 99);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::hand_discard_event_by_player(owner), 0);
    (tx, owner, others)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let (state, output) = apply_command(
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
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    (state, output.events)
}

#[test]
fn package_publishes_abi333_and_first_hand_discard_trigger() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 440);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBANXIAN)
        .unwrap();
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(
        trigger.filter,
        "owner_first_hand_discard_has_sha_in_discard"
    );
    assert!(trigger.optional);
}

#[test]
fn first_discarded_hand_sha_is_gained_then_used_ignoring_limits() {
    let (mut tx, owner, others) = setup();
    let cards = tx.state().hand_cards(owner);
    let sha = cards[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    assert!(engine_core::query::effective_card_kind(
        tx.state(),
        owner,
        tx.state().cards.get(sha).unwrap()
    )
    .is_sha());
    assert_eq!(
        tx.state()
            .turn
            .flag(turn_flag::hand_discard_event_by_player(owner)),
        0
    );
    let offered = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![sha],
        }],
    );
    assert!(engine_core::query::effective_card_kind(
        tx.state(),
        owner,
        tx.state().cards.get(sha).unwrap()
    )
    .is_sha());
    assert_eq!(offered.unwrap_err(), EffectError::ChoiceRequested);

    let (activated, _) = resolve(tx.state().clone(), owner, vec![1], 1);
    assert_eq!(activated.resolution.pending_observer, owner.raw());
    let (gained, _) = resolve(activated, owner, vec![sha.raw()], 2);
    assert!(gained.hand_cards(owner).contains(&sha));
    let distant_target = others[1];
    let (used, events) = resolve(gained, owner, vec![distant_target.raw()], 3);
    assert!(!used.hand_cards(owner).contains(&sha));
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::CardUsed { player, card, targets }
            if *player == owner && *card == sha && targets.contains(&distant_target))
    ));
}

#[test]
fn a_first_hand_discard_without_sha_blocks_later_sha() {
    let (mut tx, owner, _) = setup();
    let cards = tx.state().hand_cards(owner);
    let first = cards[0];
    let later_sha = cards[1];
    tx.state_mut().cards.get_mut(first).unwrap().kind = CardKind::TrickWuzhong;
    tx.state_mut().cards.get_mut(later_sha).unwrap().kind = CardKind::Sha;

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![first],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![later_sha],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(
        tx.state()
            .turn
            .flag(turn_flag::hand_discard_event_by_player(owner)),
        2
    );
}
}

mod original_mobile_mbhanyong {
//! Golden for MOBILE 兀突骨「悍勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBHANYONG: u32 = 2_639_258_372;
const MBRANSHANG: u32 = 3_354_404_150;
const MBRANSHANG_MARK: u32 = 1_206_920_502;
const MB_WUTUGU: u32 = 4_165_377_298;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mbhanyong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
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

fn started(seed: u64, wounded: bool) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MBHANYONG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    for card in draw_cards {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 15;
    owner_state.hp = if wounded { 14 } else { 15 };
    (tx.commit().state, owner, targets)
}

fn play(state: GameState, owner: PlayerId, card: CardId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.expect("expected choice");
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn activate(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, pipeline) = resolve(state, pipeline, owner, 1, key);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark
    );
    assert_eq!(
        pipeline
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    (state, pipeline)
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == MBRANSHANG_MARK).then_some(*count))
        .unwrap_or(0)
}

#[test]
fn package_publishes_abi114_skill_and_mobile_wutugu_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(
        content_compiler::compile_source_named("mbhanyong", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        114
    );
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBHANYONG)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_alive_wounded_card_targets_gt_one");
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == MB_WUTUGU)
        .unwrap();
    assert_eq!(character.max_hp, 15);
    assert_eq!(character.skills, [MBRANSHANG, MBHANYONG]);
}

#[test]
fn three_branches_draw_increase_damage_and_backwater_exactly() {
    for (branch, expected_hand, expected_damage, expected_mark) in
        [(1, 2, 1, 0), (2, 0, 2, 0), (3, 2, 2, 1)]
    {
        let (mut state, owner, targets) = started(20_260_724 + branch as u64, true);
        let wanjian = inject(&mut state, owner, CardKind::TrickWanjian, 99_100 + branch);
        let before: Vec<_> = targets
            .iter()
            .map(|target| state.players.get(*target).unwrap().hp)
            .collect();
        let (state, pipeline) = play(state, owner, wanjian, 10 + branch as u64);
        let (state, pipeline) = activate(state, pipeline, owner, 20 + branch as u64);
        let (state, pipeline) = resolve(state, pipeline, owner, branch, 30 + branch as u64);
        assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice());
        assert_eq!(state.hand_count(owner), expected_hand);
        assert_eq!(mark_count(&state, owner), expected_mark);
        for (target, hp) in targets.iter().zip(before.iter()) {
            assert_eq!(state.players.get(*target).unwrap().hp, hp - expected_damage);
        }
    }
}

#[test]
fn decline_full_health_and_non_damage_multi_target_preserve_legacy_gates() {
    let (mut state, owner, targets) = started(20_260_728, true);
    let wanjian = inject(&mut state, owner, CardKind::TrickWanjian, 99_201);
    let before = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, wanjian, 41);
    let (state, pipeline) = resolve(state, pipeline, owner, 0, 42);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(targets[0]).unwrap().hp, before - 1);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(mark_count(&state, owner), 0);

    let (mut state, owner, targets) = started(20_260_729, false);
    let wanjian = inject(&mut state, owner, CardKind::TrickWanjian, 99_202);
    let before = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, wanjian, 43);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    assert_eq!(state.players.get(targets[0]).unwrap().hp, before - 1);

    let (mut state, owner, _) = started(20_260_730, true);
    let wugu = inject(&mut state, owner, CardKind::TrickWugu, 99_203);
    let (mut state, mut pipeline) = play(state, owner, wugu, 44);
    let (next_state, next_pipeline) = activate(state, pipeline, owner, 45);
    state = next_state;
    pipeline = next_pipeline;
    let (next_state, next_pipeline) = resolve(state, pipeline, owner, 3, 46);
    state = next_state;
    pipeline = next_pipeline;
    // 独立锦囊队列（0615 起）逐目标排队取牌 choice；每个玩家（含 owner）各取一张。
    let mut key = 47;
    while let Some(choice) = pipeline.choice.clone() {
        let (s, p) = resolve(state, pipeline, choice.observer, choice.options[0].token, key);
        state = s;
        pipeline = p;
        key += 1;
    }
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark
    );
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(mark_count(&state, owner), 1);
}
}

mod original_mobile_mbjinzu {
//! Golden coverage for mobile 曹性「劲镞」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBJINZU: u32 = 3_257_523_591;
const MBJINZU_EFFECT: u32 = 4_156_343_864;
const MBANXIAN: u32 = 2_708_680_147;
const MB_CAOXING: u32 = 2_827_969_473;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    owner_reveal: CardId,
    sha: CardId,
    target_low: CardId,
    target_high: CardId,
    target_shan: CardId,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(owner_rank: u8, target_has_shan: bool) -> Setup {
    let source = [
        include_str!("../../../content/original/ported/mobile/mbjinzu.skill"),
        include_str!("../../../content/original/ported/mobile/mbjinzu_effect.skill"),
    ]
    .join("\n");
    let package = content_compiler::compile_source_named("mbjinzu-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_104, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(MBJINZU),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_reveal = inject(
        &mut tx,
        owner,
        CardKind::TrickWuzhong,
        Suit::Heart,
        owner_rank,
        99_901,
    );
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 6, 99_902);
    let target_low = inject(&mut tx, target, CardKind::Tao, Suit::Club, 3, 99_903);
    let target_high = inject(
        &mut tx,
        target,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        11,
        99_904,
    );
    let target_shan = if target_has_shan {
        inject(&mut tx, target, CardKind::Shan, Suit::Heart, 9, 99_905)
    } else {
        CardId::NONE
    };
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == MBJINZU)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
        owner_reveal,
        sha,
        target_low,
        target_high,
        target_shan,
    }
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

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
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

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_abi333_and_closes_caoxing() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == MB_CAOXING)
            .unwrap()
            .skills,
        [MBJINZU, MBANXIAN]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBJINZU_EFFECT && skill.locked));
}

#[test]
fn non_extreme_reveal_buffs_only_the_next_sha_against_that_target() {
    let setup = setup(7, true);
    let original_hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: vec![setup.owner_reveal],
        },
        1,
    );
    assert_eq!(activated.choice.as_ref().unwrap().observer, setup.target);
    assert_eq!(activated.choice.as_ref().unwrap().min_count, 2);
    let (state, revealed) = choose(
        state,
        setup.target,
        vec![setup.target_low.raw(), setup.target_high.raw()],
        2,
    );
    assert!(revealed.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to }
            if cards == &[setup.owner_reveal, setup.target_low, setup.target_high]
                && to.is_empty())
    }));
    assert!(has_skill(&state, setup.owner, MBJINZU_EFFECT));
    assert!(state.hand_cards(setup.target).contains(&setup.target_shan));

    let (state, played) = submit(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        3,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert!(played.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, original_hp - 2);
    assert!(!has_skill(&state, setup.owner, MBJINZU_EFFECT));
}

#[test]
fn unique_extreme_discards_all_reveals_and_refreshes_the_active_skill() {
    let setup = setup(13, false);
    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: vec![setup.owner_reveal],
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.choice.is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [setup.owner_reveal, setup.target_low, setup.target_high] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 0);
    assert!(!has_skill(&state, setup.owner, MBJINZU_EFFECT));
}
}

mod original_mobile_mbkuili {
//! Golden coverage for MOBILE 玄毌丘俭「溃离」rewritten from legacy JavaScript.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{EquipSlot, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBKUILI: u32 = 2_909_034_781;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mbkuili.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MBKUILI),
        }],
    )
    .unwrap();
    (tx, owner, source)
}

#[test]
fn package_exposes_locked_damage_source_weapon_restore_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    assert_eq!(package.trigger_index.len(), 198);
    assert_eq!(package.modifier_index.len(), 225);
    assert_eq!(package.active_index.len(), 47);
    assert_eq!(package.view_as_index.len(), 8);
    assert_eq!(package.modules.len(), 245);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBKUILI && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBKUILI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_is_target_source_alive_disabled_weapon"
    );
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("mbkuili", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 161);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == "mbkuili_damage"),
        compiled.modules.first()
    );
}

#[test]
fn damage_restores_exactly_one_disabled_weapon_slot_with_owner_attribution() {
    let (mut tx, owner, source) = setup(2_026_072_501);
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .disabled_equip_slots[EquipSlot::Weapon.index()] = 2;

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();

    assert_eq!(
        tx.state().players.get(source).unwrap().disabled_equip_slots[EquipSlot::Weapon.index()],
        1
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::EquipSlotEnabled {
                source: event_source,
                player,
                slot: 1
            } if *event_source == owner && *player == source
        )
    }));
}

#[test]
fn enabled_weapon_slot_or_unattributed_damage_does_not_fire() {
    let (mut tx, owner, source) = setup(2_026_072_502);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::EquipSlotEnabled { .. })));

    let (mut tx, owner, source) = setup(2_026_072_503);
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .disabled_equip_slots[EquipSlot::Weapon.index()] = 1;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().players.get(source).unwrap().disabled_equip_slots[EquipSlot::Weapon.index()],
        1
    );
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::EquipSlotEnabled { .. })));
}
}

mod original_mobile_mbshanjia_mb_caochun {
//! Golden coverage for MOBILE 曹纯「缮甲」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, TargetCardScope};

const MBSHANJIA: u32 = 3_457_547_943;
const MBSHANJIA_NODIS: u32 = 4_049_419_240;
const MB_CAOCHUN: u32 = 2_164_633_103;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mbshanjia.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
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

fn mark_count(state: &GameState, owner: PlayerId) -> u32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == MBSHANJIA)
        .map_or(0, |(_, count)| (*count).max(0) as u32)
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
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("mbshanjia-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(MBSHANJIA),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let (state, result) = apply_command(
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
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice {key}: {:?}",
        result.result
    );
    state
}

#[test]
fn package_publishes_mb_caochun_and_both_mbshanjia_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 326);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == MB_CAOCHUN)
            .unwrap()
            .skills,
        [MBSHANJIA]
    );
    for skill in [MBSHANJIA, MBSHANJIA_NODIS] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn equipment_loss_reduces_cost_and_noncategory_discards_grant_both_rewards() {
    let setup = setup(2_026_073_101);
    let mut tx = Transaction::begin(setup.state);
    let equipped = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        99_501,
    );
    let first = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_502,
    );
    let second = inject(
        &mut tx,
        setup.owner,
        ZoneKind::Hand,
        CardKind::Other,
        99_503,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: setup.owner,
            target: setup.owner,
            cards: vec![equipped],
            scope: TargetCardScope::Equip,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), setup.owner), 1);
    let skill = skill_id(tx.state(), setup.owner, MBSHANJIA).unwrap();
    let lost_equipment_count = mark_count(tx.state(), setup.owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbshanjia {
                player: setup.owner,
                lost_equipment_count,
                distance_skill_def: SkillDefId::new(MBSHANJIA_NODIS),
                stage: 0,
                reward_mask: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveMbshanjia
    );
    assert_eq!(state.resolution.continuation_payload[1], 2);

    let state = resolve(
        state,
        setup.owner,
        vec![first.raw(), second.raw()],
        false,
        1,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(state.resolution.continuation_payload[2], 0);
    assert_eq!(state.resolution.continuation_payload[3], 1);
    assert!(skill_id(&state, setup.owner, MBSHANJIA_NODIS).is_some());
    assert!(engine_core::query::card_distance_unlimited_for(
        &state,
        setup.owner
    ));

    let state = resolve(state, setup.owner, Vec::new(), true, 2);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: TurnPhaseTag::Play,
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
    assert!(skill_id(tx.state(), setup.owner, MBSHANJIA_NODIS).is_none());
}

#[test]
fn basic_and_trick_discards_suppress_their_rewards_independently() {
    for (kind, expect_distance, expect_sha, seed) in [
        (CardKind::Sha, true, false, 2_026_073_102),
        (CardKind::TrickWuzhong, false, true, 2_026_073_103),
    ] {
        let setup = setup(seed);
        let mut tx = Transaction::begin(setup.state);
        let selected = inject(&mut tx, setup.owner, ZoneKind::Hand, kind, 99_510);
        let skill = skill_id(tx.state(), setup.owner, MBSHANJIA).unwrap();
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveMbshanjia {
                    player: setup.owner,
                    lost_equipment_count: 2,
                    distance_skill_def: SkillDefId::new(MBSHANJIA_NODIS),
                    stage: 1,
                    reward_mask: 0,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let state = resolve(
            tx.commit().state,
            setup.owner,
            vec![selected.raw()],
            false,
            u64::from(kind.tag()),
        );
        assert_eq!(
            skill_id(&state, setup.owner, MBSHANJIA_NODIS).is_some(),
            expect_distance
        );
        assert_eq!(
            state.resolution.continuation_kind == ContinuationKind::VirtualShaTargets,
            expect_sha
        );
        if expect_sha {
            assert_eq!(state.resolution.continuation_payload[2], 0);
            assert_eq!(state.resolution.continuation_payload[3], 1);
        }
    }
}

#[test]
fn three_equipment_losses_require_no_discard_and_grant_both_rewards() {
    let setup = setup(2_026_073_104);
    let mut tx = Transaction::begin(setup.state);
    let skill = skill_id(tx.state(), setup.owner, MBSHANJIA).unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbshanjia {
                player: setup.owner,
                lost_equipment_count: 3,
                distance_skill_def: SkillDefId::new(MBSHANJIA_NODIS),
                stage: 0,
                reward_mask: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert!(skill_id(tx.state(), setup.owner, MBSHANJIA_NODIS).is_some());
}
}

mod original_mobile_mbyilie3 {
//! Golden for MOBILE「义烈」结束阶段子技能：消费全部“烈”并结算摸牌与失去体力。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const MBYILIE: u32 = 3_247_391_666;
const MBYILIE3: u32 = 2_964_107_248;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(mark_count: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MBYILIE3
            && trigger.event == "phase_end"
            && trigger.filter == "owner_mark_positive_3247391666"
            && !trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(MBYILIE3 as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: MBYILIE3 as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(MBYILIE3),
    });
    if mark_count > 0 {
        effects.push(EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(MBYILIE),
            delta: mark_count,
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 4;
    player.max_hp = 4;
    (tx, owner)
}

fn phase_end(owner: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    }
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == MBYILIE)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn positive_mark_draws_then_loses_equal_hp_and_clears_the_mark() {
    let (mut tx, owner) = setup(2);
    let hand = tx.state().hand_count(owner);

    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(tx.state().hand_count(owner), hand + 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn zero_mark_does_not_fire() {
    let (mut tx, owner) = setup(0);
    let hand = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    assert!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),)
            .unwrap()
            .is_empty()
    );
    assert_eq!(tx.state().hand_count(owner), hand);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
}
}

mod original_mobile_mbzhuguo_wuke {
//! Golden coverage for mobile 吴珂「助国」and the completed character entry.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const WUKE: u32 = 3_285_217_552;
const MBANDA: u32 = 2_849_838_127;
const MBZHUGUO: u32 = 2_303_782_932;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_073_102,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    // 批 269 起，孛明（文钦）game_started 触发会在开局打开选择；本测无关，
    // 由选择观察者取消以继续。
    if let Some(choice) = engine.last_choice() {
        let observer = choice.observer;
        let choice_id = choice.choice_id;
        assert!(matches!(
            engine.submit(Command {
                actor: observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(999_001),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
    }
    assert!(!engine.state().has_pending_choice());
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
        skill_def: SkillDefId::new(MBZHUGUO),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        clear_hand(&mut tx, player);
    }
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, others)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, offset: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(331_001 + offset),
        kind,
        suit: if offset % 2 == 0 {
            Suit::Heart
        } else {
            Suit::Spade
        },
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == MBZHUGUO)
        .unwrap()
        .id
}

fn begin(tx: &mut Transaction, owner: PlayerId, target: PlayerId) -> Result<(), EffectError> {
    let skill = skill_instance(tx.state(), owner);
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbzhuguo {
            player: owner,
            target,
            stage: 0,
            skill,
        }],
    )
    .map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
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
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    (state, output.events)
}

#[test]
fn package_publishes_current_abi_and_complete_wuke() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WUKE)
        .unwrap();
    assert_eq!(character.name_key, "wuke");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [MBANDA, MBZHUGUO]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == MBZHUGUO)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, true))
    );
}

#[test]
fn draw_to_cap_then_use_physical_sha_on_owner_selected_target() {
    let (mut tx, owner, others) = setup();
    let adjusted = others[0];
    let victim = others[2];
    let sha = inject(&mut tx, adjusted, CardKind::Sha, 0);
    let limit = tx.state().players.get(adjusted).unwrap().max_hp.min(5);
    assert_eq!(
        begin(&mut tx, owner, adjusted).unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().hand_count(adjusted), limit);

    let (target_chosen, _) = resolve(tx.state().clone(), owner, vec![victim.raw()], false, 1);
    assert_eq!(target_chosen.resolution.pending_observer, adjusted.raw());
    let (sha_chosen, _) = resolve(target_chosen, adjusted, vec![sha.raw()], false, 2);
    let (used, events) = resolve(sha_chosen, adjusted, vec![victim.raw()], false, 3);
    assert!(!used.hand_cards(adjusted).contains(&sha));
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::CardUsed { player, card, targets } if *player == adjusted && *card == sha && targets.contains(&victim))
    ));
}

#[test]
fn excess_hand_discards_to_cap_then_recovers() {
    let (mut tx, owner, others) = setup();
    let target = others[0];
    let limit = tx.state().players.get(target).unwrap().max_hp.min(5);
    for offset in 0..=limit {
        inject(&mut tx, target, CardKind::Sha, offset);
    }
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    assert_eq!(
        begin(&mut tx, owner, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let discard = tx.state().hand_cards(target)[0];
    let (resolved, events) = resolve(tx.state().clone(), target, vec![discard.raw()], false, 4);
    assert_eq!(resolved.hand_count(target), limit);
    assert_eq!(resolved.players.get(target).unwrap().hp, 3);
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)
    ));
}

#[test]
fn equal_hand_count_recovers_without_follow_up() {
    let (mut tx, owner, others) = setup();
    let target = others[0];
    let limit = tx.state().players.get(target).unwrap().max_hp.min(5);
    for offset in 0..limit {
        inject(&mut tx, target, CardKind::Sha, offset);
    }
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let skill = skill_instance(tx.state(), owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbzhuguo {
            player: owner,
            target,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(target).unwrap().hp, 3);
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)
    ));
}
}

mod original_mobile_mobiledanshou_xin_zhuran {
//! Golden coverage for MOBILE 手杀界朱然「胆守」(ABI 500).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MOBILEDANSHOU: u32 = 2_790_027_559;
const XIN_ZHURAN: u32 = 4_142_079_595;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mobiledanshou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("mobiledanshou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 500);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MOBILEDANSHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, target)
}

fn dispatch(tx: &mut Transaction, events: &[DomainEvent]) -> Result<(), EffectError> {
    let effects = dispatch_for_events(tx, events, &mut TriggerFireRecord::new())?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(state: GameState, owner: PlayerId, cards: Vec<u32>) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: cards,
                cancelled: false,
            },
        },
    )
}

#[test]
fn zero_targetings_draws_and_two_targetings_discard_then_damage() {
    let packaged = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(packaged.manifest.rule_vm_abi_version, 734);
    assert_eq!(
        packaged
            .character_index
            .iter()
            .find(|character| character.id == XIN_ZHURAN)
            .unwrap()
            .skills,
        [MOBILEDANSHOU]
    );

    let (mut tx, owner, target) = setup(20_260_802);
    let hand_before = tx.state().hand_count(owner);
    dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);

    let (mut tx, owner, target) = setup(20_260_803);
    let used = tx.state().hand_cards(target);
    for card in used.into_iter().take(2) {
        dispatch(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: target,
                card,
                targets: vec![owner],
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().turn.card_target_count(target, owner), 2);
    let payment: Vec<_> = tx.state().hand_cards(owner).into_iter().take(2).collect();
    let hp = tx.state().players.get(target).unwrap().hp;
    let result = dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::End,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let (state, result) = resolve(
        tx.commit().state,
        owner,
        payment.iter().map(|card| card.raw()).collect(),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(payment
        .iter()
        .all(|card| !state.hand_cards(owner).contains(card)));
}
}

mod original_mobile_mobilejingce {
//! Golden for MOBILE「精策」的实体用牌/响应弃置历史。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MOBILEJINGCE: u32 = 3_410_757_233;
const MOBILEJINGCE_COUNT: u32 = 3_753_058_142;
const XIN_GUOHUAI: u32 = 2_296_836_656;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(84_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
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

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(start(seed));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    remove_all_skills(&mut tx);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MOBILEJINGCE),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    (tx, owner, other)
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

fn pass_wuxie(
    mut state: GameState,
    mut pipeline: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult) {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipeline.choice.clone().expect("wuxie choice");
        key += 1;
        (state, pipeline) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    }
    (state, pipeline)
}

#[test]
fn package_declares_rewritten_skill_helper_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == MOBILEJINGCE && skill.name == "mobilejingce" && !skill.locked
    }));
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == MOBILEJINGCE_COUNT
            && skill.name == "mobilejingce_count"
            && skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MOBILEJINGCE
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_use_response_discarded_at_least_hp"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_GUOHUAI)
        .unwrap();
    assert_eq!(character.name_key, "xin_guohuai");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![MOBILEJINGCE]);
}

#[test]
fn used_and_responded_physical_cards_share_the_turn_threshold() {
    let (mut tx, owner, other) = setup(20_260_718);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let used = inject(&mut tx, owner, owner_hand, CardKind::Other, Suit::Heart, 3);
    let responded = inject(&mut tx, other, other_hand, CardKind::Shan, Suit::Club, 4);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: used,
            targets: vec![],
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![responded],
                to: discard,
                from: Some(other_hand),
            },
            EffectRequest::CardResponded {
                player: other,
                respond_to: owner,
                card: responded,
                response_kind: CardKind::Shan.tag(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::USE_RESPONSE_DISCARDED), 2);

    let hand_before = tx.state().hand_count(owner);
    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(MOBILEJINGCE as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 2);
}

#[test]
fn successful_delayed_trick_placement_does_not_count_as_discarded() {
    let (mut tx, owner, other) = setup(20_260_719);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    tx.state_mut().revision = 1;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let le = inject(&mut tx, owner, hand, CardKind::TrickLe, Suit::Heart, 6);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: le,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let (state, _) = pass_wuxie(state, pipeline, 1);
    assert_eq!(state.turn.flag(turn_flag::USE_RESPONSE_DISCARDED), 0);
    let judge = state.zones.find(ZoneKind::Judge, other).unwrap();
    assert!(state.zones.get(judge).unwrap().cards.contains(&le));
}

#[test]
fn nullified_delayed_trick_and_the_wuxie_both_count() {
    let (mut tx, owner, other) = setup(20_260_720);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    tx.state_mut().revision = 1;
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let le = inject(
        &mut tx,
        owner,
        owner_hand,
        CardKind::TrickLe,
        Suit::Spade,
        6,
    );
    let wuxie = inject(
        &mut tx,
        other,
        other_hand,
        CardKind::TrickWuxie,
        Suit::Club,
        12,
    );
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: le,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    let choice = pipeline.choice.unwrap();
    assert_eq!(choice.observer, other);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![wuxie.raw()],
                cancelled: false,
            },
        },
    );
    let (state, _) = pass_wuxie(state, pipeline, 2);
    assert_eq!(state.turn.flag(turn_flag::USE_RESPONSE_DISCARDED), 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = &state.zones.get(discard).unwrap().cards;
    assert!(cards.contains(&le));
    assert!(cards.contains(&wuxie));
}
}

mod original_mobile_redangxian {
//! Golden coverage for 手杀界廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REDANGXIAN: u32 = 4_137_190_076;
const REFULI: u32 = 2_837_551_482;
const XIN_LIAOHUA: u32 = 4_141_359_871;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/redangxian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
    draw_sha: CardId,
    discard_sha: CardId,
}

fn inject_shared_card(state: &mut GameState, zone_kind: ZoneKind, def: u32, suit: Suit) -> CardId {
    let zone = state.zones.find(zone_kind, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, REDANGXIAN as u64, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();

    let seats = tx.state().players.alive_by_seat();
    let previous = tx.state().turn.current;
    let previous_index = seats.iter().position(|player| *player == previous).unwrap();
    let owner = seats[(previous_index + 1) % seats.len()];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in seats {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REDANGXIAN),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let draw_sha = inject_shared_card(tx.state_mut(), ZoneKind::DrawPile, 94_101, Suit::Spade);
    let discard_sha =
        inject_shared_card(tx.state_mut(), ZoneKind::DiscardPile, 94_102, Suit::Heart);
    Setup {
        state: tx.commit().state,
        owner,
        previous,
        draw_sha,
        discard_sha,
    }
}

fn end_phase(state: GameState, actor: PlayerId) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("redangxian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 74);
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == REDANGXIAN && skill.name == "redangxian" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [REDANGXIAN, REFULI]);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn turn_begin_gains_sha_from_discard_only_then_opens_extra_play_phase() {
    let setup = setup();
    let (state, result) = end_phase(setup.state, setup.previous);

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert!(state.hand_cards(setup.owner).contains(&setup.discard_sha));
    assert!(!state.hand_cards(setup.owner).contains(&setup.draw_sha));

    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .contains(&setup.draw_sha));
    assert!(!state
        .zones
        .get(discard)
        .unwrap()
        .cards
        .contains(&setup.discard_sha));
}
}

mod original_mobile_refuhai {
//! Golden coverage for MOBILE 卫温诸葛直「浮海」.

use engine_api::{
    ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REFUHAI: u32 = 2_797_180_006;
const CHARACTER: u32 = 3_812_335_636;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

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

fn choose(state: GameState, actor: PlayerId, choice: u32, key: u64) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![choice],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 493);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == CHARACTER)
            .unwrap()
            .skills,
        [REFUHAI]
    );
}

#[test]
fn private_choices_reveal_together_and_draw_initial_equal_run() {
    let package = content_compiler::compile_source_named(
        "refuhai-test",
        include_str!("../../../content/original/ported/mobile/refuhai.skill"),
    )
    .unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 2_026_080_201, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
        skill_def: SkillDefId::new(REFUHAI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let before = tx.state().hand_count(owner);
    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(activated.choice.unwrap().observer, players[1]);
    let (state, first) = choose(state, players[1], 1, 2);
    assert_eq!(first.choice.unwrap().observer, players[2]);
    assert!(!first.events.iter().any(|event| matches!(
        event,
        DomainEvent::RuleNote {
            code: 0x4930_0001,
            ..
        }
    )));
    let (state, second) = choose(state, players[2], 1, 3);
    assert_eq!(second.choice.unwrap().observer, players[3]);
    let (state, final_result) = choose(state, players[3], 2, 4);
    assert!(final_result.events.iter().any(|event| matches!(
        event,
        DomainEvent::RuleNote {
            code: 0x4930_0001,
            ..
        }
    )));
    assert_eq!(state.hand_count(owner), before + 2);
}
}

mod original_mobile_refuli {
//! Golden for mobile 廖化「伏枥」rewritten from the legacy JavaScript semantics.

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

const REFULI: u32 = 2_837_551_482;
const WU: u32 = 4_065_000_621;
const SHU: u32 = 3_601_555_604;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn isolated(other_hp: [i32; 3]) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_716,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let mut players = vec![owner];
    players.extend(
        engine
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != owner),
    );
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, character_def) in players.iter().copied().zip([WU, WU, SHU, SHU]) {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character_def;
    }
    for (player, hp) in players.iter().copied().skip(1).zip(other_hp) {
        tx.state_mut().players.get_mut(player).unwrap().hp = hp;
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
        skill_def: SkillDefId::new(REFULI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: owner,
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
            expected_revision: ExpectedRevision(tx.state().revision),
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

fn has_refuli(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == REFULI)
}

#[test]
fn committed_package_exposes_optional_limited_dying_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 7);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REFULI)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("dying", "owner_is_target", true)
    );
    assert!(package
        .modules
        .iter()
        .any(|module| module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_QUERY,
                    rule_vm::bytecode::query_tag::UNIQUE_MAX_CURRENT_HP_AT,
                ]
        })));
}

#[test]
fn accepted_refuli_recovers_then_turns_over_only_when_strictly_highest() {
    let (mut tx, owner) = isolated([1, 1, 1]);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 1);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert!(accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_refuli(&accepted, owner));
}

#[test]
fn tied_current_hp_does_not_turn_over_but_still_consumes_refuli() {
    let (mut tx, owner) = isolated([2, 1, 1]);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 2);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert!(!accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_refuli(&accepted, owner));
}

#[test]
fn declining_refuli_preserves_hp_orientation_and_limited_skill() {
    let (mut tx, owner) = isolated([1, 1, 1]);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 3);
    assert_eq!(declined.players.get(owner).unwrap().hp, 0);
    assert!(!declined.players.get(owner).unwrap().turned_over);
    assert!(has_refuli(&declined, owner));
}
}

mod original_mobile_rejieming {
//! Golden coverage for MOBILE「节命」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REJIEMING: u32 = 4_088_661_605;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/rejieming.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("rejieming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_410, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let first = seats[2];
    let second = seats[3];
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
        skill_def: SkillDefId::new(REJIEMING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 6;
        participant.max_hp = 6;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        first,
        second,
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

fn damage(state: GameState, source: PlayerId, owner: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_contains_abi120_rewritten_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == REJIEMING)
        .unwrap();
    assert_eq!(skill.name, "rejieming");
    assert!(!skill.locked);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIEMING)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("rejieming", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn two_damage_points_choose_independent_targets_and_check_after_each_draw() {
    let setup = setup();
    let state = damage(setup.state, setup.source, setup.owner, 2);
    let (state, first_confirmed) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(
        first_confirmed.result,
        CommandResult::Accepted { .. }
    ));
    // 第一次触发的目标选择先于第二次触发的确认结算。
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp
    );

    let (state, first_drawn) = resolve(state, setup.owner, vec![setup.first.raw()], false, 3);
    assert!(matches!(first_drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 2);
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, second_confirmed) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(
        second_confirmed.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp
    );

    let (state, second_drawn) = resolve(state, setup.owner, vec![setup.second.raw()], false, 4);
    assert!(matches!(
        second_drawn.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.hand_count(setup.second), 2);
    assert_eq!(state.hand_count(setup.owner), 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn owner_does_not_draw_when_target_reaches_max_hp_hand_size() {
    let setup = setup();
    let mut state = setup.state;
    state.players.get_mut(setup.first).unwrap().max_hp = 2;
    let state = damage(state, setup.source, setup.owner, 1);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 5);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, drawn) = resolve(state, setup.owner, vec![setup.first.raw()], false, 6);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 2);
    assert_eq!(state.hand_count(setup.owner), 0);
    assert!(!state.has_pending_choice());
}

#[test]
fn declining_and_stale_target_leave_hands_unchanged() {
    let first_setup = setup();
    let state = damage(first_setup.state, first_setup.source, first_setup.owner, 1);
    let (state, declined) = resolve(state, first_setup.owner, vec![0], false, 7);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(first_setup.owner), 0);
    assert_eq!(state.hand_count(first_setup.first), 0);
    assert!(!state.has_pending_choice());

    let second_setup = setup();
    let state = damage(
        second_setup.state,
        second_setup.source,
        second_setup.owner,
        1,
    );
    let (mut state, confirmed) = resolve(state, second_setup.owner, vec![1], false, 8);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    state.players.get_mut(second_setup.first).unwrap().alive = false;
    let (_, stale) = resolve(
        state,
        second_setup.owner,
        vec![second_setup.first.raw()],
        false,
        9,
    );
    assert!(matches!(stale.result, CommandResult::Rejected { .. }));
}
}

mod original_mobile_rejiuchi_dongzhuo {
//! Golden coverage for MOBILE 界董卓「酒池」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REJIUCHI: u32 = 3_056_787_053;
const ROULIN: u32 = 3_168_437_611;
const BENGHUAI: u32 = 3_365_741_831;
const BAONUE: u32 = 3_659_854_073;
const RE_DONGZHUO: u32 = 3_597_733_258;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/mobile/rejiuchi.skill"),
    "\n",
    include_str!("../../../content/original/ported/shenhua/benghuai.skill")
);
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("rejiuchi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REJIUCHI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BENGHUAI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
    }
}

fn inject_sha(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(191_001),
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

#[test]
fn package_publishes_abi191_and_complete_re_dongzhuo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == REJIUCHI
            && view.as_kind == "jiu"
            && view.window == "play_phase"
            && view.filter == "spade"
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIUCHI)
        .unwrap();
    assert_eq!(
        trigger.filter,
        "owner_source_jiu_boosted_sha_damage_skill_3365741831_enabled"
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_DONGZHUO)
        .unwrap();
    assert!(character.is_lord);
    assert_eq!(character.max_hp, 8);
    assert_eq!(character.skills, [REJIUCHI, ROULIN, BENGHUAI, BAONUE]);
}

#[test]
fn jiu_boosted_sha_damage_disables_benghuai_until_next_turn() {
    let mut setup = setup(2_026_072_505);
    let sha = inject_sha(&mut setup.state, setup.owner);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: setup.owner,
            target: setup.target,
            card: sha,
            amount: 2,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: rule_api::damage_card_flag::JIU_BOOSTED,
        }],
    )
    .unwrap();
    let benghuai = skill_id(tx.state(), setup.owner, BENGHUAI);
    assert!(tx.state().skills.get(benghuai).unwrap().turn_disabled);
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(benghuai).unwrap()));

    tx.state_mut().players.get_mut(setup.owner).unwrap().hp = 3;
    tx.state_mut().players.get_mut(setup.target).unwrap().hp = 2;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
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
    assert!(effects.is_empty());

    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let benghuai = skill_id(&state, setup.owner, BENGHUAI);
    assert!(!state.skills.get(benghuai).unwrap().turn_disabled);
    assert!(state.skill_effectively_enabled(state.skills.get(benghuai).unwrap()));
}

#[test]
fn ordinary_sha_damage_keeps_benghuai_enabled() {
    let mut setup = setup(2_026_072_506);
    let sha = inject_sha(&mut setup.state, setup.owner);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: setup.owner,
            target: setup.target,
            card: sha,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    let benghuai = skill_id(tx.state(), setup.owner, BENGHUAI);
    assert!(!tx.state().skills.get(benghuai).unwrap().turn_disabled);
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(benghuai).unwrap()));
}
}

mod original_mobile_rw_tengjia3 {
//! Golden for MOBILE rewritten armor link protection.

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const RW_TENGJIA3: u32 = 4_247_798_542;
const RW_MINGUANGKAI_LINK: u32 = 2_481_253_205;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn loaded(skill_def: Option<u32>) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RW_TENGJIA3 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RW_TENGJIA3 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let removals = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, removals).unwrap();
    if let Some(skill_def) = skill_def {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            }],
        )
        .unwrap();
    }
    (tx, owner)
}

fn is_linked(tx: &Transaction, player: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

fn add_chain(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
}

#[test]
fn package_exposes_locked_armor_link_modifiers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def in [RW_MINGUANGKAI_LINK, RW_TENGJIA3] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def && skill.locked));
        let modifiers: Vec<_> = package
            .modifier_index
            .iter()
            .filter(|modifier| modifier.skill_def_id == skill_def)
            .collect();
        assert_eq!(modifiers.len(), 1);
        assert_eq!(modifiers[0].target, "armor_link_gain_cancel");
        assert_eq!(modifiers[0].value, 1);
    }
}

#[test]
fn active_armor_blocks_only_unlinked_to_linked_transition() {
    for skill_def in [RW_MINGUANGKAI_LINK, RW_TENGJIA3] {
        let (mut protected, owner) = loaded(Some(skill_def));
        assert!(engine_core::query::armor_link_gain_cancelled_for(
            protected.state(),
            owner
        ));
        add_chain(&mut protected, owner);
        assert!(!is_linked(&protected, owner));
    }

    let (mut ordinary, owner) = loaded(None);
    add_chain(&mut ordinary, owner);
    assert!(is_linked(&ordinary, owner));
    run_effects(
        &mut ordinary,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RW_TENGJIA3),
        }],
    )
    .unwrap();
    run_effects(
        &mut ordinary,
        vec![EffectRequest::RemoveMark {
            player: owner,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            amount: 0,
        }],
    )
    .unwrap();
    assert!(!is_linked(&ordinary, owner));
}
}

mod original_mobile_scsniqu {
//! Golden tests for MOBILE「逆取」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SCSNIQU: u32 = 2_407_745_929;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn started_state(seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    for player in [owner, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCSNIQU),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SCSNIQU)
        .unwrap();
    (tx.state().clone(), owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
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
                targets,
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn package_contains_the_active_contract_without_a_fake_selectable_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SCSNIQU)
        .expect("逆取 active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, true))
    );
    // scs_guosheng was added to the mobile pack by the upstream source update.
}

#[test]
fn deals_fire_damage_and_rejects_a_second_use_without_mutation() {
    let (state, owner, target, skill) = started_state(2_026_071_512);
    let hp = state.players.get(target).unwrap().hp;
    let (state, pipe) = activate(state, owner, skill, vec![target], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        } if *source == owner && *damaged == target
    )));

    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, vec![target], 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn missing_or_dead_target_is_rejected_without_mutation() {
    let (state, owner, target, skill) = started_state(2_026_071_513);
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, vec![], 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut dead = same;
    dead.players.get_mut(target).unwrap().alive = false;
    let before = canonical_state_hash(&dead);
    let (same, pipe) = activate(dead, owner, skill, vec![target], 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
}

mod original_mobile_scsxiaolu {
//! Golden coverage for MOBILE 韩悝组件「宵赂」rewritten from legacy JavaScript.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SCSXIAOLU: u32 = 3_320_236_272;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCSXIAOLU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SCSXIAOLU)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
    }
}

fn activate(setup: Setup, key: u64) -> (GameState, PipelineResult, u32, u32) {
    let owner_hand = setup.state.hand_count(setup.owner);
    let other_hand = setup.state.hand_count(setup.other);
    let revision = setup.state.revision;
    let (state, result) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    (state, result, owner_hand, other_hand)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
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

fn option_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_contains_active_without_publishing_hidden_hankui() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.manifest.capabilities.request_choice);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SCSXIAOLU && skill.name == "scsxiaolu"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SCSXIAOLU)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.is_none());
    // 十常侍 roster (08-04 round): hidden forms publish as minimal legal
    // characters; scs_hankui carries scsxiaolu.
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "scs_hankui"
            && character.skills.contains(&SCSXIAOLU)));
}

#[test]
fn give_branch_draws_then_gives_exactly_two_current_hand_cards() {
    let setup = setup(2_026_072_401);
    let owner = setup.owner;
    let other = setup.other;
    let skill = setup.skill;
    let (state, activated, owner_before, other_before) = activate(setup, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(option_tokens(&activated), [1, 2]);
    assert_eq!(state.hand_count(owner), owner_before + 2);

    let branch = activated.choice.unwrap();
    let (state, target_choice) = resolve(state, owner, branch.choice_id, vec![1], 2);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(option_tokens(&target_choice), [other.raw()]);

    let target = target_choice.choice.unwrap();
    let (state, card_choice) = resolve(state, owner, target.choice_id, vec![other.raw()], 3);
    assert!(matches!(card_choice.result, CommandResult::Accepted { .. }));
    let cards = option_tokens(&card_choice);
    assert!(cards.len() >= 2);

    let card_choice = card_choice.choice.unwrap();
    let (state, given) = resolve(state, owner, card_choice.choice_id, cards[..2].to_vec(), 4);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(state.hand_count(other), other_before + 2);

    let before = canonical_state_hash(&state);
    let revision = state.revision;
    let (same, second) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn discard_branch_discards_exactly_two_after_the_draw() {
    let setup = setup(2_026_072_402);
    let owner = setup.owner;
    let (state, activated, owner_before, _) = activate(setup, 10);
    let branch = activated.choice.unwrap();
    let (state, card_choice) = resolve(state, owner, branch.choice_id, vec![2], 11);
    assert!(matches!(card_choice.result, CommandResult::Accepted { .. }));
    let cards = option_tokens(&card_choice);
    assert!(cards.len() >= 2);

    let choice = card_choice.choice.unwrap();
    let (state, discarded) = resolve(state, owner, choice.choice_id, cards[..2].to_vec(), 12);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
}

#[test]
fn insufficient_hand_fallback_discards_every_current_discardable_hand_card() {
    let setup = setup(2_026_072_403);
    let keep = setup.state.hand_cards(setup.owner)[0];
    let discard: Vec<CardId> = setup
        .state
        .hand_cards(setup.owner)
        .into_iter()
        .filter(|card| *card != keep)
        .collect();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: setup.owner,
            cards: discard,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_cards(setup.owner), [keep]);

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player: setup.owner,
            draw_count: 0,
            payment_count: 2,
            skill: setup.skill,
            ready_to_choose: true,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(setup.owner), 0);
    assert!(!tx.state().has_pending_choice());
}
}

mod original_mobile_scsyaozhuo {
//! Golden coverage for MOBILE 十常侍组件「谣诼」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SCS_XIAYUN: u32 = 4_066_474_407;
const SCSYAOZHUO: u32 = 2_736_759_716;
const SCSYAOZHUO_WIN: u32 = 3_825_030_276;
const SCSYAOZHUO_LOSE: u32 = 3_028_413_936;
const SCSYAOZHUO_SKIP: u32 = 4_154_258_460;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/scsyaozhuo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_compare: CardId,
    target_compare: CardId,
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

fn setup(owner_rank: u8, target_rank: u8) -> Setup {
    let package = content_compiler::compile_source_named("scsyaozhuo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_073_001, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
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
            skill_def: SkillDefId::new(SCSYAOZHUO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let owner_compare = inject(tx.state_mut(), owner, 99_501, owner_rank);
    let target_compare = inject(tx.state_mut(), target, 99_502, target_rank);
    Setup {
        state: tx.commit().state,
        owner,
        target,
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
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
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
    command(
        state.clone(),
        actor,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled: false,
        },
        key,
    )
}

fn compare(setup: &Setup) -> (GameState, PipelineResult) {
    let active = skill_id(&setup.state, setup.owner, SCSYAOZHUO).unwrap();
    let (state, activated) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let (state, owner_selected) = resolve(state, setup.owner, vec![setup.owner_compare.raw()], 2);
    assert!(matches!(
        owner_selected.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    resolve(state, setup.target, vec![setup.target_compare.raw()], 3)
}

#[test]
fn package_publishes_four_rule_modules_without_exposing_hidden_xiayun() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def_id in [SCSYAOZHUO, SCSYAOZHUO_WIN, SCSYAOZHUO_LOSE, SCSYAOZHUO_SKIP] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def_id));
    }
    // 十常侍 roster: scs_xiayun publishes with 妖灼.
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == SCS_XIAYUN
            && character.skills.contains(&SCSYAOZHUO)));
}

#[test]
fn strict_win_skips_the_targets_next_draw_phase_once() {
    let setup = setup(13, 1);
    let (state, compared) = compare(&setup);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(skill_id(&state, setup.owner, SCSYAOZHUO_WIN).is_none());
    assert!(skill_id(&state, setup.target, SCSYAOZHUO_SKIP).is_some());

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.target;
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.target,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let events = run_effects(&mut tx, effects).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Draw,
        } if *current == setup.target
    )));
    assert!(skill_id(tx.state(), setup.target, SCSYAOZHUO_SKIP).is_none());
}

#[test]
fn tie_discards_every_remaining_card_when_fewer_than_two_exist() {
    let mut setup = setup(7, 7);
    let spare = inject(&mut setup.state, setup.owner, 99_503, 4);
    let (state, compared) = compare(&setup);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(skill_id(&state, setup.owner, SCSYAOZHUO_LOSE).is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(spare).unwrap().zone, discard);
    assert_eq!(state.hand_count(setup.owner), 0);
}
}

mod original_mobile_scszimou {
//! Golden for MOBILE 孙璋组件「自谋」的出牌阶段次数语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SCSZIMOU: u32 = 2_246_121_175;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup() -> (Transaction, PlayerId, engine_api::CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(SCSZIMOU as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: SCSZIMOU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let card = engine.state().hand_cards(owner)[0];
    let mut tx = Transaction::begin(engine.state().clone());
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
            skill_def: SkillDefId::new(SCSZIMOU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, card)
}

fn use_card(tx: &mut Transaction, owner: PlayerId, card: engine_api::CardId) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

fn gained_kinds(effects: &[EffectRequest]) -> Vec<u8> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            EffectRequest::GainCardByKind { kind, .. } => Some(*kind),
            _ => None,
        })
        .collect()
}

#[test]
fn package_declares_skill_without_publishing_the_hidden_component() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == SCSZIMOU && skill.name == "scszimou" && skill.locked
    }));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SCSZIMOU)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        vec![
            "owner_source_play_card_count_eq_2",
            "owner_source_play_card_count_eq_4",
            "owner_source_play_card_count_eq_6",
        ]
    );
    // 十常侍 roster: scs_sunzhang publishes with 子谋.
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "scs_sunzhang"
            && character.skills.contains(&SCSZIMOU)));
}

#[test]
fn second_fourth_and_sixth_uses_fire_and_each_play_phase_resets() {
    let (mut tx, owner, card) = setup();

    assert!(use_card(&mut tx, owner, card).is_empty());
    let responded = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card,
            response_kind: CardKind::Shan.tag(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(responded.is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::Jiu.tag()]
    );
    assert!(use_card(&mut tx, owner, card).is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::Sha.tag()]
    );
    assert!(use_card(&mut tx, owner, card).is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::TrickJuedou.tag()]
    );

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(use_card(&mut tx, owner, card).is_empty());
    assert_eq!(
        gained_kinds(&use_card(&mut tx, owner, card)),
        vec![CardKind::Jiu.tag()]
    );
}
}

mod original_mobile_shhlianhua {
//! Golden for mobile 莲花 (shhlianhua): locked — the owner draws 1 card when
//! targeted by a 【杀】.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SHHLIANHUA: u32 = 4_132_145_816;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_601,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) {
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
        skill_def: SkillDefId::new(SHHLIANHUA),
    });
    run_effects(tx, effects).unwrap();
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
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
fn committed_package_publishes_shhlianhua_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == SHHLIANHUA
            && t.event == "card_targeted"
            && t.filter == "owner_is_target_event_card_sha"
    }));
}

#[test]
fn being_targeted_by_sha_draws_one_card() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner);
    let sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    let hand_before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 1,
        "被【杀】指定为目标 → 摸一张"
    );
}

#[test]
fn being_targeted_by_a_trick_does_not_draw() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner);
    let trick = inject_hand(tx.state_mut(), other, CardKind::TrickWuzhong);
    let hand_before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: other,
            card: trick,
            target: owner,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "非【杀】目标不触发"
    );
}
}

mod original_mobile_xiahui_poxiang {
//! Golden coverage for mobile 侠怀（部分）+ 破降 upgrades (2026-08-21 placeholder
//! batch follow-up). Zero new ABI (mobile 500).

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBXIAHUI: u32 = 3_915_506_793;
const POXIANG: u32 = 3_023_488_106;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

const XIAHUI_SRC: &str = include_str!("../../../content/original/ported/mobile/mbxiahui.skill");
const POXIANG_SRC: &str = include_str!("../../../content/original/ported/mobile/poxiang.skill");

fn bootstrap(source: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId, SkillId) {
    let compiled = content_compiler::compile_source_named("upgrade-src", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx, owner, other, skill)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_101),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
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

#[test]
fn xiahui_publishes_black_ignored_hand_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == MBXIAHUI
            && modifier.target == "card_ignored_hand_limit"
            && modifier.selector.as_deref() == Some("black")
            && modifier.value == 1
    }));
    let compiled = content_compiler::compile_source_named("mbxiahui", XIAHUI_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == MBXIAHUI && modifier.target == "card_ignored_hand_limit"
    }));
}

#[test]
fn poxiang_gives_draws_and_loses_hp_in_order() {
    let (mut tx, owner, other, skill) = bootstrap(POXIANG_SRC, POXIANG);
    let gift = inject_hand(&mut tx, owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let revision = tx.state().revision;
    let (state, pipe) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(7),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    if !matches!(pipe.result, CommandResult::Accepted { .. }) {
        panic!("activate rejected: {:?}", pipe.result);
    }
    let mut state = state;
    // 选择链固定两步：选目标（other）→ 选要交出的牌（gift）。
    let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
    state = resolve(state, actor, vec![u32::from(other.raw())], 10);
    let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
    state = resolve(state, actor, vec![u32::from(gift.raw())], 11);
    assert_eq!(state.resolution.pending_choice_id, 0, "no further choices expected");
    let hand_other = state.zones.find(ZoneKind::Hand, other).unwrap();
    assert!(
        state.zones.get(hand_other).unwrap().cards.contains(&gift),
        "gift card should have moved to other's hand"
    );
    assert_eq!(state.hand_count(owner), 3, "should draw exactly three");
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
}
}

mod original_mobile_xiaoxi_hansui {
//! Golden for MOBILE 韩遂「骁袭」的主动与响应【杀】转换。

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

const XIAOXI_HANSUI: u32 = 2_531_367_472;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

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

fn add_skill(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIAOXI_HANSUI),
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
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|item| item.def.raw() == XIAOXI_HANSUI)
        })
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
        def: CardDefId::new(99_000 + kind.tag() as u32 * 20 + rank as u32),
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
fn package_exposes_both_black_hand_equip_view_as_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == XIAOXI_HANSUI
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "black"
                && view.source_zone.as_deref() == Some("hand_equip")
        }));
    }
}

#[test]
fn black_equipment_acts_as_proactive_sha_but_red_card_does_not() {
    let engine = package_engine(20_260_722);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, removals).unwrap();
    let skill = add_skill(&mut tx, actor);
    let red = inject(
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
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        3,
    );

    let (_, pipe) = apply_command(tx.state().clone(), activate(actor, skill, target, red, 1));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, source, 2),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(
        state.cards.get(source).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn duel_accepts_black_equipment_as_response_sha() {
    let engine = package_engine(20_260_723);
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
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
    let black = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        8,
    );
    let red = inject(
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
    let choice = pipe.choice.expect("骁袭应提供黑色装备牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == black.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == red.raw()));
}
}

mod original_mobile_xinanjian {
//! Golden coverage for MOBILE「暗箭」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINANJIAN: u32 = 2_652_033_709;
const TEST_MULTI_SHA: u32 = 2_652_033_710;
const XINANJIAN_SOURCE: &str =
    include_str!("../../../content/original/ported/mobile/xinanjian.skill");
/// Test-only helper (def 2652033710): grants the owner +1 sha target and makes
/// every multi-target sha target unable to respond, so the multi-target
/// scenario exercises only xinanjian picks without response choices.
const TEST_MULTI_SHA_SOURCE: &str = "\
// Test-only helper skill for multi-target sha isolation.
skill test_multi_sha(2652033710) {
  trigger card_targeted owner_source_sha_target_alive_owner_outside_target_attack_range {
    effect make_current_sha_target_unrespondable(self, event_card, event_player);
  }
  modifier sha_target_limit add 1;
}
";
const XIN_PANZHANGMAZHONG: u32 = 3_389_482_484;
/// Concatenate every committed mobile source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn mobile_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/mobile");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "skill")
        })
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(player_count: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("xinanjian-test", &format!("{XINANJIAN_SOURCE}\n{TEST_MULTI_SHA_SOURCE}")).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 2_026_072_409, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
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
        skill_def: SkillDefId::new(XINANJIAN),
    });
    if player_count > 4 {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TEST_MULTI_SHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, seats)
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
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult, CardId) {
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        XINANJIAN + key as u32,
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets,
                card_targets: vec![],
            },
        },
    );
    (state, result, sha)
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

fn equip_range_two(tx: &mut Transaction, owner: PlayerId, def: u32) {
    inject(tx, owner, ZoneKind::Equip, CardKind::Weapon, def);
}

#[test]
fn package_contains_abi120_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINANJIAN && skill.name == "xinanjian"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_PANZHANGMAZHONG)
        .unwrap();
    assert_eq!(character.name_key, "xin_panzhangmazhong");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [3_614_910_583, XINANJIAN]);

    let compiled = content_compiler::compile_source_named("xinanjian", &mobile_sources()).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn target_includes_owner_in_attack_range_so_skill_does_not_fire() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        XINANJIAN + 100,
    );
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![target], 1);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    assert!(state.resolution.vm_cont_module.is_empty());
}

#[test]
fn unrespondable_branch_skips_target_shan() {
    let (state, owner, seats) = setup(4);
    let target = seats[2];
    let mut tx = Transaction::begin(state);
    equip_range_two(&mut tx, owner, XINANJIAN + 200);
    inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        XINANJIAN + 201,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![target], 2);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, result) = resolve(state, owner, vec![1], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(state.hand_count(target), 1);
}

#[test]
fn damage_branch_keeps_response_and_adds_one_if_sha_hits() {
    let (state, owner, seats) = setup(4);
    let target = seats[2];
    let mut tx = Transaction::begin(state);
    equip_range_two(&mut tx, owner, XINANJIAN + 300);
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        XINANJIAN + 301,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![target], 4);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![2], false, 5);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    let (state, result) = resolve(state, target, vec![], true, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}

#[test]
fn multi_target_damage_bonus_changes_only_selected_target() {
    let (state, owner, seats) = setup(5);
    let first = seats[2];
    let second = seats[3];
    let mut tx = Transaction::begin(state);
    equip_range_two(&mut tx, owner, XINANJIAN + 400);
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let second_hp = tx.state().players.get(second).unwrap().hp;
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![first, second], 7);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![2], false, 8);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![1], false, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 2);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
}

#[test]
fn intent_rejects_a_stale_or_mismatched_sha_tuple() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let card = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        XINANJIAN + 500,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::IncreaseCurrentShaTargetDamage {
                source: owner,
                card,
                target,
                amount: 1,
            }],
        ),
        Err(EffectError::ContentFault(
            "Sha use refund requested outside a saved proactive Sha".into()
        ))
    );
}
}

mod original_mobile_xinduodao {
//! Golden coverage for MOBILE 界潘璋马忠「夺刀」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINDUODAO: u32 = 3_614_910_583;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/xinduodao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
        skill_def: SkillDefId::new(XINDUODAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, source)
}

fn add_equipment(tx: &mut Transaction, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 5,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
}

fn confirm(
    state: GameState,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_contains_skill_and_now_publishes_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == XINDUODAO && skill.name == "xinduodao" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINDUODAO
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_has_weapon"
            && trigger.optional
    }));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "xin_panzhangmazhong"));

    let compiled = content_compiler::compile_source_named("xinduodao", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn accepting_gains_every_current_weapon_and_leaves_other_equipment() {
    let (state, owner, source) = setup(2_026_072_404);
    let mut tx = Transaction::begin(state);
    let weapon_a = add_equipment(&mut tx, source, XINDUODAO + 1, CardKind::Weapon);
    let weapon_b = add_equipment(&mut tx, source, XINDUODAO + 2, CardKind::Weapon);
    let armor = add_equipment(&mut tx, source, XINDUODAO + 3, CardKind::Armor);
    let (state, result) = damage(tx.commit().state, source, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert!(state.has_pending_choice());

    let (state, accepted) = confirm(state, owner, true, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let source_equip = state.zones.find(ZoneKind::Equip, source).unwrap();
    for weapon in [weapon_a, weapon_b] {
        assert_eq!(state.cards.get(weapon).unwrap().zone, owner_hand);
        assert_eq!(state.cards.get(weapon).unwrap().owner, owner);
    }
    assert_eq!(state.cards.get(armor).unwrap().zone, source_equip);
    assert_eq!(state.cards.get(armor).unwrap().owner, source);
}

#[test]
fn no_weapon_opens_no_choice_and_declining_keeps_weapon_equipped() {
    let (state, owner, source) = setup(2_026_072_405);
    let mut tx = Transaction::begin(state);
    add_equipment(&mut tx, source, XINDUODAO + 4, CardKind::Armor);
    let (state, result) = damage(tx.commit().state, source, owner);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    let weapon = add_equipment(&mut tx, source, XINDUODAO + 5, CardKind::Weapon);
    let source_equip = tx.state().zones.find(ZoneKind::Equip, source).unwrap();
    let (state, result) = damage(tx.commit().state, source, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, declined) = confirm(state, owner, false, 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(weapon).unwrap().zone, source_equip);
    assert_eq!(state.cards.get(weapon).unwrap().owner, source);
}
}

mod original_mobile_xinjintao {
//! Golden for MOBILE「进讨」前两张杀的顺序修正。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINJINTAO: u32 = 2_174_751_337;
const XIN_WUBAN: u32 = 3_601_555_604;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn first_sha_is_unrespondable_and_second_sha_gains_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == XIN_WUBAN)
            .unwrap()
            .skills,
        [XINJINTAO]
    );

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XINJINTAO as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: XINJINTAO as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINJINTAO),
    });
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(XINJINTAO),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
    assert!(engine_core::query::card_distance_unlimited_for_target(
        tx.state(),
        owner,
        sha,
        target
    ));

    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    assert!(engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        sha
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, sha, 1),
        1
    );

    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 2);
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        sha
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, sha, 1),
        2
    );

    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 3);
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        sha
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, sha, 1),
        1
    );
}
}

mod original_mobile_xinpingkou {
//! Golden coverage for MOBILE 界朱桓「平寇」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XINPINGKOU: u32 = 3_441_378_011;
const XIN_ZHUHUAN: u32 = 4_000_463_828;
const FENLI: u32 = 2_848_697_859;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/xinpingkou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("xinpingkou", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(XINPINGKOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.record_skipped_phase();
    tx.state_mut().turn.record_skipped_phase();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others: [players[1], players[2], players[3]],
    }
}

fn inject_shared(state: &mut GameState, zone_kind: ZoneKind, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(zone_kind, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner: PlayerId::NONE,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn open_phase_end(state: GameState, owner: PlayerId) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    let event = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let new_revision = committed.state.revision;
    let state_hash = committed.state_hash;
    let events = committed.output.events;
    let choice = committed.output.choice;
    (
        committed.state,
        PipelineResult {
            result: CommandResult::Accepted {
                new_revision,
                state_hash,
            },
            events,
            choice,
        },
    )
}

#[test]
fn package_contains_abi124_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_ZHUHUAN)
        .unwrap();
    assert_eq!(character.name_key, "xin_zhuhuan");
    assert_eq!(character.skills, [FENLI, XINPINGKOU]);

    let compiled = content_compiler::compile_source_named("xinpingkou", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn damages_selected_targets_then_gains_draw_pile_equipment_without_discard_fallback() {
    let mut initial = setup(2_026_072_401);
    let draw_equipment = inject_shared(
        &mut initial.state,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        XINPINGKOU + 1,
    );
    let discard_equipment = inject_shared(
        &mut initial.state,
        ZoneKind::DiscardPile,
        CardKind::Armor,
        XINPINGKOU + 2,
    );
    let (state, confirm) = open_phase_end(initial.state, initial.owner);
    assert_eq!(confirm.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);

    let (state, target_offer) = resolve(state, initial.owner, vec![1], 1);
    let choice = target_offer.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));

    let first = initial.others[1];
    let second = initial.others[0];
    let first_hp = state.players.get(first).unwrap().hp;
    let second_hp = state.players.get(second).unwrap().hp;
    let (state, resolved) = resolve(state, initial.owner, vec![first.raw(), second.raw()], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
    let damage_order: Vec<_> = resolved
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::DamageDealt { source, target, .. } if *source == initial.owner => {
                Some(*target)
            }
            _ => None,
        })
        .collect();
    assert_eq!(damage_order, [first, second]);
    let hand = state.zones.find(ZoneKind::Hand, initial.owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(draw_equipment).unwrap().zone, hand);
    assert_eq!(state.cards.get(discard_equipment).unwrap().zone, discard);

    let mut no_draw_equipment = setup(2_026_072_402);
    let draw = no_draw_equipment
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = no_draw_equipment
        .state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .clone();
    for card in draw_cards {
        let state = no_draw_equipment.state.cards.get_mut(card).unwrap();
        if state.kind.is_equip() {
            state.kind = CardKind::Sha;
            state.weapon_range = 0;
        }
    }
    let discard_only = inject_shared(
        &mut no_draw_equipment.state,
        ZoneKind::DiscardPile,
        CardKind::Weapon,
        XINPINGKOU + 3,
    );
    let before = no_draw_equipment.state.hand_count(no_draw_equipment.owner);
    let mut tx = Transaction::begin(no_draw_equipment.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::GainEquipmentFromDrawPile {
            player: no_draw_equipment.owner,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(no_draw_equipment.owner), before);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(discard_only).unwrap().zone, discard);
}
}

mod original_mobile_xinqieting_xin_caifuren {
//! Golden coverage for MOBILE 界蔡夫人「窃听」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId, SkillDefId, Suit,
    TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const XIN_CAIFUREN: u32 = 3_572_525_059;
const XINQIETING: u32 = 3_722_360_517;
const XIANZHOU: u32 = 4_217_889_451;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/xinqieting.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    ending: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("xinqieting-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ending = players[1];
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
        skill_def: SkillDefId::new(XINQIETING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = ending;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        ending,
    }
}

fn inject_card(
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
        suit: Suit::Spade,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn trigger(
    state: GameState,
    ending: PlayerId,
    current_dealt_damage_other: bool,
) -> (GameState, Option<ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: true,
            current_dealt_damage_other,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
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
fn package_publishes_abi323_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 323);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINQIETING && skill.name == "xinqieting"));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINQIETING)
        .unwrap();
    assert_eq!(trigger.event, "turn_ended");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_turn_ended_no_damage_other"
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_CAIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "xin_caifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [XINQIETING, XIANZHOU]);

    let compiled = content_compiler::compile_source_named("xinqieting", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn damage_history_blocks_while_draw_and_cancel_remain_available() {
    let blocked = setup(2_026_073_101);
    let (state, choice) = trigger(blocked.state, blocked.ending, true);
    assert!(choice.is_none());
    assert!(!state.has_pending_choice());

    let draw = setup(2_026_073_102);
    let hand = draw.state.hand_count(draw.owner);
    let (state, choice) = trigger(draw.state, draw.ending, false);
    let choice = choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let (state, result) = resolve(state, draw.owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(draw.owner), hand + 1);

    let cancel = setup(2_026_073_103);
    let (state, choice) = trigger(cancel.state, cancel.ending, false);
    assert!(choice.unwrap().allow_cancel);
    let (state, result) = resolve(state, cancel.owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn hand_branch_reveals_only_two_and_rejects_an_uninspected_card_atomically() {
    let mut setup = setup(2_026_073_104);
    let cards: Vec<_> = [99_101, 99_102, 99_103]
        .into_iter()
        .map(|def| {
            inject_card(
                &mut setup.state,
                setup.ending,
                ZoneKind::Hand,
                def,
                CardKind::Other,
            )
        })
        .collect();
    let (state, choice) = trigger(setup.state, setup.ending, false);
    assert_eq!(
        choice
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, branch) = resolve(state, setup.owner, vec![2], false, 3);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    let choice = branch.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(!choice.allow_cancel);
    let inspected: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(inspected.len(), 2);

    let view = project_observer(&state, ObserverKind::Player(setup.owner));
    assert!(inspected.iter().all(|card| {
        view.known_cards
            .iter()
            .any(|known| known.id.raw() == *card && known.def.is_some())
    }));
    let hidden = cards
        .iter()
        .find(|card| !inspected.contains(&card.raw()))
        .unwrap();
    assert!(view.known_cards.iter().all(|known| known.id != *hidden));

    let before = canonical_state_hash(&state);
    let (unchanged, rejected) = resolve(state.clone(), setup.owner, vec![hidden.raw()], false, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before);

    let gained = CardId::new(inspected[0]);
    let (state, result) = resolve(state, setup.owner, vec![gained.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(gained).unwrap().owner, setup.owner);
    assert_eq!(state.hand_count(setup.ending), 2);
}

#[test]
fn compatible_equipment_can_move_to_the_owner() {
    let mut setup = setup(2_026_073_105);
    let weapon = inject_card(
        &mut setup.state,
        setup.ending,
        ZoneKind::Equip,
        99_111,
        CardKind::Weapon,
    );
    let (state, choice) = trigger(setup.state, setup.ending, false);
    assert_eq!(
        choice
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, branch) = resolve(state, setup.owner, vec![0], false, 6);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(branch.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, result) = resolve(state, setup.owner, vec![weapon.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_equip = state.zones.find(ZoneKind::Equip, setup.owner).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_equip);
    assert_eq!(state.cards.get(weapon).unwrap().owner, setup.owner);
}
}

mod original_mobile_xinqingxi_xin_caoxiu {
//! Golden coverage for MOBILE 界曹休「倾袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIN_CAOXIU: u32 = 2_884_167_754;
const QIANJU: u32 = 2_828_017_083;
const XINQINGXI: u32 = 2_962_941_234;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/xinqingxi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    seats: Vec<PlayerId>,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("xinqingxi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
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
        skill_def: SkillDefId::new(XINQINGXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        seats,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
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

fn deal_damage(
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
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn accept(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let (state, result) = deal_damage(state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    resolve(state, owner, vec![1], false, key)
}

#[test]
fn package_publishes_abi322_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 322);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINQINGXI && skill.name == "xinqingxi"));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINQINGXI)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_source_other_damage_before_once_per_turn"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_CAOXIU)
        .unwrap();
    assert_eq!(character.name_key, "xin_caoxiu");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [QIANJU, XINQINGXI]);

    let compiled = content_compiler::compile_source_named("xinqingxi", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn distance_two_target_discards_exactly_two_for_non_sha_damage() {
    let mut setup = setup(2_026_073_001);
    let target = setup.seats[2];
    let cards: Vec<_> = [99_001, 99_002, 99_003]
        .into_iter()
        .map(|def| inject_hand(&mut setup.state, target, def))
        .collect();
    let (state, accepted) = accept(setup.state, setup.owner, target, 1);
    let choice = accepted.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.min_count, 2);
    assert_eq!(choice.max_count, 2);
    let (state, result) = resolve(
        state,
        target,
        vec![cards[0].raw(), cards[1].raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 9);
    assert_eq!(state.hand_count(target), 1);
}

#[test]
fn insufficient_hand_or_cancel_increases_the_bound_damage() {
    let mut insufficient = setup(2_026_073_002);
    let target = insufficient.seats[1];
    for def in [99_011, 99_012] {
        inject_hand(&mut insufficient.state, target, def);
    }
    let (state, result) = accept(insufficient.state, insufficient.owner, target, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(state.hand_count(target), 2);

    let mut cancelled = setup(2_026_073_003);
    let target = cancelled.seats[1];
    for def in [99_021, 99_022, 99_023] {
        inject_hand(&mut cancelled.state, target, def);
    }
    let (state, accepted) = accept(cancelled.state, cancelled.owner, target, 4);
    assert_eq!(accepted.choice.unwrap().min_count, 3);
    let (state, result) = resolve(state, target, Vec::new(), true, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(state.hand_count(target), 3);
}

#[test]
fn accepted_trigger_is_limited_to_once_in_the_current_turn() {
    let mut setup = setup(2_026_073_004);
    let target = setup.seats[2];
    for def in [99_031, 99_032] {
        inject_hand(&mut setup.state, target, def);
    }
    let (state, _) = accept(setup.state, setup.owner, target, 6);
    let (state, result) = resolve(state, target, Vec::new(), true, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 8);

    let (state, result) = deal_damage(state, setup.owner, target);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, 7);
}

#[test]
fn increased_elemental_damage_updates_chain_base_without_retriggering() {
    let mut setup = setup(2_026_073_005);
    let target = setup.seats[2];
    let other = setup.seats[3];
    for def in [99_041, 99_042] {
        inject_hand(&mut setup.state, target, def);
    }
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: target,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: other,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let mut tx = Transaction::begin(tx.commit().state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::TrickJuedou.tag(),
            card_flags: 0,
        }],
    );
    let state = tx.commit().state;
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, setup.owner, vec![1], false, 8);
    let (state, result) = resolve(state, target, Vec::new(), true, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(state.players.get(other).unwrap().hp, 8);
}
}

mod original_mobile_yingjian {
//! Golden for MOBILE 孙茹「影箭」的无实体【杀】语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SUNRU: u32 = 3_566_559_635;
const YINGJIAN: u32 = 3_382_160_794;
const SHIXIN: u32 = 3_071_284_185;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn start(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, far)
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
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
fn mobile_package_contains_complete_sunru() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let sunru = package
        .character_index
        .iter()
        .find(|character| character.id == SUNRU)
        .unwrap();
    assert_eq!(sunru.name_key, "sunru");
    assert_eq!(sunru.max_hp, 3);
    assert_eq!(sunru.skills, vec![YINGJIAN, SHIXIN]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YINGJIAN));
}

#[test]
fn virtual_sha_distance_policy_controls_target_options_and_cancel_cleans_up() {
    let (mut normal, owner, far) = start(20_260_716);
    assert_eq!(
        run_effects(
            &mut normal,
            vec![EffectRequest::ChooseUseVirtualSha {
                player: owner,
                ignore_distance: false,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let virtual_card = normal
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .and_then(|zone| normal.state().zones.get(zone))
        .and_then(|zone| zone.cards.last())
        .copied()
        .unwrap();
    let committed = normal.commit();
    let choice = committed.output.choice.unwrap();
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == far.raw()));
    let (state, result) = resolve(committed.state, owner, choice.choice_id, vec![], true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let card = state.cards.get(virtual_card).unwrap();
    assert!(card.def.is_none());
    assert_eq!(card.suit, Suit::None);
    assert_eq!(state.zones.get(card.zone).unwrap().kind, ZoneKind::Removed);

    let (mut unlimited, owner, far) = start(20_260_717);
    assert_eq!(
        run_effects(
            &mut unlimited,
            vec![EffectRequest::ChooseUseVirtualSha {
                player: owner,
                ignore_distance: true,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = unlimited.commit();
    let choice = committed.output.choice.as_ref().unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == far.raw()));
}

#[test]
fn yingjian_uses_the_normal_sha_response_damage_and_history_pipeline() {
    let (mut tx, owner, far) = start(20_260_718);
    let response_card = tx.state().hand_cards(far)[0];
    tx.state_mut().cards.get_mut(response_card).unwrap().kind = CardKind::Shan;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YINGJIAN),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(far).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    let target_choice = committed.output.choice.unwrap();
    assert!(target_choice
        .options
        .iter()
        .any(|option| option.token == far.raw()));

    let (state, target_result) = resolve(
        committed.state,
        owner,
        target_choice.choice_id,
        vec![far.raw()],
        false,
        2,
    );
    assert!(matches!(
        target_result.result,
        CommandResult::Accepted { .. }
    ));
    let response = target_result.choice.unwrap();
    assert_eq!(response.observer, far);
    let (state, response_result) = resolve(state, far, response.choice_id, vec![], true, 3);
    assert!(matches!(
        response_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(far).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);

    let removed = state.zones.find(ZoneKind::Removed, PlayerId::NONE).unwrap();
    let virtual_sha = state
        .zones
        .get(removed)
        .unwrap()
        .cards
        .iter()
        .copied()
        .find(|card| {
            state.cards.get(*card).is_some_and(|card| {
                card.def.is_none() && card.kind == CardKind::Sha && card.suit == Suit::None
            })
        })
        .unwrap();
    assert_eq!(state.cards.get(virtual_sha).unwrap().owner, PlayerId::NONE);
}
}

mod original_mobile_zhaohuo {
//! Golden for MOBILE「招祸」的全局濒死语义。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{draw_count_for, optional_draw_bonuses_for, query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content, Engine};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZHAOHUO: u32 = 2_173_746_528;
const REJIANGCHI: u32 = 3_537_053_421;
const REJIANGCHI3: u32 = 2_242_073_380;
const XIN_CAOZHANG: u32 = 3_739_830_718;
const HUAIZI: u32 = 4_199_126_566;
const MBRANSHANG: u32 = 3_354_404_150;
const MBRANSHANG_MARK: u32 = 1_206_920_502;
const TIANSUAN2_DAMAGE: u32 = 2_231_135_297;
const TIANSUAN2_0: u32 = 3_462_769_895;
const TIANSUAN2_1: u32 = 3_136_054_404;
const TIANSUAN2_2: u32 = 2_298_819_832;
const TIANSUAN2_3: u32 = 3_501_902_691;
const TIANSUAN2_4: u32 = 3_604_116_303;
const DUJIN: u32 = 3_000_807_303;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn grant(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHAOHUO),
        }],
    )
    .unwrap();
}

#[test]
fn rejiangchi3_cancels_the_proactive_sha_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
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
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    grant_skill(&mut tx, owner, REJIANGCHI3);

    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 0);
}

#[test]
fn rejiangchi_publishes_the_play_phase_choice_and_complete_character() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == XIN_CAOZHANG)
        .unwrap();
    assert_eq!(character.skills, [REJIANGCHI]);

    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIANGCHI)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);

    let cleanup = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIANGCHI3)
        .unwrap();
    assert_eq!(cleanup.event, "phase_play_end");
    assert!(!cleanup.optional);
}

fn grant_skill(tx: &mut Transaction, owner: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

#[test]
fn dujin_draw_bonus_uses_half_the_equip_count_rounded_up_plus_one() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(DUJIN as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: DUJIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    let base = draw_count_for(tx.state(), owner);
    grant_skill(&mut tx, owner, DUJIN);
    assert_eq!(draw_count_for(tx.state(), owner), base);
    assert_eq!(optional_draw_bonuses_for(tx.state(), owner), vec![1]);

    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    for (index, expected) in [2, 2, 3].into_iter().enumerate() {
        let card = tx.state_mut().cards.alloc(Card {
            id: engine_api::CardId::NONE,
            def: CardDefId::new(30_000 + index as u32),
            kind: CardKind::Treasure,
            suit: Suit::Diamond,
            rank: 1,
            weapon_range: 0,
            zone: equip,
            owner,
            face_up: true,
        });
        tx.state_mut()
            .zones
            .get_mut(equip)
            .unwrap()
            .cards
            .push(card);
        assert_eq!(draw_count_for(tx.state(), owner), base);
        assert_eq!(optional_draw_bonuses_for(tx.state(), owner), vec![expected]);
    }
}

fn equip_tengjia(tx: &mut Transaction, owner: PlayerId) {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(content::DEF_TENGJIA),
        kind: CardKind::Armor,
        suit: Suit::Club,
        rank: 2,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
}

#[test]
fn tiansuan2_damage_is_optional_and_draws_the_resolved_damage_amount() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_DAMAGE as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_DAMAGE as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_DAMAGE);
    let hand = tx.state().hand_count(owner);

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(tx.state().hand_count(owner), hand);

    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);
}

#[test]
fn tiansuan2_0_prevents_damage_but_not_lose_hp() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_0 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_0 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_0);
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn tiansuan2_1_caps_final_fire_damage_then_offers_one_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_1 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_1 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_1);
    equip_tengjia(&mut tx, owner);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), hand);

    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let mut lose_hp = Transaction::begin(state);
    run_effects(
        &mut lose_hp,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: true,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(lose_hp.state().players.get(owner).unwrap().hp, hp - 3);
    assert_eq!(lose_hp.state().hand_count(owner), hand + 1);
}

#[test]
fn tiansuan2_2_turns_damage_to_fire_then_caps_after_tengjia() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_2 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_2 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_2);
    equip_tengjia(&mut tx, owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert!(events.contains(&DomainEvent::DamageDealt {
        source,
        target: owner,
        amount: 1,
        is_lose_hp: false,
        nature: rule_api::damage_nature::FIRE,
    }));
}

#[test]
fn tiansuan2_3_adds_one_to_damage_but_not_lose_hp() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_3 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_3 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_3);
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 3);
}

#[test]
fn jiu_can_self_save_while_tiansuan2_4_blocks_tao_jiu_and_adds_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_4 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_4 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    let hand = tx.state().hand_cards(owner);
    assert!(hand.len() >= 2);
    for card in &hand {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    let jiu = hand[0];
    let tao = hand[1];
    tx.state_mut().cards.get_mut(jiu).unwrap().kind = CardKind::Jiu;
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), owner),
        Some(jiu)
    );

    let mut self_save = Transaction::begin(tx.state().clone());
    self_save.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let result = run_effects(
        &mut self_save,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let choice_id = ChoiceId::new(self_save.state().resolution.pending_choice_id);
    let (saved, result) = apply_command(
        self_save.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(saved.players.get(owner).unwrap().hp, 1);
    assert!(!saved.hand_cards(owner).contains(&jiu));

    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;
    grant_skill(&mut tx, owner, TIANSUAN2_4);
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        tao
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        jiu
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind(
        tx.state(),
        owner,
        CardKind::Tao
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind(
        tx.state(),
        owner,
        CardKind::Jiu
    ));
    assert_eq!(engine_core::query::dying_save_card(tx.state(), owner), None);

    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
}

#[test]
fn huaizi_uses_max_hp_as_the_hand_limit_base() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    grant_skill(&mut tx, owner, HUAIZI);

    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(max_hp)
    );
}

#[test]
fn committed_package_only_reacts_to_another_players_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHAOHUO));

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
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut other_dying = Transaction::begin(engine.state().clone());
    {
        // Drop every booted skill: a booted 华佗 would answer the dying
        // window with 急救 and open an optional-trigger choice.
        let booted: Vec<_> = other_dying
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        if !booted.is_empty() {
            run_effects(&mut other_dying, booted).unwrap();
        }
        let player = other_dying.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    // Clear every hand so the dying-save window cannot open, independent of
    // which 桃 the booted roster was dealt.
    for p in [owner, other] {
        let hand = other_dying.state().zones.find(ZoneKind::Hand, p).unwrap();
        let cards: Vec<_> = other_dying.state().hand_cards(p);
        if !cards.is_empty() {
            let discard = other_dying
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut other_dying,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }
    grant(&mut other_dying, owner);
    let hand = other_dying.state().hand_count(owner);
    run_effects(
        &mut other_dying,
        vec![EffectRequest::EnterDying { player: other }],
    )
    .unwrap();
    assert_eq!(other_dying.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(other_dying.state().hand_count(owner), hand + 3);

    let mut self_dying = Transaction::begin(engine.state().clone());
    {
        let booted: Vec<_> = self_dying
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        if !booted.is_empty() {
            run_effects(&mut self_dying, booted).unwrap();
        }
        let player = self_dying.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    for p in [owner, other] {
        let hand = self_dying.state().zones.find(ZoneKind::Hand, p).unwrap();
        let cards: Vec<_> = self_dying.state().hand_cards(p);
        if !cards.is_empty() {
            let discard = self_dying
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut self_dying,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }
    grant(&mut self_dying, owner);
    run_effects(
        &mut self_dying,
        vec![EffectRequest::EnterDying { player: owner }],
    )
    .unwrap();
    assert_eq!(self_dying.state().players.get(owner).unwrap().max_hp, 4);
}

#[test]
fn mbranshang_counts_only_fire_damage_then_resolves_the_end_phase_penalty() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(MBRANSHANG as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: MBRANSHANG as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, MBRANSHANG);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::MarkCount {
                player: owner,
                mark: MBRANSHANG_MARK,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(2)
    ));
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::MarkCount {
                player: owner,
                mark: MBRANSHANG_MARK,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(2)
    ));

    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 8;
        player.max_hp = 8;
    }
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (6, 6));
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}
}

mod original_mobile_zhengjing2_gain {
//! 第二百零五批：gain_whole 消费池散批——zhengjing2（正争·终段）行为测试。
//! 实现：gain_whole_skill_pile(self, self_skill, 自身堆) + skip_next_judge_phase
//! (self) + remove_skill_by_def(self)。「争」堆供给依赖 zhengjing 置入面落地
//! （dormant 契约）；跳过摸牌阶段为登记债（无 draw-skip 原语）。

use engine_api::{
    CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHENGJING2: u32 = 4_211_882_480;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/zhengjing2.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("zhengjing2-test", SOURCE).unwrap();
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
        skill_def: SkillDefId::new(ZHENGJING2),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

fn fill_pile(state: GameState, owner: PlayerId, n: usize) -> GameState {
    let mut state = state;
    for i in 0..n {
        let (s2, card) = inject_hand_card(state, owner, 9_600 + i as u32);
        state = s2;
        let mut tx = Transaction::begin(state);
        let existing = tx
            .state()
            .skill_piles
            .iter()
            .find(|p| p.owner == owner && p.skill_def == ZHENGJING2)
            .map(|p| p.zone);
        let zone = match existing {
            Some(zone) => zone,
            None => {
                let zone = tx.state_mut().zones.alloc(Zone {
                    id: engine_api::ZoneId::NONE,
                    kind: ZoneKind::Expansion,
                    owner,
                    cards: Vec::new(),
                });
                tx.state_mut()
                    .skill_piles
                    .push(engine_core::state::SkillPileRef {
                        owner,
                        skill_def: ZHENGJING2,
                        zone,
                        usable_as_hand: false,
                        wants_lost_count: false,
                    });
                zone
            }
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = zone;
        }
        state = tx.commit().state;
    }
    state
}

fn fire_prepare(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}

#[test]
fn zhengjing2_gains_pile_and_removes_itself() {
    let (state, owner, _other) = setup(20_260_917_01);
    let state = fill_pile(state, owner, 3);
    let hand_before = state.hand_count(owner);
    let max_before = state.players.get(owner).unwrap().max_hp;

    let (state, outcome) = fire_prepare(state, owner);
    assert!(outcome.is_ok(), "forced effects, got {outcome:?}");
    assert_eq!(
        state.hand_count(owner),
        hand_before + 3,
        "pile cards moved into hand"
    );
    assert_eq!(
        state.skill_pile_cards(owner, ZHENGJING2).len(),
        0,
        "pile drained"
    );
    assert!(
        !state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == owner && skill.def.raw() == ZHENGJING2),
        "skill removed itself"
    );
    let _ = max_before;
}
}
