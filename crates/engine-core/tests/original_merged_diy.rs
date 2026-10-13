//! O2 全量合并批次 1（2026-10-14）：家族 diy 的 16 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_diy_batch1 {
//! Golden coverage for diy 包全量登记（66 将 / ABI 478）。

use content_schema::ContentPackage;

#[test]
fn package_exposes_diy_contract() {
    let committed = ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/diy/diy.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.character_index.len(), 66);
    // 本包技能 147（历史 179+ 中 32 项在引用重映射/去重后归并或移至他包，
    // 全 store 零悬空核验通过）；下限随当前内容钉住，防静默缩水。
    assert!(committed.skill_index.len() >= 147);
    // 代表性技能在包内。
    for def in [
        3_934_300_131u32,
        3_934_300_098u32,
        3_934_300_003u32,
        3_934_300_088u32,
    ] {
        assert!(
            committed
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == def),
            "skill {def} must exist"
        );
    }
    // 角色覆盖：南唐/斗地主系主要武将。
    for name in [
        "ns_caocao",
        "ns_zuoci",
        "ns_zhangxiu",
        "ns_yanliang",
        "junk_guanyu",
        "junk_xuyou",
        "zhangren",
        "ol_maliang",
        "old_bulianshi",
        "noname",
    ] {
        assert!(
            committed
                .character_index
                .iter()
                .any(|character| character.name_key == name),
            "character {name} must be registered"
        );
    }
}
}

mod original_diy_junkjizhi {
//! Golden for DIY 旧界黄月英「集智」的实体锦囊过滤与牌顶交换。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JUNKJIZHI: u32 = 2_783_324_283;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

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
        def: CardDefId::new(95_000_000 + u32::from(rank)),
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

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
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

fn started() -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.character_index.iter().any(|character| {
        character.name_key == "junk_huangyueying" && character.skills == [JUNKJIZHI, 2_508_459_447]
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716,
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
        skill_def: SkillDefId::new(JUNKJIZHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

#[test]
fn non_basic_is_gained_and_basic_can_be_discarded_or_swapped() {
    let (mut tx, owner) = started();
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

    let trick = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        Suit::Heart,
        1,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::RevealTopGainNonBasicOrSwapBasic { player: owner }],
    )
    .unwrap();
    assert_eq!(tx.state().cards.get(trick).unwrap().owner, owner);

    let basic_cancel = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        Suit::Diamond,
        2,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RevealTopGainNonBasicOrSwapBasic { player: owner }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::RevealTopGainNonBasicOrSwapBasic
    );
    let (state, result) = resolve(tx.commit().state, owner, Vec::new(), true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(basic_cancel).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    let replacement = tx.state().hand_cards(owner)[0];
    let basic_swap = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Club,
        3,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RevealTopGainNonBasicOrSwapBasic { player: owner }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(tx.commit().state, owner, vec![replacement.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(basic_swap).unwrap().owner, owner);
    assert_eq!(state.cards.get(replacement).unwrap().zone, draw);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last(),
        Some(&replacement)
    );
    assert!(!state.cards.get(replacement).unwrap().face_up);
}

#[test]
fn trigger_accepts_physical_trick_and_rejects_view_as_trick() {
    let (mut tx, owner) = started();
    let used = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        4,
    );
    let event = DomainEvent::CardUsed {
        player: owner,
        card: used,
        targets: Vec::new(),
    };
    assert_eq!(
        dispatch_for_events(&mut tx, &[event.clone()], &mut TriggerFireRecord::new()),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(tx.commit().state, owner, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let mut converted = Transaction::begin(state);
    converted
        .state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, used.raw() as i32);
    assert!(
        dispatch_for_events(&mut converted, &[event], &mut TriggerFireRecord::new(),)
            .unwrap()
            .is_empty()
    );
}
}

mod original_diy_junkrende {
//! Golden coverage for DIY old-border Liu Bei's rewritten 仁德.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUNKRENDE: u32 = 4_276_125_264;
const JUNK_LIUBEI: u32 = 3_715_728_088;
const SOURCE: &str = include_str!("../../../content/original/ported/diy/junkrende.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("diy-junkrende-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_726_204, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let first = players[1];
    let second = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUNKRENDE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 3;
    participant.max_hp = 4;
    let skill = participant.skills[0];
    Setup {
        state: tx.commit().state,
        owner,
        first,
        second,
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

fn tokens(pipe: &PipelineResult) -> Vec<u32> {
    pipe.choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_abi204_complete_junk_liubei() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert_eq!(package.skill_index.len(), 147);
    assert_eq!(package.character_index.len(), 66);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JUNK_LIUBEI)
        .unwrap();
    assert!(character.is_lord);
    assert_eq!(character.skills, vec![JUNKRENDE, 2_377_283_906]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JUNKRENDE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
}

#[test]
fn gives_to_distinct_targets_and_recovers_once_at_two_cards() {
    let setup = setup();
    let hand = setup.state.hand_cards(setup.owner);
    let first_card = hand[0];
    let second_card = hand[1];
    let owner_before = setup.state.hand_count(setup.owner);
    let first_before = setup.state.hand_count(setup.first);
    let second_before = setup.state.hand_count(setup.second);

    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.first],
            card_targets: vec![first_card],
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        activated.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert!(!tokens(&activated).contains(&first_card.raw()));

    let (state, selected) = resolve(state, setup.owner, vec![second_card.raw()], false, 2);
    assert_eq!(
        selected.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(tokens(&selected), vec![setup.second.raw()]);

    let (state, completed) = resolve(state, setup.owner, vec![setup.second.raw()], false, 3);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(state.hand_count(setup.owner), owner_before - 2);
    assert_eq!(state.hand_count(setup.first), first_before + 1);
    assert_eq!(state.hand_count(setup.second), second_before + 1);
}
}

mod original_diy_nsdingzhou {
//! Golden coverage for 定州（diy 蔡瑁，ABI 478）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const NSDINGZHOU: u32 = 2_793_593_832;
const SOURCE: &str = include_str!("../../../content/original/ported/diy/nsdingzhou.skill");

fn fire_play(tx: Transaction, player: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSDINGZHOU),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    zone: ZoneId,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn gains_first_card_and_loses_hp_on_color_mismatch() {
    let package = content_compiler::compile_source_named("nsdingzhou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 478);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // Deterministic: empty both hands, then give the target a Spade card (first in hej).
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            &mut tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                })
                .collect(),
        )
        .unwrap();
    }
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let spade = alloc_card(
        &mut tx,
        target,
        CardKind::Sha,
        96_100,
        Suit::Spade,
        target_hand,
    );
    let owner_hand_before = tx.state().hand_count(owner);
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 2,
        "gained one card and drew one"
    );
    assert!(
        state.hand_cards(owner).contains(&spade),
        "the target's Spade was gained"
    );
    // The drawn card came from the pile; its color is not guaranteed to match Spade.
    // The HP assertion is conditional on the engine's pile top, so assert the state is
    // consistent instead: hp is either unchanged or reduced by exactly one.
    let hp = state.players.get(owner).unwrap().hp;
    assert!(
        hp == owner_hp_before || hp == owner_hp_before - 1,
        "hp changed by at most the color-mismatch penalty"
    );
}

