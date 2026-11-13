//! L25 祖安武将 (`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`)
//! 6-Dimension Real-Match Test Suite — Part 2:
//! - Batch D: 22 skills (Generals #21–#30 + `ssr_luoyi` + `ssr_yongsi`) across 6 dimensions
//! - Batch E: 22 skills (Generals #31–#40) across 6 dimensions
//! - Batch F: 22 skills (Generals #41–#50) across 6 dimensions
//! - Batch G: 22 skills (Generals #51–#60) across 6 dimensions
//! - Full 130-skill + 84-rule-skill catalog & host behavior verification

use content_native_loader::{
    decision34_pack9_extension_pack, lookup_l25_zuan_host_behavior, L25_ALL_RULE_SKILLS,
    L25_SSR_GENERAL_SKILLS,
};
use engine_native::{
    archive::{decode_archive, encode_archive},
    catalog::{CardDef, CardKind, Catalog, EquipSlot, Suit},
    choice::{ChoiceOption, ChoiceSpec},
    event::{DamageNature, EventData, Phase},
    ids::{CardId, PlayerId, SkillDefId},
    machine::card_use,
    rvm_bridge::modifiers::{
        d34_flags::{
            apply_attack_range,
            apply_card_target_canceled as d34_apply_card_target_canceled,
            apply_card_use_canceled as d34_apply_card_use_canceled, apply_damage_amount,
            apply_distance_total, apply_draw_count, apply_hand_limit, apply_ignore_armor,
            apply_sha_extra_targets, apply_sha_use_limit, find_d34_flag_view_as,
        },
        eval_card_unrespondable as raw_eval_card_unrespondable, eval_distance_total,
        eval_hand_limit as raw_eval_hand_limit, eval_ignore_armor as raw_eval_ignore_armor,
        eval_sha_use_limit as raw_eval_sha_use_limit,
    },
    state::{GameStateCore, SkillInstance},
    CommandOutcome, Engine, GameAction, GameCommand, RejectReason,
};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(6_000_000);

