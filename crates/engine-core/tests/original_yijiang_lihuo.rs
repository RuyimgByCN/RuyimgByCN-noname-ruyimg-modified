//! Golden coverage for 程普「疠火」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIHUO: u32 = 3_289_572_540;
const OLDQIANXI: u32 = 3_282_621_718;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/lihuo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    extra: PlayerId,
    sha: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, kind: CardKind, oldqianxi: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let extra = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &seats {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIHUO),
    });
    if oldqianxi {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDQIANXI),
        });
    }
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    let sha = inject(
        tx.state_mut(),
        owner,
        ZoneKind::Hand,
        98_001,
        kind,
        Suit::Spade,
    );
    Setup {
        state: tx.commit().state,
        owner,
        target,
        extra,
        sha,
    }
}

fn play(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: setup.sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
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
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_exact_abi53_lihuo_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let compiled = content_compiler::compile_source_named("lihuo", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 2);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn converted_sha_adds_one_target_then_loses_hp_once_after_both_hits() {
    let setup = setup(2_026_072_001, CardKind::Sha, false);
    let (state, offered) = play(&setup, 1);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, add_choice) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(add_choice.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );

    let (state, resolved) = resolve(state, setup.owner, vec![setup.extra.raw()], false, 3);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded {
            player,
            card,
            target,
            targets,
        } if *player == setup.owner
            && *card == setup.sha
            && *target == setup.extra
            && targets == &[setup.target, setup.extra]
    )));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.extra).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
}

#[test]
fn fully_dodged_converted_sha_does_not_cost_hp() {
    let mut setup = setup(2_026_072_002, CardKind::Sha, false);
    let shan = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        98_002,
        CardKind::Shan,
        Suit::Heart,
    );
    let (state, _) = play(&setup, 10);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 11);
    let (state, add_cancelled) = resolve(state, setup.owner, Vec::new(), true, 12);
    assert!(matches!(
        add_cancelled.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let (state, dodged) = resolve(state, setup.target, vec![shan.raw()], false, 13);
    assert!(matches!(dodged.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 4);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
}

#[test]
fn prevented_converted_sha_does_not_cost_hp() {
    let mut setup = setup(2_026_072_003, CardKind::Sha, true);
    inject(
        &mut setup.state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        98_003,
        CardKind::Other,
        Suit::Spade,
    );
    let (state, _) = play(&setup, 20);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 21);
    let (state, _) = resolve(state, setup.owner, Vec::new(), true, 22);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, prevented) = resolve(state, setup.owner, vec![1], false, 23);
    assert!(matches!(prevented.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // 旧潜袭降低体力上限时会把满体力同步夹到新上限；这不是该【杀】造成的伤害。
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.target).unwrap().max_hp, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
}

#[test]
fn natural_fire_sha_may_add_a_target_without_conversion_hp_loss() {
    let setup = setup(2_026_072_004, CardKind::ShaFire, false);
    let (state, offered) = play(&setup, 30);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 31);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
}
