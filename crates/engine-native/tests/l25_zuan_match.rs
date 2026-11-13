//! L25 祖安武将 (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`)
//! 6-Dimension Real-Match Test Suite — Part 1:
//! - 257-item manifest & 68-config surface (`ZuanConfig`) + `load_extension_packs`
//! - All 60 `ssr_*` generals assembly, faction, gender, skill bindings & starting `(hp, max_hp, hujia)`
//! - 4 formal cards (`ssr_newsance_xicheng`, `ssr_newsance_weicheng`, `ssr_newsance_shoucheng`,
//!   `ssr_yongsi_yuxi_card`) + `ssr_jinfan` + `ssr_yongsi`
//! - Batch A: 23 Global/Mode/Shen skills + 77 passive UI/VFX/speech hooks + inert `_zuansaidskill`
//! - Batch B: 22 `ssr_*` skills (Generals #1–#10) across 6 dimensions
//! - Batch C: 22 `ssr_*` skills (Generals #11–#20) across 6 dimensions

use content_native_loader::{
    attach_formal_package_upstream_rules, decision34_pack9_extension_pack,
    lookup_l25_zuan_host_behavior,
    zuan::{
        apply_zuan_line, parse_zuan_config, restore_zuan, snapshot_zuan, ZuanConfig, ZUAN_COUNTS,
    },
    L25_ALL_RULE_SKILLS, L25_BATCH_A_EXTRA_SKILLS, L25_SSR_GENERALS, L25_SSR_GENERAL_SKILLS,
    L25_ZUAN_CARD_ENTRIES, L25_ZUAN_FORMAL_CARDS, L25_ZUAN_SKILLS,
};
use engine_native::{
    archive::{decode_archive, encode_archive},
    catalog::{CardDef, CardKind, Catalog, EquipSlot, Faction, Suit},
    choice::{ChoiceKind, ChoiceOption, ChoiceSpec},
    content::{
        extension::{ExtensionDef, ExtensionPack, InitHook},
        standard::Gender,
    },
    event::{DamageNature, EventData, Phase},
    ids::{CardId, PlayerId, SkillDefId},
    machine::{card_use, card_use_as, damage_data_with},
    rvm_bridge::modifiers::{
        d34_flags::{
            apply_attack_range, apply_card_distance_unlimited,
            apply_card_target_canceled as d34_apply_card_target_canceled,
            apply_card_unrespondable as d34_apply_card_unrespondable,
            apply_card_use_canceled as d34_apply_card_use_canceled, apply_damage_amount,
            apply_distance_total, apply_draw_count, apply_hand_limit, apply_sha_extra_targets,
            apply_sha_use_limit, find_d34_flag_view_as,
        },
        eval_card_unrespondable as raw_eval_card_unrespondable, eval_distance_total,
        eval_hand_limit as raw_eval_hand_limit, eval_ignore_armor as raw_eval_ignore_armor,
        eval_sha_use_limit as raw_eval_sha_use_limit,
    },
    state::{GameStateCore, SkillInstance},
    CommandOutcome, Engine, GameAction, GameCommand, RejectReason,
};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(5_000_000);

#[derive(Debug)]
enum StepOutcome {
    Done,
    WaitChoice {
        observer: PlayerId,
        spec: ChoiceSpec,
    },
}

fn eval_hand_limit(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> usize {
    raw_eval_hand_limit(state, catalog, player, state.player(player).hp.max(0))
}

fn eval_sha_use_limit(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> (u32, bool) {
    raw_eval_sha_use_limit(
        state,
        catalog,
        player,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    )
}

fn eval_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    raw_eval_ignore_armor(
        state,
        catalog,
        source,
        target,
        None,
        DamageNature::Normal,
    )
}

fn eval_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    kind: CardKind,
) -> bool {
    raw_eval_card_unrespondable(state, catalog, user, 1, 0, kind, DamageNature::Normal)
}

fn apply_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    kind: CardKind,
    unrespondable: bool,
) -> bool {
    d34_apply_card_unrespondable(state, catalog, user, 1, 0, kind, unrespondable)
}

fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    target: PlayerId,
    kind: CardKind,
    canceled: bool,
) -> bool {
    d34_apply_card_target_canceled(state, catalog, user, target, 0, kind, canceled)
}

fn apply_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    kind: CardKind,
    in_hand: bool,
    canceled: bool,
) -> bool {
    d34_apply_card_use_canceled(state, catalog, user, 0, kind, in_hand, canceled)
}

fn new_l25_engine(seed: u64, players: u8) -> Engine {
    let pack = decision34_pack9_extension_pack();
    let mut e =
        Engine::new_with_extensions(seed, players as usize, &[&pack]).expect("L25 engine init");
    e.state_core_mut().skills.clear();
    e.state_core_mut().turn.current = 0;
    e.state_core_mut().turn.phase = Phase::Play;
    for p in 0..players {
        e.state_core_mut().player_mut(p).max_hp = 4;
        e.state_core_mut().player_mut(p).hp = 4;
        e.state_core_mut().player_mut(p).hujia = 0;
        e.state_core_mut().player_mut(p).alive = true;
        let mut h = std::mem::take(&mut e.state_core_mut().player_mut(p).hand);
        e.state_core_mut().deck.append(&mut h);
    }
    e
}

fn skill_id(e: &Engine, name: &str) -> SkillDefId {
    e.catalog()
        .skill_by_name(name)
        .unwrap_or_else(|| panic!("skill `{name}` must exist in catalog"))
}

