//! L19 末日乱斗（`末日乱斗/extension.js`）剩余 9 项 `简化近似` 技能修复与 6 维实盘闭环测试。
//!
//! 对照上游：`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js`
//! - `ganran2` (:435)：`enable:'phaseUse'` 已注释，无调用者，不可在出牌阶段主动/自动触发
//! - `_jiangshi2` (:574)：`phaseBegin`（`TurnBegin`）检查自身 `!_tuizhi`、主公 `_tuizhi == 2`、人数倍率概率；清 `_jiangshi_force_nj=0` 并置 `_jiangshi_force_fz=1`，先 `die()` 再改 `zhong`/`zhu.fzjsNumber++`
//! - `_huzhu` (:698)：`phaseUse` 限 1（按出牌阶段独立计数，同回合多出牌阶段可再发动；无自动触发）、仅主忠且目标主忠、双步可取消、`useCard({name:'tao'}, target)` 虚拟桃链路
//! - `chanceIdentity0` (:1023) & `chanceIdentity10` (:951)：上游注释内部助手机制，拒绝 Play-phase `ActivateSkill`；仅限内部/RVM `useSkill` 逐座向 `next_alive` 赋予 `KingdomIdentity::Declared` 并按存活人数 `> N` 门控续链
//! - `wujinmoshiRE1` (:1869)：不触发于 `Dying`，由敌死 `_wujinmoshiRE` 触发，确认/取消 → `wujinmoshiRE2` 选学敌方技能
//! - `_wujinmoshiRecover` (:1994)：`recoverBefore` 按配置加成 `game.me`（Seat 0），不在 `Phase::End` 无故自愈
//! - `_relifeskill` (:2329)：仅存活 `Lord` 于 `roundStart` 对死者倒数 `zuan_relife`，保留死亡前技能并按 `orig_max` 复活、仅持 `zuan_hudun` 时恢复护盾，并按座位序依次摸 `ceil(level*0.5)`
//! - `_guandieskill` (:2847)：`dieBefore`（`priority:-99`、主公且无 `jingcheng`）来源按 `zuanExp`（`maxexp=20+2*level`）+25 经验、豁免 `charlotte` 技后禁用非白名单技、叛军+2、取消死亡回 `tamaxhp`/`tahp`、清 `hej`/连环/翻面、加 `jingcheng`/`notarget`

use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{decision34_pack9_extension_pack, DECISION34_STORAGE_DISPATCH_KEY};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, CardKind, Suit, TriggerMeta};
use engine_native::content::extension::{ExtensionPack, VmSkillDecl};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::event::{EventKind, Phase, Timing};
use engine_native::machine::LogEvent;
use engine_native::rvm_bridge::{BytecodeModule, VmChoiceActor, VmFilterPolicy};
use engine_native::state::{GameMode, KingdomIdentity, Role, INFINITE_HP};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice};

const L19_DOOMSDAY_SKILL_ID: u32 = 1_926_006_001;

static DEDUP: AtomicU64 = AtomicU64::new(2_900_000);

fn pack() -> ExtensionPack {
    let mut p = decision34_pack9_extension_pack();
    if !p.host_skills.iter().any(|s| s.name == "_tuizhi3") {
        p.host_skills.push(HostSkillDecl {
            name: "_tuizhi3",
            behavior: HostBehavior::L19DoomsdayBatchA,
            presentation_binding: None,
        });
    }
    if !p.host_skills.iter().any(|s| s.name == "lingwu") {
        p.host_skills.push(HostSkillDecl {
            name: "lingwu",
            behavior: HostBehavior::SwdLingwu,
            presentation_binding: None,
        });
    }
    if !p.host_skills.iter().any(|s| s.name == "gw_fengyin_state") {
        p.host_skills.push(HostSkillDecl {
            name: "gw_fengyin_state",
            behavior: HostBehavior::GwentFengyinState,
            presentation_binding: None,
        });
    }
    if !p.host_skills.iter().any(|s| s.name == "zuan_hudun") {
        p.host_skills.push(HostSkillDecl {
            name: "zuan_hudun",
            behavior: HostBehavior::L19DoomsdayBatchA,
            presentation_binding: None,
        });
    }
    if !p
        .host_skills
        .iter()
        .any(|s| s.name == "huanle:monkey")
    {
        p.host_skills.push(HostSkillDecl {
            name: "huanle:monkey",
            behavior: HostBehavior::HuanleMonkeyCard,
            presentation_binding: None,
        });
    }
    for name in ["l19_monkey_1", "l19_monkey_2"] {
        let mut monkey = CardDef::basic(name, CardKind::Monkey);
        monkey.suit = Suit::Diamond;
        monkey.rank = 5;
        monkey.equip_skills = vec!["huanle:monkey"];
        p.cards.push(monkey);
    }
    p
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn is_rejected(out: CommandOutcome) -> bool {
    matches!(out, CommandOutcome::Rejected(_))
}

fn fresh(p: &ExtensionPack, seats: usize, seed: u64, skills_by_seat: &[&[&str]]) -> Engine {
    let mut e = Engine::new_with_extensions(seed, seats, &[p]).expect("build L19 engine");
    for seat in 0..seats as u8 {
        e.state_core_mut().skills.retain(|&(o, _), _| o != seat);
        clear_zone(&mut e, seat);
        e.state_core_mut().player_mut(seat).max_hp = 5;
        e.state_core_mut().player_mut(seat).hp = 5;
    }
    for (seat, sks) in skills_by_seat.iter().enumerate() {
        for &sk in *sks {
            e.debug_grant(seat as u8, sk);
        }
    }
    e
}

fn clear_zone(e: &mut Engine, seat: u8) {
    let h: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    let eq: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    let jd: Vec<u32> = e.state_core_mut().player_mut(seat).judge.drain(..).collect();
    e.state_core_mut()
        .discard
        .extend(h.into_iter().chain(eq).chain(jd));
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, v: u32) {
    if v == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), v);
    }
}

fn has_skill(e: &Engine, p: u8, sk: &str) -> bool {
    e.catalog()
        .skill_by_name(sk)
        .is_some_and(|def| e.state_core().has_skill(p, def))
}

fn card_by(e: &Engine, kind: CardKind) -> u32 {
    let core = e.state_core();
    core.deck
        .iter()
        .chain(core.discard.iter())
        .copied()
        .find(|&id| e.catalog().card_kind(id) == kind)
        .expect("matching card")
}

