//! Golden coverage for TW 胡车儿「盗戟」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWDAOJI: u32 = 3_856_876_988;
const TW_HUCHEER: u32 = 4_271_674_660;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twdaoji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("twdaoji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(TWDAOJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWDAOJI)
        .unwrap()
        .id;
    Setup {
        tx,
        owner,
        target,
        skill,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone: ZoneKind, kind: CardKind) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_355_000 + kind.tag() as u32),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: zone_id,
        owner,
        face_up: zone == ZoneKind::Equip,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn resolve_choice(state: GameState, owner: PlayerId, card: CardId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn start(mut setup: Setup, kind: CardKind, zone: ZoneKind, key: u64) -> (GameState, CardId) {
    let card = inject(&mut setup.tx, setup.target, zone, kind);
    let result = run_effects(
        &mut setup.tx,
        vec![EffectRequest::ResolveDaoji {
            player: setup.owner,
            target: setup.target,
            card: CardId::NONE,
            stage: 0,
            tw: true,
            skill: setup.skill,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = setup.tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveDaoji
    );
    (resolve_choice(state, setup.owner, card, key), card)
}

#[test]
fn package_and_tw_category_rewards_are_exact() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 355);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == TW_HUCHEER)
            .unwrap()
            .skills,
        [4_258_102_225, TWDAOJI]
    );
    let targets = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == TWDAOJI)
        .unwrap()
        .targets
        .as_ref()
        .unwrap();
    assert!(targets.has_gainable_hand_equip && targets.in_attack_range);

    let basic = setup(2_026_080_101);
    let owner = basic.owner;
    let before = basic.tx.state().hand_count(owner);
    let (state, card) = start(basic, CardKind::Sha, ZoneKind::Hand, 1);
    assert_eq!(state.cards.get(card).unwrap().owner, owner);
    assert_eq!(state.hand_count(owner), before + 2);

    let equipment = setup(2_026_080_102);
    let owner = equipment.owner;
    let target = equipment.target;
    let hp = equipment.tx.state().players.get(target).unwrap().hp;
    let (state, card) = start(equipment, CardKind::Armor, ZoneKind::Equip, 2);
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, equip);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}
