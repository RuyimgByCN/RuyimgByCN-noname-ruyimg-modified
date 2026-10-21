//! Golden for 3D精选 曹爽【专摄/危秋】（ABI 635）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUANSHE: u32 = 902_778_848;
const WEIQIU: u32 = 208_021_574;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/caoshuang.skill");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-caoshuang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 635);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&standard)
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([ZHUANSHE, WEIQIU].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, serial: u32, kind: CardKind) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_635_000 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

#[test]
fn zhuanshe_gifts_matching_sha_and_adds_a_distance_ignored_target() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let first = players[2];
    let extra = players[3];
    let skill = skill_id(&state, owner, ZHUANSHE);
    let sha = inject(&mut state, owner, 1, CardKind::Sha);
    state.turn.current = actor;
    state.turn.phase = Phase::Play;

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaoshuang {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (mut state, _) = choose(tx.commit().state, owner, vec![sha.raw()], false, 1);
    assert!(state.hand_cards(actor).contains(&sha));

    assert!(!state.skills.get(skill).unwrap().state_blob.is_empty());
    state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    state.resolution.resume_payload = vec![actor.raw(), first.raw(), 1, 0, 1, sha.raw()];
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: actor,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let commit = tx.commit();
    let choice = commit.output.choice.as_ref().unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra.raw()));
    let (mut state, result) = choose(commit.state, owner, vec![extra.raw()], false, 2);
    assert!(result.choice.is_none());
    assert_eq!(state.skills.get(skill).unwrap().state_blob.last(), Some(&1));
    state.resolution.resume_kind = ContinuationKind::None;
    state.resolution.resume_payload.clear();
    state.resolution.clear_choice();

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddCaoshuang {
            player: owner,
            actor,
            card: CardId::NONE,
            action: 2,
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());

    let tao = inject(tx.state_mut(), owner, 2, CardKind::Tao);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaoshuang {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![tao.raw()], false, 3);
    let actor_hp = state.players.get(actor).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaoshuang {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 2,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![1], false, 4);
    assert_eq!(state.players.get(actor).unwrap().hp, actor_hp - 1);
}

#[test]
fn weiqiu_cancels_recovery_before_hp_changes_and_draws_for_each_empty_owner() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    state.players.get_mut(target).unwrap().hp -= 2;
    let hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: target,
            player: target,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp);
    assert_eq!(tx.state().hand_count(owner), 1);
    assert_eq!(tx.state().resolution.current_recovery_id, 0);
    assert!(tx.state().resolution.prevented_recovery_ids.is_empty());
}