fn give_kind(e: &mut Engine, seat: u8, kind: CardKind) -> u32 {
    let c = card_by(e, kind);
    e.state_core_mut().deck.retain(|&x| x != c);
    e.state_core_mut().discard.retain(|&x| x != c);
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

fn equip_kind(e: &mut Engine, seat: u8, kind: CardKind) -> u32 {
    let c = card_by(e, kind);
    e.state_core_mut().deck.retain(|&x| x != c);
    e.state_core_mut().discard.retain(|&x| x != c);
    e.state_core_mut().player_mut(seat).equips.push(c);
    if kind == CardKind::Monkey {
        e.debug_grant(seat, "huanle:monkey");
    }
    c
}

fn activate(e: &mut Engine, seat: u8, sk: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(sk).expect("skill def");
    submit(e, seat, GameAction::ActivateSkill { skill: def })
}

fn play(e: &mut Engine, seat: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let out = submit(e, seat, GameAction::PlayCard { card, targets });
    assert!(!is_rejected(out.clone()), "play rejected: {out:?}");
    out
}

fn open(e: &Engine, prompt: &str) -> OpenChoice {
    let c = e.open_choice().cloned().expect("open choice");
    assert_eq!(c.spec.prompt, prompt, "unexpected prompt");
    c
}

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> CommandOutcome {
    let c = open(e, prompt);
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
    assert!(!is_rejected(out.clone()), "answer rejected for {prompt}: {out:?}");
    out
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

fn roundtrip_bytes(e: &Engine, p: &ExtensionPack) -> Engine {
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let dec = decode_archive(&bytes).expect("decode archive");
    let restored = Engine::restore_with_extensions(&dec, &[p]).expect("restore");
    assert_eq!(restored.state_hash(), e.state_hash());
    restored
}

fn snapshot_roundtrip(e: &Engine, p: &ExtensionPack, sel: Vec<ChoiceOption>) {
    if let Some(c) = e.open_choice() {
        let planned = e.plan_action(c.observer, &BoundedPlanner::new(4));
        assert!(
            matches!(planned.action, GameAction::Answer { choice_id, .. } if choice_id == c.id),
            "AI must answer open {} cleanly, got {:?}",
            c.spec.prompt,
            planned.action
        );
        let mut ai_probe = roundtrip_bytes(e, p);
        let ai_out = submit(&mut ai_probe, c.observer, planned.action);
        assert!(
            !is_rejected(ai_out.clone()),
            "AI planned action rejected for {}: {ai_out:?}",
            c.spec.prompt
        );
    }
    let mut restored = roundtrip_bytes(e, p);
    if let Some(c) = restored.open_choice().cloned() {
        let out = submit(
            &mut restored,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
        assert!(!is_rejected(out.clone()), "restored answer rejected: {out:?}");
    }
}

fn advance_to_round(e: &mut Engine, target_seat: u8, prev_seat: u8, cur_round: u32) {
    e.debug_set_turn(prev_seat, Phase::End, cur_round);
    let _ = submit(e, prev_seat, GameAction::NextPhase);
    let _ = target_seat;
}

// =============================================================================
// 1. ganran2（末日乱斗/extension.js:435-475）：enable:'phaseUse' 注释停用且无调用者
// =============================================================================

#[test]
fn l19_fix_ganran2_disabled_in_play_and_no_auto_trigger() {
    let p = pack();
    let mut e = fresh(&p, 4, 19901, &[&["ganran", "ganran2"], &[], &[], &[]]);
    let eq = give_kind(&mut e, 0, CardKind::Weapon);
    e.debug_set_turn(0, Phase::Draw, 1);

    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw -> Play
    assert!(
        e.open_choice().is_none(),
        "ganran2 上游无 trigger，进入出牌阶段不得弹出 ganran2:choose_equip"
    );
    assert!(e.debug_hand(0).contains(&eq));
    assert!(
        is_rejected(activate(&mut e, 0, "ganran2")),
        "ganran2 上游 enable:'phaseUse' 在注释中，出牌阶段不得主动发动"
    );

    let ai = e.plan_action(0, &BoundedPlanner::new(4));
    assert!(!matches!(
        ai.action,
        GameAction::ActivateSkill { skill } if e.catalog().skill_name(skill) == "ganran2"
    ));
}

// =============================================================================
// 2. _jiangshi2（末日乱斗/extension.js:574-591）：主公 _tuizhi==2 + 概率 + die + zhong
// =============================================================================

#[test]
fn l19_fix_jiangshi2_lord_tuizhi_probability_die_and_loyalist() {
    let p = pack();

    // 负例 1：prob == 0 (jisuangailv == 0) 时普通人不死不变
    let mut neg0 = fresh(&p, 4, 19902, &[&[], &["_jiangshi2"], &[], &[]]);
    neg0.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    set_flag(&mut neg0, 0, "_tuizhi", 2);
    set_flag(&mut neg0, 0, "jisuangailv", 0);
    advance_to_round(&mut neg0, 1, 0, 1);
    drain(&mut neg0);
    assert!(neg0.state_core().player(1).alive);
    assert_eq!(flag(&neg0, 1, "_jiangshi2_transformed"), 0);

    // 负例 2：普通人自己计 2、主公仅计 1 时不触发
    let mut neg_self = fresh(&p, 4, 19903, &[&[], &["_jiangshi2"], &[], &[]]);
    neg_self.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    set_flag(&mut neg_self, 0, "_tuizhi", 1);
    set_flag(&mut neg_self, 1, "_tuizhi", 2);
    set_flag(&mut neg_self, 0, "jisuangailv", 3);
    advance_to_round(&mut neg_self, 1, 0, 1);
    drain(&mut neg_self);
    assert!(neg_self.state_core().player(1).alive);
    assert_eq!(flag(&neg_self, 1, "_jiangshi2_transformed"), 0);

    // 正例 1（单持 _jiangshi2 无 _jiangshi，概率=1）：走 player.die() + identity='zhong' + fzjs=0 + zhu.fzjsNumber++
    let mut pos1 = fresh(&p, 4, 19904, &[&[], &["_jiangshi2"], &[], &[]]);
    pos1.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Rebel, Role::Traitor],
    };
    set_flag(&mut pos1, 0, "_tuizhi", 2);
    set_flag(&mut pos1, 0, "jisuangailv", 3); // 3 / (4 - 1) = 1.0
    advance_to_round(&mut pos1, 1, 0, 1);
    drain(&mut pos1);
    assert!(!pos1.state_core().player(1).alive, "无 _jiangshi 时 _jiangshi2 的 player.die() 应真实死亡");
    assert_eq!(
        pos1.state_core().identity_roles().and_then(|r| r.get(1).copied()),
        Some(Role::Loyalist),
        "_jiangshi2 须置 identity='zhong'（Role::Loyalist），不得直接变 Rebel 加 5 上限"
    );
    assert_eq!(flag(&pos1, 1, "_jiangshi_force_fz"), 1);
    assert_eq!(flag(&pos1, 0, "fzjs_number"), 1);
    assert_eq!(pos1.state_core().player(1).max_hp, 5);
    let _ = roundtrip_bytes(&pos1, &p);

    // 正例 2（搭配 _jiangshi）：die() 由 _jiangshi 拦截并按 fzjs=0 变 jiangshifz（Rebel/maxHp=5）
    let mut pos2 = fresh(&p, 4, 19905, &[&[], &["_jiangshi2", "_jiangshi"], &[], &[]]);
    pos2.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Loyalist, Role::Loyalist],
    };
    set_flag(&mut pos2, 0, "_tuizhi", 2);
    set_flag(&mut pos2, 0, "jisuangailv", 3);
    advance_to_round(&mut pos2, 1, 0, 1);
    drain(&mut pos2);
    assert!(pos2.state_core().player(1).alive);
    assert_eq!(
        pos2.state_core().identity_roles().and_then(|r| r.get(1).copied()),
        Some(Role::Rebel)
    );
    assert_eq!(pos2.state_core().player(1).max_hp, 5);
    assert_eq!(flag(&pos2, 0, "fzjs_number"), 1);

    // 7..=12 人局倍率 mult=2（n=8 -> (4*2)/7 - 1 = 1/7 > 0，验证大于 6 人倍率生效）
    let mut pos8 = fresh(
        &p,
        8,
        19906,
        &[&[], &["_jiangshi2"], &[], &[], &[], &[], &[], &[]],
    );
    pos8.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord; 1]
            .into_iter()
            .chain(std::iter::repeat_n(Role::Loyalist, 7))
            .collect(),
    };
    set_flag(&mut pos8, 0, "_tuizhi", 2);
    set_flag(&mut pos8, 0, "jisuangailv", 7); // (7*2)/7 - 1 = 1.0
    set_flag(&mut pos8, 0, "fzjs_number", 1);
    advance_to_round(&mut pos8, 1, 0, 1);
    drain(&mut pos8);
    assert!(!pos8.state_core().player(1).alive);
    assert_eq!(flag(&pos8, 0, "fzjs_number"), 2);
}

