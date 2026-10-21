//! Golden coverage for BINGSHI 势陆郁生「润微」「霜怀」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBRUNWEI: u32 = 3_010_472_386;
const MBSHUANGHUAI: u32 = 4_109_631_346;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn inject(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(506_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn runwei_resets_after_qualified_loss_and_cleans_own_gain() {
    let (mut tx, owner, _) = setup(MBRUNWEI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill(tx.state(), owner, MBRUNWEI);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 1;
    let filler = inject(&mut tx, owner);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    let color: u32 = match tx.state().cards.get(top).unwrap().suit {
        Suit::Heart | Suit::Diamond => 1,
        Suit::Spade | Suit::Club => 2,
        Suit::None => 0,
    };
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbrunwei {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                value: 0,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![1], 1);
    let state = resolve(state, vec![color], 2);
    let state = resolve(state, vec![owner.raw()], 3);
    assert!(state.hand_cards(owner).contains(&top));
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![filler],
        }],
    )
    .unwrap();
    let blob = &tx.state().skills.get(skill).unwrap().state_blob;
    assert_eq!(u32::from_le_bytes(blob[8..12].try_into().unwrap()), 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbrunwei {
            player: owner,
            target: PlayerId::NONE,
            stage: 5,
            value: 0,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert!(!tx.state().hand_cards(owner).contains(&top));
}

#[test]
fn shuanghuai_prevents_the_bound_damage() {
    let (mut tx, owner, target) = setup(MBSHUANGHUAI);
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        schedule_damage(&mut tx, owner, target, 1),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![1], 4);
    let state = resolve(state, vec![1], 5);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let skill = skill(&state, owner, MBSHUANGHUAI);
    assert_eq!(&state.skills.get(skill).unwrap().state_blob[..4], b"SHH1");
}
