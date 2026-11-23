//! U05-2 (`gujian` 古剑奇谭 · 136 项：18 武将 / 36 卡牌 / 82 技能) 上游等价回归套件。
//!
//! 上游来源：`archive/upstream/noname/apps/core/extension/杀海拾遗/`：
//! - `card/gujian/{card,skill}.js` (32 卡牌 + 23 卡牌/状态技)
//! - `character/gujian/{character,card,skill}.js` (18 武将 + 4 衍射牌 + 59 武将技)
//!
//! 约束：全程真实对局事件流（`PlayCard`/`ActivateSkill`/`Answer`/`NextPhase`/AI 公示投影/字节存档恢复），
//! 零 `debug_execute_effect`，零猜测断言。

#[path = "u05_b2_gujian/cards_foods.rs"]
mod cards_foods;
#[path = "u05_b2_gujian/chars_ab.rs"]
mod chars_ab;
#[path = "u05_b2_gujian/chars_cd.rs"]
mod chars_cd;
#[path = "u05_b2_gujian/chars_e.rs"]
mod chars_e;
#[path = "u05_b2_gujian/round2_fixes.rs"]
mod round2_fixes;
#[path = "u05_b2_gujian/round3_fixes.rs"]
mod round3_fixes;
#[path = "u05_b2_gujian/round4_fixes.rs"]
mod round4_fixes;

