//! Golden coverage for 享乐/放权/若愚（shenhua 刘禅，ABI 560）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XIANGLE: u32 = 2_880_679_348;
const FANGQUAN: u32 = 283_021_192;
const RUOYU: u32 = 2_227_756_813;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/liushan.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("liushan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(XIANGLE),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANGQUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
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

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_liushan_contract() {
    let compiled = content_compiler::compile_source_named("liushan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 560);
    assert_eq!(compiled.skill_index.len(), 3);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| {
            t.skill_def_id == XIANGLE || t.skill_def_id == FANGQUAN || t.skill_def_id == RUOYU
        })
        .collect();
    assert_eq!(triggers.len(), 4);
    let xiangle = triggers.iter().find(|t| t.skill_def_id == XIANGLE).unwrap();
    assert_eq!(xiangle.event, "card_targeted");
    assert!(!xiangle.optional);
    let fangquan: Vec<_> = triggers
        .iter()
        .filter(|t| t.skill_def_id == FANGQUAN)
        .collect();
    assert_eq!(fangquan.len(), 2);
    assert!(fangquan.iter().all(|t| t.optional));
}

/// 享乐：使用者弃基本牌则杀正常；拒绝则此杀对刘禅无效（无响应、无伤害）。
#[test]
fn xiangle_refusal_excludes_owner_from_sha() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    // The Sha user has a basic (桃) and a non-basic (锦囊牌用 sha 类占位) to discard.
    let other_hand = hand_zone(&tx, setup.other);
    let basic = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_700,
        Suit::Heart,
        6,
    );
    // The Sha card targeting the owner.
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_701,
        Suit::Spade,
        7,
    );
    let mut state = tx.commit().state;
    // Register the currently-resolving Sha so the saved-target exclusion works.
    let mut resolution = &mut state.resolution;
    resolution.resume_kind = engine_core::state::ContinuationKind::ResumePlayedSha;
    resolution.resume_payload = vec![setup.other.raw(), setup.owner.raw(), 1, 0, 1, sha.raw()];
    resolution.register_sha_resolution(setup.other, sha, &[setup.owner], false);
    drop(resolution);
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.owner],
        },
    );
    // The user's discard-basic choice opened.
    assert!(state.has_pending_choice(), "享乐 discard prompt");
    // Refuse: the Sha becomes invalid vs the owner.
    let state = resolve(state, setup.other, Vec::new(), true, 9_001);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.other).contains(&basic),
        "basic still in hand"
    );
    let saved_targets = state
        .resolution
        .sha_resolution_records
        .iter()
        .find(|record| record.card == sha)
        .map(|record| record.targets.clone())
        .unwrap_or_default();
    assert!(
        !saved_targets.contains(&setup.owner),
        "owner excluded from the saved Sha targets"
    );
    let _ = basic;
}

/// 享乐：使用者弃基本牌后杀继续正常结算。
#[test]
fn xiangle_discard_keeps_sha() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let basic = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_702,
        Suit::Heart,
        6,
    );
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_703,
        Suit::Spade,
        7,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.owner],
        },
    );
    assert!(state.has_pending_choice(), "享乐 discard prompt");
    let state = resolve(state, setup.other, vec![basic.raw()], false, 9_002);
    assert!(!state.has_pending_choice());
    assert!(
        !state.hand_cards(setup.other).contains(&basic),
        "basic discarded"
    );
    let _ = sha;
}

/// 放权：出牌阶段跳过 + 结束阶段弃牌授额外回合。
#[test]
fn fangquan_skips_play_and_grants_extra_turn() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_704,
        Suit::Club,
        8,
    );
    let state = tx.commit().state;
    // Accept the optional 放权 confirm: the play phase is skipped.
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Play,
        },
    );
    assert!(state.has_pending_choice(), "放权 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert_eq!(state.turn.phase, Phase::Discard, "play phase skipped");
    // Move to the end phase and trigger 放权's grant.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "放权 grant confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_011);
    assert!(state.has_pending_choice(), "放权 discard prompt");
    let state = resolve(state, setup.owner, vec![card.raw()], false, 9_012);
    assert!(state.has_pending_choice(), "放权 target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_013);
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.extra_turn_queue.first(), Some(&setup.far));
    assert!(
        !state.hand_cards(setup.owner).contains(&card),
        "card discarded"
    );
}

/// 若愚：体力值最低时觉醒：加上限、回复、授予激将。
#[test]
fn ruoyu_awakens_at_min_hp() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(
        &content_compiler::compile_source_named("jijiang", "skill jijiang(2377283906) {}").unwrap(),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.owner,
            skill_def: SkillDefId::new(RUOYU),
        }],
    )
    .unwrap();
    // Owner at min HP (1 of 3): the others at 3.
    // Bootstrap characters are 5-max; drop the owner to the lowest hp.
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: setup.owner,
            delta: -2,
        }],
    )
    .unwrap();
    let max_before = tx.state().players.get(setup.owner).unwrap().max_hp;
    let hp_before = tx.state().players.get(setup.owner).unwrap().hp;
    assert_eq!((hp_before, max_before), (3, 3), "owner wounded to min hp");
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    let owner = state.players.get(setup.owner).unwrap();
    assert_eq!(owner.max_hp, 4, "max hp raised by one");
    assert_eq!(owner.hp, 4, "recovered to full");
    assert!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == 2_377_283_906),
        "激将 granted"
    );
    // Awaken once only: a second Prepare dispatch does nothing further.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    let owner = state.players.get(setup.owner).unwrap();
    assert_eq!(owner.max_hp, 4, "no double awaken");
}
