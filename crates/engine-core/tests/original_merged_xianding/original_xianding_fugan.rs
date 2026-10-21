//! Golden for XIANDING 傅干【巧谏/析察】（ABI 586）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FUGAN: u32 = 398_847_554;
const QIAOJIAN: u32 = 105_744_963;
const QIAOJIAN_EFFECT: u32 = 9_694_479;
const XICHA: u32 = 2_494_439_810;
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

fn inject(state: &mut GameState, player: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_586),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_closes_fugan_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == FUGAN)
        .unwrap();
    assert_eq!(character.name_key, "fugan");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [QIAOJIAN, XICHA]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QIAOJIAN_EFFECT));
}

#[test]
fn qiaojian_lower_sum_gains_both_cards_and_exempts_them_from_hand_limit() {
    let (mut state, owner, target, _) = setup(QIAOJIAN);
    let owner_card = inject(&mut state, owner, CardKind::Sha, 2);
    let target_card = inject(&mut state, target, CardKind::Tao, 9);
    let skill = skill_id(&state, owner, QIAOJIAN);
    state = run(
        state,
        EffectRequest::ResolveQiaojian {
            player: owner,
            target,
            helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
            action: 0,
            cards: Vec::new(),
            split: 0,
            skill,
        },
    );
    state = choose(state, owner, vec![owner_card.raw()], 1);
    state = choose(state, target, vec![target_card.raw()], 2);
    assert_eq!(state.cards.get(owner_card).unwrap().owner, owner);
    assert_eq!(state.cards.get(target_card).unwrap().owner, owner);
    assert!(engine_core::query::card_ignored_for_hand_limit(
        &state, owner, owner_card
    ));
    assert!(engine_core::query::card_ignored_for_hand_limit(
        &state,
        owner,
        target_card
    ));
}

#[test]
fn qiaojian_next_turn_policies_merge_and_ban_only_damage_cards() {
    let (mut state, owner, target, _) = setup(QIAOJIAN);
    let main = skill_id(&state, owner, QIAOJIAN);
    for branch in [1, 2] {
        state = run(
            state,
            EffectRequest::ResolveQiaojian {
                player: owner,
                target,
                helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
                action: 10,
                cards: Vec::new(),
                split: branch,
                skill: main,
            },
        );
    }
    let helper = skill_id(&state, target, QIAOJIAN_EFFECT);
    state = run(
        state,
        EffectRequest::ResolveQiaojian {
            player: target,
            target,
            helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
            action: 5,
            cards: Vec::new(),
            split: 0,
            skill: helper,
        },
    );
    let sha = inject(&mut state, target, CardKind::Sha, 7);
    let wuxie = inject(&mut state, target, CardKind::TrickWuxie, 8);
    assert!(engine_core::query::card_use_cancelled_for_card(
        &state, target, sha
    ));
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &state, target, wuxie
    ));
    assert_eq!(state.skills.get(helper).unwrap().state_blob[4], 3);
    state = run(
        state,
        EffectRequest::ResolveQiaojian {
            player: target,
            target,
            helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
            action: 6,
            cards: vec![sha],
            split: 0,
            skill: helper,
        },
    );
    assert!(state.has_pending_choice(), "合并策略仍可询问杀的额外结算");
}

#[test]
fn xicha_matching_equipment_use_is_cancelled_consumed_and_gained() {
    let (mut state, owner, source, _) = setup(XICHA);
    let watched = inject(&mut state, source, CardKind::Weapon, 5);
    let skill = skill_id(&state, owner, XICHA);
    state = run(
        state,
        EffectRequest::ResolveXicha {
            player: owner,
            source,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![watched.raw()], 3);
    state
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::EmitRuleEvent {
            code: 50,
            payload: vec![source.raw(), watched.raw()],
        });
    state = run(
        state,
        EffectRequest::ResolveXicha {
            player: owner,
            source,
            card: watched,
            action: 1,
            cards: vec![watched],
            skill,
        },
    );
    assert!(state.resolution.deferred_tail_effects.is_empty());
    assert_eq!(state.cards.get(watched).unwrap().owner, owner);
    assert_eq!(&state.skills.get(skill).unwrap().state_blob, b"XCH1");
}