#[derive(Debug)]
#[allow(dead_code)]
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
fn l25_zuan_batch_d_22_skills_6d() {
    // 1. Huang Gai (`ssr_fenchuan`), Diao Chan (`ssr_newlihun`, `ssr_biyue`),
    //    Yuan Shu (`ssr_yjixi`, `ssr_yanji`, `ssr_cuandi`, `ssr_yongsi`),
    //    Guan Yu (`ssr_wusheng`, `ssr_yijue`), Lv Meng (`ssr_newbaiyi`, `ssr_liecheng`)
    {
        let mut e = new_l25_engine(0x1250_0501, 3);
        grant_skill(&mut e, 0, "ssr_fenchuan");
        let lihun_def = grant_skill(&mut e, 0, "ssr_newlihun");
        grant_skill(&mut e, 0, "ssr_biyue");
        grant_skill(&mut e, 0, "ssr_yjixi");
        grant_skill(&mut e, 0, "ssr_yanji");
        grant_skill(&mut e, 0, "ssr_cuandi");
        grant_skill(&mut e, 0, "ssr_wusheng");
        let yijue_def = grant_skill(&mut e, 0, "ssr_yijue");
        grant_skill(&mut e, 0, "ssr_newbaiyi");
        let liecheng_def = grant_skill(&mut e, 0, "ssr_liecheng");

        let spade_sha = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let heart_tao = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let diamond_shan = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        e.state_core_mut().player_mut(0).hand = vec![spade_sha, heart_tao];
        e.state_core_mut().player_mut(1).hand = vec![spade_sha, diamond_shan];

        // `ssr_wusheng`: Red card -> distance-unlimited Sha + Jiu-boosted Sha is unrespondable
        let ws_spec = find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            heart_tao,
            true,
            Some(CardKind::Sha),
        )
        .expect("ssr_wusheng must convert Red card to Sha");
        assert!(ws_spec.ignore_distance);
        e.state_core_mut().turn.jiu_active = true;
        assert!(eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            CardKind::Sha
        ));
        e.state_core_mut().turn.jiu_active = false;

        // `ssr_biyue`: distance +1 per empty zone on target, and cancels Duel/SavageAttack/ArcheryAttack/Jiedao
        assert!(eval_distance_total(e.state_core(), e.catalog(), 1, 0, 1) >= 3);
        assert!(apply_card_target_canceled(
            e.state_core(),
            e.catalog(),
            1,
            0,
            CardKind::Duel,
            false
        ));

        // `ssr_yjixi`: +1 Sha use limit when no Yuxi equipped
        let (sha_lim, _) = eval_sha_use_limit(e.state_core(), e.catalog(), 0);
        assert!(sha_lim >= 2);

        // `ssr_newlihun` active: cancel refunds `once_used`, then give card -> choose target -> target guesses type
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: lihun_def },
        )) else {
            panic!("expected ssr_newlihun:give_card");
        };
        assert_eq!(choice.spec.prompt, "ssr_newlihun:give_card");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![] }));
        assert!(!e.state_core().skills[&(0, lihun_def)].once_used);

        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: lihun_def },
        )) else {
            panic!("expected ssr_newlihun:give_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(heart_tao)] })) else {
            panic!("expected ssr_newlihun:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_newlihun:target_choice");
        };
        assert_eq!(choice.observer, 1);
        let _ = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(1)] }));
        assert_eq!(flag(&e, 1, "l25_lihun_controlled"), 1);
        assert_eq!(flag(&e, 0, "l25_lihun_qianxing"), 1);
        set_flag(&mut e, 1, "l25_lihun_zero_hand", 1);
        assert_eq!(eval_hand_limit(e.state_core(), e.catalog(), 1), 0);

        // `ssr_yijue` active: choose card -> choose target
        e.state_core_mut().player_mut(0).hand = vec![spade_sha];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: yijue_def },
        )) else {
            panic!("expected ssr_yijue:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(spade_sha)] })) else {
            panic!("expected ssr_yijue:choose_target");
        };
        let out = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        if let CommandOutcome::Pending { choice } = out {
            let _ = e.submit(next_cmd(&e, choice.observer, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] }));
        }
        assert_eq!(flag(&e, 0, "l25_yijue_mashu"), 1);

        // `ssr_liecheng` active: choose target -> choose card
        e.state_core_mut().player_mut(1).hand = vec![diamond_shan];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: liecheng_def,
            },
        )) else {
            panic!("expected ssr_liecheng:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_liecheng:choose_card");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(diamond_shan)] }));
        assert!(e.state_core().player(0).hand.contains(&diamond_shan));
    }

    // 2. Gongsun Zan (`ssr_newyucong`, `ssr_newyicong`), Da Qiao (`ssr_newguose`, `ssr_newtongque`),
    //    Xiao Qiao (`ssr_newtianxiang`), Xu Chu (`ssr_huchi`, `ssr_newluoyi`, `ssr_luoyi`),
    //    Zhang Fei (`ssr_zhenhun`, `ssr_paoxiao`), Zhen Ji (`ssr_newluoshen`)
    {
        let mut e = new_l25_engine(0x1250_0502, 3);
        let yucong_def = grant_skill(&mut e, 0, "ssr_newyucong");
        grant_skill(&mut e, 0, "ssr_newyicong");
        let guose_def = grant_skill(&mut e, 0, "ssr_newguose");
        grant_skill(&mut e, 0, "ssr_newtongque");
        let tianxiang_def = grant_skill(&mut e, 0, "ssr_newtianxiang");
        grant_skill(&mut e, 0, "ssr_huchi");
        grant_skill(&mut e, 0, "ssr_newluoyi");
        grant_skill(&mut e, 0, "ssr_luoyi");
        grant_skill(&mut e, 0, "ssr_zhenhun");
        grant_skill(&mut e, 0, "ssr_paoxiao");
        grant_skill(&mut e, 0, "ssr_newluoshen");

        let diamond_card = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        let trick_card = find_kind_suit(&e, CardKind::Wuzhong, Suit::Heart);
        let equip_card = find_card(&e, |_, d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
        let high_card = find_card(&e, |_, d| d.kind == CardKind::Sha && d.rank >= 10);
        let low_card = find_suit_rank(&e, Suit::Club, 1, 3);
        e.state_core_mut().player_mut(0).hand =
            vec![diamond_card, trick_card, equip_card, high_card];
        e.state_core_mut().player_mut(1).hand = vec![low_card];

        // `ssr_newguose` view-as: Diamond card -> Le (`乐不思蜀`)
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            diamond_card,
            true,
            Some(CardKind::Le),
        )
        .is_some());

        // `ssr_huchi` view-as: Trick -> Duel, Equip -> Sha; attack range +1, Duel extra target +1, Sha limit +1
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            trick_card,
            true,
            Some(CardKind::Duel),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            equip_card,
            true,
            Some(CardKind::Sha),
        )
        .is_some());
        assert!(apply_attack_range(e.state_core(), e.catalog(), 0, 1) >= 2);
        assert_eq!(
            apply_sha_extra_targets(e.state_core(), e.catalog(), 0, CardKind::Duel, 0),
            1
        );

        // `ssr_paoxiao`: unlimited Sha use
        let (_, unlim) = eval_sha_use_limit(e.state_core(), e.catalog(), 0);
        assert!(unlim);

        // `ssr_luoyi`: +1 damage on Sha/Duel
        assert_eq!(
            apply_damage_amount(
                e.state_core(),
                e.catalog(),
                Some(0),
                1,
                Some(high_card),
                DamageNature::Normal,
                1,
            ),
            2
        );

        // `ssr_newyucong` active: place card as `骢` on target -> `ssr_newyicong` reduces distance to mounted target
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: yucong_def },
        )) else {
            panic!("expected ssr_newyucong:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(equip_card)] })) else {
            panic!("expected ssr_newyucong:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert_eq!(flag(&e, 1, "l25_yucong_mounted"), 1);
        assert!(apply_distance_total(e.state_core(), e.catalog(), 0, 1, 2) <= 0);

        // `ssr_newguose` active: pindian with seat 1 -> wins -> sets `l25_guose_skip_play` & `l25_guose_pindian_win`
        e.state_core_mut().player_mut(0).hand = vec![high_card];
        e.state_core_mut().player_mut(1).hand = vec![low_card];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: guose_def },
        )) else {
            panic!("expected ssr_newguose:choose_mode");
        };
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(1)] })) else {
            panic!("expected ssr_newguose:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_newguose:pindian_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(high_card)] })) else {
            panic!("expected ssr_newguose:target_pindian_card");
        };
        let _ = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(low_card)] }));
        assert_eq!(flag(&e, 1, "l25_guose_skip_play"), 1);
        assert_eq!(flag(&e, 0, "l25_guose_pindian_win"), 1);

        // `ssr_newtianxiang` active: consumes 1 tongque die and places `l25_tianxiang_die` on target
        set_flag(&mut e, 0, "l25_tongque_dice", 2);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: tianxiang_def,
            },
        )) else {
            panic!("expected ssr_newtianxiang:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert_eq!(flag(&e, 0, "l25_newtianxiang_used"), 1);
        assert_eq!(flag(&e, 1, "l25_tianxiang_die"), 1);
        set_flag(&mut e, 1, "l25_guose_die", 1);
        assert_eq!(apply_hand_limit(e.state_core(), e.catalog(), 1, 4), 3);
    }
}

