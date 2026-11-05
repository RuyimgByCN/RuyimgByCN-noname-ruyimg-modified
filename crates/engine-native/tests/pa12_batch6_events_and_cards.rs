//! PA12 迁移批⑥（2026-11-05，零版本号变更）：
//! 1. `phase_any_end`（§2 B，ported/**/*.skill 80 处）：`content-native-loader`
//!    映射 `"phase_any_end" => EventKind::PhaseEnd`（无阶段门，任一阶段结束触发），
//!    `trigger_ctx_of` 为 `Phase`/`TurnBegin`/`TurnEndWindow` 补填 `event_source`。
//! 2. `trigger_ctx_of` 四事件臂补齐（`CardsDrawn`/`Recover`/`LoseHp`/`DyingSaved`）：
//!    此前装载器已映射 `cards_drawn`/`hp_recovered`/`hp_lost`/`dying_after` 的
//!    `EventKind`，但 `trigger_ctx_of` 落入默认空臂（`event_player=None`、
//!    `event_amount=0`），导致 `owner_is_target*`、`owner_gained_*` 恒假且模块体
//!    内 `event_player`/`event_amount` 失真。
//! 3. `EngineVmHost::event_cards`（§2 I，`HOST_EVENT_CARD_COUNT`）：此前恒读默认
//!    空切片=0，现从 `CardsDrawn`/`CardsMoved`/`CardsDiscarded`/`CardsGiven`/
//!    `CardUsed`/`CardResponded` 派生（内存瞬态，不改存档快照与 canonical_hash）。
//!
//! 全链验证全部采用 `content/original/ported/` 真实 `.skill` 源码 →
//! `content_compiler::compile_source_named` → `content_native_loader::bridge_vm_skill_decls`
//! → `Engine::new_with_extensions` 整局。

use content_compiler::compile_source_named;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};

const DCMOULI: &str = include_str!("../../../content/original/ported/huicui/dcmouli.skill");
const DCGANGJIAN: &str = include_str!("../../../content/original/ported/huicui/dcgangjian.skill");
const PEERSHENG: &str = include_str!("../../../content/original/ported/offline/peersheng.skill");
const HAOXUE: &str = include_str!("../../../content/original/ported/collab/haoxue.skill");
const DCJIEXING: &str = include_str!("../../../content/original/ported/xianding/dcjiexing.skill");
const DCGANGU: &str = include_str!("../../../content/original/ported/huicui/dcgangu.skill");
const KUNFEN: &str = include_str!("../../../content/original/ported/key/yuzuru_kunfen.skill");
const XINJUXIANG: &str = include_str!("../../../content/original/ported/shiji/xinjuxiang.skill");
const MINGZHE: &str = include_str!("../../../content/original/ported/sp/mingzhe.skill");

