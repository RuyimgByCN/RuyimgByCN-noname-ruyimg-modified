//! L27 选将美化 6 维实盘闭环测试（Part 1：10 项 `configs:*` + 6 项专有 `HostBehavior` 技能 + 19 项 Batch A 技能/桩）。
//!
//! 覆盖范围（Queue Item 2 Source 9 `L27 选将美化` 前 35 项）：
//! - 10 项 `configs:*`：`twotwo`、`gaibiaoji`、`dianjibiaoji`、`shoupaihuase`、`feichuxuanxiang`、
//!   `shuzianniu`、`xuanzejineng`、`zediewugu`、`jinxuanbiaoyu`、`jinengtexiaoxiugai`
//! - 6 项专有 `HostBehavior`：`xiefang`、`sbzhaxiang`、`new_meibu`、`new_zhixi`、`jishe`、`mbcmqingzheng`
//! - 19 项 Batch A：`regushe`、`kuangcai`、`sbtongye`、`baonvezhi`、`clandianzhan`、`rehuashen`、
//!   `rexinsheng`、`taoluan`、`shouxi`、`dinghan`、`quanji`、`potzhuangshi`、`qimou`、`reqimou`、
//!   `sbjianxiong`、`jinghe`、`baonvezhi_mark`、`rehuashen_mark`、`huamu`

use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::select_beautify::{
    parse_select_beautify_config, SelectBeautifyError, JINXUAN_BIAOYU_ITEMS, JINXUAN_BIAOYU_LABELS,
};
use content_native_loader::{decision34_pack9_extension_pack, L27_SELECT_BEAUTIFY_SKILLS};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::standard::Gender;
use engine_native::event::{DamageNature, Phase};
use engine_native::rvm_bridge::modifiers::{
    eval_card_distance_unlimited, eval_card_unrespondable, eval_damage_amount, eval_draw_count,
    eval_sha_use_limit,
};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice};

static DEDUP: AtomicU64 = AtomicU64::new(2_700_000);

pub(crate) fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

pub(crate) fn is_rejected(out: CommandOutcome) -> bool {
    matches!(out, CommandOutcome::Rejected(_))
}

pub(crate) fn pack() -> ExtensionPack {
    let mut p = decision34_pack9_extension_pack();
    for (name, behavior) in [
        ("new_zhixi_mark2", HostBehavior::NewZhixi),
        ("new_meibu_range", HostBehavior::NewMeibuRange),
        ("bolan_g", HostBehavior::L27BeautifyBatchB),
    ] {
        if !p.host_skills.iter().any(|s| s.name == name) {
            p.host_skills.push(HostSkillDecl {
                name,
                behavior,
                presentation_binding: None,
            });
        }
    }
    p
}

pub(crate) fn fresh(p: &ExtensionPack, seed: u64, skills_by_seat: &[&[&str]]) -> Engine {
    let mut e = Engine::new_with_extensions(seed, 4, &[p]).expect("L27 engine");
    for seat in 0..4u8 {
        e.state_core_mut().skills.retain(|&(owner, _), _| owner != seat);
        clear_hand(&mut e, seat);
        e.state_core_mut().player_mut(seat).max_hp = 5;
        e.state_core_mut().player_mut(seat).hp = 5;
    }
    for (seat, names) in skills_by_seat.iter().enumerate() {
        for &sk in *names {
            e.debug_grant(seat as u8, sk);
        }
    }
    e.debug_set_turn(0, Phase::Play, 1);
    e
}

pub(crate) fn clear_hand(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    let eqs: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    let jds: Vec<u32> = e.state_core_mut().player_mut(seat).judge.drain(..).collect();
    e.state_core_mut().discard.extend(cards);
    e.state_core_mut().discard.extend(eqs);
    e.state_core_mut().discard.extend(jds);
}

pub(crate) fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(crate) fn set_flag(e: &mut Engine, p: u8, key: &str, v: u32) {
    if v == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), v);
    }
}

