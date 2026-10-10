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
