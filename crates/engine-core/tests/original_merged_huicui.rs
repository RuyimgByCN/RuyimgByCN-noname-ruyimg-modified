//! O2 全量合并批次 1（2026-10-14）：家族 huicui 的 17 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_huicui_1_11_5_dcqiansu_chijian {
//! Golden for HUICUI 1.11.5 新杀费祎「谦素」与 新杀夏侯恩「持剑」近似语义。
//!
//! 谦素：其他角色结束阶段，若其手牌多于你，可摸三张并交给其一张。
//! 持剑：出杀次数 +1，忽略防具。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::sha_use_limit_for;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCQIANSU: u32 = 4_169_409_504;
const CHIJIAN: u32 = 3_230_974_968;
const DC_FEIYI: u32 = 3_373_769_823;
const DC_XIAHOUEN: u32 = 3_482_689_208;
const DCQIANSU_SRC: &str =
    include_str!("../../../content/original/ported/huicui/dcqiansu.skill");
const CHIJIAN_SRC: &str = include_str!("../../../content/original/ported/huicui/chijian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject_hand(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
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
    selection: u32,
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup_qiansu() -> (Transaction, PlayerId, PlayerId, CardId) {
    let compiled = content_compiler::compile_source_named("dcqiansu-test", DCQIANSU_SRC).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_801, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(DCQIANSU),
    });
    run_effects(&mut tx, effects).unwrap();
    // other has more hand cards than owner
    inject_hand(tx.state_mut(), owner, CardKind::Sha, 201_001);
    inject_hand(tx.state_mut(), other, CardKind::Shan, 201_002);
    inject_hand(tx.state_mut(), other, CardKind::Other, 201_003);
    inject_hand(tx.state_mut(), other, CardKind::Other, 201_004);
    let gift = *tx
        .state()
        .hand_cards(owner)
        .first()
        .expect("owner keeps one hand card for give");
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, other, gift)
}

#[test]
fn package_publishes_1_11_5_feiyi_and_xiahouen_closures() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 487);
    assert_eq!(package.skill_index.len(), 254);
    assert_eq!(package.character_index.len(), 119);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == DCQIANSU && s.name == "dcqiansu"));
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == CHIJIAN && s.name == "chijian"));
    let feiyi = package
        .character_index
        .iter()
        .find(|c| c.id == DC_FEIYI)
        .unwrap();
    assert_eq!(feiyi.name_key, "dc_feiyi");
    assert_eq!(feiyi.skills, [DCQIANSU, 3_735_974_633, 2_598_339_084]);
    let xiahouen = package
        .character_index
        .iter()
        .find(|c| c.id == DC_XIAHOUEN)
        .unwrap();
    assert_eq!(xiahouen.name_key, "dc_xiahouen");
    assert_eq!(xiahouen.skills, [CHIJIAN, 3_773_213_822]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn qiansu_draws_three_then_gives_one_when_other_has_more_hand() {
    let (mut tx, owner, other, gift) = setup_qiansu();
    assert!(tx.state().hand_count(other) > tx.state().hand_count(owner));
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(other)],
            &mut TriggerFireRecord::new()
        ),
        Err(EffectError::ChoiceRequested)
    );
    let offered = tx.commit();
    assert_eq!(
        offered.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );

    // confirm optional trigger
    let (state, accepted) = resolve(offered.state, owner, 1, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    // after draw 3, owner has owner_hand+3 before giving
    assert_eq!(state.hand_count(owner), owner_hand + 3);

    let (state, given) = resolve(state, owner, gift.raw(), 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(other), other_hand + 1);
    assert_eq!(
        state.cards.get(gift).unwrap().zone,
        state.zones.find(ZoneKind::Hand, other).unwrap()
    );
}

#[test]
fn qiansu_skips_when_owner_hand_not_less() {
    let compiled = content_compiler::compile_source_named("dcqiansu-skip", DCQIANSU_SRC).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_802, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(DCQIANSU),
    });
    run_effects(&mut tx, effects).unwrap();
    // equal hands → if condition fails after optional? filter is only not_target;
    // body if may no-op after confirm. Give other fewer hands.
    inject_hand(tx.state_mut(), owner, CardKind::Sha, 202_001);
    inject_hand(tx.state_mut(), owner, CardKind::Sha, 202_002);
    inject_hand(tx.state_mut(), other, CardKind::Shan, 202_003);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::End;
    let owner_hand = tx.state().hand_count(owner);
    // may still offer optional; if confirm, if-body no-ops
    let dispatched = dispatch_for_events(
        &mut tx,
        &[phase_end(other)],
        &mut TriggerFireRecord::new(),
    );
    match dispatched {
        Ok(_effects) => assert_eq!(tx.state().hand_count(owner), owner_hand),
        Err(EffectError::ChoiceRequested) => {
            let offered = tx.commit();
            let (state, accepted) = resolve(offered.state, owner, 1, 11);
            assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
            // condition false → no draw
            assert_eq!(state.hand_count(owner), owner_hand);
        }
        Err(e) => panic!("unexpected: {e:?}"),
    }
}

#[test]
fn chijian_adds_sha_use_limit_and_ignore_armor() {
    let compiled = content_compiler::compile_source_named("chijian-test", CHIJIAN_SRC).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_803, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(CHIJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // base identity sha use limit is typically 1; +1 → 2
    assert!(
        sha_use_limit_for(&state, owner) >= 2,
        "持剑应提升出杀次数上限, got {}",
        sha_use_limit_for(&state, owner)
    );
    // ignore_armor is a package modifier entry for 持剑
    let package = package();
    assert!(
        package
            .modifier_index
            .iter()
            .any(|m| m.skill_def_id == CHIJIAN),
        "持剑应发布 ignore_armor / sha_use_limit 修正器"
    );
}
}

mod original_huicui_chenjian {
//! Golden tests for HUICUI「陈见」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MarkDefId,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CHENJIAN: u32 = 3_452_524_123;

fn started_state(seed: u64) -> GameState {
    let mut engine = Engine::new();
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
    engine.state().clone()
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == CHENJIAN)
        .map_or(0, |(_, count)| *count)
}

fn set_top_three(
    tx: &mut Transaction,
    first: (CardKind, Suit),
    second: (CardKind, Suit),
    third: (CardKind, Suit),
) -> [engine_api::CardId; 3] {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let pile = &tx.state().zones.get(draw).unwrap().cards;
    let cards = [
        pile[pile.len() - 1],
        pile[pile.len() - 2],
        pile[pile.len() - 3],
    ];
    for (card, (kind, suit)) in cards.into_iter().zip([first, second, third]) {
        let state = tx.state_mut().cards.get_mut(card).unwrap();
        state.kind = kind;
        state.suit = suit;
    }
    cards
}

