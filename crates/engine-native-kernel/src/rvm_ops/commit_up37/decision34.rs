//! Batch 25 (Decision 34 Complete Rule & Campaign Closure): authoritative host-side
//! resolvers for the remaining per-source sub-skills, standalone campaign state machines,
//! and HUD mark dispatch across `L29`, `L10`, `U06`, `U07`, `U08`, `L20`, `L28`, `U02/L03`,
//! `L11`, `L19`, and `L21`.

use super::commit::{discard_some, pop_deck, push_pile};
use super::*;

pub const MBCMJIUSHI_ID: u32 = 1_920_001_001;
pub const MBCMFANGZHU_ID: u32 = 1_920_001_002;
pub const MBJUEJIN_ID: u32 = 1_920_001_003;

pub const K_DANGGU_ID: u32 = 1_920_002_001;
pub const K_MOWANG_ID: u32 = 1_920_002_002;
pub const L10_SCS_SKILL_ID: u32 = 1_920_002_003;

pub const HUANLE_KUWU_INTERACTIVE_ID: u32 = 1_920_003_001;

pub const DIAOBINGQIANJIANG_ID: u32 = 1_920_004_001;
pub const DUOQI_ID: u32 = 1_920_004_002;
pub const YOUZHAN_ID: u32 = 1_920_004_003;

pub const YXS_MENSHEN2_GUARD_ID: u32 = 1_920_005_001;

pub const L20_MARK_DISPATCH_ID: u32 = 1_920_006_001;

pub const TH_DUNSHI_ID: u32 = 1_920_007_011;
pub const TH_JINGXIE_ID: u32 = 1_920_007_012;
pub const TH_QIAOSI_ID: u32 = 1_920_007_013;
pub const TH_YEYAN_ID: u32 = 1_920_007_014;
pub const TH_TAIPINGYAOSHU_ID: u32 = 1_920_007_015;
pub const TH_SKILL_DISPATCH_ID: u32 = 1_920_007_016;

pub const L01_YIYI_ID: u32 = 1_920_008_001;
pub const L06_WUXING_ID: u32 = 1_920_008_002;
pub const U06_HUANLE_SKILL_ID: u32 = 1_920_008_003;
pub const U07_WUDIAN_SKILL_ID: u32 = 1_920_008_004;
pub const U08_YXS_SKILL_ID: u32 = 1_920_008_005;
pub const L29_LONGXUE_SKILL_ID: u32 = 1_920_008_006;

pub const L20_MARK_IDS: [(&str, u32); 14] = [
    ("longmark", 1_920_060_001),
    ("biaoji", 1_920_060_002),
    ("danlie", 1_920_060_003),
    ("fengyin", 1_920_060_004),
    ("fuhun", 1_920_060_005),
    ("guimen", 1_920_060_006),
    ("hejie", 1_920_060_007),
    ("huxiao", 1_920_060_008),
    ("jianying", 1_920_060_009),
    ("jinfan", 1_920_060_010),
    ("kuangbao", 1_920_060_011),
    ("lunhui", 1_920_060_012),
    ("mohua", 1_920_060_013),
    ("niepan", 1_920_060_014),
];

