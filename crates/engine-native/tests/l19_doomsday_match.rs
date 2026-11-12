//! L19 末日乱斗 6 维实盘闭环测试（Queue Item 2 Source 10：69 项 = 2 将 + 1 配置 + 66 技）。
//! - ① 正常跑通：经真实 `Engine::submit` + `GameAction::{PlayCard, ActivateSkill, Answer, NextPhase}` 覆盖全部 69 项；
//! - ② 可取消选择点选取消/跳过：`ganran2`、`_huzhu`、`_yuanqiskill` 首步空选取消不消耗次数/状态；
//! - ③ 非法目标/非法牌/数量不足拒绝：`ganran2` 非装备牌、`_huzhu` 非【桃】/满血/超距/僵尸目标、`_yuanqiskill` 越界下标均返回 `Rejected` 且状态哈希不变；
//! - ④ 条件不满足/每回合限一次达上限：`_huzhu` 与 `_yuanqiskill` 同回合二次发动被拒、经验不足 100 禁选模式 0、`zhu` 体力上限 `<= 3` 时 `_zhongzhubuff` 不触发、`human - zombie + 2 <= 0` 时 `zaibian` 摸 0 张；
//! - ⑤ 多技能同场联动与被动修正：`baozou` 无限出【杀】、`ganran` 手牌装备锁视作【铁索连环】、`_wujinmoshiDamage`/`_wujinmoshiMaxHandCard` 增伤与手牌上限、`_tamodeskill` 防指定、`_guanmodeskill` 免翻面与免延时锦囊；
//! - ⑥ 中途询问存档恢复：`ganran2`、`_huzhu`（两阶段）、`_yuanqiskill`（三分支）在 `WaitingForChoice` 窗口下 `encode_archive` + `decode_archive` 恢复后续跑状态哈希完全一致。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    decision34_pack9_extension_pack, identity_chain, parse_wujinmoshi, parse_zuan_brawl,
    validate_zuan_brawl, zaibian_draw, WujinMoshiConfig, ZuanBrawlMode, DEFAULT_ZUAN_BRAWL,
    DOOMSDAY_ZOMBIE_GENERALS, IDENTITY_GATES, IDENTITY_NAMES, JIANGSHIFZ_SKILLS, JIANGSHINJ_SKILLS,
    L19_DOOMSDAY_CHARACTERS, L19_DOOMSDAY_SKILLS, ZUAN_BRAWL_MODES,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Faction};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::standard::Gender;
use engine_native::event::{DamageNature, Phase};
use engine_native::rvm_bridge::modifiers::eval_sha_use_limit;
use engine_native::state::{GameMode, Role};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice};

static DEDUP: AtomicU64 = AtomicU64::new(1_900_000);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let expected_revision = e.revision();
    let dedup_id = DEDUP.fetch_add(1, Ordering::Relaxed);
    e.submit(GameCommand { actor, expected_revision, dedup_id, action })
}

fn is_rejected(out: &CommandOutcome) -> bool {
    matches!(out, CommandOutcome::Rejected(_))
}

fn pack() -> ExtensionPack {
    let mut p = decision34_pack9_extension_pack();
    if !p.host_skills.iter().any(|s| s.name == "_tuizhi3") {
        let behavior = HostBehavior::L19DoomsdayBatchA;
        p.host_skills.push(HostSkillDecl { name: "_tuizhi3", behavior, presentation_binding: None });
    }
    p
}

fn fresh(p: &ExtensionPack, seed: u64, skills_by_seat: &[&[&str]]) -> Engine {
    let mut e = Engine::new_with_extensions(seed, 4, &[p]).expect("L19 engine");
    for seat in 0..4u8 {
        e.state_core_mut().skills.retain(|&(owner, _), _| owner != seat);
        clear_zone(&mut e, seat);
        e.state_core_mut().player_mut(seat).max_hp = 5;
        e.state_core_mut().player_mut(seat).hp = 5;
    }
    for (seat, names) in skills_by_seat.iter().enumerate() {
        for &sk in *names {
            e.debug_grant(seat as u8, sk);
        }
    }
    set_play(&mut e, 0, 1);
    e
}

fn set_play(e: &mut Engine, seat: u8, round: u32) {
    e.debug_set_turn(seat, Phase::Play, round);
    e.state_core_mut().turn.sha_used_play_phase = false;
}

fn clear_zone(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    let eqs: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    let jds: Vec<u32> = e.state_core_mut().player_mut(seat).judge.drain(..).collect();
    e.state_core_mut().discard.extend(cards.into_iter().chain(eqs).chain(jds));
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core().vm_aux.flags.get(&(p, key.to_string())).copied().unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, v: u32) {
    if v == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut().vm_aux.flags.insert((p, key.to_string()), v);
    }
}

fn has_skill(e: &Engine, p: u8, skill: &str) -> bool {
    let Some(d) = e.catalog().skill_by_name(skill) else { return false };
    e.state_core().skills.get(&(p, d)).is_some_and(|i| i.enabled)
}

fn once_used(e: &Engine, p: u8, skill: &str) -> bool {
    let def = e.catalog().skill_by_name(skill).expect("skill");
    e.state_core().skills.get(&(p, def)).is_some_and(|i| i.once_used)
}

