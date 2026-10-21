//! Golden for XIANDING 胡遵【斩涛/安境】（ABI 587）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HUZUN: u32 = 2_150_724_644;
const DCZHANTAO: u32 = 4_294_211_147;
const DCANJING: u32 = 2_716_085_756;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, third) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, source, third)
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, rank: u8) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_587),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn package_closes_huzun_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == HUZUN)
        .unwrap();
    assert_eq!(character.name_key, "huzun");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [DCZHANTAO, DCANJING]);
}

#[test]
fn dczhantao_compares_judgment_with_all_physical_materials() {
    let (mut state, owner, source, _) = setup(DCZHANTAO);
    let first = inject(&mut state, owner, ZoneKind::Hand, 2);
    let second = inject(&mut state, owner, ZoneKind::Hand, 3);
    inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, 6);
    let before = state.players.get(source).unwrap().hp;
    let skill = skill_id(&state, owner, DCZHANTAO);
    state = run(
        state,
        EffectRequest::ResolveHuzun {
            player: owner,
            source,
            card: first,
            action: 0,
            cards: vec![first, second],
            targets: Vec::new(),
            skill,
        },
    );
    assert_eq!(state.players.get(source).unwrap().hp, before - 1);
}

#[test]
fn dcanjing_second_activation_draws_two_and_recovers_one_lowest_hp_target() {
    let (mut state, owner, first, second) = setup(DCANJING);
    state.players.get_mut(first).unwrap().hp = 1;
    state.players.get_mut(second).unwrap().hp = 2;
    for rank in [7, 8] {
        inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, rank);
    }
    let skill = skill_id(&state, owner, DCANJING);
    state.skills.get_mut(skill).unwrap().state_blob =
        [b"ANJ1".as_slice(), &1u32.to_le_bytes()].concat();
    state = run(
        state,
        EffectRequest::ResolveHuzun {
            player: owner,
            source: PlayerId::NONE,
            card: CardId::NONE,
            action: 1,
            cards: Vec::new(),
            targets: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![first.raw(), second.raw()], 1);
    assert_eq!(state.hand_count(first), 1);
    assert_eq!(state.hand_count(second), 1);
    assert_eq!(state.players.get(first).unwrap().hp, 2);
    assert_eq!(state.players.get(second).unwrap().hp, 2);
    assert_eq!(
        state.skills.get(skill).unwrap().state_blob,
        [b"ANJ1".as_slice(), &2u32.to_le_bytes()].concat()
    );
}