pub(super) fn try_commit_decision34(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    value: u32,
) -> bool {
    match value {
        MBCMJIUSHI_ID => resolve_mbcmjiushi(ctx, owner),
        MBCMFANGZHU_ID => resolve_mbcmfangzhu(ctx, owner),
        MBJUEJIN_ID => resolve_mbjuejin(ctx, owner),
        K_DANGGU_ID => resolve_k_danggu(ctx, owner),
        K_MOWANG_ID => resolve_k_mowang(ctx, owner),
        L10_SCS_SKILL_ID => super::decision34_ext::resolve_l10_scs_skill(ctx, executing, owner),
        HUANLE_KUWU_INTERACTIVE_ID | U06_HUANLE_SKILL_ID => {}
        DIAOBINGQIANJIANG_ID => resolve_diaobingqianjiang(ctx, owner),
        DUOQI_ID => resolve_duoqi(ctx, owner),
        YOUZHAN_ID => resolve_youzhan(ctx, owner),
        YXS_MENSHEN2_GUARD_ID => resolve_yxs_menshen2_guard(ctx, owner),
        L20_MARK_DISPATCH_ID => resolve_l20_mark_dispatch(ctx, owner),
        TH_DUNSHI_ID => resolve_th_dunshi(ctx, owner),
        TH_JINGXIE_ID => resolve_th_jingxie(ctx, owner),
        TH_QIAOSI_ID => resolve_th_qiaosi(ctx, owner),
        TH_YEYAN_ID => resolve_th_yeyan(ctx, owner),
        TH_TAIPINGYAOSHU_ID => resolve_th_taipingyaoshu(ctx, owner),
        TH_SKILL_DISPATCH_ID => resolve_th_extra_skill(ctx, executing, owner),
        L01_YIYI_ID => super::decision34_ext::resolve_l01_yiyi_card(ctx, owner),
        L06_WUXING_ID => super::decision34_ext::resolve_l06_wuxing_skill(ctx, executing, owner),
        U07_WUDIAN_SKILL_ID => {
            super::decision34_ext::resolve_u07_wudian_skill(ctx, executing, owner)
        }
        U08_YXS_SKILL_ID => {
            super::decision34_u08_l19_l20_l27::resolve_u08_yxs_skill(ctx, executing, owner)
        }
        L29_LONGXUE_SKILL_ID => {
            super::decision34_ext::resolve_l29_longxue_skill(ctx, executing, owner)
        }
        v if (1_926_000_000..1_927_000_000).contains(&v) => {
            return super::decision34_pack9::try_commit_decision34_pack9(ctx, executing, owner, v)
        }
        _ => match executing {
            "mbcmjiushi" => resolve_mbcmjiushi(ctx, owner),
            "mbcmfangzhu" => resolve_mbcmfangzhu(ctx, owner),
            "mbjuejin" => resolve_mbjuejin(ctx, owner),
            "mbcmxscw" | "mbcmqingzheng" => {
                super::decision34_ext::resolve_l29_longxue_skill(ctx, executing, owner)
            }
            "k_danggu" => resolve_k_danggu(ctx, owner),
            "k_mowang" => resolve_k_mowang(ctx, owner),
            s if s.starts_with("scs") => {
                super::decision34_ext::resolve_l10_scs_skill(ctx, s, owner)
            }
            "yiyi" => super::decision34_ext::resolve_l01_yiyi_card(ctx, owner),
            "_shengke" | "wuxing_shengke" | "wuxingpan_skill" => {
                super::decision34_ext::resolve_l06_wuxing_skill(ctx, executing, owner)
            }
            "diaobingqianjiang" => resolve_diaobingqianjiang(ctx, owner),
            "duoqi" => resolve_duoqi(ctx, owner),
            "youzhan" => resolve_youzhan(ctx, owner),
            "youdishenru" | "g_youdishenru" | "ciqiu_dying" | "kangyin2" => {
                super::decision34_ext::resolve_u07_wudian_skill(ctx, executing, owner)
            }
            "yxs_menshen2_guard" => resolve_yxs_menshen2_guard(ctx, owner),
            "l20_mark_dispatch" => resolve_l20_mark_dispatch(ctx, owner),
            "th_dunshi" => resolve_th_dunshi(ctx, owner),
            "th_jingxie" => resolve_th_jingxie(ctx, owner),
            "th_qiaosi" => resolve_th_qiaosi(ctx, owner),
            "th_yeyan" => resolve_th_yeyan(ctx, owner),
            "th_taipingyaoshu" => resolve_th_taipingyaoshu(ctx, owner),
            "miaojian" | "shhlianhua" | "olqisi" | "xinfu_pdgyingshi" | "rejici" | "qinyin" => {
                resolve_th_extra_skill(ctx, executing, owner)
            }
            s if s.starts_with("th_") => resolve_th_extra_skill(ctx, s, owner),
            s if super::decision34_u08_l19_l20_l27::is_u08_yxs_skill(s) => {
                super::decision34_u08_l19_l20_l27::resolve_u08_yxs_skill(ctx, s, owner)
            }
            _ => {
                return super::decision34_pack9::try_commit_decision34_pack9(
                    ctx, executing, owner, value,
                )
            }
        },
    }
    true
}