pub mod common {
    use content_native_loader::{
        prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
        validate_and_freeze_raw_package, LockedStartMode, U05_GUJIAN_ALL_CARDS,
        U05_GUJIAN_GENERALS,
    };
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::catalog::{CardKind, Suit};
    use engine_native::content::extension::ExtensionPack;
    use engine_native::{
        ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, StateSnapshot,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DEDUP: AtomicU64 = AtomicU64::new(9_100_000);

    pub fn next_dedup() -> u64 {
        DEDUP.fetch_add(1, Ordering::Relaxed)
    }

    pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
        e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action,
        })
    }

    pub fn gujian_verified_and_engine(
        seed: u64,
        seats: u8,
    ) -> (content_native_loader::VerifiedLockedContent, Engine) {
        let ext_root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
        let pkg_dir = ext_root.join("shahai-gujian-upstream-noname");
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap()
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap()
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze gujian");
        let verified =
            prepare_locked_content_with_bridge(vec![frozen]).expect("bridge gujian verified");
        let engine = start_locked_engine(seed, usize::from(seats), LockedStartMode::Prototype, &verified)
            .expect("start gujian engine");
        (verified, engine)
    }

    #[allow(dead_code)]
    pub fn gujian_pack_and_engine(seed: u64, seats: u8) -> (ExtensionPack, Engine) {
        let ext_root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
        let json =
            std::fs::read_to_string(ext_root.join("shahai-gujian-upstream-noname/assets/cards.json"))
                .expect("read gujian assets/cards.json");
        let out = content_native_loader::bridge_pack(&json).expect("bridge gujian cards pack");
        let engine = Engine::new_with_extensions(seed, usize::from(seats), &[&out.pack]).expect("start engine");
        (out.pack, engine)
    }

    pub fn drain(e: &mut Engine) {
        for _ in 0..48 {
            let Some(c) = e.open_choice().cloned() else { break };
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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

    pub fn answer_choice(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
        let c = e.open_choice().cloned().expect("expected open_choice");
        submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection,
            },
        )
    }

    pub fn pass_nullify_window(e: &mut Engine) {
        while let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt != "nullify_window" {
                break;
            }
            let _ = submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
    }

    pub fn drain_until_prompt(e: &mut Engine, prompt: &str) {
        for _ in 0..24 {
            let Some(c) = e.open_choice().cloned() else { return };
            if c.spec.prompt == prompt {
                return;
            }
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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

    pub fn advance_to_phase(e: &mut Engine, seat: u8, phase: Phase) {
        for _ in 0..128 {
            if e.turn().current == seat && e.turn().phase == phase && e.open_choice().is_none() {
                return;
            }
            if e.open_choice().is_some() {
                drain(e);
                continue;
            }
            let actor = e.turn().current;
            let _ = submit(e, actor, GameAction::NextPhase);
            drain(e);
        }
        panic!("128 步内未能进入座位 {seat} 阶段 {phase:?}");
    }

    pub fn clear_all_hands(e: &mut Engine) {
        let seats = e.state_core().players.len() as u8;
        for p in 0..seats {
            e.debug_clear_hand(p);
        }
    }

    pub fn move_card_to_hand(e: &mut Engine, seat: u8, cid: u32) {
        let core = e.state_core_mut();
        for p in 0..core.players.len() as u8 {
            core.player_mut(p).hand.retain(|&x| x != cid);
            core.player_mut(p).equips.retain(|&x| x != cid);
            core.player_mut(p).judge.retain(|&x| x != cid);
            core.player_mut(p).off_court.retain(|&x| x != cid);
        }
        core.deck.retain(|&x| x != cid);
        core.discard.retain(|&x| x != cid);
        core.table.retain(|&x| x != cid);
        core.detached_cards.retain(|&(x, _)| x != cid);
        core.player_mut(seat).hand.push(cid);
    }

    pub fn find_card_by(
        e: &Engine,
        mut pred: impl FnMut(u32, &engine_native::catalog::CardDef) -> bool,
    ) -> u32 {
        e.catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(idx, def)| pred(*idx as u32, def))
            .map(|(idx, _)| idx as u32)
            .expect("matching card in catalog")
    }

    pub fn find_unused_card_by(
        e: &Engine,
        used: &[u32],
        mut pred: impl FnMut(&engine_native::catalog::CardDef) -> bool,
    ) -> u32 {
        find_card_by(e, |id, d| !used.contains(&id) && pred(d))
    }

    pub fn matches_gujian_name(card_name: &str, name: &str) -> bool {
        card_name == name
            || card_name.starts_with(name)
            || card_name
                .strip_prefix("gj_")
                .is_some_and(|s| s == name || s.starts_with(name))
    }

    pub fn move_named_to_hand(e: &mut Engine, seat: u8, name_prefix: &str) -> u32 {
        let cid = find_card_by(e, |_, d| matches_gujian_name(d.name, name_prefix));
        move_card_to_hand(e, seat, cid);
        cid
    }

    pub fn move_kind_suit_to_hand(
        e: &mut Engine,
        seat: u8,
        kind: CardKind,
        suit: Suit,
        used: &mut Vec<u32>,
    ) -> u32 {
        let cid = find_unused_card_by(e, used, |d| d.kind == kind && d.suit == suit);
        used.push(cid);
        move_card_to_hand(e, seat, cid);
        cid
    }

    pub fn flag(e: &Engine, p: u8, key: &str) -> u32 {
        e.state_core()
            .vm_aux
            .flags
            .get(&(p, key.to_string()))
            .copied()
            .unwrap_or(0)
    }

    pub fn save_restore_verified(
        e: &Engine,
        verified: &content_native_loader::VerifiedLockedContent,
    ) -> Engine {
        let saved: StateSnapshot = e.save();
        let bytes = encode_archive(&saved);
        let decoded = decode_archive(&bytes).expect("decode archive");
        restore_locked_engine(&decoded, verified).expect("restore locked engine")
    }

    pub fn enemy_seat(e: &Engine, owner: u8) -> u8 {
        let n = e.state_core().players.len() as u8;
        (0..n)
            .find(|&p| p != owner && !e.state_core().is_friend_of(e.catalog(), owner, p))
            .unwrap_or((owner + 1) % n)
    }

    pub fn assert_zone_conservation_all(e: &Engine) {
        let snap = e.save();
        let mut ids: Vec<u32> = Vec::new();
        for p in &snap.core.players {
            ids.extend(p.hand.iter().copied());
            ids.extend(p.equips.iter().copied());
            ids.extend(p.judge.iter().copied());
            ids.extend(p.off_court.iter().copied());
        }
        ids.extend(snap.core.deck.iter().copied());
        ids.extend(snap.core.discard.iter().copied());
        ids.extend(snap.core.table.iter().copied());
        ids.extend(snap.core.detached_cards.iter().map(|&(c, _)| c));
        ids.extend(snap.core.vm_aux.piles.values().flatten().copied());
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "实体牌跨区重复或丢失");
        assert_eq!(ids.len(), e.catalog().cards.len(), "全区总牌数必须与目录一致");
    }

    pub fn roster_generals() -> &'static [(&'static str, engine_native::catalog::Faction, engine_native::content::standard::Gender, u32, &'static [&'static str]); 18] {
        &U05_GUJIAN_GENERALS
    }

    pub fn roster_cards() -> &'static [(&'static str, CardKind, u32, Suit, u8); 36] {
        &U05_GUJIAN_ALL_CARDS
    }
}