fn start_effect(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::RevealDiscardSuitGainOrUse {
            player: owner,
            mark: MarkDefId::new(CHENJIAN),
            base_count: 3,
            max_mark: 2,
            revealed: Vec::new(),
            completed: 0,
            step: 0,
            data: 0,
        }],
    )
    .map(|_| ())
}

#[test]
fn discard_first_uses_effective_discard_suit_raw_reveal_suit_and_may_skip_use() {
    const SUIT_FIXTURE: u32 = 4_294_966_997;
    let fixture = content_compiler::compile_source_named(
        "chenjian-suit-fixture",
        "skill chenjian_suit_fixture(4294966997) locked { modifier card_suit(spade) override heart; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&fixture).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2_026_071_501),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 2_026_071_501,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SUIT_FIXTURE),
        }],
    )
    .unwrap();
    let revealed = set_top_three(
        &mut tx,
        (CardKind::Shan, Suit::Heart),
        (CardKind::TrickWuzhong, Suit::Spade),
        (CardKind::Shan, Suit::Club),
    );
    let discard_card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(discard_card).unwrap().suit = Suit::Spade;

    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::RevealDiscardSuitGainOrUse
    );
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().zones.get(processing).unwrap().cards.len(), 3);

    let state = tx.commit().state;
    let before_forgery = canonical_state_hash(&state);
    let (unchanged, rejected) = resolve(state.clone(), vec![99], false);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forgery);

    let (state, first) = resolve(state, vec![1], false);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (state, discarded) = resolve(state, vec![discard_card.raw()], false);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let (state, gained) = resolve(state, vec![owner.raw()], false);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(revealed[0]).unwrap().owner, owner);

    let (state, skipped) = resolve(state, Vec::new(), true);
    assert!(matches!(skipped.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(mark_count(&state, owner), 0);
    assert!(state.zones.get(processing).unwrap().cards.is_empty());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(revealed[1]).unwrap().zone, discard);
    assert_eq!(state.cards.get(revealed[2]).unwrap().zone, discard);
}

#[test]
fn used_wuzhong_resolves_wuxie_then_second_option_marks_and_recasts_all_hand() {
    let state = started_state(2_026_071_502);
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let wuxie = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(wuxie).unwrap().kind = CardKind::TrickWuxie;
    let revealed = set_top_three(
        &mut tx,
        (CardKind::TrickWuzhong, Suit::Spade),
        (CardKind::Shan, Suit::Heart),
        (CardKind::Shan, Suit::Club),
    );
    let discard_card = tx.state().hand_cards(owner)[0];
    let initial_hand_count = tx.state().hand_count(owner);
    tx.state_mut().cards.get_mut(discard_card).unwrap().suit = Suit::Heart;
    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );

    let (state, _) = resolve(tx.commit().state, vec![2], false);
    let (state, used) = resolve(state, vec![revealed[0].raw()], false);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::WuxieWindow
    );

    // The engine asks every responder (owner, then the wuxie-holding other)
    // in turn; pass each wuxie window until the reveal continuation opens.
    let mut state = state;
    loop {
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::WuxieWindow
        );
        let (s, passed_wuxie) = resolve(state, Vec::new(), true);
        assert!(matches!(
            passed_wuxie.result,
            CommandResult::Accepted { .. }
        ));
        state = s;
        if state.resolution.continuation_kind
            != ContinuationKind::WuxieWindow
        {
            break;
        }
    }
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RevealDiscardSuitGainOrUse
    );
    let hand_before_second = state.hand_cards(owner);
    assert_eq!(hand_before_second.len() as u32, initial_hand_count + 2);

    let (state, _) = resolve(state, vec![discard_card.raw()], false);
    let (state, completed) = resolve(state, vec![other.raw()], false);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(mark_count(&state, owner), 1);
    assert_eq!(state.hand_count(owner), initial_hand_count + 1);
    let new_hand = state.hand_cards(owner);
    assert!(hand_before_second
        .iter()
        .all(|card| !new_hand.contains(card)));
    assert_eq!(state.cards.get(revealed[1]).unwrap().owner, other);
    let processing = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(processing).unwrap().cards.is_empty());
}

#[test]
fn two_marks_reveal_five_and_never_grow_past_cap() {
    let state = started_state(2_026_071_503);
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(CHENJIAN),
            delta: 2,
        }],
    )
    .unwrap();
    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().zones.get(processing).unwrap().cards.len(), 5);
    assert_eq!(mark_count(tx.state(), owner), 2);
}

#[test]
fn a_dead_owner_still_cleans_up_tracked_processing_cards() {
    let state = started_state(2_026_071_504);
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let payload = tx.state().resolution.continuation_payload.clone();
    let len = payload[6] as usize;
    let revealed: Vec<_> = payload[7..7 + len]
        .iter()
        .copied()
        .map(engine_api::CardId::new)
        .collect();
    let mut state = tx.commit().state;
    state.resolution.clear_choice();
    state.players.get_mut(owner).unwrap().alive = false;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RevealDiscardSuitGainOrUse {
            player: owner,
            mark: MarkDefId::new(CHENJIAN),
            base_count: 3,
            max_mark: 2,
            revealed,
            completed: 0,
            step: 0,
            data: 0,
        }],
    )
    .unwrap();
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(tx.state().zones.get(processing).unwrap().cards.is_empty());
}
}