#[test]
fn l19_fix_jiangshi2_turn_begin_window_and_force_nj_overwrite() {
    let p = pack();

    // Counterexample 1a (`#68` #1): `_jiangshi2` runs on `EventKind::TurnBegin` (`phaseBegin`, pri 15)
    // BEFORE `Phase::Prepare`, so killing Seat 1 prevents Seat 1's `Phase::Prepare` trigger `洛神`
    // from ever opening a `trigger_confirm` prompt (`extension.js:575`).
    let meta = Decision34MetaProbe::jiangshi2_turn_begin(&p);
    assert_eq!((meta.event, meta.timing, meta.priority), (EventKind::TurnBegin, Timing::Normal, 15));

    let mut win = fresh(&p, 4, 19919, &[&[], &["_jiangshi2", "洛神"], &[], &[]]);
    win.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Loyalist, Role::Loyalist],
    };
    set_flag(&mut win, 0, "_tuizhi", 2);
    set_flag(&mut win, 0, "jisuangailv", 3);
    advance_to_round(&mut win, 1, 0, 1);
    assert!(win.open_choice().is_none(), "TurnBegin 致死后不得再触发 Phase::Prepare 的洛神询问");
    assert!(!win.state_core().player(1).alive);
    assert_eq!(
        win.state_core().identity_roles().and_then(|r| r.get(1).copied()),
        Some(Role::Loyalist)
    );
    assert_eq!(flag(&win, 0, "fzjs_number"), 1);

    // Counterexample 1b (`#68` #1): `_jiangshi2` unconditionally clears `_jiangshi_force_nj` (`fzjs = 0`)
    // before `die()`, so pre-existing `_jiangshi_force_nj == 1` transforms Seat 1 into `jiangshifz`
    // (`Role::Rebel`, `max_hp == 5`), NOT `jiangshinj` (`Role::Traitor`, `max_hp == 3`, `extension.js:588`).
    let mut ov = fresh(&p, 4, 19920, &[&[], &["_jiangshi2", "_jiangshi"], &[], &[]]);
    ov.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Loyalist, Role::Loyalist],
    };
    set_flag(&mut ov, 0, "_tuizhi", 2);
    set_flag(&mut ov, 0, "jisuangailv", 3);
    set_flag(&mut ov, 1, "_jiangshi_force_nj", 1);
    advance_to_round(&mut ov, 1, 0, 1);
    drain(&mut ov);
    assert_eq!(flag(&ov, 1, "_jiangshi_force_nj"), 0, "fzjs=0 必须覆盖既有 _jiangshi_force_nj=1");
    assert!(ov.state_core().player(1).alive);
    assert_eq!(
        ov.state_core().identity_roles().and_then(|r| r.get(1).copied()),
        Some(Role::Rebel)
    );
    assert_eq!(ov.state_core().player(1).max_hp, 5);
    assert_eq!(ov.state_core().player(1).hp, 5);
    assert_eq!(ov.debug_hand(1).len(), 4);
    assert_eq!(flag(&ov, 0, "fzjs_number"), 1);
}

struct Decision34MetaProbe;
impl Decision34MetaProbe {
    fn jiangshi2_turn_begin(p: &ExtensionPack) -> TriggerMeta {
        let e = fresh(p, 4, 19921, &[&[], &["_jiangshi2"], &[], &[]]);
        let def = e.catalog().skill_by_name("_jiangshi2").expect("_jiangshi2");
        e.catalog().skills[def as usize]
            .behavior
            .kind_trigger_meta(EventKind::TurnBegin)
            .expect("_jiangshi2 kind_trigger_meta(TurnBegin)")
    }
}

// =============================================================================
// 3. _huzhu（末日乱斗/extension.js:698-714）：enable phaseUse + 双步可取消 + 虚拟桃 useCard
// =============================================================================

#[test]
fn l19_fix_huzhu_virtual_tao_card_used_and_role_cancel() {
    let p = pack();
    let mut e = fresh(&p, 4, 19907, &[&["_huzhu"], &[], &["_huzhu"], &[]]);
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    let tao0 = give_kind(&mut e, 0, CardKind::Tao);
    let tao2 = give_kind(&mut e, 2, CardKind::Tao);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 2);
    e.debug_set_hp(3, 2);
    // 救援目标与第三人装备猴子（检查 virtual tao 链路）
    equip_kind(&mut e, 1, CardKind::Monkey);
    equip_kind(&mut e, 2, CardKind::Monkey);

    // 进入出牌阶段不得自动触发 _huzhu
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "_huzhu 无 trigger，进入出牌阶段不得自动弹窗");

    // 负例：Rebel Seat 2 不可对受伤角色发动 _huzhu
    e.debug_set_turn(2, Phase::Play, 1);
    assert!(is_rejected(activate(&mut e, 2, "_huzhu")));
    assert!(e.debug_hand(2).contains(&tao2));

    // 取消分支 2：选桃后在 _huzhu:choose_target 点选取消 -> 退还次数、不弃桃、不回血
    e.debug_set_turn(0, Phase::Play, 1);
    activate(&mut e, 0, "_huzhu");
    answer(&mut e, "_huzhu:discard_tao", vec![ChoiceOption::Card(tao0)]);
    let tgt_ask = open(&e, "_huzhu:choose_target");
    assert!(tgt_ask.spec.allow_cancel, "_huzhu:choose_target 在承诺前须允许取消");
    assert!(!tgt_ask.spec.options.contains(&ChoiceOption::Player(2)), "不得包含 Rebel");
    assert!(!tgt_ask.spec.options.contains(&ChoiceOption::Player(3)), "不得包含 Traitor");
    snapshot_roundtrip(&e, &p, Vec::new());
    answer(&mut e, "_huzhu:choose_target", Vec::new());
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&tao0), "第二阶段取消不得弃置手牌桃");
    assert_eq!(e.state_core().player(1).hp, 2);

    // 正例：重新发动 → 弃桃 + 走 CardUsed(Virtual Tao -> target=1) 回复 1 点
    activate(&mut e, 0, "_huzhu");
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Card(tao0)]);
    answer(&mut e, "_huzhu:discard_tao", vec![ChoiceOption::Card(tao0)]);
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Player(1)]);
    answer(&mut e, "_huzhu:choose_target", vec![ChoiceOption::Player(1)]);
    drain(&mut e);

    assert!(!e.debug_hand(0).contains(&tao0));
    assert_eq!(e.state_core().player(1).hp, 3, "目标回复 1 点体力");
    assert!(
        e.log().iter().any(|ev| matches!(
            ev,
            LogEvent::VirtualCardPlayed { kind: CardKind::Tao, player: 0, targets, .. }
                if targets == &vec![1]
        )),
        "互助应经过真实 CardUsed(Tao) 事件链而非绕过用牌直接回血"
    );
    assert!(is_rejected(activate(&mut e, 0, "_huzhu")), "同出牌阶段限一次");
}

