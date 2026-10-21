//! Golden coverage for TW 桥蕤「狭威/穷技」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWXIAWEI: u32 = 3_007_668_730;
const TWQIONGJI: u32 = 2_723_606_029;
const WANGXING: u32 = 3_476_660_685;
const TWQIONGJI_DRAW: u32 = 3_469_371_140;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twxiawei.skill");

fn inject_draw(tx: &mut Transaction, kind: CardKind, rank: u8) -> CardId {
    let zone = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_375_000 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn wei_cards_are_playable_draw_once_and_empty_pile_increases_damage() {
    let package = content_compiler::compile_source_named("twxiawei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 375);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_116, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
    for skill_def in [TWXIAWEI, TWQIONGJI, TWQIONGJI_DRAW] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let xiawei = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWXIAWEI)
        .unwrap()
        .id;
    let sha = inject_draw(&mut tx, CardKind::Sha, 1);
    let spare = inject_draw(&mut tx, CardKind::ShaFire, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 0,
            stage: 0,
            skill: xiawei,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, TWXIAWEI), [spare, sha]);
    assert!(tx.state().playable_hand_cards(owner).contains(&sha));
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = tx.state().hand_count(owner);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let mut state = tx.commit().state;

    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 1,
            stage: 0,
            skill: xiawei,
        }],
    )
    .unwrap();
    assert!(tx.state().skill_pile_cards(owner, TWXIAWEI).is_empty());
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: target,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp - 2);

    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 2,
            stage: 2,
            skill: xiawei,
        }],
    )
    .unwrap();
    let wangxing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == WANGXING)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 4,
            stage: 0,
            skill: wangxing,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp - 1);
    assert!(tx.state().skills.get(wangxing).is_none());
}