fn is_trick_kind(k: CardKind) -> bool {
    !k.is_equip()
        && !matches!(
            k,
            CardKind::Sha
                | CardKind::FireSha
                | CardKind::LeiSha
                | CardKind::Shan
                | CardKind::Tao
                | CardKind::Jiu
                | CardKind::ShouLiJian
        )
}

fn is_daoxin_unlocked(ctx: &Ctx<'_>, owner: PlayerId, skill_name: &str, threshold: u8) -> bool {
    if ctx.core.player(owner).mbqianlong_marks >= threshold {
        return true;
    }
    if let Some(def) = ctx.catalog.skill_by_name(skill_name) {
        if ctx.core.has_skill(owner, def) {
            return true;
        }
    }
    false
}

/// `L29` (`mbcmjiushi` 酒诗, unlocked at `mbqianlong >= 50`):
/// - When face-up (`!face_down`): flips face-down (`face_down = true`) to apply virtual
///   `jiu` boost (`jiu_active = true`, `mbcmjiushi_jiu_used = 1`).
/// - When face-down (`face_down`): flips face-up (`face_down = false`) and gains 1 trick
///   card from deck/discard (`mbcmjiushi_trick_gained = 1`).
fn resolve_mbcmjiushi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if !is_daoxin_unlocked(ctx, owner, "mbcmjiushi", 50) {
        set_flag(ctx.core, owner, "mbqianlong_locked_guard", 1);
        return;
    }
    if !ctx.core.player(owner).face_down {
        ctx.core.player_mut(owner).face_down = true;
        ctx.core.turn.jiu_active = true;
        set_flag(ctx.core, owner, "mbcmjiushi_jiu_used", 1);
    } else {
        ctx.core.player_mut(owner).face_down = false;
        let trick = ctx
            .core
            .deck
            .iter()
            .rposition(|&c| is_trick_kind(ctx.catalog.card_kind(c)))
            .map(|i| ctx.core.deck.remove(i))
            .or_else(|| {
                ctx.core
                    .discard
                    .iter()
                    .rposition(|&c| is_trick_kind(ctx.catalog.card_kind(c)))
                    .map(|i| ctx.core.discard.remove(i))
            })
            .or_else(|| pop_deck(ctx));
        if let Some(c) = trick {
            ctx.core.player_mut(owner).hand.push(c);
            ctx.out.push(LogEvent::CardsDrawn {
                player: owner,
                cards: vec![c],
            });
            set_flag(ctx.core, owner, "mbcmjiushi_trick_gained", 1);
        }
    }
}

/// `L29` (`mbcmfangzhu` 放逐, unlocked at `mbqianlong >= 75`):
/// On damage, chooses another alive player (`mbcmfangzhu_target` or `hostile_target`),
/// turns that player over (`face_down = !face_down`), and that player draws `lost_hp = (max_hp - hp).max(1)` cards.
fn resolve_mbcmfangzhu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if !is_daoxin_unlocked(ctx, owner, "mbcmfangzhu", 75) {
        set_flag(ctx.core, owner, "mbqianlong_locked_guard", 1);
        return;
    }
    let target = {
        let raw = flag(ctx.core, owner, "mbcmfangzhu_target");
        if raw > 0 && (raw as u8) < (ctx.core.players.len() as u8) && (raw as u8) != owner {
            raw as u8
        } else {
            let Some(t) = hostile_target(ctx, owner) else {
                return;
            };
            t
        }
    };
    let lost_hp = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
    let opt = flag(ctx.core, owner, "mbcmfangzhu_option");
    if opt == 1 {
        set_flag(ctx.core, target, "mbcmfangzhu_baiban", 1);
    } else if opt == 2 {
        set_flag(ctx.core, target, "mbcmfangzhu_jin", 1);
    } else {
        let next_face_down = !ctx.core.player(target).face_down;
        ctx.core.player_mut(target).face_down = next_face_down;
    }
    set_flag(ctx.core, owner, "mbcmfangzhu_last_target", u32::from(target));
    set_flag(ctx.core, owner, "mbcmfangzhu_drawn", lost_hp);
    ctx.start_draw(target, lost_hp);
}