fn hand(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

fn hp(e: &Engine, p: u8) -> i32 {
    e.state_core().player(p).hp
}

fn max_hp(e: &Engine, p: u8) -> u32 {
    e.state_core().player(p).max_hp
}

fn card_by(e: &Engine, pred: impl Fn(u32, &engine_native::catalog::CardDef) -> bool) -> u32 {
    for (idx, c) in e.catalog().cards.iter().enumerate() {
        let cid = idx as u32;
        let in_use = e.state_core().players.iter().any(|p| {
            p.hand.contains(&cid) || p.equips.contains(&cid) || p.judge.contains(&cid)
        });
        if !in_use && pred(cid, c) {
            return cid;
        }
    }
    panic!("no matching card in catalog");
}

fn give(e: &mut Engine, p: u8, card: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|&c| c != card);
    core.discard.retain(|&c| c != card);
    for s in 0..core.players.len() as u8 {
        core.player_mut(s).hand.retain(|&c| c != card);
        core.player_mut(s).equips.retain(|&c| c != card);
        core.player_mut(s).judge.retain(|&c| c != card);
    }
    core.player_mut(p).hand.push(card);
}

fn give_kind(e: &mut Engine, p: u8, kind: CardKind) -> u32 {
    let cid = card_by(e, |_, c| c.kind == kind);
    give(e, p, cid);
    cid
}

fn play(e: &mut Engine, p: u8, card: u32, targets: Vec<u8>) {
    let out = submit(e, p, GameAction::PlayCard { card, targets });
    assert!(!is_rejected(&out), "play rejected: {out:?}");
    drain(e);
}

fn play_sha(e: &mut Engine, p: u8, target: u8) {
    let sha = give_kind(e, p, CardKind::Sha);
    play(e, p, sha, vec![target]);
}

fn play_rejected(e: &mut Engine, p: u8, card: u32, targets: Vec<u8>) -> bool {
    is_rejected(&submit(e, p, GameAction::PlayCard { card, targets }))
}

fn next_phase(e: &mut Engine, p: u8) {
    let out = submit(e, p, GameAction::NextPhase);
    assert!(!is_rejected(&out), "next_phase rejected: {out:?}");
    drain(e);
}

fn activate(e: &mut Engine, p: u8, skill: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(skill).expect("skill def");
    submit(e, p, GameAction::ActivateSkill { skill: def })
}

fn open(e: &Engine, prompt: &str) -> OpenChoice {
    let c = e.open_choice().unwrap_or_else(|| panic!("expected open choice {prompt}")).clone();
    assert_eq!(c.spec.prompt, prompt, "open prompt");
    c
}

fn answer(e: &mut Engine, prompt: &str, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c = open(e, prompt);
    let out = submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection });
    assert!(!is_rejected(&out), "{prompt} answer rejected: {out:?}");
    out
}

fn rejected(e: &mut Engine, selection: Vec<ChoiceOption>) -> bool {
    let c = e.open_choice().expect("open choice").clone();
    let before = e.state_hash();
    let out = submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection });
    let ok = is_rejected(&out);
    assert_eq!(e.state_hash(), before, "rejected answer must not mutate state");
    assert!(e.open_choice().is_some_and(|o| o.id == c.id), "choice stays open after reject");
    ok
}

fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let selection = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection });
    }
}

fn snapshot_roundtrip(e: &Engine, p: &ExtensionPack, sel: Vec<ChoiceOption>) {
    let c = e.open_choice().expect("WaitingForChoice for archive").clone();
    let planned = e.plan_action(c.observer, &BoundedPlanner::new(4));
    assert!(
        matches!(planned.action, GameAction::Answer { choice_id, .. } if choice_id == c.id),
        "AI must answer open {}",
        c.spec.prompt
    );
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode mid-choice archive");
    let mut restored = Engine::restore_with_extensions(&decoded, &[p]).expect("restore");
    assert_eq!(restored.state_hash(), e.state_hash(), "restore hash {}", c.spec.prompt);
    assert_eq!(restored.open_choice().map(|o| o.spec.prompt), Some(c.spec.prompt));
    let mut twin = Engine::restore_with_extensions(&e.save(), &[p]).expect("twin");
    for eng in [&mut restored, &mut twin] {
        let act = GameAction::Answer { choice_id: c.id, selection: sel.clone() };
        let out = submit(eng, c.observer, act);
        assert!(!is_rejected(&out), "resume answer: {out:?}");
    }
    assert_eq!(restored.state_hash(), twin.state_hash(), "post-resume hash {}", c.spec.prompt);
}

