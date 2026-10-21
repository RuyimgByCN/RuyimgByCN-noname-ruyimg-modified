//! Golden coverage for 周仓「新忠勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHOUCANG: u32 = 2_590_799_493;
const XINZHONGYONG: u32 = 3_382_820_136;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/xinzhongyong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

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
        skill_def: SkillDefId::new(XINZHONGYONG),
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

#[test]
fn package_publishes_complete_zhoucang_and_exact_abi54_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHOUCANG)
        .unwrap();
    assert_eq!(character.skills, [XINZHONGYONG]);
    let compiled = content_compiler::compile_source_named("xinzhongyong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 54);
    assert_eq!(compiled.modules.len(), 1);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn black_resolved_sha_is_gifted_only_to_a_non_owner_non_original_target() {
    let mut setup = setup(2_026_073_401);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        CardKind::Sha,
        Suit::Spade,
        99_401,
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
    let (state, recipient_choice) = confirm_trigger(state, setup.owner, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha
    );
    let options = choice_tokens(&recipient_choice);
    assert!(!options.contains(&setup.owner.raw()));
    assert!(!options.contains(&setup.target.raw()));
    assert!(options.contains(&setup.recipient.raw()));
    assert!(options.contains(&setup.other.raw()));

    let before = canonical_state_hash(&state);
    let (same, forged) = resolve(state, setup.owner, vec![setup.target.raw()], false, 3);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, gifted) = resolve(same, setup.owner, vec![setup.recipient.raw()], false, 4);
    assert!(matches!(gifted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, setup.recipient);
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::None);
    assert!(state.resolution.sha_resolution_records.is_empty());
}

#[test]
fn red_shan_group_enables_count_free_sha_with_owner_range_and_owner_exclusion() {
    let mut setup = setup(2_026_073_402);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        CardKind::Sha,
        Suit::Club,
        99_411,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        CardKind::Shan,
        Suit::Heart,
        99_412,
    );
    let followup = inject(
        &mut setup.state,
        setup.recipient,
        CardKind::Sha,
        Suit::Spade,
        99_413,
    );
    let (state, response) = submit(
        setup.state,
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(response.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let (state, responded) = resolve(state, setup.target, vec![shan.raw()], false, 11);
    assert!(matches!(responded.result, CommandResult::Accepted { .. }));
    let (state, recipient_choice) = confirm_trigger(state, setup.owner, 12);
    assert!(choice_tokens(&recipient_choice).contains(&setup.recipient.raw()));
    let (state, group_choice) = resolve(state, setup.owner, vec![setup.recipient.raw()], false, 13);
    assert_eq!(choice_tokens(&group_choice), [1, 2]);
    let (state, sha_source_choice) = resolve(state, setup.owner, vec![2], false, 14);
    assert_eq!(state.cards.get(shan).unwrap().owner, setup.recipient);
    assert_eq!(state.resolution.continuation_payload.first(), Some(&2));
    assert!(choice_tokens(&sha_source_choice).contains(&followup.raw()));

    let counted_before = state.turn.flag(turn_flag::SHA_PLAYED);
    let (state, target_choice) = resolve(state, setup.recipient, vec![followup.raw()], false, 15);
    let targets = choice_tokens(&target_choice);
    assert!(!targets.contains(&setup.owner.raw()));
    assert!(targets.contains(&setup.target.raw()));
    assert!(targets.contains(&setup.other.raw()));
    assert!(!targets.contains(&setup.recipient.raw()));

    let hp = state.players.get(setup.other).unwrap().hp;
    let (state, used) = resolve(state, setup.recipient, vec![setup.other.raw()], false, 16);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.other).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), counted_before);
    assert_eq!(
        state.cards.get(followup).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
