//! Golden coverage for TW 夏侯恩「剑威」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIANWEI: u32 = 3_685_532_230;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twjianwei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_367_000 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: (kind == CardKind::Weapon).then_some(3).unwrap_or(0),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn attack_range_changes_winner_and_both_prepare_branches_settle_exactly() {
    let package = content_compiler::compile_source_named("twjianwei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 367);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_331_986_699 && character.skills == [2_940_701_334, TWJIANWEI]
    }));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_107, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(TWJIANWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let weapon = inject(&mut tx, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    let owner_card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 4);
    let other_card = inject(&mut tx, other, ZoneKind::Hand, CardKind::Shan, 6);
    let other_hand = inject(&mut tx, other, ZoneKind::Hand, CardKind::Tao, 2);
    let other_equip = inject(&mut tx, other, ZoneKind::Equip, CardKind::Armor, 3);
    let other_judge = inject(&mut tx, other, ZoneKind::Judge, CardKind::TrickLe, 5);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWJIANWEI)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwjianwei {
                player: owner,
                actor: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                winner: PlayerId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, owner, vec![other.raw()], 1);
    state = choose(state, owner, vec![owner_card.raw()], 2);
    state = choose(state, other, vec![other_card.raw()], 3);
    state = choose(
        state,
        owner,
        vec![other_hand.raw(), other_equip.raw(), other_judge.raw()],
        4,
    );
    assert!([other_hand, other_equip, other_judge]
        .iter()
        .all(|card| state.hand_cards(owner).contains(card)));

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Prepare;
    let challenger = inject(&mut tx, other, ZoneKind::Hand, CardKind::Sha, 13);
    let owner_low = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Shan, 1);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwjianwei {
                player: owner,
                actor: other,
                target: PlayerId::NONE,
                card: CardId::NONE,
                winner: PlayerId::NONE,
                stage: 1,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, other, vec![1], 5);
    state = choose(state, other, vec![challenger.raw()], 6);
    state = choose(state, owner, vec![owner_low.raw()], 7);
    assert!(state.hand_cards(other).contains(&weapon));
}