pub(crate) fn once_used(e: &Engine, p: u8, skill: &str) -> bool {
    let def = e.catalog().skill_by_name(skill).expect("skill");
    e.state_core()
        .skills
        .get(&(p, def))
        .is_some_and(|i| i.once_used)
}

pub(crate) fn has_skill(e: &Engine, p: u8, skill: &str) -> bool {
    let Some(def) = e.catalog().skill_by_name(skill) else {
        return false;
    };
    e.state_core()
        .skills
        .get(&(p, def))
        .is_some_and(|i| i.enabled)
}

pub(crate) fn hand(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

pub(crate) fn hp(e: &Engine, p: u8) -> i32 {
    e.state_core().player(p).hp
}

pub(crate) fn card_by(e: &Engine, pred: impl Fn(u32, &engine_native::catalog::CardDef) -> bool) -> u32 {
    for (idx, c) in e.catalog().cards.iter().enumerate() {
        let cid = idx as u32;
        let in_use = e
            .state_core()
            .players
            .iter()
            .any(|p| p.hand.contains(&cid) || p.equips.contains(&cid) || p.judge.contains(&cid));
        if !in_use && pred(cid, c) {
            return cid;
        }
    }
    panic!("no matching card in catalog");
}

pub(crate) fn take_out(e: &mut Engine, card: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|&c| c != card);
    core.discard.retain(|&c| c != card);
    for p in 0..core.players.len() as u8 {
        core.player_mut(p).hand.retain(|&c| c != card);
        core.player_mut(p).equips.retain(|&c| c != card);
        core.player_mut(p).judge.retain(|&c| c != card);
    }
}

pub(crate) fn give(e: &mut Engine, p: u8, card: u32) {
    take_out(e, card);
    e.state_core_mut().player_mut(p).hand.push(card);
}

pub(crate) fn activate(e: &mut Engine, p: u8, skill: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(skill).expect("skill def");
    submit(e, p, GameAction::ActivateSkill { skill: def })
}

pub(crate) fn open(e: &Engine, prompt: &str) -> OpenChoice {
    let c = e
        .open_choice()
        .unwrap_or_else(|| panic!("expected open choice {prompt}"))
        .clone();
    assert_eq!(c.spec.prompt, prompt, "open prompt");
    c
}

pub(crate) fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> CommandOutcome {
    let c = open(e, prompt);
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "{prompt} answer rejected: {out:?}"
    );
    out
}

pub(crate) fn rejected(e: &mut Engine, sel: Vec<ChoiceOption>) -> bool {
    let c = e.open_choice().expect("open choice").clone();
    let before = e.state_hash();
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
    let ok = matches!(out, CommandOutcome::Rejected(_));
    assert_eq!(e.state_hash(), before, "rejected answer must not mutate state");
    assert!(
        e.open_choice().is_some_and(|o| o.id == c.id),
        "choice stays open after reject"
    );
    ok
}

pub(crate) fn tag(t: u8) -> ChoiceOption {
    ChoiceOption::Tag(t)
}
pub(crate) fn pl(p: u8) -> ChoiceOption {
    ChoiceOption::Player(p)
}
pub(crate) fn cd(c: u32) -> ChoiceOption {
    ChoiceOption::Card(c)
}

pub(crate) fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
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

pub(crate) fn snapshot_roundtrip(e: &Engine, p: &ExtensionPack, sel: Vec<ChoiceOption>) {
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
    assert_eq!(
        restored.open_choice().map(|o| o.spec.prompt),
        Some(c.spec.prompt)
    );
    let mut twin = Engine::restore_with_extensions(&e.save(), &[p]).expect("twin");
    for eng in [&mut restored, &mut twin] {
        let out = submit(
            eng,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel.clone(),
            },
        );
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "resume answer: {out:?}"
        );
    }
    assert_eq!(
        restored.state_hash(),
        twin.state_hash(),
        "post-resume hash {}",
        c.spec.prompt
    );
}

