//! Golden coverage for BINGSHI 势程普「督作」「避憾」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTDUZUO: u32 = 3_906_103_687;
const POTBIHAN: u32 = 4_060_934_534;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(515),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 515,
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
        .find(|role| *role != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([POTDUZUO, POTBIHAN].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
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
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(515_000 + serial),
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

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
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

#[test]
fn package_publishes_complete_pot_chengpu_at_abi517() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_chengpu")
            .unwrap()
            .skills,
        [POTDUZUO, POTBIHAN]
    );
}

#[test]
fn duzuo_moves_the_topmost_fire_sha_to_the_chosen_role() {
    let (mut tx, owner, target) = setup();
    let fire_sha = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::ShaFire,
        1,
    );
    let duzuo = skill(tx.state(), owner, POTDUZUO);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchengpu {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                damage_id: 0,
                skill: duzuo,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], 1);
    assert!(state.hand_cards(target).contains(&fire_sha));
}

#[test]
fn bihan_discards_to_hp_then_reduces_the_bound_sha_damage() {
    let (mut tx, owner, target) = setup();
    clear_hand(&mut tx, target);
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let cards: Vec<_> = (10..14)
        .map(|serial| inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, serial))
        .collect();
    let bihan = skill(tx.state(), owner, POTBIHAN);
    tx.state_mut().resolution.current_damage_id = 515;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::CommitDamage {
            damage_id: 515,
            source: owner,
            target,
            amount: 2,
            base_amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        });
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchengpu {
                player: owner,
                target,
                stage: 2,
                damage_id: 515,
                skill: bihan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], 2);
    let state = choose(state, cards[..2].iter().map(|card| card.raw()).collect(), 3);
    assert_eq!(state.hand_count(target), 2);
    assert_eq!(state.players.get(target).unwrap().hp, 1);
}