fn pack_for(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let pkg = compile_source_named(*id, source).expect("编译真实 ported .skill");
        assert!(
            pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
            "批⑥不引入新 ABI：{} > {}",
            pkg.manifest.rule_vm_abi_version,
            rule_vm::RULE_VM_ABI_VERSION
        );
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑥ 事件与卡牌宿主桥接",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
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

fn drain(e: &mut Engine, confirm: bool) -> usize {
    let mut gates = 0;
    for _ in 0..20 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "trigger_confirm" {
            gates += 1;
            let selection = if confirm {
                vec![ChoiceOption::Confirm]
            } else {
                Vec::new()
            };
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection,
                },
            );
            continue;
        }
        if let Some(pass) = c
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Pass))
            .copied()
        {
            submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![pass],
                },
            );
            continue;
        }
        let sel: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(c.spec.min.max(1))
            .collect();
        let sel = if sel.is_empty() {
            vec![c.spec.options[0]]
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
    gates
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 1. `phase_any_end`（§2 B）：
/// - `dcmouli`（`phase_any_end owner_is_target` → 回 1 + 上限 +1）：在持有者本人
///   阶段结束（如 Judge→Draw 交界）即触发，他人阶段结束不触发；
/// - `dcgangjian`（`phase_any_end owner_alive_not_target` → 摸 1）：在他人阶段
///   结束时触发，自己阶段结束不触发。
#[test]
fn pa12_batch6_phase_any_end_owner_and_non_owner_gates() {
    let pack = pack_for(
        &[("huicui-dcmouli", DCMOULI), ("huicui-dcgangjian", DCGANGJIAN)],
        "pa12-batch6-phase-any-end",
    );
    let mut e = Engine::new_with_extensions(996_101, 4, &[&pack]).expect("建局");
    // 1 号持 dcmouli（owner_is_target），2 号持 dcgangjian（owner_alive_not_target）
    e.debug_grant(1, "dcmouli");
    e.debug_grant(2, "dcgangjian");
    e.debug_set_hp(1, 2);
    let max1_0 = e.state_core().player(1).max_hp;
    let h2_0 = hand_len(&e, 2);

    // 当前为 0 号回合（Judge 阶段）：推进一个阶段触发 0 号的 PhaseEnd(Judge)
    assert_eq!(e.turn().current, 0);
    submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e, true);
    // 0 号阶段结束：1 号（owner_is_target）不触发；2 号（owner_alive_not_target）摸 1
    assert_eq!(
        (e.debug_hp(1), e.state_core().player(1).max_hp),
        (2, max1_0),
        "他人阶段结束不触发 dcmouli（owner_is_target）"
    );
    assert!(
        hand_len(&e, 2) > h2_0,
        "他人阶段结束触发 dcgangjian（owner_alive_not_target）摸牌"
    );

    // 推进到 1 号回合，并在 1 号阶段结束时验证 dcmouli 触发
    while e.turn().current != 1 {
        drain(&mut e, false);
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let max1_before = e.state_core().player(1).max_hp;
    submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e, true);
    assert!(
        e.state_core().player(1).max_hp > max1_before,
        "1 号自身阶段结束触发 dcmouli（体力上限增加）"
    );
}

/// 2. `cards_drawn`（`EventData::CardsDrawn` + `Atom::Gained`）：
/// - `peersheng`（`cards_drawn owner_alive_not_target optional` → 摸 1）：他人摸牌时
///   开确认门摸 1，自己摸牌时不触发；
/// - `haoxue`（`cards_drawn owner_gained_cards_during_play_phase_wounded` → 回 1）：
///   出牌阶段内已受伤持有者摸牌（如使用无中生有）后回复 1 点体力。
#[test]
fn pa12_batch6_cards_drawn_owner_is_target_and_gained_atom() {
    let pack = pack_for(
        &[("offline-peersheng", PEERSHENG), ("collab-haoxue", HAOXUE)],
        "pa12-batch6-cards-drawn",
    );
    let mut e = Engine::new_with_extensions(996_102, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "peersheng");
    e.debug_grant(0, "haoxue");
    e.debug_set_hp(0, 2); // 0 号已受伤（2/4）

    // 0 号回合进入出牌阶段：0 号摸牌阶段摸牌时，1 号 peersheng 放弃确认；0 号 haoxue 因非出牌阶段不回血
    let h1_init = hand_len(&e, 1);
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        drain(&mut e, false);
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    drain(&mut e, false);
    assert_eq!(hand_len(&e, 1), h1_init, "放弃确认时 1 号 peersheng 不摸牌");
    assert_eq!(e.debug_hp(0), 2, "摸牌阶段非出牌阶段，haoxue 不触发");

    // 0 号出牌阶段使用【无中生有】摸 2 张 →
    //   ① 0 号 haoxue（出牌阶段+已受伤+Gained）回复 1 体力；
    //   ② 1 号 peersheng（cards_drawn owner_alive_not_target optional）确认后摸 1 张（1 号自己因此次摸牌不再递归触发自身 peersheng）。
    let wuzhong = e.debug_place(0, "wuzhong");
    let h1_before = hand_len(&e, 1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    );
    let gates = drain(&mut e, true);
    assert_eq!(
        e.debug_hp(0),
        3,
        "0 号出牌阶段摸牌触发 haoxue（CardsDrawn + Atom::Gained）回复 1 体力"
    );
    assert_eq!(
        gates, 1,
        "他人（0 号）摸牌触发 1 号 peersheng 恰好 1 次确认门（1 号自身摸牌不触发 owner_alive_not_target）"
    );
    assert_eq!(
        hand_len(&e, 1),
        h1_before + 1,
        "1 号确认 peersheng 后摸 1 张"
    );
}