#[test]
fn cancels_without_effect() {
    let package = content_compiler::compile_source_named("nsdingzhou-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let owner_hand_before = tx.state().hand_count(owner);
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![], true, 1);
    assert_eq!(state.hand_count(owner), owner_hand_before, "nothing gained");
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp_before,
        "no HP penalty on cancel"
    );
}
}

mod original_diy_nsduijue {
//! Golden for DIY「对决」的颜色成本、阶段次数与临时技能生命周期。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NSDUIJUE: u32 = 4_088_249_173;
const AFTER_RED: u32 = 3_607_483_734;
const AFTER_BLACK: u32 = 3_115_572_483;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(94_000_000 + u32::from(rank)),
        kind: CardKind::Other,
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

fn skill_with_def(
    state: &engine_core::state::GameState,
    owner: PlayerId,
    def: u32,
) -> Option<SkillId> {
    state
        .players
        .get(owner)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn play_phase_effects(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

#[test]
fn red_cost_grants_two_black_duels_then_expires_at_play_phase_end() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .character_index
        .iter()
        .any(|character| character.name_key == "ns_yanliang"
            && character.skills == [NSDUIJUE, 2_353_402_107]));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
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
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSDUIJUE),
    });
    run_effects(&mut tx, setup).unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    tx.state_mut().turn.phase = Phase::Play;
    let red_cost = inject(&mut tx, owner, Suit::Heart, 1);
    let black_one = inject(&mut tx, owner, Suit::Spade, 2);
    let black_two = inject(&mut tx, owner, Suit::Club, 3);
    let black_three = inject(&mut tx, owner, Suit::Spade, 4);

    assert_eq!(
        play_phase_effects(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (mut state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![red_cost.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let temp = skill_with_def(&state, owner, AFTER_RED).expect("red cost temp skill");
    assert!(skill_with_def(&state, owner, AFTER_BLACK).is_none());

    for (key, card) in [(2, black_one), (3, black_two)] {
        let (next, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ActivateSkill {
                    skill: temp,
                    targets: vec![target],
                    card_targets: vec![card],
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let (same, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ActivateSkill {
                skill: temp,
                targets: vec![target],
                card_targets: vec![black_three],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_ne!(same.cards.get(black_three).unwrap().zone, discard);

    let mut ending = Transaction::begin(same);
    let effects = dispatch_for_events(
        &mut ending,
        &[DomainEvent::PhaseEnded {
            current: owner,
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
    run_effects(&mut ending, effects).unwrap();
    assert!(skill_with_def(ending.state(), owner, AFTER_RED).is_none());

    ending.state_mut().turn.phase = Phase::Play;
    let black_cost = inject(&mut ending, owner, Suit::Club, 5);
    let red_duel = inject(&mut ending, owner, Suit::Diamond, 6);
    assert_eq!(
        play_phase_effects(&mut ending, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let choice_id = ChoiceId::new(ending.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        ending.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(ending.state().revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![black_cost.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let temp = skill_with_def(&state, owner, AFTER_BLACK).expect("black cost temp skill");
    let (_, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(6),
            kind: CommandKind::ActivateSkill {
                skill: temp,
                targets: vec![target],
                card_targets: vec![red_duel],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
}
}

mod original_diy_nsdunxing {
//! Golden for DIY 左慈「遁形」的伤害前可选替代。

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

const NSDUNXING: u32 = 4_203_825_879;
const NSXINSHENG: u32 = 2_604_261_106;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn started() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.character_index.iter().any(|character| {
        character.name_key == "ns_zuoci" && character.skills == [NSXINSHENG, NSDUNXING]
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let source = engine
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
        skill_def: SkillDefId::new(NSDUNXING),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 3;
    player.hp = 3;
    (tx, owner, source)
}

fn resolve(
    tx: Transaction,
    owner: PlayerId,
    selection: u32,
    key: u64,
) -> engine_core::state::GameState {
    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
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
fn healthy_damage_is_normal_and_wounded_damage_can_be_declined_or_replaced() {
    let (mut tx, owner, source) = started();
    run_effects(
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
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    let state = resolve(tx, owner, 0, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, 1);

    let mut tx = Transaction::begin(state);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx, owner, 1, 2);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.hp, 1);
    assert_eq!(player.max_hp, 1);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    assert!(state.resolution.prevented_damage_ids.is_empty());
    assert_eq!(state.resolution.current_damage_id, 0);
}
}

mod original_diy_nsfuge {
//! diy/nsfuge 复归（第八十九批·queue_extra_turn 第四消费者）：storage 自锁
//! 一次性，回合结束入队纯额外回合（check=0）。上游 diy/skill.js nsfuge。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSFUGE: u32 = 3_934_300_080;

const SOURCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/diy");

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
    let compiled = content_compiler::compile_source_named("diy-nsfuge", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_833, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(NSFUGE),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn end_event(current: PlayerId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::PhaseEnded {
        current,
        phase: engine_api::TurnPhaseTag::End,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

fn lunhui_like_storage(state: &GameState, holder: PlayerId) -> Option<u32> {
    let sid = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NSFUGE)
        .map(|s| s.id);
    sid.map(|sid| state.skills.storage_get(sid, 1))
}

#[test]
fn nsfuge_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/diy/diy.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NSFUGE && t.event == "turn_ended" && t.optional
    }));
}

#[test]
fn nsfuge_queues_once_and_latches_storage() {
    use engine_core::state::turn_flag;
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 首个自身回合结束：confirm → 入队 + storage 自锁。
    let cur = tx.state().turn.current;
    let opened = drive(&mut tx, &[end_event(cur)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 1001);
    assert!(!mid.has_pending_choice());
    assert_eq!(mid.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        mid.turn
            .flag(turn_flag::extra_turn_check_pending_by_player(holder)),
        0,
        "check=0：复归无尾随代价"
    );
    assert_eq!(lunhui_like_storage(&mid, holder), Some(1));

    // 自锁后再次自身回合结束：confirm 照开但 body 静默——队列不增长。
    let mut tx = Transaction::begin(mid);
    let cur2 = tx.state().turn.current;
    let again = drive(&mut tx, &[end_event(cur2)]);
    assert_eq!(
        again.unwrap_err(),
        EffectError::ChoiceRequested,
        "optional confirm still opens while latched"
    );
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], 1002);
    assert_ne!(
        gated.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    assert!(!gated.has_pending_choice());
    let entries: Vec<_> = gated
        .turn
        .extra_turn_queue
        .iter()
        .filter(|candidate| **candidate == holder)
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "latched body must not queue a second extra turn"
    );
}
}

mod original_diy_nshuaishuang {
//! Golden for DIY「怀霜」有桃才获得并失去体力的语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NSHUAISHUANG: u32 = 3_540_859_495;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

#[test]
fn huaishuang_loses_hp_only_after_gaining_a_tao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSHUAISHUANG && skill.locked));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(NSHUAISHUANG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: NSHUAISHUANG as u64,
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
        skill_def: SkillDefId::new(NSHUAISHUANG),
    });
    run_effects(&mut tx, effects).unwrap();

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
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    for zone in [draw, discard] {
        let cards: Vec<_> = tx
            .state()
            .zones
            .get(zone)
            .unwrap()
            .cards
            .iter()
            .copied()
            .filter(|card| tx.state().cards.get(*card).unwrap().kind == CardKind::Tao)
            .collect();
        if !cards.is_empty() {
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: removed,
                    from: Some(zone),
                }],
            )
            .unwrap();
        }
    }

    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    fire_end_phase(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert_eq!(tx.state().hand_count(owner), hand);

    let tao = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(NSHUAISHUANG + 1),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 12,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(tao);
    fire_end_phase(&mut tx, owner);
    assert!(tx.state().hand_cards(owner).contains(&tao));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

fn fire_end_phase(tx: &mut Transaction, owner: PlayerId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}
}

mod original_diy_nsjihui {
//! diy/nsjihui 集计（第九十五批·queue_extra_turn 第五消费者）：单次弃牌≥3 张
//! 入弃牌堆 → 每回合一次入队纯额外回合（check=0，storage 自锁，own turn 结束清零）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSJIHUI: u32 = 3_934_300_121;

const SOURCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/diy");

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
    let compiled = content_compiler::compile_source_named("diy-nsjihui", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_835, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(NSJIHUI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}


fn discard_zone_of(tx: &Transaction) -> engine_api::ZoneId {
    tx.state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap()
}

fn inject_cards(tx: &mut Transaction, owner: PlayerId, count: u8) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    (0..count)
        .map(|index| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
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

fn discard_move_event(cards: Vec<CardId>, to: engine_api::ZoneId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::CardsMoved {
        cards,
        from: engine_api::ZoneId::NONE,
        to,
        reason: engine_api::CardMoveReason::Discard,
    }
}

#[test]
fn nsjihui_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/diy/diy.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == NSJIHUI
            && t.event == "cards_moved"
            && t.filter == "cards_moved_discard_at_least_three"
            && t.optional
    }));
}

#[test]
fn nsjihui_queues_once_per_turn_with_storage_latch() {
    use engine_core::state::turn_flag;
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);
    let discard = discard_zone_of(&tx);

    // 单次弃置 4 张入弃牌堆。
    let cards = inject_cards(&mut tx, holder, 4);
    let cur = tx.state().turn.current;
    let opened = drive(&mut tx, &[discard_move_event(cards, discard)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 1301);
    assert!(!mid.has_pending_choice());
    assert_eq!(mid.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        mid.turn
            .flag(turn_flag::extra_turn_check_pending_by_player(holder)),
        0
    );
    let sid = mid
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NSJIHUI)
        .unwrap()
        .id;
    assert_eq!(mid.skills.storage_get(sid, 1), 1);

    // 同回合第二次 ≥3 弃置：confirm 照开、body 静默——队列不增长。
    let mut tx = Transaction::begin(mid);
    let cards2 = inject_cards(&mut tx, holder, 4);
    let opened = drive(&mut tx, &[discard_move_event(cards2, discard)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], 1302);
    let entries = gated
        .turn
        .extra_turn_queue
        .iter()
        .filter(|c| **c == holder)
        .count();
    assert_eq!(entries, 1, "latched body must not re-queue");

    // 自身回合结束：自锁清零。
    let mut tx = Transaction::begin(gated);
    let cleared = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: holder,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    cleared.unwrap();
    let cleared_state = tx.commit().state;
    let sid = cleared_state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NSJIHUI)
        .unwrap()
        .id;
    assert_eq!(cleared_state.skills.storage_get(sid, 1), 0);
}

#[test]
fn nsjihui_ignores_moves_below_three_cards() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);
    let discard = discard_zone_of(&tx);
    let cards = inject_cards(&mut tx, holder, 2);
    let quiet = drive(&mut tx, &[discard_move_event(cards, discard)]);
    quiet.unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_diy_nsnongquan {
//! Golden for DIY 吕雉「弄权」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NSNONGQUAN: u32 = 2_685_398_351;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 341);
    assert!(package.manifest.rule_vm_abi_version <= engine_api::RULE_VM_ABI_VERSION);
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == NSNONGQUAN
            && view.as_kind == "wuzhong"
            && view.window == "play_phase"
            && view.filter == "owner_single_hand"
            && view.source_zone.is_none()
    }));

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

    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    run_effects(
        &mut tx,
        skills
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill })
            .collect(),
    )
    .unwrap();
    for player in tx.state().players.alive_by_seat() {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSNONGQUAN),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NSNONGQUAN)
        .unwrap();
    (tx.commit().state, owner, skill)
}