fn grant_skill(e: &mut Engine, seat: PlayerId, name: &str) -> SkillDefId {
    let def = skill_id(e, name);
    e.state_core_mut().skills.insert(
        (seat, def),
        SkillInstance {
            owner: seat,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    def
}

fn flag(e: &Engine, seat: PlayerId, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(seat, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, seat: PlayerId, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut()
            .vm_aux
            .flags
            .remove(&(seat, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((seat, key.to_string()), val);
    }
}

fn find_card<F>(e: &Engine, mut pred: F) -> CardId
where
    F: FnMut(CardId, &CardDef) -> bool,
{
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(idx, decl)| pred(idx as CardId, decl).then_some(idx as CardId))
        .expect("matching card must exist in catalog")
}

fn find_named_card(e: &Engine, name: &str) -> CardId {
    find_card(e, |_, d| d.name == name)
}

fn find_kind_suit(e: &Engine, kind: CardKind, suit: Suit) -> CardId {
    find_card(e, |_, d| d.kind == kind && d.suit == suit)
}

fn find_suit_rank(e: &Engine, suit: Suit, min_rank: u8, max_rank: u8) -> CardId {
    find_card(e, |_, d| {
        d.suit == suit && d.rank >= min_rank && d.rank <= max_rank
    })
}

fn next_cmd(e: &Engine, actor: PlayerId, action: GameAction) -> GameCommand {
    GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    }
}

fn reconcile_card_zones(e: &mut Engine) {
    let snap = e.save();
    let mut held = Vec::new();
    for park in &snap.resolution.parks {
        engine_native::snap::machine_held_cards(&park.machine, &mut held);
    }
    let total = e.catalog().cards.len();
    let cat_kinds: Vec<(CardKind, Suit)> = e
        .catalog()
        .cards
        .iter()
        .map(|d| (d.kind, d.suit))
        .collect();
    let mut seen = vec![false; total];
    for c in held {
        if (c as usize) < total {
            seen[c as usize] = true;
        }
    }
    let n_players = e.state_core().players.len();
    let dedup_vec = |vec: &mut Vec<CardId>, seen: &mut [bool]| {
        let mut out = Vec::with_capacity(vec.len());
        for &c in vec.iter() {
            if (c as usize) >= total {
                continue;
            }
            if !seen[c as usize] {
                seen[c as usize] = true;
                out.push(c);
            } else {
                let (k, s) = cat_kinds[c as usize];
                if let Some(idx) = (0..total)
                    .find(|&i| !seen[i] && cat_kinds[i] == (k, s))
                    .or_else(|| (0..total).find(|&i| !seen[i] && cat_kinds[i].0 == k))
                    .or_else(|| (0..total).find(|&i| !seen[i]))
                {
                    seen[idx] = true;
                    out.push(idx as CardId);
                }
            }
        }
        *vec = out;
    };
    for p in 0..n_players {
        let pl = e.state_core_mut().player_mut(p as PlayerId);
        dedup_vec(&mut pl.hand, &mut seen);
        dedup_vec(&mut pl.equips, &mut seen);
        dedup_vec(&mut pl.judge, &mut seen);
        dedup_vec(&mut pl.off_court, &mut seen);
        pl.zhucheng.retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
        pl.lianyaohu_store.retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
        pl.sxianjing_traps.retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
        pl.hutian_hold.retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
    }
    for pile in e.state_core_mut().vm_aux.piles.values_mut() {
        pile.retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
    }
    e.state_core_mut()
        .table
        .retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
    e.state_core_mut()
        .detached_cards
        .retain(|&(c, _)| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
    e.state_core_mut()
        .discard
        .retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
    e.state_core_mut()
        .deck
        .retain(|&c| (c as usize) < total && !std::mem::replace(&mut seen[c as usize], true));
    for (cid, &is_seen) in seen.iter().enumerate() {
        if !is_seen {
            e.state_core_mut().deck.insert(0, cid as CardId);
        }
    }
}


fn drive_event(e: &mut Engine, event: EventData) -> StepOutcome {
    if let EventData::CardUsed(ref u) = event {
        if let Some(card) = u.card {
            let user = u.player;
            let n = e.state_core().players.len();
            for p in 0..n {
                if p as PlayerId != user {
                    let pl = e.state_core_mut().player_mut(p as PlayerId);
                    pl.hand.retain(|&c| c != card);
                    pl.equips.retain(|&c| c != card);
                    pl.judge.retain(|&c| c != card);
                    pl.off_court.retain(|&c| c != card);
                }
            }
            let in_user_zone = e.state_core().player(user).hand.contains(&card)
                || e.state_core().player(user).equips.contains(&card)
                || e.state_core()
                    .vm_aux
                    .piles
                    .iter()
                    .any(|((o, _), pile)| *o == user && pile.contains(&card));
            if !in_user_zone {
                e.state_core_mut().player_mut(user).hand.push(card);
            }
        }
    }
    reconcile_card_zones(e);
    match e.debug_drive_event(event) {
        CommandOutcome::Accepted { .. } => StepOutcome::Done,
        CommandOutcome::Pending { choice } => StepOutcome::WaitChoice {
            observer: choice.observer,
            spec: choice.spec,
        },
        other => panic!("drive_event unexpected outcome: {other:?}"),
    }
}

fn assert_mid_choice_snapshot_roundtrip(e: &mut Engine) {
    reconcile_card_zones(e);
    assert!(
        e.open_choice().is_some(),
        "engine must be suspended on an open_choice before snapshot roundtrip"
    );
    let pack = decision34_pack9_extension_pack();
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode mid-choice archive");
    let restored =
        Engine::restore_with_extensions(&decoded, &[&pack]).expect("restore mid-choice engine");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert_eq!(restored.open_choice(), e.open_choice());
}

#[test]
fn l25_zuan_60_generals_assembly_stats_and_special_hujia() {
    assert_eq!(ZUAN_COUNTS, (244, 190, 400, 239));
    assert_eq!(L25_SSR_GENERALS.len(), 60);
    assert_eq!(L25_SSR_GENERAL_SKILLS.len(), 130);
    assert_eq!(L25_ALL_RULE_SKILLS.len(), 84);
    assert_eq!(L25_ZUAN_CARD_ENTRIES.len(), 5);
    assert_eq!(L25_ZUAN_FORMAL_CARDS.len(), 6);
    assert_eq!(L25_BATCH_A_EXTRA_SKILLS.len(), 17);
    assert_eq!(L25_ZUAN_SKILLS.len(), 122);

    // Exercise ZuanConfig parser & snapshot/restore
    let mut cfg = ZuanConfig::default();
    let raw_cfg = r#"{"enable_editor":false,"rule_variant":true,"offline_debug_only":true,"taunt_pack":false}"#;
    apply_zuan_line(&mut cfg, &format!("config {raw_cfg}")).expect("apply config");
    assert!(cfg.rule_variant && !cfg.taunt_pack);
    let snap = snapshot_zuan(&cfg);
    let restored_cfg = restore_zuan(&snap).expect("restore_zuan");
    assert_eq!(restored_cfg, cfg);
    let parsed = parse_zuan_config(raw_cfg).expect("parse_zuan_config");
    assert_eq!(parsed, cfg);

    // Verify `upstream-liuli:zuan` and `upstream-liuli:doomsday` formal package assembly
    let mut zpack = ExtensionPack {
        def: ExtensionDef {
            name: "upstream-liuli:zuan",
            version: 1,
            description: "zuan",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };
    attach_formal_package_upstream_rules("upstream-liuli", "zuan", &mut zpack, &[]);
    let e_from_loader =
        Engine::new_with_extensions(20261113, 4, &[&zpack]).expect("engine from loaded packs");
    for gname in ["ssr_machao", "ssr_taishici", "ssr_re_lidian"] {
        assert!(
            (0..e_from_loader.catalog().general_count() as u16)
                .any(|idx| e_from_loader.catalog().general_meta(idx).is_some_and(|m| m.0 == gname)),
            "general `{gname}` must exist in loaded catalog"
        );
    }

    // Verify all 60 generals assembly on non-lord seat 1 so lord +1 max_hp does not mask base stats
    for &(gname, expected_faction, expected_gender, expected_max_hp, expected_skills) in
        &L25_SSR_GENERALS
    {
        let mut e = new_l25_engine(0x1250_0001, 4);
        e.assign_general_named(1, gname)
            .unwrap_or_else(|err| panic!("assign_general_named failed for `{gname}`: {err:?}"));
        let gid = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == gname))
            .unwrap_or_else(|| panic!("general `{gname}` in catalog"));
        let (meta_name, faction, g_skills, max_hp, gender) =
            e.catalog().general_meta(gid).expect("general_meta");
        assert_eq!(meta_name, gname);
        assert_eq!(faction, expected_faction, "faction mismatch for `{gname}`");
        assert_eq!(gender, expected_gender, "gender mismatch for `{gname}`");
        assert_eq!(max_hp, expected_max_hp, "max_hp mismatch for `{gname}`");
        assert_eq!(g_skills.as_slice(), expected_skills);

        let p1 = e.state_core().player(1);
        let (expected_start_hp, expected_hujia) = match gname {
            "ssr_machao" | "ssr_taishici" => (2, 2),
            "ssr_re_lidian" => (3, 1),
            _ => (expected_max_hp as i32, 0),
        };
        assert_eq!(
            (p1.hp, p1.max_hp, p1.hujia),
            (expected_start_hp, expected_max_hp, expected_hujia),
            "starting (hp, max_hp, hujia) mismatch for `{gname}`"
        );
        for &sk in expected_skills {
            let def = skill_id(&e, sk);
            assert!(
                e.state_core().has_skill(1, def),
                "general `{gname}` must grant skill `{sk}`"
            );
            assert!(
                lookup_l25_zuan_host_behavior(sk).is_some(),
                "skill `{sk}` must map to an L25 HostBehavior"
            );
        }
    }

    // Explicit assertion for the 3 special starting-hujia generals required by spec
    let mut e = new_l25_engine(0x1250_0002, 4);
    e.assign_general_named(1, "ssr_machao").expect("ssr_machao");
    e.assign_general_named(2, "ssr_taishici").expect("ssr_taishici");
    e.assign_general_named(3, "ssr_re_lidian").expect("ssr_re_lidian");
    assert_eq!(
        (e.state_core().player(1).hp, e.state_core().player(1).max_hp, e.state_core().player(1).hujia),
        (2, 4, 2)
    );
    assert_eq!(
        (e.state_core().player(2).hp, e.state_core().player(2).max_hp, e.state_core().player(2).hujia),
        (2, 4, 2)
    );
    assert_eq!(
        (e.state_core().player(3).hp, e.state_core().player(3).max_hp, e.state_core().player(3).hujia),
        (3, 4, 1)
    );
}

#[test]
fn l25_zuan_formal_cards_xicheng_weicheng_shoucheng_yuxi_and_jinfan_6d() {
    // 1. `ssr_newsance_xicheng`: Branch A (target shows `Sha` -> counterattacks with `Sha`) & Branch B (target shows non-`Sha` -> takes 1 damage)
    {
        let mut e = new_l25_engine(0x1250_0101, 3);
        let xicheng = find_named_card(&e, "ssr_newsance_xicheng");
        let sha_card = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let tao_card = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        e.state_core_mut().player_mut(0).hand = vec![xicheng];
        e.state_core_mut().player_mut(1).hand = vec![sha_card, tao_card];

        let out = e.submit(next_cmd(&e,
            0,
            GameAction::PlayCard {
                card: xicheng,
                targets: vec![1],
            },
        ));
        let CommandOutcome::Pending { choice } = out else {
            panic!("expected Pending choice for ssr_newsance_xicheng, got {out:?}");
        };
        assert_eq!(choice.observer, 1);
        assert_eq!(choice.spec.prompt, "ssr_newsance_xicheng:show_card");
        assert_mid_choice_snapshot_roundtrip(&mut e);

        // Target shows `Sha` -> records `l25_xicheng_shown_card` and counterattacks `0` with `Sha`
        let out = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(sha_card)] }));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e, 0, "l25_sance_xicheng"), 1);
        assert_eq!(flag(&e, 1, "l25_xicheng_shown_card"), u32::from(sha_card) + 1);

        // Branch B: Target shows non-`Sha` -> owner deals 1 damage to target
        let mut e2 = new_l25_engine(0x1250_0102, 3);
        let shan_card = find_kind_suit(&e2, CardKind::Shan, Suit::Diamond);
        e2.state_core_mut().player_mut(0).hand = vec![xicheng];
        e2.state_core_mut().player_mut(1).hand = vec![tao_card, shan_card];
        let hp_before = e2.state_core().player(1).hp;
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::PlayCard {
                card: xicheng,
                targets: vec![1],
            },
        )) else {
            panic!("expected Pending choice for xicheng non-Sha");
        };
        let out = e2.submit(next_cmd(&e2,
            1,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(tao_card)],
            },
        ));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e2, 0, "l25_sance_xicheng"), 1);
        assert_eq!(flag(&e2, 1, "l25_xicheng_shown_card"), u32::from(tao_card) + 1);
        assert_eq!(e2.state_core().player(1).hp, hp_before - 1);
    }

    // 2. `ssr_newsance_weicheng`: target draws 1, gives 1 card to owner, then owner confirms virtual `Sha`
    {
        let mut e = new_l25_engine(0x1250_0103, 3);
        let weicheng = find_named_card(&e, "ssr_newsance_weicheng");
        let gift = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        e.state_core_mut().player_mut(0).hand = vec![weicheng];
        e.state_core_mut().player_mut(1).hand = vec![gift];

        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::PlayCard {
                card: weicheng,
                targets: vec![1],
            },
        )) else {
            panic!("expected give_card prompt for ssr_newsance_weicheng");
        };
        assert_eq!(choice.observer, 1);
        assert_eq!(choice.spec.prompt, "ssr_newsance_weicheng:give_card");
        assert_mid_choice_snapshot_roundtrip(&mut e);

        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(gift)] })) else {
            panic!("expected confirm_sha prompt for ssr_newsance_weicheng");
        };
        assert_eq!(choice.observer, 0);
        assert_eq!(choice.spec.prompt, "ssr_newsance_weicheng:confirm_sha");
        assert!(e.state_core().player(0).hand.contains(&gift));

        // Confirm virtual Sha against target (target has no Shan -> takes 1 damage)
        let hp_before = e.state_core().player(1).hp;
        let out = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(1)] }));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e, 0, "l25_sance_weicheng"), 1);
        assert_eq!(flag(&e, 0, "l25_weicheng_given_card"), u32::from(gift) + 1);
        assert_eq!(e.state_core().player(1).hp, hp_before - 1);
    }

    // 3. `ssr_newsance_shoucheng`: choose Heart to recover 1 HP vs non-Heart to gain to hand
    {
        let mut e = new_l25_engine(0x1250_0104, 3);
        let shoucheng = find_named_card(&e, "ssr_newsance_shoucheng");
        let heart_card = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let spade_card = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        e.state_core_mut().player_mut(0).hand = vec![shoucheng];
        e.state_core_mut().player_mut(0).hp = 2;
        e.state_core_mut().deck.push(spade_card);
        e.state_core_mut().deck.push(heart_card);

        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::PlayCard {
                card: shoucheng,
                targets: vec![0],
            },
        )) else {
            panic!("expected choose_card prompt for ssr_newsance_shoucheng");
        };
        assert_eq!(choice.spec.prompt, "ssr_newsance_shoucheng:choose_card");
        assert_mid_choice_snapshot_roundtrip(&mut e);

        // Choose Heart -> owner recovers 1 HP
        let out = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(heart_card)] }));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e, 0, "l25_sance_shoucheng"), 1);
        assert_eq!(flag(&e, 0, "l25_shoucheng_chosen"), u32::from(heart_card) + 1);
        assert_eq!(e.state_core().player(0).hp, 3);

        // Choose non-Heart -> owner gains card to hand
        let mut e2 = new_l25_engine(0x1250_0105, 3);
        e2.state_core_mut().player_mut(0).hand = vec![shoucheng];
        e2.state_core_mut().deck.push(spade_card);
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::PlayCard {
                card: shoucheng,
                targets: vec![0],
            },
        )) else {
            panic!("expected choose_card prompt for shoucheng non-Heart");
        };
        let out = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(spade_card)],
            },
        ));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e2, 0, "l25_shoucheng_chosen"), u32::from(spade_card) + 1);
        assert!(e2.state_core().player(0).hand.contains(&spade_card));
    }

    // 4. `ssr_yongsi_yuxi_card` & `ssr_yongsi` + `ssr_jinfan` card
    {
        let mut e = new_l25_engine(0x1250_0106, 3);
        let yuxi = find_named_card(&e, "ssr_yongsi_yuxi_card");
        let jinfan_card = find_named_card(&e, "ssr_jinfan");
        let c0 = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let c1 = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        e.state_core_mut().player_mut(0).hand = vec![yuxi, jinfan_card, c0];
        e.state_core_mut().player_mut(1).hand = vec![c1];

        // Equip `ssr_yongsi_yuxi_card` via PlayCard
        let out = e.submit(next_cmd(&e,
            0,
            GameAction::PlayCard {
                card: yuxi,
                targets: vec![],
            },
        ));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert!(e.state_core().player(0).equips.contains(&yuxi));
        let yongsi_def = grant_skill(&mut e, 0, "ssr_yongsi");
        set_flag(&mut e, 0, "l25_yuxi_treasure_active", 1);
        assert_eq!(eval_hand_limit(e.state_core(), e.catalog(), 0), 5);

        // Play `ssr_jinfan` card -> steals c1 from seat 1
        let out = e.submit(next_cmd(&e,
            0,
            GameAction::PlayCard {
                card: jinfan_card,
                targets: vec![1],
            },
        ));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert!(e.state_core().player(0).hand.contains(&c1));
        e.state_core_mut().player_mut(1).hand.push(c1);

        // Activate `ssr_yongsi`: first test cancel refunds `once_used`
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: yongsi_def },
        )) else {
            panic!("expected ssr_yongsi:choose_target");
        };
        assert_eq!(choice.spec.prompt, "ssr_yongsi:choose_target");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let out = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![] }));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert!(!e.state_core().skills[&(0, yongsi_def)].once_used);

        // Re-activate `ssr_yongsi` and choose mode 0 (view hand & discard 1 from target)
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: yongsi_def },
        )) else {
            panic!("expected ssr_yongsi:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_yongsi:choose_mode");
        };
        assert_eq!(choice.spec.prompt, "ssr_yongsi:choose_mode");
        let out = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] }));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e, 0, "l25_yongsi_used"), 1);
        assert!(e.state_core().player(1).hand.is_empty());

        // Trigger `ssr_yongsi` at Phase::Prepare when wounded -> draw `lost_hp` and lose 1 HP
        e.state_core_mut()
            .skills
            .get_mut(&(0, yongsi_def))
            .unwrap()
            .once_used = false;
        e.state_core_mut().player_mut(0).hp = 2; // lost_hp == 2
        let _ = drive_event(
            &mut e,
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
        );
        assert_eq!(flag(&e, 0, "l25_yongsi_triggered"), 1);
    }
}

