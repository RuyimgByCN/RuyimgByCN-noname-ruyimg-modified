//! Golden for XIANDING 柏灵筠【灵慧/黠策/御心】（ABI 593）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BAILINGYUN: u32 = 4_278_690_663;
const DCLINGHUI: u32 = 2_581_852_991;
const DCXIACE: u32 = 2_254_753_634;
const DCYUXIN: u32 = 4_174_094_056;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, [PlayerId; 3]) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
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
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
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
    let (state, result) = apply_command(
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
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn package_closes_bailingyun_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 593);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == BAILINGYUN)
        .unwrap();
    assert_eq!(character.name_key, "bailingyun");
    assert_eq!(character.skills, [DCLINGHUI, DCXIACE, DCYUXIN]);
}

#[test]
fn linghui_uses_one_gains_one_and_returns_the_rest_in_order() {
    let (mut state, [owner, _other, _]) = setup(DCLINGHUI);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        10,
        CardKind::Tao,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        11,
        CardKind::Tao,
    );
    let third = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        12,
        CardKind::Shan,
    );
    let second = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        13,
        CardKind::Shan,
    );
    let usable = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        14,
        CardKind::TrickWuzhong,
    );
    let skill = skill_id(&state, owner, DCLINGHUI);
    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![usable.raw()], 1);

    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let gained = [second, third]
        .into_iter()
        .filter(|card| state.cards.get(*card).unwrap().zone == owner_hand)
        .collect::<Vec<_>>();
    assert_eq!(gained.len(), 1);
    let returned = if gained[0] == second { third } else { second };
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.zones.get(draw).unwrap().cards.last(), Some(&returned));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(usable).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );

    let (mut gated, [owner, other, _]) = setup(DCLINGHUI);
    gated.turn.current = other;
    let skill = skill_id(&gated, owner, DCLINGHUI);
    let untouched = run(
        gated.clone(),
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    assert!(!untouched.has_pending_choice());
    gated.turn.record_entered_dying(other);
    inject(
        &mut gated,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        20,
        CardKind::TrickWuzhong,
    );
    let offered = run(
        gated,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    assert!(offered.has_pending_choice());
}

#[test]
fn xiace_two_branches_are_independently_once_per_turn() {
    let (mut state, [owner, target, _]) = setup(DCXIACE);
    state.turn.phase = Phase::Play;
    state.players.get_mut(owner).unwrap().hp = 2;
    let cost = inject(&mut state, owner, ZoneKind::Hand, 30, CardKind::Shan);
    let extra = inject(&mut state, owner, ZoneKind::Hand, 31, CardKind::Shan);
    let skill = skill_id(&state, owner, DCXIACE);

    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 10,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 2);
    assert!(state
        .turn
        .non_locked_skill_suppressed_players
        .contains(&target));

    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 11,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![cost.raw()], 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.turn.flag(turn_flag::xiace_used_by_player(owner)), 3);

    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 11,
            cards: Vec::new(),
            skill,
        },
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(extra).unwrap().owner, owner);
}
