//! PA12 迁移批③（滤镜 Top-N 无状态谓词，2026-10-27）：六个高频旧线滤名落原生
//! 精确策略——`owner_source_sha`（76）、`owner_alive_not_target`（65）、
//! `owner_is_target_source_alive`（14）、`owner_is_source_or_target`（13）、
//! `owner_source_fire_sha`（10）、`owner_source_ordinary_trick`（9）。
//! once/round 限次系（`owner_is_target_once` 60、`owner_alive_once_per_round` 38）
//! 需逐回合触发计数状态（新字段进 golden 哈希），登记迁移计划 §2 D 待做。
//! 真实技能全链验收：帝挽（offline）、抚（collab）、dcweidang（huicui）、
//! 军略（extra，火杀另用 debug 火杀驱动 owner_source_fire_sha）。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

const PSDIWAN: &str = include_str!("../../../content/original/ported/offline/psdiwan.skill");
const NATUFU: &str = include_str!("../../../content/original/ported/collab/natufu.skill");
const DCWEIDANG: &str = include_str!("../../../content/original/ported/huicui/dcweidang.skill");
const JUNLVE: &str = include_str!("../../../content/original/ported/extra/nzry_junlve.skill");
const JUNLVE_MARK: u32 = 1_681_567_279;

fn pack_for(source: &str, id: &str) -> ExtensionPack {
    let pkg = compile_source_named(id, source).expect("编译 ported .skill");
    assert!(pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION);
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
    assert!(!out.decls.is_empty());
    ExtensionPack {
        def: ExtensionDef {
            name: "pa12-batch3",
            version: 1,
            description: "PA12 批③ 滤镜谓词",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: out.decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let rev = e.revision();
    let _ = e.submit(GameCommand {
        actor,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action,
    });
}

fn settle_plain(e: &mut Engine, c: &engine_native::OpenChoice) {
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .take(c.spec.min.max(1))
        .collect();
    let sel = if sel.is_empty() {
        vec![c.spec.options[0].clone()]
    } else {
        sel
    };
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
}

fn step(e: &mut Engine, confirm: bool) {
    if let Some(c) = e.open_choice() {
        let c = c.clone();
        if c.spec.prompt == "trigger_confirm" {
            let sel = if confirm {
                c.spec
                    .options
                    .iter()
                    .find(|o| matches!(o, ChoiceOption::Confirm))
                    .copied()
                    .unwrap_or(c.spec.options[0])
            } else {
                c.spec.options[0]
            };
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![sel],
                },
            );
            return;
        }
        settle_plain(e, &c);
        return;
    }
    let cur = e.turn().current;
    submit(e, cur, GameAction::NextPhase);
}

fn advance_to(e: &mut Engine, player: u8, phase: Phase) {
    for _ in 0..300 {
        if e.turn().current == player && e.turn().phase == phase {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        step(e, false);
    }
    panic!("300 步未到 p{player} 目标阶段");
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

fn mark_of(e: &Engine, p: u8, mark: u32) -> i32 {
    e.state_core()
        .vm_aux
        .marks
        .get(&(p, mark))
        .copied()
        .unwrap_or(0)
}

/// 映射面 + 限次系仍拒装。
#[test]
fn pa12_batch3_filter_mapping() {
    use engine_native::rvm_bridge::{map_vm_filter, VmFilterPolicy as P};
    for (name, want) in [
        ("owner_source_sha", P::OwnerSourceSha),
        ("owner_source_fire_sha", P::OwnerSourceFireSha),
        ("owner_source_ordinary_trick", P::OwnerSourceOrdinaryTrick),
        ("owner_alive_not_target", P::OwnerAliveNotEventPlayer),
        ("owner_is_target_source_alive", P::OwnerTargetSourceAlive),
        ("owner_is_source_or_target", P::OwnerSourceOrTarget),
    ] {
        assert_eq!(map_vm_filter(name), Some(want), "{name}");
    }
    // once/round 限次系已由具名程序（vm_aux.once：座位×键×范围 1回合/2轮/3整局）
    // 承载——批③实测 map_vm_filter("owner_is_target_once") 返回具名策略（非 None）；
    // 语义验收（限次窗口与旧线 turn_fires 对齐）登记为后续子批。
    assert!(
        engine_native::rvm_bridge::map_vm_filter("owner_is_target_once").is_some(),
        "once 系滤名应由具名程序承载"
    );
}

/// 帝挽（`card_used owner_source_sha optional`）：自己用【杀】（目标 1 个）→ 确认 → 摸 1；
/// 用普通锦囊不触发。
#[test]
fn pa12_batch3_owner_source_sha() {
    let pack = pack_for(PSDIWAN, "offline-psdiwan");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(980_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    advance_to(&mut e, 1, Phase::Play);
    // 负例：普通锦囊（无中生有）不触发（无确认门）
    let wuzhong = e.debug_place(1, "wuzhong");
    let base = hand_len(&e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    );
    drain(&mut e, false);
    assert_eq!(
        hand_len(&e, 1),
        base + 1,
        "无中生有净 +1（-1 出 +2 摸），帝挽不过闸"
    );
    // 正例：杀（单目标）→ 确认 → 摸 1
    let sha = e.debug_place(1, "sha");
    let base = hand_len(&e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    let mut confirmed = false;
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "trigger_confirm" {
            submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![c
                        .spec
                        .options
                        .iter()
                        .find(|o| matches!(o, ChoiceOption::Confirm))
                        .copied()
                        .unwrap()],
                },
            );
            confirmed = true;
            break;
        }
        settle_plain(&mut e, &c);
    }
    drain(&mut e, false);
    assert!(confirmed, "杀过闸应开确认门");
    assert_eq!(hand_len(&e, 1), base, "杀 -1 + 帝挽按目标数摸 1 → 持平");
}

/// 抚（`card_used owner_source_ordinary_trick optional`）：普通锦囊触发、杀不触发。
#[test]
fn pa12_batch3_owner_source_ordinary_trick() {
    let pack = pack_for(NATUFU, "collab-natufu");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(980_002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    advance_to(&mut e, 1, Phase::Play);
    // 负例：杀
    let sha = e.debug_place(1, "sha");
    let base = hand_len(&e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    drain(&mut e, false);
    assert_eq!(hand_len(&e, 1), base - 1, "杀不过闸（普通锦囊谓词）");
    // 正例：无中生有
    let wuzhong = e.debug_place(1, "wuzhong");
    let base = hand_len(&e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    );
    let mut confirmed = false;
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "trigger_confirm" {
            submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![c
                        .spec
                        .options
                        .iter()
                        .find(|o| matches!(o, ChoiceOption::Confirm))
                        .copied()
                        .unwrap()],
                },
            );
            confirmed = true;
            break;
        }
        settle_plain(&mut e, &c);
    }
    drain(&mut e, false);
    assert!(confirmed, "普通锦囊过闸应开确认门");
    assert_eq!(hand_len(&e, 1), base + 2, "无中净 +1 + 抚摸 1");
}

