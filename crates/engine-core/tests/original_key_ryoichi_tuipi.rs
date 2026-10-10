//! Golden for KEY「褪皮」的锦囊目标免疫与弃牌阶段装备计数语义。

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const RYOICHI_TUIPI: u32 = 3_426_318_690;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let tuipi: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == RYOICHI_TUIPI)
        .collect();
    assert_eq!(tuipi.len(), 3);
    assert!(tuipi.iter().any(|modifier| {
        modifier.target == "card_target_cancel" && modifier.selector.as_deref() == Some("shunshou")
    }));
    assert!(tuipi.iter().any(|modifier| {
        modifier.target == "card_target_cancel" && modifier.selector.as_deref() == Some("guohe")
    }));
    assert!(tuipi
        .iter()
        .any(|modifier| modifier.target == "discard_phase_includes_equip"));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(RYOICHI_TUIPI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: RYOICHI_TUIPI as u64,
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
        skill_def: SkillDefId::new(RYOICHI_TUIPI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1])
}

fn play_trick_at_owner(base: &Transaction, owner: PlayerId, actor: PlayerId, kind: CardKind) {
    let mut state = base.state().clone();
    let card = state.hand_cards(actor)[0];
    state.cards.get_mut(card).unwrap().kind = kind;
    state.turn.current = actor;
    state.turn.phase = Phase::Play;
    let (after, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(kind as u64),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(
        after, state,
        "rejected target declaration must not mutate state"
    );
}

#[test]
fn shunshou_and_guohe_cannot_target_the_owner() {
    let (tx, owner, actor) = isolated();
    play_trick_at_owner(&tx, owner, actor, CardKind::TrickShunshou);
    play_trick_at_owner(&tx, owner, actor, CardKind::TrickGuohe);
}

#[test]
fn discard_phase_counts_equipment_and_accepts_it_as_the_mandatory_discard() {
    let (mut tx, owner, _) = isolated();
    let hand_count = tx.state().hand_cards(owner).len() as u32;
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = hand_count as i32;
        player.max_hp = hand_count;
    }
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(90_001),
        kind: CardKind::Armor,
        suit: Suit::Club,
        rank: 2,
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
        .push(equipment);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Discard;

    assert!(engine_core::query::discard_phase_includes_equip(
        tx.state(),
        owner
    ));
    assert_eq!(
        engine_core::query::hand_count_for_limit(tx.state(), owner),
        hand_count + 1
    );

    let (pending, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.min_count, 1);
    assert_eq!(choice.max_count, 1);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));

    let (resolved, result) = apply_command(
        pending,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![equipment.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.turn.phase, Phase::End);
    assert_eq!(
        resolved.cards.get(equipment).unwrap().zone,
        resolved
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
