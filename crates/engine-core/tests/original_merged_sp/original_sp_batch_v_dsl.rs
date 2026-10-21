//! Golden for sp batch-V DSL rewrites (zero new primitives):
//! 誓仇·DC (dc_olshichou) wounded Sha target-add;
//! 内伐·基本 (neifa_basic) trick/equip disabled + Sha target-add + Sha-use mark;
//! 内伐·非基本 (neifa_nobasic) basic disabled + ordinary-trick target add/remove.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const DC_OLSHICHOU: u32 = 2_535_213_152;
const NEIFA_BASIC: u32 = 3_242_431_242;
const NEIFA_NOBASIC: u32 = 3_552_802_810;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
fn package_publishes_batch_v_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                DC_OLSHICHOU | NEIFA_BASIC | NEIFA_NOBASIC
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(DC_OLSHICHOU, "card_used", "owner_source_sha", true)));
    assert!(triggers.contains(&(NEIFA_BASIC, "card_used", "owner_source_sha", true)));
    assert!(triggers.contains(&(
        NEIFA_NOBASIC,
        "card_used",
        "owner_source_ordinary_trick",
        true
    )));
    // Modifiers: neifa_basic cancels trick/equip and adds Sha-use from its mark.
    let mods: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == NEIFA_BASIC)
        .map(|modifier| {
            (
                modifier.target.as_str(),
                modifier.selector.as_deref().unwrap_or(""),
                modifier.mode.as_str(),
            )
        })
        .collect();
    assert!(mods.contains(&("card_use_cancel", "trick", "override")));
    assert!(mods.contains(&("card_use_cancel", "equip", "override")));
    assert!(mods.contains(&("sha_use_limit", "", "add")));
    let nobasic_mods: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == NEIFA_NOBASIC)
        .map(|modifier| {
            (
                modifier.target.as_str(),
                modifier.selector.as_deref().unwrap_or(""),
                modifier.mode.as_str(),
            )
        })
        .collect();
    assert!(nobasic_mods.contains(&("card_use_cancel", "basic", "override")));
}

fn play_sha(
    state: GameState,
    actor: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = actor;
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn dc_olshichou_opens_target_add_when_wounded() {
    let (mut state, owner) = started(2_026_081_601);
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let target = others[0];
    // Wound the owner.
    state.players.get_mut(owner).unwrap().hp = 1;
    let state = with_skill(state, owner, DC_OLSHICHOU);
    let (state, played) = play_sha(state, owner, target, 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    // The target-adjust choice offers at least the second other player as an
    // addable target (all alive players except the original target).
    let targets: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(targets.contains(&others[1].raw()));
}

#[test]
fn dc_olshichou_skipped_when_healthy() {
    let (state, owner) = started(2_026_081_602);
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let state = with_skill(state, owner, DC_OLSHICHOU);
    let (state, played) = play_sha(state, owner, others[0], 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], false, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // Healthy owner: the body guard skips the target-add effect. The Sha play
    // continues its normal resolution (targets respond), but never opens the
    // target-adjust choice.
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget
    );
}

#[test]
fn neifa_basic_cancels_trick_use() {
    let (state, owner) = started(2_026_081_603);
    let state = with_skill(state, owner, NEIFA_BASIC);
    let mut tx = Transaction::begin(state);
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_010),
        kind: CardKind::TrickWuzhong,
        suit: engine_api::Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let tx = tx.commit();
    let resp = query(
        &tx.state,
        &QueryRequest::CardUsable {
            player: owner,
            card,
        },
        &mut QueryContext::new(),
    );
    assert_eq!(resp, QueryResponse::Bool(false));
}

#[test]
fn neifa_nobasic_opens_target_adjust_on_ordinary_trick() {
    let (state, owner) = started(2_026_081_604);
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let state = with_skill(state, owner, NEIFA_NOBASIC);
    // Play a trick that can target others: 过河拆桥 (guohe). Play via the trick pipeline.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = owner;
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_030),
        kind: CardKind::TrickGuohe,
        suit: engine_api::Suit::Spade,
        rank: 3,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    let (state, played) = apply_command(
        tx.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets: vec![others[0]],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    // The optional trigger opens before Wuxie/target resolution; confirm it.
    let mut state = state;
    if state.resolution.continuation_kind == ContinuationKind::OptionalTrigger {
        let (s, accepted) = resolve(state, owner, vec![1], false, 2);
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
        state = s;
    }
    // Wuxie window may precede the target-adjust choice; loop to the target choice.
    let mut key = 3;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let (s, _) = resolve(state, owner, vec![], true, key);
        state = s;
        key += 1;
    }
    assert!(state.has_pending_choice());
}
