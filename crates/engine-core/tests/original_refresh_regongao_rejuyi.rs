//! Golden coverage for REFRESH 界诸葛诞「界功獒」「界举义」.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag, RULE_VM_ABI_VERSION};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RE_ZHUGEDAN: u32 = 3_722_727_883;
const REGONGAO: u32 = 2_805_903_172;
const REJUYI: u32 = 2_754_242_355;
const BENGHUAI: u32 = 3_365_741_831;
const REWEIZHONG: u32 = 4_262_017_394;
const REGONGAO_SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/regongao.skill");
const REJUYI_SOURCE: &str = include_str!("../../../content/original/ported/refresh/rejuyi.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

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
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn fire_dying(tx: &mut Transaction, player: PlayerId) {
    {
        let target = tx.state_mut().players.get_mut(player).unwrap();
        target.hp = 0;
        target.dying = true;
    }
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn enter_prepare(tx: &mut Transaction, owner: PlayerId) {
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_and_runtime_preserve_first_dying_per_target_and_exact_awakening() {
    let handle = std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(|| {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(refresh.manifest.rule_vm_abi_version, 723);
    assert!(refresh
        .manifest
        .dependencies
        .iter()
        .any(|dependency| { dependency.package_id == "shenhua" && dependency.version == "0.1.0" }));
    assert!(refresh.character_index.iter().any(|character| {
        character.id == RE_ZHUGEDAN
            && character.name_key == "re_zhugedan"
            && character.skills == [REGONGAO, REJUYI]
    }));
    let mut dependency_probe = GameState::empty();
    std::sync::Arc::make_mut(&mut dependency_probe.content).load_package(&shenhua).unwrap();
    std::sync::Arc::make_mut(&mut dependency_probe.content).load_package(&refresh).unwrap();

    let source = format!(
        "{REGONGAO_SOURCE}\n{REJUYI_SOURCE}\nskill benghuai({BENGHUAI}) {{}}\nskill reweizhong({REWEIZHONG}) {{}}"
    );
    let package = content_compiler::compile_source_named("regongao-rejuyi-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_073_111, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
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
            skill_def: SkillDefId::new(REGONGAO),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REJUYI),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    {
        let owner = tx.state_mut().players.get_mut(owner).unwrap();
        owner.max_hp = 4;
        owner.hp = 3;
    }

    fire_dying(&mut tx, seats[1]);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    fire_dying(&mut tx, seats[1]);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    fire_dying(&mut tx, seats[2]);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);

    enter_prepare(&mut tx, owner);
    assert!(!has_skill(tx.state(), owner, REJUYI));
    assert!(has_skill(tx.state(), owner, BENGHUAI));
    assert!(has_skill(tx.state(), owner, REWEIZHONG));
    assert_eq!(tx.state().hand_count(owner), 6);
    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), 6);
    })
    .unwrap();
    // 分离线程会吞掉断言 panic（历史 `== 356` 失真未被发现的原因），join 使失败可见。
    handle.join().expect("test thread panicked");
}
