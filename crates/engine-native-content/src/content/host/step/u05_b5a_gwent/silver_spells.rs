//! `card/gwent/card.js` 12 张银卡法术结算（不含 `gw_fuyuan`，`gw_fuyuan` 走 `DyingSaveMachine`）。

use super::super::super::*;
use super::{
    clean_gwent_name, discard_he_card, finish_gwent_card, pull_card_matching, BRONZE_SPELL_NAMES,
};
use crate::catalog::Catalog;
use crate::state::GameStateCore;

pub(super) fn step_silver_spell(m: &mut HostMachine, ctx: &mut Ctx<'_>, cname: &str) -> Flow {
    match cname {
        "gw_youer" => step_youer(m, ctx),
        "gw_tongdi" => step_tongdi(m, ctx),
        "gw_zhuoshao" => step_zhuoshao(m, ctx),
        "gw_butianshu" => step_butianshu(m, ctx),
        "gw_shizizhaohuan" => step_shizizhaohuan(m, ctx),
        "gw_zuihouyuanwang" => step_zuihouyuanwang(m, ctx),
        "gw_zirankuizeng" => step_zirankuizeng(m, ctx),
        "gw_nuhaifengbao" => step_nuhaifengbao(m, ctx),
        "gw_baishuang" => step_baishuang(m, ctx),
        "gw_baobaoshu" => step_baobaoshu(m, ctx),
        "gw_guaiwuchaoxue" => step_guaiwuchaoxue(m, ctx),
        _ => finish_gwent_card(m, ctx, false),
    }
}

/// 诱饵 (`card/gwent/card.js:930`): 移出目标全部手牌至 `gw_youer_pile` 并赋 `gw_youer`，结算后再摸 1。
fn step_youer(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            if let Some(target) = m.target {
                if ctx.core.player(target).alive {
                    let cards = std::mem::take(&mut ctx.core.player_mut(target).hand);
                    if !cards.is_empty() {
                        let lost_n = cards.len();
                        ctx.core
                            .vm_aux
                            .piles
                            .entry((target, "gw_youer_pile".to_string()))
                            .or_default()
                            .extend(cards.iter().copied());
                        ctx.note_cards_lost(target, &cards);
                        ctx.emit(EventData::CardsLost {
                            player: target,
                            cards,
                            origins: vec![crate::event::LostOrigin::Hand; lost_n],
                        });
                    }
                    if let Some(def) = ctx.catalog.skill_by_name("gw_youer") {
                        ctx.insert_skill(
                            (target, def),
                            crate::state::SkillInstance {
                                owner: target,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                }
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(m.owner).alive {
                ctx.start_draw(m.owner, 1);
            }
            finish_gwent_card(m, ctx, false)
        }
        _ => finish_gwent_card(m, ctx, false),
    }
}

/// 通敌 (`card/gwent/card.js:968`): 观看并获得目标一张手牌，然后目标获得一张【杀】（普通杀优先）。
fn step_tongdi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Some(t) = m.target else {
                return finish_gwent_card(m, ctx, false);
            };
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                return finish_gwent_card(m, ctx, false);
            }
            let opts: Vec<ChoiceOption> = ctx
                .core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gw_tongdi_pick",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            if let (Some(t), Some(ChoiceOption::Card(picked))) = (m.target, ctx.answer().first()) {
                let picked = *picked;
                if ctx.core.player(t).hand.contains(&picked) {
                    ctx.take_hand_card(t, picked);
                    ctx.note_cards_lost(t, &[picked]);
                    ctx.core.player_mut(m.owner).hand.push(picked);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![picked],
                        from: t,
                        to: m.owner,
                    });
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
            }
            m.stage = HostStage::DrawSettle;
            step_tongdi(m, ctx)
        }
        HostStage::DrawSettle => {
            if let Some(t) = m.target {
                if ctx.core.player(t).alive {
                    if let Some(sha) =
                        pull_card_matching(ctx, |cid| ctx.catalog.card_kind(cid) == CardKind::Sha)
                            .or_else(|| {
                                pull_card_matching(ctx, |cid| {
                                    ctx.catalog.card_kind(cid).is_sha_like()
                                })
                            })
                    {
                        ctx.core.player_mut(t).hand.push(sha);
                        ctx.emit(EventData::CardsDrawn {
                            player: t,
                            cards: vec![sha],
                        });
                    }
                }
            }
            finish_gwent_card(m, ctx, false)
        }
        _ => finish_gwent_card(m, ctx, false),
    }
}