pub(crate) fn hand_turn_to(e: &mut Engine, seat: u8, round: u32) -> CommandOutcome {
    let prev = (seat + 3) % 4;
    e.debug_set_turn(prev, Phase::End, round);
    submit(e, prev, GameAction::NextPhase)
}

pub(crate) fn play(e: &mut Engine, p: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let out = submit(e, p, GameAction::PlayCard { card, targets });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "play rejected: {out:?}"
    );
    out
}

#[test]
fn l27_configs_and_skill_registration() {
    let raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packages/extensions/select-beautify-upstream-liuli/assets/config.json"
    ))
    .expect("read L27 config.json");
    let cfg = parse_select_beautify_config(&raw).expect("parse default config.json");
    assert_eq!(cfg.source, "noname-liuli");
    assert_eq!(cfg.upstream_path, "noname/extension/选将美化/extension.js");
    assert!(cfg.twotwo);
    assert!(cfg.gaibiaoji);
    assert!(cfg.dianjibiaoji);
    assert!(cfg.shoupaihuase);
    assert!(cfg.feichuxuanxiang);
    assert!(cfg.shuzianniu);
    assert!(cfg.xuanzejineng);
    assert!(cfg.zediewugu);
    assert_eq!(cfg.jinxuanbiaoyu, "1");
    assert_eq!(cfg.jinxuan_biaoyu_label(), "黑底红叉");
    assert!(cfg.jinengtexiaoxiugai);
    assert_eq!(JINXUAN_BIAOYU_ITEMS, ["1", "2", "3"]);
    assert_eq!(
        JINXUAN_BIAOYU_LABELS,
        ["黑底红叉", "禁选锁链", "禁选锁链（动态）"]
    );

    let custom1 = r#"{
        "source":"noname-liuli","upstream_path":"noname/extension/选将美化/extension.js","upstream_commit":"8485dbb",
        "twotwo":false,"gaibiaoji":false,"dianjibiaoji":false,"shoupaihuase":false,
        "feichuxuanxiang":false,"shuzianniu":false,"xuanzejineng":false,"zediewugu":false,
        "jinxuanbiaoyu":"2","jinengtexiaoxiugai":false
    }"#;
    let c1 = parse_select_beautify_config(custom1).expect("parse custom1");
    assert!(!c1.twotwo && !c1.gaibiaoji && !c1.dianjibiaoji && !c1.shoupaihuase);
    assert!(!c1.feichuxuanxiang && !c1.shuzianniu && !c1.xuanzejineng && !c1.zediewugu);
    assert_eq!(c1.jinxuanbiaoyu, "2");
    assert_eq!(c1.jinxuan_biaoyu_label(), "禁选锁链");
    assert!(!c1.jinengtexiaoxiugai);

    let custom2 = custom1.replace("\"jinxuanbiaoyu\":\"2\"", "\"jinxuanbiaoyu\":\"3\"");
    let c2 = parse_select_beautify_config(&custom2).expect("parse custom2");
    assert_eq!(c2.jinxuan_biaoyu_label(), "禁选锁链（动态）");

    assert!(matches!(
        parse_select_beautify_config("not-json"),
        Err(SelectBeautifyError::BadField(_))
    ));
    assert_eq!(
        parse_select_beautify_config("{}"),
        Err(SelectBeautifyError::MissingField("source".into()))
    );
    for (bad_key, bad_json) in [
        ("twotwo", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","twotwo":"bad"}"#),
        ("gaibiaoji", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","gaibiaoji":1}"#),
        ("dianjibiaoji", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","dianjibiaoji":0}"#),
        ("shoupaihuase", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","shoupaihuase":"yes"}"#),
        ("feichuxuanxiang", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","feichuxuanxiang":2}"#),
        ("shuzianniu", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","shuzianniu":"no"}"#),
        ("xuanzejineng", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","xuanzejineng":3}"#),
        ("zediewugu", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","zediewugu":"no"}"#),
        ("jinxuanbiaoyu", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","jinxuanbiaoyu":"9"}"#),
        ("jinengtexiaoxiugai", r#"{"source":"a","upstream_path":"b","upstream_commit":"c","jinengtexiaoxiugai":0}"#),
    ] {
        let err = parse_select_beautify_config(bad_json).unwrap_err();
        assert_eq!(err, SelectBeautifyError::BadField(bad_key.into()));
        assert!(err.message().contains(bad_key));
    }

    let p = pack();
    let mut e = fresh(&p, 2700, &[&["baonvezhi_mark", "rehuashen_mark", "huamu"], &[]]);
    assert_eq!(L27_SELECT_BEAUTIFY_SKILLS.len(), 43);
    for &sk in &L27_SELECT_BEAUTIFY_SKILLS {
        let def = e.catalog().skill_by_name(sk).unwrap_or_else(|| panic!("missing {sk}"));
        let b = &e.catalog().skills[def as usize].behavior;
        assert!(
            !b.is_vm_skill() && b.as_host_behavior().is_some(),
            "{sk} must be native host skill"
        );
    }
    for stub in ["baonvezhi_mark", "rehuashen_mark", "huamu"] {
        assert!(is_rejected(activate(&mut e, 0, stub)));
    }
}

#[test]
fn l27_dedicated_six_skills_six_dimensions() {
    let p = pack();

    // 1. xiefang: distance -count(alive female)
    {
        let mut e = fresh(&p, 2701, &[&["xiefang"], &[], &[], &[]]);
        let before = e.state_core().computed_distance(e.catalog(), 0, 2);
        let female_gid = (0..e.catalog().general_count() as u16)
            .find(|g| {
                e.catalog()
                    .general_meta(*g)
                    .is_some_and(|(_, _, _, _, gender)| gender == Gender::Female)
            })
            .expect("female general");
        let female_name = e.catalog().general_meta(female_gid).unwrap().0;
        e.debug_set_general(2, &female_name);
        let after = e.state_core().computed_distance(e.catalog(), 0, 2);
        assert!(after < before);
        assert!(is_rejected(activate(&mut e, 0, "xiefang")));
    }

    // 2. sbzhaxiang: damaged HP -> extra draw & unlimited/directHit Sha for first damaged_hp cards
    {
        let mut e = fresh(&p, 2702, &[&["sbzhaxiang"], &[]]);
        e.debug_set_hp(0, 3); // max_hp=5 -> damaged=2
        e.debug_set_turn(0, Phase::Judge, 1);
        let before = hand(&e, 0);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge -> Draw
        drain(&mut e);
        assert_eq!(hand(&e, 0), before + 4, "normal 2 + damaged 2");
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw -> Play
        drain(&mut e);
        let sha1 = card_by(&e, |_, c| c.kind == CardKind::Sha);
        let sha2 = card_by(&e, |id, c| c.kind == CardKind::Sha && id != sha1);
        give(&mut e, 0, sha1);
        give(&mut e, 0, sha2);
        let hp1 = hp(&e, 1);
        play(&mut e, 0, sha1, vec![1]);
        drain(&mut e);
        play(&mut e, 0, sha2, vec![1]);
        drain(&mut e);
        assert_eq!(hp(&e, 1), hp1 - 2);
    }

    // 3. new_meibu + 4. new_zhixi
    {
        let mut e = fresh(&p, 2703, &[&["new_meibu"], &["new_zhixi"]]);
        let red_basic = card_by(&e, |_, c| {
            c.kind.is_basic_kind()
                && matches!(c.suit, Suit::Heart | Suit::Diamond)
                && !c.kind.is_sha_like()
        });
        give(&mut e, 0, red_basic);
        let wuzhong = card_by(&e, |_, c| c.kind == CardKind::Wuzhong);
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        give(&mut e, 1, wuzhong);
        give(&mut e, 1, sha);
        // Seat 1 in Play phase uses trick wuzhong -> new_zhixi locks further card play
        e.debug_set_turn(1, Phase::Play, 1);
        play(&mut e, 1, wuzhong, vec![]);
        drain(&mut e);
        assert!(is_rejected(submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: sha,
                targets: vec![0]
            }
        )));

        // Advance to next round Seat 1 Draw -> Play begin -> triggers new_meibu on Seat 0
        let mut e2 = fresh(&p, 27031, &[&["new_meibu"], &[]]);
        give(&mut e2, 0, red_basic);
        give(&mut e2, 1, sha);
        e2.debug_set_turn(1, Phase::Draw, 1);
        let _ = submit(&mut e2, 1, GameAction::NextPhase);
        if e2.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            answer(&mut e2, "trigger_confirm", vec![ChoiceOption::Confirm]);
        }
        assert!(rejected(&mut e2, vec![cd(999)]));
        snapshot_roundtrip(&e2, &p, vec![cd(red_basic)]);
        answer(&mut e2, "new_meibu_discard", vec![cd(red_basic)]);
        drain(&mut e2);
        assert!(is_rejected(submit(
            &mut e2,
            1,
            GameAction::PlayCard {
                card: sha,
                targets: vec![0]
            }
        )));
    }

    // 5. jishe + 6. mbcmqingzheng
    {
        let mut e = fresh(&p, 2704, &[&["jishe"], &["mbcmqingzheng"]]);
        let h0 = hand(&e, 0);
        assert!(!is_rejected(activate(&mut e, 0, "jishe")));
        drain(&mut e);
        assert_eq!(hand(&e, 0), h0 + 1);
        // Set jishe2 = 5 so hand_limit becomes 0 -> jishe precondition fails
        set_flag(&mut e, 0, "jishe2", 5);
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 0);
        assert!(is_rejected(activate(&mut e, 0, "jishe")));

        // mbcmqingzheng: Seat 1 enters Play phase with 2 Spades while Seat 0 has 1 Heart
        clear_hand(&mut e, 0);
        clear_hand(&mut e, 1);
        let s1 = card_by(&e, |_, c| c.suit == Suit::Spade);
        let s2 = card_by(&e, |id, c| c.suit == Suit::Spade && id != s1);
        let h1 = card_by(&e, |_, c| c.suit == Suit::Heart);
        give(&mut e, 1, s1);
        give(&mut e, 1, s2);
        give(&mut e, 0, h1);
        let hp0 = hp(&e, 0);
        e.debug_set_turn(1, Phase::Draw, 1);
        let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw -> Play begin
        if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
        }
        let s_tag = open(&e, "mbcmqingzheng_suit").spec.options[0];
        snapshot_roundtrip(&e, &p, vec![s_tag]);
        answer(&mut e, "mbcmqingzheng_suit", vec![s_tag]);
        answer(&mut e, "mbcmqingzheng_target", vec![pl(0)]);
        let t_tag = open(&e, "mbcmqingzheng_target_suit").spec.options[0];
        answer(&mut e, "mbcmqingzheng_target_suit", vec![t_tag]);
        drain(&mut e);
        assert_eq!(hp(&e, 0), hp0 - 1);
    }
}

#[test]
fn l27_batch_a_active_and_pindian_skills() {
    let p = pack();

    // 1. regushe: pindian up to 3 targets, win re-enables, lose adds raoshe mark
    {
        let mut e = fresh(&p, 2705, &[&["regushe"], &[]]);
        let high = card_by(&e, |_, c| c.rank == 13);
        let low = card_by(&e, |_, c| c.rank == 1);
        let extra = card_by(&e, |id, c| c.rank == 2 && id != low);
        give(&mut e, 0, high);
        give(&mut e, 1, low);
        give(&mut e, 1, extra);
        activate(&mut e, 0, "regushe");
        snapshot_roundtrip(&e, &p, Vec::new());
        // ② Cancel refunds once_used
        answer(&mut e, "regushe:choose_targets", Vec::new());
        assert!(!once_used(&e, 0, "regushe"));

        activate(&mut e, 0, "regushe");
        // ③ Cannot target self
        assert!(rejected(&mut e, vec![pl(0)]));
        answer(&mut e, "regushe:choose_targets", vec![pl(1)]);
        snapshot_roundtrip(&e, &p, vec![cd(high)]);
        answer(&mut e, "regushe:pindian_card", vec![cd(high)]);
        answer(&mut e, "regushe:target_pindian_card", vec![cd(low)]);
        snapshot_roundtrip(&e, &p, Vec::new());
        answer(&mut e, "regushe:target_discard", Vec::new()); // loser refuses discard -> owner draws 1
        assert_eq!(hand(&e, 0), 1);
        assert!(!once_used(&e, 0, "regushe"), "winning pindian allows reuse");
    }

    // 2. taoluan: virtual card use + helper give / refuse penalty
    {
        let mut e = fresh(&p, 2706, &[&["taoluan"], &[]]);
        let equip = card_by(&e, |_, c| c.kind.equip_slot().is_some());
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        give(&mut e, 0, equip);
        give(&mut e, 1, sha);
        activate(&mut e, 0, "taoluan");
        snapshot_roundtrip(&e, &p, Vec::new());
        answer(&mut e, "taoluan:choose_card", Vec::new());
        assert!(!once_used(&e, 0, "taoluan"));

        activate(&mut e, 0, "taoluan");
        answer(&mut e, "taoluan:choose_card", vec![cd(equip)]);
        answer(&mut e, "taoluan:choose_virtual", vec![tag(3)]); // Wuzhong
        answer(&mut e, "taoluan:choose_helper", vec![pl(1)]);
        snapshot_roundtrip(&e, &p, Vec::new());
        let hp0 = hp(&e, 0);
        // Helper refuses -> owner loses 1 HP and taoluan is disabled for the turn
        answer(&mut e, "taoluan:helper_give", Vec::new());
        assert_eq!(hp(&e, 0), hp0 - 1);
        assert_eq!(flag(&e, 0, "taoluan_disabled_turn"), 1);
        assert!(is_rejected(activate(&mut e, 0, "taoluan")));
    }

    // 3. qimou & 4. reqimou: limited skill, lose X HP, distance -X, +X Sha, reqimou draws X
    {
        let mut e = fresh(&p, 2707, &[&["qimou", "reqimou"], &[], &[]]);
        activate(&mut e, 0, "qimou");
        assert!(rejected(&mut e, vec![tag(9)]));
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "qimou:choose_hp", vec![tag(1)]);
        assert_eq!(hp(&e, 0), 4);
        assert!(is_rejected(activate(&mut e, 0, "qimou")));

        activate(&mut e, 0, "reqimou");
        snapshot_roundtrip(&e, &p, vec![tag(2)]);
        answer(&mut e, "reqimou:choose_hp", vec![tag(2)]);
        assert_eq!(hp(&e, 0), 2);
        assert_eq!(hand(&e, 0), 2);
        assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1);
        assert_eq!(
            eval_sha_use_limit(
                e.state_core(),
                e.catalog(),
                0,
                None,
                CardKind::Sha,
                DamageNature::Normal,
                1
            ),
            (3, false)
        );
        assert!(is_rejected(activate(&mut e, 0, "reqimou")));
    }

    // 5. jinghe: show distinct-suit cards, pick N targets, grant skills from pool
    {
        let mut e = fresh(&p, 2708, &[&["jinghe"], &[]]);
        let sp = card_by(&e, |_, c| c.suit == Suit::Spade);
        let sp2 = card_by(&e, |id, c| c.suit == Suit::Spade && id != sp);
        let ht = card_by(&e, |_, c| c.suit == Suit::Heart);
        give(&mut e, 0, sp);
        give(&mut e, 0, sp2);
        give(&mut e, 0, ht);
        activate(&mut e, 0, "jinghe");
        snapshot_roundtrip(&e, &p, Vec::new());
        answer(&mut e, "jinghe:show_cards", Vec::new());
        assert!(!once_used(&e, 0, "jinghe"));

        activate(&mut e, 0, "jinghe");
        // ③ Invalid card rejected
        assert!(rejected(&mut e, vec![cd(999)]));
        answer(&mut e, "jinghe:show_cards", vec![cd(sp), cd(ht)]);
        snapshot_roundtrip(&e, &p, vec![pl(0), pl(1)]);
        answer(&mut e, "jinghe:choose_targets", vec![pl(0), pl(1)]);
        assert_eq!(flag(&e, 0, "jinghe_granted_skill"), 1);
        assert_eq!(flag(&e, 1, "jinghe_granted_skill"), 2);
        assert_eq!(flag(&e, 0, "jinghe_active_round"), 1);
        assert!(is_rejected(activate(&mut e, 0, "jinghe")));
    }
}

#[test]
fn l27_batch_a_trigger_and_modifier_skills() {
    let p = pack();

    // 1. kuangcai + 2. sbtongye + 3. baonvezhi + 4. clandianzhan
    {
        let mut e = fresh(
            &p,
            2709,
            &[&["kuangcai", "sbtongye", "baonvezhi", "clandianzhan"], &[]],
        );
        let sha1 = card_by(&e, |_, c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
        let sha2 = card_by(&e, |id, c| {
            c.kind == CardKind::Sha && c.suit == Suit::Spade && id != sha1
        });
        assert!(
            eval_sha_use_limit(e.state_core(), e.catalog(), 0, None, CardKind::Sha, DamageNature::Normal, 1).1
        );
        assert!(eval_card_distance_unlimited(
            e.state_core(), e.catalog(), 0, sha1, CardKind::Sha, DamageNature::Normal
        ));
        give(&mut e, 0, sha1);
        give(&mut e, 0, sha2);
        play(&mut e, 0, sha1, vec![1]);
        drain(&mut e);
        assert!(e.state_core().player(1).linked);
        assert_eq!(flag(&e, 0, "clandianzhan_suit_mask") & 1, 1);
        assert_eq!(flag(&e, 0, "baonvezhi"), 1);
        let h_after_1 = hand(&e, 0);
        // ④ 2nd Spade Sha does not re-trigger clandianzhan
        play(&mut e, 0, sha2, vec![1]);
        drain(&mut e);
        assert_eq!(hand(&e, 0), h_after_1 - 1);

        clear_hand(&mut e, 0);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard (kuangcai)
        drain(&mut e);
        assert_eq!(flag(&e, 0, "kuangcai_delta"), 1);
        assert_eq!(eval_draw_count(e.state_core(), e.catalog(), 0, 2), 3);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End (sbtongye:predict)
        assert!(rejected(&mut e, vec![tag(9)]));
        snapshot_roundtrip(&e, &p, vec![tag(0)]);
        answer(&mut e, "sbtongye:predict", vec![tag(0)]);
        drain(&mut e);
    }

    // 5. rehuashen + 6. rexinsheng
    {
        let mut e = fresh(&p, 2710, &[&["rehuashen", "rexinsheng"], &[]]);
        let _ = hand_turn_to(&mut e, 0, 1); // triggers TurnBegin -> rehuashen:choose_skill
        snapshot_roundtrip(&e, &p, vec![tag(0)]);
        answer(&mut e, "rehuashen:choose_skill", vec![tag(0)]);
        drain(&mut e);
        assert_eq!(flag(&e, 0, "rehuashen_init"), 1);
        assert_eq!(flag(&e, 0, "rehuashen_pool_count"), 3);

        // Seat 1 deals 1 damage to Seat 0 -> rexinsheng adds +1 huashen card (3 -> 4)
        e.debug_set_turn(1, Phase::Play, 1);
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        give(&mut e, 1, sha);
        play(&mut e, 1, sha, vec![0]);
        drain(&mut e);
        assert_eq!(flag(&e, 0, "rehuashen_pool_count"), 4);
    }

    // 7. shouxi + 8. dinghan
    {
        let mut e = fresh(&p, 2711, &[&[], &["shouxi", "dinghan"]]);
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        let duel1 = card_by(&e, |_, c| c.kind == CardKind::Duel);
        let duel2 = card_by(&e, |id, c| c.kind == CardKind::Duel && id != duel1);
        let tao = card_by(&e, |_, c| c.kind == CardKind::Tao);
        give(&mut e, 0, sha);
        give(&mut e, 0, duel1);
        give(&mut e, 0, duel2);
        give(&mut e, 0, tao);
        let hp1 = hp(&e, 1);
        play(&mut e, 0, sha, vec![1]);
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "shouxi:declare_kind", vec![tag(1)]); // Shan
        snapshot_roundtrip(&e, &p, Vec::new());
        answer(&mut e, "shouxi:attacker_discard", Vec::new()); // attacker refuses -> Sha nullified
        drain(&mut e);
        assert_eq!(flag(&e, 1, "shouxi_nullified"), 1);
        assert_eq!(hp(&e, 1), hp1);

        // 1st Duel -> dinghan records & nullifies Duel
        play(&mut e, 0, duel1, vec![1]);
        drain(&mut e);
        assert_eq!(flag(&e, 1, "dinghan_nullified"), 1);
        assert_eq!(hp(&e, 1), hp1);
        // ④ 2nd Duel -> already recorded, so dinghan does not trigger and Seat 1 takes 1 damage
        play(&mut e, 0, duel2, vec![1]);
        drain(&mut e);
        assert_eq!(hp(&e, 1), hp1 - 1);
    }

    // 9. quanji + 10. potzhuangshi + 11. sbjianxiong
    {
        let mut e = fresh(&p, 2712, &[&["quanji", "potzhuangshi"], &["sbjianxiong"]]);
        set_flag(&mut e, 1, "sbjianxiong", 1);
        set_flag(&mut e, 1, "sbjianxiong_marks", 1);
        // Seat 0 Play -> Discard without HP loss -> triggers quanji:confirm
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "quanji:confirm", vec![tag(1)]);
        let c = open(&e, "quanji:put_card").spec.options[0];
        answer(&mut e, "quanji:put_card", vec![c]);
        assert_eq!(flag(&e, 0, "quanji_count"), 1);
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 6);

        // Seat 0 Draw -> Play begin -> triggers potzhuangshi
        let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
        let extra = card_by(&e, |id, c| c.kind == CardKind::Tao && id != sha);
        give(&mut e, 0, sha);
        give(&mut e, 0, extra);
        e.debug_set_turn(0, Phase::Draw, 2);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        snapshot_roundtrip(&e, &p, vec![cd(extra)]);
        answer(&mut e, "potzhuangshi:discard_cards", vec![cd(extra)]);
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "potzhuangshi:lose_hp", vec![tag(1)]);
        drain(&mut e);
        assert_eq!(flag(&e, 0, "potzhuangshi_unblockable_x"), 1);
        assert_eq!(flag(&e, 0, "potzhuangshi_damage_y"), 1);
        assert!(eval_card_unrespondable(
            e.state_core(), e.catalog(), 0, 1, sha, CardKind::Sha, DamageNature::Normal
        ));
        assert_eq!(
            eval_damage_amount(e.state_core(), e.catalog(), Some(0), 1, Some(sha), DamageNature::Normal, 1),
            2
        );

        // Seat 0 plays Sha on Seat 1 -> deals 2 damage -> triggers sbjianxiong:confirm on Seat 1
        play(&mut e, 0, sha, vec![1]);
        snapshot_roundtrip(&e, &p, vec![tag(1)]);
        answer(&mut e, "sbjianxiong:confirm", vec![tag(1)]);
        answer(&mut e, "sbjianxiong:remove_mark", vec![tag(1)]);
        drain(&mut e);
        assert_eq!(flag(&e, 1, "sbjianxiong"), 0);
        assert!(e.state_core().player(1).hand.contains(&sha));
    }
}
