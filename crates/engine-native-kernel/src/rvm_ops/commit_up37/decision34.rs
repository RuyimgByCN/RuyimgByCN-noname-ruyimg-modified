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

pub const HUANLE_KUWU_INTERACTIVE_ID: u32 = 1_920_003_001;

pub const DIAOBINGQIANJIANG_ID: u32 = 1_920_004_001;
pub const DUOQI_ID: u32 = 1_920_004_002;
pub const YOUZHAN_ID: u32 = 1_920_004_003;

pub const YXS_MENSHEN2_GUARD_ID: u32 = 1_920_005_001;
pub const YXS_TAFANG_WAVE_ID: u32 = 1_920_005_002;

pub const L20_MARK_DISPATCH_ID: u32 = 1_920_006_001;

pub const TH_CHONGZOU_ID: u32 = 1_920_007_001;
pub const TH_HUANYIN_ID: u32 = 1_920_007_002;
pub const TH_XUANLV_ID: u32 = 1_920_007_003;

pub const BOSS_BAONU_STAGE_ID: u32 = 1_920_008_001;
pub const TAIXU_EXPEDITION_STEP_ID: u32 = 1_920_009_001;
pub const DOOMSDAY_INFECTION_STEP_ID: u32 = 1_920_010_001;
pub const MARTIAL_FORMATION_STEP_ID: u32 = 1_920_011_001;

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
        HUANLE_KUWU_INTERACTIVE_ID => resolve_kuwu_interactive(ctx, owner),
        DIAOBINGQIANJIANG_ID => resolve_diaobingqianjiang(ctx, owner),
        DUOQI_ID => resolve_duoqi(ctx, owner),
        YOUZHAN_ID => resolve_youzhan(ctx, owner),
        YXS_MENSHEN2_GUARD_ID => resolve_yxs_menshen2_guard(ctx, owner),
        YXS_TAFANG_WAVE_ID => resolve_yxs_tafang_wave(ctx, owner),
        L20_MARK_DISPATCH_ID => resolve_l20_mark_dispatch(ctx, owner),
        TH_CHONGZOU_ID => resolve_th_chongzou(ctx, owner),
        TH_HUANYIN_ID => resolve_th_huanyin(ctx, owner),
        TH_XUANLV_ID => resolve_th_xuanlv(ctx, owner),
        BOSS_BAONU_STAGE_ID => resolve_boss_baonu_stage(ctx, owner),
        TAIXU_EXPEDITION_STEP_ID => resolve_taixu_expedition_step(ctx, owner),
        DOOMSDAY_INFECTION_STEP_ID => resolve_doomsday_infection_step(ctx, owner),
        MARTIAL_FORMATION_STEP_ID => resolve_martial_formation_step(ctx, owner),
        _ => match executing {
            "mbcmjiushi" => resolve_mbcmjiushi(ctx, owner),
            "mbcmfangzhu" => resolve_mbcmfangzhu(ctx, owner),
            "mbjuejin" => resolve_mbjuejin(ctx, owner),
            "k_danggu" => resolve_k_danggu(ctx, owner),
            "k_mowang" => resolve_k_mowang(ctx, owner),
            "huanle_kuwu_interactive" => resolve_kuwu_interactive(ctx, owner),
            "diaobingqianjiang" => resolve_diaobingqianjiang(ctx, owner),
            "duoqi" => resolve_duoqi(ctx, owner),
            "youzhan" => resolve_youzhan(ctx, owner),
            "yxs_menshen2_guard" => resolve_yxs_menshen2_guard(ctx, owner),
            "yxs_tafang_wave" => resolve_yxs_tafang_wave(ctx, owner),
            "l20_mark_dispatch" => resolve_l20_mark_dispatch(ctx, owner),
            "th_chongzou" => resolve_th_chongzou(ctx, owner),
            "th_huanyin" => resolve_th_huanyin(ctx, owner),
            "th_xuanlv" => resolve_th_xuanlv(ctx, owner),
            "boss_baonu_stage" => resolve_boss_baonu_stage(ctx, owner),
            "taixu_expedition_step" => resolve_taixu_expedition_step(ctx, owner),
            "doomsday_infection_step" => resolve_doomsday_infection_step(ctx, owner),
            "martial_formation_step" => resolve_martial_formation_step(ctx, owner),
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
/// On damage, chooses another alive player (`mbcmfangzhu_target` or `first_other_alive`),
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
            let Some(t) = first_other_alive(ctx.core, owner) else {
                return;
            };
            t
        }
    };
    let lost_hp = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
    let next_face_down = !ctx.core.player(target).face_down;
    ctx.core.player_mut(target).face_down = next_face_down;
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
        ctx.core.player_mut(owner).dying = false;
        if ctx.core.player(owner).hp <= 0 {
            ctx.core.player_mut(owner).hp = 1;
        }
        ctx.start_draw(owner, 1);
    } else {
        set_flag(ctx.core, owner, "k_mowang_exhausted", 1);
    }
}