/// 3. `hp_recovered` 与 `hp_lost`（`EventData::Recover` / `EventData::LoseHp`）：
/// - `dcjiexing`（三触发段：`damage`/`hp_recovered`/`hp_lost` + `owner_is_target optional` → 摸 1）：
///   他人回血/失体不开门；持有者自身回血与失体均开确认门并摸 1；
/// - `dcgangu`（`hp_lost owner_alive_not_target_once` → 摸 3 + 失 1 体力）：
///   他人失体时触发摸 3 并失 1 体力，且 `once` 阻止自身失体递归重入。
#[test]
fn pa12_batch6_hp_recovered_and_hp_lost_triggers() {
    let pack = pack_for(
        &[
            ("xianding-dcjiexing", DCJIEXING),
            ("huicui-dcgangu", DCGANGU),
            ("key-kunfen", KUNFEN),
        ],
        "pa12-batch6-hp-events",
    );
    let mut e = Engine::new_with_extensions(996_103, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "dcjiexing");
    e.debug_grant(2, "dcgangu");
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 4);

    // ① 他人（0 号）回复体力：1 号 dcjiexing（owner_is_target）不开门
    e.debug_recover(0, 1);
    assert_eq!(drain(&mut e, true), 0, "他人回复体力不触发 1 号 dcjiexing");

    // ② 持有者（1 号）回复体力：1 号 dcjiexing 开确认门，确认后摸 1
    let h1_before = hand_len(&e, 1);
    e.debug_recover(1, 1);
    assert_eq!(drain(&mut e, true), 1, "1 号自身回复体力触发 dcjiexing 确认门");
    assert_eq!(hand_len(&e, 1), h1_before + 1, "dcjiexing 回复体力分支摸 1");

    // ③ 1 号获授 yuzuru_kunfen（结束阶段失去 1 点体力 + 摸 2）：
    // 推进到 1 号结束阶段 → 1 号 LoseHp(1) → 同时触发：
    //   - 1 号 dcjiexing（hp_lost owner_is_target optional）确认摸 1
    //   - 2 号 dcgangu（hp_lost owner_alive_not_target_once）锁定摸 3 + 失 1 体力（不递归）
    e.debug_grant(1, "yuzuru_kunfen");
    let h2_before = hand_len(&e, 2);
    let hp2_before = e.debug_hp(2);
    while !(e.turn().current == 1 && e.turn().phase == Phase::End) {
        drain(&mut e, true);
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let gates = drain(&mut e, true);
    assert!(gates >= 1, "1 号失去体力应触发 dcjiexing 的 hp_lost 确认门");
    assert_eq!(
        hand_len(&e, 2),
        h2_before + 3,
        "他人（1 号）失去体力触发 2 号 dcgangu 摸 3 张"
    );
    assert_eq!(
        e.debug_hp(2),
        hp2_before - 1,
        "dcgangu 自身失去 1 点体力且 once 阻止递归重入"
    );
}

