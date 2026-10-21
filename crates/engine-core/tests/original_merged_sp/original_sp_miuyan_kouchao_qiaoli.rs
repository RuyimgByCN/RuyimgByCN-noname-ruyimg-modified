//! Golden for the sp view_as rewrite batch:
//!   - 鸣雁 olmiuyan: black hand/equip card as 【火攻】.
//!   - 酷嘲 olkouchao: a hand/equip card as 【杀】/【火攻】/【过河拆桥】, once each per turn.
//!   - 巧力 qiaoli: an equipment card as 【决斗】, up to twice per turn.

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

const OLMIUYAN: u32 = 3_010_058_014;
const OLKOUCHAO: u32 = 2_986_388_793;
const QIAOLI: u32 = 3_514_459_394;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

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
            rng_seed: 20_261_201,
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

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == def)
        .unwrap()
        .id
}

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
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
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
) -> engine_api::CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets: cards,
        },
    }
}

/// Clears any pending response choices (无懈 / 决斗出杀 / 火攻选牌) by declining
/// them, or by picking the first option when decline is not allowed.
fn clear_choices(
    state: &engine_core::state::GameState,
    mut key: u64,
) -> (engine_core::state::GameState, u64) {
    use engine_core::state::GameState;
    let mut state = state.clone();
    while state.has_pending_choice() {
        let observer = PlayerId::new(state.resolution.pending_observer);
        let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
        let (next, pipe) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        key += 1;
        if matches!(pipe.result, CommandResult::Rejected { .. }) {
            // Decline not allowed → accept the first option.
            let next_observer = PlayerId::new(next.resolution.pending_observer);
            let fallback_id = engine_api::ChoiceId::new(next.resolution.pending_choice_id);
            let (next2, pipe2) = apply_command(
                next,
                Command {
                    actor: next_observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: fallback_id,
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            key += 1;
            assert!(
                matches!(pipe2.result, CommandResult::Accepted { .. }),
                "fallback resolve must accept"
            );
            state = next2;
        } else {
            state = next;
        }
    }
    (state, key)
}

#[test]
fn package_declares_the_view_as_entries() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let miuyan = pkg
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == OLMIUYAN)
        .unwrap();
    assert_eq!(miuyan.as_kind, "huogong");
    assert_eq!(miuyan.filter, "black");
    assert_eq!(miuyan.source_zone.as_deref(), Some("hand_equip"));
    for kind in ["sha", "huogong", "guohe"] {
        assert!(pkg.view_as_index.iter().any(|v| {
            v.skill_def_id == OLKOUCHAO && v.as_kind == kind && v.uses_per_turn == Some(1)
        }));
    }
    let qiaoli = pkg
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == QIAOLI)
        .unwrap();
    assert_eq!(qiaoli.as_kind, "juedou");
    assert_eq!(qiaoli.filter, "card_equip");
    assert_eq!(qiaoli.uses_per_turn, Some(2));
}

#[test]
fn miuyan_accepts_black_cards_as_huogong_and_rejects_red() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, OLMIUYAN);
    let black = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, Suit::Spade);
    let red = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Tao, Suit::Heart);

    let (_, black_pipe) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![black], 1),
    );
    assert!(
        matches!(black_pipe.result, CommandResult::Accepted { .. }),
        "black hand card usable as 火攻"
    );

    let (_, red_pipe) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![red], 2),
    );
    assert!(
        matches!(red_pipe.result, CommandResult::Rejected { .. }),
        "red hand card rejected"
    );
}

#[test]
fn kouchao_offers_all_three_convertibles() {
    // Each convertible kind works in a fresh game (sha / huogong / guohe).
    for (kind, key) in [("sha", 1u64), ("huogong", 2), ("guohe", 3)] {
        let (engine, owner, other) = start();
        let mut tx = Transaction::begin(engine.state().clone());
        strip_skills(&mut tx, owner);
        clear_hand(&mut tx, owner);
        tx.state_mut().turn.phase = Phase::Play;
        let skill = add_skill(&mut tx, owner, OLKOUCHAO);
        let card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, Suit::Spade);
        let (_, pipe) = apply_command(
            tx.state().clone(),
            activate(owner, skill, vec![other], vec![card], key),
        );
        assert!(
            matches!(pipe.result, CommandResult::Accepted { .. }),
            "{kind} conversion accepted"
        );
    }
}

#[test]
fn qiaoli_accepts_equipment_twice_then_rejects() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, owner);
    clear_hand(&mut tx, owner);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, owner, QIAOLI);
    let equip1 = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Heart,
    );
    let equip2 = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Diamond,
    );
    let equip3 = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::HorseOff,
        Suit::Club,
    );

    let mut key = 1u64;
    for card in [equip1, equip2] {
        let (state, pipe) = apply_command(
            tx.state().clone(),
            activate(owner, skill, vec![other], vec![card], key),
        );
        assert!(
            matches!(pipe.result, CommandResult::Accepted { .. }),
            "equipment convertible to 决斗 (use {key})"
        );
        let (state, next_key) = clear_choices(&state, key + 1);
        key = next_key;
        tx = Transaction::begin(state);
    }

    let (_, third) = apply_command(
        tx.state().clone(),
        activate(owner, skill, vec![other], vec![equip3], key),
    );
    assert!(
        matches!(third.result, CommandResult::Rejected { .. }),
        "third equipment conversion rejected (限两次)"
    );
}