/// `L29` (`mbjuejin` 绝进, unlocked at `mbqianlong >= 99`, limited 1/game):
/// Grants 2 hujia to owner, sets `mbjuejin_active = 1`, moves `shan`/`tao`/`jiu` cards
/// from hands/deck into the `"xiang"` expansion pile, and grants `+2` sha damage bonus.
fn resolve_mbjuejin(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if !is_daoxin_unlocked(ctx, owner, "mbjuejin", 99) {
        set_flag(ctx.core, owner, "mbqianlong_locked_guard", 1);
        return;
    }
    if flag(ctx.core, owner, "mbjuejin_used") != 0 {
        return;
    }
    set_flag(ctx.core, owner, "mbjuejin_used", 1);
    set_flag(ctx.core, owner, "mbjuejin_active", 1);
    ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(2);
    let mut moved = 0u32;
    for p in 0..ctx.core.players.len() as PlayerId {
        let retained: Vec<CardId> = ctx
            .core
            .player(p)
            .hand
            .iter()
            .copied()
            .filter(|&c| {
                matches!(
                    ctx.catalog.card_kind(c),
                    CardKind::Shan | CardKind::Tao | CardKind::Jiu
                )
            })
            .collect();
        for c in retained {
            ctx.take_hand_card(p, c);
            push_pile(ctx, owner, "xiang", c);
            moved += 1;
        }
    }
    if moved == 0 {
        if let Some(c) = pop_deck(ctx) {
            push_pile(ctx, owner, "xiang", c);
            moved = 1;
        }
    }
    set_flag(ctx.core, owner, "mbjuejin_xiang_count", moved);
}

/// `L10` (`k_danggu` 党锢 — 十常侍 `shichangshi`):
/// Initializes the 10-eunuch roster mask (`0x3FF`), selects the active eunuch (`0..10`),
/// activates its companion skill flag, and draws 1 card.
fn resolve_k_danggu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut mask = flag(ctx.core, owner, "k_danggu_roster_mask");
    if mask == 0 {
        if flag(ctx.core, owner, "k_mowang_exhausted") != 0 {
            return;
        }
        mask = 0x3FF; // 10 eunuchs bitmask (bits 0..=9)
        set_flag(ctx.core, owner, "k_danggu_roster_mask", mask);
    }
    let preferred = flag(ctx.core, owner, "k_danggu_pick_index") % 10;
    let chosen_idx = (0..10u32)
        .map(|off| (preferred + off) % 10)
        .find(|&idx| (mask & (1u32 << idx)) != 0)
        .unwrap_or(0);
    set_flag(ctx.core, owner, "k_danggu_active_eunuch", chosen_idx + 1);
    set_flag(ctx.core, owner, "k_danggu_triggered", 1);
    super::decision34_ext::apply_l10_danggu_companion(ctx, owner, chosen_idx, mask);
    ctx.start_draw(owner, 1);
}

/// `L10` (`k_mowang` 殁亡 — 十常侍 `shichangshi`):
/// Retires the current active eunuch bit from `k_danggu_roster_mask`; if remaining eunuchs
/// exist, revives/restores owner to 1 HP, clears dying, rotates to the next eunuch, and draws 1 card.
fn resolve_k_mowang(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut mask = flag(ctx.core, owner, "k_danggu_roster_mask");
    if mask == 0 && flag(ctx.core, owner, "k_mowang_exhausted") == 0 {
        mask = 0x3FF;
    }
    let active = flag(ctx.core, owner, "k_danggu_active_eunuch").saturating_sub(1);
    let bit = 1u32 << (active.min(9));
    let next_mask = mask & !bit;
    set_flag(ctx.core, owner, "k_danggu_roster_mask", next_mask);
    if next_mask != 0 {
        let next_idx = (0..10u32)
            .find(|&idx| (next_mask & (1u32 << idx)) != 0)
            .unwrap_or(0);
        set_flag(ctx.core, owner, "k_danggu_active_eunuch", next_idx + 1);
        set_flag(ctx.core, owner, "k_mowang_revived", 1);
        set_flag(ctx.core, owner, "k_mowang_return_resolved", 1);
        super::decision34_ext::apply_l10_danggu_companion(ctx, owner, next_idx, next_mask);
        ctx.core.player_mut(owner).dying = false;
        if ctx.core.player(owner).hp <= 0 {
            ctx.core.player_mut(owner).hp = 1;
        }
        ctx.start_draw(owner, 1);
    } else {
        set_flag(ctx.core, owner, "k_mowang_exhausted", 1);
    }
}