/// 灼烧 (`card/gwent/card.js:1027`): 对指定的 1 名或多名体力最高角色造成 1 点火焰伤害。
fn step_zhuoshao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let ordered: Vec<PlayerId> = ctx
        .core
        .seat_order_from(m.owner)
        .into_iter()
        .filter(|p| m.targets.contains(p) && ctx.core.player(*p).alive)
        .collect();
    for t in ordered.into_iter().rev() {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(m.owner),
            t,
            1,
            crate::event::DamageNature::Fire,
            m.card,
        )));
    }
    finish_gwent_card(m, ctx, false)
}

/// 卜天术 (`card/gwent/card.js:1057`): 为目标判定区添加任意一种尚未持有的延时锦囊。
fn step_butianshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let Some(t) = m.target else {
                return finish_gwent_card(m, ctx, false);
            };
            let opts = eligible_delay_options(ctx.core, ctx.catalog, t);
            if opts.is_empty() {
                return finish_gwent_card(m, ctx, false);
            }
            m.stage = HostStage::SelectNature;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gw_butianshu_delay",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectNature => {
            if let (Some(t), ChoiceOption::Tag(tag)) = (m.target, ctx.answer_first()) {
                if let Some(kind) = delay_kind_for_tag(tag) {
                    if let Some(c) = pull_card_matching(ctx, |id| ctx.catalog.card_kind(id) == kind) {
                        if ctx.core.player(t).alive {
                            ctx.core.player_mut(t).judge.push(c);
                        } else {
                            ctx.to_discard(c);
                        }
                    }
                }
            }
            finish_gwent_card(m, ctx, false)
        }
        _ => finish_gwent_card(m, ctx, false),
    }
}

pub(super) fn delay_kind_for_tag(tag: u8) -> Option<CardKind> {
    match tag {
        0 => Some(CardKind::Le),
        1 => Some(CardKind::Bing),
        2 => Some(CardKind::ShanDian),
        3 => Some(CardKind::Guiyoujie),
        _ => None,
    }
}

pub(super) fn eligible_delay_options(
    core: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
) -> Vec<ChoiceOption> {
    if !core.player(target).alive {
        return Vec::new();
    }
    let mut opts = Vec::new();
    for tag in 0..4u8 {
        let Some(kind) = delay_kind_for_tag(tag) else {
            continue;
        };
        let already = core
            .player(target)
            .judge
            .iter()
            .any(|c| catalog.card_kind(*c) == kind);
        let available = core
            .deck
            .iter()
            .chain(core.discard.iter())
            .any(|c| catalog.card_kind(*c) == kind);
        if !already && available {
            opts.push(ChoiceOption::Tag(tag));
        }
    }
    opts
}

