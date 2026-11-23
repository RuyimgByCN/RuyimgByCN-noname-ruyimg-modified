pub mod asks_card;
pub mod asks_cde;
pub mod cards_and_foods;
pub mod chars_a;
pub mod chars_b;
pub mod chars_c;
pub mod chars_c2;
pub mod chars_d;
pub mod chars_e;
pub mod chars_e2;
pub mod helpers;
mod subplayer;

use self::helpers::*;
use super::super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::EventData;
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

pub(in crate::content::host) fn trigger_meta_u05_gujian(
    beh: HostBehavior,
    name: &str,
) -> Option<TriggerMeta> {
    match beh {
        HostBehavior::U05GujianCardBatch => cards_and_foods::trigger_meta_card_batch(name),
        HostBehavior::U05GujianCharBatchA => chars_a::trigger_meta_char_a(name),
        HostBehavior::U05GujianCharBatchB => chars_b::trigger_meta_char_b(name),
        HostBehavior::U05GujianCharBatchC => chars_c::trigger_meta_char_c(name),
        HostBehavior::U05GujianCharBatchD => chars_d::trigger_meta_char_d(name),
        HostBehavior::U05GujianCharBatchE => chars_e::trigger_meta_char_e1(name)
            .or_else(|| chars_e2::trigger_meta_char_e2(name)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u05_gujian(beh: HostBehavior, name: &str) -> SkillLimits {
    match beh {
        HostBehavior::U05GujianCardBatch => cards_and_foods::limits_card_batch(name),
        HostBehavior::U05GujianCharBatchA => chars_a::limits_char_a(name),
        HostBehavior::U05GujianCharBatchB => chars_b::limits_char_b(name),
        HostBehavior::U05GujianCharBatchC => chars_c::limits_char_c(name),
        HostBehavior::U05GujianCharBatchD => chars_d::limits_char_d(name),
        HostBehavior::U05GujianCharBatchE => {
            if matches!(name, "lingyan" | "xuanci") {
                chars_e::limits_char_e1(name)
            } else {
                chars_e2::limits_char_e2(name)
            }
        }
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn is_phase_use_active_u05_gujian(
    beh: HostBehavior,
    name: &str,
) -> bool {
    let clean = name.rsplit_once("__ref_").map(|(_, s)| s).unwrap_or(name);
    match beh {
        HostBehavior::U05GujianCardBatch => {
            matches!(name, "gjyuheng_plus_skill" | "gjyuheng_pro_skill")
        }
        HostBehavior::U05GujianCharBatchB => matches!(name, "zuiji" | "yanjia"),
        HostBehavior::U05GujianCharBatchC => {
            matches!(clean, "xuanning" | "xuanning1" | "yangming")
        }
        HostBehavior::U05GujianCharBatchD => name == "jiehuo",
        HostBehavior::U05GujianCharBatchE => matches!(name, "lingyan" | "xuanci" | "yunyou"),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u05_gujian(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    match skill.behavior {
        HostBehavior::U05GujianCardBatch => cards_and_foods::activate_card_batch(skill, ctx, owner),
        HostBehavior::U05GujianCharBatchB => chars_b::activate_char_b(skill, ctx, owner),
        HostBehavior::U05GujianCharBatchC => chars_c::activate_char_c(skill, ctx, owner),
        HostBehavior::U05GujianCharBatchD => chars_d::activate_char_d(skill, ctx, owner),
        HostBehavior::U05GujianCharBatchE => chars_e::activate_char_e1(skill, ctx, owner)
            .or_else(|| chars_e2::activate_char_e2(skill, ctx, owner)),
        _ => None,
    }
}

pub(in crate::content::host) fn filter_u05_gujian(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05GujianCardBatch => {
            cards_and_foods::filter_card_batch(skill, ctx, owner, event)
        }
        HostBehavior::U05GujianCharBatchA => chars_a::filter_char_a(skill, ctx, owner, event),
        HostBehavior::U05GujianCharBatchB => chars_b::filter_char_b(skill, ctx, owner, event),
        HostBehavior::U05GujianCharBatchC => chars_c::filter_char_c(skill, ctx, owner, event),
        HostBehavior::U05GujianCharBatchD => chars_d::filter_char_d(skill, ctx, owner, event),
        HostBehavior::U05GujianCharBatchE => {
            chars_e::filter_char_e1(skill, ctx, owner, event)
                || chars_e2::filter_char_e2(skill, ctx, owner, event)
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_u05_gujian(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    match skill.behavior {
        HostBehavior::U05GujianCardBatch => {
            cards_and_foods::machine_card_batch(skill, owner, event)
        }
        HostBehavior::U05GujianCharBatchA => chars_a::machine_char_a(skill, owner, event),
        HostBehavior::U05GujianCharBatchB => chars_b::machine_char_b(skill, owner, event),
        HostBehavior::U05GujianCharBatchC => chars_c::machine_char_c(skill, owner, event),
        HostBehavior::U05GujianCharBatchD => chars_d::machine_char_d(skill, owner, event),
        HostBehavior::U05GujianCharBatchE => HostMachine {
            def_key: skill.name.clone(),
            behavior: HostBehavior::U05GujianCharBatchE,
            owner,
            target: None,
            card: match event {
                EventData::CardUsed(u) => u.card,
                EventData::CardResponded { card, .. } => Some(*card),
                EventData::Damage(d) => Some(d.amount.max(0) as u32),
                _ => None,
            },
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        },
        _ => HostMachine {
            def_key: skill.name.clone(),
            behavior: skill.behavior,
            owner,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        },
    }
}

pub(in crate::content::host) fn step_u05_gujian(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    match machine.behavior {
        HostBehavior::U05GujianCardBatch => cards_and_foods::step_card_batch(machine, ctx),
        HostBehavior::U05GujianCharBatchA => chars_a::step_char_a(machine, ctx),
        HostBehavior::U05GujianCharBatchB => chars_b::step_char_b(machine, ctx),
        HostBehavior::U05GujianCharBatchC => chars_c::step_char_c(machine, ctx),
        HostBehavior::U05GujianCharBatchD => chars_d::step_char_d(machine, ctx),
        HostBehavior::U05GujianCharBatchE => {
            let flow = if matches!(
                machine.def_key.as_str(),
                "lingyan" | "xunjian" | "humeng" | "xianju" | "xuanci" | "woxue" | "yange"
                    | "lianjing" | "cihong"
            ) {
                chars_e::step_char_e1(machine, ctx)
            } else {
                chars_e2::step_char_e2(machine, ctx)
            };
            flow
        }
        _ => Flow::Done,
    }
}

pub(in crate::content::host) fn ask_u05_gujian(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<crate::snap::ExpectedAsk> {
    match behavior {
        HostBehavior::U05GujianCardBatch => asks_card::ask_card_batch(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GujianCharBatchA => chars_a::ask_char_a(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GujianCharBatchB => chars_b::ask_char_b(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GujianCharBatchC => asks_cde::ask_char_c(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GujianCharBatchD => chars_d::ask_char_d(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U05GujianCharBatchE => asks_cde::ask_char_e(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        _ => None,
    }
}

pub(crate) fn mod_food_key(kind: crate::catalog::CardKind) -> Option<&'static str> {
    helpers::food_listener_key(kind)
}

pub(crate) fn on_game_start_gujian(core: &mut GameStateCore, catalog: &Catalog) {
    let seats = core.players.len() as PlayerId;
    for owner in 0..seats {
        if !core.player(owner).alive {
            continue;
        }
        if has_skill(core, catalog, owner, "yange") && flag(core, owner, "gj_yange_inited") == 0 {
            let enemies = core
                .alive_players()
                .filter(|&p| p != owner && !core.is_friend_of(catalog, owner, p))
                .count() as u32;
            set_flag(core, owner, "gj_yange_inited", 1);
            set_flag(core, owner, "gj_yange_marks", enemies);
        }
        if has_skill(core, catalog, owner, "zhaolu") {
            chars_d::ensure_zhaolu_init(core, owner);
        }
        if has_skill(core, catalog, owner, "yuling") || has_skill(core, catalog, owner, "yuling2") {
            if flag(core, owner, "gj_yuling_game_draw_done") == 0 {
                set_flag(core, owner, "gj_yuling_game_draw_done", 1);
                while core.player(owner).hand.len() < 5 {
                    if let Some(c) = core.deck.pop().or_else(|| core.discard.pop()) {
                        core.player_mut(owner).hand.push(c);
                    } else {
                        break;
                    }
                }
            }
        }
        sync_gujian_owner_flags(core, catalog, owner);
    }
}

pub(crate) fn on_round_start_gujian(ctx: &mut Ctx<'_>) {
    let seats = ctx.core.players.len() as PlayerId;
    let odd = ctx.core.turn.round % 2 == 1;
    for owner in 0..seats {
        if !ctx.core.player(owner).alive {
            continue;
        }
        if has_skill(ctx.core, ctx.catalog, owner, "xianju") {
            set_flag(ctx.core, owner, "u05_mod:gj_qianxing", if odd { 1 } else { 0 });
        }
    }
}

pub(super) fn sync_gujian_owner_flags(core: &mut GameStateCore, catalog: &Catalog, owner: PlayerId) {
    set_flag(
        core,
        owner,
        "u05_mod:gjqtwuxie",
        u32::from(has_skill(core, catalog, owner, "gjqtwuxie")),
    );
    set_flag(
        core,
        owner,
        "u05_mod:gj_xuanci",
        u32::from(has_skill(core, catalog, owner, "xuanci")),
    );
    set_flag(
        core,
        owner,
        "u05_mod:gj_qianxing",
        u32::from(has_skill(core, catalog, owner, "xianju") && core.turn.round % 2 == 1),
    );
    set_flag(
        core,
        owner,
        "u05_mod:gj_xlqianhuan",
        u32::from(has_skill(core, catalog, owner, "xlqianhuan")),
    );
    set_flag(
        core,
        owner,
        "u05_mod:yuling2",
        u32::from(has_skill(core, catalog, owner, "yuling") || has_skill(core, catalog, owner, "yuling2")),
    );
    set_flag(
        core,
        owner,
        "u05_mod:gj_yuling4",
        u32::from(has_skill(core, catalog, owner, "yuling") || has_skill(core, catalog, owner, "yuling4")),
    );
}

pub(crate) fn on_death_before_gujian(ctx: &mut Ctx<'_>, dying: PlayerId) -> bool {
    subplayer::restore_yange_subplayer(ctx, dying)
        || subplayer::restore_humeng_subplayer(ctx, dying)
}

pub(crate) fn on_turn_phase_after_gujian(ctx: &mut Ctx<'_>, player: PlayerId) {
    subplayer::restore_yange_subplayer(ctx, player);
}

pub(crate) fn on_card_use_after_gujian(ctx: &mut Ctx<'_>, u: &crate::event::CardUseData) {
    let owner = u.player;
    if let Some(cid) = u.card {
        let zkey = format!("gj_zhenying_card:{cid}");
        let is_zhenying = flag(ctx.core, owner, &zkey) > 0;
        if is_zhenying {
            set_flag(ctx.core, owner, &zkey, 0);
            if !u.cancelled
                && u.use_as.is_none()
                && has_skill(ctx.core, ctx.catalog, owner, "zhenying")
                && !hand_and_equips(ctx.core, owner).is_empty()
            {
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Intrinsic {
                        frame: crate::resolution::NO_FRAME,
                    },
                    awaiting_answer: false,
                    machine: Box::new(HostMachine {
                        def_key: "zhenying".to_string(),
                        behavior: HostBehavior::U05GujianCharBatchE,
                        owner,
                        target: None,
                        card: Some(u32::MAX),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }),
                });
            }
        }
    }
    if u.cancelled {
        return;
    }
    if u.targets.iter().any(|&t| t != owner)
        && (0..ctx.core.players.len() as PlayerId)
            .any(|p| p != owner && ctx.core.player(p).alive && has_skill(ctx.core, ctx.catalog, p, "meiying"))
    {
        set_flag(ctx.core, owner, "gj_turn_targeted_other", 1);
    }
    if owner == ctx.core.turn.current {
        if let Some(cid) = u.card {
            if has_skill(ctx.core, ctx.catalog, owner, "yunyin") {
                let bit = 1u32 << (ctx.catalog.resolved_card_suit(ctx.core, cid) as u32);
                let prev = flag(ctx.core, owner, "gj_yunyin_suit_mask");
                set_flag(ctx.core, owner, "gj_yunyin_suit_mask", prev | bit);
            }
        }
    }
    let eff_kind = u.effective_kind_in(ctx.core, ctx.catalog);
    if eff_kind.is_sha_like() && has_skill(ctx.core, ctx.catalog, owner, "liuying") {
        for &t in &u.targets {
            set_flag(ctx.core, owner, &format!("gj_liuying_hit:{t}"), 1);
        }
    }
    if let Some(idx) = land_kind_to_idx(eff_kind) {
        if has_skill(ctx.core, ctx.catalog, owner, "qingshu")
            && flag(ctx.core, owner, &format!("gj_qingshu_all:{idx}")) == 0
        {
            set_flag(ctx.core, owner, &format!("gj_qingshu_all:{idx}"), 1);
            set_flag(ctx.core, owner, &format!("gj_qingshu_pool:{idx}"), 1);
        }
        if has_skill(ctx.core, ctx.catalog, owner, "yunyou") {
            set_flag(ctx.core, owner, &format!("gj_yunyou_used:{idx}"), 1);
        }
    }
    if let Some(cid) = u.card {
        let cname = ctx.catalog.cards[cid as usize].name;
        if let Some(suit_str) = cname.strip_prefix("yanjiadan_") {
            if has_skill(ctx.core, ctx.catalog, owner, "humeng") {
                let bit = match suit_str {
                    "spade" => 1u32,
                    "heart" => 2,
                    "club" => 4,
                    "diamond" => 8,
                    _ => 0,
                };
                if bit > 0 {
                    let mask = flag(ctx.core, owner, "gj_humeng_mask") | bit;
                    set_flag(ctx.core, owner, "gj_humeng_mask", mask);
                    if mask == 0xF {
                        set_flag(ctx.core, owner, "gj_humeng_sub", 1);
                    }
                }
            }
        }
    }
    if eff_kind == CardKind::Feibiao
        && ctx.core.player(owner).alive
        && has_skill(ctx.core, ctx.catalog, owner, "xuanci")
        && !ctx.catalog.is_tieji_sealed(ctx.core, owner)
    {
        for &t in &u.targets {
            if t != owner
                && ctx.core.player(t).alive
                && flag(ctx.core, t, "u05_mod:gj_qianxing") == 0
            {
                spawn_virtual_sha(ctx, owner, vec![t], crate::event::DamageNature::Normal);
            }
        }
    }
    // Note: yange end-of-turn cleanup & food passive decay handled on phaseEnd
}

pub(crate) fn on_phase_end_gujian(ctx: &mut Ctx<'_>) {
    let player = ctx.core.turn.current;
    let skip_xiajiao = flag(ctx.core, player, "gj_xiajiao3") > 0;
    // 被动食物回合倒计时（仅在 Phase::End 收束时扣减一次）
    for key in [
        GJ_FOOD_CHUNBING_KEY,
        GJ_FOOD_YUANBAOROU_KEY,
        GJ_FOOD_MOLICHA_KEY,
        GJ_FOOD_MIZHLIANOU_KEY,
        GJ_FOOD_QINGTUAN_KEY,
        GJ_FOOD_LUYUGENG_KEY,
        GJ_FOOD_XIAJIAO_KEY,
        "chunbing",
        "yuanbaorou",
        "molicha",
        "mizhilianou",
        "qingtuan",
        "luyugeng",
        "xiajiao",
    ] {
        if skip_xiajiao && matches!(key, "xiajiao" | GJ_FOOD_XIAJIAO_KEY) {
            continue;
        }
        food_tick(ctx, player, key);
    }
    set_flag(ctx.core, player, "gj_xuanning1_used_turn", 0);
    set_flag(ctx.core, player, "gj_yunyin_suit_mask", 0);
    set_flag(ctx.core, player, "gj_mingkong2_pending", 0);
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        set_flag(ctx.core, p, "gj_turn_targeted_other", 0);
        set_flag(ctx.core, p, "gj_dangping_used_turn", 0);
        set_flag(ctx.core, p, "gj_xiajiao3", 0);
        if flag(ctx.core, p, "gj_buff_qianxing_temp") > 0 && p == player {
            set_flag(ctx.core, p, "gj_buff_qianxing_temp", 0);
            set_flag(ctx.core, p, "u05_mod:gj_qianxing", 0);
        }
    }
}

pub(crate) fn on_cards_to_discard_gujian(
    ctx: &mut Ctx<'_>,
    actor: PlayerId,
    cards: &[CardId],
) {
    let seats = ctx.core.players.len() as PlayerId;
    for &c in cards {
        subplayer::clean_yange_clone_flags(ctx.core, c);
        crate::kernel::release_yanjia_merged_to_discard(ctx.core, c);
        for p in 0..seats {
            set_flag(ctx.core, p, &format!("gj_zhenying_card:{c}"), 0);
        }
    }
    let equips: Vec<CardId> = cards
        .iter()
        .copied()
        .filter(|&c| {
            ctx.catalog.card_kind(c).equip_slot().is_some() && ctx.core.discard.contains(&c)
        })
        .collect();
    if equips.is_empty() {
        return;
    }
    let seats = ctx.core.players.len() as PlayerId;
    for owner in 0..seats {
        if owner == actor
            || !ctx.core.player(owner).alive
            || !has_skill(ctx.core, ctx.catalog, owner, "xiuhua")
        {
            continue;
        }
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Intrinsic {
                frame: crate::resolution::NO_FRAME,
            },
            awaiting_answer: false,
            machine: Box::new(HostMachine {
                def_key: "xiuhua".to_string(),
                behavior: HostBehavior::U05GujianCharBatchB,
                owner,
                target: Some(actor),
                card: None,
                cards: equips.clone(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }),
        });
    }
}