#[test]
fn l19_fix_huzhu_twice_across_two_play_phases_same_turn() {
    let p = pack();
    // Counterexample 2 (`#68` #2): `enable: 'phaseUse', usable: 1` (`extension.js:699-700`) —
    // two `Phase::Play` phases in the same Turn 1 (via real `lingwu`) must allow `_huzhu` twice.
    let mut e = fresh(&p, 4, 19922, &[&["_huzhu", "lingwu"], &[], &[], &[]]);
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    e.debug_set_hp(0, 1);
    e.debug_set_hp(1, 2);
    let tao1 = give_kind(&mut e, 0, CardKind::Tao);
    let tao2 = give_kind(&mut e, 0, CardKind::Tao);

    e.debug_set_turn(0, Phase::Play, 1);
    assert!(!is_rejected(activate(&mut e, 0, "_huzhu")));
    answer(&mut e, "_huzhu:discard_tao", vec![ChoiceOption::Card(tao1)]);
    answer(&mut e, "_huzhu:choose_target", vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3);
    assert!(is_rejected(activate(&mut e, 0, "_huzhu")), "同一出牌阶段第 2 次必须被拒");

    // Advance Play -> Discard -> End -> phaseAfter (`lingwu` inserts 2nd Phase::Play in Turn 1).
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!((e.turn().current, e.turn().round, e.turn().phase), (0, 1, Phase::Play));
    assert!(e.state_core().turn.extra_phase_lingwu);

    assert!(!is_rejected(activate(&mut e, 0, "_huzhu")), "同回合第 2 个出牌阶段必须允许再发动");
    answer(&mut e, "_huzhu:discard_tao", vec![ChoiceOption::Card(tao2)]);
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Player(1)]);
    answer(&mut e, "_huzhu:choose_target", vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "两个出牌阶段应各救 1 血 (2 -> 3 -> 4)");
    assert!(e.debug_hand(0).is_empty());
    let tao_uses = e
        .log()
        .iter()
        .filter(|ev| matches!(ev, LogEvent::VirtualCardPlayed { kind: CardKind::Tao, player: 0, .. }))
        .count();
    assert_eq!(tao_uses, 2);
    assert!(is_rejected(activate(&mut e, 0, "_huzhu")), "第 2 个出牌阶段内再次发动仍被拒");
}

// =============================================================================
// 4 & 5. chanceIdentity0 & chanceIdentity10（末日乱斗/extension.js:843-1032）
// =============================================================================

fn vm_chance_pack() -> ExtensionPack {
    let mut p = pack();
    p.host_skills
        .retain(|s| !matches!(s.name, "chanceIdentity0" | "chanceIdentity10"));
    for (idx, name) in ["chanceIdentity0", "chanceIdentity10"].into_iter().enumerate() {
        let mod_name = format!("{name}#l19vmtest");
        let mut code = Vec::with_capacity(14);
        code.push(0x57); // HOST_SKILL
        code.push(0x10); // CONST_I32
        code.extend_from_slice(&(DECISION34_STORAGE_DISPATCH_KEY as i32).to_le_bytes());
        code.push(0x10); // CONST_I32
        code.extend_from_slice(&(L19_DOOMSDAY_SKILL_ID as i32).to_le_bytes());
        code.push(0x5A); // HOST_EFFECT_EXT
        code.extend_from_slice(&649u16.to_le_bytes()); // STORAGE_SET = 649
        code.push(0x01); // HALT
        let module = BytecodeModule {
            logic_hash: BytecodeModule::compute_logic_hash(&mod_name, &code, 0, 1, 0),
            name: mod_name,
            code,
            local_count: 0,
            max_stack: 4,
            entry_pc: 0,
            caps_mask: 1,
        };
        p.vm_skills.push(VmSkillDecl {
            declaration_hash: None,
            name: name.to_string(),
            module,
            trigger: TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            },
            filter: VmFilterPolicy::Declared {
                skill_def_id: L19_DOOMSDAY_SKILL_ID.wrapping_add(idx as u32),
                hand_limit_base_max_hp: false,
                choice_actor: VmChoiceActor::Owner,
                modifiers: Vec::new(),
                inner: Box::new(VmFilterPolicy::OwnerIsTarget),
            },
            r05_identity: None,
            presentation_binding: None,
        });
    }
    p
}