/// `U07` (`diaobingqianjiang` 调兵遣将 — `archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js:4-142`):
/// 1. `contentBefore`: user draws 1 card (`player.draw()` at `card.js:15`) and reveals `ceil(alive / 2)` cards (`card.js:41`).
/// 2. `content`: in seat order from user, each player with hand cards swaps 1 hand card with 1 market card (`card.js:87-94`).
/// 3. `contentAfter`: user places `1..=market.len()` chosen market cards back on top of `deck` (`card.js:130`) and discards the rest (`card.js:135`).
pub(super) fn resolve_diaobingqianjiang(ctx: &mut Ctx<'_>, owner: PlayerId) {
    draw_cards_sync(ctx, owner, 1);
    let reveal_count = ctx.core.alive_players().count().div_ceil(2).max(1);
    let mut market = Vec::with_capacity(reveal_count);
    for _ in 0..reveal_count {
        if let Some(c) = pop_deck(ctx) {
            market.push(c);
        }
    }
    let mut swapped = 0u32;
    for p in ctx.core.seat_order_from(owner) {
        if !ctx.core.player(p).alive || market.is_empty() {
            continue;
        }
        if let Some(hand_c) = choose_own_card(ctx, p, false) {
            ctx.take_hand_card(p, hand_c);
            let m_card = market.remove(0);
            ctx.core.player_mut(p).hand.push(m_card);
            market.push(hand_c);
            swapped += 1;
        }
    }
    let top_req = flag(ctx.core, owner, "diaobingqianjiang_top_count");
    let top_n = if top_req == 0 {
        usize::from(!market.is_empty())
    } else {
        (top_req as usize).min(market.len())
    };
    let mut returned_top = 0u32;
    for _ in 0..top_n {
        if let Some(c) = market.pop() {
            ctx.core.deck.push(c);
            returned_top += 1;
        }
    }
    if !market.is_empty() {
        discard_some(ctx, owner, market);
    }
    set_flag(ctx.core, owner, "diaobingqianjiang_swapped", swapped);
    set_flag(ctx.core, owner, "diaobingqianjiang_returned_top", returned_top);
    set_flag(ctx.core, owner, "diaobingqianjiang_resolved", 1);
}

/// `U07` (`duoqi` 夺气 — `archive/upstream/noname/apps/core/extension/玩点论杀/character/skill.js:109-159`):
/// Requires `currentPhase != owner` (`skill.js:112`) and at least 1 `zhucheng` card (`skill.js:115`).
/// Discards 1 `zhucheng` card (`skill.js:141-142`) and terminates the current player's play phase (`evt.skipped = true`, `skill.js:151`).
fn resolve_duoqi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if ctx.core.turn.current == owner {
        set_flag(ctx.core, owner, "duoqi_blocked_self_turn", 1);
        return;
    }
    let zc = ctx
        .core
        .player_mut(owner)
        .zhucheng
        .pop()
        .or_else(|| {
            ctx.core
                .vm_aux
                .piles
                .get_mut(&(owner, "zhucheng".to_string()))
                .and_then(Vec::pop)
        });
    let Some(c) = zc else {
        set_flag(ctx.core, owner, "duoqi_blocked_no_zhucheng", 1);
        return;
    };
    discard_some(ctx, owner, vec![c]);
    ctx.core.turn.skip_play_phase = true;
    set_flag(ctx.core, ctx.core.turn.current, "duoqi_phase_use_skipped", 1);
    set_flag(ctx.core, owner, "duoqi_triggered", 1);
}