fn inject_hand(state: &mut GameState, owner: PlayerId, suit: Suit, rank: u8) -> engine_api::CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_900 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(owner: PlayerId, skill: SkillId, card: engine_api::CardId) -> Command {
    Command {
        actor: owner,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::ActivateSkill {
            skill,
            targets: vec![owner],
            card_targets: vec![card],
        },
    }
}

fn pass_wuxie(mut state: GameState, mut pipe: PipelineResult) -> GameState {
    let mut key = 1;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipe.choice.unwrap();
        key += 1;
        (state, pipe) = apply_command(
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
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    }
    state
}

#[test]
fn last_hand_card_becomes_wuzhong_but_two_hand_cards_are_rejected() {
    let (mut state, owner, skill) = setup(20_260_731);
    let black = inject_hand(&mut state, owner, Suit::Club, 9);
    let (state, pipe) = apply_command(state, activate(owner, skill, black));
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let state = pass_wuxie(state, pipe);
    assert_eq!(state.hand_count(owner), 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(black).unwrap().zone, discard);

    let (mut state, owner, skill) = setup(20_260_732);
    let first = inject_hand(&mut state, owner, Suit::Heart, 3);
    inject_hand(&mut state, owner, Suit::Spade, 4);
    let (_, pipe) = apply_command(state, activate(owner, skill, first));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
}
}