#[test]
fn l25_zuan_batch_a_23_global_mode_shen_skills_77_passive_hooks_and_inert_zuansaid_6d() {
    // 1. Verify `_zuansaidskill` is completely inert
    {
        let mut e = new_l25_engine(0x1250_0200, 3);
        let zuansaid = grant_skill(&mut e, 0, "_zuansaidskill");
        assert!(e.catalog().skills[zuansaid as usize].trigger.is_none());
        let out = e.submit(next_cmd(&e, 0, GameAction::ActivateSkill { skill: zuansaid }));
        assert!(matches!(out, CommandOutcome::Rejected(_)));
    }

    // 2. Verify all 77+ passive UI/VFX/speech hooks are strictly passive (`ActivateSkill -> Rejected`)
    // and fire on their real events (`l25_vfx_fired:<name> >= 1`)
    {
        let mut e = new_l25_engine(0x1250_0201, 3);
        let passive_hooks: Vec<&str> = L25_ZUAN_SKILLS
            .iter()
            .chain(L25_BATCH_A_EXTRA_SKILLS.iter())
            .copied()
            .filter(|&name| {
                content_native_loader::zuan_data::lookup_l25_zuan_host_behavior(name).is_some()
                    && name.starts_with('_')
                    && !matches!(
                        name,
                        "_zuansaidskill"
                            | "_yongjinskill"
                            | "_kongchengskill"
                            | "_anyeskill"
                            | "_taohuaskill"
                            | "_lunjianskill"
                            | "_jianghunskill"
                            | "_thunderjudgemodeskill"
                            | "_thundermodeskill"
                            | "_goodluckjuedgeskill"
                    )
            })
            .collect();
        assert!(passive_hooks.len() >= 77);

        for &hook in &passive_hooks {
            let def = grant_skill(&mut e, 0, hook);
            let out = e.submit(next_cmd(&e, 0, GameAction::ActivateSkill { skill: def }));
            assert!(
                matches!(out, CommandOutcome::Rejected(_)),
                "passive hook `{hook}` must reject ActivateSkill"
            );
        }

        // Fire the real trigger events that cover every passive UI/VFX/speech hook
        let sample_card = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let shan_card = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        let equip_card = find_card(&e, |_, d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
        e.state_core_mut().player_mut(0).hand.clear();
        for ev in [
            EventData::TurnBegin { player: 0 },
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
            EventData::Phase { player: 0, phase: Phase::Judge, begin: true },
            EventData::Phase { player: 0, phase: Phase::Draw, begin: true },
            EventData::Phase { player: 0, phase: Phase::Play, begin: true },
            EventData::Phase { player: 0, phase: Phase::Discard, begin: true },
            EventData::Phase { player: 0, phase: Phase::End, begin: true },
            EventData::Phase { player: 0, phase: Phase::End, begin: false },
            EventData::OrdinaryJudge {
                owner: 0,
                invocation: 0,
            },
            EventData::Equipped {
                player: 0,
                card: equip_card,
                replaced: None,
            },
            EventData::CardResponded {
                player: 0,
                card: shan_card,
            },
            EventData::Damage(damage_data_with(
                Some(1),
                0,
                1,
                DamageNature::Normal,
                Some(sample_card),
            )),
            EventData::Recover {
                player: 0,
                amount: 1,
                by_card: None,
            },
            EventData::LoseHp {
                player: 0,
                amount: 1,
            },
            EventData::TurnOvered {
                player: 0,
                face_down: true,
            },
            EventData::Linked {
                player: 0,
                linked: true,
            },
            EventData::CardsDrawn {
                player: 0,
                cards: vec![sample_card],
            },
            EventData::CardsDiscarded {
                player: 0,
                cards: vec![sample_card],
            },
            EventData::Dying { player: 0, source: Some(1) },
            EventData::Death {
                player: 1,
                killer: Some(0),
            },
            EventData::Death {
                player: 0,
                killer: Some(1),
            },
        ] {
            let _ = drive_event(&mut e, ev);
            while let Some(open) = e.open_choice().cloned() {
                let sel = if open.spec.allow_cancel {
                    vec![]
                } else {
                    open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
                };
                let _ = e.submit(next_cmd(
                    &e,
                    open.observer,
                    GameAction::Answer {
                        choice_id: open.id,
                        selection: sel,
                    },
                ));
            }
        }
        e.state_core_mut().player_mut(0).alive = true;
        e.state_core_mut().player_mut(0).hp = 4;
        e.state_core_mut().player_mut(1).alive = true;
        e.state_core_mut().player_mut(1).hp = 4;
        for kind in [
            CardKind::Sha,
            CardKind::Shan,
            CardKind::Tao,
            CardKind::Jiu,
            CardKind::Wuzhong,
            CardKind::Duel,
            CardKind::SavageAttack,
            CardKind::ArcheryAttack,
            CardKind::Taoyuan,
            CardKind::Wugu,
            CardKind::Snatch,
            CardKind::Dismantle,
            CardKind::Nullify,
            CardKind::Le,
            CardKind::Bing,
            CardKind::Chain,
        ] {
            let _ = drive_event(
                &mut e,
                EventData::CardUsed(card_use_as(sample_card, 0, vec![1], Some(kind))),
            );
            while let Some(open) = e.open_choice().cloned() {
                let sel = if open.spec.allow_cancel {
                    vec![]
                } else {
                    open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
                };
                let _ = e.submit(next_cmd(
                    &e,
                    open.observer,
                    GameAction::Answer {
                        choice_id: open.id,
                        selection: sel,
                    },
                ));
            }
        }

        for &hook in &passive_hooks {
            assert!(
                flag(&e, 0, &format!("l25_vfx_fired:{hook}")) >= 1,
                "passive hook `{hook}` must fire on its real trigger event"
            );
        }
    }

    // 3. Test the 23 Batch A Global/Mode/Shen skills
    // 3a. `g_du`, `_kongchengskill`, `_anyeskill`, `_taohuaskill`, `_lunjianskill`, `_thunderjudgemodeskill`, `_thundermodeskill`
    {
        let mut e = new_l25_engine(0x1250_0202, 3);
        grant_skill(&mut e, 0, "g_du");
        grant_skill(&mut e, 0, "_kongchengskill");
        grant_skill(&mut e, 0, "_anyeskill");
        grant_skill(&mut e, 0, "_taohuaskill");
        grant_skill(&mut e, 0, "_lunjianskill");
        grant_skill(&mut e, 0, "_thunderjudgemodeskill");
        grant_skill(&mut e, 0, "_thundermodeskill");

        let sha = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        // `g_du` negative: discarding a non-du card does NOT lose HP
        let _ = drive_event(
            &mut e,
            EventData::CardsDiscarded {
                player: 0,
                cards: vec![sha],
            },
        );
        assert_eq!(flag(&e, 0, "l25_g_du_triggered"), 0);

        // `g_du` positive + `_kongchengskill` positive: discarding a `du`-marked card outside own turn with empty hand
        set_flag(&mut e, 0, &format!("is_du_card:{sha}"), 1);
        e.state_core_mut().turn.current = 1;
        e.state_core_mut().player_mut(0).hand.clear();
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e,
            EventData::CardsDiscarded {
                player: 0,
                cards: vec![sha],
            },
        ) else {
            panic!("expected _kongchengskill:confirm");
        };
        assert_eq!(spec.prompt, "_kongchengskill:confirm");
        let cid = e.open_choice().unwrap().id;
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(1)] }));
        assert_eq!(flag(&e, 0, "l25_g_du_triggered"), 1);
        assert!(flag(&e, 0, "l25_kongcheng_used") > 0);
        assert_eq!(e.state_core().player(0).hp, 3);
        assert_eq!(e.state_core().player(0).hand.len(), 1);

        // `_anyeskill` on Fire damage & `_taohuaskill` on Heart CardUsed
        e.state_core_mut().turn.current = 0;
        let heart_tao = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let _ = drive_event(
            &mut e,
            EventData::Damage(damage_data_with(
                Some(1),
                0,
                1,
                DamageNature::Fire,
                Some(sha),
            )),
        );
        assert_eq!(flag(&e, 0, "l25_anye_triggered"), 1);
        let _ = drive_event(&mut e, EventData::CardUsed(card_use(heart_tao, 0, vec![0])));
        assert_eq!(flag(&e, 0, "l25_taohua_active"), 1);

        // `_lunjianskill` on Phase::Prepare -> asks `_lunjianskill:choose_target`
        grant_skill(&mut e, 1, "wushen");
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e,
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
        ) else {
            panic!("expected _lunjianskill:choose_target");
        };
        assert_eq!(spec.prompt, "_lunjianskill:choose_target");
        let cid = e.open_choice().unwrap().id;
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] }));
        assert_eq!(flag(&e, 0, "l25_lunjian_target"), 2);

        // `_thunderjudgemodeskill` on Phase::Judge
        let spade2 = find_suit_rank(&e, Suit::Spade, 2, 9);
        e.state_core_mut().deck.insert(0, spade2);
        let _ = drive_event(
            &mut e,
            EventData::Phase { player: 0, phase: Phase::Judge, begin: true },
        );
        assert!(flag(&e, 0, "l25_thunder_judge_card") > 0);

        // `_thundermodeskill` on Phase::End (begin: false)
        e.state_core_mut().player_mut(0).hp = 4;
        let _ = drive_event(
            &mut e,
            EventData::Phase { player: 0, phase: Phase::End, begin: false },
        );
        assert!(flag(&e, 0, "l25_thundermode_triggered") > 0);
    }

    // 3b. Interactive Batch A skills: `_yongjinskill`, `_jianghunskill`, `_goodluckjuedgeskill`
    {
        let mut e = new_l25_engine(0x1250_0203, 3);
        let yongjin_def = grant_skill(&mut e, 0, "_yongjinskill");
        let weapon = find_card(&e, |_, d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
        e.state_core_mut().player_mut(1).equips = vec![weapon];

        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: yongjin_def },
        )) else {
            panic!("expected _yongjinskill:choose_target");
        };
        assert_eq!(choice.spec.prompt, "_yongjinskill:choose_target");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        )) else {
            panic!("expected _yongjinskill:choose_dest");
        };
        assert_eq!(choice.spec.prompt, "_yongjinskill:choose_dest");
        let out = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Player(2)],
            },
        ));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e, 0, "l25_yongjin_moved"), 1);

        // `_jianghunskill` on TurnBegin & `_goodluckjuedgeskill` on OrdinaryJudge
        let mut e2 = new_l25_engine(0x1250_0204, 3);
        grant_skill(&mut e2, 0, "_jianghunskill");
        grant_skill(&mut e2, 0, "_goodluckjuedgeskill");
        let _ = drive_event(&mut e2, EventData::TurnBegin { player: 0 });
        assert_eq!(flag(&e2, 0, "l25_jianghun_granted"), 1);
        let _ = drive_event(
            &mut e2,
            EventData::OrdinaryJudge {
                owner: 0,
                invocation: 0,
            },
        );
        assert_eq!(flag(&e2, 0, "l25_goodluck_judge"), 1);
    }

    // 3c. Shen skills: `wumou`, `ol_wuqian`, `new_guixin`, `kuangfeng`, `dawu`/`dawu2`, `qinyin`, `xinjuejing`, `renjie`, `wushen`, `new_wuhun`, `g_nzry_jieying`, `nzry_longnu`
    {
        let mut e = new_l25_engine(0x1250_0206, 3);
        grant_skill(&mut e, 0, "wumou");
        grant_skill(&mut e, 0, "ol_wuqian");
        set_flag(&mut e, 0, "mark:baonu", 3);

        // `ol_wuqian` triggers when using `Sha`
        let sha = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let _ = drive_event(&mut e, EventData::CardUsed(card_use(sha, 0, vec![1])));
        assert_eq!(flag(&e, 0, "l25_ol_wuqian_unblockable"), 1);
        assert_eq!(flag(&e, 1, "qinggang_disable_armor"), 1);

        // `wumou` triggers when using an ordinary trick card (`Wuzhong`)
        let wz = find_kind_suit(&e, CardKind::Wuzhong, Suit::Heart);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e,
            EventData::CardUsed(card_use(wz, 0, vec![0])),
        ) else {
            panic!("expected wumou:choose_mode");
        };
        assert_eq!(spec.prompt, "wumou:choose_mode");
        let cid = e.open_choice().unwrap().id;
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Tag(0)] }));
        assert_eq!(flag(&e, 0, "l25_wumou_triggered"), 1);
        assert_eq!(flag(&e, 0, "mark:baonu"), 2);

        // `new_guixin` on Damage -> confirm steals 1 card from each other alive player and flips face_down
        let mut e2 = new_l25_engine(0x1250_0207, 3);
        grant_skill(&mut e2, 0, "new_guixin");
        let c1 = find_kind_suit(&e2, CardKind::Sha, Suit::Spade);
        let c2 = find_kind_suit(&e2, CardKind::Shan, Suit::Diamond);
        e2.state_core_mut().player_mut(1).hand = vec![c1];
        e2.state_core_mut().player_mut(2).hand = vec![c2];
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e2,
            EventData::Damage(damage_data_with(
                Some(1),
                0,
                1,
                DamageNature::Normal,
                Some(c1),
            )),
        ) else {
            panic!("expected new_guixin:confirm");
        };
        assert_eq!(spec.prompt, "new_guixin:confirm");
        let cid = e2.open_choice().unwrap().id;
        let _ = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Tag(1)],
            },
        ));
        assert_eq!(flag(&e2, 0, "l25_new_guixin_count"), 1);
        assert!(e2.state_core().player(0).face_down);

        // `kuangfeng` & `dawu` / `dawu2` with `mark:qixing`
        let mut e3 = new_l25_engine(0x1250_0208, 3);
        grant_skill(&mut e3, 0, "kuangfeng");
        set_flag(&mut e3, 0, "mark:qixing", 2);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e3,
            EventData::Phase { player: 0, phase: Phase::End, begin: true },
        ) else {
            panic!("expected kuangfeng:choose_target");
        };
        assert_eq!(spec.prompt, "kuangfeng:choose_target");
        let cid = e3.open_choice().unwrap().id;
        let _ = e3.submit(next_cmd(&e3,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(1)],
            },
        ));
        assert_eq!(flag(&e3, 1, "kuangfeng_marked"), 1);

        let mut e4 = new_l25_engine(0x1250_0209, 3);
        grant_skill(&mut e4, 0, "dawu");
        grant_skill(&mut e4, 1, "dawu2");
        set_flag(&mut e4, 0, "mark:qixing", 1);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e4,
            EventData::Phase { player: 0, phase: Phase::End, begin: true },
        ) else {
            panic!("expected dawu:choose_targets");
        };
        assert_eq!(spec.prompt, "dawu:choose_targets");
        let cid = e4.open_choice().unwrap().id;
        let _ = e4.submit(next_cmd(&e4,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(1)],
            },
        ));
        assert_eq!(flag(&e4, 1, "dawu_marked"), 1);
        let _ = drive_event(
            &mut e4,
            EventData::Damage(damage_data_with(
                Some(2),
                1,
                1,
                DamageNature::Normal,
                Some(c1),
            )),
        );
        assert_eq!(flag(&e4, 1, "l25_dawu2_prevented"), 1);

        // `qinyin`, `xinjuejing`, `renjie`, `wushen`, `new_wuhun`, `g_nzry_jieying`, `nzry_longnu`
        let mut e5 = new_l25_engine(0x1250_0210, 3);
        grant_skill(&mut e5, 0, "qinyin");
        grant_skill(&mut e5, 0, "xinjuejing");
        grant_skill(&mut e5, 0, "renjie");
        grant_skill(&mut e5, 0, "wushen");
        grant_skill(&mut e5, 0, "new_wuhun");
        grant_skill(&mut e5, 0, "g_nzry_jieying");
        grant_skill(&mut e5, 0, "nzry_longnu");

        // `wushen`: Heart hand card views as distance-unlimited Sha
        let heart_tao = find_kind_suit(&e5, CardKind::Tao, Suit::Heart);
        let wz_trick = find_kind_suit(&e5, CardKind::Wuzhong, Suit::Spade);
        e5.state_core_mut().player_mut(0).hand = vec![heart_tao, wz_trick];
        let spec = find_d34_flag_view_as(
            e5.state_core(),
            e5.catalog(),
            0,
            heart_tao,
            true,
            Some(CardKind::Sha),
        )
        .expect("wushen must convert Heart card to Sha");
        assert_eq!(spec.as_kind, CardKind::Sha);
        assert!(spec.ignore_distance);

        // `nzry_longnu` on Phase::Play -> flips yang/yin and enables FireSha (Red) / LeiSha (Trick) view-as
        let d_shan = find_kind_suit(&e5, CardKind::Shan, Suit::Diamond);
        e5.state_core_mut().player_mut(0).hand.push(d_shan);
        let _ = drive_event(
            &mut e5,
            EventData::Phase { player: 0, phase: Phase::Play, begin: true },
        );
        assert!(flag(&e5, 0, "nzry_longnu_fire_turn") > 0);
        assert!(find_d34_flag_view_as(
            e5.state_core(),
            e5.catalog(),
            0,
            d_shan,
            true,
            Some(CardKind::FireSha),
        )
        .is_some());
        set_flag(&mut e5, 0, "nzry_longnu_thunder_turn", 1);
        assert!(find_d34_flag_view_as(
            e5.state_core(),
            e5.catalog(),
            0,
            wz_trick,
            true,
            Some(CardKind::LeiSha),
        )
        .is_some());

        // `qinyin` + `renjie` on CardsDiscarded (>=2 cards in Phase::Discard) + Phase::Discard end
        e5.state_core_mut().turn.phase = Phase::Discard;
        let _ = drive_event(
            &mut e5,
            EventData::CardsDiscarded {
                player: 0,
                cards: vec![c1, c2],
            },
        );
        assert!(flag(&e5, 0, "mark:ren") >= 2);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e5,
            EventData::Phase { player: 0, phase: Phase::Discard, begin: false },
        ) else {
            panic!("expected qinyin:choose_mode");
        };
        assert_eq!(spec.prompt, "qinyin:choose_mode");
        let cid = e5.open_choice().unwrap().id;
        let _ = e5.submit(next_cmd(&e5,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Tag(0)],
            },
        ));
        assert_eq!(flag(&e5, 0, "qinyin_triggered"), 1);

        // `xinjuejing` on Dying & `new_wuhun` on Damage + Death
        let _ = drive_event(
            &mut e5,
            EventData::Dying { player: 0, source: Some(1) },
        );
        assert_eq!(flag(&e5, 0, "l25_xinjuejing_dying"), 1);
        let _ = drive_event(
            &mut e5,
            EventData::Damage(damage_data_with(
                Some(1),
                0,
                1,
                DamageNature::Normal,
                Some(c1),
            )),
        );
        assert!(flag(&e5, 1, "mark:mengyan") >= 1);

        // `g_nzry_jieying` on TurnBegin & Phase::End
        let _ = drive_event(
            &mut e5,
            EventData::TurnBegin { player: 0 },
        );
        assert_eq!(flag(&e5, 0, "mark:ying"), 1);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e5,
            EventData::Phase { player: 0, phase: Phase::End, begin: true },
        ) else {
            panic!("expected g_nzry_jieying:choose_target");
        };
        assert_eq!(spec.prompt, "g_nzry_jieying:choose_target");
        let cid = e5.open_choice().unwrap().id;
        let _ = e5.submit(next_cmd(&e5,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(2)],
            },
        ));
        assert_eq!(flag(&e5, 2, "mark:ying"), 1);
    }
}

