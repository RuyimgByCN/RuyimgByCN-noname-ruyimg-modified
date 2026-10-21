//! Golden coverage for 刘表「界自守」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIUBIAO: u32 = 4_020_542_401;
const QUANCONG: u32 = 3_313_708_633;
const CAOXIU: u32 = 3_428_883_420;
const REZISHOU: u32 = 4_220_178_807;
const REZISHOU2: u32 = 2_249_965_594;
const ZISHOU: u32 = 3_804_877_958;
const ZISHOU2: u32 = 4_152_430_403;
const ZONGSHI: u32 = 2_639_072_445;
const REZONGSHI: u32 = 3_490_528_973;
const RE_LIUBIAO: u32 = 1_069_046_603;
const OLDRE_LIUBIAO: u32 = 2_422_267_398;
const REZISHOU_SOURCE: &str =
    include_str!("../../../../content/original/ported/refresh/rezishou.skill");
const ZISHOU_SOURCE: &str =
    include_str!("../../../../content/original/ported/yijiang/zishou.skill");
const REFRESH_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const MOBILE_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&refresh)
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&yijiang)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REZISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = LIUBIAO;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = QUANCONG;
    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = CAOXIU;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        other,
    }
}

fn setup_zishou(seed: u64) -> Setup {
    let setup = setup(seed);
    let mut tx = Transaction::begin(setup.state);
    let rezishou = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == REZISHOU)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: rezishou },
            EffectRequest::AddSkill {
                player: setup.owner,
                skill_def: SkillDefId::new(ZISHOU),
            },
        ],
    )
    .unwrap();
    Setup {
        state: tx.commit().state,
        owner: setup.owner,
        other: setup.other,
    }
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn enter_draw(state: GameState, owner: PlayerId, accept: bool) -> (GameState, PipelineResult) {
    let (state, offered) = submit(state, owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    resolve(state, owner, vec![u32::from(accept)], 2)
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn packages_publish_complete_liubiao_dependency_and_exact_abi61_module() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "refresh"
            && dependency.version == "0.1.0"
    }));
    // 上游 refresh 已移除 liubiao 角色（仅存 xin_liubiao），仅验证技能模块契约。
    let compiled = content_compiler::compile_source_named("rezishou", REZISHOU_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 61);
    for expected in compiled.modules {
        let actual = refresh
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn declining_rezishou_keeps_the_ordinary_single_draw_batch() {
    let setup = setup(2_026_074_101);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert!(!has_skill(&state, setup.owner, REZISHOU2));
    let draws: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn { player, cards } if *player == setup.owner => {
                Some(cards.len())
            }
            _ => None,
        })
        .collect();
    assert_eq!(draws, [2]);
}

#[test]
fn accepting_rezishou_adds_alive_groups_to_one_draw_and_consumes_the_bonus() {
    let setup = setup(2_026_074_102);
    let groups = engine_core::query::alive_group_count(&setup.state);
    assert_eq!(groups, 3);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let expected = 2 + groups as u32;
    assert_eq!(state.hand_count(setup.owner), before + expected);
    assert!(has_skill(&state, setup.owner, REZISHOU2));
    assert_eq!(state.turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS), 0);
    let draws: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn { player, cards } if *player == setup.owner => {
                Some(cards.len() as u32)
            }
            _ => None,
        })
        .collect();
    assert_eq!(draws, [expected]);
}

#[test]
fn rezishou2_prevents_outgoing_damage_to_others_but_not_self_damage() {
    let setup = setup(2_026_074_103);
    let (state, _) = enter_draw(setup.state, setup.owner, true);
    let other_hp = state.players.get(setup.other).unwrap().hp;
    let (state, other_result) = deal_damage(state, setup.owner, setup.other);
    assert!(other_result.is_ok());
    assert_eq!(state.players.get(setup.other).unwrap().hp, other_hp);

    let owner_hp = state.players.get(setup.owner).unwrap().hp;
    let (state, self_result) = deal_damage(state, setup.owner, setup.owner);
    assert!(self_result.is_ok());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
}

#[test]
fn rezishou2_expires_when_the_owners_turn_ends() {
    let setup = setup(2_026_074_104);
    let (mut state, _) = enter_draw(setup.state, setup.owner, true);
    assert!(has_skill(&state, setup.owner, REZISHOU2));
    state.turn.phase = Phase::End;
    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, setup.owner, REZISHOU2));
    assert_ne!(state.turn.current, setup.owner);
}

#[test]
fn zishou_package_closes_both_legacy_liubiao_characters() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE_PACKAGE).unwrap();
    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("zishou", ZISHOU_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 61);
    for expected in compiled.modules {
        let actual = yijiang
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
    assert!(mobile.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let re_liubiao = mobile
        .character_index
        .iter()
        .find(|character| character.id == RE_LIUBIAO)
        .unwrap();
    assert_eq!(re_liubiao.name_key, "re_liubiao");
    assert_eq!(re_liubiao.skills, [ZISHOU, REZONGSHI]);
    let oldre_liubiao = old
        .character_index
        .iter()
        .find(|character| character.id == OLDRE_LIUBIAO)
        .unwrap();
    assert_eq!(oldre_liubiao.name_key, "oldre_liubiao");
    assert_eq!(oldre_liubiao.skills, [ZISHOU, ZONGSHI]);
}

#[test]
fn zishou_draws_by_alive_groups_locks_targets_and_expires_at_turn_end() {
    let setup = setup_zishou(2_026_074_105);
    let groups = engine_core::query::alive_group_count(&setup.state);
    let before = setup.state.hand_count(setup.owner);
    let (mut state, result) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 2 + groups as u32);
    assert!(has_skill(&state, setup.owner, ZISHOU2));
    assert!(engine_core::query::card_targets_self_only_for(
        &state,
        setup.owner
    ));

    state.turn.phase = Phase::End;
    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, setup.owner, ZISHOU2));
}