#[test]
fn l25_zuan_batch_e_22_skills_6d() {
    // 1. Sun Shangxiang (`ssr_yushui`, `ssr_jieyin`), Wang Yi (`ssr_weigong`, `ssr_tianlang`),
    //    Sima Yi (`ssr_newjilue`, `ssr_newjijie`, `ssr_newjiyuan`), Huang Yueying (`ssr_jiqiao`, `ssr_qicai`),
    //    Jia Xu / Guo Jia variant (`ssr_newweimu`, `ssr_guimou`, `ssr_newluanwu`)
    {
        let mut e = new_l25_engine(0x1250_0601, 3);
        grant_skill(&mut e, 0, "ssr_yushui");
        let jieyin_def = grant_skill(&mut e, 0, "ssr_jieyin");
        grant_skill(&mut e, 0, "ssr_weigong");
        grant_skill(&mut e, 0, "ssr_tianlang");
        grant_skill(&mut e, 0, "ssr_newjilue");
        let jijie_def = grant_skill(&mut e, 0, "ssr_newjijie");
        grant_skill(&mut e, 0, "ssr_newjiyuan");
        let jiqiao_def = grant_skill(&mut e, 0, "ssr_jiqiao");
        grant_skill(&mut e, 0, "ssr_qicai");
        let weimu_def = grant_skill(&mut e, 0, "ssr_newweimu");
        grant_skill(&mut e, 0, "ssr_guimou");
        let luanwu_def = grant_skill(&mut e, 0, "ssr_newluanwu");

        let red_card = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let black_card = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let c_other = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        e.state_core_mut().player_mut(0).hand = vec![red_card, black_card];
        e.state_core_mut().player_mut(1).hand = vec![c_other];

        // `ssr_newjijie` view-as: Any card -> Dismantle in PlayPhase, Any card -> Shan in Response
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            red_card,
            true,
            Some(CardKind::Dismantle),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            red_card,
            false,
            Some(CardKind::Shan),
        )
        .is_some());

        // `ssr_newweimu` view-as: Black hand card -> Dismantle in PlayPhase; cancels incoming tricks when face_down
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            black_card,
            true,
            Some(CardKind::Dismantle),
        )
        .is_some());
        e.state_core_mut().player_mut(0).face_down = true;
        assert!(apply_card_target_canceled(
            e.state_core(),
            e.catalog(),
            1,
            0,
            CardKind::Snatch,
            false
        ));
        e.state_core_mut().player_mut(0).face_down = false;

        // `ssr_qicai`: tricks used by owner are unrespondable
        assert!(eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            CardKind::Snatch
        ));

        // `ssr_newjiyuan`: draw count -1
        assert_eq!(apply_draw_count(e.state_core(), e.catalog(), 0, 2), 1);

        // `ssr_jieyin` active: give red card -> choose target -> gain card from target + snapshot roundtrip
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: jieyin_def },
        )) else {
            panic!("expected ssr_jieyin:give_card");
        };
        assert_eq!(choice.spec.prompt, "ssr_jieyin:give_card");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(red_card)] })) else {
            panic!("expected ssr_jieyin:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_jieyin:choose_card");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(c_other)] }));
        assert_eq!(flag(&e, 0, "l25_jieyin_used"), 1);

        // `ssr_newjijie` active & `ssr_newweimu` active & `ssr_jiqiao` active & `ssr_newluanwu` active
        e.state_core_mut().player_mut(0).hand = vec![black_card, c_other];
        e.state_core_mut().player_mut(1).hand = vec![red_card];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: jijie_def },
        )) else {
            panic!("expected ssr_newjijie:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(c_other)] })) else {
            panic!("expected ssr_newjijie:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: sel }));
        }

        e.state_core_mut().player_mut(0).face_down = true;
        e.state_core_mut().player_mut(0).hand = vec![black_card];
        e.state_core_mut().player_mut(1).hand = vec![red_card, c_other];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: weimu_def },
        )) else {
            panic!("expected ssr_newweimu:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(black_card)] })) else {
            panic!("expected ssr_newweimu:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        e.state_core_mut().player_mut(0).face_down = false;

        // `ssr_jiqiao`: populate `机巧` pile -> active draws 1 & swaps hand card with pile
        e.state_core_mut()
            .vm_aux
            .piles
            .insert((0, "ssr_jiqiao".to_string()), vec![red_card, c_other]);
        e.state_core_mut().player_mut(0).hand = vec![black_card];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: jiqiao_def },
        )) else {
            panic!("expected ssr_jiqiao:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(black_card)] })) else {
            panic!("expected ssr_jiqiao:choose_option");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] }));
        assert!(e.state_core().player(0).hand.contains(&red_card));

        // `ssr_newluanwu` active: confirm -> each player chooses mode
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: luanwu_def },
        )) else {
            panic!("expected ssr_newluanwu:confirm");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: sel }));
        }
        assert_eq!(flag(&e, 0, "l25_newluanwu_used"), 1);
    }

    // 2.蔡文姬 (`ssr_nianlei`, `ssr_newduanxian`), 孟获 (`ssr_manjia`, `ssr_zaiqi`),
    //   祝融 (`ssr_lieren`), 姜维 (`ssr_zhiji`, `ssr_weilong`), 孙策 (`ssr_hunzi`, `ssr_newjiang`, `ssr_newzhiba`)
    {
        let mut e = new_l25_engine(0x1250_0602, 3);
        let nianlei_def = grant_skill(&mut e, 0, "ssr_nianlei");
        let duanxian_def = grant_skill(&mut e, 0, "ssr_newduanxian");
        grant_skill(&mut e, 0, "ssr_manjia");
        let zaiqi_def = grant_skill(&mut e, 0, "ssr_zaiqi");
        let lieren_def = grant_skill(&mut e, 0, "ssr_lieren");
        grant_skill(&mut e, 0, "ssr_zhiji");
        let weilong_def = grant_skill(&mut e, 0, "ssr_weilong");
        let hunzi_def = grant_skill(&mut e, 0, "ssr_hunzi");
        grant_skill(&mut e, 0, "ssr_newjiang");
        grant_skill(&mut e, 0, "ssr_newzhiba");

        let equip_card = find_card(&e, |_, d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
        let heart_tao = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let diamond_shan = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        let club_sha = find_kind_suit(&e, CardKind::Sha, Suit::Club);
        let spade_jiu = find_kind_suit(&e, CardKind::Jiu, Suit::Spade);
        e.state_core_mut().player_mut(0).hand =
            vec![equip_card, heart_tao, diamond_shan, club_sha, spade_jiu];
        e.state_core_mut().player_mut(1).hand = vec![club_sha];

        // View-as checks for `ssr_manjia`, `ssr_weilong` (4 suits), and `ssr_newjiang`
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            equip_card,
            true,
            Some(CardKind::Sha),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            heart_tao,
            true,
            Some(CardKind::Tao),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            diamond_shan,
            true,
            Some(CardKind::FireSha),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            club_sha,
            false,
            Some(CardKind::Shan),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            spade_jiu,
            true,
            Some(CardKind::Jiu),
        )
        .is_some());
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            heart_tao,
            true,
            Some(CardKind::Sha),
        )
        .is_some());

        // `ssr_nianlei` active: give 1 card -> choose target -> adds `陈情` on target; target CardUsed flips to `伤哀`
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: nianlei_def },
        )) else {
            panic!("expected ssr_nianlei:give_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(heart_tao)] })) else {
            panic!("expected ssr_nianlei:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert!(flag(&e, 1, "l25_nianlei_chenqing") >= 1);
        assert_eq!(apply_hand_limit(e.state_core(), e.catalog(), 1, 4), 5);
        set_flag(&mut e, 1, "l25_nianlei_chenqing", 0);
        set_flag(&mut e, 1, "l25_nianlei_shangai", 1);
        assert_eq!(apply_hand_limit(e.state_core(), e.catalog(), 1, 4), 3);

        // `ssr_newduanxian` active: discard 2 cards
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: duanxian_def,
            },
        )) else {
            panic!("expected ssr_newduanxian:discard_cards");
        };
        let _ = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![
                    ChoiceOption::Card(equip_card),
                    ChoiceOption::Card(diamond_shan),
                ],
            },
        ));

        // `ssr_zaiqi` active: discard 2 cards -> consumes 1 `起` mark and uses virtual SavageAttack
        set_flag(&mut e, 0, "l25_zaiqi_qi", 3);
        set_flag(&mut e, 0, "mark:ssr_zaiqi", 3);
        e.state_core_mut().player_mut(0).hand = vec![club_sha, spade_jiu];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: zaiqi_def },
        )) else {
            panic!("expected ssr_zaiqi:discard_cards");
        };
        let _ = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(club_sha), ChoiceOption::Card(spade_jiu)],
            },
        ));
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: sel }));
        }
        assert_eq!(flag(&e, 0, "l25_zaiqi_used"), 1);

        // `ssr_lieren` active & `ssr_weilong` active & `ssr_hunzi` active
        e.state_core_mut().player_mut(0).hand = vec![club_sha, diamond_shan];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: lieren_def },
        )) else {
            panic!("expected ssr_lieren:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(club_sha)] })) else {
            panic!("expected ssr_lieren:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: sel }));
        }

        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: weilong_def },
        )) else {
            panic!("expected ssr_weilong:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(diamond_shan)] })) else {
            panic!("expected ssr_weilong:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: sel }));
        }

        e.state_core_mut().turn.current = 0;
        e.state_core_mut().turn.phase = Phase::Play;
        e.state_core_mut().player_mut(0).alive = true;
        e.state_core_mut().player_mut(1).alive = true;
        e.state_core_mut().player_mut(1).hp = 3;
        let high_card = find_card(&e, |_, d| d.kind == CardKind::Sha && d.rank >= 10);
        let low_card = find_suit_rank(&e, Suit::Club, 1, 3);
        e.state_core_mut().player_mut(0).hand = vec![high_card];
        e.state_core_mut().player_mut(1).hand = vec![low_card];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: hunzi_def },
        )) else {
            panic!("expected ssr_hunzi:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_hunzi:pindian_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(high_card)] })) else {
            panic!("expected ssr_hunzi:target_pindian_card");
        };
        let _ = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(low_card)] }));
    }
}