/// 静态核对：18 古剑武将、36 张古剑卡牌、82 项技能全宿主化（无 VM 占位），
/// 且 `yanjiadan_*` (4 张) 与 `gjyuheng_{plus,pro}` (2 张) 开局进入 `detached_cards`（不入牌堆）。
#[test]
fn u05_2_gujian_18_generals_36_cards_82_skills_roster_parity() {
    use common::*;
    let (_verified, mut e) = gujian_verified_and_engine(2026_1120, 4);

    // 1. 6 张离牌堆牌不进初始摸牌堆，位于 detached_cards
    let core = e.state_core();
    for off_deck in [
        "yanjiadan_heart",
        "yanjiadan_diamond",
        "yanjiadan_club",
        "yanjiadan_spade",
        "gjyuheng_plus",
        "gjyuheng_pro",
    ] {
        let cid = find_card_by(&e, |_, d| d.name == off_deck || d.name.ends_with(off_deck));
        assert!(
            !core.deck.contains(&cid) && core.detached_cards.iter().any(|&(c, _)| c == cid),
            "{off_deck} (id={cid}) must start in detached_cards, not in deck"
        );
    }

    // 2. 全部 36 张古剑卡牌进入 Catalog
    for &(cname, _, _, _, _) in roster_cards() {
        let found = e
            .catalog()
            .cards
            .iter()
            .any(|c| c.name == cname || c.name.contains(cname));
        assert!(found, "gujian card {cname} missing from catalog");
    }

    // 3. 全部 18 名古剑武将装配后拥有准确势力/性别/上限与真实原生宿主技（非 VM 占位）
    for &(gname, faction, gender, max_hp, skills) in roster_generals() {
        e.assign_general_named(0, gname)
            .unwrap_or_else(|err| panic!("assign_general_named({gname}) failed: {err:?}"));
        let p = e.state_core().player(0);
        assert_eq!(p.max_hp, max_hp, "{gname} max_hp");
        let gid = p.general_slots[0].expect("slot 0 populated");
        let (_, meta_faction, _, meta_hp, meta_gender) = e.catalog().general_meta(gid).unwrap();
        assert_eq!(meta_faction, faction, "{gname} faction");
        assert_eq!(meta_hp, max_hp, "{gname} meta max_hp");
        assert_eq!(meta_gender, gender, "{gname} gender");
        for &sk in skills {
            let lookup_name = match sk {
                "xuanning" => "shahai-gujian__ref_xuanning",
                "dangping" => "shahai-gujian__ref_dangping",
                other => other,
            };
            let def = e
                .catalog()
                .skill_by_name(lookup_name)
                .unwrap_or_else(|| panic!("{gname} skill {lookup_name} missing"));
            assert!(
                e.catalog().skills[def as usize]
                    .behavior
                    .package_skill_id()
                    .is_none(),
                "{gname} skill {lookup_name} must be native HostSkillBehavior, not VM stub"
            );
            assert!(
                e.state_core().has_skill(0, def),
                "{gname} missing skill {lookup_name}"
            );
        }
    }
    assert_zone_conservation_all(&e);
}