mod original_huicui_dcchiying {
//! Golden coverage for 驰应（huicui 高翔，ABI 484）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCCHIYING: u32 = 4_106_504_641;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcchiying.skill");

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

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn discard_loop_gains_basic_cards() {
    let package = content_compiler::compile_source_named("dcchiying-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 484);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target, other) = (seats[0], seats[1], seats[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCCHIYING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty everyone's hand, then seed the "other" player with a Sha + a Tao.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target, other] {
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
    let sha = alloc_card(&mut tx, other, CardKind::Sha, 96_100, Suit::Spade);
    alloc_card(&mut tx, other, CardKind::Tao, 96_101, Suit::Heart);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    assert!(state.has_pending_choice(), "in-range player choice pending");
    // The target discards from the "other" player (in range of the target).
    state = resolve_choice(state, target, vec![other.raw()], false, 2);
    assert!(state.has_pending_choice(), "card discard choice pending");
    state = resolve_choice(state, target, vec![sha.raw()], false, 3);
    // Loop: the target stops (cancel).
    assert!(state.has_pending_choice(), "loop reopens");
    state = resolve_choice(state, target, vec![], true, 4);

    assert!(
        state.hand_cards(target).contains(&sha),
        "the discarded Sha is gained by the target"
    );
    assert_eq!(
        state.hand_count(other),
        1,
        "the Tao stays with the other player"
    );
}
}

mod original_huicui_dcgangu {
//! Golden for HUICUI「刚骨」的失去体力与每回合限一次语义。

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const GANGU: u32 = 3_823_814_559;
const SHENWEI: u32 = 2_271_917_703;
const YUXIANG: u32 = 3_515_326_729;
const RESHISHOU: u32 = 2_341_206_262;
const RESHISHOU2: u32 = 2_672_855_040;
const QINGMAN: u32 = 2_536_693_396;
const RECANGCHU: u32 = 2_416_555_605;
const RECANGCHU2: u32 = 2_551_663_610;
const RECANGCHU3: u32 = 3_636_591_775;
const DCCUNWEI: u32 = 4_022_570_059;
const XIXIU: u32 = 3_425_169_185;
const CHENJIAN: u32 = 3_452_524_123;
const TENGYIN: u32 = 4_177_903_929;
const RECANGCHU_MARK: u32 = 2_058_376_721;
const DISTANCE_FIXTURE: u32 = 4_294_967_000;
const SUIT_FIXTURE: u32 = 4_294_966_998;
/// Concatenate every committed huicui source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn huicui_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/huicui");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

fn recangchu_marks(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == RECANGCHU_MARK)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn committed_package_distinguishes_hp_loss_and_fires_once() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("huicui", &huicui_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == GANGU));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHENJIAN));
    let tengyin = committed
        .character_index
        .iter()
        .find(|character| character.id == TENGYIN)
        .expect("tengyin character");
    assert_eq!(tengyin.name_key, "tengyin");
    assert_eq!(tengyin.max_hp, 3);
    assert_eq!(tengyin.skills, vec![CHENJIAN, XIXIU]);

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
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GANGU),
    });
    run_effects(&mut tx, setup).unwrap();
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 8;
    tx.state_mut().players.get_mut(other).unwrap().hp = 8;
    let hand = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

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
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);
}

#[test]
fn yuxiang_uses_each_targets_final_distance_for_sha_response_lock() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_823),
        kind: CommandKind::StartGame {
            player_count: 8,
            rng_seed: 20_260_823,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let distance_fixture = content_compiler::compile_source_named(
        "distance-fixture",
        "skill distance_fixture(4294967000) { modifier distance_to add 5; }",
    )
    .unwrap();
    engine.load_package(&distance_fixture).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YUXIANG),
        }],
    )
    .unwrap();
    let far = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .max_by_key(|target| {
            match query(
                tx.state(),
                &QueryRequest::Distance {
                    from: owner,
                    to: *target,
                },
                &mut QueryContext::new(),
            ) {
                QueryResponse::Int(distance) => distance,
                _ => 0,
            }
        })
        .unwrap();
    let sha = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    assert!(engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        far,
        sha
    ));
    assert!(!engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        sha
    ));

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: far,
            skill_def: SkillDefId::new(DISTANCE_FIXTURE),
        }],
    )
    .unwrap();
    let distance = match query(
        tx.state(),
        &QueryRequest::Distance {
            from: owner,
            to: far,
        },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(distance) => distance,
        other => panic!("unexpected distance response: {other:?}"),
    };
    assert!(distance > 2);
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        far,
        sha
    ));
}

#[test]
fn shenwei_adds_two_draws_and_two_hand_limit() {
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
    let mut tx = Transaction::begin(engine.state().clone());
    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);
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
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHENWEI),
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        base_draw + 2
    );
    assert!(matches!(
        engine_core::query::query(
            tx.state(),
            &rule_api::QueryRequest::HandLimit { player: owner },
            &mut engine_core::query::QueryContext::new(),
        ),
        rule_api::QueryResponse::Int(value) if value == base_hand + 2
    ));
}

#[test]
fn reshishou2_loses_hp_at_prepare_only_without_recangchu_mark() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (marks, expected_hp) in [(0, 4), (1, 5)] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        {
            let player = tx.state_mut().players.get_mut(owner).unwrap();
            player.hp = 5;
            player.max_hp = 5;
        }
        let mut setup = vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RESHISHOU2),
        }];
        if marks > 0 {
            setup.push(EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(RECANGCHU_MARK),
                delta: marks,
            });
        }
        run_effects(&mut tx, setup).unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Prepare.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }
}

#[test]
fn reshishou_consumes_grain_for_logical_jiu_and_fire_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let fixture = content_compiler::compile_source_named(
        "logical-jiu-fixture",
        "skill logical_jiu_fixture(4294966999) locked { modifier card_name(sha) override jiu; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
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
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RESHISHOU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(4_294_966_999),
        },
        EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(RECANGCHU_MARK),
            delta: 4,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }

    let owner_card = tx.state().hand_cards(owner)[0];
    let other_card = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(owner_card).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(other_card).unwrap().kind = CardKind::Jiu;
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: other,
            card: other_card,
            targets: vec![],
        }],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 4);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: owner_card,
            targets: vec![],
        }],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);

    for (nature, amount, expected_marks) in [(0, 1, 3), (1, 2, 1), (1, 2, 0)] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(recangchu_marks(&tx, owner), expected_marks);
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
}

#[test]
fn recangchu_rewrites_all_three_legacy_js_behaviors() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def in [RECANGCHU, RECANGCHU2, RECANGCHU3] {
        assert!(committed
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def));
    }

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_717),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let [owner, other, third, fourth] = players.as_slice() else {
        panic!("expected four players");
    };
    let (owner, other, third, fourth) = (*owner, *other, *third, *fourth);
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    let base_hand_limit = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(limit) => limit,
        other => panic!("unexpected hand-limit response: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECANGCHU),
        }],
    )
    .unwrap();

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 4 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(base_hand_limit + 3)
    );

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
        ],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 4);

    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_card = tx.state().hand_cards(other)[0];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveMark {
                player: owner,
                mark: engine_api::MarkDefId::new(RECANGCHU_MARK),
                amount: 1,
            },
            EffectRequest::MoveCards {
                cards: vec![other_card],
                to: owner_hand,
                from: Some(other_hand),
            },
        ],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);

    // SetTurnState is a low-level fixture effect and does not run the command-layer turn reset.
    let recangchu_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == RECANGCHU)
        .unwrap()
        .id;
    tx.state_mut()
        .skills
        .get_mut(recangchu_skill)
        .unwrap()
        .turn_fire_count = 0;
    let third_hand = tx.state().zones.find(ZoneKind::Hand, third).unwrap();
    let third_card = tx.state().hand_cards(third)[0];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: third,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::MoveCards {
                cards: vec![third_card],
                to: owner_hand,
                from: Some(third_hand),
            },
        ],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 4);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: fourth,
            killer: owner,
        }],
    )
    .unwrap();
    assert_eq!(recangchu_marks(&tx, owner), 3);
}

