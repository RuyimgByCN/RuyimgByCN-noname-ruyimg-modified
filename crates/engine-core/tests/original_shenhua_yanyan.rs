//! Golden coverage for 拒战（shenhua 严颜，转换技，ABI 571）.

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

const JUZHAN: u32 = 3_174_379_257;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/yanyan.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("yanyan-test", SOURCE).unwrap();
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
        skill_def: SkillDefId::new(JUZHAN),
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

/// 阴分支（首次，受杀）：双方各摸一张，来源本回合不能对持有者使用牌。
#[test]
fn juzhan_yin_both_draw_and_ban_source() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_900,
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
    assert!(state.has_pending_choice(), "拒战 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 1, "owner drew one");
    assert_eq!(
        state.hand_count(setup.other),
        2,
        "source drew one on top of the Sha"
    );
    assert!(
        state
            .turn
            .use_ban_pairs
            .contains(&(setup.other, setup.owner)),
        "source banned against owner"
    );
    let _ = sha;
}

/// 阳分支（第二次，用杀）：获得目标一张牌并禁己方用牌。
#[test]
fn juzhan_yang_gains_card_and_bans_owner() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let gift = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_901,
        Suit::Heart,
        6,
    );
    let owner_hand = hand_zone(&tx, setup.owner);
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_902,
        Suit::Spade,
        8,
    );
    let state = tx.commit().state;
    // First use flips to branch 阳; the 阴 branch fired earlier is simulated by
    // pre-seeding the branch so the first card_used is branch 阳.
    let mut tx = Transaction::begin(state);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == JUZHAN)
        .map(|skill| skill.id)
        .unwrap();
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"JZ1\x01".to_vec();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: sha,
            targets: vec![setup.other],
        },
    );
    assert!(state.has_pending_choice(), "拒战 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "拒战 gain prompt");
    let state = resolve(state, setup.owner, vec![gift.raw()], false, 9_011);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.owner).contains(&gift),
        "owner gained the card"
    );
    assert!(
        state
            .turn
            .use_ban_pairs
            .contains(&(setup.owner, setup.other)),
        "owner banned against the target"
    );
}

/// 修正器：被禁角色使用牌指定持有者时目标被取消。
#[test]
fn juzhan_ban_cancels_banned_user_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .use_ban_pairs
        .push((setup.other, setup.owner));
    let other_hand = hand_zone(&tx, setup.other);
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_903,
        Suit::Spade,
        9,
    );
    let state = tx.commit().state;
    let cancelled = engine_core::query::card_target_cancelled_for_kind(
        &state,
        setup.other,
        setup.owner,
        CardKind::Tao,
    );
    assert!(cancelled, "banned user's Sha cancelled against the owner");
    // Unbanned user is unaffected.
    let ok = engine_core::query::card_target_cancelled_for_kind(
        &state,
        setup.far,
        setup.owner,
        CardKind::Tao,
    );
    assert!(!ok, "unbanned user not cancelled");
}

#[test]
fn package_exposes_juzhan_contract() {
    let compiled = content_compiler::compile_source_named("yanyan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 571);
    assert_eq!(compiled.skill_index.len(), 1);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == JUZHAN)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|t| t.optional));
    assert_eq!(compiled.modifier_index.len(), 1);
}