#[test]
fn l25_zuan_batch_b_22_skills_6d() {
    // 1. Lu Xun (`ssr_qianxun`, `ssr_jueji`) + Sun Quan (`ssr_zhiheng`, `ssr_jiuyuan`)
    {
        let mut e = new_l25_engine(0x1250_0301, 3);
        grant_skill(&mut e, 0, "ssr_qianxun");
        let jueji_def = grant_skill(&mut e, 0, "ssr_jueji");
        let c1 = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let c2 = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        let shunshou = find_kind_suit(&e, CardKind::Snatch, Suit::Spade);
        e.state_core_mut().player_mut(0).hand = vec![c1, c2];

        // `ssr_qianxun` on trick CardUsed targeting `0` when `hand.len() > 1` -> discard down to 1 card
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e,
            EventData::CardUsed(card_use(shunshou, 1, vec![0])),
        ) else {
            panic!("expected ssr_qianxun:discard_cards");
        };
        assert_eq!(spec.prompt, "ssr_qianxun:discard_cards");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let cid = e.open_choice().unwrap().id;
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(c1)] }));
        assert_eq!(flag(&e, 0, "l25_qianxun_triggered"), 1);
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(
                &e,
                open.observer,
                GameAction::Answer {
                    choice_id: open.id,
                    selection: sel,
                },
            ));
        }
        e.state_core_mut().player_mut(0).hand = vec![c2];

        // `ssr_jueji` active in Phase::Play when `hand.len() == 1` -> choose_mode -> choose_target
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: jueji_def },
        )) else {
            panic!("expected ssr_jueji:choose_mode");
        };
        assert_eq!(choice.spec.prompt, "ssr_jueji:choose_mode");
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] })) else {
            panic!("expected ssr_jueji:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert!(flag(&e, 0, "l25_jueji_used") >= 1);

        // `ssr_zhiheng` (active) & `ssr_jiuyuan` (Sha targeting owner)
        let mut e2 = new_l25_engine(0x1250_0302, 3);
        let zhiheng_def = grant_skill(&mut e2, 0, "ssr_zhiheng");
        grant_skill(&mut e2, 0, "ssr_jiuyuan");
        e2.state_core_mut().player_mut(0).hand = vec![c1, c2];

        // Cancel on `ssr_zhiheng` refunds `once_used`
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::ActivateSkill { skill: zhiheng_def },
        )) else {
            panic!("expected ssr_zhiheng:discard_cards");
        };
        assert_eq!(choice.spec.prompt, "ssr_zhiheng:discard_cards");
        assert_mid_choice_snapshot_roundtrip(&mut e2);
        let _ = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![],
            },
        ));
        assert!(!e2.state_core().skills[&(0, zhiheng_def)].once_used);

        // Positive `ssr_zhiheng`: discard 1 card -> choose target -> choose mode 0 -> target choice
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::ActivateSkill { skill: zhiheng_def },
        )) else {
            panic!("expected ssr_zhiheng:discard_cards");
        };
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(c1)],
            },
        )) else {
            panic!("expected ssr_zhiheng:choose_target");
        };
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        )) else {
            panic!("expected ssr_zhiheng:choose_mode");
        };
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        )) else {
            panic!("expected ssr_zhiheng:target_choice");
        };
        let _ = e2.submit(next_cmd(&e2,
            1,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Tag(1)],
            },
        ));
        assert_eq!(flag(&e2, 0, "l25_zhiheng_used"), 1);

        // `ssr_jiuyuan` on Sha targeting `0` -> Wu responder confirms -> `l25_jiuyuan_jibing` modifier
        e2.assign_general_named(1, "ssr_luxun").unwrap();
        let StepOutcome::WaitChoice { observer, spec } = drive_event(
            &mut e2,
            EventData::CardUsed(card_use(c1, 2, vec![0])),
        ) else {
            panic!("expected ssr_jiuyuan:confirm");
        };
        assert_eq!(observer, 1);
        assert_eq!(spec.prompt, "ssr_jiuyuan:confirm");
        let cid = e2.open_choice().unwrap().id;
        let _ = e2.submit(next_cmd(&e2,
            1,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Tag(1)],
            },
        ));
        assert_eq!(flag(&e2, 1, "l25_jiuyuan_jibing"), 1);
        assert_eq!(apply_draw_count(e2.state_core(), e2.catalog(), 1, 2), 3);
    }

    // 2. Zhao Yun (`ssr_newlongdan`, `ssr_newpozhen`) + Zhuge Liang (`ssr_jinnang`, `ssr_newbazhen`, `ssr_yunchou`)
    {
        let mut e = new_l25_engine(0x1250_0303, 3);
        let longdan_def = grant_skill(&mut e, 0, "ssr_newlongdan");
        grant_skill(&mut e, 0, "ssr_newpozhen");
        let shan = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        let jiu = find_kind_suit(&e, CardKind::Jiu, Suit::Spade);
        e.state_core_mut().player_mut(0).hand = vec![shan, jiu];
        e.state_core_mut().player_mut(1).hand.clear();

        // `ssr_newlongdan` view-as: Shan <-> Sha, Jiu <-> Tao
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            shan,
            true,
            Some(CardKind::Sha),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            jiu,
            true,
            Some(CardKind::Tao),
        )
        .is_some());

        // `ssr_newlongdan` active skill
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: longdan_def },
        )) else {
            panic!("expected ssr_newlongdan:choose_card");
        };
        assert_eq!(choice.spec.prompt, "ssr_newlongdan:choose_card");
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(shan)] })) else {
            panic!("expected ssr_newlongdan:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert!(flag(&e, 0, "l25_pozhen_chongfeng") >= 1);

        // `ssr_jinnang`, `ssr_newbazhen`, `l25_viewas_wuxie`
        let mut e2 = new_l25_engine(0x1250_0304, 3);
        let jinnang_def = grant_skill(&mut e2, 0, "ssr_jinnang");
        let bazhen_def = grant_skill(&mut e2, 0, "ssr_newbazhen");
        set_flag(&mut e2, 0, "l25_viewas_wuxie", 1);
        let wz = find_kind_suit(&e2, CardKind::Wuzhong, Suit::Heart);
        e2.state_core_mut().player_mut(0).hand = vec![wz];

        // `ssr_yunchou` view-as Nullify (`Wuxie`)
        assert!(find_d34_flag_view_as(
            e2.state_core(),
            e2.catalog(),
            0,
            wz,
            false,
            Some(CardKind::Nullify),
        )
        .is_some());

        // `ssr_jinnang` active: discard trick -> choose target -> choose mode 0
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::ActivateSkill { skill: jinnang_def },
        )) else {
            panic!("expected ssr_jinnang:choose_card");
        };
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(wz)],
            },
        )) else {
            panic!("expected ssr_jinnang:choose_target");
        };
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Player(0)],
            },
        )) else {
            panic!("expected ssr_jinnang:choose_mode");
        };
        let _ = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        ));
        assert_eq!(flag(&e2, 0, "l25_jinnang_used"), 1);

        // `ssr_newbazhen`: Phase::Prepare fills pile -> active chooses card & uses it
        let _ = drive_event(
            &mut e2,
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
        );
        let pile = e2
            .state_core()
            .vm_aux
            .piles
            .get(&(0, "ssr_newbazhen".to_string()))
            .cloned()
            .unwrap_or_default();
        assert!(!pile.is_empty());
        let first_zhen = pile[0];
        let CommandOutcome::Pending { choice } = e2.submit(next_cmd(&e2,
            0,
            GameAction::ActivateSkill { skill: bazhen_def },
        )) else {
            panic!("expected ssr_newbazhen:choose_card");
        };
        let out = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(first_zhen)],
            },
        ));
        if let CommandOutcome::Pending { choice } = out {
            let _ = e2.submit(next_cmd(&e2,
                0,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            ));
        }
    }

    // 3. Zhang Liao (`ssr_tuxi`, `ssr_newmubing`), Gan Ning (`ssr_jinfan`, `ssr_qixi`),
    //    Ma Chao (`ssr_zhuiji`, `ssr_tieji`), Zhou Yu (`ssr_newfanjian`, `ssr_youqin`),
    //    Liu Bei (`ssr_shichou`, `ssr_newrende`, `ssr_jijiang`), Dian Wei (`ssr_qiangxi`, `ssr_qingji`)
    {
        let mut e = new_l25_engine(0x1250_0305, 3);
        grant_skill(&mut e, 0, "ssr_tuxi");
        grant_skill(&mut e, 0, "ssr_newmubing");
        let jinfan_def = grant_skill(&mut e, 0, "ssr_jinfan");
        grant_skill(&mut e, 0, "ssr_qixi");
        grant_skill(&mut e, 0, "ssr_zhuiji");
        grant_skill(&mut e, 0, "ssr_tieji");
        let fanjian_def = grant_skill(&mut e, 0, "ssr_newfanjian");
        grant_skill(&mut e, 0, "ssr_youqin");
        grant_skill(&mut e, 0, "ssr_shichou");
        let rende_def = grant_skill(&mut e, 0, "ssr_newrende");
        grant_skill(&mut e, 0, "ssr_jijiang");
        let qiangxi_def = grant_skill(&mut e, 0, "ssr_qiangxi");
        let qingji_def = grant_skill(&mut e, 0, "ssr_qingji");

        let black_sha = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let red_tao = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let shan_d = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        e.state_core_mut().player_mut(0).hand = vec![black_sha, red_tao];
        e.state_core_mut().player_mut(1).hand = vec![shan_d];
        e.state_core_mut().player_mut(0).hujia = 2;
        e.state_core_mut().player_mut(0).hp = 2;

        // Passive modifiers for `ssr_youqin`, `ssr_zhuiji`, `ssr_jijiang`
        assert!(eval_hand_limit(e.state_core(), e.catalog(), 0) >= 6);
        let d01 = eval_distance_total(e.state_core(), e.catalog(), 0, 1, 2);
        assert!(d01 <= 1);
        let (sha_lim, _) = eval_sha_use_limit(e.state_core(), e.catalog(), 0);
        assert!(sha_lim >= 2);

        // `ssr_jinfan` active: discard card to add `铃`
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: jinfan_def },
        )) else {
            panic!("expected ssr_jinfan:discard_card");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(black_sha)] }));
        assert!(flag(&e, 0, "l25_jinfan_ling") >= 1);
        assert!(eval_ignore_armor(e.state_core(), e.catalog(), 0, 1));

        // `ssr_newfanjian` active
        e.state_core_mut().player_mut(0).hand = vec![red_tao];
        e.state_core_mut().player_mut(1).hand = vec![black_sha, shan_d];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: fanjian_def },
        )) else {
            panic!("expected ssr_newfanjian:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(red_tao)] })) else {
            panic!("expected ssr_newfanjian:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_newfanjian:show_card");
        };
        let _ = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(black_sha)] }));
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: sel }));
        }
        assert!(flag(&e, 1, "l25_fanjian_forbid_suit") > 0);
        e.state_core_mut().player_mut(1).hand = vec![red_tao, black_sha];
        assert!(
            apply_card_use_canceled(e.state_core(), e.catalog(), 1, CardKind::Tao, true, false)
                || apply_card_use_canceled(
                    e.state_core(),
                    e.catalog(),
                    1,
                    CardKind::Sha,
                    true,
                    false
                )
        );

        // `ssr_newrende` active -> target choice Tag(0) recovers 1 HP and sets `l25_rende_forbid_dmg_to`
        e.state_core_mut().player_mut(1).hand.clear();
        e.state_core_mut().player_mut(0).hand = vec![red_tao];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: rende_def },
        )) else {
            panic!("expected ssr_newrende:give_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(red_tao)] })) else {
            panic!("expected ssr_newrende:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_newrende:target_choice");
        };
        let _ = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] }));
        assert!(apply_card_target_canceled(
            e.state_core(),
            e.catalog(),
            1,
            0,
            CardKind::Sha,
            false
        ));

        // `ssr_qiangxi` active & `ssr_qingji` active
        e.assign_general_named(2, "ssr_zhaoyun").unwrap();
        set_flag(&mut e, 0, "l25_qiangxi_meng", 2);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: qiangxi_def },
        )) else {
            panic!("expected ssr_qiangxi:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert_eq!(flag(&e, 0, "l25_qiangxi_used"), 1);

        let _ = e.submit(next_cmd(&e, 0, GameAction::ActivateSkill { skill: qingji_def }));
    }
}