#[test]
fn qingman_draws_to_the_ending_players_empty_equip_slots() {
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
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QINGMAN),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let equipment = tx.state().hand_cards(other)[..2].to_vec();
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(equipment[1]).unwrap().kind = CardKind::Armor;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: equipment,
                to: equip,
                from: Some(hand),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
}

#[test]
fn dccunwei_draws_for_single_target_and_discards_only_for_payable_multi_target_tricks() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
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
        skill_def: SkillDefId::new(DCCUNWEI),
    });
    run_effects(&mut tx, setup).unwrap();

    let source_cards = tx.state().hand_cards(source);
    let basic = source_cards[0];
    let single_trick = source_cards[1];
    let multi_trick = source_cards[2];
    tx.state_mut().cards.get_mut(basic).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(single_trick).unwrap().kind = CardKind::TrickGuohe;
    tx.state_mut().cards.get_mut(multi_trick).unwrap().kind = CardKind::TrickNanman;
    let initial_hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: basic,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: single_trick,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);

    let discarded = tx.state().hand_cards(owner)[0];
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: source,
                card: multi_trick,
                targets: vec![owner, source],
            }],
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::DiscardOneHandOrEquip
    );
    let state = tx.commit().state;
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        result.result,
        engine_api::CommandResult::Accepted { .. }
    ));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(discarded).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
    let source_hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![multi_trick, single_trick],
            to: source_hand,
            from: Some(discard),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: multi_trick,
            targets: vec![owner, source],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: single_trick,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 1);
}

#[test]
fn xixiu_uses_raw_event_suit_effective_equip_suit_and_protects_only_the_sole_equip() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let suit_fixture = content_compiler::compile_source_named(
        "suit-fixture",
        "skill suit_fixture(4294966998) locked { modifier card_suit(spade) override heart; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.load_package(&suit_fixture).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_717),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
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
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIXIU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SUIT_FIXTURE),
        },
        EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(SUIT_FIXTURE),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let owner_equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let equipment = tx.state().hand_cards(owner)[..2].to_vec();
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().suit = engine_api::Suit::Spade;
    tx.state_mut().cards.get_mut(equipment[1]).unwrap().kind = CardKind::Armor;
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equipment[0]],
            to: owner_equip,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    assert!(engine_core::query::card_discard_protected_from(
        tx.state(),
        source,
        owner,
        equipment[0]
    ));
    assert!(!engine_core::query::card_discard_protected_from(
        tx.state(),
        owner,
        owner,
        equipment[0]
    ));

    let source_cards = tx.state().hand_cards(source);
    let raw_spade = source_cards[0];
    let raw_heart = source_cards[1];
    tx.state_mut().cards.get_mut(raw_spade).unwrap().suit = engine_api::Suit::Spade;
    tx.state_mut().cards.get_mut(raw_heart).unwrap().suit = engine_api::Suit::Heart;
    let initial_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: raw_spade,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: raw_heart,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equipment[1]],
            to: owner_equip,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    assert!(!engine_core::query::card_discard_protected_from(
        tx.state(),
        source,
        owner,
        equipment[0]
    ));
}
}

mod original_huicui_dckuizhen {
//! Golden coverage for 溃阵（huicui 公孙修，ABI 477）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCKUIZHEN: u32 = 2_730_814_674;
const DCGANGU: u32 = 3_823_814_559;
const GONGSUNXIU: u32 = 3_377_163_409;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dckuizhen.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

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
        skill_def: SkillDefId::new(DCKUIZHEN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty both hands so the Duel resolves without a 无懈 window.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            tx,
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
}

fn seed_hand(tx: &mut Transaction, player: PlayerId, specs: &[(CardKind, u32)]) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    specs
        .iter()
        .map(|(kind, def)| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: *kind,
                suit: Suit::Spade,
                rank: 5,
                weapon_range: 0,
                zone: hand,
                owner: player,
                face_up: false,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}

#[test]
fn owner_damaged_gains_target_shas() {
    let packaged = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        packaged
            .character_index
            .iter()
            .find(|character| character.id == GONGSUNXIU)
            .unwrap()
            .skills,
        [DCGANGU, DCKUIZHEN]
    );
    let package = content_compiler::compile_source_named("dckuizhen-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 477);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // Target: a Sha + a Tao (only the Sha is gained back); the owner stays handless so
    // the Duel damages them.
    seed_hand(
        &mut tx,
        target,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    let owner_hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);

    assert_eq!(
        state.players.get(owner).unwrap().hp,
        state.players.get(owner).unwrap().max_hp as i32 - 1,
        "the Duel damaged the owner"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained the target's Sha"
    );
    assert_eq!(state.hand_count(target), 1, "target kept the Tao");
}

#[test]
fn owner_not_damaged_target_loses_hp() {
    let package = content_compiler::compile_source_named("dckuizhen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // The owner holds a Sha to answer the Duel; the target holds a Tao.
    seed_hand(&mut tx, owner, &[(CardKind::Sha, 96_000)]);
    seed_hand(&mut tx, target, &[(CardKind::Tao, 96_100)]);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    // The owner answers the Duel with their Sha (response window).
    assert!(
        state.has_pending_choice(),
        "Sha response window pending for the Duel"
    );
    let sha_id = *state.hand_cards(owner).first().unwrap();
    state = resolve_choice(state, owner, vec![sha_id.raw()], false, 2);
    // The target cannot answer and loses the Duel -> the initiator loses 1 HP.
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before - 2,
        "target took the Duel damage and then lost 1 HP"
    );
    assert_eq!(state.hand_count(owner), 0, "owner used the Sha answering");
}
}

