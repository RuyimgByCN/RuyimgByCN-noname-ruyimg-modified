//! Golden coverage for 成宜/独探's source-free multi-target Duel.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WN_DUTAN: u32 = 4_152_033_537;
const WN_CHENGYI: u32 = 2_292_190_551;
const OFFLINE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, Vec<PlayerId>, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(
        (224..=RULE_VM_ABI_VERSION).contains(&package.manifest.rule_vm_abi_version),
        "unexpected package ABI {}",
        package.manifest.rule_vm_abi_version
    );
    assert!(package.skill_index.len() >= 71);
    assert!(package.character_index.len() >= 25);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WN_DUTAN && skill.name == "wn_dutan"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WN_CHENGYI)
        .unwrap();
    assert_eq!(character.name_key, "wn_chengyi");
    assert_eq!(character.skills, [WN_DUTAN]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == WN_DUTAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let target_contract = active.targets.as_ref().unwrap();
    assert_eq!((target_contract.min, target_contract.max), (1, 255));
    assert!(!target_contract.allow_self);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_301, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    assert_eq!(targets.len(), 2);

    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
        let player_state = tx.state_mut().players.get_mut(player).unwrap();
        player_state.hp = 8;
        player_state.max_hp = 8;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WN_DUTAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == WN_DUTAN)
        .unwrap();
    (tx.state().clone(), owner, targets, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    targets: &[PlayerId],
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: targets.to_vec(),
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn wn_dutan_uses_one_virtual_duel_on_every_selected_target_once() {
    let (state, owner, targets, skill) = setup();
    let hp: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();

    let (state, result) = activate(state, owner, &targets, skill, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    for (target, before) in targets.iter().zip(hp) {
        assert_eq!(state.players.get(*target).unwrap().hp, before - 1);
    }
    let uses: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardUsed {
                player,
                card,
                targets: used_targets,
            } if *player == owner => Some((*card, used_targets)),
            _ => None,
        })
        .collect();
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].1.as_slice(), targets.as_slice());
    let virtual_card = state.cards.get(uses[0].0).unwrap();
    assert!(virtual_card.def.is_none());

    let before = canonical_state_hash(&state);
    let (same, second) = activate(state, owner, &targets, skill, 2);
    assert!(matches!(second.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
