//! Golden for 3D精选 刘宏【失兴/党锢/覆宗】（ABI 637）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHIXING: u32 = 42_478_118;
const DANGGU: u32 = 1_734_030_406;
const FUZONG: u32 = 1_289_813_886;
const XIANGLE: u32 = 2_880_679_348;
const JIUSHI: u32 = 4_293_489_189;
const JIUSHI1: u32 = 3_647_393_232;
const JIUSHI3: u32 = 4_156_803_269;
const REZHIHENG: u32 = 2_455_708_599;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/liuhong.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuhong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let mut tx = Transaction::begin(GameState::empty());
    for bytes in [STANDARD, SHENHUA, YIJIANG, REFRESH] {
        let package = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    }
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_804, "identity-standard").unwrap();
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
    effects.extend(
        [SHIXING, DANGGU, FUZONG].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.character_def = 1;
        participant.max_hp = 4;
        participant.hp = 4;
    }
    let female = tx.state_mut().players.get_mut(players[2]).unwrap();
    female.character_def = 25;
    female.max_hp = 3;
    female.hp = 3;
    tx.state_mut().players.get_mut(players[0]).unwrap().identity = Identity::Lord;
    tx.state_mut()
        .group_overrides
        .insert(players[0], "qun".into());
    tx.state_mut()
        .group_overrides
        .insert(players[1], "qun".into());
    tx.state_mut()
        .group_overrides
        .insert(players[2], "wei".into());
    tx.state_mut()
        .group_overrides
        .insert(players[3], "shu".into());
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

fn has_def(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state.players.get(owner).is_some_and(|player| {
        player.skills.iter().any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
    })
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_637_000 + serial),
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

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => tx.commit().state,
        Err(error) => panic!("dispatch failed: {error:?}"),
    }
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn shixing_switches_existing_derived_skills_and_removes_its_own_grants() {
    let (state, players) = setup();
    let owner = players[0];
    let female = players[2];
    let shixing = skill_id(&state, owner, SHIXING);
    let state = dispatch(state, DomainEvent::GameStarted { player_count: 4 });
    for def in [XIANGLE, JIUSHI, JIUSHI1, JIUSHI3] {
        assert!(has_def(&state, owner, def), "失兴授予 {def}");
    }
    assert!(!has_def(&state, owner, REZHIHENG));

    let mut state = state;
    let trick = inject(&mut state, owner, ZoneKind::Hand, 1, CardKind::TrickJuedou);
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: owner,
            card: trick,
            targets: vec![players[1]],
        },
    );
    assert!(has_def(&state, owner, XIANGLE));
    for def in [JIUSHI, JIUSHI1, JIUSHI3, REZHIHENG] {
        assert!(!has_def(&state, owner, def));
    }

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: female,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!has_def(tx.state(), owner, XIANGLE));
    assert!(has_def(tx.state(), owner, REZHIHENG));

    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill: shixing }]).unwrap();
    assert!(!has_def(tx.state(), owner, REZHIHENG));
}

#[test]
fn danggu_links_at_least_the_qun_shortfall_and_counts_other_linked_equipment() {
    let (mut state, players) = setup();
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::End;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![owner.raw(), players[2].raw()], 1);
    assert!(result.choice.is_none());
    assert!(!engine_core::query::discard_phase_includes_equip(
        &state, owner
    ));
    assert!(engine_core::query::discard_phase_includes_equip(
        &state, players[2]
    ));
    assert!(!engine_core::query::discard_phase_includes_equip(
        &state, players[1]
    ));
}

#[test]
fn fuzong_draws_for_living_qun_and_dead_qun_chooses_one_owner_card_to_discard() {
    let (mut state, players) = setup();
    let owner = players[0];
    let dead = players[1];
    let skill = skill_id(&state, owner, FUZONG);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuhong {
            player: owner,
            actor: PlayerId::NONE,
            action: 4,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);
    state = tx.commit().state;
    let chosen = inject(&mut state, owner, ZoneKind::Hand, 2, CardKind::Sha);
    state.players.get_mut(dead).unwrap().alive = false;
    let state = dispatch(
        state,
        DomainEvent::PlayerKilled {
            player: dead,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        },
    );
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.pending_observer, dead.raw());
    let (state, result) = choose(state, dead, vec![chosen.raw()], 2);
    assert!(result.choice.is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(chosen).unwrap().zone, discard);
}

#[test]
fn shixing_granted_jiushi_uses_source_free_jiu_and_turns_back_after_damage() {
    let (state, players) = setup();
    let owner = players[0];
    let state = dispatch(state, DomainEvent::GameStarted { player_count: 4 });
    let skill = skill_id(&state, owner, JIUSHI1);
    let mut state = state;
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.players.get(owner).unwrap().turned_over);

    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(!matches!(result.result, CommandResult::Accepted { .. }));

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: players[1],
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![1], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(owner).unwrap().turned_over);
}