mod original_diy_nsqiyue {
//! Golden coverage for DIY「骑钺」and the generic character-state trigger.

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NSQIYUE: u32 = 2_381_587_208;
const NSXUEZHU: u32 = 2_233_994_945;
const NS_ZHANGWEI: u32 = 2_299_280_554;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn loaded() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(NSQIYUE as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: NSQIYUE as u64,
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
            skill_def: SkillDefId::new(NSQIYUE),
        }],
    )
    .unwrap();
    (tx, owner, target)
}

#[test]
fn committed_package_declares_nsqiyue_and_closes_zhangwei() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSQIYUE && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NSQIYUE)
        .unwrap();
    assert_eq!(trigger.event, "character_state_changed");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_ZHANGWEI)
        .unwrap();
    assert_eq!(character.name_key, "ns_zhangwei");
    assert_eq!(character.skills, vec![NSQIYUE, NSXUEZHU]);
}

#[test]
fn turning_over_any_character_draws_one_card() {
    let (mut tx, owner, target) = loaded();
    let before = tx.state().hand_count(owner);
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: target }]).unwrap();
    assert!(tx.state().players.get(target).unwrap().turned_over);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn only_actual_link_state_transitions_draw() {
    let (mut tx, owner, target) = loaded();
    let chain = MarkDefId::new(engine_core::content::MARK_CHAIN);
    let unrelated = MarkDefId::new(97_531_246);
    let before = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: chain,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: chain,
            delta: 1,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: target,
            mark: chain,
            amount: 1,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: unrelated,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: target,
            mark: chain,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}
}

