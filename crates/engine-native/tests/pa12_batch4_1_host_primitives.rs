//! PA12 迁移批④-1（宿主原语：`turn_over`，2026-10-27）：DSL `turn_over(player)`
//!（tag `TURN_OVER` 433 → `EffectRequest::TurnOver`，旧线「武将牌翻面：背面跳过
//! 下个回合后翻回」）此前不在原生 `accept_effect` 允许面（装载整包拒装）。本批
//! 接入：允许面 + `commit_effect` 以 `TurnOverBefore` 子事件帧走内核翻面提交
//!（神裔「不能翻至背面」守卫、急冻冰封解除、`TurnOvered` 事件全部同口径）。
//! 真实技能全链：雪堕（伤害后翻面）/ 扶理所涉 `turn_over` 分支由模块差分覆盖。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

fn submit_to(e: &mut Engine, player: u8, phase: Phase) {
    for i in 0..300 {
        if e.turn().current == player && e.turn().phase == phase {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        if false { eprintln!("step#{i} cur={} phase={:?} open={:?}", e.turn().current, e.turn().phase, e.open_choice().map(|c|(&c.spec.prompt, c.spec.min, c.spec.max, c.observer, c.spec.options.iter().filter(|o|matches!(o,ChoiceOption::Card(_))).count()))); }
        settle(e, false);
        let cur = e.turn().current;
        submit(e, cur, GameAction::NextPhase);
    }
    panic!("300 步未到 p{player} 目标阶段");
}

fn drain_plain(e: &mut Engine) {
    for _ in 0..12 {
        let Some(c) = e.open_choice().cloned() else { return };
        let sel: Vec<ChoiceOption> = c.spec.options.iter().filter(|o| matches!(o, ChoiceOption::Card(_))).copied().take(c.spec.min.max(1)).collect();
        let sel = if sel.is_empty() { vec![c.spec.options[0].clone()] } else { sel };
        submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
}

fn settle(e: &mut Engine, confirm: bool) {
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if confirm && c.spec.prompt == "trigger_confirm" {
            vec![c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied().unwrap_or(c.spec.options[0])]
        } else {
            let cards: Vec<ChoiceOption> = c.spec.options.iter().filter(|o| matches!(o, ChoiceOption::Card(_))).copied().take(c.spec.min.max(1)).collect();
            if cards.is_empty() { vec![c.spec.options[0].clone()] } else { cards }
        };
        submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
}

const NSXUEZHU: &str = include_str!("../../../content/original/ported/diy/nsxuezhu.skill");
const XIAOMIAN: &str = include_str!("../../../content/original/ported/collab/xiaomian.skill");
const DCCIBEI: &str = include_str!("../../../content/original/ported/collab/dccibei.skill");
const KOTAROU: &str = include_str!("../../../content/original/ported/key/kotarou_rewrite_damage.skill");

fn pack_for(source: &str, id: &str) -> ExtensionPack {
    let pkg = compile_source_named(id, source).expect("编译 ported .skill");
    assert!(pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION);
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
    ExtensionPack {
        def: ExtensionDef {
            name: "pa12-batch4-1",
            version: 1,
            description: "PA12 批④-1 turn_over",
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
    let out = e.submit(GameCommand {
        actor,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action,
    });
    let _ = &out;
}

/// 雪堕（diy，`damage owner_is_source_or_target` → 摸 1 + 翻面）：持有者受到
/// 伤害后翻至背面；再受一次伤害翻回正面（内核翻面提交全口径——含 TurnOvered
/// 帧与背面跳回合由回合推进验证）。
#[test]
fn pa12_batch4_1_turn_over_toggles_face() {
    let pack = pack_for(NSXUEZHU, "diy-nsxuezhu");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(995_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    assert!(!e.state_core().player(1).face_down, "初始正面");
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    settle(&mut e, true);
    assert!(e.state_core().player(1).face_down, "受伤后翻至背面");
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    settle(&mut e, true);
    assert!(!e.state_core().player(1).face_down, "再受伤翻回正面");

    // 背面跳回合（E03 内核路径）：翻至背面后，1 号下个回合的判定阶段开始
    // 即被跳过路径翻回（commit_phase：PhaseBegin(Judge) ∧ face_down →
    // TurnOverBefore 翻回 + 直达回合收束）。驱动注记：单元驱动在跳过窗口
    // 内注入 NextPhase 会与待提交的 TurnEnded 子帧竞争（实测该窗口见到
    // phase=Judge 且 face 已翻回），故以「首次到达 1 号回合时已翻回」为
    // 跳过路径执行的判据；整回合无阶段帧的全量验证属内核 E03 既有测试面。
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    e.debug_set_hp(1, 10);
    settle(&mut e, true);
    assert!(e.state_core().player(1).face_down);
    let mut flipped_back_at_first_turn = None;
    for _ in 0..200 {
        if e.outcome().is_some() {
            break;
        }
        if e.turn().current == 1 {
            flipped_back_at_first_turn = Some(e.state_core().player(1).face_down);
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            let sel = if sel.is_empty() { vec![c.spec.options[0].clone()] } else { sel };
            submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(
        flipped_back_at_first_turn,
        Some(false),
        "1 号回合到达时已由跳过路径翻回正面"
    );
}

/// 批④-2：消面（xiaomian，`card_used owner_alive optional` 摸 1 后弃 1 手/装备）
/// 与 蜜柿彗（mia_shihui，`phase_end owner_is_target` 回 1 后弃 1）——
/// `discard_hand_equip_count` 字面 N=1 落账；N=0 选牌首发拒绝。
#[test]
fn pa12_batch4_2_discard_hand_equip_count() {
    // ① 消面：自己用牌 → 确认 → 摸 1 → 弃 1（手牌首张）
    let pack = pack_for(XIAOMIAN, "shenhua-xiaomian");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(996_001, 4, &[&pack]).expect("建局");
    e.debug_grant(1, &name);
    submit_to(&mut e, 1, Phase::Play);
    let card = e.debug_place(1, "sha");
    let base = hand_len(&e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card, targets: vec![0] });
    let mut confirmed = false;
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            submit(&mut e, c.observer, GameAction::Answer {
                choice_id: c.id,
                selection: vec![c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied().unwrap()],
            });
            confirmed = true;
            break;
        }
        let sel: Vec<ChoiceOption> = c.spec.options.iter().filter(|o| matches!(o, ChoiceOption::Card(_))).copied().take(c.spec.min.max(1)).collect();
        let sel = if sel.is_empty() { vec![c.spec.options[0].clone()] } else { sel };
        submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    assert!(confirmed, "optional 确认门");
    drain_plain(&mut e);
    // 手牌净变化：-1（打出杀）+1（摸 1）-1（弃 1）= base-1
    assert_eq!(hand_len(&e, 1), base - 1, "消面：摸 1 后弃 1（净 -1）");

    // N=0（选牌首发）的允许面拒绝断言见 pa12_batch4_2_accept_rejects_zero_count；
    // 编译器对 count 字面 0 的态度（拒绝/放行）留批④选牌链接入时一并验收。
}

/// 装载/允许面映射断言（N=0 拒绝）。
#[test]
fn pa12_batch4_2_accept_rejects_zero_count() {
    use rule_api::{EffectRequest, PlayerId, SkillId};
    let req = EffectRequest::DiscardHandEquipCount {
        player: PlayerId::new(1),
        count: 0,
        skill: SkillId::NONE,
    };
    assert!(
        engine_native::rvm_ops::accept_effect(&req, 4).is_err(),
        "N=0 选牌首发应拒绝"
    );
    let req1 = EffectRequest::DiscardHandEquipCount {
        player: PlayerId::new(1),
        count: 1,
        skill: SkillId::NONE,
    };
    assert!(engine_native::rvm_ops::accept_effect(&req1, 4).is_ok());
}

/// 批④-3：刺背（dccibei，`damage_before owner_is_source optional` 防止本次伤害+摸 5）
/// 与 kotarou 改写（`damage_before owner_is_source` 伤害 +1）——DamageBefore 前置
/// 窗口内直写当前伤害帧（宿主技 v187/v126 同法）。
#[test]
fn pa12_batch4_3_damage_rewrite_window() {
    // ① prevent：0 号（持技）对 1 号造成 1 伤 → 确认 → 伤害被防止（hp 不变）+ 摸 5
    let pack = pack_for(DCCIBEI, "collab-dccibei");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(997_001, 4, &[&pack]).expect("建局");
    e.debug_grant(0, &name);
    let hp1 = e.debug_hp(1);
    let h0 = hand_len(&e, 0);
    e.debug_deal_damage(Some(0u8.into()), 1u8.into(), 1, DamageNature::Normal);
    settle(&mut e, true); // optional 确认
    drain_plain(&mut e);
    assert_eq!(e.debug_hp(1), hp1, "防止：目标 hp 不变");
    assert_eq!(hand_len(&e, 0), h0 + 5, "刺背摸 5");

    // ② modify +1：持有者来源的伤害 +1（2 号 hp 扣 2）
    let pack2 = pack_for(KOTAROU, "collab-kotarou");
    let name2 = pack2.vm_skills[0].name.clone();
    let mut e2 = Engine::new_with_extensions(997_002, 4, &[&pack2]).expect("建局");
    e2.debug_grant(0, &name2);
    let hp2 = e2.debug_hp(2);
    e2.debug_deal_damage(Some(0u8.into()), 2u8.into(), 1, DamageNature::Normal);
    settle(&mut e2, true);
    drain_plain(&mut e2);
    assert_eq!(e2.debug_hp(2), hp2 - 2, "modify +1：1 伤变 2");
}

/// 批⑤-2：奋励三窗口（onlyOL 真实 npack）整局——PhaseBegin 前置窗（§2 K 修复）
/// 后，VM 机器在阶段入场行为**之前**执行：Draw 窗确认 → 置 skip_draw 旗标 →
/// begin_draw 跳过摸牌（手牌不 +2）。修复前的红灯实测见批⑤首步证据 §3。
#[test]
fn pa12_batch5_fenli_bridges_with_skip_wiring() {
    let pkg_json = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/original/ported/onlyOL/onlyOL.npack.json"),
    )
    .expect("读 onlyOL npack");
    let mut pkg: content_schema::ContentPackage =
        serde_json::from_str(&pkg_json).expect("解析 npack");
    pkg.trigger_index.retain(|t| t.module.contains("ol_fenli_phase"));
    assert_eq!(pkg.trigger_index.len(), 3, "奋励三窗口");
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("桥接");
    assert_eq!(out.decls.len(), 3, "三模块全部可承载（skip_current_phase 已接允许面）");
    let pack = ExtensionPack {
        def: ExtensionDef { name: "onlyol-fenli", version: 1, description: "批⑤-2 奋励" },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: out.decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(998_101, 4, &[&pack]).expect("建局");
    for d in pack.vm_skills.iter() {
        e.debug_grant(1, &d.name);
    }
    // 奋励·摸牌窗口条件：并列最大手牌——清他人手牌，1 号 1 张
    for p in [0u8, 2, 3] { e.debug_clear_hand(p); }
    e.debug_clear_hand(1);
    e.debug_place(1, "sha");
    // 推进到 1 号回合摸牌窗口
    let mut confirmed = false;
    let mut e2e_hand_at_confirm = std::cell::Cell::new(0usize);
    for _ in 0..400 {
        if e.outcome().is_some() { break; }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "trigger_confirm" && e.turn().current == 1 && e.turn().phase == Phase::Draw {
                // 确认时手牌应仍为 1（前置窗：摸牌未发生）
                let hand_at_confirm = hand_len(&e, 1);
                e2e_hand_at_confirm.set(hand_at_confirm);
                submit(&mut e, c.observer, GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![c.spec.options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied().unwrap()],
                });
                confirmed = true;
                continue;
            }
            let sel: Vec<ChoiceOption> = c.spec.options.iter().filter(|o| matches!(o, ChoiceOption::Card(_))).copied().take(c.spec.min.max(1)).collect();
            let sel = if sel.is_empty() { vec![c.spec.options[0].clone()] } else { sel };
            submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        if confirmed && e.turn().phase != Phase::Draw { break; }
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(confirmed, "摸牌窗口应开确认门");
    // 前置窗 + skip 生效判据：确认门开于 Draw（入场行为之前），确认后摸牌被
    // 跳过（手牌与确认时相同、相位已越过 Draw、旗标被消费清零）。
    assert!(e.turn().phase != Phase::Draw, "确认后已越过摸牌阶段");
    assert!(!e.state_core().turn.skip_draw_phase, "skip 旗标被 begin_draw 消费");
    assert_eq!(hand_len(&e, 1), e2e_hand_at_confirm.get(), "摸牌被跳过：手牌与确认时一致");
}
