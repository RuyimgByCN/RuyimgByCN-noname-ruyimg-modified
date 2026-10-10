//! Golden coverage for TW 渐营 / 武神 / 振良 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIANYING: u32 = 3_378_394_717;
const TWWUSHEN: u32 = 2_977_352_429;
const TWZHENLIANG: u32 = 3_711_733_177;
const JIANYING_SRC: &str =
    include_str!("../../../content/original/ported/tw/twjianying.skill");
const WUSHEN_SRC: &str = include_str!("../../../content/original/ported/tw/twwushen.skill");
const ZHENLIANG_SRC: &str =
    include_str!("../../../content/original/ported/tw/twzhenliang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn twjianying_source_and_npack_publish_last_used_suit_or_rank() {
    let compiled = content_compiler::compile_source_named("twjianying", JIANYING_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TWJIANYING)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_source_matches_last_used_suit_or_rank");
    assert!(trigger.optional);

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == TWJIANYING
            && trigger.filter == "owner_source_matches_last_used_suit_or_rank"));
}

#[test]
fn twjianying_offers_draw_when_rank_matches_previous_used_card() {
    let (mut tx, owner, _) = bootstrap(JIANYING_SRC, "twjianying-test", TWJIANYING);
    let first = inject_hand(&mut tx, owner, Suit::Spade, 7, CardKind::Other);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: first,
            targets: vec![],
        }],
    )
    .unwrap();
    let initial = tx.state().hand_count(owner);
    let second = inject_hand(&mut tx, owner, Suit::Heart, 7, CardKind::Other);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card: second,
                targets: vec![],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), initial + 1);
}

#[test]
fn twwushen_source_and_npack_rewrite_hearts_to_unlimited_sha() {
    let compiled = content_compiler::compile_source_named("twwushen", WUSHEN_SRC).unwrap();
    let mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|item| item.skill_def_id == TWWUSHEN)
        .map(|item| item.target.as_str())
        .collect();
    assert!(mods.contains(&"card_name"));
    assert!(mods.contains(&"card_distance_unlimited"));
    assert!(mods.contains(&"card_use_unlimited"));
    assert!(mods.contains(&"sha_unrespondable"));

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .modifier_index
        .iter()
        .any(|item| item.skill_def_id == TWWUSHEN && item.target == "card_name"));
}

#[test]
fn twwushen_heart_tao_is_effective_sha() {
    let (mut tx, owner, _) = bootstrap(WUSHEN_SRC, "twwushen-test", TWWUSHEN);
    let heart = inject_hand(&mut tx, owner, Suit::Heart, 3, CardKind::Tao);
    let physical = tx.state().cards.get(heart).unwrap();
    assert_eq!(physical.kind, CardKind::Tao);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        heart
    ));
    assert!(engine_core::query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        heart
    ));
}

#[test]
fn twzhenliang_source_and_npack_publish_active_damage_and_reduce() {
    let compiled = content_compiler::compile_source_named("twzhenliang", ZHENLIANG_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == TWZHENLIANG));
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TWZHENLIANG
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target_other_source_alive"
            && trigger.optional
    }));

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == TWZHENLIANG));
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TWZHENLIANG && trigger.event == "damage_before"
    }));
}

#[test]
fn twzhenliang_can_cancel_incoming_damage_of_one() {
    let (tx, owner, other) = bootstrap(ZHENLIANG_SRC, "twzhenliang-test", TWZHENLIANG);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let mut tx = tx;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}