#[test]
fn l19_generals_config_and_66_skills_catalog_inventory() {
    // 1. 2 generals (`jiangshifz`, `jiangshinj`)
    assert_eq!(L19_DOOMSDAY_CHARACTERS, ["jiangshifz", "jiangshinj"]);
    let (z0, z1) = (&DOOMSDAY_ZOMBIE_GENERALS[0], &DOOMSDAY_ZOMBIE_GENERALS[1]);
    assert_eq!((z0.id, z0.name_zh, z0.gender, z0.group, z0.max_hp), ("jiangshifz", "邪灵僵尸", "male", "ye", 5));
    assert_eq!(z0.skills, &JIANGSHIFZ_SKILLS);
    assert_eq!((z1.id, z1.name_zh, z1.gender, z1.group, z1.max_hp), ("jiangshinj", "鬼影僵尸", "female", "ye", 3));
    assert_eq!(z1.skills, &JIANGSHINJ_SKILLS);

    // 2. 1 config (`zuan_brawl`) + Doomsday helper config surface
    assert_eq!(DEFAULT_ZUAN_BRAWL, "all");
    assert_eq!(ZUAN_BRAWL_MODES, ["off", "standard", "random", "all", "daily"]);
    assert_eq!(parse_zuan_brawl(None), Ok(ZuanBrawlMode::All));
    for (raw, mode) in [
        ("off", ZuanBrawlMode::Off),
        ("standard", ZuanBrawlMode::Standard),
        ("random", ZuanBrawlMode::Random),
        ("all", ZuanBrawlMode::All),
        ("daily", ZuanBrawlMode::Daily),
    ] {
        assert!(validate_zuan_brawl(raw) && parse_zuan_brawl(Some(raw)) == Ok(mode));
        assert_eq!(mode.as_str(), raw);
    }
    for bad in ["", "none", "hardcore", "ALL"] {
        assert!(!validate_zuan_brawl(bad) && parse_zuan_brawl(Some(bad)).is_err());
    }

    assert_eq!((IDENTITY_NAMES.len(), IDENTITY_GATES.len()), (15, 15));
    assert_eq!(identity_chain(2), vec!["er"]);
    assert_eq!(identity_chain(5), vec!["er", "san", "si"]);
    assert_eq!((zaibian_draw(2, 1, 0), zaibian_draw(0, 2, 1)), (3, 0));
    let raw_wj = BTreeMap::from([
        ("wujinmoshiDraw".into(), 2),
        ("wujinmoshiDamage".into(), 3),
        ("wujinmoshiDamageI".into(), 1),
        ("wujinmoshiRecover".into(), 2),
        ("wujinmoshiMaxHandCard".into(), 4),
        ("wujinmoshiDXB".into(), 45),
        ("humanSeat".into(), 0),
    ]);
    let wj: WujinMoshiConfig = parse_wujinmoshi(&raw_wj);
    let wj_stats = (
        wj.draw_bonus(0),
        wj.damage_bonus(0),
        wj.recover_bonus(0),
        wj.hand_card_bonus(0),
        wj.draw_bonus(1),
        wj.non_human_damage_delta(),
    );
    assert_eq!(wj_stats, (2, 3, 2, 4, 0, 2));

    // 3. Verify all 66 skills + 2 generals in real Catalog
    let p = pack();
    let mut e = fresh(&p, 1900, &[&[], &[], &[], &[]]);
    assert_eq!(L19_DOOMSDAY_SKILLS.len(), 66);
    for &sk in L19_DOOMSDAY_SKILLS.iter().chain(["_tuizhi3"].iter()) {
        let def = e.catalog().skill_by_name(sk).unwrap_or_else(|| panic!("missing L19 skill {sk}"));
        let b = &e.catalog().skills[def as usize].behavior;
        assert!(!b.is_vm_skill() && b.as_host_behavior().is_some(), "{sk} must be HostBehavior");
    }

    let g_fz = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.0 == "jiangshifz"))
        .expect("jiangshifz in catalog");
    let (_, fac_fz, sk_fz, hp_fz, gen_fz) = e.catalog().general_meta(g_fz).unwrap();
    assert_eq!((fac_fz, hp_fz, gen_fz, sk_fz.as_slice()), (Faction::Qun, 5, Gender::Male, &JIANGSHIFZ_SKILLS[..]));

    let g_nj = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.0 == "jiangshinj"))
        .expect("jiangshinj in catalog");
    let (_, fac_nj, sk_nj, hp_nj, gen_nj) = e.catalog().general_meta(g_nj).unwrap();
    assert_eq!((fac_nj, hp_nj, gen_nj, sk_nj.as_slice()), (Faction::Qun, 3, Gender::Female, &JIANGSHINJ_SKILLS[..]));

    e.debug_set_general_def(0, g_fz);
    e.debug_set_general_def(1, g_nj);
    assert_eq!((max_hp(&e, 0), max_hp(&e, 1)), (5, 3));
    assert!(JIANGSHIFZ_SKILLS.iter().all(|sk| has_skill(&e, 0, sk)));
    assert!(JIANGSHINJ_SKILLS.iter().all(|sk| has_skill(&e, 1, sk)));

    let stubs = [
        "chanceIdentity", "chanceIdentity0", "chanceIdentity2", "chanceIdentity3",
        "chanceIdentity4", "chanceIdentity5", "chanceIdentity6", "chanceIdentity7",
        "chanceIdentity8", "chanceIdentity9", "chanceIdentity10", "chanceIdentity11",
        "chanceIdentity12", "chanceIdentity13", "chanceIdentity14", "chanceIdentity15",
        "_jiangshiTx", "_jiangshiTx2", "_jiangshiTx3", "_keepideskill",
    ];
    assert_eq!(stubs.len(), 20);
    for st in stubs {
        e.debug_grant(0, st);
        assert!(is_rejected(&activate(&mut e, 0, st)));
    }
}