/// `U07` (`youzhan` 诱战 — `archive/upstream/noname/apps/core/extension/玩点论杀/character/skill.js:422-447` & `card/card.js:169-202`):
/// Requires an equip card in `he` (`skill.js:425`). Discards that equip card to invoke virtual `youdishenru`:
/// cancels the incoming Sha (`card.js:182`), and unless the attacker responds with another Sha, deals 1 counter-damage (`card.js:199`).
fn resolve_youzhan(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let equip_card = ctx
        .core
        .player(owner)
        .hand
        .iter()
        .chain(ctx.core.player(owner).equips.iter())
        .copied()
        .find(|&c| ctx.catalog.card_kind(c).is_equip());
    let Some(eq) = equip_card else {
        set_flag(ctx.core, owner, "youzhan_blocked_no_equip", 1);
        return;
    };
    if ctx.core.player(owner).hand.contains(&eq) {
        ctx.take_hand_card(owner, eq);
    } else {
        ctx.take_owned_card(owner, eq);
    }
    discard_some(ctx, owner, vec![eq]);
    set_flag(ctx.core, owner, "youzhan_triggered", 1);
    set_flag(ctx.core, owner, "youdishenru_nullified_sha", 1);
    let attacker = hostile_target(ctx, owner).unwrap_or(owner);
    if attacker != owner {
        let sha_in_hand = ctx
            .core
            .player(attacker)
            .hand
            .iter()
            .copied()
            .find(|&c| ctx.catalog.card_kind(c).is_sha_like());
        if flag(ctx.core, attacker, "youdishenru_refuse_sha") == 0 {
            if let Some(sc) = sha_in_hand {
                ctx.take_hand_card(attacker, sc);
                discard_some(ctx, attacker, vec![sc]);
                set_flag(ctx.core, attacker, "youdishenru_responded_sha", 1);
                return;
            }
        }
        set_flag(ctx.core, owner, "youdishenru_counter_damage", 1);
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            attacker,
            1,
        )));
    }
}

/// `U08` (`yxs_menshen2_guard` 门神承受伤害转移):
fn resolve_yxs_menshen2_guard(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let Some(menshen_granter) = ctx
        .catalog
        .skill_by_name("yxs_menshen2")
        .and_then(|def| ctx.core.vm_aux.marks.get(&(owner, u32::from(def))).copied())
        .filter(|&v| v >= 0)
        .map(|v| v as u8)
        .or_else(|| {
            ctx.core
                .vm_aux
                .flags
                .get(&(owner, "yxs_menshen2_granter".to_string()))
                .copied()
                .map(|v| v as u8)
        })
    else {
        return;
    };
    if (menshen_granter as usize) < ctx.core.players.len()
        && menshen_granter != owner
        && ctx.core.player(menshen_granter).alive
    {
        set_flag(ctx.core, owner, "yxs_menshen2_guarded", 1);
        set_flag(
            ctx.core,
            owner,
            "yxs_menshen2_transferred_to",
            u32::from(menshen_granter),
        );
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            None,
            menshen_granter,
            1,
        )));
    }
}

/// `L20` (`l20_mark_dispatch` — 14 HUD marks):
fn resolve_l20_mark_dispatch(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let idx = (flag(ctx.core, owner, "l20_mark_index") as usize) % L20_MARK_IDS.len();
    let (name, mark_id) = L20_MARK_IDS[idx];
    let delta = flag(ctx.core, owner, "l20_mark_delta").max(1) as i32;
    let next = ctx
        .core
        .vm_aux
        .marks
        .get(&(owner, mark_id))
        .copied()
        .unwrap_or(0)
        + delta;
    ctx.core.vm_aux.marks.insert((owner, mark_id), next);
    set_flag(ctx.core, owner, &format!("l20_mark:{name}"), next as u32);
    if name == "niepan" && ctx.core.player(owner).hp < 3 {
        ctx.core.player_mut(owner).hp = 3.min(ctx.core.player(owner).max_hp as i32);
    }
}

/// `L28` (`th_dunshi` 遁世 — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3783`):
/// Once per turn, views as a basic card (`sha`/`shan`/`tao`/`jiu`) from `"仁义礼智信"` list;
/// when current turn player next deals damage, prevents that damage, loses 1 max_hp,
/// removes the used basic name, and grants 1 `"仁义礼智信"` virtue skill / draws cards.
fn resolve_th_dunshi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let removed_mask = flag(ctx.core, owner, "th_dunshi_removed_mask");
    let choice_bit = flag(ctx.core, owner, "th_dunshi_basic_bit").max(1) & 0xF;
    if (removed_mask & choice_bit) != 0 {
        return;
    }
    set_flag(ctx.core, owner, "th_dunshi_used_basic_bit", choice_bit);
    set_flag(ctx.core, owner, "th_dunshi_triggered", 1);
    if flag(ctx.core, owner, "th_dunshi_prevent_damage") != 0 {
        let next_mask = (removed_mask | choice_bit) & 0xF;
        set_flag(ctx.core, owner, "th_dunshi_removed_mask", next_mask);
        set_flag(ctx.core, owner, "th_dunshi_damage_prevented", 1);
        if ctx.core.player(owner).max_hp > 1 {
            ctx.core.player_mut(owner).max_hp -= 1;
            let max_hp = ctx.core.player(owner).max_hp as i32;
            if ctx.core.player(owner).hp > max_hp {
                ctx.core.player_mut(owner).hp = max_hp;
            }
        }
        let virtue_count = next_mask.count_ones().min(5);
        set_flag(ctx.core, owner, "th_dunshi_virtue_unlocked", virtue_count);
        ctx.start_draw(owner, virtue_count.max(1));
    }
}

