//! Golden coverage for 旧朱然「胆守」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDDANSHOU: u32 = 3_778_601_240;
const OLD_ZHURAN: u32 = 4_091_953_259;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/olddanshou.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("olddanshou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 286);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let current = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDDANSHOU),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;

    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_601),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    (tx, owner, target, current, card)
}

fn damage(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::RecoverHp {
            source: PlayerId::NONE,
            player: target,
            amount: 1,
        });
    assert_eq!(
        run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

fn resolve_optional(tx: Transaction, owner: PlayerId, accepted: bool, key: u64) -> GameState {
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: accepted.then_some(1).into_iter().collect(),
                cancelled: !accepted,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn olddanshou_aborts_the_resolution_and_phase_only_when_accepted() {
    let (mut declined, owner, target, current, processing_card) = setup(20_260_729_11);
    let hand = declined.state().hand_count(owner);
    let hp = declined.state().players.get(target).unwrap().hp;
    damage(&mut declined, owner, target);
    let declined = resolve_optional(declined, owner, false, 1);
    assert_eq!(declined.hand_count(owner), hand);
    assert_eq!(declined.players.get(target).unwrap().hp, hp);
    assert_eq!(declined.turn.current, current);
    assert_eq!(declined.turn.phase, Phase::Play);
    assert_eq!(
        declined.cards.get(processing_card).unwrap().zone,
        declined
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .unwrap()
    );

    let (mut accepted, owner, target, current, processing_card) = setup(20_260_729_12);
    let hand = accepted.state().hand_count(owner);
    let hp = accepted.state().players.get(target).unwrap().hp;
    damage(&mut accepted, owner, target);
    let accepted = resolve_optional(accepted, owner, true, 2);
    assert_eq!(accepted.hand_count(owner), hand + 1);
    assert_eq!(accepted.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(accepted.turn.current, current);
    assert_eq!(accepted.turn.phase, Phase::Discard);
    assert_eq!(accepted.turn.flag(turn_flag::SKIPPED_PHASE_COUNT), 0);
    assert_eq!(
        accepted.cards.get(processing_card).unwrap().zone,
        accepted
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn packages_publish_old_zhuran_with_exact_skill() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDDANSHOU && skill.name == "olddanshou"));
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(
        old.character_index
            .iter()
            .find(|character| character.id == OLD_ZHURAN)
            .unwrap()
            .skills,
        [OLDDANSHOU]
    );
}