/// 十字召唤 (`card/gwent/card.js:1137`): 仅从牌堆按类型等概率随机获得 1 张决斗/火攻/南蛮/火烧连营 + 1 张【杀】（无锦囊则 2 张杀）。
fn step_shizizhaohuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let deck_pick = |ctx: &mut Ctx<'_>, bucket: &[usize]| -> Option<CardId> {
        if bucket.is_empty() {
            None
        } else {
            let pick = bucket[ctx.core.rng.gen_range(bucket.len() as u32) as usize];
            Some(ctx.core.deck.remove(pick))
        }
    };
    let mut gained = Vec::new();
    let mut buckets: [Vec<usize>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for (i, &c) in ctx.core.deck.iter().enumerate() {
        let k = ctx.catalog.card_kind(c);
        let n = clean_gwent_name(ctx.catalog.cards[c as usize].name);
        if k == CardKind::Duel {
            buckets[0].push(i);
        } else if n == "huogong" {
            buckets[1].push(i);
        } else if k == CardKind::SavageAttack {
            buckets[2].push(i);
        } else if matches!(n, "huoshaolianying" | "txhj_huoshaolianying") {
            buckets[3].push(i);
        }
    }
    let non_empty: Vec<usize> = buckets
        .iter()
        .enumerate()
        .filter_map(|(bi, b)| (!b.is_empty()).then_some(bi))
        .collect();
    let mut found_attack = false;
    if !non_empty.is_empty() {
        let b_idx = non_empty[ctx.core.rng.gen_range(non_empty.len() as u32) as usize];
        if let Some(c) = deck_pick(ctx, &buckets[b_idx]) {
            gained.push(c);
            found_attack = true;
        }
    }
    let sha_need = if found_attack { 1 } else { 2 };
    for _ in 0..sha_need {
        let plain_idx: Vec<usize> = ctx
            .core
            .deck
            .iter()
            .enumerate()
            .filter(|(_, &c)| ctx.catalog.card_kind(c) == CardKind::Sha)
            .map(|(i, _)| i)
            .collect();
        let sha = if !plain_idx.is_empty() {
            deck_pick(ctx, &plain_idx)
        } else {
            let any_sha: Vec<usize> = ctx
                .core
                .deck
                .iter()
                .enumerate()
                .filter(|(_, &c)| ctx.catalog.card_kind(c).is_sha_like())
                .map(|(i, _)| i)
                .collect();
            deck_pick(ctx, &any_sha)
        };
        if let Some(s) = sha {
            gained.push(s);
        }
    }
    if !gained.is_empty() && ctx.core.player(m.owner).alive {
        ctx.core.player_mut(m.owner).hand.extend(gained.iter().copied());
        ctx.emit(EventData::CardsDrawn {
            player: m.owner,
            cards: gained,
        });
    }
    finish_gwent_card(m, ctx, false)
}

/// 最后愿望 (`card/gwent/card.js:1189`): 摸 X 张牌并强制弃置 X 张牌（X = 存活角色数）。
fn step_zuihouyuanwang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let n = ctx.core.alive_count() as u32;
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "u05_gw_zuihou_n".to_string()), n);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(m.owner, n);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let n = ctx
                .core
                .vm_aux
                .flags
                .get(&(m.owner, "u05_gw_zuihou_n".to_string()))
                .copied()
                .unwrap_or(0) as usize;
            let he: Vec<ChoiceOption> = ctx
                .core
                .player(m.owner)
                .hand
                .iter()
                .chain(ctx.core.player(m.owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let req = n.min(he.len());
            if req == 0 || !ctx.core.player(m.owner).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(m.owner, "u05_gw_zuihou_n".to_string()));
                return finish_gwent_card(m, ctx, false);
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gw_zuihouyuanwang_discard",
                    options: he,
                    min: req,
                    max: req,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            ctx.core
                .vm_aux
                .flags
                .remove(&(m.owner, "u05_gw_zuihou_n".to_string()));
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            for c in picks {
                discard_he_card(ctx, m.owner, c);
            }
            finish_gwent_card(m, ctx, false)
        }
        _ => finish_gwent_card(m, ctx, false),
    }
}

