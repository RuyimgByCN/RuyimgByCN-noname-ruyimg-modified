//! Golden semantics for EXTRA 神司马懿「忍戒」「拜印」「极略」「连破」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, SkillId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{
    can_rewrite_judgment, judgment_rewrite_includes_equip, judgment_rewrite_mark_cost, query,
    QueryContext,
};
use engine_core::state::{turn_flag, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SHEN_SIMAYI: u32 = 4_212_877_947;
const RENJIE: u32 = 3_085_167_840;
const RENJIE2: u32 = 2_678_110_361;
const SBAIYIN: u32 = 2_725_045_502;
const LIANPO: u32 = 2_556_576_493;
const JILUE: u32 = 4_138_291_948;
const JILUE_GUICAI: u32 = 3_991_027_258;
const JILUE_FANGZHU: u32 = 3_045_250_601;
const JILUE_JIZHI: u32 = 2_747_240_883;
const JILUE_ZHIHENG: u32 = 3_079_429_948;
const JILUE_WANSHA: u32 = 3_522_766_701;
const DUORUI: u32 = 4_266_461_274;
const ZHITI: u32 = 3_298_844_877;
const EXTRA: &[u8] = include_bytes!("../../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(SHEN_SIMAYI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: SHEN_SIMAYI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, skills).unwrap();
    (tx.state().clone(), players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
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

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(definition, _)| *definition == mark)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_closes_shen_simayi_skill_family() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_SIMAYI)
        .unwrap();
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [RENJIE, SBAIYIN, LIANPO]);
    for definition in [
        RENJIE,
        RENJIE2,
        SBAIYIN,
        LIANPO,
        JILUE,
        JILUE_GUICAI,
        JILUE_FANGZHU,
        JILUE_JIZHI,
        JILUE_ZHIHENG,
        JILUE_WANSHA,
    ] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == definition));
    }
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
}

#[test]
fn renjie_discard_and_sbaiyin_awaken_resolve_in_engine() {
    let (state, players) = setup();
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Discard;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_DISCARDED, 4);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RENJIE),
        }],
    )
    .unwrap();
    let renjie = skill(tx.state(), owner, RENJIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShenSimayi {
            player: owner,
            event_player: owner,
            event_amount: 0,
            mark: MarkDefId::new(RENJIE),
            mode: 0,
            cards: Vec::new(),
            targets: Vec::new(),
            skill: renjie,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 4);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SBAIYIN),
        }],
    )
    .unwrap();
    assert!(has_skill(tx.state(), owner, SBAIYIN));
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 4);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(!effects.is_empty());
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp - 1);
    assert!(!has_skill(tx.state(), owner, SBAIYIN));
    for definition in [
        JILUE,
        JILUE_GUICAI,
        JILUE_FANGZHU,
        JILUE_JIZHI,
        JILUE_ZHIHENG,
        JILUE_WANSHA,
    ] {
        assert!(has_skill(tx.state(), owner, definition));
    }
}

#[test]
fn jilue_zhiheng_and_wansha_pay_ren_and_apply_their_turn_state() {
    let (state, players) = setup();
    let owner = players[0];
    let other = players[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILUE_ZHIHENG),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILUE_WANSHA),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(DUORUI),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(ZHITI),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(RENJIE),
                delta: 2,
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_cards(owner);
    let zhiheng = skill(tx.state(), owner, JILUE_ZHIHENG);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShenSimayi {
            player: owner,
            event_player: owner,
            event_amount: 0,
            mark: MarkDefId::new(RENJIE),
            mode: 3,
            cards: hand.clone(),
            targets: Vec::new(),
            skill: zhiheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_cards(owner).len(), hand.len() + 1);
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 1);

    let wansha = skill(tx.state(), owner, JILUE_WANSHA);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShenSimayi {
            player: owner,
            event_player: owner,
            event_amount: 0,
            mark: MarkDefId::new(RENJIE),
            mode: 2,
            cards: Vec::new(),
            targets: Vec::new(),
            skill: wansha,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner, RENJIE), 0);
    assert_eq!(mark_count(tx.state(), owner, JILUE_WANSHA), 1);
    let duorui = skill(tx.state(), other, DUORUI);
    let zhiti = skill(tx.state(), other, ZHITI);
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(duorui).unwrap()));
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(zhiti).unwrap()));
    tx.state_mut().players.get_mut(other).unwrap().dying = true;
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(duorui).unwrap()));
}

#[test]
fn jilue_jizhi_discards_the_basic_draw_for_one_turn_hand_limit() {
    let (state, players) = setup();
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILUE_JIZHI),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(RENJIE),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let before = query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    );
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let drawn = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(drawn).unwrap().kind = CardKind::Sha;
    let jizhi = skill(tx.state(), owner, JILUE_JIZHI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveShenSimayi {
                player: owner,
                event_player: owner,
                event_amount: 0,
                mark: MarkDefId::new(RENJIE),
                mode: 4,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: jizhi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseJilueJizhiDiscard
    );
    let revision = tx.state().revision;
    let pending = tx.state().resolution.pending_choice_id;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(80),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(pending),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&drawn));
    assert_eq!(mark_count(&state, owner, RENJIE), 0);
    assert_eq!(mark_count(&state, owner, JILUE_JIZHI), 1);
    let after = query(
        &state,
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    );
    let (QueryResponse::Int(before), QueryResponse::Int(after)) = (before, after) else {
        panic!("hand-limit query must return integers");
    };
    assert_eq!(after, before + 1);
}

#[test]
fn jilue_guicai_requires_one_ren_and_accepts_equipment() {
    let (state, players) = setup();
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JILUE_GUICAI),
        }],
    )
    .unwrap();
    assert!(!can_rewrite_judgment(tx.state(), owner));
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(RENJIE),
            delta: 1,
        }],
    )
    .unwrap();
    assert!(can_rewrite_judgment(tx.state(), owner));
    assert!(judgment_rewrite_includes_equip(tx.state(), owner));
    assert_eq!(judgment_rewrite_mark_cost(tx.state(), owner), Some(RENJIE));
}

#[test]
fn lianpo_extra_turn_returns_to_the_original_turn_anchor() {
    let (state, players) = setup();
    let owner = players[0];
    let current = players[1];
    let normal_next = players[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LIANPO),
        }],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: current,
                killer: owner,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let pending = tx.state().resolution.pending_choice_id;
    let revision = tx.state().revision;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(90),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(pending),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, owner);
    assert_eq!(state.turn.phase, Phase::Prepare);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    assert_eq!(tx.state().turn.current, normal_next);
    assert_eq!(tx.state().turn.phase, Phase::Prepare);
}