/// `U06` (`huanle_kuwu_interactive` 苦无交互式分支):
/// Guards against chained damage (`kuwu_from_chain == 0`), then lets target either discard 1
/// `he` card (hand or equip) OR lose 1 HP when target chooses lose-HP or has no `he` cards.
fn resolve_kuwu_interactive(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if flag(ctx.core, owner, "kuwu_from_chain") != 0 {
        set_flag(ctx.core, owner, "kuwu_skipped_from_chain", 1);
        return;
    }
    let target = {
        let raw = flag(ctx.core, owner, "kuwu_target");
        if raw > 0 && (raw as u8) < (ctx.core.players.len() as u8) {
            raw as u8
        } else {
            let Some(t) = first_other_alive(ctx.core, owner) else {
                return;
            };
            t
        }
    };
    let choose_lose_hp = flag(ctx.core, target, "kuwu_choose_lose_hp") != 0;
    let prefer_equip = flag(ctx.core, target, "kuwu_prefer_equip") != 0;
    let chosen_card = if choose_lose_hp {
        None
    } else if prefer_equip {
        ctx.core
            .player(target)
            .equips
            .first()
            .copied()
            .or_else(|| ctx.core.player(target).hand.first().copied())
    } else {
        ctx.core
            .player(target)
            .hand
            .first()
            .copied()
            .or_else(|| ctx.core.player(target).equips.first().copied())
    };
    if let Some(c) = chosen_card {
        if ctx.core.player(target).hand.contains(&c) {
            ctx.take_hand_card(target, c);
        } else {
            ctx.take_owned_card(target, c);
        }
        discard_some(ctx, target, vec![c]);
        set_flag(ctx.core, target, "kuwu_discarded_he", c.max(1));
    } else {
        set_flag(ctx.core, target, "kuwu_lost_hp", 1);
        ctx.emit(EventData::LoseHp {
            player: target,
            amount: 1,
        });
    }
}

/// `U07` (`diaobingqianjiang` 调兵遣将, `duoqi` 夺气, `youzhan` 诱战):
fn resolve_diaobingqianjiang(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let reveal_count = (ctx.core.alive_players().count() / 2).max(2);
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
        if let Some(hand_c) = ctx.core.player(p).hand.first().copied() {
            ctx.take_hand_card(p, hand_c);
            let m_card = market.remove(0);
            ctx.core.player_mut(p).hand.push(m_card);
            market.push(hand_c);
            swapped += 1;
        }
    }
    if let Some(rem) = market.pop() {
        ctx.core.player_mut(owner).hand.push(rem);
    }
    for c in market {
        ctx.core.discard.push(c);
    }
    set_flag(ctx.core, owner, "diaobingqianjiang_swapped", swapped);
    set_flag(ctx.core, owner, "diaobingqianjiang_resolved", 1);
}

fn resolve_duoqi(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let Some(c) = ctx
        .core
        .player(owner)
        .hand
        .first()
        .copied()
        .or_else(|| ctx.core.player(owner).equips.first().copied())
    else {
        return;
    };
    if ctx.core.player(owner).hand.contains(&c) {
        ctx.take_hand_card(owner, c);
    } else {
        ctx.take_owned_card(owner, c);
    }
    discard_some(ctx, owner, vec![c]);
    let target = first_other_alive(ctx.core, owner).unwrap_or(owner);
    set_flag(ctx.core, target, "duoqi_draw_penalty", 1);
    set_flag(ctx.core, owner, "duoqi_triggered", 1);
    ctx.start_draw(owner, 1);
}

fn resolve_youzhan(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let target = first_other_alive(ctx.core, owner).unwrap_or(owner);
    set_flag(ctx.core, owner, "youzhan_triggered", 1);
    if target != owner {
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            target,
            1,
        )));
    }
    ctx.start_draw(owner, 1);
}

/// `U08` (`yxs_menshen2_guard` 门神承受伤害转移 + `yxs_tafang_wave` 塔防波次推进):
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