/// 自然馈赠 (`card/gwent/card.js:1228`): 选择任意一张铜卡法术并立即使用之。
fn step_zirankuizeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let opts = eligible_bronze_tags(ctx.core, ctx.catalog, m.owner);
            if opts.is_empty() {
                return finish_gwent_card(m, ctx, false);
            }
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "gw_zirankuizeng_spell",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            let ChoiceOption::Tag(tag) = ctx.answer_first() else {
                return finish_gwent_card(m, ctx, false);
            };
            let Some(&(_, kind)) = BRONZE_SPELL_NAMES.get(tag as usize) else {
                return finish_gwent_card(m, ctx, false);
            };
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "u05_gw_ziran_tag".to_string()), u32::from(tag));
            if matches!(kind, CardKind::Poxiao | CardKind::Wenyi | CardKind::Shanbengshu) {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(m.owner, "u05_gw_ziran_tag".to_string()));
                if let Some(c) = m.card {
                    ctx.core.table.retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                ctx.use_virtual_card(m.owner, kind, Vec::new());
                return Flow::Done;
            }
            let targets = eligible_bronze_targets(ctx.core, ctx.catalog, m.owner, kind);
            if targets.is_empty() {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(m.owner, "u05_gw_ziran_tag".to_string()));
                return finish_gwent_card(m, ctx, false);
            }
            m.stage = HostStage::TargetChoice;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gw_zirankuizeng_target",
                    options: targets,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::TargetChoice => {
            let tag = ctx
                .core
                .vm_aux
                .flags
                .remove(&(m.owner, "u05_gw_ziran_tag".to_string()))
                .unwrap_or(0);
            let target = ctx.answer().iter().find_map(|o| match *o {
                ChoiceOption::Player(p) => Some(p),
                _ => None,
            });
            let Some(&(_, kind)) = BRONZE_SPELL_NAMES.get(tag as usize) else {
                return finish_gwent_card(m, ctx, false);
            };
            if let Some(c) = m.card {
                ctx.core.table.retain(|x| *x != c);
                ctx.to_discard(c);
            }
            if let Some(t) = target {
                ctx.use_virtual_card(m.owner, kind, vec![t]);
            }
            Flow::Done
        }
        _ => finish_gwent_card(m, ctx, false),
    }
}

pub(in crate::content::host::step) fn eligible_bronze_tags(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    BRONZE_SPELL_NAMES
        .iter()
        .enumerate()
        .filter(|&(_, &(_, kind))| match kind {
            CardKind::Poxiao | CardKind::Wenyi => true,
            CardKind::Shanbengshu => !crate::content::host::step::u05_b5a_gwent::enemies_of(
                core, catalog, owner,
            )
            .into_iter()
            .all(|e| core.player(e).equips.is_empty()),
            _ => !eligible_bronze_targets(core, catalog, owner, kind).is_empty(),
        })
        .map(|(i, _)| ChoiceOption::Tag(i as u8))
        .collect()
}

pub(in crate::content::host::step) fn eligible_bronze_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
) -> Vec<ChoiceOption> {
    let owner_hp = core.player(owner).hp;
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&t| {
            let pl = core.player(t);
            if !pl.alive {
                return false;
            }
            match kind {
                CardKind::Qinpendayu => !super::has_named_skill(core, catalog, t, GW_QINPEN_STATE_KEY)
                    && !super::has_named_skill(core, catalog, t, "gw_qinpendayu"),
                CardKind::Birinongwu => !super::has_named_skill(core, catalog, t, GW_BIRIN_STATE_KEY)
                    && !super::has_named_skill(core, catalog, t, "gw_birinongwu"),
                CardKind::Ciguhanshuang => !super::has_named_skill(core, catalog, t, GW_CIGU_STATE_KEY)
                    && !super::has_named_skill(core, catalog, t, "gw_ciguhanshuang"),
                CardKind::Zhihuanjun => pl.hp < pl.max_hp as i32,
                CardKind::Zumoshoukao => pl.hujia > 0 || !super::has_named_skill(core, catalog, t, GW_FENGYIN_STATE_KEY),
                CardKind::Aozuzhilei => pl.hp >= owner_hp,
                CardKind::Kunenfayin => !super::has_named_skill(core, catalog, t, GW_KUNEN_STATE_KEY)
                    && !super::has_named_skill(core, catalog, t, "gw_kunenfayin"),
                _ => true,
            }
        })
        .map(ChoiceOption::Player)
        .collect()
}

