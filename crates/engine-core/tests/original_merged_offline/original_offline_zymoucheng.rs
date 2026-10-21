//! Golden for OFFLINE「谋逞」的装备来源、借刀流程与每回合次数限制。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZYMOUCHENG: u32 = 3_584_095_001;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

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
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: (kind == CardKind::Weapon).then_some(3).unwrap_or(0),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZYMOUCHENG),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZYMOUCHENG)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    holder: PlayerId,
    victim: PlayerId,
    source: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![holder, victim],
                card_targets: vec![source],
            },
        },
    )
}

#[test]
fn package_exposes_the_full_legacy_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == ZYMOUCHENG)
        .expect("谋逞 view_as entry");
    assert_eq!(view.as_kind, "jiedao");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "black");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(view.uses_per_turn, Some(1));
}

#[test]
fn black_equipment_can_be_used_as_jiedao_and_the_skill_cannot_repeat() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_715, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let players = tx.state().players.alive_by_seat();
    let (actor, holder, victim) = (tx.state().turn.current, players[1], players[2]);
    for player in players {
        clear_hand(&mut tx, player);
    }
    let skill = add_skill(&mut tx, actor);
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        7,
    );
    let second_source = inject(&mut tx, actor, ZoneKind::Hand, CardKind::Tao, Suit::Club, 8);
    inject(
        &mut tx,
        holder,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        9,
    );

    let (state, pipe) = activate(tx.state().clone(), actor, skill, holder, victim, source, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("借刀持刀者选择");
    assert_eq!(choice.observer, holder);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(source).unwrap().zone, discard);

    let (state, pipe) = apply_command(
        state,
        Command {
            actor: holder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));

    let mut tx = Transaction::begin(state);
    inject(
        &mut tx,
        holder,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Diamond,
        10,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, actor, skill, holder, victim, second_source, 3);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