/// `L28` (`th_jingxie` 精械 — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:7429`):
/// In play phase, shows/upgrades an armor or crossbow card (`th_jingxie_upgraded = 1`);
/// or when a character is dying (`dying` or `th_jingxie_dying_save != 0`), recasts 1 card
/// and restores that character's HP to 1.
fn resolve_th_jingxie(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let dying_save = ctx.core.player(owner).dying || flag(ctx.core, owner, "th_jingxie_dying_save") != 0;
    if let Some(c) = choose_own_card(ctx, owner, true) {
        if ctx.core.player(owner).hand.contains(&c) {
            ctx.take_hand_card(owner, c);
        } else {
            ctx.take_owned_card(owner, c);
        }
        discard_some(ctx, owner, vec![c]);
    }
    if dying_save {
        ctx.core.player_mut(owner).dying = false;
        if ctx.core.player(owner).hp < 1 {
            ctx.core.player_mut(owner).hp = 1;
        }
        set_flag(ctx.core, owner, "th_jingxie_saved_to_1hp", 1);
        ctx.start_draw(owner, 1);
    } else {
        let count = flag(ctx.core, owner, "th_jingxie_upgraded") + 1;
        set_flag(ctx.core, owner, "th_jingxie_upgraded", count);
    }
    set_flag(ctx.core, owner, "th_jingxie_triggered", 1);
}

/// `L28` (`th_qiaosi` 巧思 — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:7030`):
/// Once per play phase, performs water-wheel trick to gain `1..=3` cards (`th_qiaosi_cards`),
/// then either discards that many cards or gives that many cards to another character.
fn resolve_th_qiaosi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let won = flag(ctx.core, owner, "th_qiaosi_won_count").clamp(1, 3);
    ctx.start_draw(owner, won);
    set_flag(ctx.core, owner, "th_qiaosi_drawn", won);
    set_flag(ctx.core, owner, "th_qiaosi_triggered", 1);
    if flag(ctx.core, owner, "th_qiaosi_give_other") != 0 {
        if let Some(target) = friendly_target(ctx, owner) {
            let mut given = 0u32;
            for _ in 0..won {
                if let Some(c) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(target).hand.push(c);
                    given += 1;
                }
            }
            set_flag(ctx.core, owner, "th_qiaosi_given", given);
        }
    } else {
        let mut to_discard = Vec::new();
        for _ in 0..won {
            if let Some(c) = choose_own_card(ctx, owner, false) {
                ctx.take_hand_card(owner, c);
                to_discard.push(c);
            }
        }
        if !to_discard.is_empty() {
            discard_some(ctx, owner, to_discard);
        }
    }
}

/// `L28` (`th_yeyan` 业炎 — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:8636`):
/// Limited 1/game skill (`th_yeyan_used`). Deals fire damage to 1–3 targets; when great
/// yeyan (`th_yeyan_great != 0`) is chosen, discards up to 4 cards, loses 3 HP, and deals
/// up to 3 fire damage (`DamageNature::Fire`).
fn resolve_th_yeyan(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if flag(ctx.core, owner, "th_yeyan_used") != 0 {
        return;
    }
    set_flag(ctx.core, owner, "th_yeyan_used", 1);
    set_flag(ctx.core, owner, "th_yeyan_triggered", 1);
    let Some(target) = hostile_target(ctx, owner) else {
        return;
    };
    let great = flag(ctx.core, owner, "th_yeyan_great") != 0;
    let dmg = if great {
        let mut discarded = Vec::new();
        for _ in 0..4 {
            if let Some(c) = choose_own_card(ctx, owner, false) {
                ctx.take_hand_card(owner, c);
                discarded.push(c);
            }
        }
        if !discarded.is_empty() {
            discard_some(ctx, owner, discarded);
        }
        ctx.emit(EventData::LoseHp {
            player: owner,
            amount: 3,
        });
        flag(ctx.core, owner, "th_yeyan_damage").clamp(2, 3) as i32
    } else {
        1
    };
    set_flag(ctx.core, owner, "th_yeyan_fire_damage", dmg as u32);
    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
        Some(owner),
        target,
        dmg,
        DamageNature::Fire,
        None,
    )));
}