/// 怒海风暴 (`card/gwent/card.js:1345`)：授予目标 `gw_nuhaifengbao`（2 回合）。
fn step_nuhaifengbao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let (Some(t), Some(def)) = (m.target, ctx.catalog.skill_by_name("gw_nuhaifengbao")) {
        if ctx.core.player(t).alive {
            ctx.insert_skill(
                (t, def),
                crate::state::SkillInstance {
                    owner: t,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 2,
                },
            );
            ctx.core
                .vm_aux
                .flags
                .insert((t, "gw_nuhaifengbao_rounds".to_string()), 2);
        }
    }
    finish_gwent_card(m, ctx, false)
}

/// 白霜 (`card/gwent/card.js:1367`): 为至多 3 名目标授予 `gw_cigu_state`。
pub(in crate::content::host) fn apply_gw_baishuang_to_targets(ctx: &mut Ctx<'_>, targets: &[PlayerId]) {
    let def = ctx
        .catalog
        .skill_by_name(GW_CIGU_STATE_KEY)
        .or_else(|| ctx.catalog.skill_by_name("gw_ciguhanshuang"));
    let alt_def = ctx.catalog.skill_by_name("gw_ciguhanshuang");
    if let Some(def) = def {
        for &t in targets {
            if ctx.core.player(t).alive
                && !ctx.core.skills.contains_key(&(t, def))
                && !alt_def.is_some_and(|ad| ctx.core.skills.contains_key(&(t, ad)))
            {
                ctx.insert_skill(
                    (t, def),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }
    }
}

fn step_baishuang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    apply_gw_baishuang_to_targets(ctx, &m.targets);
    finish_gwent_card(m, ctx, false)
}

/// 雹暴术 (`card/gwent/card.js:1388`): 为至多 2 名目标授予 `gw_baobaoshu` 直至其回合结束。
fn step_baobaoshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(def) = ctx.catalog.skill_by_name("gw_baobaoshu") {
        for &t in &m.targets {
            if ctx.core.player(t).alive {
                ctx.add_skill_until_turn_end(t, def);
            }
        }
    }
    finish_gwent_card(m, ctx, false)
}

/// 怪物巢穴 (`card/gwent/card.js:1411`): 获得随机卖血技至下回合开始，受随机敌方 1 点伤害并回 1 体力。
fn step_guaiwuchaoxue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            ctx.core
                .vm_aux
                .flags
                .insert((m.owner, "gw_guaiwuchaoxue_used_play".to_string()), 1);
            let maixie_names = [
                crate::vocabulary::names::JIAN_XIONG,
                crate::vocabulary::names::FAN_KUI,
                crate::vocabulary::names::GANG_LIE,
                crate::vocabulary::names::JI_JI,
                "jieming",
                "fangzhu",
                "enyuan",
                "chengxiang",
            ];
            let pool: Vec<crate::ids::SkillDefId> = maixie_names
                .iter()
                .filter_map(|&n| ctx.catalog.skill_by_name(n))
                .filter(|&def| !ctx.core.has_skill(m.owner, def))
                .collect();
            if pool.is_empty() {
                return finish_gwent_card(m, ctx, false);
            }
            let sk_def = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
            ctx.insert_skill(
                (m.owner, sk_def),
                crate::state::SkillInstance {
                    owner: m.owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
            ctx.core.vm_aux.flags.insert(
                (m.owner, "gw_guaiwuchaoxue_temp_def".to_string()),
                u32::from(sk_def) + 1,
            );
            let enemies = super::enemies_of(ctx.core, ctx.catalog, m.owner);
            if enemies.is_empty() {
                return finish_gwent_card(m, ctx, false);
            }
            let src = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(src),
                m.owner,
                1,
                crate::event::DamageNature::Normal,
                None,
            )));
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(m.owner).alive {
                ctx.recover(m.owner, 1, None);
            }
            finish_gwent_card(m, ctx, false)
        }
        _ => finish_gwent_card(m, ctx, false),
    }
}