#[test]
fn l19_fix_chance_identity_0_and_10_length_gated_chains() {
    let p = pack();

    // Counterexample 3 (`#68` #3): upstream `末日乱斗/extension.js:843-1032` 处于注释块且无
    // `enable: 'phaseUse'`，真实包 `pack()` 上无论 Prepare 还是 Play `ActivateSkill` 都必须拒绝，
    // 且不得出现在 `legal_actions` / `BoundedPlanner` 候选里。
    let mut neg_play = fresh(&p, 4, 19908, &[&["chanceIdentity0", "chanceIdentity10"], &[], &[], &[]]);
    neg_play.state_core_mut().mode = GameMode::Kingdoms;
    neg_play.debug_set_turn(0, Phase::Play, 1);
    let c0_def = neg_play.catalog().skill_by_name("chanceIdentity0").unwrap();
    let c10_def = neg_play.catalog().skill_by_name("chanceIdentity10").unwrap();
    assert!(!neg_play
        .legal_actions(0)
        .iter()
        .any(|a| matches!(a, GameAction::ActivateSkill { skill } if *skill == c0_def || *skill == c10_def)));
    assert!(is_rejected(activate(&mut neg_play, 0, "chanceIdentity0")));
    assert!(is_rejected(activate(&mut neg_play, 0, "chanceIdentity10")));
    let planned = neg_play.plan_action(0, &BoundedPlanner::new(4));
    assert!(
        !matches!(planned.action, GameAction::ActivateSkill { skill } if skill == c0_def || skill == c10_def),
        "AI 不得主动发动内部助手机制 chanceIdentity*"
    );
    assert_eq!(neg_play.state_core().player(1).kingdom_identity, KingdomIdentity::Unknown);
    advance_to_round(&mut neg_play, 0, 3, 1);
    drain(&mut neg_play);
    assert_eq!(neg_play.state_core().player(1).kingdom_identity, KingdomIdentity::Unknown);

    // 内部 RVM `useSkill` 链式派发实盘闭环（经 `PhaseBegin` VM 桥接触发，不使用 `debug_execute_effect`）：
    let vp = vm_chance_pack();

    // 4 人局（mode: guozhan / GameMode::Kingdoms）：chanceIdentity0 -> 1:yi, 2:er, 3:san, 0:si
    // 同时核对 slot 索引 `idx + 1` 不再与 `chanceIdentity` (`idx = 1`) 发生 `0.max(1) == 1` 碰撞
    let mut e4 = fresh(&vp, 4, 19923, &[&["chanceIdentity0"], &[], &[], &[]]);
    e4.state_core_mut().mode = GameMode::Kingdoms;
    advance_to_round(&mut e4, 0, 3, 1);
    drain(&mut e4);
    assert_eq!(e4.state_core().player(1).kingdom_identity, KingdomIdentity::Declared("yi".into()));
    assert_eq!(e4.state_core().player(2).kingdom_identity, KingdomIdentity::Declared("er".into()));
    assert_eq!(e4.state_core().player(3).kingdom_identity, KingdomIdentity::Declared("san".into()));
    assert_eq!(e4.state_core().player(0).kingdom_identity, KingdomIdentity::Declared("si".into()));
    assert_eq!(flag(&e4, 1, "l19_chance_identity_slot"), 1);
    assert_eq!(flag(&e4, 2, "l19_chance_identity_slot"), 2);
    assert_eq!(flag(&e4, 3, "l19_chance_identity_slot"), 3);
    assert_eq!(flag(&e4, 0, "l19_chance_identity_slot"), 4);
    assert_eq!(flag(&e4, 0, "l19_chance_identity_assigned"), 1);
    assert_eq!(flag(&e4, 1, "l19_chance_identity_assigned"), 2);
    let _ = roundtrip_bytes(&e4, &vp);

    // 11 人局：chanceIdentity10 仅给 1 号设 shiyi，11 > 11 为假 -> 不继续调用 chanceIdentity11
    let mut e11 = fresh(
        &vp,
        11,
        19909,
        &[&["chanceIdentity10"], &[], &[], &[], &[], &[], &[], &[], &[], &[], &[]],
    );
    e11.state_core_mut().mode = GameMode::Kingdoms;
    advance_to_round(&mut e11, 0, 10, 1);
    drain(&mut e11);
    assert_eq!(e11.state_core().player(1).kingdom_identity, KingdomIdentity::Declared("shiyi".into()));
    assert_eq!(flag(&e11, 1, "l19_chance_identity_slot"), 11);
    assert_eq!(e11.state_core().player(2).kingdom_identity, KingdomIdentity::Unknown);

    // 12 人局：chanceIdentity10 给 1 号设 shiyi，12 > 11 为真 -> 续链给 2 号设 shier，3 号仍 Unknown
    let mut e12 = fresh(
        &vp,
        12,
        19910,
        &[&["chanceIdentity10"], &[], &[], &[], &[], &[], &[], &[], &[], &[], &[], &[]],
    );
    e12.state_core_mut().mode = GameMode::Kingdoms;
    advance_to_round(&mut e12, 0, 11, 1);
    drain(&mut e12);
    assert_eq!(e12.state_core().player(1).kingdom_identity, KingdomIdentity::Declared("shiyi".into()));
    assert_eq!(e12.state_core().player(2).kingdom_identity, KingdomIdentity::Declared("shier".into()));
    assert_eq!(flag(&e12, 2, "l19_chance_identity_slot"), 12);
    assert_eq!(e12.state_core().player(3).kingdom_identity, KingdomIdentity::Unknown);

    // 负例：即使在 VM 测试包上也拒绝 Play-phase ActivateSkill
    e4.debug_set_turn(1, Phase::Play, 2);
    assert!(is_rejected(activate(&mut e4, 0, "chanceIdentity0")));
}

// =============================================================================
// 6. wujinmoshiRE1（末日乱斗/extension.js:1833-1925）：敌死确认+学技 / 濒死不复活
// =============================================================================

#[test]
fn l19_fix_wujinmoshi_re1_reward_prompt_cancel_and_no_dying_revive() {
    let p = pack();

    // 负例：me（Seat 0）持 RE1/RE2 濒死时不得复活，真实阵亡
    let mut neg_dying = fresh(
        &p,
        4,
        19911,
        &[
            &["_wujinmoshiRE", "wujinmoshiRE1", "wujinmoshiRE2"],
            &["baozou"],
            &[],
            &[],
        ],
    );
    neg_dying.debug_set_turn(1, Phase::Play, 1);
    neg_dying.debug_set_hp(0, 1);
    let sha_kill_me = give_kind(&mut neg_dying, 1, CardKind::Sha);
    play(&mut neg_dying, 1, sha_kill_me, vec![0]);
    drain(&mut neg_dying);
    assert!(
        !neg_dying.state_core().player(0).alive,
        "me 濒死时 wujinmoshiRE1 不得当作濒死复活技使用"
    );

    // 分支 A（取消）& 分支 B（确定 → wujinmoshiRE2 选择敌方技能）
    let mut e = fresh(
        &p,
        4,
        19912,
        &[
            &["_wujinmoshiRE", "wujinmoshiRE1", "wujinmoshiRE2", "baozou"],
            &["_wujinmoshiRE", "wujinmoshiRE1", "wujinmoshiRE2", "英姿"],
            &["_wujinmoshiRE", "wujinmoshiRE1", "wujinmoshiRE2", "制衡"],
            &[],
        ],
    );
    e.debug_set_turn(0, Phase::Play, 1);

    // 击杀敌方 Seat 1 → 触发 wujinmoshiRE1:confirm → 取消不授技
    e.debug_set_hp(1, 1);
    let sha1 = give_kind(&mut e, 0, CardKind::Sha);
    play(&mut e, 0, sha1, vec![1]);
    let c1 = open(&e, "wujinmoshiRE1:confirm");
    assert_eq!(c1.observer, 0);
    assert!(c1.spec.allow_cancel);
    snapshot_roundtrip(&e, &p, Vec::new());
    answer(&mut e, "wujinmoshiRE1:confirm", Vec::new());
    drain(&mut e);
    assert!(!e.state_core().player(1).alive);
    assert!(!has_skill(&e, 0, "英姿"), "取消后不得获得敌方技能");
    assert_eq!(flag(&e, 0, "_wujinmoshiRE1_declined"), 1, "空选取消也须记录 declined 标志");

    // 击杀敌方 Seat 2 → 确认 wujinmoshiRE1:confirm → wujinmoshiRE2:choose_skill 选学“制衡”
    e.debug_set_hp(2, 1);
    let sha2 = give_kind(&mut e, 0, CardKind::Sha);
    play(&mut e, 0, sha2, vec![2]);
    answer(&mut e, "wujinmoshiRE1:confirm", vec![ChoiceOption::Confirm]);
    let c2 = open(&e, "wujinmoshiRE2:choose_skill");
    assert_eq!(c2.observer, 0);
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, "wujinmoshiRE2:choose_skill", vec![ChoiceOption::Tag(0)]);
    drain(&mut e);
    assert!(!e.state_core().player(2).alive);
    assert!(has_skill(&e, 0, "制衡"), "确认后须从对方身上学会所选技能");
}

// =============================================================================
// 7. _wujinmoshiRecover（末日乱斗/extension.js:1994-2005）：仅 recoverBefore 加配置量，End 不自愈
// =============================================================================

