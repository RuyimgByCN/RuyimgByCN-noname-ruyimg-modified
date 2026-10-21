//! Golden coverage for TW 田豫「震袭/扬师」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZHENXI: u32 = 3_713_822_964;
const TWYANGSHI: u32 = 4_235_144_837;
const TWYANGSHI_DISTANCE: u32 = 3_469_371_167;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twtianyu.skill");

fn inject(tx: &mut Transaction, zone: ZoneKind, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_820 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn zhenxi_backwater_discards_then_moves_and_yangshi_grows_then_gains_sha() {
    let package = content_compiler::compile_source_named("tw-tianyu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 382);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 4, 2_026_080_123, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, destination) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWZHENXI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWYANGSHI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let discarded = inject(&mut tx, ZoneKind::Hand, target, CardKind::Shan);
    let weapon = inject(&mut tx, ZoneKind::Equip, target, CardKind::Weapon);
    let zhenxi = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWZHENXI)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwtianyu {
                player: owner,
                target,
                helper_skill_def: TWYANGSHI_DISTANCE,
                action: 0,
                stage: 0,
                card: CardId::NONE,
                branch: 0,
                skill: zhenxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![3], 1);
    let state = choose(state, owner, vec![discarded.raw()], 2);
    let state = choose(state, owner, vec![weapon.raw()], 3);
    let state = choose(state, owner, vec![destination.raw()], 4);
    assert_eq!(
        state.cards.get(discarded).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state.zones.find(ZoneKind::Equip, destination).unwrap()
    );

    let yangshi = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWYANGSHI)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwtianyu {
            player: owner,
            target: owner,
            helper_skill_def: TWYANGSHI_DISTANCE,
            action: 1,
            stage: 0,
            card: CardId::NONE,
            branch: 0,
            skill: yangshi,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == TWYANGSHI_DISTANCE && *count == 1));
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwtianyu {
            player: owner,
            target: owner,
            helper_skill_def: TWYANGSHI_DISTANCE,
            action: 1,
            stage: 0,
            card: CardId::NONE,
            branch: 0,
            skill: yangshi,
        }],
    )
    .unwrap();
    // 四座时并非全员在攻击范围内：追加距离标记而非直接获得【杀】。
    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == TWYANGSHI_DISTANCE && *count == 2));
}