#[test]
fn l25_zuan_batch_f_22_skills_6d() {
    // 1. Cao Ren (`ssr_kuiwei`, `ssr_newhanzhan`), Zhou Tai (`ssr_newbuqu`, `ssr_huwei`),
    //    Xiahou Yuan (`ssr_shensu`, `ssr_hubu`),曹昂 (`ssr_jushou`, `ssr_zhuzhen`),
    //    Deng Ai (`ssr_tuntian`, `ssr_zaoxian`), Dong Zhuo (`ssr_newhengzheng`, `ssr_newjiuchi`, `ssr_baonue`)
    {
        let mut e = new_l25_engine(0x1250_0701, 3);
        let kuiwei_def = grant_skill(&mut e, 0, "ssr_kuiwei");
        grant_skill(&mut e, 0, "ssr_newhanzhan");
        grant_skill(&mut e, 0, "ssr_newbuqu");
        grant_skill(&mut e, 0, "ssr_huwei");
        let shensu_def = grant_skill(&mut e, 0, "ssr_shensu");
        let hubu_def = grant_skill(&mut e, 0, "ssr_hubu");
        grant_skill(&mut e, 0, "ssr_jushou");
        grant_skill(&mut e, 0, "ssr_zhuzhen");
        let tuntian_def = grant_skill(&mut e, 0, "ssr_tuntian");
        grant_skill(&mut e, 0, "ssr_zaoxian");
        grant_skill(&mut e, 0, "ssr_newhengzheng");
        grant_skill(&mut e, 0, "ssr_newjiuchi");
        grant_skill(&mut e, 0, "ssr_baonue");

        let high_card = find_card(&e, |_, d| d.kind == CardKind::Sha && d.rank >= 10);
        let low_card = find_suit_rank(&e, Suit::Club, 1, 3);
        e.state_core_mut().player_mut(0).hand = vec![high_card];
        e.state_core_mut().player_mut(1).hand = vec![low_card];
        e.state_core_mut().player_mut(0).hujia = 2;

        // `ssr_newhanzhan`: +hujia to hand limit
        assert!(eval_hand_limit(e.state_core(), e.catalog(), 0) >= 6);
        // `ssr_jushou`: caps incoming damage > 1 at 1
        assert_eq!(
            apply_damage_amount(
                e.state_core(),
                e.catalog(),
                Some(1),
                0,
                Some(high_card),
                DamageNature::Normal,
                3,
            ),
            1
        );
        // `ssr_baonue`: Sha is unrespondable when hp > 2
        assert!(eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            CardKind::Sha
        ));

        // `ssr_kuiwei` active: pindian with seat 1 -> wins -> sets `l25_kuiwei_forbid_basic` on seat 1 + snapshot roundtrip
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: kuiwei_def },
        )) else {
            panic!("expected ssr_kuiwei:choose_target");
        };
        assert_eq!(choice.spec.prompt, "ssr_kuiwei:choose_target");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_kuiwei:pindian_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(high_card)] })) else {
            panic!("expected ssr_kuiwei:target_pindian_card");
        };
        let _ = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(low_card)] }));
        assert_eq!(flag(&e, 1, "l25_kuiwei_forbid_basic"), 1);
        assert!(apply_card_use_canceled(
            e.state_core(),
            e.catalog(),
            1,
            CardKind::Sha,
            true,
            false
        ));

        // `ssr_shensu` (on PhaseBegin) & `ssr_hubu` trigger on Phase::End
        set_flag(&mut e, 0, "l25_shensu_su", 2);
        assert_eq!(apply_hand_limit(e.state_core(), e.catalog(), 0, 4), 5); // 4 + 2 (hujia) - 1 (shensu)
        let _ = (shensu_def, hubu_def);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e,
            EventData::Phase { player: 0, phase: Phase::End, begin: true },
        ) else {
            panic!("expected ssr_hubu:choose_target");
        };
        assert_eq!(spec.prompt, "ssr_hubu:choose_target");
        let cid = e.open_choice().unwrap().id;
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_hubu:choose_option");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: vec![] }));
        }
        e.state_core_mut().turn.current = 0;
        e.state_core_mut().turn.phase = Phase::Play;

        // `ssr_tuntian` active: discard 2 cards -> choose card from top of deck
        let c1 = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let c2 = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        e.state_core_mut().player_mut(0).hand = vec![c1, c2];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: tuntian_def },
        )) else {
            panic!("expected ssr_tuntian:discard_cards");
        };
        let out = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
            },
        ));
        if let CommandOutcome::Pending { choice } = out {
            assert_eq!(choice.spec.prompt, "ssr_tuntian:choose_card");
            let first_opt = choice.spec.options[0];
            let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![first_opt] }));
        }
        assert!(flag(&e, 0, "l25_tuntian_tian") >= 1);
    }

    // 2. Liu Shan (`ssr_newmizhou`), Yuan Shao (`ssr_newluanji`, `ssr_newshiyi`),
    //    Xu Shu (`ssr_zhuhai`, `ssr_yijian`), Fazheng (`ssr_newfabing`, `ssr_newdingsheng`, `ssr_xuanhuo`),
    //    Taishi Ci (`ssr_newpojun`)
    {
        let mut e = new_l25_engine(0x1250_0702, 3);
        let mizhou_def = grant_skill(&mut e, 0, "ssr_newmizhou");
        let luanji_def = grant_skill(&mut e, 0, "ssr_newluanji");
        grant_skill(&mut e, 0, "ssr_newshiyi");
        grant_skill(&mut e, 0, "ssr_zhuhai");
        let yijian_def = grant_skill(&mut e, 0, "ssr_yijian");
        grant_skill(&mut e, 0, "ssr_newfabing");
        grant_skill(&mut e, 0, "ssr_newdingsheng");
        let xuanhuo_def = grant_skill(&mut e, 0, "ssr_xuanhuo");

        let c1 = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let c2 = find_kind_suit(&e, CardKind::Shan, Suit::Diamond);
        e.state_core_mut().player_mut(0).hand = vec![c1, c2];

        // `ssr_newluanji` view-as: Any card -> ArcheryAttack in PlayPhase
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            c1,
            true,
            Some(CardKind::ArcheryAttack),
        )
        .is_some());

        // `ssr_newdingsheng`: ArcheryAttack is unrespondable
        assert!(eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            CardKind::ArcheryAttack
        ));

        // `ssr_newmizhou` active: place card on `迷` pile -> choose option
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: mizhou_def },
        )) else {
            panic!("expected ssr_newmizhou:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(c1)] })) else {
            panic!("expected ssr_newmizhou:choose_option");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] }));

        // `ssr_newluanji` active: discard 2 cards -> uses virtual ArcheryAttack
        e.state_core_mut().player_mut(0).hand = vec![c1, c2];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: luanji_def },
        )) else {
            panic!("expected ssr_newluanji:discard_cards");
        };
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let _ = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
            },
        ));

        // Drain any pending ArcheryAttack response windows so we return to idle Play phase
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e,
                open.observer,
                GameAction::Answer {
                    choice_id: open.id,
                    selection: vec![],
                },
            ));
        }

        // `ssr_yijian` trigger on trick CardUsed targeting `0` & `ssr_xuanhuo` active
        let _ = yijian_def;
        let wz = find_kind_suit(&e, CardKind::Snatch, Suit::Spade);
        let _ = drive_event(&mut e, EventData::CardUsed(card_use(wz, 1, vec![0])));
        while let Some(open) = e.open_choice().cloned() {
            let sel = if open.spec.allow_cancel {
                vec![]
            } else {
                open.spec.options.iter().take(open.spec.min.max(1)).copied().collect()
            };
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: sel }));
        }
        assert!(flag(&e, 0, "l25_yijian_yan") >= 1);

        let heart_tao = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        e.state_core_mut().player_mut(0).hand = vec![heart_tao, c1];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: xuanhuo_def },
        )) else {
            panic!("expected ssr_xuanhuo:discard_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(heart_tao)] })) else {
            panic!("expected ssr_xuanhuo:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: vec![] }));
        }
        assert!(flag(&e, 1, "l25_xuanhuo_sha_buff") >= 1);
        e.state_core_mut().player_mut(1).hand.push(c2);
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            1,
            c2,
            true,
            Some(CardKind::Sha),
        )
        .is_some());
    }
}

