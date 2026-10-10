//! Golden for 3D精选 丁奉【短兵】（ABI 621）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDDUANBING: u32 = 2_748_520_735;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/dingfeng.skill");

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, serial: u32) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_621_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("ddd-dingfeng", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(DDDDUANBING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(tx.state_mut(), owner, ZoneKind::Hand, 1);
    inject(tx.state_mut(), PlayerId::NONE, ZoneKind::DrawPile, 2);
    inject(tx.state_mut(), PlayerId::NONE, ZoneKind::DrawPile, 3);
    (tx.commit().state, owner, target, cost)
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == DDDDUANBING)
        })
        .unwrap()
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
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

#[test]
fn package_exposes_exact_abi_and_optional_card_damage_trigger() {
    let package = content_compiler::compile_source_named("ddd-dingfeng", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 621);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    let trigger = &package.trigger_index[0];
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_source_card_damage_target_can_receive_owner_bingliang"
    );
    assert!(trigger.optional);
}

#[test]
fn duanbing_places_bingliang_draws_two_uses_sha_and_transfers_it() {
    let (state, owner, target, cost) = setup();
    let hp = state.players.get(target).unwrap().hp;
    let active = skill(&state, owner);
    let (state, opened) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(opened.choice.unwrap().kind, ChoiceKind::SelectCards);

    let (state, sha_target) = choose(state, owner, vec![cost.raw()], 2);
    assert_eq!(sha_target.choice.unwrap().kind, ChoiceKind::SelectTargets);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(
        state.cards.get(cost).unwrap().kind,
        CardKind::TrickBingliang
    );

    let (state, trigger) = choose(state, owner, vec![target.raw()], 3);
    assert_eq!(trigger.choice.unwrap().kind, ChoiceKind::Confirm);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let (state, result) = choose(state, owner, vec![1], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    let target_judge = state.zones.find(ZoneKind::Judge, target).unwrap();
    assert!(!state.zones.get(owner_judge).unwrap().cards.contains(&cost));
    assert!(state.zones.get(target_judge).unwrap().cards.contains(&cost));
}