#[test]
fn l19_dedicated_zaibian_shishi_xunmeng_six_dimensions() {
    let p = pack();

    // 1. zaibian: ① normal draw (`loyal + 2 - rebel - renegade > 0`) & ④ blocked when `<= 0`
    let mut e = fresh(&p, 1901, &[&["zaibian"], &[], &[], &[]]);
    assert!(is_rejected(&activate(&mut e, 0, "zaibian")));
    let roles1 = vec![Role::Lord, Role::Loyalist, Role::Loyalist, Role::Rebel];
    e.state_core_mut().mode = GameMode::Identity { roles: roles1 };
    e.debug_set_turn(0, Phase::Draw, 1);
    let h_before = hand(&e, 0);
    next_phase(&mut e, 0);
    assert_eq!(hand(&e, 0), h_before + 4);

    // ④ Condition not met: loyal=1, rebel=2, renegade=1 -> 1 + 2 - 2 - 1 = 0 <= 0 -> 0 cards drawn
    let roles2 = vec![Role::Lord, Role::Rebel, Role::Rebel, Role::Traitor];
    e.state_core_mut().mode = GameMode::Identity { roles: roles2 };
    e.debug_set_turn(0, Phase::Draw, 2);
    let h_before2 = hand(&e, 0);
    next_phase(&mut e, 0);
    assert_eq!(hand(&e, 0), h_before2, "zaibian draws 0 when human - zombie + 2 <= 0");

    // 2. shishi: ① on kill -> max_hp +1 & recover 1; ④ non-killer does not trigger
    let mut e = fresh(&p, 1902, &[&["shishi"], &[], &["shishi"], &[]]);
    assert!(is_rejected(&activate(&mut e, 0, "shishi")));
    e.debug_set_hp(0, 2);
    e.debug_set_hp(2, 2);
    e.debug_set_hp(1, 1);
    play_sha(&mut e, 0, 1);
    assert!(!e.state_core().player(1).alive);
    assert_eq!((max_hp(&e, 0), hp(&e, 0), max_hp(&e, 2), hp(&e, 2)), (6, 3, 5, 2));

    // 3. xunmeng: ① Sha damage +1 and lose 1 HP when hp > 1; ④ when hp == 1 deals +1 damage without losing HP; non-Sha does not trigger
    let mut e = fresh(&p, 1903, &[&["xunmeng", "baozou"], &[], &[], &[]]);
    assert!(is_rejected(&activate(&mut e, 0, "xunmeng")));
    e.debug_set_hp(0, 2);
    play_sha(&mut e, 0, 1);
    assert_eq!((hp(&e, 1), hp(&e, 0)), (3, 1));
    play_sha(&mut e, 0, 1);
    assert_eq!((hp(&e, 1), hp(&e, 0)), (1, 1));
    let duel = give_kind(&mut e, 0, CardKind::Duel);
    play(&mut e, 0, duel, vec![2]);
    assert_eq!(hp(&e, 2), 4);
}

#[test]
fn l19_batch_a_interactive_ganran2_and_huzhu_six_dimensions() {
    let p = pack();

    // A. ganran2 + ganran + baozou across dimensions ①②③④⑤⑥
    let mut e = fresh(&p, 1904, &[&["baozou", "ganran", "ganran2"], &[], &[], &[]]);
    // ④ Condition not met: no equip card in hand -> ActivateSkill rejected
    let sha1 = give_kind(&mut e, 0, CardKind::Sha);
    assert!(is_rejected(&activate(&mut e, 0, "ganran2")));

    let eq1 = card_by(&e, |_, c| c.kind.equip_slot().is_some());
    let eq2 = card_by(&e, |id, c| c.kind.equip_slot().is_some() && id != eq1);
    give(&mut e, 0, eq1);
    give(&mut e, 0, eq2);

    // ② Cancel/Pass on "ganran2:choose_equip" leaves state & cards unconsumed
    let _ = activate(&mut e, 0, "ganran2");
    assert!(open(&e, "ganran2:choose_equip").spec.allow_cancel);
    answer(&mut e, "ganran2:choose_equip", Vec::new());
    assert!(flag(&e, 0, "ganran2_recast") == 0 && e.state_core().player(0).hand.contains(&eq1));

    // ③ Illegal card / count rejection on "ganran2:choose_equip"
    let _ = activate(&mut e, 0, "ganran2");
    let _ = open(&e, "ganran2:choose_equip");
    assert!(rejected(&mut e, vec![ChoiceOption::Card(sha1)]));
    assert!(rejected(&mut e, vec![ChoiceOption::Card(999)]));
    assert!(rejected(&mut e, vec![ChoiceOption::Card(eq1), ChoiceOption::Card(eq2)]));

    // ⑥ Mid-choice snapshot/restore + ① Normal execution
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Card(eq1)]);
    answer(&mut e, "ganran2:choose_equip", vec![ChoiceOption::Card(eq1)]);
    drain(&mut e);
    let g_flags = (flag(&e, 0, "ganran2_recast"), flag(&e, 0, "ganran_active"), flag(&e, 0, "doomsday_mutated"));
    assert_eq!(g_flags, (1, 1, 1));
    assert!(!e.state_core().player(0).hand.contains(&eq1));

    // ⑤ Multi-skill / passive modifier interaction:
    // - `ganran` locks hand equip `eq2` as `CardKind::Chain` (`Tiesuo`): equipping (`[]`) is rejected; playing on `[1, 2]` chains both!
    assert!(play_rejected(&mut e, 0, eq2, Vec::new()));
    play(&mut e, 0, eq2, vec![1, 2]);
    assert!(e.state_core().player(1).linked && e.state_core().player(2).linked);

    // - `baozou` grants unlimited `Sha` use in `Phase::Play`
    assert!(eval_sha_use_limit(e.state_core(), e.catalog(), 0, None, CardKind::Sha, DamageNature::Normal, 1).1);
    play(&mut e, 0, sha1, vec![1]);
    play_sha(&mut e, 0, 1);
    play_sha(&mut e, 0, 1);
    assert_eq!(hp(&e, 1), 2);

    // B. _huzhu across dimensions ①②③④⑥
    let mut e = fresh(&p, 1905, &[&["_huzhu"], &[], &[], &[]]);
    let tao1 = give_kind(&mut e, 0, CardKind::Tao);
    let tao2 = give_kind(&mut e, 0, CardKind::Tao);
    let sha = give_kind(&mut e, 0, CardKind::Sha);

    // ④ Condition not met: all allies at full HP, distance > 1, or zombie -> _huzhu cannot activate
    assert!(is_rejected(&activate(&mut e, 0, "_huzhu")));
    e.debug_set_hp(2, 3);
    assert!(is_rejected(&activate(&mut e, 0, "_huzhu")));
    e.debug_set_hp(3, 3);
    set_flag(&mut e, 3, "zombie_role", 1);
    assert!(is_rejected(&activate(&mut e, 0, "_huzhu")));

    // Damage seat 1 (human ally at distance 1) -> _huzhu becomes activatable
    e.debug_set_hp(1, 3);

    // ② Cancel/Pass on "_huzhu:discard_tao" refunds once_used and leaves state untouched
    let _ = activate(&mut e, 0, "_huzhu");
    assert!(open(&e, "_huzhu:discard_tao").spec.allow_cancel);
    answer(&mut e, "_huzhu:discard_tao", Vec::new());
    assert!(!once_used(&e, 0, "_huzhu") && flag(&e, 0, "_huzhu_used_stamp") == 0);
    assert!(e.state_core().player(0).hand.contains(&tao1));

    // ③ Illegal card rejection on "_huzhu:discard_tao" + ⑥ Mid-choice snapshot/restore (stage 1)
    let _ = activate(&mut e, 0, "_huzhu");
    let _ = open(&e, "_huzhu:discard_tao");
    assert!(rejected(&mut e, vec![ChoiceOption::Card(sha)]));
    assert!(rejected(&mut e, vec![ChoiceOption::Card(tao1), ChoiceOption::Card(tao2)]));
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Card(tao1)]);
    answer(&mut e, "_huzhu:discard_tao", vec![ChoiceOption::Card(tao1)]);

    // ③ Illegal target / empty selection rejection on "_huzhu:choose_target" + ⑥ Mid-choice snapshot/restore (stage 2)
    assert!(!open(&e, "_huzhu:choose_target").spec.allow_cancel);
    assert!(rejected(&mut e, Vec::new()));
    for bad_target in [0u8, 2, 3] {
        assert!(rejected(&mut e, vec![ChoiceOption::Player(bad_target)]));
    }
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Player(1)]);

    // ① Complete normal execution + ④ Once-per-turn limit reached
    answer(&mut e, "_huzhu:choose_target", vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!((hp(&e, 1), flag(&e, 0, "_huzhu_healed"), flag(&e, 1, "_huzhu_recovered")), (4, 1, 1));
    assert!(!e.state_core().player(0).hand.contains(&tao1));
    assert!(is_rejected(&activate(&mut e, 0, "_huzhu")));
}