/// 4. `dying_after`（`EventData::DyingSaved`）：
/// `xinjuxiang`（`dying_after owner_alive_not_target optional` → 移除自身技能 +
/// 对 `event_player` 造成 1 点伤害）：他人濒死被桃救回后，持有者可发动拒降移除本技
/// 并对该脱离濒死的角色造成 1 点伤害。
#[test]
fn pa12_batch6_dying_after_xinjuxiang_damages_saved_player_and_removes_self() {
    let pack = pack_for(&[("shiji-xinjuxiang", XINJUXIANG)], "pa12-batch6-dying-after");
    let mut e = Engine::new_with_extensions(996_104, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "xinjuxiang");
    e.debug_set_hp(2, 1);
    let tao = e.debug_place(2, "tao");

    // 0 号对 2 号造成 1 点伤害 → 2 号进入濒死 → 2 号用桃自救（体力回 1，发 DyingSaved{player:2}）
    // → 1 号 xinjuxiang（dying_after owner_alive_not_target optional）开确认门
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    let mut confirmed_juxiang = false;
    for _ in 0..20 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "dying_save" {
            submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(tao)],
                },
            );
            continue;
        }
        if c.spec.prompt == "trigger_confirm" && c.observer == 1 {
            confirmed_juxiang = true;
            let confirm = c
                .spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Confirm))
                .copied()
                .unwrap();
            submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![confirm],
                },
            );
            continue;
        }
        // 第二次濒死（拒降造成的 1 点伤害）放弃求桃或结算普通询问
        let cancel_or_first = c
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Pass))
            .copied()
            .unwrap_or(c.spec.options[0]);
        submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![cancel_or_first],
            },
        );
    }
    assert!(confirmed_juxiang, "2 号脱离濒死后应触发 1 号 xinjuxiang 确认门");
    assert!(
        !e.debug_has_skill(1, "xinjuxiang"),
        "限定技 xinjuxiang 发动后 remove_skill(self_skill) 应移除自身"
    );
    assert!(
        !e.debug_alive(2) || e.debug_hp(2) <= 0,
        "xinjuxiang 对 event_player（2 号）造成 1 点伤害生效"
    );
}

/// 5. `HOST_EVENT_CARD_COUNT` / `EngineVmHost::event_cards`（§2 I）：
/// `mingzhe`（`cards_moved owner_is_target` → `if event_card_count > 0 && in_play_phase() == 0 { draw 1 }`）：
/// - 出牌阶段外（如 End 阶段）发生 `CardsMoved`（1 号收到 1 张牌）：`event_card_count == 1 > 0`
///   且 `in_play_phase() == 0` → 明哲触发摸 1 张牌；
/// - 出牌阶段内（Play 阶段）发生 `CardsMoved`：`in_play_phase() == 1` → 不摸牌。
#[test]
fn pa12_batch6_event_card_count_mingzhe_outside_vs_inside_play_phase() {
    let pack = pack_for(&[("sp-mingzhe", MINGZHE)], "pa12-batch6-event-cards");
    let mut e = Engine::new_with_extensions(996_105, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "mingzhe");

    // ① 出牌阶段内（0 号 Play 阶段）：0 号给 1 号 1 张手牌 → in_play_phase()==1 → 不触发明哲额外摸牌
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        drain(&mut e, false);
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let c_play = e.debug_place(0, "sha");
    let h1_play = hand_len(&e, 1);
    e.debug_give_hand_card(0, 1, c_play);
    drain(&mut e, true);
    assert_eq!(
        hand_len(&e, 1),
        h1_play + 1,
        "出牌阶段内 CardsMoved 仅得所给 1 张，明哲不额外摸牌"
    );

    // ② 出牌阶段外（0 号 End 阶段）：0 号给 1 号 1 张手牌 → event_card_count==1 && in_play_phase()==0 → 明哲摸 1
    while !(e.turn().current == 0 && e.turn().phase == Phase::End) {
        drain(&mut e, false);
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let c_end = e.debug_place(0, "sha");
    let h1_end = hand_len(&e, 1);
    e.debug_give_hand_card(0, 1, c_end);
    drain(&mut e, true);
    assert_eq!(
        hand_len(&e, 1),
        h1_end + 2,
        "出牌阶段外 CardsMoved（event_card_count=1 > 0）：得牌 +1 且明哲触发摸 1（共 +2）"
    );
}
