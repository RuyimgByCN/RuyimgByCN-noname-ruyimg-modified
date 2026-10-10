//! Golden coverage for OFFLINE 荆扬黄盖「烈舟」「诈降」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_use_unrespondable_for_kind, sha_unrespondable_for_card};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{damage_nature, EffectRequest};

const JYLIEZHOU: u32 = 3_307_262_280;
const JYZHAXIANG: u32 = 2_988_024_345;
const JYZHAXIANG_EFFECT: u32 = 3_250_673_088;
const JY_HUANGGAI: u32 = 2_216_797_450;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/offline/jyliezhou.skill"),
    "\n",
    include_str!("../../../content/original/ported/offline/jyzhaxiang.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    zhaxiang: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("jy-huanggai-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JYLIEZHOU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JYZHAXIANG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let zhaxiang = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JYZHAXIANG)
        .unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        zhaxiang,
    }
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn set_linked(tx: &mut Transaction, player: PlayerId, linked: bool) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    if linked {
        marks.push((engine_core::content::MARK_CHAIN, 1));
        marks.sort_by_key(|(mark, _)| *mark);
    }
}

fn inject_sha(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(306_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
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
                targets: vec![],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn package_publishes_abi306_skills_helper_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 306);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYLIEZHOU && skill.locked));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JYZHAXIANG && !skill.locked));
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == JYZHAXIANG_EFFECT && skill.name == "jyzhaxiang_effect" && skill.locked
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == JY_HUANGGAI
            && character.name_key == "jy_huanggai"
            && character.sex == "male"
            && character.group == "wu"
            && character.max_hp == 5
            && character.initial_hp == 4
            && character.skills == [JYLIEZHOU, JYZHAXIANG]
    }));
}

#[test]
fn liezhou_converts_outgoing_damage_and_draws_final_amount_from_pre_damage_link() {
    let setup = setup(306_01);
    let mut tx = Transaction::begin(setup.state);
    set_linked(&mut tx, setup.target, true);
    let before = tx.state().hand_count(setup.owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.owner,
            target: setup.target,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::THUNDER,
        }],
    )
    .unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
            ..
        } if *source == setup.owner && *target == setup.target
    )));
    assert_eq!(tx.state().hand_count(setup.owner), before + 2);
    assert!(!tx
        .state()
        .players
        .get(setup.target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
}

#[test]
fn liezhou_ignores_unlinked_zero_source_free_and_hp_loss_damage() {
    let setup = setup(306_02);
    let mut tx = Transaction::begin(setup.state);
    let before = tx.state().hand_count(setup.owner);
    let events = run_effects(
        &mut tx,
        vec![
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.target,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: setup.other,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::THUNDER,
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.other,
                amount: 1,
                is_lose_hp: true,
                nature: damage_nature::NORMAL,
            },
            EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.target,
                amount: 0,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            },
        ],
    )
    .unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            nature: damage_nature::FIRE,
            is_lose_hp: false,
            ..
        } if *source == setup.owner
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            nature: damage_nature::THUNDER,
            ..
        } if source.is_none()
    )));
    assert_eq!(tx.state().hand_count(setup.owner), before);
}

#[test]
fn liezhou_chain_draws_once_for_each_player_linked_before_propagation() {
    let setup = setup(306_03);
    let mut tx = Transaction::begin(setup.state);
    set_linked(&mut tx, setup.target, true);
    set_linked(&mut tx, setup.other, true);
    let target_hp = tx.state().players.get(setup.target).unwrap().hp;
    let other_hp = tx.state().players.get(setup.other).unwrap().hp;
    let before = tx.state().hand_count(setup.owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.owner,
            target: setup.target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    )
    .unwrap();

    assert_eq!(
        tx.state().players.get(setup.target).unwrap().hp,
        target_hp - 1
    );
    assert_eq!(
        tx.state().players.get(setup.other).unwrap().hp,
        other_hp - 1
    );
    assert_eq!(tx.state().hand_count(setup.owner), before + 2);
}

#[test]
fn zhaxiang_reduces_max_hp_once_installs_both_response_locks_and_expires() {
    let mut setup = setup(306_04);
    let sha = inject_sha(&mut setup.state, setup.owner);
    let before_max = setup.state.players.get(setup.owner).unwrap().max_hp;

    let (state, result) = activate(setup.state, setup.owner, setup.zhaxiang, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        before_max - 1
    );
    assert!(has_skill(&state, setup.owner, JYZHAXIANG_EFFECT));
    assert!(sha_unrespondable_for_card(&state, setup.owner, sha));
    assert!(card_use_unrespondable_for_kind(
        &state,
        setup.owner,
        CardKind::TrickJuedou,
        Suit::Spade
    ));
    assert!(!card_use_unrespondable_for_kind(
        &state,
        setup.target,
        CardKind::TrickJuedou,
        Suit::Spade
    ));

    let max_after_first = state.players.get(setup.owner).unwrap().max_hp;
    let (state, second) = activate(state, setup.owner, setup.zhaxiang, 2);
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        max_after_first
    );

    let mut state = state;
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, end) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(end.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, setup.owner, JYZHAXIANG_EFFECT));
    assert!(!sha_unrespondable_for_card(&state, setup.owner, sha));
}

#[test]
fn zhaxiang_rejects_outside_play_and_max_hp_zero_kills_the_owner() {
    let setup = setup(306_05);
    let mut invalid = setup.state.clone();
    invalid.turn.phase = Phase::End;
    let before = invalid.players.get(setup.owner).unwrap().max_hp;
    let (invalid, result) = activate(invalid, setup.owner, setup.zhaxiang, 1);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(invalid.players.get(setup.owner).unwrap().max_hp, before);

    let mut lethal = setup.state;
    {
        let owner = lethal.players.get_mut(setup.owner).unwrap();
        owner.max_hp = 1;
        owner.hp = 1;
    }
    let (lethal, result) = activate(lethal, setup.owner, setup.zhaxiang, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner = lethal.players.get(setup.owner).unwrap();
    assert_eq!(owner.max_hp, 0);
    assert!(!owner.alive);
}