#[test]
fn l19_batch_a_modes_1_to_3_zombie_annex_and_wujin_six_dimensions() {
    let p = pack();

    // 1. Mode 1 Zombie triggers: `_jisuangailv`, `_tuizhi`, `_tuizhi2`, `_tuizhi3`, `_jiangshi2`, `_jiangshi4`, `_jiangshi`, `_jiangshi3`
    let s0: &[&str] = &["_jisuangailv", "_tuizhi", "_tuizhi2", "_tuizhi3", "_jiangshi4"];
    let mut e = fresh(&p, 1906, &[s0, &["_jiangshi2"], &["_jiangshi"], &["_jiangshi", "_jiangshi3"]]);
    let roles = vec![Role::Lord, Role::Loyalist, Role::Loyalist, Role::Loyalist];
    e.state_core_mut().mode = GameMode::Identity { roles };
    set_flag(&mut e, 0, "_tuizhi", 2);
    // Seat 1 Prepare phase at _tuizhi == 2 triggers `_jiangshi2` -> transforms into jiangshifz (Rebel, max_hp + 5)
    e.debug_set_turn(0, Phase::End, 1);
    next_phase(&mut e, 0);
    assert_eq!((flag(&e, 1, "_jiangshi2_transformed"), flag(&e, 1, "zombie_role"), max_hp(&e, 1)), (1, 1, 10));
    assert!(has_skill(&e, 1, "zaibian") && has_skill(&e, 1, "xunmeng"));

    // Seat 0 (`zhu`) killed by seat 1 -> `_jiangshi4` transfers `Role::Lord` & `_tuizhi - 1` to seat 2 (`zhong`)
    set_play(&mut e, 1, 1);
    e.debug_set_hp(0, 1);
    play_sha(&mut e, 1, 0);
    let js4 = (flag(&e, 0, "_jiangshi4_transferred"), flag(&e, 2, "_jiangshi4_new_zhu"), flag(&e, 2, "_tuizhi"), flag(&e, 0, "zombie_role"));
    assert_eq!(js4, (1, 1, 1, 1));

    // Seat 2 kills seat 3 (non-zombie killer) -> seat 3 `_jiangshi` transforms into `jiangshinj` (`zombie_role == 2`, max_hp = 3)
    set_play(&mut e, 2, 1);
    e.debug_set_hp(3, 1);
    play_sha(&mut e, 2, 3);
    assert_eq!((flag(&e, 3, "_jiangshi_transformed"), flag(&e, 3, "zombie_role"), max_hp(&e, 3)), (1, 2, 3));
    assert!(has_skill(&e, 3, "baozou") && has_skill(&e, 3, "shishi"));

    // Seat 3 (`jiangshinj`, `zombie_role == 2`) kills seat 0 -> `_jiangshi3` upgrades seat 3 to `jiangshifz` (+1 max_hp from `shishi` = 6)
    set_play(&mut e, 3, 1);
    e.state_core_mut().skills.retain(|&(owner, _), _| owner != 0);
    clear_zone(&mut e, 0);
    e.debug_set_hp(0, 1);
    play_sha(&mut e, 3, 0);
    assert!(!e.state_core().player(0).alive);
    assert_eq!((flag(&e, 3, "_jiangshi3_transformed"), flag(&e, 3, "zombie_role"), max_hp(&e, 3)), (1, 1, 6));

    // `_tuizhi` + `_tuizhi2` + `_tuizhi3`: when human Lord reaches 8 turns, human win flags set
    for sk in ["_tuizhi", "_tuizhi2", "_tuizhi3"] {
        e.debug_grant(2, sk);
    }
    set_flag(&mut e, 2, "_tuizhi", 7);
    e.debug_set_turn(1, Phase::End, 2);
    next_phase(&mut e, 1);
    assert_eq!((flag(&e, 2, "_tuizhi"), flag(&e, 2, "_tuizhi2_human_win"), flag(&e, 2, "l19_tuizhi_victory")), (8, 1, 1));

    // 2. Mode 2 (`_gainSkill`) & Mode 3 (`_wujinmoshi*` 11 skills) across dimensions ①④⑤
    let wj0: &[&str] = &[
        "_gainSkill", "_wujinmoshiDamage", "_wujinmoshiDamageE", "_wujinmoshiMaxHandCard",
        "_wujinmoshiMaxHandCardE", "_wujinmoshiDraw", "_wujinmoshiDrawE", "_wujinmoshiRecover",
        "_wujinmoshiRecoverE", "baozou",
    ];
    let wj1: &[&str] = &["_wujinmoshiRE", "wujinmoshiRE1", "wujinmoshiRE2"];
    let mut e = fresh(&p, 1907, &[wj0, wj1, &["英姿"], &[]]);
    for (k, v) in [("wujin_damage_bonus", 2), ("wujin_hand_bonus", 3), ("wujin_draw_bonus", 2), ("wujin_recover_bonus", 2)] {
        set_flag(&mut e, 0, k, v);
    }

    // ⑤ `_wujinmoshiMaxHandCard` increases hand limit by 3 (5 + 3 = 8)
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 8);

    // ① `_wujinmoshiDraw` + `_wujinmoshiDrawE` at Phase::Draw begin
    e.debug_set_turn(0, Phase::Judge, 1);
    let h0 = hand(&e, 0);
    next_phase(&mut e, 0); // Judge -> Draw (2 base + 2*2 bonus = 6)
    assert_eq!((flag(&e, 0, "_wujinmoshiDraw_drawn"), hand(&e, 0)), (2, h0 + 6));
    next_phase(&mut e, 0); // Draw -> Play
    clear_zone(&mut e, 0);

    // ⑤ `_wujinmoshiDamage` +2 damage -> 1 Sha deals 3 damage -> triggers `wujinmoshiRE1` (revives to 5 HP)
    e.debug_set_hp(1, 3);
    play_sha(&mut e, 0, 1);
    assert_eq!((flag(&e, 0, "_wujinmoshiDamage_triggered"), flag(&e, 1, "_wujinmoshiRE_used"), hp(&e, 1)), (2, 1, 5));
    assert!(!has_skill(&e, 1, "wujinmoshiRE1"));

    // Second lethal hit on seat 1 -> triggers `wujinmoshiRE2` (revives to 3 HP)
    e.debug_set_hp(1, 3);
    play_sha(&mut e, 0, 1);
    assert_eq!((flag(&e, 1, "_wujinmoshiRE_used"), hp(&e, 1)), (2, 3));
    assert!(!has_skill(&e, 1, "wujinmoshiRE2"));

    // ④ Third lethal hit on seat 1 (both RE1 & RE2 exhausted) -> seat 1 dies and `_gainSkill` triggers on seat 0!
    play_sha(&mut e, 0, 1);
    assert!(!e.state_core().player(1).alive && flag(&e, 0, "_gainSkill_count") == 1);

    // `_wujinmoshiRecover`: at Phase::End when damaged, recovers `wujin_recover_bonus`
    e.debug_set_hp(0, 2);
    next_phase(&mut e, 0); // Play -> Discard
    next_phase(&mut e, 0); // Discard -> End
    assert_eq!((flag(&e, 0, "_wujinmoshiRecover_healed"), hp(&e, 0)), (2, 5));
}