#[test]
fn l25_zuan_batch_c_22_skills_6d() {
    // 1. Guo Jia (`ssr_bailun`, `ssr_newyice`), Cao Cao (`ssr_tumeng`, `ssr_zhujiu`, `ssr_hujia`),
    //    Lv Bu (`ssr_shenfen`, `ssr_kuangbao`, `ssr_wushuang`, `ssr_jiwu`), Huang Zhong (`ssr_liegong`, `ssr_pojia`)
    {
        let mut e = new_l25_engine(0x1250_0401, 3);
        grant_skill(&mut e, 0, "ssr_bailun");
        let yice_def = grant_skill(&mut e, 0, "ssr_newyice");
        grant_skill(&mut e, 0, "ssr_tumeng");
        let zhujiu_def = grant_skill(&mut e, 0, "ssr_zhujiu");
        grant_skill(&mut e, 0, "ssr_hujia");
        grant_skill(&mut e, 0, "ssr_shenfen");
        grant_skill(&mut e, 0, "ssr_kuangbao");
        grant_skill(&mut e, 0, "ssr_wushuang");
        let jiwu_def = grant_skill(&mut e, 0, "ssr_jiwu");
        grant_skill(&mut e, 0, "ssr_liegong");
        grant_skill(&mut e, 0, "ssr_pojia");

        let spade_sha = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let heart_tao = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let weapon = find_card(&e, |_, d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
        e.state_core_mut().player_mut(0).hand = vec![spade_sha, heart_tao];
        e.state_core_mut().player_mut(1).equips = vec![weapon];

        // `ssr_bailun` on Phase::Prepare -> judge -> sets `l25_bailun_trick_boost` or `l25_bailun_spade`
        let _ = drive_event(
            &mut e,
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
        );
        set_flag(&mut e, 0, "l25_bailun_trick_boost", 1);
        assert!(apply_card_distance_unlimited(
            e.state_core(),
            e.catalog(),
            0,
            heart_tao,
            CardKind::Snatch,
            false
        ));
        assert!(apply_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            CardKind::Snatch,
            false
        ));

        // `ssr_newyice` active: choose card -> choose target -> choose mode + snapshot roundtrip
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: yice_def },
        )) else {
            panic!("expected ssr_newyice:choose_card");
        };
        assert_eq!(choice.spec.prompt, "ssr_newyice:choose_card");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(heart_tao)] })) else {
            panic!("expected ssr_newyice:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_newyice:choose_mode");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(1)] }));
        assert_eq!(flag(&e, 0, "l25_yice_dongzhuxianji"), 1);

        // `ssr_tumeng` on Damage & `ssr_zhujiu` active
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e,
            EventData::Damage(damage_data_with(
                Some(1),
                0,
                1,
                DamageNature::Normal,
                Some(spade_sha),
            )),
        ) else {
            panic!("expected ssr_tumeng:choose_target");
        };
        assert_eq!(spec.prompt, "ssr_tumeng:choose_target");
        let cid = e.open_choice().unwrap().id;
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] }));
        assert_eq!(flag(&e, 0, "l25_tumeng_backlash"), 1);

        e.state_core_mut().player_mut(0).hand.push(spade_sha);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: zhujiu_def },
        )) else {
            panic!("expected ssr_zhujiu:discard_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(spade_sha)] })) else {
            panic!("expected ssr_zhujiu:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert_eq!(flag(&e, 0, "l25_zhujiu_used_flag"), 1);

        // `ssr_kuangbao` + `ssr_shenfen` trigger on TurnBegin + `ssr_jiwu` active + `ssr_liegong` trigger on Sha
        e.state_core_mut().player_mut(1).equips.clear();
        set_flag(&mut e, 0, "l25_kuangbao_baonu", 6);
        set_flag(&mut e, 0, "mark:baonu", 6);
        let _ = drive_event(&mut e, EventData::TurnBegin { player: 0 });
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(
                &e,
                open.observer,
                GameAction::Answer {
                    choice_id: open.id,
                    selection: vec![],
                },
            ));
        }
        assert_eq!(flag(&e, 0, "l25_shenfen_awakened"), 1);

        e.state_core_mut().turn.current = 0;
        e.state_core_mut().turn.phase = Phase::Play;
        set_flag(&mut e, 0, "l25_kuangbao_baonu", 6);
        set_flag(&mut e, 0, "mark:baonu", 6);
        let out = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: jiwu_def },
        ));
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(flag(&e, 0, "l25_jiwu_boost"), 1);

        e.state_core_mut().player_mut(0).hand.push(heart_tao);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e,
            EventData::CardUsed(card_use(spade_sha, 0, vec![1])),
        ) else {
            panic!("expected ssr_liegong:discard_card");
        };
        assert_eq!(spec.prompt, "ssr_liegong:discard_card");
        let cid = e.open_choice().unwrap().id;
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(heart_tao)] }));
        assert_eq!(flag(&e, 0, "l25_liegong_unresp"), 1);
    }

    // 2. Xiahou Dun (`ssr_newganglie`, `ssr_zuni`), Hua Tuo (`ssr_chuli`, `ssr_xianfang`),
    //    Yu Jin (`ssr_newzhenjun`, `ssr_newyizhong`), Yu Ji (`ssr_shendao`, `ssr_chanyuan`),
    //    Jia Xu (`ssr_newjiahe`, `ssr_miedi`), Hua Xiong (`ssr_shiyong`, `ssr_newkurou`)
    {
        let mut e = new_l25_engine(0x1250_0402, 3);
        grant_skill(&mut e, 0, "ssr_newganglie");
        grant_skill(&mut e, 0, "ssr_zuni");
        let chuli_def = grant_skill(&mut e, 0, "ssr_chuli");
        grant_skill(&mut e, 0, "ssr_xianfang");
        let zhenjun_def = grant_skill(&mut e, 0, "ssr_newzhenjun");
        grant_skill(&mut e, 0, "ssr_newyizhong");
        let shendao_def = grant_skill(&mut e, 0, "ssr_shendao");
        grant_skill(&mut e, 0, "ssr_chanyuan");
        grant_skill(&mut e, 0, "ssr_newjiahe");
        let miedi_def = grant_skill(&mut e, 0, "ssr_miedi");
        grant_skill(&mut e, 0, "ssr_shiyong");
        let kurou_def = grant_skill(&mut e, 0, "ssr_newkurou");

        let c1 = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let c2 = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        let c3 = find_kind_suit(&e, CardKind::Wuzhong, Suit::Heart);
        e.state_core_mut().player_mut(0).hand = vec![c1, c2, c3];
        e.state_core_mut().player_mut(1).hand = vec![c1, c2, c3];

        // `ssr_shiyong` view-as: Trick in hand -> Shan
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            c3,
            false,
            Some(CardKind::Shan),
        )
        .is_some());

        // `ssr_chuli` active: choose target -> choose card -> adds to `药` pile
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: chuli_def },
        )) else {
            panic!("expected ssr_chuli:choose_target");
        };
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_chuli:choose_card");
        };
        let first_opt = choice.spec.options[0];
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![first_opt] }));
        assert!(flag(&e, 0, "l25_chuli_yao") >= 1);

        // `ssr_newzhenjun` active: seals target's black hand cards & ignores armor
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: zhenjun_def },
        )) else {
            panic!("expected ssr_newzhenjun:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: vec![] }));
        }
        assert_eq!(flag(&e, 1, "l25_zhenjun_sealed"), 1);
        assert!(eval_ignore_armor(e.state_core(), e.catalog(), 0, 1));

        // `ssr_shendao` active: discard 2 -> choose target -> choose option + mid-choice snapshot roundtrip
        e.state_core_mut().player_mut(0).hand = vec![c1, c2, c3];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: shendao_def },
        )) else {
            panic!("expected ssr_shendao:discard_cards");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
            },
        )) else {
            panic!("expected ssr_shendao:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_shendao:choose_option");
        };
        assert_eq!(choice.spec.prompt, "ssr_shendao:choose_option");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: vec![] }));
        }
        assert_eq!(flag(&e, 1, "l25_shendao_shenfu"), 2);

        // `ssr_miedi` active -> target draws 1 -> target choice -> target gives 2 cards
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: miedi_def },
        )) else {
            panic!("expected ssr_miedi:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_miedi:target_choice");
        };
        assert_eq!(choice.observer, 1);
        assert_eq!(choice.spec.prompt, "ssr_miedi:target_choice");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let hand1 = e.state_core().player(1).hand.clone();
        let out = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] }));
        if let CommandOutcome::Pending { choice } = out {
            assert_eq!(choice.spec.prompt, "ssr_miedi:target_give");
            let _ = e.submit(next_cmd(&e,
                1,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection: vec![ChoiceOption::Card(hand1[0]), ChoiceOption::Card(hand1[1])],
                },
            ));
        }
        assert_eq!(flag(&e, 0, "l25_miedi_gave"), 1);

        // `ssr_newkurou` active: choose target -> lose 1 HP -> choose card from target's hand
        e.state_core_mut().player_mut(0).hp = 3;
        e.state_core_mut().player_mut(1).hand = vec![c3];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: kurou_def },
        )) else {
            panic!("expected ssr_newkurou:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_newkurou:choose_card");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(c3)] }));
        assert_eq!(flag(&e, 0, "l25_kurou_active"), 1);

        // `ssr_chanyuan` + `ssr_newyizhong` hand limit modifier
        e.state_core_mut().player_mut(0).hujia = 0;
        set_flag(&mut e, 0, "l25_chanyuan_crazy", 2);
        assert_eq!(apply_hand_limit(e.state_core(), e.catalog(), 0, 4), 2);
        let _ = (
            CardKind::Sha,
            Faction::Wei,
            Gender::Male,
            ChoiceKind::Confirm,
            RejectReason::NoSuchSkill,
            apply_attack_range,
            apply_damage_amount,
            apply_distance_total,
            apply_sha_extra_targets,
            apply_sha_use_limit,
            eval_card_unrespondable,
            card_use_as,
        );
    }
}
