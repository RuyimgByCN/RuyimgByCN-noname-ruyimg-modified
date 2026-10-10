//! Golden for XIANDING（限定包）批次二技能：
//! 精弓（dcjinggong）、忠节（dczhongjie）、花火（dchuahuo）、
//! 节行（dcjiexing）、静隐（dcjingyin）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJINGGONG: u32 = 3_793_668_177;
const DCZHONGJIE: u32 = 2_275_941_720;
const DCHUAHUO: u32 = 2_857_165_081;
const DCJIEXING: u32 = 4_014_891_370;
const DCJINGYIN: u32 = 3_501_643_698;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn dispatch(tx: &mut Transaction, event: DomainEvent) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new())
}

#[test]
fn package_exposes_batch2_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let jinggong = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DCJINGGONG)
        .expect("dcjinggong view_as");
    assert_eq!(jinggong.as_kind, "sha");
    assert_eq!(jinggong.window, "play_phase");
    assert_eq!(jinggong.filter, "card_equip");
    assert_eq!(jinggong.source_zone.as_deref(), Some("hand_equip"));
    assert!(jinggong.ignore_distance);

    let huahuo = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DCHUAHUO)
        .expect("dchuahuo view_as");
    assert_eq!(huahuo.as_kind, "sha");
    assert_eq!(huahuo.filter, "red");
    assert_eq!(huahuo.nature.as_deref(), Some("fire"));

    let zhongjie = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCZHONGJIE)
        .expect("dczhongjie trigger");
    assert_eq!(zhongjie.event, "dying");
    assert_eq!(zhongjie.filter, "owner_alive_other_dying_once_per_round");
    assert!(zhongjie.optional);

    let jiexing: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCJIEXING)
        .map(|trigger| {
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert_eq!(
        jiexing,
        vec![
            ("damage", "owner_is_target", true),
            ("hp_recovered", "owner_is_target", true),
            ("hp_lost", "owner_is_target", true),
        ]
    );

    let jingyin = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJINGYIN)
        .expect("dcjingyin trigger");
    assert_eq!(jingyin.event, "sha_resolved");
    assert_eq!(
        jingyin.filter,
        "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn"
    );
    assert!(jingyin.optional);
}

#[test]
fn dczhongjie_recovers_and_draws_for_dying_player() {
    let (mut tx, owner, first, _second) = setup(20_260_802);
    add_skill(&mut tx, owner, DCZHONGJIE);
    tx.state_mut().players.get_mut(first).unwrap().hp = 0;
    tx.state_mut().players.get_mut(first).unwrap().dying = true;
    let err = dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        },
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, 1);
    assert_eq!(state.hand_count(first), 1);
}

#[test]
fn dcjiexing_draws_after_damage_recovery_and_hp_loss() {
    let (mut tx, owner, first, _second) = setup(20_260_803);
    add_skill(&mut tx, owner, DCJIEXING);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;

    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = participant.max_hp as i32 - 1;
    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 2);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcjinggong_equip_can_be_viewed_as_sha_in_play_phase() {
    let (mut tx, owner, _first, second) = setup(20_260_804);
    add_skill(&mut tx, owner, DCJINGGONG);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = owner;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCJINGGONG)
        .expect("dcjinggong skill instance")
        .id;
    let source = inject_hand(&mut tx, owner, CardKind::Weapon, Suit::Spade, 3);
    let state = tx.state().clone();

    let (state, pipe) = activate(state, owner, skill, vec![second], source, 5);
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "jinggong rejected: {:?}",
        pipe.result
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(source).unwrap().zone, discard);
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    source: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![source],
            },
        },
    )
}