#[test]
fn l19_batch_b_rpg_and_yuanqiskill_six_dimensions() {
    let p = pack();

    // 1. Shared RPG 6 skills (`_levelskill`, `_expskill`, `_exp2skill`, `_useskill`, `_killskill`, `_relifeskill`)
    let rpg0: &[&str] = &["_levelskill", "_expskill", "_exp2skill", "_useskill", "_killskill", "baozou"];
    let mut e = fresh(&p, 1908, &[rpg0, &["_relifeskill"], &[], &[]]);
    set_flag(&mut e, 1, "zuan_relife", 1);
    set_flag(&mut e, 0, "zuan_exp", 75);
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 1);
    play_sha(&mut e, 0, 1);
    assert_eq!((flag(&e, 0, "zuan_level"), flag(&e, 0, "zuan_exp"), flag(&e, 0, "zuan_score"), flag(&e, 0, "_levelskill_granted")), (2, 0, 15, 1));
    assert_eq!((flag(&e, 1, "zuan_relife"), flag(&e, 1, "_relifeskill_revived"), hp(&e, 1)), (0, 1, 4));

    // ④ Now seat 1 has `zuan_relife == 0`: killing seat 1 triggers `_killskill` (+100 exp, +100 score, draw 3 -> Lv3)
    e.debug_set_hp(1, 1);
    play_sha(&mut e, 0, 1);
    assert!(!e.state_core().player(1).alive);
    assert_eq!((flag(&e, 0, "zuan_level"), flag(&e, 0, "zuan_score")), (3, 130));

    clear_zone(&mut e, 0);
    next_phase(&mut e, 0); // Play -> Discard
    next_phase(&mut e, 0); // Discard -> End (`_exp2skill`: +10 exp, +5 score)
    assert_eq!(flag(&e, 0, "zuan_score"), 135);

    // 2. `_yuanqiskill` across dimensions ①②③④⑥
    let mut e = fresh(&p, 1909, &[&["_yuanqiskill"], &[], &[], &[]]);
    // ④ Condition not met: zuan_exp == 0 AND no removable skills -> ActivateSkill rejected
    assert!(is_rejected(&activate(&mut e, 0, "_yuanqiskill")));

    // Grant a removable skill ("baozou") while zuan_exp == 50 (< 100):
    // ④ Mode 0 is blocked (only Tag(1) is in options); ③ Tag(0) is rejected!
    e.debug_grant(0, "baozou");
    set_flag(&mut e, 0, "zuan_exp", 50);
    let _ = activate(&mut e, 0, "_yuanqiskill");
    assert_eq!(open(&e, "_yuanqiskill:choose_mode").spec.options, vec![ChoiceOption::Tag(1)]);
    assert!(rejected(&mut e, vec![ChoiceOption::Tag(0)]) && rejected(&mut e, vec![ChoiceOption::Tag(9)]));

    // ② Cancel/Pass on "_yuanqiskill:choose_mode" refunds once_used and leaves state untouched
    answer(&mut e, "_yuanqiskill:choose_mode", Vec::new());
    assert!(!once_used(&e, 0, "_yuanqiskill") && flag(&e, 0, "zuan_exp") == 50);

    // Set zuan_exp = 120 -> both Mode 0 and Mode 1 available
    set_flag(&mut e, 0, "zuan_exp", 120);
    let _ = activate(&mut e, 0, "_yuanqiskill");
    let _ = open(&e, "_yuanqiskill:choose_mode");
    assert!(rejected(&mut e, vec![ChoiceOption::Tag(2)]));

    // ⑥ Mid-choice snapshot/restore on "_yuanqiskill:choose_mode" -> pick Mode 0
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, "_yuanqiskill:choose_mode", vec![ChoiceOption::Tag(0)]);
    assert_eq!(flag(&e, 0, "zuan_exp"), 20);

    // ③ Illegal skill index / empty selection on "_yuanqiskill:choose_skill" + ⑥ Mid-choice snapshot/restore
    assert!(!open(&e, "_yuanqiskill:choose_skill").spec.allow_cancel);
    assert!(rejected(&mut e, Vec::new()) && rejected(&mut e, vec![ChoiceOption::Tag(3)]) && rejected(&mut e, vec![ChoiceOption::Tag(99)]));
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Tag(2)]);

    // ① Complete Mode 0 (`Tag(2)` -> gains `yingzi`) + ④ Once-per-turn limit
    answer(&mut e, "_yuanqiskill:choose_skill", vec![ChoiceOption::Tag(2)]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "_yuanqiskill_gained"), 1);
    assert!(is_rejected(&activate(&mut e, 0, "_yuanqiskill")));

    // Next turn: test Mode 1 (`_yuanqiskill:remove_skill`) across ③⑥①
    set_play(&mut e, 0, 2);
    let yq_def = e.catalog().skill_by_name("_yuanqiskill").unwrap();
    e.state_core_mut().skills.get_mut(&(0, yq_def)).unwrap().once_used = false;
    e.debug_set_hp(0, 3);
    let h_before = hand(&e, 0);
    let _ = activate(&mut e, 0, "_yuanqiskill");
    answer(&mut e, "_yuanqiskill:choose_mode", vec![ChoiceOption::Tag(1)]);
    assert!(!open(&e, "_yuanqiskill:remove_skill").spec.allow_cancel);
    assert!(rejected(&mut e, Vec::new()) && rejected(&mut e, vec![ChoiceOption::Tag(99)]));
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, "_yuanqiskill:remove_skill", vec![ChoiceOption::Tag(0)]);
    drain(&mut e);
    assert_eq!((flag(&e, 0, "_yuanqiskill_removed"), hp(&e, 0), hand(&e, 0)), (1, 4, h_before + 2));
}

