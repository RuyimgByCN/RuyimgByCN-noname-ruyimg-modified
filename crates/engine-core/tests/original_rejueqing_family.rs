//! Golden coverage for the rewritten 界张春华/游佐「绝情」family.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::{damage_card_flag, EffectRequest};

const REJUEQING: u32 = 4_234_607_754;
const RESHANGSHI: u32 = 2_171_600_412;
const SHANGSHI: u32 = 3_386_412_611;
const JUEQING: u32 = 3_481_582_878;
const ABYUSA_JUEQING: u32 = 3_981_540_585;
const ABYUSA_REWRITE: u32 = 3_148_657_445;
const KEY_ABYUSA: u32 = 4_288_818_948;
const RE_ZHANGCHUNHUA: u32 = 1_658_871_287;

const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(package: &[u8], skill_def: u32, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 4;
        owner_state.hp = 4;
        let target_state = tx.state_mut().players.get_mut(target).unwrap();
        target_state.max_hp = 6;
        target_state.hp = 6;
    }
    (tx, owner, target)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == def)
}

fn assert_rewrite(package: &[u8], original: u32, replacement: u32, chain: bool, seed: u64) {
    let (mut tx, owner, target) = setup(package, original, seed);
    let damage = if chain {
        EffectRequest::DealCardDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: damage_card_flag::CHAIN_PROPAGATED,
        }
    } else {
        EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }
    };
    assert_eq!(
        run_effects(&mut tx, vec![damage]),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(target).unwrap().hp, 4);
    assert!(!has_skill(&state, owner, original));
    assert!(has_skill(&state, owner, replacement));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, 3);
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn packages_publish_complete_characters_and_distinct_chain_filters() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == REJUEQING
            && trigger.filter == "owner_source_other_direct_damage_before"));
    assert!(yijiang
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == RESHANGSHI
            && trigger.filter == "owner_is_target_discardable_hand_equip"));

    let key = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert_eq!(key.manifest.rule_vm_abi_version, 691);
    assert_eq!(
        key.character_index
            .iter()
            .find(|character| character.id == KEY_ABYUSA)
            .unwrap()
            .skills,
        [ABYUSA_JUEQING, 3_195_600_790]
    );
    assert!(key
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == ABYUSA_JUEQING
            && trigger.filter == "owner_source_other_damage_before"));

    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert_eq!(
        refresh
            .character_index
            .iter()
            .find(|character| character.id == RE_ZHANGCHUNHUA)
            .unwrap()
            .skills,
        [REJUEQING, SHANGSHI]
    );
}

#[test]
fn acceptance_pays_hp_doubles_damage_and_permanently_rewrites() {
    assert_rewrite(YIJIANG, REJUEQING, JUEQING, false, 2_026_072_801);
    assert_rewrite(KEY, ABYUSA_JUEQING, ABYUSA_REWRITE, true, 2_026_072_802);
}

#[test]
fn rejueqing_ignores_chain_propagation() {
    let (mut tx, owner, target) = setup(YIJIANG, REJUEQING, 2_026_072_803);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: damage_card_flag::CHAIN_PROPAGATED,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 5);
    assert!(has_skill(tx.state(), owner, REJUEQING));
    assert!(!tx.state().has_pending_choice());
}
