//! Golden coverage for SP「谋溃」and 联动包 SP穆顺.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MOUKUI: u32 = 2_368_472_328;
const SP_MUSHUN: u32 = 3_974_832_852;
const SP: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const COLLAB: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source(include_str!(
        "../../../../content/original/ported/sp/moukui.skill"
    ))
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(MOUKUI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
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

fn play_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

#[test]
fn packages_publish_moukui_and_sp_mushun_dependency() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MOUKUI));

    let collab = content_schema::ContentPackage::from_json_bytes(COLLAB).unwrap();
    assert!(collab.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "sp"
            && dependency.version == "0.1.0"
    }));
    let mushun = collab
        .character_index
        .iter()
        .find(|character| character.id == SP_MUSHUN)
        .unwrap();
    assert_eq!(mushun.name_key, "sp_mushun");
    assert_eq!(mushun.max_hp, 4);
    assert_eq!(mushun.skills, [MOUKUI]);
}

#[test]
fn draw_branch_arms_exact_sha_and_target_for_forced_counter_discard() {
    let (mut state, owner, target) = setup(20_260_725_01);
    let sha = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 92_001);
    let owner_card = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::HorseOff,
        92_002,
    );
    let shan = inject(&mut state, target, ZoneKind::Hand, CardKind::Shan, 92_003);

    let (state, played) = play_sha(state, owner, target, sha, 1);
    let branch = played.choice.unwrap();
    assert_eq!(branch.observer, owner);
    assert_eq!(branch.kind, ChoiceKind::Custom);
    assert_eq!(
        branch
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let hand_before = state.hand_count(owner);
    let (state, chosen) = resolve(state, owner, vec![1], false, 2);
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(chosen.choice.as_ref().unwrap().observer, target);

    let (state, dodged) = resolve(state, target, vec![shan.raw()], false, 3);
    let discard = dodged.choice.unwrap();
    assert_eq!(discard.observer, target);
    assert_eq!(discard.kind, ChoiceKind::SelectCards);
    assert!(discard
        .options
        .iter()
        .any(|option| option.token == owner_card.raw()));

    let (state, finished) = resolve(state, target, vec![owner_card.raw()], false, 4);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(owner_card).unwrap().owner, PlayerId::NONE);
}

#[test]
fn discard_branch_discards_target_then_forces_counter_discard_after_dodge() {
    let (mut state, owner, target) = setup(20_260_725_02);
    let sha = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 92_011);
    let owner_card = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::HorseOff,
        92_012,
    );
    let shan = inject(&mut state, target, ZoneKind::Hand, CardKind::Shan, 92_013);
    let target_card = inject(
        &mut state,
        target,
        ZoneKind::Equip,
        CardKind::HorseDef,
        92_014,
    );

    let (state, _) = play_sha(state, owner, target, sha, 10);
    let (state, selected_branch) = resolve(state, owner, vec![2], false, 11);
    let discard_target = selected_branch.choice.unwrap();
    assert_eq!(discard_target.observer, owner);
    assert!(discard_target
        .options
        .iter()
        .any(|option| option.token == target_card.raw()));

    let (state, responded) = resolve(state, owner, vec![target_card.raw()], false, 12);
    assert_eq!(state.cards.get(target_card).unwrap().owner, PlayerId::NONE);
    assert_eq!(responded.choice.as_ref().unwrap().observer, target);

    let (state, counter) = resolve(state, target, vec![shan.raw()], false, 13);
    let counter_discard = counter.choice.unwrap();
    assert_eq!(counter_discard.observer, target);
    assert!(counter_discard
        .options
        .iter()
        .any(|option| option.token == owner_card.raw()));
    let (state, _) = resolve(state, target, vec![owner_card.raw()], false, 14);
    assert_eq!(state.cards.get(owner_card).unwrap().owner, PlayerId::NONE);
}

#[test]
fn cancelling_initial_branch_does_not_arm_the_dodge_consequence() {
    let (mut state, owner, target) = setup(20_260_725_03);
    let sha = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 92_021);
    let owner_card = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::HorseOff,
        92_022,
    );
    let shan = inject(&mut state, target, ZoneKind::Hand, CardKind::Shan, 92_023);

    let (state, _) = play_sha(state, owner, target, sha, 20);
    let (state, declined) = resolve(state, owner, Vec::new(), true, 21);
    assert_eq!(declined.choice.as_ref().unwrap().observer, target);
    let (state, finished) = resolve(state, target, vec![shan.raw()], false, 22);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert!(finished.choice.is_none());
    assert_eq!(state.cards.get(owner_card).unwrap().owner, owner);
}
