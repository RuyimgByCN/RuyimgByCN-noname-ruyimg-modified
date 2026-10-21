//! Golden for XIANDING 崔琰毛玠【征辟/奉迎】（ABI 585）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CUIMAO: u32 = 2_841_277_757;
const ZHENGBI: u32 = 2_512_758_877;
const FENGYING: u32 = 3_395_124_491;
const FENGYING_EFFECT: u32 = 597_375_372;
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
    let (owner, target, third) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target, third)
}

fn inject(state: &mut GameState, player: PlayerId, zone: ZoneKind, kind: CardKind) -> CardId {
    let zone = state.zones.find(zone, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_585),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: player,
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

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

#[test]
fn package_closes_cuimao_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CUIMAO)
        .unwrap();
    assert_eq!(character.name_key, "cuimao");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [ZHENGBI, FENGYING]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENGYING_EFFECT));
}

#[test]
fn zhengbi_gains_one_hand_and_one_equip_after_target_gained() {
    let (mut state, owner, target, third) = setup(ZHENGBI);
    let target_hand = inject(&mut state, target, ZoneKind::Hand, CardKind::Tao);
    let target_equip = inject(&mut state, target, ZoneKind::Equip, CardKind::Weapon);
    let gained = inject(&mut state, third, ZoneKind::Hand, CardKind::Sha);
    let skill = skill_id(&state, owner, ZHENGBI);
    state = run(
        state,
        EffectRequest::ResolveZhengbi {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 1);
    state = choose(state, owner, vec![0], 2);
    let from = state.cards.get(gained).unwrap().zone;
    let to = state.zones.find(ZoneKind::Hand, target).unwrap();
    state = run(
        state,
        EffectRequest::MoveCards {
            cards: vec![gained],
            to,
            from: Some(from),
        },
    );
    state = run(
        state,
        EffectRequest::ResolveZhengbi {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 1,
            skill,
        },
    );
    state = choose(state, owner, vec![target_hand.raw(), target_equip.raw()], 3);
    assert_eq!(state.cards.get(target_hand).unwrap().owner, owner);
    assert_eq!(state.cards.get(target_equip).unwrap().owner, owner);
}

#[test]
fn zhengbi_basic_gift_receives_non_basic_payment() {
    let (mut state, owner, target, _third) = setup(ZHENGBI);
    let gift = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha);
    let payment = inject(&mut state, target, ZoneKind::Equip, CardKind::Weapon);
    let skill = skill_id(&state, owner, ZHENGBI);
    state = run(
        state,
        EffectRequest::ResolveZhengbi {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 1);
    state = choose(state, owner, vec![1], 2);
    state = choose(state, owner, vec![gift.raw()], 3);
    state = choose(state, target, vec![0], 4);
    state = choose(state, target, vec![payment.raw()], 5);
    assert_eq!(state.cards.get(gift).unwrap().owner, target);
    assert_eq!(state.cards.get(payment).unwrap().owner, owner);
}

#[test]
fn fengying_discards_hand_and_refills_only_its_extra_turn() {
    let (mut state, owner, target, third) = setup(FENGYING);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Tao);
    state.players.get_mut(owner).unwrap().hp = 1;
    state.players.get_mut(owner).unwrap().max_hp = 3;
    state.players.get_mut(target).unwrap().hp = 2;
    state.players.get_mut(third).unwrap().hp = 3;
    let skill = skill_id(&state, owner, FENGYING);
    state = run(
        state,
        EffectRequest::ResolveFengying {
            player: owner,
            helper_def: SkillDefId::new(FENGYING_EFFECT),
            action: 0,
            skill,
        },
    );
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.turn.extra_turn_queue, [owner]);
    let helper = skill_id(&state, owner, FENGYING_EFFECT);
    state = run(
        state,
        EffectRequest::ResolveFengying {
            player: owner,
            helper_def: SkillDefId::new(FENGYING_EFFECT),
            action: 2,
            skill: helper,
        },
    );
    assert_eq!(state.hand_count(owner), 3);
    assert!(state.skills.get(helper).is_none());
}