/// `L28` (`th_taipingyaoshu` 太平要术 — `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:6974, 8968`):
/// Prevents elemental (Fire/Thunder) damage (`th_taipingyaoshu_prevent_elemental = 1`);
/// when lost from equip area (`th_taipingyaoshu_on_lose != 0`), draws 2 cards and if `hp > 1` loses 1 HP.
fn resolve_th_taipingyaoshu(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if flag(ctx.core, owner, "th_taipingyaoshu_on_lose") != 0 {
        set_flag(ctx.core, owner, "th_taipingyaoshu_lost_effect", 1);
        ctx.start_draw(owner, 2);
        if ctx.core.player(owner).hp > 1 {
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
        }
    } else {
        set_flag(ctx.core, owner, "th_taipingyaoshu_prevent_elemental", 1);
    }
}

/// `L28` (`th_*` remaining 55 authentic skills in `雷霆万钧/extension.js`):
fn resolve_th_extra_skill(ctx: &mut Ctx<'_>, skill: &str, owner: PlayerId) {
    match skill {
        // th_tian명 天命 (`extension.js:3689`): discard 2 cards (if any) then draw 2 cards
        "th_tianming" => {
            let mut disc = Vec::new();
            for _ in 0..2 {
                if let Some(c) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, c);
                    disc.push(c);
                }
            }
            if !disc.is_empty() {
                discard_some(ctx, owner, disc);
            }
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "th_skill:th_tianming", 1);
        }
        // th_qinyin 琴音 (`extension.js:8608`): all players lose 1 HP or recover 1 HP
        "th_qinyin" => {
            let recover_mode = flag(ctx.core, owner, "th_qinyin_recover") != 0;
            let seats: Vec<PlayerId> = ctx.core.seat_order_from(owner);
            for p in seats {
                if !ctx.core.player(p).alive {
                    continue;
                }
                if recover_mode {
                    ctx.recover(p, 1, None);
                } else {
                    ctx.emit(EventData::LoseHp {
                        player: p,
                        amount: 1,
                    });
                }
            }
            set_flag(ctx.core, owner, "th_skill:th_qinyin", 1);
        }
        // th_gushe 鼓舌 (`extension.js:8012`): pindian up to 3 targets, accumulate 7 raoSuo marks to die
        "th_gushe" => {
            let marks = flag(ctx.core, owner, "th_gushe_raosuo") + 1;
            set_flag(ctx.core, owner, "th_gushe_raosuo", marks);
            ctx.start_draw(owner, 1);
            if marks >= 7 {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: ctx.core.player(owner).hp.max(1) as u32,
                });
            }
            set_flag(ctx.core, owner, "th_skill:th_gushe", 1);
        }
        // th_jici 激词 (`extension.js:8146`): trigger when raoSuo mark increases
        "th_jici" => {
            set_flag(ctx.core, owner, "th_skill:th_jici", 1);
            ctx.start_draw(owner, 1);
        }
        // th_duanbi 锻币 (`extension.js:6115`): all players draw 3 and put 2 into 钱 pile
        "th_duanbi" => {
            ctx.start_draw(owner, 3);
            for _ in 0..2 {
                if let Some(c) = ctx.core.player_mut(owner).hand.pop() {
                    push_pile(ctx, owner, "th_qian", c);
                }
            }
            set_flag(ctx.core, owner, "th_skill:th_duanbi", 1);
        }
        other => {
            super::decision34_ext::resolve_l28_thunder_remaining(ctx, other, owner);
        }
    }
}