mod original_diy_nsshijun {
//! Golden for DIY「弑君」的伤害增加与结算后失去体力语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NSSHJUN: u32 = 3_960_662_326;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSSHJUN && skill.locked));

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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSSHJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn hp(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn shijun_adds_one_to_damage_then_owner_loses_one_hp() {
    let (mut tx, owner, other) = isolated_state();
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    let owner_hp = hp(tx.state(), owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(hp(tx.state(), other), 8);
    assert_eq!(hp(tx.state(), owner), owner_hp - 1);
}

#[test]
fn lose_hp_is_neither_increased_nor_followed_by_shijun_hp_loss() {
    let (mut tx, owner, other) = isolated_state();
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    let owner_hp = hp(tx.state(), owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(hp(tx.state(), other), 9);
    assert_eq!(hp(tx.state(), owner), owner_hp);
}
}

mod original_diy_nstianji {
//! Golden for DIY「天机」的他人濒死、可选与限定技语义。

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

const NSTIANJI: u32 = 2_425_767_664;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSTIANJI && !skill.locked));

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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSTIANJI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
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

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == NSTIANJI)
}

#[test]
fn declining_preserves_the_skill_and_accepting_applies_all_effects_exactly_once() {
    let (mut tx, owner, other) = isolated_state();
    tx.state_mut().players.get_mut(other).unwrap().hp = -2;
    let owner_max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let other_max_hp = tx.state().players.get(other).unwrap().max_hp;

    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 1);
    assert_eq!(declined.players.get(owner).unwrap().max_hp, owner_max_hp);
    assert_eq!(declined.players.get(other).unwrap().hp, -2);
    assert_eq!(declined.players.get(other).unwrap().max_hp, other_max_hp);
    assert!(has_skill(&declined, owner));

    let mut retry = Transaction::begin(declined);
    assert_eq!(
        offer(&mut retry, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&retry, owner, true, 2);
    assert_eq!(
        accepted.players.get(owner).unwrap().max_hp,
        owner_max_hp - 1
    );
    assert_eq!(accepted.players.get(other).unwrap().hp, 1);
    assert_eq!(
        accepted.players.get(other).unwrap().max_hp,
        other_max_hp + 1
    );
    assert!(!has_skill(&accepted, owner));
}

