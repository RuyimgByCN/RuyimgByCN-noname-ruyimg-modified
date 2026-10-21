//! Golden coverage for JSRG 高翔「驰应」(ABI 501).

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRGCHIYING: u32 = 2_375_612_422;
const SOURCE: &str = include_str!("../../../../content/original/ported/jsrg/jsrgchiying.skill");

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

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == JSRGCHIYING)
        .unwrap()
}

#[test]
fn ordered_forced_discards_then_target_gains_only_basic_cards() {
    let package = content_compiler::compile_source_named("jsrgchiying-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 501);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 5, 20_260_802, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, first, target, second) = (seats[0], seats[1], seats[2], seats[3]);
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
        skill_def: SkillDefId::new(JSRGCHIYING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    let basic = inject(&mut tx, first, CardKind::Sha, Suit::Spade, 99_100);
    let trick = inject(&mut tx, second, CardKind::TrickGuohe, Suit::Heart, 99_101);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_id(tx.state(), owner);

    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![],
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.expect("first forced discard");
    assert_eq!(choice.observer, first);
    let (state, resolved) = submit(
        state,
        first,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![basic.raw()],
            cancelled: false,
        },
        2,
    );
    let choice = resolved.choice.expect("second forced discard");
    assert_eq!(choice.observer, second);
    let (state, resolved) = submit(
        state,
        second,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![trick.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(target).contains(&basic));
    assert!(!state.hand_cards(target).contains(&trick));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&trick));
}