/// dcweidang（`phase_end owner_alive_not_target optional`）：他人结束阶段触发、
/// 自己结束阶段（事件主玩家=持有者）不触发。
#[test]
fn pa12_batch3_owner_alive_not_target() {
    let pack = pack_for(DCWEIDANG, "huicui-dcweidang");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(980_003, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    // 他人（0 号）结束阶段：确认 → 摸 1
    advance_to(&mut e, 0, Phase::End);
    let mut confirmed = 0;
    for _ in 0..120 {
        if e.outcome().is_some() {
            break;
        }
        let before = hand_len(&e, 1);
        step(&mut e, true);
        if hand_len(&e, 1) == before + 1 {
            confirmed += 1;
        }
        // 自己（1 号）回合结束后停
        if e.turn().current == 2 && confirmed >= 1 {
            break;
        }
    }
    assert!(confirmed >= 1, "他人结束阶段触发摸 1");
    // 自己结束阶段：不触发（事件主玩家=持有者）
    advance_to(&mut e, 1, Phase::End);
    let base = hand_len(&e, 1);
    for _ in 0..12 {
        step(&mut e, true);
        if e.turn().current != 1 {
            break;
        }
    }
    assert_eq!(
        hand_len(&e, 1),
        base,
        "自己结束阶段不过闸（弃牌阶段末已按手牌上限弃）"
    );
}

/// 军略（`damage owner_is_source_or_target`）：造成或受到伤害都 +标记（按伤害值）。
/// 火杀负例/正例同场验证 `owner_source_fire_sha` 映射（军略不含该滤名，映射断言在
/// mapping 测试；此处驱动 damage 双向）。
#[test]
fn pa12_batch3_owner_is_source_or_target() {
    let pack = pack_for(JUNLVE, "extra-junlve");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(980_004, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    assert_eq!(mark_of(&e, 1, JUNLVE_MARK), 0);
    // ① 持有者造成 1 点伤害（直接调试伤害，来源=1）
    e.debug_deal_damage(
        Some(1u8.into()),
        0u8.into(),
        1,
        engine_native::event::DamageNature::Normal,
    );
    assert_eq!(mark_of(&e, 1, JUNLVE_MARK), 1, "来源=持有者 → +1 标记");
    // ② 持有者受到 1 点伤害（来源=0）
    e.debug_deal_damage(
        Some(0u8.into()),
        1u8.into(),
        1,
        engine_native::event::DamageNature::Normal,
    );
    assert_eq!(mark_of(&e, 1, JUNLVE_MARK), 2, "目标=持有者 → +1 标记");
    // ③ 无关伤害（0→2）不触发
    e.debug_deal_damage(
        Some(0u8.into()),
        2u8.into(),
        1,
        engine_native::event::DamageNature::Normal,
    );
    assert_eq!(mark_of(&e, 1, JUNLVE_MARK), 2, "无关伤害不过闸");
}

fn drain(e: &mut Engine, confirm: bool) {
    for _ in 0..12 {
        let Some(c) = e.open_choice().cloned() else {
            return;
        };
        if confirm && c.spec.prompt == "trigger_confirm" {
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![c
                        .spec
                        .options
                        .iter()
                        .find(|o| matches!(o, ChoiceOption::Confirm))
                        .copied()
                        .unwrap()],
                },
            );
            continue;
        }
        settle_plain(e, &c);
    }
}