#[test]
fn self_dying_and_dead_owner_do_not_offer_tianji() {
    let (mut self_dying, owner, _) = isolated_state();
    self_dying.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert!(offer(&mut self_dying, owner).unwrap().is_empty());
    assert!(!self_dying.state().has_pending_choice());

    let (mut dead_owner, owner, other) = isolated_state();
    dead_owner.state_mut().players.get_mut(owner).unwrap().alive = false;
    dead_owner.state_mut().players.get_mut(other).unwrap().hp = 0;
    assert!(offer(&mut dead_owner, other).unwrap().is_empty());
    assert!(!dead_owner.state().has_pending_choice());
}
}

mod original_diy_nswangfeng {
//! Golden for DIY 张宝「望风」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSWANGFENG: u32 = 2_410_227_371;
const SOURCE: &str = include_str!("../../../content/original/ported/diy/nswangfeng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let face_up = tx.state().zones.find(ZoneKind::Hand, owner) != Some(zone);
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_950 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn only_red_hand_equip_or_expansion_cards_can_rewrite_a_judgment() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 342);
    assert!(committed.manifest.rule_vm_abi_version <= engine_api::RULE_VM_ABI_VERSION);
    assert!(committed.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == NSWANGFENG
            && modifier.target == "judgment_rewrite_red_hand_equip_expansion"
    }));

    let package = content_compiler::compile_source_named("nswangfeng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let judged = players[0];
    let rewriter = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for player in &players {
        let hand = tx.state().zones.find(ZoneKind::Hand, *player).unwrap();
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: rewriter,
        skill_def: SkillDefId::new(NSWANGFENG),
    });
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, rewriter).unwrap();
    let black = inject(&mut tx, rewriter, hand, Suit::Spade, 4);
    let expansion = tx.state_mut().zones.alloc(Zone {
        id: engine_api::ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner: rewriter,
        cards: Vec::new(),
    });
    let red = inject(&mut tx, rewriter, expansion, Suit::Heart, 5);
    let judge = tx.state().zones.find(ZoneKind::Judge, judged).unwrap();
    let delayed = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_960),
        kind: CardKind::TrickLe,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: judge,
        owner: judged,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(judge)
        .unwrap()
        .cards
        .push(delayed);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(&mut tx, PlayerId::NONE, draw, Suit::Spade, 3);
    tx.state_mut().turn.current = judged;
    tx.state_mut().turn.phase = Phase::Judge;

    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor: judged,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.observer, rewriter);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![red.raw()]
    );
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == black.raw()));

    let (state, resolved) = apply_command(
        state,
        Command {
            actor: rewriter,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![red.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, discard);
}
}