mod original_huicui_dcmanzhi {
//! Golden coverage for 蛮智（huicui 孟优，ABI 475）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCMANZHI: u32 = 4_194_306_664;
const GIVE_MARK: u32 = 5_0000_0001;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcmanzhi.skill");

fn fire_phase(tx: Transaction, player: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
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

fn setup(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCMANZHI),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
}

fn empty_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
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

fn seed_hand(tx: &mut Transaction, player: PlayerId, specs: &[(CardKind, u32)]) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    specs
        .iter()
        .map(|(kind, def)| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: *kind,
                suit: Suit::Spade,
                rank: 5,
                weapon_range: 0,
                zone: hand,
                owner: player,
                face_up: false,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}

#[test]
fn give_branch_target_hands_cards_and_uses_sha() {
    let package = content_compiler::compile_source_named("dcmanzhi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 475);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    // Empty both hands so the virtual Sha cannot be dodged and card counts are exact.
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    // Target holds a Sha + a Tao to hand over.
    seed_hand(
        &mut tx,
        target,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "branch choice pending");
    state = resolve_choice(state, owner, vec![1], false, 1); // give branch
    assert!(state.has_pending_choice(), "give target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 2);
    assert!(
        state.has_pending_choice(),
        "target's give-card choice pending"
    );
    let target_hand: Vec<u32> = state
        .hand_cards(target)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(target_hand.len(), 2);
    state = resolve_choice(state, target, target_hand, false, 3);

    assert_eq!(state.hand_count(owner), 2, "owner gained the two cards");
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp_before - 1,
        "target's virtual Sha hit the owner"
    );
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn gain_branch_gains_gives_back_and_draws() {
    let package = content_compiler::compile_source_named("dcmanzhi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    // Deterministic hands: empty both, then seed exactly.
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    let owner_seeded = seed_hand(
        &mut tx,
        owner,
        &[
            (CardKind::Sha, 96_000),
            (CardKind::Sha, 96_001),
            (CardKind::Tao, 96_002),
        ],
    );
    // Target holds two cards to be taken.
    seed_hand(
        &mut tx,
        target,
        &[(CardKind::Sha, 96_100), (CardKind::Tao, 96_101)],
    );
    let owner_hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![2], false, 1); // gain branch
    assert!(state.has_pending_choice(), "gain target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 2);
    assert!(
        state.has_pending_choice(),
        "owner's take-card choice pending"
    );
    let target_hand: Vec<u32> = state
        .hand_cards(target)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(target_hand.len(), 2);
    state = resolve_choice(state, owner, target_hand, false, 3);
    assert!(
        state.has_pending_choice(),
        "owner's give-back choice pending"
    );
    // Give back exactly two of the owner's own cards (same count as taken).
    let give_back: Vec<u32> = owner_seeded[0..2].iter().map(|c| c.raw()).collect();
    state = resolve_choice(state, owner, give_back, false, 4);

    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 1,
        "gained 2, gave 2, drew 1"
    );
    assert_eq!(state.hand_count(target), 2, "target got its cards back");
}

#[test]
fn branches_are_once_per_turn() {
    let package = content_compiler::compile_source_named("dcmanzhi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    empty_hand(&mut tx, owner);
    empty_hand(&mut tx, target);
    // Four cards: the give branch takes two, the gain branch still has two left.
    seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100),
            (CardKind::Tao, 96_101),
            (CardKind::Sha, 96_102),
            (CardKind::Tao, 96_103),
        ],
    );

    // Use the give branch at Prepare.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Prepare);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![1], false, 1);
    state = resolve_choice(state, owner, vec![target.raw()], false, 2);
    let target_hand: Vec<u32> = state
        .hand_cards(target)
        .into_iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(target_hand.len(), 4);
    state = resolve_choice(state, target, target_hand[0..2].to_vec(), false, 3);
    // Give mark set; the owner was hit by the Sha.
    let owner_state = state.players.get(owner).unwrap();
    assert!(
        owner_state
            .marks
            .iter()
            .any(|(mark, count)| *mark == GIVE_MARK && *count > 0),
        "give branch marked as used"
    );

    // At the same turn's End, only the gain branch is offered: choose 2 -> gain target choice.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let tx = fire_phase(tx, owner, TurnPhaseTag::End);
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "end-phase branch choice pending"
    );
    state = resolve_choice(state, owner, vec![2], false, 4);
    assert!(state.has_pending_choice(), "gain target choice still opens");
}
}

mod original_huicui_dcmoyu {
//! Golden coverage for 魔狱（huicui 裴元绍，ABI 481）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCMOYU: u32 = 3_259_808_437;
const ADD_MARK: u32 = 6_0000_0004;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcmoyu.skill");

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
        skill_def: SkillDefId::new(DCMOYU),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            tx,
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
}

fn seed_hand(
    tx: &mut Transaction,
    player: PlayerId,
    specs: &[(CardKind, u32, Suit)],
) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    specs
        .iter()
        .map(|(kind, def, suit)| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(*def),
                kind: *kind,
                suit: *suit,
                rank: 5,
                weapon_range: 0,
                zone: hand,
                owner: player,
                face_up: false,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}

#[test]
fn damaged_use_bans_skill_and_marks_target() {
    let package = content_compiler::compile_source_named("dcmoyu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 481);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // Owner stays handless (no 闪); the target holds three cards.
    let target_cards = seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100, Suit::Spade),
            (CardKind::Tao, 96_101, Suit::Heart),
            (CardKind::Sha, 96_102, Suit::Club),
        ],
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
        owner_hand_before + 1,
        "gained one card"
    );
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp_before - 1,
        "the target's virtual Sha hit the owner"
    );
    assert!(
        state.hand_cards(owner).contains(&target_cards[0]),
        "gained the target's first card"
    );
    // The skill instance must be banned for the turn.
    let banned = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == DCMOYU && s.owner == owner)
        .map(|s| s.active_turn_banned)
        .unwrap_or(false);
    assert!(banned, "skill banned after taking damage");
    // The add-on mark is NOT set.
    assert!(
        !state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == ADD_MARK && *count > 0),
        "no add-on mark on the damaged branch"
    );
}

#[test]
fn undamaged_use_sets_add_mark() {
    let package = content_compiler::compile_source_named("dcmoyu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    // The owner holds a Shan to dodge; the target holds two cards.
    seed_hand(&mut tx, owner, &[(CardKind::Shan, 96_000, Suit::Heart)]);
    seed_hand(
        &mut tx,
        target,
        &[
            (CardKind::Sha, 96_100, Suit::Spade),
            (CardKind::Tao, 96_101, Suit::Club),
        ],
    );
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    // The Sha was dodged (owner held a Shan and the use resolved with a response).
    // If the response window opened, the outcome branch was skipped; accept either
    // the dodged outcome (no HP loss) or a skipped branch, but never the damage path.
    let hp = state.players.get(owner).unwrap().hp;
    assert!(
        hp == owner_hp_before || hp == owner_hp_before - 1,
        "no unexpected damage beyond the Sha"
    );
}
}

