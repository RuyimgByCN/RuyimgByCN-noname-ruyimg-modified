//! Golden for XIANDING 阮籍【昭文/酒遁】（ABI 584）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCZHAOWEN: u32 = 829_715_916;
const DCJIUDUN: u32 = 2_926_675_148;
const RUANJI: u32 = 2_897_563_211;
const MARK_JIU: u32 = 1;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64, skill_def: u32) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, third) = (players[0], players[1], players[2]);
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
    tx.state_mut().turn.round = 1;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, source, third)
}

fn inject_card(state: &mut GameState, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_584),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn command(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "command rejected: {:?}",
        result.result
    );
    state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn cancel_pending(mut state: GameState, actors: &[PlayerId], mut key: u64) -> GameState {
    for _ in 0..20 {
        if !state.has_pending_choice() {
            return state;
        }
        let mut accepted = None;
        for actor in actors {
            let (next, result) = apply_command(
                state.clone(),
                Command {
                    actor: *actor,
                    expected_revision: ExpectedRevision(state.revision),
                    idempotency_key: IdempotencyKey(key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                        selections: Vec::new(),
                        cancelled: true,
                    },
                },
            );
            key += 1;
            if matches!(result.result, CommandResult::Accepted { .. }) {
                accepted = Some(next);
                break;
            }
        }
        state = accepted.expect("pending choice has an expected observer");
    }
    panic!("pending choices did not drain")
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

fn jiu_mark(state: &GameState, player: PlayerId) -> u32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == MARK_JIU).then_some((*count).max(0) as u32))
        .unwrap_or(0)
}

#[test]
fn package_closes_ruanji_with_exact_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RUANJI)
        .expect("ruanji");
    assert_eq!(character.name_key, "ruanji");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [DCZHAOWEN, DCJIUDUN]);
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == DCZHAOWEN)
            .count(),
        2
    );
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == DCJIUDUN)
            .count(),
        2
    );
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DCZHAOWEN)
        .expect("昭文 view-as");
    assert_eq!(view.as_kind, "ordinary_trick");
    assert_eq!(view.cost.as_deref(), Some("dczhaowen"));
}

#[test]
fn zhaowen_tracks_current_hand_converts_black_once_per_name_and_rewards_red_use() {
    let (mut state, owner, _source, _third) = setup(20_260_810, DCZHAOWEN);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let black = inject_card(&mut state, owner, CardKind::Sha, Suit::Spade);
    let red = inject_card(&mut state, owner, CardKind::Weapon, Suit::Heart);
    let skill = skill_id(&state, owner, DCZHAOWEN);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveRuanji {
            player: owner,
            source: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;

    state = command(
        state,
        owner,
        CommandKind::UseSkillCard {
            skill,
            card_kind: CardKind::TrickWuzhong.tag(),
            targets: Vec::new(),
            equip_slot: 0,
        },
        1,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveFaceDownViewAs
    );
    state = choose(state, owner, vec![black.raw()], 2);
    state = cancel_pending(state, &[owner, _source, _third], 20);
    let revision = state.revision;
    let (_unchanged, duplicate) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: Vec::new(),
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));

    let before = state.hand_count(owner);
    state = command(
        state,
        owner,
        CommandKind::PlayCard {
            card: red,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        4,
    );
    assert_eq!(state.hand_count(owner), before, "红装备离手，昭文额外摸一");
}

#[test]
fn jiudun_rewrites_jiu_recovery_to_next_sha_boost() {
    let (mut state, owner, _source, _third) = setup(20_260_811, DCJIUDUN);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    state.players.get_mut(owner).unwrap().hp = 2;
    let jiu = inject_card(&mut state, owner, CardKind::Jiu, Suit::Diamond);
    state = command(
        state,
        owner,
        CommandKind::PlayCard {
            card: jiu,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert_eq!(jiu_mark(&state, owner), 1);
}

#[test]
fn jiudun_black_target_branches_draw_drink_or_discard_exclude() {
    let (mut state, owner, source, _third) = setup(20_260_812, DCJIUDUN);
    state.turn.current = source;
    state.turn.phase = Phase::Play;
    let sha = inject_card(&mut state, source, CardKind::Sha, Suit::Spade);
    let before = state.hand_count(owner);
    state = command(
        state,
        source,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    state = choose(state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(jiu_mark(&state, owner), 1);
    state = cancel_pending(state, &[owner, source], 40);

    let payment = inject_card(&mut state, owner, CardKind::Tao, Suit::Heart);
    let second_sha = inject_card(&mut state, source, CardKind::TrickGuohe, Suit::Club);
    let hp = state.players.get(owner).unwrap().hp;
    state = command(
        state,
        source,
        CommandKind::PlayCard {
            card: second_sha,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        3,
    );
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    state = choose(state, owner, vec![payment.raw()], 4);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(payment).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
}