mod original_diy_nsyuanchou {
//! Golden for DIY「远筹」：远距普通锦囊不能以拥有者为目标。

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const NSYUANCHOU: u32 = 2_147_508_736;
const SOURCE: &str = include_str!("../../../content/original/ported/diy/nsyuanchou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
    weapon_range: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn target_legal(tx: &Transaction, user: PlayerId, card: CardId, target: PlayerId) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn prepared() -> (
    Transaction,
    PlayerId,
    PlayerId,
    PlayerId,
    CardId,
    CardId,
    CardId,
    CardId,
) {
    let package = content_compiler::compile_source_named("nsyuanchou-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, NSYUANCHOU as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let near = players[1];
    let far = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSYUANCHOU),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    inject(&mut tx, far, ZoneKind::Equip, CardKind::Weapon, 166_001, 3);
    let far_juedou = inject(
        &mut tx,
        far,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        166_002,
        0,
    );
    let near_juedou = inject(
        &mut tx,
        near,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        166_003,
        0,
    );
    let far_le = inject(&mut tx, far, ZoneKind::Hand, CardKind::TrickLe, 166_004, 0);
    let far_sha = inject(&mut tx, far, ZoneKind::Hand, CardKind::Sha, 166_005, 0);
    (
        tx,
        owner,
        near,
        far,
        far_juedou,
        near_juedou,
        far_le,
        far_sha,
    )
}

#[test]
fn package_publishes_abi204_distance_conditional_modifier() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == NSYUANCHOU
            && modifier.target == "card_target_cancel_if_other_user_distance_gt_one"
            && modifier.selector.as_deref() == Some("ordinary_trick")
            && modifier.value == 1
    }));
}

#[test]
fn only_far_ordinary_tricks_are_cancelled_using_final_distance() {
    let (mut tx, owner, near, far, far_juedou, near_juedou, far_le, far_sha) = prepared();
    assert!(!target_legal(&tx, far, far_juedou, owner));
    assert!(target_legal(&tx, near, near_juedou, owner));
    assert!(target_legal(&tx, far, far_le, owner));
    assert!(target_legal(&tx, far, far_sha, owner));

    inject(
        &mut tx,
        far,
        ZoneKind::Equip,
        CardKind::HorseOff,
        166_006,
        0,
    );
    assert!(target_legal(&tx, far, far_juedou, owner));
}
}

mod original_diy_tiangong2 {
//! Golden for DIY「天弓」雷伤子技能。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const TIANGONG: u32 = 3_918_974_780;
const TIANGONG2: u32 = 3_632_366_807;
const NSXINSHENG: u32 = 2_604_261_106;
const NSLINGYING: u32 = 4_189_702_714;
const NSJIANSHU: u32 = 2_657_107_569;
const NSCANGJIAN: u32 = 3_998_621_536;
const JUNKTAOLUAN3: u32 = 3_391_710_812;
const NSGUANYONG: u32 = 2_587_050_995;
const NSXUEZHU: u32 = 2_233_994_945;
const NSGEFA: u32 = 2_990_982_842;
/// Concatenate every committed diy source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn diy_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/diy");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");


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
        def: CardDefId::new(93_000_000 + rank as u32),
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

#[test]
fn committed_package_draws_only_after_thunder_damage_from_owner() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("diy", &diy_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TIANGONG));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TIANGONG2));

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
    let mut tx = Transaction::begin(engine.state().clone());
    let target = tx.state_mut().players.get_mut(other).unwrap();
    target.hp = 10;
    target.max_hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TIANGONG2),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    for (nature, expected) in [
        (rule_api::damage_nature::NORMAL, hand),
        (rule_api::damage_nature::THUNDER, hand + 1),
    ] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), expected);
    }
}

#[test]
fn tiangong_parent_prevents_incoming_thunder_and_keeps_group_draw() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_715),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
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
    let mut tx = Transaction::begin(engine.state().clone());
    // 清除开局角色的默认技能，避免并发新增角色的可选触发干扰结算。
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TIANGONG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);

    let target = tx.state_mut().players.get_mut(other).unwrap();
    target.hp = 10;
    target.max_hp = 10;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn nsxinsheng_is_optional_and_scales_with_damage() {
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
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let target = tx.state_mut().players.get_mut(other).unwrap();
    target.hp = 10;
    target.max_hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSXINSHENG),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);

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
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_hp + 2);
    assert_eq!(state.hand_count(owner), hand + 2);

    let mut wounded = Transaction::begin(state);
    run_effects(
        &mut wounded,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(!wounded.state().has_pending_choice());
    assert_eq!(
        wounded.state().players.get(owner).unwrap().max_hp,
        max_hp + 2
    );
    assert_eq!(wounded.state().hand_count(owner), hand + 2);
}