#[test]
fn l19_fix_wujinmoshi_recover_before_hook_only_not_phase_end() {
    let p = pack();
    let mut e = fresh(
        &p,
        4,
        19913,
        &[&["_wujinmoshiRecover"], &["_wujinmoshiRecover"], &[], &[]],
    );
    set_flag(&mut e, 0, "wujin_recover_bonus", 1);
    set_flag(&mut e, 1, "wujin_recover_bonus", 3);

    // 负例 1：Phase::End 没用桃不得无故自愈
    e.debug_set_hp(0, 2);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 2, "结束阶段无回复事件不得无故回复");

    // 正例：game.me (Seat 0) 缺 3 血、bonus=+1 出牌阶段用桃 → 单次 Recover(2)
    e.debug_set_turn(0, Phase::Play, 2);
    let tao0 = give_kind(&mut e, 0, CardKind::Tao);
    let ev_base = e.log().len();
    play(&mut e, 0, tao0, vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 4, "2 + (1 + 1) = 4");
    let rec_events: Vec<u32> = e.log()[ev_base..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::HpRecovered { player: 0, amount, .. } => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(rec_events, vec![2], "必须合并为单次 Recover(2)");

    // 负例 2：bonus == 0 时用桃仅回 1
    set_flag(&mut e, 0, "wujin_recover_bonus", 0);
    e.debug_set_hp(0, 2);
    let tao0_b = give_kind(&mut e, 0, CardKind::Tao);
    play(&mut e, 0, tao0_b, vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3);

    // 负例 3：非 game.me（Seat 1）即便持有技能也不享受 wujinmoshiRecover 加成
    e.debug_set_hp(1, 2);
    e.debug_set_turn(1, Phase::Play, 2);
    let tao1 = give_kind(&mut e, 1, CardKind::Tao);
    play(&mut e, 1, tao1, vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3, "非 Seat 0 仅回 1");

    // 边界：`_wujinmoshiRecoverE` 负增量（RecoverI=1, DXB=0 => delta=-1）可扣减回复至 0；正增量加回
    e.debug_grant(1, "_wujinmoshiRecoverE");
    set_flag(&mut e, 1, "wujinmoshiRecoverI", 1);
    set_flag(&mut e, 1, "wujinmoshiDXB", 0);
    e.debug_set_hp(1, 2);
    let tao1_neg = give_kind(&mut e, 1, CardKind::Tao);
    play(&mut e, 1, tao1_neg, vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 2, "1 + (0 - 1) = 0 恢复量");

    set_flag(&mut e, 1, "wujinmoshiDXB", 20);
    let tao1_pos = give_kind(&mut e, 1, CardKind::Tao);
    play(&mut e, 1, tao1_pos, vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "1 + (2 - 1) = 2 恢复量");
}

// =============================================================================
// 8. _relifeskill（末日乱斗/extension.js:2329-2374）：主公 roundStart 倒数复活
// =============================================================================

#[test]
fn l19_fix_relifeskill_lord_round_start_countdown_and_maxhp_draw() {
    let p = pack();

    // 负例：非主公持有 _relifeskill 时不得于濒死/死亡或 roundStart 触发
    let mut neg = fresh(&p, 4, 19914, &[&[], &["_relifeskill"], &["baozou"], &[]]);
    neg.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    set_flag(&mut neg, 1, "zuan_relife", 1);
    neg.debug_set_hp(1, 1);
    neg.debug_set_turn(2, Phase::Play, 1);
    let s = give_kind(&mut neg, 2, CardKind::Sha);
    play(&mut neg, 2, s, vec![1]);
    drain(&mut neg);
    assert!(!neg.state_core().player(1).alive, "濒死/死亡不得立即复活");
    advance_to_round(&mut neg, 0, 3, 1);
    drain(&mut neg);
    assert!(!neg.state_core().player(1).alive, "主公未持 _relifeskill 时轮始不得复活");

    // 正例（samples.md:1131）：主公持有 _relifeskill，死者 Seat 1 level=10/relife=2/maxhp=6
    let mut e = fresh(&p, 4, 19915, &[&["_relifeskill"], &[], &["baozou"], &[]]);
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    e.state_core_mut().player_mut(1).max_hp = 6;
    e.state_core_mut().player_mut(1).hp = 1;
    set_flag(&mut e, 1, "zuan_level", 10);
    set_flag(&mut e, 1, "_levelskill", 10);
    set_flag(&mut e, 1, "zuan_relife", 2);
    set_flag(&mut e, 1, "zuan_maxhujia", 3);

    e.debug_set_turn(2, Phase::Play, 1);
    let sha = give_kind(&mut e, 2, CardKind::Sha);
    play(&mut e, 2, sha, vec![1]);
    drain(&mut e);
    assert!(!e.state_core().player(1).alive, "死亡瞬间 relife=2 不得触发复活");

    // 下一轮（round 1 -> 2）：倒数 2 -> 1，仍死
    advance_to_round(&mut e, 0, 3, 1);
    drain(&mut e);
    assert!(!e.state_core().player(1).alive);
    assert_eq!(flag(&e, 1, "zuan_relife"), 1);

    // 再下一轮（round 2 -> 3）：倒数 1 -> 0，按原 max_hp=6 复活并摸 ceil(10*0.5)=5，relife 重设 ceil(10*0.4+0.1)=5
    advance_to_round(&mut e, 0, 3, 2);
    drain(&mut e);
    assert!(e.state_core().player(1).alive);
    assert_eq!(e.state_core().player(1).max_hp, 6);
    assert_eq!(e.state_core().player(1).hp, 6);
    assert_eq!(e.debug_hand(1).len(), 5, "ceil(10 * 0.5) == 5");
    assert_eq!(flag(&e, 1, "zuan_level"), 10, "等级保留 10 不被清空为 1");
    assert_eq!(flag(&e, 1, "zuan_relife"), 5, "ceil(10 * 0.4 + 0.1) == 5");
    assert_eq!(flag(&e, 1, "zuan_hudun"), 0, "未持 zuan_hudun 技能时不得凭空造护盾");
    let _ = roundtrip_bytes(&e, &p);
}

