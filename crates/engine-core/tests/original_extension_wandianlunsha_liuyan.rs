//! Golden for archived extension 玩点论杀 刘焉【绝道/割据】（ABI 611）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUEDAO: u32 = 3_304_739_063;
const GEJU: u32 = 4_049_779_435;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/liuyan.skill");

fn inject_hand(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_111),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn juedao_links_with_bidirectional_distance_and_geju_counts_whole_groups() {
    let package = content_compiler::compile_source_named("wandianlunsha-liuyan", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 611);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let adjacent = players[1];
    let opposite = players[2];
    let other_adjacent = players[3];
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
            skill_def: SkillDefId::new(JUEDAO),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GEJU),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().group_overrides.insert(owner, "qun".into());
    tx.state_mut()
        .group_overrides
        .insert(adjacent, "wei".into());
    tx.state_mut()
        .group_overrides
        .insert(opposite, "shu".into());
    tx.state_mut()
        .group_overrides
        .insert(other_adjacent, "wei".into());
    assert!(engine_core::query::player_in_attack_range(
        tx.state(),
        owner,
        adjacent
    ));
    assert!(engine_core::query::player_in_attack_range(
        tx.state(),
        adjacent,
        owner
    ));

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject_hand(&mut tx, owner);
    let juedao = skill(tx.state(), owner, JUEDAO);
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: juedao,
                targets: Vec::new(),
                card_targets: vec![cost],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(!state.hand_cards(owner).contains(&cost));
    assert!(!engine_core::query::player_in_attack_range(
        &state, owner, adjacent
    ));
    assert!(!engine_core::query::player_in_attack_range(
        &state, adjacent, owner
    ));

    let mut tx = Transaction::begin(state);
    let second_cost = inject_hand(&mut tx, owner);
    let state = tx.commit().state;
    let (state, rejected) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill: juedao,
                targets: Vec::new(),
                card_targets: vec![second_cost],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert!(state.hand_cards(owner).contains(&second_cost));

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let hand_before = tx.state().hand_count(owner);
    let geju = skill(tx.state(), owner, GEJU);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveWdbLiuyan {
            player: owner,
            card: CardId::NONE,
            action: 1,
            skill: geju,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 2);
}