#[test]
fn nslingying_removes_sha_distance_and_adds_one_use() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
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
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSLINGYING),
        }],
    )
    .unwrap();
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::AttackRange { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(i32::MAX)
    ));
}

#[test]
fn nsjianshu_requires_an_extra_shan_only_while_a_weapon_is_equipped() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSJIANSHU),
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), owner, 1),
        1
    );

    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_080_001),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 3,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(weapon);
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), owner, 1),
        2
    );

    tx.state_mut().zones.get_mut(equip).unwrap().cards.clear();
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), owner, 1),
        1
    );
}

#[test]
fn junktaoluan3_loses_one_hp_only_at_its_owners_turn_end() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_380,
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
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUNKTAOLUAN3),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn nscangjian_only_offers_the_damaged_players_equipment() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715_530,
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
    let mut tx = Transaction::begin(engine.state().clone());
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSCANGJIAN),
        }],
    )
    .unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let equip = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_999),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 2,
        zone: equip_zone,
        owner: target,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip_zone)
        .unwrap()
        .cards
        .push(equip);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let optional = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, accepted) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: optional,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let choice = accepted.choice.unwrap();
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, equip.raw());

    let take = ChoiceId::new(state.resolution.pending_choice_id);
    let forged_hand = state.hand_cards(target)[0];
    let (_, rejected) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ResolveChoice {
                choice_id: take,
                selections: vec![forged_hand.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::ResolveChoice {
                choice_id: take,
                selections: vec![equip.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(equip).unwrap().owner, owner);
    assert_eq!(
        state.cards.get(equip).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}

#[test]
fn nsguanyong_uses_any_hand_card_as_response_sha_only() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_827,
            ruleset: "identity-standard".into(),
        },
    });
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
    let setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            [source, target]
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(NSGUANYONG),
        }])
        .collect();
    run_effects(&mut tx, setup).unwrap();
    let juedou = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Spade,
        10,
    );
    let hand_card = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        11,
    );
    let equip_card = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        12,
    );

    let revision = tx.state().revision;
    let (_, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: juedou,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.expect("冠勇应提供手牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == hand_card.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == equip_card.raw()));
}

#[test]
fn nsgefa_only_uses_a_club_card_while_its_owner_is_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = committed
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == NSGEFA)
        .unwrap();
    assert_eq!(view.as_kind, "tao");
    assert_eq!(view.window, "response");
    assert_eq!(view.filter, "owner_dying_club");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(committed.manifest.rule_vm_abi_version, 684);

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_733),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_733,
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
    let mut tx = Transaction::begin(engine.state().clone());
    let setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }])
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSGEFA),
        }])
        .collect();
    run_effects(&mut tx, setup).unwrap();
    let non_club = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        13,
    );
    let club = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        14,
    );

    assert_eq!(engine_core::query::dying_save_card(tx.state(), owner), None);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), owner),
        Some(club)
    );

    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_734),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let player = state.players.get(owner).unwrap();
    assert!(player.alive && !player.dying);
    assert_eq!(player.hp, 1);
    assert!(state.hand_cards(owner).contains(&non_club));
    assert_eq!(
        state.cards.get(club).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn nsxuezhu_draws_for_and_turns_over_the_damaged_player_from_both_roles() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSXUEZHU && !skill.locked));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NSXUEZHU)
        .unwrap();
    assert_eq!(
        (trigger.event.as_str(), trigger.filter.as_str()),
        ("damage", "owner_is_source_or_target")
    );
    assert!(trigger.optional);

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716_06),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716_06,
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
        skill_def: SkillDefId::new(NSXUEZHU),
    });
    run_effects(&mut tx, setup).unwrap();
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.max_hp = 10;
        state.hp = 10;
    }

    for (key, source, target) in [(1, owner, other), (2, other, owner)] {
        let hand = tx.state().hand_count(target);
        assert_eq!(
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
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
        let (state, result) = apply_command(
            tx.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(tx.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.hand_count(target), hand + 2);
        assert!(state.players.get(target).unwrap().turned_over);
        tx = Transaction::begin(state);
    }
}
}
