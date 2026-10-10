//! Golden coverage for 蛊惑/禅缘（shenhua 于吉，ABI 552）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_GUHUO: u32 = 2_102_070_101;
const CHANYUAN: u32 = 4_099_161_204;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/yuji.skill");

struct Setup {
    state: GameState,
    owner: PlayerId,
    challenger: PlayerId,
    target: PlayerId,
    source: CardId,
}

fn setup(kind: CardKind, suit: Suit) -> Setup {
    let package = content_compiler::compile_source_named("yuji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_072_931, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let challenger = players[1];
    let target = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_GUHUO),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CHANYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let source = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_273),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(source);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        challenger,
        target,
        source,
    }
}

fn choose(state: GameState, token: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn declare_sha(setup: Setup, decisions: [u32; 2]) -> GameState {
    let skill = setup.state.players.get(setup.owner).unwrap().skills[0];
    let revision = setup.state.revision;
    let (state, result) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Sha.tag(),
                targets: vec![setup.target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = choose(state, setup.source.raw(), 2);
    let state = choose(state, decisions[0], 3);
    choose(state, decisions[1], 4)
}

#[test]
fn package_exposes_yuji_contract() {
    let compiled = content_compiler::compile_source_named("yuji", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 552);
    let guhuo_views: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|v| v.skill_def_id == XINFU_GUHUO)
        .collect();
    assert_eq!(guhuo_views.len(), 2);
    assert!(guhuo_views
        .iter()
        .any(|v| v.as_kind == "basic" && v.cost.as_deref() == Some("guhuo")));
    assert!(guhuo_views
        .iter()
        .any(|v| v.as_kind == "ordinary_trick" && v.cost.as_deref() == Some("guhuo")));
    let chanyuan_mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == CHANYUAN)
        .map(|m| m.target.as_str())
        .collect();
    assert!(chanyuan_mods.contains(&"skills_disabled_when_hp_one"));
    let chanyuan = compiled
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == CHANYUAN)
        .unwrap();
    assert!(chanyuan.locked);
    assert_eq!(compiled.skill_index.len(), 2);
}

#[test]
fn guhuo_declared_sha_hits_when_unchallenged() {
    let s = setup(CardKind::Sha, Suit::Spade);
    let target = s.target;
    let target_hp = s.state.players.get(target).unwrap().hp;
    let state = declare_sha(s, [0, 0]);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp - 1,
        "unchallenged declared Sha deals damage"
    );
}

#[test]
fn chanyuan_disables_other_skills_at_one_hp() {
    let s = setup(CardKind::Sha, Suit::Spade);
    let mut tx = Transaction::begin(s.state);
    // 于吉 at 1 HP: 蛊惑 must be disabled, 禅缘 itself stays enabled.
    tx.state_mut().players.get_mut(s.owner).unwrap().hp = 1;
    let state = tx.commit().state;
    let enabled_defs: Vec<_> = state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == s.owner && state.skill_effectively_enabled(skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(
        !enabled_defs.contains(&XINFU_GUHUO),
        "蛊惑 disabled at one HP: {enabled_defs:?}"
    );
    assert!(
        enabled_defs.contains(&CHANYUAN),
        "禅缘 itself remains enabled"
    );
    // At two HP the 蛊惑 view-as is enabled again.
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(s.owner).unwrap().hp = 2;
    let state = tx.commit().state;
    let enabled_defs: Vec<_> = state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == s.owner && state.skill_effectively_enabled(skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(
        enabled_defs.contains(&XINFU_GUHUO),
        "蛊惑 re-enabled at two HP"
    );
}

#[test]
fn guhuo_use_rejected_at_one_hp() {
    let s = setup(CardKind::Sha, Suit::Spade);
    let mut tx = Transaction::begin(s.state);
    tx.state_mut().players.get_mut(s.owner).unwrap().hp = 1;
    let state = tx.commit().state;
    let skill = state.players.get(s.owner).unwrap().skills[0];
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: s.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Sha.tag(),
                targets: vec![s.target],
                equip_slot: 0,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Rejected { .. }),
        "蛊惑 use rejected at one HP: {:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn guhuo_truthful_declaration_resolves_with_challenge() {
    // 真实声明（红桃杀）被质疑：按验真结算——目标受伤。
    let s = setup(CardKind::Sha, Suit::Heart);
    let target = s.target;
    let target_hp = s.state.players.get(target).unwrap().hp;
    let state = declare_sha(s, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}