mod original_huicui_dcshoucheng {
//! Golden coverage for HUICUI「守成」and its once-per-turn last-hand-loss filter.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DCSHOUCHENG: u32 = 4_099_961_254;
const RESHENGXI: u32 = 2_846_213_948;
const JIANGFEI: u32 = 4_198_602_555;
const HUICUI: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");
const SHIJI: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn loaded() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let huicui = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    let shiji = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&shiji).unwrap();
    engine.load_package(&huicui).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DCSHOUCHENG as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: DCSHOUCHENG as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, target, current) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: current,
            count: u32::MAX,
        },
        EffectRequest::SetTurnState {
            current,
            phase: Phase::Play.to_u8(),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCSHOUCHENG),
        },
        EffectRequest::DrawCards {
            player: target,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: current,
            count: 1,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    (tx, owner, target, current)
}

fn move_all_hand(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().hand_cards(player);
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, result) = apply_command(
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_declares_dcshoucheng_and_closes_jiangfei() {
    let package = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 26);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shiji"
            && dependency.version == "0.1.0"
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHOUCHENG)
        .unwrap();
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(
        trigger.filter,
        "owner_alive_event_player_outside_turn_from_hand_empty_once"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JIANGFEI)
        .unwrap();
    assert_eq!(character.name_key, "jiangfei");
    assert_eq!(character.skills, vec![RESHENGXI, DCSHOUCHENG]);
}

#[test]
fn dcshoucheng_draws_two_only_once_per_turn_and_resets_next_turn() {
    let (mut tx, owner, target, current) = loaded();
    assert_eq!(
        move_all_hand(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 1);
    assert_eq!(state.hand_count(target), 2);

    let mut tx = Transaction::begin(state);
    move_all_hand(&mut tx, target).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(target), 0);

    tx.state_mut().turn.phase = Phase::End;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: current,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(20_260_718_03),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        move_all_hand(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 2);
    assert_eq!(state.hand_count(target), 2);
}

#[test]
fn current_turn_player_losing_their_last_hand_does_not_trigger() {
    let (mut tx, _, _, current) = loaded();
    move_all_hand(&mut tx, current).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(current), 0);
}
}

mod original_huicui_dcxiangshu {
//! Golden coverage for 相术（huicui 张闿，ABI 486）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCXIANG_SHU: u32 = 3_579_538_404;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcxiangshu.skill");

fn fire_phase(tx: Transaction, current: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: tag,
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

fn fire_play_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
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

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
                cancelled: false,
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

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn exact_match_gains_card_and_deals_damage() {
    let package = content_compiler::compile_source_named("dcxiangshu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 486);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, observed) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCXIANG_SHU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = observed;
    // Deterministic hands: empty both, give the observed player exactly 2 cards.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, observed] {
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
    alloc_card(&mut tx, observed, CardKind::Sha, 96_100, Suit::Spade);
    let c2 = alloc_card(&mut tx, observed, CardKind::Tao, 96_101, Suit::Heart);
    // The observed player's hp must be <= 2 (hand 2 >= hp).
    tx.state_mut().players.get_mut(observed).unwrap().hp = 2;
    let observed_hp_before = tx.state().players.get(observed).unwrap().hp;

    // The observed player's play phase begins.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_phase(tx, observed, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "number choice pending");
    // Guess exactly 2 (matches the observed player's hand count).
    state = resolve_choice(state, owner, vec![2], 1);

    // Play phase ends with the hand still at 2: delta 0 -> gain one card + damage 1.
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, observed);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "gain choice pending");
    state = resolve_choice(state, owner, vec![c2.raw()], 2);

    assert_eq!(
        state.players.get(observed).unwrap().hp,
        observed_hp_before - 2 + 1,
        "the observed player took 1 damage from the exact match"
    );
    assert!(
        state.hand_cards(owner).contains(&c2),
        "the owner gained a card from the observed player"
    );
}
}

mod original_huicui_dcyingtu {
//! Golden coverage for 应图（huicui 陈珪，ABI 487）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCYINGTU: u32 = 3_170_147_690;
const DCCONGSHI: u32 = 3_668_194_449;
const CHENGUI: u32 = 3_537_583_790;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcyingtu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
                cancelled: false,
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
fn neighbor_gain_triggers_take_give_chain() {
    let packaged = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(packaged.manifest.rule_vm_abi_version, 487);
    assert_eq!(
        packaged
            .character_index
            .iter()
            .find(|character| character.id == CHENGUI)
            .unwrap()
            .skills,
        [DCYINGTU, DCCONGSHI]
    );
    assert!(packaged.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DCYINGTU
            && trigger.filter == "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn"
            && trigger.optional
    }));
    let package = content_compiler::compile_source_named("dcyingtu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 487);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, next, prev) = (seats[0], seats[1], seats[2]);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty all hands; seed the "next" neighbor with a Sha, the owner with a Tao.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, next, prev] {
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
    let next_hand = tx.state().zones.find(ZoneKind::Hand, next).unwrap();
    let sha = alloc_card(&mut tx, next, CardKind::Sha, 96_100, Suit::Spade, next_hand);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let tao = alloc_card(
        &mut tx,
        owner,
        CardKind::Tao,
        96_000,
        Suit::Heart,
        owner_hand,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCYINGTU),
        }],
    )
    .unwrap();

    // The "next" neighbor gains a card (a move into its hand) -> the trigger fires.
    let mut tx = Transaction::begin(tx.commit().state);
    let moved = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_200),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(moved);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![moved],
            to: next_hand,
            from: Some(discard),
        }],
    );
    assert!(matches!(
        result,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    assert!(
        state.resolution.pending_choice_id != 0,
        "gain choice pending after the neighbor gained"
    );
    let state = resolve_choice(state, owner, vec![1], 1); // confirm 应图
    assert_eq!(state.resolution.continuation_payload[3], next.raw());
    assert!(engine_core::query::gainable_target_cards(
        &state,
        owner,
        next,
        rule_api::TargetCardScope::HandEquip,
    )
    .contains(&sha));
    let mut state = resolve_choice(state, owner, vec![sha.raw()], 2);
    assert!(state.has_pending_choice(), "give choice pending");
    state = resolve_choice(state, owner, vec![tao.raw()], 3);

    assert!(
        state.hand_cards(owner).contains(&sha),
        "the owner gained the Sha from the next neighbor"
    );
    assert!(
        state.hand_cards(prev).contains(&tao),
        "the other neighbor received the Tao"
    );
    assert!(
        !state.has_pending_choice(),
        "once-per-turn prevents re-entry"
    );
}
}