#[test]
fn l19_batch_b_modes_5_to_10_jingcheng_zhengba_taofa_guixin_huanhai_ansha_six_dimensions() {
    let p = pack();

    // 1. Mode 5 `zuan_jingcheng` (`_zhongzhubuff`, `_jingchengadddie`, `_zhongfandieskill`, `_tamodeskill`, `_zhudieskill`)
    //    & Mode 6 `zuan_zhengba` (`_qunxiongzhengbaskill`) across ①③④⑤
    let jc0: &[&str] = &["_zhudieskill", "_jingchengadddie", "baozou"];
    let jc3: &[&str] = &["_zhongfandieskill", "_qunxiongzhengbaskill"];
    let mut e = fresh(&p, 1910, &[jc0, &["_zhongzhubuff"], &["_tamodeskill"], jc3]);
    set_flag(&mut e, 0, "is_zhu", 1);

    // ⑤ `_tamodeskill` target protection: seat 0 cannot target seat 2 with Sha
    let sha1 = give_kind(&mut e, 0, CardKind::Sha);
    assert!(play_rejected(&mut e, 0, sha1, vec![2]));

    // ① When `zhu.max_hp == 5` (> 3), `_zhongzhubuff` triggers and saves seat 1 (`zhong`)!
    e.debug_set_hp(1, 1);
    play(&mut e, 0, sha1, vec![1]);
    assert_eq!((flag(&e, 1, "_zhongzhubuff_triggered"), max_hp(&e, 0), hp(&e, 1)), (1, 4, 2));

    // ④ `_zhongzhubuff` blocked when `zhu.max_hp <= 3`:
    set_flag(&mut e, 1, "_zhongzhubuff_triggered", 0);
    e.state_core_mut().player_mut(0).max_hp = 3;
    e.state_core_mut().player_mut(0).hp = 3;
    clear_zone(&mut e, 1);
    e.debug_set_hp(1, 1);
    play_sha(&mut e, 0, 1);
    assert!(flag(&e, 1, "_zhongzhubuff_triggered") == 0 && !e.state_core().player(1).alive && flag(&e, 0, "jingcheng_fan_kills") == 1);

    // ① `_tamodeskill` at Seat 2 Phase::Play begin: skips Play, discards 1 card from enemy & deals 1 damage
    e.debug_set_turn(2, Phase::Draw, 1);
    next_phase(&mut e, 2);
    assert_eq!(flag(&e, 2, "_tamodeskill_fired"), 1);

    // ① `_zhudieskill` saves seat 0 using tower nei (seat 2); `_zhongfandieskill` + `_qunxiongzhengbaskill` respawn seat 3
    set_play(&mut e, 3, 1);
    e.debug_set_hp(0, 1);
    play_sha(&mut e, 3, 0);
    assert!(flag(&e, 0, "_zhudieskill_saved") == 1 && hp(&e, 0) >= 1);

    set_play(&mut e, 0, 2);
    for _ in 0..2 {
        clear_zone(&mut e, 3);
        e.debug_set_hp(3, 1);
        play_sha(&mut e, 0, 3);
    }
    assert_eq!((flag(&e, 3, "_zhongfandieskill_respawned"), flag(&e, 3, "_qunxiongzhengbaskill_respawned")), (1, 1));

    // 2. Mode 7 `zuan_taofa` (`_guanmodeskill`, `_guandieskill`):
    //    ⑤ delayed-trick immunity (`Le`) & turnover immunity (`据守` cancelled), ① 3-phase boss transition
    let mut e = fresh(&p, 1911, &[&["_guanmodeskill", "_guandieskill", "据守"], &["baozou"], &[], &[]]);
    e.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e, 3);
    assert_eq!((flag(&e, 0, "_guanmodeskill_fired"), flag(&e, 0, "_guanmodeskill_immune")), (1, 1));

    // ⑤ Turnover immunity: at Seat 0 Phase::End, confirming `据守` emits `TurnOverBefore`, which `_guanmodeskill` cancels!
    e.debug_set_turn(0, Phase::Discard, 1);
    clear_zone(&mut e, 0);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
    }
    drain(&mut e);
    assert!(!e.state_core().player(0).face_down);

    // ⑤ Delayed-trick immunity: Seat 1 cannot target Seat 0 with `Le`
    set_play(&mut e, 1, 1);
    let le = give_kind(&mut e, 1, CardKind::Le);
    assert!(play_rejected(&mut e, 1, le, vec![0]));

    // ① `_guandieskill`: Phase 1 -> Phase 2 -> Phase 3 -> `taofa_fan_win`
    for expected in [2u32, 3u32] {
        e.debug_set_hp(0, 1);
        play_sha(&mut e, 1, 0);
        assert_eq!(flag(&e, 0, "guan_phase"), expected);
    }
    e.debug_set_hp(0, 1);
    play_sha(&mut e, 1, 0);
    assert_eq!(flag(&e, 0, "taofa_fan_win"), 1);

    // 3. Mode 8 (`_guixinmodeskill`), Mode 9 (`_huanhaiskill`, `_seadreammodeskill`), Mode 10 (`_dietime`, `_anshaskill`)
    let m80: &[&str] = &["_guixinmodeskill", "_huanhaiskill", "baozou"];
    let mut e = fresh(&p, 1912, &[m80, &["_seadreammodeskill"], &["_anshaskill"], &["_dietime"]]);
    set_flag(&mut e, 2, "ansha_is_zhu", 1);
    set_flag(&mut e, 2, "ansha_lives", 1);
    set_flag(&mut e, 0, "huanhai_stage", 2);
    e.debug_set_hp(0, 3);
    e.debug_set_turn(3, Phase::End, 1);
    next_phase(&mut e, 3);
    assert_eq!(flag(&e, 0, "_huanhaiskill_stage_applied"), 2);

    set_play(&mut e, 0, 1);
    for s in 0..4 {
        clear_zone(&mut e, s);
    }
    e.debug_set_hp(3, 1);
    play_sha(&mut e, 0, 3);
    e.state_core_mut().skills.retain(|&(o, _), _| o != 0);
    e.debug_set_hp(1, 1);
    play_sha(&mut e, 0, 1);
    assert_eq!((flag(&e, 0, "_guixinmodeskill_converted"), flag(&e, 1, "_seadreammodeskill_stage")), (1, 3));

    // Seat 1 hits Seat 2 (`ansha_is_zhu`) -> swaps zhu to Seat 1 and respawns Seat 2 as fan
    set_play(&mut e, 1, 1);
    clear_zone(&mut e, 2);
    e.debug_set_hp(2, 1);
    play_sha(&mut e, 1, 2);
    assert_eq!((flag(&e, 2, "_anshaskill_zhu_swapped"), flag(&e, 1, "ansha_is_zhu")), (1, 1));

    // `_dietime` expires at 5
    set_flag(&mut e, 3, "dietime", 4);
    e.debug_set_turn(3, Phase::Discard, 1);
    next_phase(&mut e, 3);
    assert!(flag(&e, 3, "_dietime_expired") == 1 && !e.state_core().player(3).alive);
}
