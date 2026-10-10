//! Golden coverage for SP 庞德「决死」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUESI: u32 = 2_412_286_994;
const MASHU: u32 = 2;
const SP_PANGDE: u32 = 2_874_744_170;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/juesi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("juesi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    for player in [owner, target] {
        tx.state_mut().players.get_mut(player).unwrap().hp = 4;
        tx.state_mut().players.get_mut(player).unwrap().max_hp = 4;
    }
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
        skill_def: SkillDefId::new(JUESI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JUESI)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(setup: Setup, cost: CardId, key: u64) -> (GameState, PipelineResult) {
    let revision = setup.state.revision;
    apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![cost],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    card: CardId,
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
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi192_and_complete_sp_pangde() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JUESI)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.in_attack_range && targets.hand_equip_nonempty);
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_sha")
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SP_PANGDE)
        .unwrap();
    assert_eq!(character.name_key, "sp_pangde");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [MASHU, JUESI]);

    let compiled = content_compiler::compile_source_named("juesi", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn non_sha_discard_at_equal_hp_uses_duel_after_both_discards() {
    let mut setup = setup(2_026_072_609);
    let cost = inject(&mut setup.state, setup.owner, 192_001, CardKind::Sha);
    let discarded = inject(
        &mut setup.state,
        setup.target,
        192_002,
        CardKind::TrickWuzhong,
    );
    inject(
        &mut setup.state,
        setup.target,
        192_003,
        CardKind::TrickTaoyuan,
    );
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let target = setup.target;
    let (state, offered) = activate(setup, cost, 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner
    );

    let (state, resolved) = resolve(state, target, discarded, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed { player, targets, .. }
            if *player != target && targets == &[target]
    )));
}

#[test]
fn sha_discard_or_lower_target_hp_suppresses_duel() {
    let mut sha_case = setup(2_026_072_610);
    let cost = inject(&mut sha_case.state, sha_case.owner, 192_011, CardKind::Sha);
    let discarded_sha = inject(&mut sha_case.state, sha_case.target, 192_012, CardKind::Sha);
    inject(
        &mut sha_case.state,
        sha_case.target,
        192_013,
        CardKind::TrickWuzhong,
    );
    let target = sha_case.target;
    let hp = sha_case.state.players.get(target).unwrap().hp;
    let (state, _) = activate(sha_case, cost, 10);
    let (state, resolved) = resolve(state, target, discarded_sha, 11);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert!(!resolved
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardUsed { .. })));

    let mut hp_case = setup(2_026_072_611);
    hp_case.state.players.get_mut(hp_case.target).unwrap().hp = 3;
    let cost = inject(&mut hp_case.state, hp_case.owner, 192_021, CardKind::Sha);
    let discarded = inject(
        &mut hp_case.state,
        hp_case.target,
        192_022,
        CardKind::TrickWuzhong,
    );
    inject(
        &mut hp_case.state,
        hp_case.target,
        192_023,
        CardKind::TrickTaoyuan,
    );
    let target = hp_case.target;
    let (state, _) = activate(hp_case, cost, 20);
    let (state, resolved) = resolve(state, target, discarded, 21);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert!(!resolved
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardUsed { .. })));
}
