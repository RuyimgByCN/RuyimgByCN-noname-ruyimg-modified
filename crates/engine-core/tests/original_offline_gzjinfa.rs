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