#[test]
fn l19_fix_relifeskill_preserves_skills_and_draws_in_seat_order() {
    let p = pack();
    // Counterexample 4 (`#68` #4):
    // - 死者保留死亡前全部技能，复活后 `奸雄` / `英姿` / `zuan_hudun` 必须继续真实生效；
    // - 多名死者同轮复活时，摸牌日志顺序必须按座位升序（先 Seat 1、后 Seat 2），不得受 LIFO 栈颠倒；
    // - 仅真正持有 `zuan_hudun` 的角色才恢复 `zuan_hudun` 护盾。
    let mut e = fresh(
        &p,
        4,
        19924,
        &[
            &["_relifeskill"],
            &["奸雄", "英姿", "zuan_hudun"],
            &["英姿", "奸雄"],
            &["baozou"],
        ],
    );
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    set_flag(&mut e, 1, "zuan_level", 2);
    set_flag(&mut e, 1, "zuan_relife", 1);
    set_flag(&mut e, 1, "zuan_maxhujia", 3);
    set_flag(&mut e, 2, "zuan_level", 4);
    set_flag(&mut e, 2, "zuan_relife", 1);
    set_flag(&mut e, 2, "zuan_maxhujia", 3);
    e.debug_set_hp(1, 1);
    e.debug_set_hp(2, 1);

    e.debug_set_turn(3, Phase::Play, 1);
    let sha2 = give_kind(&mut e, 3, CardKind::Sha);
    play(&mut e, 3, sha2, vec![2]);
    drain(&mut e);
    let sha1 = give_kind(&mut e, 3, CardKind::Sha);
    play(&mut e, 3, sha1, vec![1]);
    drain(&mut e);
    assert!(!e.state_core().player(1).alive && !e.state_core().player(2).alive);
    assert!(has_skill(&e, 1, "奸雄") && has_skill(&e, 1, "英姿") && has_skill(&e, 1, "zuan_hudun"));
    assert!(has_skill(&e, 2, "英姿") && has_skill(&e, 2, "奸雄"));

    let ev_base = e.log().len();
    advance_to_round(&mut e, 0, 3, 1);
    drain(&mut e);
    assert!(e.state_core().player(1).alive && e.state_core().player(2).alive);
    assert!(has_skill(&e, 1, "奸雄") && has_skill(&e, 1, "英姿") && has_skill(&e, 1, "zuan_hudun"));
    assert!(has_skill(&e, 2, "英姿") && !has_skill(&e, 2, "zuan_hudun"));
    assert_eq!(flag(&e, 1, "zuan_hudun"), 3);
    assert_eq!(flag(&e, 2, "zuan_hudun"), 0);
    assert_eq!(e.debug_hand(1).len(), 1, "ceil(2 * 0.5) == 1");
    assert_eq!(e.debug_hand(2).len(), 2, "ceil(4 * 0.5) == 2");

    let draw_order: Vec<u8> = e.log()[ev_base..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::CardsDrawn { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(draw_order, vec![1, 2], "多死者同轮复活必须按座位序先 Seat 1 后 Seat 2 摸牌");

    // 复活后 Seat 2 受伤触发 `奸雄` 真实弹窗并收回伤害牌【杀】
    e.debug_set_turn(0, Phase::Play, 2);
    equip_kind(&mut e, 0, CardKind::HorseMinus);
    let hand2_before = e.debug_hand(2).len();
    let atk = give_kind(&mut e, 0, CardKind::Sha);
    play(&mut e, 0, atk, vec![2]);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" && c.observer == 2 {
            break;
        }
        let pass = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer { choice_id: c.id, selection: pass },
        );
    }
    let jx_prompt = e.open_choice().cloned().expect("复活后奸雄必须弹窗询问");
    assert_eq!(jx_prompt.observer, 2);
    answer(&mut e, &jx_prompt.spec.prompt, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 4);
    assert_eq!(e.debug_hand(2).len(), hand2_before + 1);
    assert!(e.debug_hand(2).contains(&atk));
}

#[test]
fn l19_fix_relifeskill_revokes_discarded_equip_skills_like_zhangba() {
    let p = pack();
    // Counterexample (`#81` #1): Lord holds `_relifeskill` -> `preserves_skills_on_death` preserves
    // innate character skills (`奸雄`), while discarded equipment skills (`丈八蛇矛`) MUST be
    // revoked via `revoke_equip_skills` so the revived player cannot activate `丈八蛇矛` empty-handed.
    let mut e = fresh(
        &p,
        4,
        19926,
        &[&["_relifeskill"], &["奸雄"], &["baozou"], &[]],
    );
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Traitor, Role::Rebel],
    };
    set_flag(&mut e, 1, "zuan_level", 2);
    set_flag(&mut e, 1, "zuan_relife", 1);

    e.debug_set_turn(1, Phase::Play, 1);
    let zhangba = give_kind(&mut e, 1, CardKind::Zhangba);
    play(&mut e, 1, zhangba, vec![]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).equips, vec![zhangba]);
    assert!(has_skill(&e, 1, "丈八蛇矛"));

    e.debug_set_hp(1, 1);
    e.debug_set_turn(2, Phase::Play, 1);
    let sha = give_kind(&mut e, 2, CardKind::Sha);
    play(&mut e, 2, sha, vec![1]);
    drain(&mut e);
    assert!(!e.state_core().player(1).alive);
    assert!(e.state_core().player(1).equips.is_empty());
    assert!(e.state_core().discard.contains(&zhangba));
    assert!(has_skill(&e, 1, "奸雄"), "武将自身技能须保留");
    assert!(!has_skill(&e, 1, "丈八蛇矛"), "弃置装备的 equip_skill 必须随离场撤销");

    advance_to_round(&mut e, 0, 3, 1);
    drain(&mut e);
    assert!(e.state_core().player(1).alive);
    assert!(e.state_core().player(1).equips.is_empty());
    assert!(has_skill(&e, 1, "奸雄"));
    assert!(!has_skill(&e, 1, "丈八蛇矛"));

    e.debug_set_turn(1, Phase::Play, 2);
    let _c1 = give_kind(&mut e, 1, CardKind::Shan);
    let _c2 = give_kind(&mut e, 1, CardKind::Tao);
    let zhangba_def = e.catalog().skill_by_name("丈八蛇矛").expect("丈八蛇矛 skill");
    assert!(!e
        .legal_actions(1)
        .iter()
        .any(|a| matches!(a, GameAction::ActivateSkill { skill } if *skill == zhangba_def)));
    assert!(is_rejected(activate(&mut e, 1, "丈八蛇矛")), "复活后无装备不得发动丈八蛇矛");

    // 验证武将固有技 `奸雄` 在复活后仍真实触发
    e.debug_set_turn(0, Phase::Play, 2);
    let atk = give_kind(&mut e, 0, CardKind::Sha);
    play(&mut e, 0, atk, vec![1]);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" && c.observer == 1 {
            break;
        }
        let pass = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer { choice_id: c.id, selection: pass },
        );
    }
    let jx = e.open_choice().cloned().expect("复活后 Seat 1 奸雄仍应询问");
    assert_eq!(jx.observer, 1);
    answer(&mut e, &jx.spec.prompt, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert!(e.debug_hand(1).contains(&atk));
    let _ = roundtrip_bytes(&e, &p);
}

// =============================================================================
// 9. _guandieskill（末日乱斗/extension.js:2847-2962）：主公 dieBefore 取消死亡与清 hej 加 jingcheng
// =============================================================================

