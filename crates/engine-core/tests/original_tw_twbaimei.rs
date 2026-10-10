//! Golden coverage for TW 马良「白眉」.

use engine_api::{CardId, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWBAIMEI: u32 = 3_304_922_519;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twbaimei.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twbaimei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 358);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWBAIMEI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    kind: CardKind,
    nature: u8,
) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature,
            card: CardId::NONE,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn empty_hand_prevents_elemental_or_ordinary_trick_only() {
    let (state, owner, source) = setup(20_260_801);
    let hp = state.players.get(owner).unwrap().hp;
    let state = damage(
        state,
        source,
        owner,
        CardKind::ShaFire,
        rule_api::damage_nature::FIRE,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp);

    let (state, owner, source) = setup(20_260_802);
    let hp = state.players.get(owner).unwrap().hp;
    let state = damage(
        state,
        source,
        owner,
        CardKind::TrickJuedou,
        rule_api::damage_nature::NORMAL,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp);

    let (state, owner, source) = setup(20_260_803);
    let hp = state.players.get(owner).unwrap().hp;
    let state = damage(
        state,
        source,
        owner,
        CardKind::Sha,
        rule_api::damage_nature::NORMAL,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
}
