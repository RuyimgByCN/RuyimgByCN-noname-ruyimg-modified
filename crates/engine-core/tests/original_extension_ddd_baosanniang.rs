//! Golden for 3D精选 鲍三娘【枝连/赍剑】（ABI 629）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHILIAN: u32 = 3_056_536_406;
const JIJIAN: u32 = 887_278_272;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/baosanniang.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-baosanniang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 629);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    tx.state_mut()
        .players
        .get_mut(players[0])
        .unwrap()
        .character_def = 25;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = 1;
    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = 2;
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
    effects.extend([ZHILIAN, JIJIAN].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.entered_discard_cards.clear();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::DiscardPile) {
        PlayerId::NONE
    } else {
        owner
    };
    let zone = state.zones.find(zone_kind, zone_owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_629_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: if zone_owner.is_some() {
            owner
        } else {
            PlayerId::NONE
        },
        face_up: zone_owner.is_none(),
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
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
    )
}

fn arm_zhilian(state: &mut GameState, owner: PlayerId, male: PlayerId) {
    state.turn.phase = Phase::End;
    state.turn.current = male;
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(owner), 1);
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(male), 1);
}

#[test]
fn zhilian_draws_both_then_gains_all_turn_entered_sha_still_in_discard() {
    let (mut state, players) = setup();
    let (owner, male, third) = (players[0], players[1], players[2]);
    arm_zhilian(&mut state, owner, male);
    let sha = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::ShaThunder,
        Suit::Spade,
        1,
    );
    state.turn.entered_discard_cards.insert(sha);
    let owner_before = state.hand_count(owner);
    let male_before = state.hand_count(male);
    let skill = skill_id(&state, owner, ZHILIAN);
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(third), 1);
    state.players.get_mut(third).unwrap().alive = false;
    let mut tx = Transaction::begin(state);
    assert!(run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddBaosanniang {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            targets: Vec::new(),
            used_kind: 0,
            skill,
        }],
    )
    .unwrap()
    .is_empty());
    let mut state = tx.commit().state;
    state.players.get_mut(third).unwrap().alive = true;
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(third), 0);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddBaosanniang {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                used_kind: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![0], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_before + 2);
    assert_eq!(state.hand_count(male), male_before + 1);
    assert!(state.hand_cards(owner).contains(&sha));
}

#[test]
fn zhilian_discard_branch_recovers_the_sha_owner_just_discarded() {
    let (mut state, players) = setup();
    let (owner, male) = (players[0], players[1]);
    arm_zhilian(&mut state, owner, male);
    let owner_sha = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        10,
    );
    let male_card = inject(
        &mut state,
        male,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        11,
    );
    let skill = skill_id(&state, owner, ZHILIAN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddBaosanniang {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                used_kind: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![1], 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, result) = choose(state, male, vec![male_card.raw()], 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, result) = choose(state, owner, vec![owner_sha.raw()], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&owner_sha));
    assert!(!state.hand_cards(male).contains(&male_card));
}

#[test]
fn jijian_gives_sha_reveals_same_color_and_blocks_second_sha_name() {
    let (mut state, players) = setup();
    let (owner, male, victim) = (players[0], players[1], players[2]);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        20,
    );
    let fire_sha = inject(
        &mut state,
        male,
        ZoneKind::Hand,
        CardKind::ShaFire,
        Suit::Heart,
        21,
    );
    let wuzhong = inject(
        &mut state,
        male,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        22,
    );
    let skill = skill_id(&state, owner, JIJIAN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddBaosanniang {
                player: owner,
                target: male,
                card: gift,
                action: 1,
                cards: vec![gift],
                targets: vec![male],
                used_kind: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert!(state.hand_cards(male).contains(&gift));
    let (state, result) = choose(state, male, vec![fire_sha.raw(), wuzhong.raw()], 30);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == u32::from(CardKind::ShaFire.tag())));
    let (state, result) = choose(state, owner, vec![u32::from(CardKind::ShaFire.tag())], 31);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, owner);
    let (state, result) = choose(state, owner, vec![victim.raw()], 32);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, male);
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options, vec![u32::from(CardKind::TrickWuzhong.tag())]);
    let (state, result) = choose(
        state,
        male,
        vec![u32::from(CardKind::TrickWuzhong.tag())],
        33,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(state.resolution.pending_choice_id == 0);
}
