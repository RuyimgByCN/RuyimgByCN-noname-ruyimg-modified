//! Golden for 3D精选 赵昂【奋计】（ABI 631）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENJI: u32 = 782_265_781;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/zhaoang.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup(phase: Phase) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhaoang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 631);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
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
        player: players[0],
        skill_def: SkillDefId::new(FENJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = phase;
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == FENJI)
        .unwrap()
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

fn begin(state: GameState, owner: PlayerId, action: u8) -> (GameState, Vec<u32>) {
    let skill = skill_id(&state, owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhaoang {
                player: owner,
                action,
                stage: 0,
                card: CardId::NONE,
                kind: 0,
                count: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let options = committed
        .output
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    (committed.state, options)
}

fn inject(state: &mut GameState, owner: PlayerId, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_631_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn draw_phase_wanjian_replaces_normal_draw_with_total_damage() {
    let (state, players) = setup(Phase::Draw);
    let owner = players[0];
    let hp_before = [
        state.players.get(players[1]).unwrap().hp,
        state.players.get(players[2]).unwrap().hp,
    ];
    let (state, _) = begin(state, owner, 0);
    let (state, result) = choose(state, owner, vec![CardKind::TrickWanjian.tag().into()], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(players[1]).unwrap().hp, hp_before[0] - 1);
    assert_eq!(state.players.get(players[2]).unwrap().hp, hp_before[1] - 1);
}

#[test]
fn used_trick_name_is_permanently_removed_from_later_choices() {
    let (state, players) = setup(Phase::Draw);
    let owner = players[0];
    let (state, _) = begin(state, owner, 0);
    let (mut state, result) = choose(state, owner, vec![CardKind::TrickWuzhong.tag().into()], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state.turn.clear_draw_phase_fixed_count();
    let (_, options) = begin(state, owner, 0);
    assert!(!options.contains(&u32::from(CardKind::TrickWuzhong.tag())));
}

#[test]
fn discard_phase_uses_declared_target_count_and_skips_normal_discard() {
    let (mut state, players) = setup(Phase::Discard);
    let owner = players[0];
    for serial in 0..4 {
        inject(&mut state, owner, serial);
    }
    let (state, _) = begin(state, owner, 1);
    let (state, result) = choose(state, owner, vec![CardKind::TrickTaoyuan.tag().into()], 3);
    let choice = result
        .choice
        .as_ref()
        .expect("奋计应要求弃置三个目标数的牌");
    assert_eq!(choice.min_count, 3);
    assert_eq!(choice.max_count, 3);
    let cards = choice
        .options
        .iter()
        .take(3)
        .map(|option| option.token)
        .collect();
    let (state, result) = choose(state, owner, cards, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.turn.flag(turn_flag::SKIP_DISCARD), 1);
}
