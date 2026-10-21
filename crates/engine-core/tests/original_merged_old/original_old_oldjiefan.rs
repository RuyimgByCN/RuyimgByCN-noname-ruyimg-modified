//! Golden coverage for old「解烦」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDGONGJI: u32 = 4_171_347_410;
const OLDJIEFAN: u32 = 3_185_096_783;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/old/oldgongji.skill"),
    include_str!("../../../../content/original/ported/old/oldjiefan.skill")
);
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn resolve(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn oldgongji_equipment_sha_uses_normal_response_chain_then_saves_the_bound_dying_player() {
    let package = content_compiler::compile_source_named("oldjiefan-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_913, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let current = players[1];
    let dying = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let current_hand = tx.state().zones.find(ZoneKind::Hand, current).unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::MoveCards {
        cards: tx.state().hand_cards(current),
        to: discard,
        from: Some(current_hand),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDGONGJI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDJIEFAN),
    });
    run_effects(&mut tx, effects).unwrap();

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_270),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 2,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(equipment);
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(dying).unwrap().hp = 1;
    let current_hp = tx.state().players.get(current).unwrap().hp;

    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: current,
                target: dying,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve(tx.commit().state, owner, 1, 1);
    assert_eq!(
        state.resolution.pending_observer,
        owner.raw(),
        "accepting 解烦 must ask for its Sha source"
    );
    let state = resolve(state, owner, equipment.raw(), 2);

    assert_eq!(state.players.get(current).unwrap().hp, current_hp);
    assert_eq!(state.players.get(dying).unwrap().hp, 1);
    assert!(!state.players.get(dying).unwrap().dying);
    assert_eq!(
        state.cards.get(equipment).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn package_publishes_oldjiefan_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDJIEFAN && skill.name == "oldjiefan"));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == 3_688_822_524 && character.name_key == "old_handang"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