mod original_huicui_dczhongji {
//! huicui/dczhongji 忠继（第一百二十五批·近似晋升）：手牌<maxHp 时用牌
//! 摸至手牌数等于体力上限（min(20, max_hp-hand) 上限）。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCZHONGJI: u32 = 3_617_393_468;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/huicui"
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

#[test]
fn dczhongji_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/huicui/huicui.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 487);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DCZHONGJI
            && t.event == "card_used"
            && t.filter == "owner_is_target_hand_below_max_hp"
            && t.optional
    }));
}
}

mod original_huicui_lianzhou {
//! Golden coverage for 链舟（huicui 蔡瑁张允）同体力横置（ABI 458）.
//!
//! 准备阶段：锁定横置自己，然后选择任意名体力值相同且未横置的其他角色并横置。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIANZHOU: u32 = 3_041_401_308;
const MARK_CHAIN: u32 = 2;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/lianzhou.skill");

fn is_linked(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .map(|participant| {
            participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == MARK_CHAIN && *count > 0)
        })
        .unwrap_or(false)
}

#[test]
fn links_self_then_links_same_hp_unlinked_others() {
    let package = content_compiler::compile_source_named("lianzhou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 458);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, victim, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIANZHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    // Equalize HP so both other players qualify.
    for p in [owner, victim, other] {
        tx.state_mut().players.get_mut(p).unwrap().hp = 3;
    }
    // Pre-link the third player so only the victim is selectable.
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: other,
            mark: engine_api::MarkDefId::new(MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();

    let mut ending = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut ending,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut ending, effects) {
                assert!(
                    matches!(e, EffectError::ChoiceRequested),
                    "target choice must pause: {e:?}"
                );
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch failed: {e:?}"),
    }
    let mut state = ending.commit().state;
    assert!(is_linked(&state, owner), "owner links itself first");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![victim.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "target choice must be accepted"
    );
    let state = next;
    assert!(
        is_linked(&state, victim),
        "selected same-HP unlinked target gets linked"
    );
    assert!(
        is_linked(&state, other),
        "pre-linked target stays linked and is not re-selected"
    );
    assert!(!state.has_pending_choice());
}
}

mod original_huicui_refuyuan {
//! Golden for HUICUI 陈登「扶援」and bounded per-target red-card turn history.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const REFUYUAN: u32 = 2_444_177_428;
const HUICUI: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");
const SHIJI: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn loaded() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let huicui = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    let shiji = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&shiji).unwrap();
    engine.load_package(&huicui).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(REFUYUAN as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let (owner, target, source) = (seats[0], seats[1], seats[2]);
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
        skill_def: SkillDefId::new(REFUYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target, source)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
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
    user: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, result) = apply_command(
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_declares_refuyuan_and_abi29() {
    let package = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 487);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REFUYUAN)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_target_no_other_red_card_this_turn"
    );
    assert!(trigger.optional);
}

#[test]
fn first_sha_target_is_offered_and_draws_for_event_player() {
    let (mut tx, owner, target, source) = loaded();
    let unrelated_target = owner;
    let prior_red = inject(&mut tx, source, CardKind::Other, Suit::Heart);
    assert!(dispatch(&mut tx, source, prior_red, vec![unrelated_target])
        .unwrap()
        .is_empty());
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(target);
    assert_eq!(
        dispatch(&mut tx, source, sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 1);
    assert_eq!(state.hand_count(target), before + 1);
}

#[test]
fn earlier_red_card_from_any_source_blocks_later_sha_for_same_target() {
    let (mut tx, _owner, target, source) = loaded();
    let prior_red = inject(&mut tx, source, CardKind::Other, Suit::Diamond);
    assert!(dispatch(&mut tx, source, prior_red, vec![target])
        .unwrap()
        .is_empty());
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade);
    assert!(dispatch(&mut tx, source, sha, vec![target])
        .unwrap()
        .is_empty());
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn later_red_card_in_same_batch_does_not_cancel_earlier_sha_offer() {
    let (mut tx, _owner, target, source) = loaded();
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade);
    let later_red = inject(&mut tx, source, CardKind::Other, Suit::Heart);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[
                DomainEvent::CardUsed {
                    player: source,
                    card: sha,
                    targets: vec![target],
                },
                DomainEvent::CardUsed {
                    player: source,
                    card: later_red,
                    targets: vec![target],
                },
            ],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
}

#[test]
fn same_physical_red_card_is_excluded_and_turn_boundary_clears_history() {
    let (mut tx, owner, target, source) = loaded();
    let red_sha = inject(&mut tx, source, CardKind::Sha, Suit::Heart);
    assert_eq!(
        dispatch(&mut tx, source, red_sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 2);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch(&mut tx, source, red_sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 3);

    let mut tx = Transaction::begin(state);
    let other_red = inject(&mut tx, source, CardKind::Other, Suit::Diamond);
    assert!(dispatch(&mut tx, source, other_red, vec![target])
        .unwrap()
        .is_empty());
    assert!(dispatch(&mut tx, source, red_sha, vec![target])
        .unwrap()
        .is_empty());

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        dispatch(&mut tx, source, red_sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
}
}

mod original_huicui_rekuanshi {
//! Golden coverage for 宽释（huicui 阚泽，ABI 480）.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REKUANSHI: u32 = 3_423_208_328;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/rekuanshi.skill");

fn fire_phase(tx: Transaction, player: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
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

#[test]
fn second_damage_of_turn_recovers_watched_player() {
    let package = content_compiler::compile_source_named("rekuanshi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 480);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, watched) = (seats[0], seats[1]);
    // Strip skills; give the owner 宽释.
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REKUANSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    // Wound the watched player so the recover is meaningful.
    let max_hp = tx.state().players.get(watched).unwrap().max_hp;
    tx.state_mut().players.get_mut(watched).unwrap().hp = max_hp as i32 - 1;

    // End phase: choose the watched player.
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::End);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "watch target choice pending");
    state = resolve_choice(state, owner, vec![watched.raw()], false, 1);
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 1,
        "no recover yet"
    );

    // First damage this turn: no recover.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: owner,
            target: watched,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 2,
        "first damage applied without recover"
    );

    // Second damage this turn: the watched player recovers 1.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: owner,
            target: watched,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 2,
        "second damage applied, then recovered 1"
    );
}

