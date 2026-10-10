//! 第 17 轮独立复核探针：验证本轮修复关闭的反例与仍开放项。
//! 独立可执行文件，链接本轮 libengine_native；不计入工程测试。

use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::content::extension::{example_extension, ExtensionDef, ExtensionRegistry};
use engine_native::state::GameMode;
use engine_native::view::PolicyInput;
use engine_native::view::SeatPolicy;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot, Phase,
    RejectReason,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn play_ready(seed: u64, seats: u8) -> Engine {
    let mut e = Engine::new(seed, seats as usize);
    for _ in 0..12 {
        if e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        assert!(!matches!(submit(&mut e, actor, GameAction::NextPhase), CommandOutcome::Rejected(_)));
    }
    for p in 0..seats {
        e.debug_clear_hand(p);
    }
    e
}

struct Passive;
impl SeatPolicy for Passive {
    fn act(&mut self, _: &PolicyInput) -> Option<GameAction> { None }
}

fn main() {
    // ---- 1) 字节存档 v2：ChoiceKind 显式编码（原 nullify/jiji 反例） ----
    for (name, prompt) in [("duel", "duel_sha"), ("savage", "aoe_response")] {
        let mut e = play_ready(2401, 3);
        e.debug_place(1, "sha");
        let card = e.debug_place(0, name);
        let targets = if name == "savage" { vec![] } else { vec![1] };
        submit(&mut e, 0, GameAction::PlayCard { card, targets });
        assert_eq!(e.open_choice().unwrap().spec.prompt, prompt);
        let decoded = decode_archive(&encode_archive(&e.save())).unwrap();
        Engine::restore(&decoded).expect("byte roundtrip");
        println!("FIXED_ARCHIVE_CHOICE: {prompt} byte roundtrip restores");
    }
    // 使用时无懈窗口（v1 曾还原为 RespondCard → OpenChoiceMismatch）
    {
        let mut e = play_ready(2402, 3);
        e.debug_place(2, "nullify");
        let le = e.debug_place(0, "le");
        submit(&mut e, 0, GameAction::PlayCard { card: le, targets: vec![1] });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "nullify_window");
        let decoded = decode_archive(&encode_archive(&e.save())).unwrap();
        Engine::restore(&decoded).expect("nullify byte roundtrip");
        println!("FIXED_ARCHIVE_NULLIFY: nullify_window byte roundtrip restores");
    }
    // v1 字节被版本门禁拒绝
    {
        let e = Engine::new(1, 3);
        let mut bytes = encode_archive(&e.save());
        bytes[4] = 1;
        bytes[5] = 0;
        match decode_archive(&bytes) {
            Err(engine_native::archive::ArchiveError::UnsupportedFormatVersion(1)) => {}
            other => panic!("v1 gating: {other:?}"),
        }
        println!("FIXED_ARCHIVE_V2: v1 bytes rejected by format gate");
    }

    // ---- 2) N04-f 牌归属完整核算 ----
    {
        let e = Engine::new(443, 4);
        let mut snapshot = e.save();
        let highest = e.catalog().cards.len() as u32 - 1;
        for p in &mut snapshot.core.players {
            p.hand.retain(|c| *c != highest);
            p.equips.retain(|c| *c != highest);
            p.judge.retain(|c| *c != highest);
        }
        snapshot.core.deck.retain(|c| *c != highest);
        snapshot.core.discard.retain(|c| *c != highest);
        snapshot.core.table.retain(|c| *c != highest);
        assert!(Engine::restore(&snapshot).is_err());
        println!("FIXED_OWNERSHIP: trailing Card({highest}) absent, restore rejected");
        let mut snapshot = Engine::new(444, 3).save();
        for p in &mut snapshot.core.players {
            p.hand.clear();
            p.equips.clear();
            p.judge.clear();
        }
        snapshot.core.deck.clear();
        snapshot.core.discard.clear();
        snapshot.core.table.clear();
        assert!(Engine::restore(&snapshot).is_err());
        println!("FIXED_OWNERSHIP: all-cards-missing, restore rejected");
    }

    // ---- 3) N10 命令序号：规划克隆模拟序号与高水位统一 ----
    {
        let mut e = play_ready(2601, 3);
        e.debug_place(0, "sha");
        let low = e.plan_action(0, &BoundedPlanner::new(64));
        let mut snapshot = e.save();
        snapshot.last_dedup = 2_000_000;
        let high = Engine::restore(&snapshot).unwrap();
        let high_plan = high.plan_action(0, &BoundedPlanner::new(64));
        assert!(matches!(low.action, GameAction::PlayCard { .. }));
        assert!(matches!(high_plan.action, GameAction::PlayCard { .. }));
        println!("FIXED_AI_SIM_DEDUP: high-water 2M plan simulates Sha (not NextPhase)");

        let mut snapshot = e.save();
        snapshot.last_dedup = u64::MAX;
        let mut ex = Engine::restore(&snapshot).unwrap();
        let out = ex.run_policy(0, &mut Passive);
        assert!(matches!(out, CommandOutcome::Rejected(RejectReason::DuplicateCommand)));
        println!("FIXED_AI_DEDUP_EXHAUST: u64::MAX clean DuplicateCommand (no overflow)");
    }

    // ---- 4) 护驾逐人询问（ADR 0776）----
    {
        // 首位魏将 Pass → 第二位被问（旧反例：一位结束）
        let mut e = play_ready(2601, 4);
        e.debug_grant(1, "护驾");
        e.debug_set_general(2, "夏侯惇");
        e.debug_set_general(3, "张辽");
        e.debug_place(2, "shan");
        let sha = e.debug_place(0, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let c = e.open_choice().unwrap();
        let cid = c.id;
        assert_eq!((c.spec.prompt, c.observer), ("hujia_respond", 2));
        drop(c);
        submit(&mut e, 2, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
        let c2 = e.open_choice().expect("second helper asked");
        assert_eq!((c2.spec.prompt, c2.observer), ("hujia_respond", 3));
        println!("FIXED_HUJIA_QUEUE: first Pass continues to second Wei helper");

        // 挂起等待态恢复（旧反例：holder 等待态 OpenChoiceMismatch）
        let decoded = decode_archive(&encode_archive(&e.save())).unwrap();
        Engine::restore(&decoded).expect("hujia wait-state restore");
        println!("FIXED_HUJIA_RESTORE: holder wait state restores (memory+byte)");

        // 无效 holder 越界拒绝（旧反例：254 被接受）
        let mut snapshot = e.save();
        let mut changed = false;
        for park in &mut snapshot.resolution.parks {
            if let MachineSnapshot::Sha { hujia_holder, .. } = &mut park.machine {
                if hujia_holder.is_some() {
                    *hujia_holder = Some(254);
                    changed = true;
                }
            }
        }
        assert!(changed);
        assert!(Engine::restore(&snapshot).is_err());
        println!("FIXED_HUJIA_BOUNDS: holder=254 rejected");

        // 公开询问不泄密：互换暗闪，攻击者视图与询问序列相同
        let build = |seed: u64| -> Engine {
            let mut e = play_ready(seed, 4);
            e.debug_grant(1, "护驾");
            e.debug_set_general(2, "夏侯惇");
            e.debug_set_general(3, "张辽");
            e
        };
        let mut a = build(2602);
        let _s2 = a.debug_place(2, "shan");
        let _h3 = a.debug_place(3, "sha");
        let mut b = build(2602);
        let _h2 = b.debug_place(2, "sha");
        let _s3 = b.debug_place(3, "shan");
        let sha_a = a.debug_place(0, "sha");
        let sha_b = b.debug_place(0, "sha");
        assert_eq!(a.view_for(0), b.view_for(0), "attacker views equal");
        submit(&mut a, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
        submit(&mut b, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] });
        let ca = a.open_choice().unwrap();
        let cb = b.open_choice().unwrap();
        assert_eq!((ca.observer, ca.spec.prompt), (cb.observer, cb.spec.prompt));
        println!("FIXED_HUJIA_LEAK: swapped hidden shan keeps identical public asks");
    }

    // ---- 5) 激将逐人询问（旧反例：首人拒绝即终止）----
    {
        let mut e = play_ready(2502, 4);
        let mut snap = e.save();
        snap.core.mode = GameMode::Kingdoms;
        for p in snap.core.players.iter_mut() { p.revealed_generals = [true, false]; }
        snap.core.players[0].general_slots[0] = Some(22);
        snap.core.players[1].general_slots[0] = Some(5);
        snap.core.players[2].general_slots[0] = Some(5);
        snap.core.players[3].general_slots[0] = Some(2);
        e = Engine::restore(&snap).unwrap();
        e.debug_grant(0, "激将");
        let sha2 = e.debug_place(2, "sha");
        e.debug_place(1, "shan"); // 1 号无杀
        e.debug_set_hp(3, 4);
        let jj = e.catalog().skill_by_name("激将").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: jj });
        let r = e.open_choice().unwrap();
        let rid = r.id;
        assert_eq!((r.spec.prompt, r.observer), ("jijiang_respond", 1));
        drop(r);
        submit(&mut e, 1, GameAction::Answer { choice_id: rid, selection: vec![ChoiceOption::Pass] });
        let r2 = e.open_choice().expect("second shu asked");
        let r2id = r2.id;
        assert_eq!((r2.spec.prompt, r2.observer), ("jijiang_respond", 2));
        drop(r2);
        submit(&mut e, 2, GameAction::Answer { choice_id: r2id, selection: vec![ChoiceOption::Confirm] });
        let t = e.open_choice().expect("target ask");
        let tid = t.id;
        drop(t);
        submit(&mut e, 0, GameAction::Answer { choice_id: tid, selection: vec![ChoiceOption::Player(3)] });
        assert_eq!(e.debug_hp(3), 3);
        assert!(!e.debug_hand(2).contains(&sha2));
        println!("FIXED_JIJIANG_QUEUE: refusal continues; borrowed sha resolves same channel");
    }

    // ---- 6) 主动技阶段门控 + 仁德累计（旧反例：判定阶段可发动）----
    {
        let mut e = Engine::new(982, 3);
        assert_eq!(e.turn().phase, Phase::Judge);
        e.debug_grant(0, "仁德");
        e.debug_place(0, "shan");
        let rid = e.catalog().skill_by_name("仁德").unwrap();
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: rid });
        assert!(matches!(out, CommandOutcome::Rejected(RejectReason::WrongPhase)));
        println!("FIXED_PHASE_GATE: rende in Judge phase rejected WrongPhase");

        // 仁德分次累计 1+1 跨 2 回血（旧反例：第二次发动被拒）
        let mut d = Engine::new(951, 3);
        advance_play(&mut d);
        for p in 0..3 { d.debug_clear_hand(p); }
        d.debug_grant(0, "仁德");
        d.debug_set_hp(0, 3);
        let one = d.debug_place(0, "shan");
        let two = d.debug_place(0, "shan");
        let rid = d.catalog().skill_by_name("仁德").unwrap();
        submit(&mut d, 0, GameAction::ActivateSkill { skill: rid });
        answer(&mut d, vec![ChoiceOption::Card(one)]);
        answer(&mut d, vec![ChoiceOption::Player(1)]);
        assert_eq!(d.debug_hp(0), 3);
        submit(&mut d, 0, GameAction::ActivateSkill { skill: rid });
        answer(&mut d, vec![ChoiceOption::Card(two)]);
        answer(&mut d, vec![ChoiceOption::Player(2)]);
        assert_eq!(d.debug_hp(0), 4, "cumulative 1+1 crosses 2 → recover");
        println!("FIXED_RENDE_CUMULATIVE: 1+1 second activation heals once");
    }

    // ---- 7) 离间性别筛选恢复（旧反例：恢复派生未过滤）----
    {
        let mut e = Engine::new(2501, 4);
        let mut snapshot = e.save();
        for (p, g) in [(1usize, 2u16), (2, 9), (3, 5)] {
            snapshot.core.players[p].general_slots = [Some(g), None];
        }
        e = Engine::restore(&snapshot).unwrap();
        advance_play(&mut e); // 阶段门控（本轮新增）下须在出牌阶段发动
        e.debug_grant(0, "离间");
        let card = e.debug_place(0, "tao");
        let skill = e.catalog().skill_by_name("离间").unwrap();
        submit(&mut e, 0, GameAction::ActivateSkill { skill });
        let choice = e.open_choice().unwrap().clone();
        submit(&mut e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(card)] });
        let decoded = decode_archive(&encode_archive(&e.save())).unwrap();
        Engine::restore(&decoded).expect("lijian gender-filtered ask restores");
        println!("FIXED_LIJIAN_GENDER_RESTORE: gender-filtered targets restore");
    }

    // ---- 8) E02/E03 扩展边界（旧反例：可启用未安装/包内重复放行）----
    {
        let mut registry = ExtensionRegistry::new();
        assert!(registry.enable("not_installed").is_err());
        assert!(!registry.is_enabled("not_installed"));
        println!("FIXED_EXTENSION_VALIDATION: uninstalled enable rejected");

        let ext = example_extension();
        let mut dup = ext.cards.clone();
        dup.push(ext.cards[0].clone());
        assert!(engine_native::content::extension::merge_extension_cards(vec![], &dup).is_err());
        println!("FIXED_EXTENSION_VALIDATION: duplicate extension cards rejected");

        // 内容锁：带扩展存档不能被纯标准目录恢复
        let packs = vec![&ext];
        let mut e = Engine::new_with_extensions(3001, 3, &packs).unwrap();
        assert_eq!(e.catalog().cards.len(), 81);
        let snap = e.save();
        assert!(Engine::restore(&snap).is_err(), "content lock rejects plain restore");
        Engine::restore_with_extensions(&snap, &packs).expect("same-set restore");
        println!("FIXED_EXTENSION_CONTENT_LOCK: extension set bound into ruleset version");
    }

    // ---- AI 规划信息集（v29 修订）：未知池重分配，守恒且可恢复 ----
    {
        // 原反例 OPEN_AI_SAMPLED_OWNERSHIP：对手持暗牌时采样复制牌堆
        // ID → card_duplicate。现在「对手暗手牌+牌库」作为未知牌池
        // 确定性重分配——只移动不复制。
        let mut e = play_ready(2304, 3);
        e.debug_place(1, "shan");
        let before = e.save();
        let mut sim = e.clone();
        engine_native::ai::redact_opponent_hands(&mut sim, 0);
        let after = sim.save();
        Engine::restore(&after).expect("redacted sim must restore (ownership complete)");
        // 多重集守恒
        let mut ms = |s: &engine_native::StateSnapshot| {
            let mut ids: Vec<u32> = Vec::new();
            for p in &s.core.players {
                ids.extend(p.hand.iter().copied());
                ids.extend(p.equips.iter().copied());
                ids.extend(p.judge.iter().copied());
            }
            ids.extend(s.core.deck.iter().copied());
            ids.extend(s.core.discard.iter().copied());
            ids.extend(s.core.table.iter().copied());
            ids.sort_unstable();
            ids
        };
        assert_eq!(ms(&before), ms(&after), "multiset conserved");
        assert_ne!(before.core.deck, after.core.deck, "unknown pool shuffled");
        println!("FIXED_AI_SAMPLE_OWNERSHIP: pool-reshuffle conserves ownership; restore accepted");

        // 端到端：规划仍产出合法动作并被应用接受
        let plan = e.plan_action(0, &BoundedPlanner::new(64));
        let out = e.apply_plan(&plan).expect("apply");
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
        println!("FIXED_AI_SAMPLE_PIPELINE: plan→apply works with hidden opponent cards");
    }

    // ---- 仍开放项（不 assert，只报告）----
    {
        // 单选候选枚举缺合法闪（P2 仍开放）
        let mut e = play_ready(2303, 3);
        let sha = e.debug_place(0, "sha");
        e.debug_place(1, "shan");
        e.debug_place(2, "sha");
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let plan = e.plan_action(1, &BoundedPlanner::new(64));
        println!("OPEN_AI_CANDIDATES: single-choice plan = {:?}", plan.action);
    }

    println!("ROUND17_PROBES_DONE");
}

fn advance_play(e: &mut Engine) {
    for _ in 0..12 {
        if e.turn().phase == Phase::Play && e.turn().current == 0 {
            return;
        }
        let actor = e.turn().current;
        submit(e, actor, GameAction::NextPhase);
    }
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection });
}
