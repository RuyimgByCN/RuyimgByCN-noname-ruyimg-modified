//! Golden coverage for 贾诩「乱武」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LUANWU: u32 = 2_772_376_501;
const RED_SHA: u32 = 4_290_000_090;
const LUANWU_SOURCE: &str = include_str!("../../../content/original/ported/shenhua/luanwu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

fn setup(seed: u64, view_as: bool) -> Setup {
    let source = if view_as {
        format!(
            "{LUANWU_SOURCE}\nskill red_sha({RED_SHA}) {{ view_as sha play_phase red from hand; }}"
        )
    } else {
        LUANWU_SOURCE.into()
    };
    let package = content_compiler::compile_source_named("luanwu-test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let others = [players[1], players[2], players[3]];
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
        skill_def: SkillDefId::new(LUANWU),
    });
    if view_as {
        effects.push(EffectRequest::AddSkill {
            player: others[1],
            skill_def: SkillDefId::new(RED_SHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 5;
        participant.max_hp = 5;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        others,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_090 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId, skill_def: u32) -> SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let skill = skill_instance(&state, owner, LUANWU);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn option_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_exact_luanwu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == LUANWU)
        .unwrap();
    assert!(active.once_per_game);
    assert!(active.targets.is_none());
    let compiled = content_compiler::compile_source_named("luanwu", LUANWU_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 90);
    assert_eq!(compiled.modules.len(), 1);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash,
        compiled.modules[0].logic_hash
    );
}

#[test]
fn luanwu_runs_clockwise_uses_only_nearest_targets_and_does_not_count_sha() {
    let mut setup = setup(2_026_072_290, false);
    let [first, user, last] = setup.others;
    let sha = inject_hand(&mut setup.state, user, CardKind::Sha, Suit::Spade);
    let sha_count = setup.state.turn.flag(turn_flag::sha_used_by_player(user));

    let (state, activated) = activate(setup.state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let source_choice = activated.choice.as_ref().unwrap();
    assert_eq!(source_choice.observer, user);
    assert_eq!(source_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(option_tokens(&activated), [sha.raw()]);
    assert_eq!(state.players.get(first).unwrap().hp, 4);

    let (state, selected) = resolve(state, vec![sha.raw()], false, 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selected.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(option_tokens(&selected), [last.raw(), first.raw()]);
    assert!(!option_tokens(&selected).contains(&setup.owner.raw()));

    let (state, used) = resolve(state, vec![first.raw()], false, 3);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    assert_eq!(state.players.get(last).unwrap().hp, 4);
    assert_eq!(
        state.turn.flag(turn_flag::sha_used_by_player(user)),
        sha_count
    );
    assert_eq!(
        state.cards.get(sha).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (_, repeated) = activate(state, setup.owner, 4);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn cancelling_a_legal_luanwu_sha_loses_hp_and_preserves_the_source_card() {
    let mut setup = setup(2_026_072_291, false);
    let [first, user, last] = setup.others;
    let sha = inject_hand(&mut setup.state, user, CardKind::Sha, Suit::Club);
    let (state, activated) = activate(setup.state, setup.owner, 10);
    assert_eq!(activated.choice.as_ref().unwrap().observer, user);
    let (state, cancelled) = resolve(state, Vec::new(), true, 11);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    for player in [first, user, last] {
        assert_eq!(state.players.get(player).unwrap().hp, 4);
    }
    assert_eq!(state.cards.get(sha).unwrap().owner, user);
    assert_eq!(
        state.cards.get(sha).unwrap().zone,
        state.zones.find(ZoneKind::Hand, user).unwrap()
    );
}

#[test]
fn luanwu_accepts_play_phase_view_as_sha_sources() {
    let mut setup = setup(2_026_072_292, true);
    let [first, user, _last] = setup.others;
    let red_other = inject_hand(&mut setup.state, user, CardKind::Other, Suit::Heart);
    let (state, activated) = activate(setup.state, setup.owner, 20);
    assert_eq!(activated.choice.as_ref().unwrap().observer, user);
    assert_eq!(option_tokens(&activated), [red_other.raw()]);
    let (state, selected) = resolve(state, vec![red_other.raw()], false, 21);
    assert!(option_tokens(&selected).contains(&first.raw()));
    let (state, used) = resolve(state, vec![first.raw()], false, 22);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    assert_eq!(
        state.cards.get(red_other).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn luanwu_queue_resumes_after_a_dying_player_is_killed() {
    let mut setup = setup(2_026_072_293, false);
    let [dying, user, _last] = setup.others;
    setup.state.players.get_mut(dying).unwrap().hp = 1;
    let _tao = inject_hand(&mut setup.state, dying, CardKind::Tao, Suit::Heart);
    let sha = inject_hand(&mut setup.state, user, CardKind::Sha, Suit::Diamond);
    let (state, activated) = activate(setup.state, setup.owner, 30);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(activated.choice.as_ref().unwrap().observer, dying);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::DyingSave
    );

    let (state, died) = resolve(state, Vec::new(), true, 31);
    assert!(matches!(died.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(dying).unwrap().alive);
    assert_eq!(died.choice.as_ref().unwrap().observer, user);
    assert_eq!(option_tokens(&died), [sha.raw()]);
}