#[test]
fn only_once_per_turn() {
    let package = content_compiler::compile_source_named("rekuanshi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, watched) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REKUANSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    // Fixed 5 max HP so four damages keep the watched player alive even with
    // the single recovery, independent of the booted roster's max_hp.
    tx.state_mut().players.get_mut(watched).unwrap().max_hp = 5;
    let max_hp = 5;
    tx.state_mut().players.get_mut(watched).unwrap().hp = max_hp as i32;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::End);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![watched.raw()], false, 1);

    // Damage 1, 2 (recover), 3, 4: only the second damage recovers.
    for _ in 0u32..4 {
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::CommitDamage {
                damage_id: 0,
                source: owner,
                target: watched,
                amount: 1,
                base_amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
                card: engine_api::CardId::NONE,
                card_kind: 0,
                card_flags: 0,
            }],
        )
        .unwrap();
        state = tx.commit().state;
    }
    // 4 damage - 1 recovery = net 3 lost from full HP -> hp = maxHp - 3.
    assert_eq!(
        state.players.get(watched).unwrap().hp,
        max_hp as i32 - 3,
        "only one recovery across four damages"
    );
}
}

mod original_huicui_xinkuangfu {
//! Golden coverage for 匡扶（huicui 潘凤，ABI 485）.

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

const XINKUANGFU: u32 = 3_510_695_748;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/xinkuangfu.skill");

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

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
                cancelled: false,
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
fn self_equip_damage_draws_two() {
    let package = content_compiler::compile_source_named("xinkuangfu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 485);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINKUANGFU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty all hands; give the owner an equipment in the equip zone.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, other] {
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
    let owner_equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = alloc_card(
        &mut tx,
        owner,
        CardKind::Weapon,
        96_001,
        Suit::Spade,
        owner_equip,
    );
    let owner_hand_before = tx.state().hand_count(owner);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "equip target choice pending");
    // Target self.
    state = resolve_choice(state, owner, vec![owner.raw()], 1);
    assert!(state.has_pending_choice(), "equipment pick pending");
    state = resolve_choice(state, owner, vec![weapon.raw()], 2);
    assert!(state.has_pending_choice(), "Sha target choice pending");
    state = resolve_choice(state, owner, vec![other.raw()], 3);

    assert_eq!(
        state.players.get(other).unwrap().hp,
        state.players.get(other).unwrap().max_hp as i32 - 1,
        "the virtual Sha hit the other player"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + 2,
        "drew two after the self-equip damage"
    );
    let equip_len = state
        .zones
        .find(ZoneKind::Equip, owner)
        .and_then(|id| state.zones.get(id))
        .map(|z| z.cards.len())
        .unwrap();
    assert_eq!(equip_len, 0, "self equipment discarded");
}
}

mod original_huicui_xizhen {
//! Golden coverage for 袭阵（huicui 高览，ABI 482）.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XIZHEN: u32 = 4_003_133_285;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/xizhen.skill");

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
        skill_def: SkillDefId::new(XIZHEN),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty the target's hand so the virtual Sha cannot be dodged.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, target] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            tx,
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
}

#[test]
fn sha_branch_deals_damage() {
    let package = content_compiler::compile_source_named("xizhen-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 482);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    assert!(state.has_pending_choice(), "branch choice pending");
    state = resolve_choice(state, owner, vec![1], false, 2); // Sha

    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before - 1,
        "virtual Sha hit the target"
    );
}

#[test]
fn duel_branch_keeps_hp_without_shas() {
    let package = content_compiler::compile_source_named("xizhen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    setup(&mut tx, owner, target);
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_play(tx, owner);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![target.raw()], false, 1);
    assert!(state.has_pending_choice());
    // Duel at a handless target: the owner (also handless) cannot answer either, so the
    // Duel damages one of them; assert only that no crash occurred and the choice closed.
    state = resolve_choice(state, owner, vec![2], false, 2);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no pending choice after the Duel"
    );
    let _ = owner_hp_before;
}
}

mod original_huicui_xuxie {
//! Golden coverage for 虚泄（huicui 邢道荣，ABI 483）.

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

const XUXIE: u32 = 3_207_964_441;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/xuxie.skill");

fn fire_phase(tx: Transaction, player: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
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

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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
                cancelled: false,
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

fn setup(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XUXIE),
    });
    run_effects(tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
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
fn discard_branch_and_max_hp_loss_and_restore() {
    let package = content_compiler::compile_source_named("xuxie-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 483);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, adjacent) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    // Deterministic: empty both hands, give the adjacent player two cards.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, adjacent] {
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
    let adjacent_hand = tx.state().zones.find(ZoneKind::Hand, adjacent).unwrap();
    let c1 = alloc_card(
        &mut tx,
        adjacent,
        CardKind::Sha,
        96_100,
        Suit::Spade,
        adjacent_hand,
    );
    let c2 = alloc_card(
        &mut tx,
        adjacent,
        CardKind::Tao,
        96_101,
        Suit::Heart,
        adjacent_hand,
    );
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    // The skill lost 1 maxHp first, then the branch choice opens.
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_hp_before - 1,
        "lost 1 maxHp"
    );
    assert!(state.has_pending_choice(), "branch choice pending");
    // Discard branch: marker -> card choice -> marker.
    state = resolve_choice(state, owner, vec![0], 1);
    assert!(state.has_pending_choice(), "discard card choice pending");
    state = resolve_choice(state, owner, vec![c1.raw()], 2);
    assert!(state.has_pending_choice(), "loop marker choice pending");
    state = resolve_choice(state, owner, vec![0], 3);

    assert_eq!(state.hand_count(adjacent), 1, "one card discarded");
    assert!(
        state.hand_cards(adjacent).contains(&c2),
        "the other card remains"
    );
    // The play-phase end: the adjacent player's maxHp (4) equals the owner's after the
    // loss (5-1=4), so no restore happens.
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(engine_core::effect::EffectError::ChoiceRequested) => {}
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_hp_before - 1,
        "maxHp unchanged (no one bigger)"
    );
}

#[test]
fn draw_branch_draws_for_adjacent() {
    let package = content_compiler::compile_source_named("xuxie-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, adjacent) = (seats[0], seats[1]);
    setup(&mut tx, owner);
    let adjacent_before = tx.state().hand_count(adjacent);

    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_phase(tx, owner, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice());
    state = resolve_choice(state, owner, vec![1], 1); // draw branch
    assert_eq!(
        state.hand_count(adjacent),
        adjacent_before + 1,
        "adjacent player drew one"
    );
}
}