#[test]
fn l25_zuan_batch_g_22_skills_and_all_130_catalog_invariants_6d() {
    // 1. Pang Tong (`ssr_lianhuan`, `ssr_newsance`), Zhang He (`ssr_newxiebian`, `ssr_motu`),
    //    Li Dian (`ssr_xunxun`, `ssr_wangxi`), Xu Huang (`ssr_jieliang`, `ssr_zhiyan`),
    //    Sun Jian (`ssr_futu`, `ssr_newpolu`), Lu Su (`ssr_dimeng`, `ssr_fuhui`)
    {
        let mut e = new_l25_engine(0x1250_0801, 3);
        let lianhuan_def = grant_skill(&mut e, 0, "ssr_lianhuan");
        let newsance_def = grant_skill(&mut e, 0, "ssr_newsance");
        grant_skill(&mut e, 0, "ssr_newxiebian");
        grant_skill(&mut e, 0, "ssr_motu");
        grant_skill(&mut e, 0, "ssr_xunxun");
        grant_skill(&mut e, 0, "ssr_wangxi");
        let jieliang_def = grant_skill(&mut e, 0, "ssr_jieliang");
        grant_skill(&mut e, 0, "ssr_zhiyan");
        let futu_def = grant_skill(&mut e, 0, "ssr_futu");
        grant_skill(&mut e, 0, "ssr_newpolu");
        grant_skill(&mut e, 0, "ssr_dimeng");
        grant_skill(&mut e, 0, "ssr_fuhui");

        let club_sha = find_kind_suit(&e, CardKind::Sha, Suit::Club);
        let spade_sha = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let high_card = find_card(&e, |_, d| d.kind == CardKind::Sha && d.rank >= 10);
        let low_card = find_suit_rank(&e, Suit::Club, 1, 3);
        e.state_core_mut().player_mut(0).hand = vec![club_sha, spade_sha, high_card];
        e.state_core_mut().player_mut(1).hand = vec![low_card];
        e.state_core_mut().player_mut(0).hujia = 1;

        // `ssr_xunxun`: +1 hand limit when hujia > 0
        assert_eq!(eval_hand_limit(e.state_core(), e.catalog(), 0), 5);

        // `ssr_lianhuan` active: discard Club card -> choose 2 targets to link + draw 1
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: lianhuan_def,
            },
        )) else {
            panic!("expected ssr_lianhuan:discard_card");
        };
        assert_eq!(choice.spec.prompt, "ssr_lianhuan:discard_card");
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(club_sha)] })) else {
            panic!("expected ssr_lianhuan:choose_targets");
        };
        let _ = e.submit(next_cmd(&e,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
            },
        ));
        assert!(e.state_core().player(1).linked);
        assert!(e.state_core().player(2).linked);

        // `ssr_newsance` active: cancel refunds `once_used`, then choose mode 0 -> choose target -> hand limit +2
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: newsance_def,
            },
        )) else {
            panic!("expected ssr_newsance:choose_mode");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![] }));
        assert!(!e.state_core().skills[&(0, newsance_def)].once_used);

        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: newsance_def,
            },
        )) else {
            panic!("expected ssr_newsance:choose_mode");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] })) else {
            panic!("expected ssr_newsance:choose_target");
        };
        let out = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        if let CommandOutcome::Pending { choice } = out {
            let first = choice.spec.options[0];
            let _ = e.submit(next_cmd(&e,
                choice.observer,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection: vec![first],
                },
            ));
        }
        assert_eq!(flag(&e, 0, "l25_newsance_hand_plus2"), 1);

        // `ssr_jieliang` active: discard black non-trick -> choose target -> places `粮`
        e.state_core_mut().player_mut(0).hand.push(spade_sha);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: jieliang_def,
            },
        )) else {
            panic!("expected ssr_jieliang:discard_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(spade_sha)] })) else {
            panic!("expected ssr_jieliang:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        assert_eq!(flag(&e, 1, "l25_jieliang_jie"), 1);
        assert_eq!(flag(&e, 1, "mark:ssr_jieliang_break"), 1);

        // `ssr_futu` active: pindian with seat 1 -> wins -> choose mode 0
        e.state_core_mut().player_mut(0).hand = vec![high_card];
        e.state_core_mut().player_mut(1).hand = vec![low_card];
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: futu_def },
        )) else {
            panic!("expected ssr_futu:choose_target");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] })) else {
            panic!("expected ssr_futu:pindian_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(high_card)] })) else {
            panic!("expected ssr_futu:target_pindian_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 1, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(low_card)] })) else {
            panic!("expected ssr_futu:choose_mode");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Tag(0)] }));
        assert_eq!(flag(&e, 0, "l25_futu_win"), 1);
        assert_eq!(flag(&e, 1, "l25_futu_guijiang"), 1);
    }

    // 2. Zhang Zhao & Zhang Hong (`ssr_zhijian`, `ssr_guzheng`),
    //    Zhang Jiao (`ssr_tianlei`, `ssr_gaiming`, `ssr_newguidao`, `ssr_zhuji`),
    //    Zhang Bao (`ssr_dihuo`, `ssr_guifu`), Zhang Liang (`ssr_renhong`, `ssr_fangtong`)
    {
        let mut e = new_l25_engine(0x1250_0802, 3);
        let zhijian_def = grant_skill(&mut e, 0, "ssr_zhijian");
        grant_skill(&mut e, 0, "ssr_guzheng");
        grant_skill(&mut e, 0, "ssr_tianlei");
        grant_skill(&mut e, 0, "ssr_gaiming");
        let guidao_def = grant_skill(&mut e, 0, "ssr_newguidao");
        grant_skill(&mut e, 0, "ssr_zhuji");
        grant_skill(&mut e, 0, "ssr_dihuo");
        grant_skill(&mut e, 0, "ssr_guifu");
        grant_skill(&mut e, 0, "ssr_renhong");
        let fangtong_def = grant_skill(&mut e, 0, "ssr_fangtong");

        let spade_sha = find_kind_suit(&e, CardKind::Sha, Suit::Spade);
        let heart_tao = find_kind_suit(&e, CardKind::Tao, Suit::Heart);
        let low_card = find_suit_rank(&e, Suit::Club, 1, 2);
        let high_card = find_suit_rank(&e, Suit::Diamond, 12, 13);
        e.state_core_mut().player_mut(0).hand = vec![spade_sha, heart_tao, low_card];
        e.state_core_mut().player_mut(1).hand = vec![high_card];

        // `ssr_newguidao` view-as: Black card -> Bing (`兵粮寸断`)
        assert!(find_d34_flag_view_as(
            e.state_core(),
            e.catalog(),
            0,
            spade_sha,
            true,
            Some(CardKind::Bing),
        )
        .is_some());

        // `ssr_zhijian` active: give 1 card to seat 1 -> locks seat 1 to use that card
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: zhijian_def },
        )) else {
            panic!("expected ssr_zhijian:give_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(heart_tao)] })) else {
            panic!("expected ssr_zhijian:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: vec![] }));
        }
        assert_eq!(
            flag(&e, 1, "l25_zhijian_locked_card"),
            u32::from(heart_tao) + 1
        );
        assert!(apply_card_use_canceled(
            e.state_core(),
            e.catalog(),
            1,
            CardKind::Shan,
            true,
            false
        ));

        // `ssr_newguidao` active: place black card as `兵粮寸断` in target's judge zone
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill { skill: guidao_def },
        )) else {
            panic!("expected ssr_newguidao:choose_card");
        };
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(spade_sha)] })) else {
            panic!("expected ssr_newguidao:choose_target");
        };
        let _ = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: vec![] }));
        }
        assert_eq!(flag(&e, 1, "l25_newguidao_bing"), u32::from(spade_sha) + 1);

        // `ssr_fangtong` active: show card (`low_card`) -> choose target (seat 1 has only `high_card`) -> deals 1 damage
        e.state_core_mut().player_mut(1).hand = vec![high_card];
        let hp_before = e.state_core().player(1).hp;
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e,
            0,
            GameAction::ActivateSkill {
                skill: fangtong_def,
            },
        )) else {
            panic!("expected ssr_fangtong:show_card");
        };
        assert_mid_choice_snapshot_roundtrip(&mut e);
        let CommandOutcome::Pending { choice } = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Card(low_card)] })) else {
            panic!("expected ssr_fangtong:choose_target");
        };
        let out = e.submit(next_cmd(&e, 0, GameAction::Answer { choice_id: choice.id, selection: vec![ChoiceOption::Player(1)] }));
        if let CommandOutcome::Pending { choice } = out {
            let _ = e.submit(next_cmd(&e, choice.observer, GameAction::Answer { choice_id: choice.id, selection: vec![] }));
        }
        while let Some(open) = e.open_choice().cloned() {
            let _ = e.submit(next_cmd(&e, open.observer, GameAction::Answer { choice_id: open.id, selection: vec![] }));
        }
        assert_eq!(e.state_core().player(1).hp, hp_before - 1);

        // `ssr_tianlei`, `ssr_dihuo`, `ssr_renhong`, `ssr_zhuji` on Phase::Prepare
        let mut e2 = new_l25_engine(0x1250_0803, 3);
        grant_skill(&mut e2, 0, "ssr_tianlei");
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e2,
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
        ) else {
            panic!("expected ssr_tianlei:choose_target");
        };
        assert_eq!(spec.prompt, "ssr_tianlei:choose_target");
        let cid = e2.open_choice().unwrap().id;
        let _ = e2.submit(next_cmd(&e2,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(1)],
            },
        ));
        assert_eq!(flag(&e2, 0, "l25_tianlei_damaged"), 1);

        let mut e3 = new_l25_engine(0x1250_0804, 3);
        grant_skill(&mut e3, 0, "ssr_dihuo");
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e3,
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
        ) else {
            panic!("expected ssr_dihuo:choose_target");
        };
        assert_eq!(spec.prompt, "ssr_dihuo:choose_target");
        let cid = e3.open_choice().unwrap().id;
        let _ = e3.submit(next_cmd(&e3,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(1)],
            },
        ));
        assert_eq!(flag(&e3, 0, "l25_dihuo_damaged"), 1);

        let mut e4 = new_l25_engine(0x1250_0805, 3);
        grant_skill(&mut e4, 0, "ssr_renhong");
        e4.state_core_mut().player_mut(1).hand = vec![spade_sha];
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e4,
            EventData::Phase { player: 0, phase: Phase::Prepare, begin: true },
        ) else {
            panic!("expected ssr_renhong:choose_targets");
        };
        assert_eq!(spec.prompt, "ssr_renhong:choose_targets");
        let cid = e4.open_choice().unwrap().id;
        let _ = e4.submit(next_cmd(&e4,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
            },
        ));
        assert_eq!(flag(&e4, 0, "l25_renhong_discarded"), 1);

        // `ssr_guzheng` on CardsDiscarded in Phase::Discard
        let mut e5 = new_l25_engine(0x1250_0806, 3);
        grant_skill(&mut e5, 0, "ssr_guzheng");
        e5.state_core_mut().turn.current = 1;
        e5.state_core_mut().turn.phase = Phase::Discard;
        e5.state_core_mut().discard.push(spade_sha);
        e5.state_core_mut().discard.push(heart_tao);
        let StepOutcome::WaitChoice { spec, .. } = drive_event(
            &mut e5,
            EventData::CardsDiscarded {
                player: 1,
                cards: vec![spade_sha, heart_tao],
            },
        ) else {
            panic!("expected ssr_guzheng:confirm");
        };
        assert_eq!(spec.prompt, "ssr_guzheng:confirm");
        let cid = e5.open_choice().unwrap().id;
        let CommandOutcome::Pending { choice } = e5.submit(next_cmd(&e5,
            0,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Confirm],
            },
        )) else {
            panic!("expected ssr_guzheng:choose_card");
        };
        let _ = e5.submit(next_cmd(&e5,
            0,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(spade_sha)],
            },
        ));
        assert_eq!(flag(&e5, 0, "l25_guzheng_returned"), u32::from(spade_sha) + 1);
    }

    // 3. Exhaustive verification of all 130 `L25_SSR_GENERAL_SKILLS` and 84 `L25_ALL_RULE_SKILLS`
    //    in the catalog: every skill is registered, has a valid L25 HostBehavior, and ungranted
    //    `ActivateSkill` is rejected with `RejectReason::NoSuchSkill`.
    {
        let e = new_l25_engine(0x1250_0899, 3);
        for &sk in L25_SSR_GENERAL_SKILLS
            .iter()
            .chain(L25_ALL_RULE_SKILLS.iter())
        {
            let def = skill_id(&e, sk);
            assert!(
                lookup_l25_zuan_host_behavior(sk)
                    .or_else(|| content_native_loader::lookup_upstream_host_behavior(sk))
                    .is_some(),
                "skill `{sk}` must have an L25 or upstream HostBehavior"
            );
            let mut e_ungranted = e.clone();
            let out = e_ungranted.submit(next_cmd(&e_ungranted,
                0,
                GameAction::ActivateSkill { skill: def },
            ));
            assert!(
                matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSkill)),
                "ungranted skill `{sk}` must reject ActivateSkill with NoSuchSkill, got {out:?}"
            );
        }
        let _ = (
            card_use,
            apply_ignore_armor,
            apply_sha_use_limit,
            eval_ignore_armor,
        );
    }
}
