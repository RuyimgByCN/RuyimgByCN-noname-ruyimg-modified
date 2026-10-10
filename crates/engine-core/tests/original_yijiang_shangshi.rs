//! Golden for 一将成名「伤逝」的相关变化过滤与补至损失体力值。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHANGSHI: u32 = 3_386_412_611;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_000_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

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

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.character_index.iter().any(|character| {
        character.name_key == "zhangchunhua" && character.skills == [3_481_582_878, SHANGSHI]
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_717),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_717,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, std::mem::take(&mut effects)).unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, other);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHANGSHI),
        }],
    )
    .unwrap();
    (tx, owner, other)
}

fn confirm(
    state: engine_core::state::GameState,
    owner: PlayerId,
    key: u64,
) -> engine_core::state::GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn owner_gain_refills_to_lost_hp_but_unrelated_moves_do_not_trigger() {
    let (mut tx, owner, other) = setup();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 4;
    owner_state.hp = 1;
    let other_card = inject(&mut tx, other, ZoneKind::Hand, 1);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![other_card],
                to: owner_hand,
                from: Some(other_hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = confirm(tx.commit().state, owner, 1);
    assert_eq!(state.hand_count(owner), 3);

    let mut unrelated = Transaction::begin(state);
    let a = inject(&mut unrelated, other, ZoneKind::Hand, 2);
    let discard = unrelated
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(run_effects(
        &mut unrelated,
        vec![EffectRequest::MoveCards {
            cards: vec![a],
            to: discard,
            from: Some(other_hand),
        }],
    )
    .is_ok());
    assert!(!unrelated.state().has_pending_choice());
}

#[test]
fn damage_refills_using_post_damage_lost_hp() {
    let (mut tx, owner, other) = setup();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 4;
    owner_state.hp = 3;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = confirm(tx.commit().state, owner, 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert_eq!(state.hand_count(owner), 2);
}