fn resolve_yxs_tafang_wave(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let wave = (flag(ctx.core, owner, "tafang_wave") + 1).min(5);
    let fort_hp = flag(ctx.core, owner, "tafang_fort_hp").max(10);
    set_flag(ctx.core, owner, "tafang_wave", wave);
    set_flag(ctx.core, owner, "tafang_fort_hp", fort_hp);
    set_flag(
        ctx.core,
        owner,
        "tafang_Cleared",
        u32::from(wave >= 5),
    );
    ctx.start_draw(owner, 1);
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

/// `L28` (`th_chongzou` 重奏, `th_huanyin` 幻音, `th_xuanlv` 旋律):
fn resolve_th_chongzou(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mask = flag(ctx.core, owner, "th_melody_mask").max(1);
    set_flag(ctx.core, owner, "th_chongzou_replayed_mask", mask);
    set_flag(ctx.core, owner, "th_chongzou_triggered", 1);
    ctx.start_draw(owner, 1);
}

fn resolve_th_huanyin(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mask = flag(ctx.core, owner, "th_melody_mask");
    set_flag(ctx.core, owner, "th_huanyin_triggered", 1);
    if mask.count_ones() >= 2 && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
        ctx.recover(owner, 1, None);
    } else {
        ctx.start_draw(owner, 1);
    }
}

fn resolve_th_xuanlv(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let suit_bit = flag(ctx.core, owner, "th_xuanlv_suit_bit").max(1);
    let next_mask = (flag(ctx.core, owner, "th_melody_mask") | suit_bit) & 0xF;
    set_flag(ctx.core, owner, "th_melody_mask", next_mask);
    set_flag(ctx.core, owner, "th_xuanlv_triggered", 1);
    if next_mask == 0xF {
        set_flag(ctx.core, owner, "th_xuanlv_chord_complete", 1);
        set_flag(ctx.core, owner, "th_melody_mask", 0);
        if let Some(t) = first_other_alive(ctx.core, owner) {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(owner),
                t,
                1,
                DamageNature::Thunder,
                None,
            )));
        }
    }
}

/// `U02 / L03` (`boss_baonu_stage` Boss 暴怒二阶段变身与剑阁攻城器械/守护兽联动):
fn resolve_boss_baonu_stage(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let rage = flag(ctx.core, owner, "boss_rage") + 2;
    set_flag(ctx.core, owner, "boss_rage", rage);
    if rage >= 4 || ctx.core.player(owner).hp <= 4 {
        set_flag(ctx.core, owner, "boss_stage", 2);
        set_flag(ctx.core, owner, "jiange_siege_linked", 1);
        ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.max(6);
        ctx.core.player_mut(owner).hp = ctx.core.player(owner).hp.max(6);
        ctx.start_draw(owner, 2);
    }
}

/// `L11` (`taixu_expedition_step` 太虚幻境奇遇/赐福/灵宠联动):
fn resolve_taixu_expedition_step(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let floor = flag(ctx.core, owner, "taixu_floor") + 1;
    let blessing = flag(ctx.core, owner, "taixu_blessing_id").max(1);
    set_flag(ctx.core, owner, "taixu_floor", floor);
    set_flag(ctx.core, owner, "taixu_blessing_id", blessing);
    set_flag(ctx.core, owner, "taixu_spirit_damage", 1);
    if let Some(t) = first_other_alive(ctx.core, owner) {
        ctx.emit(EventData::Damage(crate::machine::damage_data(
            Some(owner),
            t,
            1,
        )));
    }
}

/// `L19` (`doomsday_infection_step` 末日乱斗感染/变异/血清结算):
fn resolve_doomsday_infection_step(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let serum = flag(ctx.core, owner, "doomsday_serum");
    if serum > 0 {
        set_flag(ctx.core, owner, "doomsday_serum", serum - 1);
        set_flag(ctx.core, owner, "doomsday_infection", 0);
        set_flag(ctx.core, owner, "doomsday_cured", 1);
        ctx.start_draw(owner, 1);
        return;
    }
    let inf = flag(ctx.core, owner, "doomsday_infection") + 1;
    set_flag(ctx.core, owner, "doomsday_infection", inf);
    if inf >= 3 {
        set_flag(ctx.core, owner, "doomsday_mutated", 1);
        ctx.core.player_mut(owner).max_hp += 1;
        ctx.core.player_mut(owner).hp += 1;
    }
}

/// `L21` (`martial_formation_step` 武皇争霸 5v5 阵型与替补联动):
fn resolve_martial_formation_step(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let subs = flag(ctx.core, owner, "martial_bench_sub_count") + 1;
    set_flag(ctx.core, owner, "martial_bench_sub_count", subs);
    set_flag(ctx.core, owner, "martial_linked_bonus", 1);
    ctx.start_draw(owner, 1);
}
