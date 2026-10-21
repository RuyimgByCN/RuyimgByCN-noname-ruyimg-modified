//! Golden coverage for 界周仓「忠勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_ZHOUCANG: u32 = 2_752_214_624;
const REZHONGYONG: u32 = 3_348_933_734;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/rezhongyong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    recipient: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(REZHONGYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        recipient: players[2],
        other: players[3],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
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

fn confirm_trigger(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    resolve(state, owner, vec![1], false, key)
}

fn choice_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    state.zones.get(hand).unwrap().cards.len()
}

#[test]
fn package_publishes_complete_re_zhoucang_and_exact_abi310_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 310);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHOUCANG)
        .unwrap();
    assert_eq!(character.skills, [REZHONGYONG]);
    let compiled = content_compiler::compile_source_named("rezhongyong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 310);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn all_sha_and_shan_sources_are_gifted_and_red_black_rewards_both_resolve() {
    let mut setup = setup(2_026_073_403);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        CardKind::Sha,
        Suit::Club,
        99_421,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        CardKind::Shan,
        Suit::Heart,
        99_422,
    );
    let (state, played) = submit(
        setup.state,
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let (state, responded) = resolve(state, setup.target, vec![shan.raw()], false, 2);
    assert!(matches!(responded.result, CommandResult::Accepted { .. }));
    let (state, recipient_choice) = confirm_trigger(state, setup.owner, 3);
    let recipients = choice_tokens(&recipient_choice);
    assert!(!recipients.contains(&setup.owner.raw()));
    assert!(!recipients.contains(&setup.target.raw()));
    assert!(recipients.contains(&setup.recipient.raw()));
    assert!(recipients.contains(&setup.other.raw()));

    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![setup.target.raw()], false, 4);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, gifted) = resolve(state, setup.owner, vec![setup.recipient.raw()], false, 5);
    assert!(matches!(gifted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, setup.recipient);
    assert_eq!(state.cards.get(shan).unwrap().owner, setup.recipient);
    assert_eq!(hand_count(&state, setup.recipient), 2);
    assert_eq!(state.resolution.continuation_payload.first(), Some(&2));

    let (state, cancelled_sha) = resolve(state, setup.recipient, Vec::new(), true, 6);
    assert!(matches!(
        cancelled_sha.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(hand_count(&state, setup.recipient), 3);
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::None);
}
