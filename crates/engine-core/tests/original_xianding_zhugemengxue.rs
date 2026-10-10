//! Golden for XIANDING 诸葛梦雪【寄春/寒英】（ABI 588）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHUGEMENGXUE: u32 = 2_758_962_825;
const DCJICHUN: u32 = 3_609_714_118;
const DCHANYING: u32 = 2_250_842_878;
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
    tx.state_mut().turn.phase = Phase::Play;
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
fn package_closes_zhugemengxue_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 588);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHUGEMENGXUE)
        .unwrap();
    assert_eq!(character.name_key, "zhugemengxue");
    assert_eq!(character.skills, [DCJICHUN, DCHANYING]);
}

#[test]
fn dcjichun_uses_each_branch_once_and_counts_localized_name() {
    let (mut state, [owner, low, high]) = setup(DCJICHUN);
    let selected = inject(&mut state, owner, ZoneKind::Hand, 1, CardKind::Sha);
    inject(&mut state, owner, ZoneKind::Hand, 2, CardKind::Shan);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        3,
        CardKind::Tao,
    );
    let skill = skill_id(&state, owner, DCJICHUN);
    state = run(
        state,
        EffectRequest::ResolveZhugemengxue {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: vec![selected],
            skill,
        },
    );
    state = choose(state, owner, vec![low.raw()], 1);
    assert_eq!(state.hand_count(low), 1);
    assert_eq!(state.hand_count(owner), 2);

    let four_chars = inject(&mut state, owner, ZoneKind::Hand, 4, CardKind::TrickWuzhong);
    let high_cards: Vec<_> = (0..4)
        .map(|index| inject(&mut state, high, ZoneKind::Hand, 10 + index, CardKind::Shan))
        .collect();
    state = run(
        state,
        EffectRequest::ResolveZhugemengxue {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: vec![four_chars],
            skill,
        },
    );
    state = choose(state, owner, vec![high.raw()], 2);
    state = choose(
        state,
        owner,
        high_cards.iter().map(|card| card.raw()).collect(),
        3,
    );
    assert_eq!(state.hand_count(high), 0);
    assert_eq!(state.skills.get(skill).unwrap().state_blob, b"JCH1\x03");
}

#[test]
fn dchanying_uses_draw_pile_equipment_without_a_fake_hand_gain() {
    let (mut state, [owner, target, _]) = setup(DCHANYING);
    let equipment = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        100,
        CardKind::Weapon,
    );
    let skill = skill_id(&state, owner, DCHANYING);
    state = run(
        state,
        EffectRequest::ResolveZhugemengxue {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 10,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 4);
    let card = state.cards.get(equipment).unwrap();
    assert_eq!(card.owner, target);
    assert_eq!(state.zones.get(card.zone).unwrap().kind, ZoneKind::Equip);
    assert_eq!(state.hand_count(target), 0);
}
