//! Golden coverage for TW 夏侯尚「探锋」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWTANFENG: u32 = 4_187_893_298;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twtanfeng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_368_001),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn arbitrary_card_sha_and_cancelled_fire_branch_both_resolve() {
    let package = content_compiler::compile_source_named("twtanfeng-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 368);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| { character.id == 3_609_594_146 && character.skills == [TWTANFENG] }));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_108, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWTANFENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let discarded = inject(&mut tx, target, ZoneKind::Judge, CardKind::TrickLe);
    let converted = inject(&mut tx, target, ZoneKind::Hand, CardKind::Tao);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWTANFENG)
        .unwrap()
        .id;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwtanfeng {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                stage: 0,
                phase: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve(state, owner, vec![target.raw()], false, 1);
    state = resolve(state, owner, vec![discarded.raw()], false, 2);
    state = resolve(state, target, vec![converted.raw()], false, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);

    let mut tx = Transaction::begin(state);
    let discarded = inject(&mut tx, target, ZoneKind::Judge, CardKind::TrickLe);
    let _available_sha_source = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwtanfeng {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                stage: 0,
                phase: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve(state, owner, vec![target.raw()], false, 4);
    state = resolve(state, owner, vec![discarded.raw()], false, 5);
    state = resolve(state, target, Vec::new(), true, 6);
    state = resolve(state, target, vec![Phase::Draw.to_u8() as u32], false, 7);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(state.turn.skip_draw());
}
