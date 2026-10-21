//! Golden for 四象标文鸯「却敌」的实体【杀】视为【决斗】语义。

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

const STD_DB_WENYANG: u32 = 2_396_957_062;
const STDQUEDI: u32 = 3_311_025_145;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

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
        def: CardDefId::new(97_000_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_716, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    for player in [owner, target] {
        clear_hand(&mut tx, player);
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDQUEDI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDQUEDI)
        .unwrap()
        .id;
    (tx.state().clone(), owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn package_contains_complete_std_db_wenyang_and_view_as_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_DB_WENYANG)
        .expect("committed 标文鸯 character");
    assert_eq!(character.name_key, "std_db_wenyang");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.initial_hp, 0, "omitted means start at max_hp");
    assert_eq!(character.skills, [STDQUEDI]);

    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == STDQUEDI)
        .expect("却敌 view_as entry");
    assert_eq!(view.as_kind, "juedou");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "card_sha");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(view.uses_per_turn, None);
}

#[test]
fn stdquedi_rejects_non_sha_and_uses_hand_or_equipment_sha_as_duel() {
    let (state, owner, target, skill) = setup();
    let mut tx = Transaction::begin(state);
    let non_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        1,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, target, skill, non_sha, 1);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut tx = Transaction::begin(same);
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        2,
    );
    let hand_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        3,
    );
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (state, result) = activate(tx.state().clone(), owner, target, skill, hand_sha, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.cards.get(shan).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(hand_sha).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    let equip_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Sha,
        Suit::Club,
        4,
    );
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (state, result) = activate(tx.state().clone(), owner, target, skill, equip_sha, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.cards.get(equip_sha).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let late_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        5,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (same, result) = activate(state, owner, target, skill, late_sha, 4);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