#[test]
fn l19_fix_guandieskill_die_before_lord_cancel_hej_and_jingcheng() {
    let p = pack();

    // 负例 1：非主公（Loyalist）持有 _guandieskill 时濒死/死亡不得触发
    let mut neg = fresh(&p, 4, 19916, &[&[], &["_guandieskill"], &["baozou"], &[]]);
    neg.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    neg.debug_set_hp(1, 1);
    neg.debug_set_turn(2, Phase::Play, 1);
    let sha_n = give_kind(&mut neg, 2, CardKind::Sha);
    play(&mut neg, 2, sha_n, vec![1]);
    drain(&mut neg);
    assert!(!neg.state_core().player(1).alive);
    assert_eq!(flag(&neg, 1, "guan_phase"), 0);
    assert_eq!(flag(&neg, 1, "zuan_panjun"), 0);

    // 正例：主公首次 dieBefore -> 击杀者+25经验、禁用非白名单技、叛军+2、取消死亡恢复 tamaxhp/tahp、
    // 清空 hej 与横置翻面、加 jingcheng/notarget；二次致死因已持 jingcheng 而真实阵亡
    let mut e = fresh(
        &p,
        4,
        19917,
        &[
            &["_guandieskill", "据守", "奸雄", "gw_fengyin_state"],
            &["baozou"],
            &[],
            &[],
        ],
    );
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    set_flag(&mut e, 0, "zuan_tamaxhp", 6);
    set_flag(&mut e, 0, "zuan_tahp", 5);
    e.debug_set_hp(0, 1);
    e.state_core_mut().player_mut(0).linked = true;
    e.state_core_mut().player_mut(0).face_down = true;
    let h0 = give_kind(&mut e, 0, CardKind::Shan);
    let eq0 = equip_kind(&mut e, 0, CardKind::Armor);
    let jd0 = card_by(&e, CardKind::Le);
    e.state_core_mut().deck.retain(|&x| x != jd0);
    e.state_core_mut().discard.retain(|&x| x != jd0);
    e.state_core_mut().player_mut(0).judge.push(jd0);

    let ev_base = e.log().len();
    e.debug_set_turn(1, Phase::Play, 1);
    let sha = give_kind(&mut e, 1, CardKind::Sha);
    play(&mut e, 1, sha, vec![0]);
    drain(&mut e);

    assert!(e.state_core().player(0).alive, "首次主公 dieBefore 须被取消存活");
    assert_eq!(e.state_core().player(0).max_hp, 6, "恢复 tamaxhp（不得凭空变 8）");
    assert_eq!(e.state_core().player(0).hp, 5, "恢复 tahp");
    assert_eq!(flag(&e, 0, "zuan_panjun"), 2);
    assert_eq!(flag(&e, 0, "_guandieskill_jingcheng"), 1);
    assert_eq!(flag(&e, 0, "notarget_active"), 1);
    assert_eq!(flag(&e, 0, "zuan_identity_wang"), 1);
    // Counterexample 5 (`#68` #5): `game.zuanExp(source, 25)` at Lv 1 (`maxexp = 20+2*1 = 22`):
    // 0 + 25 = 25 >= 22 -> Lv 2, exp = 3, draw 1 (if !balance), relife = ceil(2*0.4+0.1)=1,
    // NO +1 max_hp / recover / draw 2 / RPG `制衡`.
    assert_eq!(flag(&e, 1, "zuan_level"), 2);
    assert_eq!(flag(&e, 1, "_levelskill"), 2);
    assert_eq!(flag(&e, 1, "zuan_exp"), 3);
    assert_eq!(flag(&e, 1, "zuan_relife"), 1);
    assert_eq!(e.state_core().player(1).max_hp, 5);
    assert_eq!(e.state_core().player(1).hp, 5);
    assert_eq!(e.debug_hand(1).len(), 1);
    assert!(!has_skill(&e, 1, "制衡"));

    // Counterexample 5 (`#68` #5): `charlotte: true` (`gw_fengyin_state`) 及 `_guandieskill` 须豁免禁用，
    // 非白名单技 `据守`/`奸雄` 被移除；且 `hej` 仅合并为单条 `LogEvent::CardDiscarded`。
    assert!(!has_skill(&e, 0, "据守") && !has_skill(&e, 0, "奸雄"), "非白名单技能被禁用");
    assert!(has_skill(&e, 0, "gw_fengyin_state"), "charlotte: true 技能必须豁免禁用");
    assert!(has_skill(&e, 0, "_guandieskill"));
    let discarded_logs: Vec<&Vec<u32>> = e.log()[ev_base..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::CardDiscarded { player: 0, cards } => Some(cards),
            _ => None,
        })
        .collect();
    assert_eq!(discarded_logs.len(), 1, "hej 清牌应合并为单条 CardDiscarded 日志");
    assert!(discarded_logs[0].contains(&h0) && discarded_logs[0].contains(&eq0) && discarded_logs[0].contains(&jd0));
    assert!(!e.state_core().player(0).linked && !e.state_core().player(0).face_down);
    assert!(e.debug_hand(0).is_empty() && !e.debug_hand(0).contains(&h0));
    assert!(e.state_core().player(0).equips.is_empty() && e.state_core().discard.contains(&eq0));
    assert!(e.state_core().player(0).judge.is_empty() && e.state_core().discard.contains(&jd0));
    let _ = roundtrip_bytes(&e, &p);

    // 已有 jingcheng 后再次被杀，_guandieskill 不再触发，真实死亡
    e.debug_set_hp(0, 1);
    let sha2 = give_kind(&mut e, 1, CardKind::Sha);
    play(&mut e, 1, sha2, vec![0]);
    drain(&mut e);
    assert!(!e.state_core().player(0).alive, "持 jingcheng 后再次致死不应再被 _guandieskill 取消");

    // 无限体力边界：未指定 zuan_tamaxhp 时按 _guanmodeskill:2814 恢复 INFINITE_HP
    let mut e_inf = fresh(&p, 4, 19918, &[&["_guanmodeskill", "_guandieskill"], &["baozou"], &[], &[]]);
    e_inf.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    e_inf.debug_set_hp(0, 1);
    e_inf.debug_set_turn(1, Phase::Play, 1);
    let sha_inf = give_kind(&mut e_inf, 1, CardKind::Sha);
    play(&mut e_inf, 1, sha_inf, vec![0]);
    drain(&mut e_inf);
    assert!(e_inf.state_core().player(0).alive);
    assert_eq!(e_inf.state_core().player(0).max_hp, INFINITE_HP);

    // Counterexample 5b (`#68` #5): Lv 1, `exp = 80` + 25 = 105 -> 单次升级 `if exp >= 22` 得
    // `Lv 2, exp = 83` (不得按旧 `>=100` 涨 max_hp/回血/摸2/授制衡); `zuan_balance == 1` 时摸 0 张。
    let mut e_bal = fresh(&p, 4, 19925, &[&["_guandieskill"], &["baozou"], &[], &[]]);
    e_bal.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    set_flag(&mut e_bal, 1, "zuan_exp", 80);
    set_flag(&mut e_bal, 1, "zuan_balance", 1);
    e_bal.debug_set_hp(0, 1);
    e_bal.debug_set_turn(1, Phase::Play, 1);
    let sha_bal = give_kind(&mut e_bal, 1, CardKind::Sha);
    play(&mut e_bal, 1, sha_bal, vec![0]);
    drain(&mut e_bal);
    assert_eq!(flag(&e_bal, 1, "zuan_level"), 2);
    assert_eq!(flag(&e_bal, 1, "zuan_exp"), 83);
    assert_eq!(flag(&e_bal, 1, "zuan_relife"), 1);
    assert_eq!(e_bal.state_core().player(1).max_hp, 5);
    assert_eq!(e_bal.state_core().player(1).hp, 5);
    assert!(e_bal.debug_hand(1).is_empty(), "zuan_balance=1 升级时不得摸牌");
    assert!(!has_skill(&e_bal, 1, "制衡"));
}
