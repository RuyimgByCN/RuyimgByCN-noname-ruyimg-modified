//! `character/gwent/card.js` 末尾冒险金卡：霍格及其合法用牌/目标过滤。

use super::super::super::*;
use super::adventure_b::airuiting_sha_targets_for_card;
use super::finish_gwent_card;
use super::silver_spells::eligible_bronze_targets;
use crate::catalog::Catalog;
use crate::state::GameStateCore;

/// 霍格 (`character/gwent/card.js:875`): 亮出牌堆顶 6 张牌，选择至多两张依次使用之（其余弃置）。
pub(super) fn step_huoge(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => {
            let mut peeked = Vec::new();
            for _ in 0..6 {
                if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
                    let mut d = std::mem::take(&mut ctx.core.discard);
                    crate::kernel::clear_u05_discard_vanishtags(ctx.core, &d);
                    ctx.core.rng.shuffle(&mut d);
                    ctx.core.deck = d;
                }
                if let Some(c) = ctx.core.deck.pop() {
                    peeked.push(c);
                } else {
                    break;
                }
            }
            m.cards = peeked;
            let usable: Vec<ChoiceOption> = m
                .cards
                .iter()
                .copied()
                .filter(|&c| huoge_card_can_use(ctx.core, ctx.catalog, m.owner, c))
                .map(ChoiceOption::Card)
                .collect();
            if usable.is_empty() {
                for c in std::mem::take(&mut m.cards) {
                    ctx.to_discard(c);
                }
                return finish_gwent_card(m, ctx, true);
            }
            let max = usable.len().min(2);
            m.stage = HostStage::SelectCard;
            Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "gwmaoxian_huoge_pick",
                    options: usable,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            }
        }
        HostStage::SelectCard => {
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter(|c| m.cards.contains(c))
                .take(2)
                .collect();
            let leftover: Vec<CardId> = m
                .cards
                .iter()
                .copied()
                .filter(|c| !chosen.contains(c))
                .collect();
            for c in leftover {
                ctx.to_discard(c);
            }
            m.cards = chosen;
            m.stage = HostStage::TargetChoice;
            advance_huoge_queue(m, ctx)
        }
        HostStage::TargetChoice => advance_huoge_queue(m, ctx),
        HostStage::SelectNature => {
            if m.cards.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            let c = m.cards[0];
            if ctx.catalog.card_kind(c) == CardKind::Jiedao {
                let Some(holder) = ctx.answer().iter().find_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                }) else {
                    m.cards.remove(0);
                    ctx.to_discard(c);
                    m.stage = HostStage::TargetChoice;
                    return advance_huoge_queue(m, ctx);
                };
                let victims = jiedao_victim_options(ctx.core, ctx.catalog, m.owner, holder);
                if victims.is_empty() {
                    m.cards.remove(0);
                    ctx.to_discard(c);
                    m.stage = HostStage::TargetChoice;
                    return advance_huoge_queue(m, ctx);
                }
                m.target = Some(holder);
                m.stage = HostStage::CardsAsk;
                return Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "gwmaoxian_huoge_target",
                        options: victims,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                };
            }
            m.cards.remove(0);
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if targets.is_empty() {
                ctx.to_discard(c);
                m.stage = HostStage::TargetChoice;
                return advance_huoge_queue(m, ctx);
            }
            emit_huoge_use(ctx, m.owner, c, targets);
            m.stage = HostStage::TargetChoice;
            Flow::Yield
        }
        HostStage::CardsAsk => {
            if m.cards.is_empty() {
                return finish_gwent_card(m, ctx, true);
            }
            let c = m.cards.remove(0);
            let holder = m.target.take();
            let victim = ctx.answer().iter().find_map(|o| match *o {
                ChoiceOption::Player(p) => Some(p),
                _ => None,
            });
            let (Some(h), Some(v)) = (holder, victim) else {
                ctx.to_discard(c);
                m.stage = HostStage::TargetChoice;
                return advance_huoge_queue(m, ctx);
            };
            emit_huoge_use(ctx, m.owner, c, vec![h, v]);
            m.stage = HostStage::TargetChoice;
            Flow::Yield
        }
        _ => finish_gwent_card(m, ctx, true),
    }
}

fn advance_huoge_queue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while let Some(&c) = m.cards.first() {
        if !huoge_card_can_use(ctx.core, ctx.catalog, m.owner, c) {
            m.cards.remove(0);
            ctx.to_discard(c);
            continue;
        }
        let target_opts = huoge_card_targets(ctx.core, ctx.catalog, m.owner, c);
        if let Some((opts, min_t, max_t)) = target_opts {
            m.stage = HostStage::SelectNature;
            return Flow::WaitChoice {
                observer: m.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "gwmaoxian_huoge_target",
                    options: opts,
                    min: min_t,
                    max: max_t,
                    allow_cancel: false,
                },
            };
        }
        m.cards.remove(0);
        emit_huoge_use(ctx, m.owner, c, Vec::new());
        return Flow::Yield;
    }
    finish_gwent_card(m, ctx, true)
}

pub(in crate::content::host::step) fn emit_huoge_use(ctx: &mut Ctx<'_>, owner: PlayerId, c: CardId, mut targets: Vec<PlayerId>) {
    let name = super::clean_gwent_name(ctx.catalog.cards[c as usize].name);
    let kind = ctx.catalog.card_kind(c);
    ctx.core.table.push(c);
    if kind.is_equip() {
        ctx.emit(EventData::Equipped {
            player: owner,
            card: c,
            replaced: None,
        });
        return;
    }
    if kind.aoe_need().is_some() && targets.is_empty() {
        targets = ctx
            .core
            .seat_order_from(ctx.core.next_alive(owner))
            .into_iter()
            .filter(|&p| p != owner)
            .collect();
        if matches!(kind, CardKind::Jingleishan | CardKind::Chiyuxi) {
            targets.reverse();
        }
    }
    let use_as = match name {
        "gw_wuyao" => Some(CardKind::Sha),
        "gw_lang" => Some(CardKind::Jiu),
        _ => None,
    };
    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
        c, owner, targets, use_as,
    )));
}

fn is_huoge_no_target_card(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    name: &str,
    kind: CardKind,
) -> bool {
    if kind.equip_slot().is_some() {
        return true;
    }
    if kind.aoe_need().is_some() {
        return core.alive_players().any(|p| p != owner);
    }
    if kind == CardKind::Tao {
        return core.player(owner).hp < core.player(owner).max_hp as i32;
    }
    if kind == CardKind::Jiu || name == "gw_lang" {
        return !core.turn.jiu_used;
    }
    if name == "gw_xianzumaijiu" {
        return !super::has_named_skill(core, catalog, owner, "gw_xianzumaijiu")
            && core
                .vm_aux
                .flags
                .get(&(owner, "gw_xianzumaijiu_used_play".to_string()))
                .copied()
                .unwrap_or(0)
                == 0;
    }
    if kind == CardKind::ShanDian {
        return !core
            .player(owner)
            .judge
            .iter()
            .any(|jc| catalog.card_kind(*jc) == CardKind::ShanDian);
    }
    if matches!(
        kind,
        CardKind::Taoyuan | CardKind::Wugu | CardKind::Poxiao | CardKind::Wenyi
    ) {
        return core.alive_count() > 0;
    }
    if kind == CardKind::Shanbengshu {
        return (0..core.players.len() as PlayerId).any(|p| {
            p != owner
                && core.player(p).alive
                && !core.is_friend_of(catalog, owner, p)
                && !core.player(p).equips.is_empty()
        });
    }
    if kind == CardKind::Wuzhong
        && (matches!(name, "无中生有" | "wuzhong") || name.starts_with("wuzhong_"))
    {
        return true;
    }
    match name {
        "gw_guaiwuchaoxue" => {
            core.vm_aux
                .flags
                .get(&(owner, "gw_guaiwuchaoxue_used_play".to_string()))
                .copied()
                .unwrap_or(0)
                == 0
        }
        "gw_hudiewu" => core.seat_order_from(owner).into_iter().any(|p| {
            p != owner
                && core.player(p).alive
                && (!core.player(p).equips.is_empty() || !core.player(p).judge.is_empty())
        }),
        "gw_aerdeyin" => core.seat_order_from(owner).into_iter().any(|p| {
            p != owner && core.player(p).alive && !core.is_friend_of(catalog, owner, p)
        }),
        "gw_ansha" => core.seat_order_from(owner).into_iter().any(|p| {
            p != owner
                && core.player(p).alive
                && !core.is_friend_of(catalog, owner, p)
                && core.player(p).hp == 1
        }),
        "gw_xinsheng" => core
            .seat_order_from(owner)
            .into_iter()
            .any(|p| core.player(p).alive && super::gold_spells::is_player_seen(core, p)),
        "gw_niuquzhijing" => {
            let cur_hp = core.player(owner).hp;
            core.seat_order_from(owner)
                .into_iter()
                .any(|p| core.player(p).alive && core.player(p).hp != cur_hp)
        }
        "gw_tunshi" => {
            if core.player(owner).max_hp <= 1 {
                return false;
            }
            core.seat_order_from(owner).into_iter().any(|p| {
                p != owner
                    && core.player(p).alive
                    && !core.is_friend_of(catalog, owner, p)
                    && core.skills.iter().any(|((o, def), _)| {
                        *o == p
                            && !catalog.is_charlotte_skill(*def)
                            && !core.player(p).equips.iter().any(|ec| {
                                catalog.cards[*ec as usize]
                                    .equip_skills
                                    .contains(&catalog.skills[*def as usize].name)
                            })
                    })
            })
        }
        "gw_yigeniyin"
        | "gw_leizhoushu"
        | "gw_zhongmozhizhan"
        | "gw_ganhan"
        | "gw_huangjiashenpan"
        | "gw_chongci"
        | "gw_shizizhaohuan"
        | "gw_zuihouyuanwang"
        | "gw_zirankuizeng"
        | "gwmaoxian_yenaifa"
        | "gwmaoxian_telisi"
        | "gwmaoxian_hengsaite"
        | "gwmaoxian_bulanwang"
        | "gwmaoxian_haluo"
        | "gwmaoxian_airuiting"
        | "gwmaoxian_aisinie"
        | "gwmaoxian_falanxisika"
        | "gwmaoxian_huoge" => true,
        _ => false,
    }
}

pub(in crate::content::host::step) fn huoge_card_can_use(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    let name = super::clean_gwent_name(catalog.cards[c as usize].name);
    let kind = catalog.card_kind(c);
    if matches!(
        name,
        "gw_dieyi" | "gwjinli_jiu" | "gw_fuyuan" | "gw_dudayuanshuai1" | "gw_dudayuanshuai2"
    ) {
        return false;
    }
    if kind == CardKind::Shan || kind == CardKind::Nullify {
        return false;
    }
    let is_tao_blocked = kind == CardKind::Tao
        && (crate::kernel::tao_futai2_banned(core, catalog, owner)
            || crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core,
                catalog,
                owner,
                owner,
                c,
                CardKind::Tao,
                crate::event::DamageNature::Normal,
            ));
    if is_tao_blocked {
        return false;
    }
    let (eff_kind, eff_nature) = huoge_eff_kind_nature(name, kind);
    if core.player(owner).jileng_active
        || catalog.blocks_all_cards(core, owner)
        || crate::rvm_bridge::modifiers::eval_card_use_canceled(
            core, catalog, owner, c, eff_kind, eff_nature,
        )
    {
        return false;
    }
    if kind.is_sha_like() || name == "gw_wuyao" {
        return !crate::kernel::sha_blocked_for(core, catalog, owner)
            && !airuiting_sha_targets_for_card(core, catalog, owner, Some(c)).is_empty();
    }
    if let Some((opts, min_t, _)) = huoge_card_targets(core, catalog, owner, c) {
        return opts.len() >= min_t;
    }
    is_huoge_no_target_card(core, catalog, owner, name, kind)
}

fn huoge_eff_kind_nature(name: &str, kind: CardKind) -> (CardKind, crate::event::DamageNature) {
    match name {
        "gw_wuyao" => (CardKind::Sha, crate::event::DamageNature::Normal),
        "gw_lang" => (CardKind::Jiu, crate::event::DamageNature::Normal),
        _ => {
            let nature = match kind {
                CardKind::FireSha => crate::event::DamageNature::Fire,
                CardKind::LeiSha => crate::event::DamageNature::Thunder,
                _ => crate::event::DamageNature::Normal,
            };
            (kind, nature)
        }
    }
}

fn has_weimu_skill(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    [crate::vocabulary::names::BOSS_WEIMU, "weimu", "reweimu"]
        .into_iter()
        .any(|sk| catalog.skill_by_name(sk).is_some_and(|d| core.has_skill(p, d)))
}

fn filter_huoge_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
    kind: CardKind,
    nature: crate::event::DamageNature,
    raw: (Vec<ChoiceOption>, usize, usize),
) -> (Vec<ChoiceOption>, usize, usize) {
    let (opts, min_t, max_t) = raw;
    let is_black_trick = !kind.is_basic_kind()
        && !kind.is_equip()
        && matches!(
            catalog.effective_card_suit(core, owner, c),
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        );
    let swd_wuxie = matches!(kind, CardKind::Le | CardKind::Bing)
        && catalog
            .skill_by_name("swd_wuxie")
            .is_some_and(|d| core.has_skill(owner, d));
    let filtered: Vec<ChoiceOption> = opts
        .into_iter()
        .filter(|opt| {
            let ChoiceOption::Player(t) = *opt else {
                return false;
            };
            if t != owner && is_target_hidden(core, t) {
                return false;
            }
            if crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core, catalog, owner, t, c, kind, nature,
            ) {
                return false;
            }
            if is_black_trick && has_weimu_skill(core, catalog, t) {
                return false;
            }
            if kind == CardKind::Chain
                && crate::rvm_bridge::modifiers::d34_l28_mods::rewrite_tengjia_blocks_link(
                    core, catalog, t,
                )
            {
                return false;
            }
            if matches!(kind, CardKind::Le | CardKind::Bing)
                && (catalog.blocks_delayed_trick(core, t) || (swd_wuxie && t != owner))
            {
                return false;
            }
            true
        })
        .collect();
    let clamped_max = max_t.min(filtered.len());
    (filtered, min_t, clamped_max)
}

/// 若牌需手选目标返回 `Some((options, min, max))`，否则（装备/无目标/全自动目标）返回 `None`。
pub(in crate::content::host::step) fn huoge_card_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> Option<(Vec<ChoiceOption>, usize, usize)> {
    let name = super::clean_gwent_name(catalog.cards[c as usize].name);
    let kind = catalog.card_kind(c);
    if kind.is_sha_like() || name == "gw_wuyao" {
        return Some((airuiting_sha_targets_for_card(core, catalog, owner, Some(c)), 1, 1));
    }
    if kind == CardKind::Jiedao {
        return Some((jiedao_holder_options(core, catalog, owner, c), 1, 1));
    }
    if kind.aoe_need().is_some()
        || kind.equip_slot().is_some()
        || matches!(
            kind,
            CardKind::Wuzhong | CardKind::Taoyuan | CardKind::Wugu | CardKind::ShanDian | CardKind::Poxiao | CardKind::Wenyi | CardKind::Shanbengshu
        ) && !(kind == CardKind::Wuzhong && name.starts_with("gw")) {
        return None;
    }
    if matches!(
        name,
        "gw_hudiewu"
            | "gw_yigeniyin"
            | "gw_leizhoushu"
            | "gw_aerdeyin"
            | "gw_ansha"
            | "gw_xinsheng"
            | "gw_niuquzhijing"
            | "gw_zhongmozhizhan"
            | "gw_ganhan"
            | "gw_huangjiashenpan"
            | "gw_tunshi"
            | "gw_chongci"
            | "gw_shizizhaohuan"
            | "gw_zuihouyuanwang"
            | "gw_zirankuizeng"
            | "gw_guaiwuchaoxue"
            | "gw_xianzumaijiu"
            | "gwmaoxian_yenaifa"
            | "gwmaoxian_telisi"
            | "gwmaoxian_hengsaite"
            | "gwmaoxian_bulanwang"
            | "gwmaoxian_haluo"
            | "gwmaoxian_airuiting"
            | "gwmaoxian_aisinie"
            | "gwmaoxian_falanxisika"
            | "gwmaoxian_huoge"
    ) {
        return None;
    }
    let (eff_kind, eff_nature) = huoge_eff_kind_nature(name, kind);
    huoge_card_targets_raw(core, catalog, owner, name, kind)
        .map(|raw| filter_huoge_targets(core, catalog, owner, c, eff_kind, eff_nature, raw))
}

fn huoge_card_targets_raw(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    name: &str,
    kind: CardKind,
) -> Option<(Vec<ChoiceOption>, usize, usize)> {
    let others_with_cards = |need_hand: bool| -> Vec<ChoiceOption> {
        core.seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && !is_target_hidden(core, p)
                    && (!need_hand || !core.player(p).hand.is_empty())
            })
            .map(ChoiceOption::Player)
            .collect()
    };
    if kind == CardKind::Chain {
        let opts: Vec<ChoiceOption> = core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                core.player(p).alive
                    && (p == owner
                        || (!core.player(p).jidong_active
                            && !core.player(p).yinshen_active
                            && !core.player(p).jileng_active
                            && !core.player(p).yinmi_active))
            })
            .map(ChoiceOption::Player)
            .collect();
        let max = opts.len().min(2);
        return Some((opts, 1, max));
    }
    let not_hidden = |p: PlayerId| {
        !core.player(p).jidong_active
            && !core.player(p).yinshen_active
            && !core.player(p).jileng_active
            && !core.player(p).yinmi_active
    };
    let qicai = catalog
        .skill_by_name(crate::content::standard::QI_CAI)
        .is_some_and(|d| core.has_skill(owner, d));
    let wuxin = catalog
        .skill_by_name(crate::content::standard::BOSS_WUXIN)
        .is_some_and(|d| core.has_skill(owner, d));
    let qianxun = catalog.skill_by_name(crate::content::standard::QIAN_XUN);
    let kongcheng = catalog.skill_by_name(crate::content::standard::KONG_CHENG);
    if kind == CardKind::Duel {
        let opts = core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && not_hidden(p)
                    && !(kongcheng.is_some_and(|d| core.has_skill(p, d))
                        && core.player(p).hand.is_empty())
            })
            .map(ChoiceOption::Player)
            .collect();
        return Some((opts, 1, 1));
    }
    if kind == CardKind::Dismantle {
        let opts = core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && not_hidden(p)
                    && (!core.player(p).hand.is_empty()
                        || !core.player(p).equips.is_empty()
                        || !core.player(p).judge.is_empty())
            })
            .map(ChoiceOption::Player)
            .collect();
        return Some((opts, 1, 1));
    }
    if kind == CardKind::Snatch {
        let opts = core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && not_hidden(p)
                    && !qianxun.is_some_and(|d| core.has_skill(p, d))
                    && (!core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
                    && (qicai || core.computed_distance(catalog, owner, p) <= 1)
            })
            .map(ChoiceOption::Player)
            .collect();
        return Some((opts, 1, 1));
    }
    if kind == CardKind::Le {
        if wuxin {
            return Some((Vec::new(), 1, 1));
        }
        let opts = core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && not_hidden(p)
                    && !qianxun.is_some_and(|d| core.has_skill(p, d))
                    && !catalog.blocks_delayed_trick(core, p)
                    && !core.player(p).judge.iter().any(|jc| catalog.card_kind(*jc) == CardKind::Le)
            })
            .map(ChoiceOption::Player)
            .collect();
        return Some((opts, 1, 1));
    }
    if kind == CardKind::Bing {
        if wuxin {
            return Some((Vec::new(), 1, 1));
        }
        let opts = core
            .seat_order_from(owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && not_hidden(p)
                    && !catalog.blocks_delayed_trick(core, p)
                    && !core.player(p).judge.iter().any(|jc| catalog.card_kind(*jc) == CardKind::Bing)
                    && (qicai || core.computed_distance(catalog, owner, p) <= 1)
            })
            .map(ChoiceOption::Player)
            .collect();
        return Some((opts, 1, 1));
    }
    if matches!(
        kind,
        CardKind::Qinpendayu
            | CardKind::Birinongwu
            | CardKind::Ciguhanshuang
            | CardKind::Baoxueyaoshui
            | CardKind::Zhihuanjun
            | CardKind::Zumoshoukao
            | CardKind::Aozuzhilei
            | CardKind::Kunenfayin
            | CardKind::Yanziyaoshui
    ) {
        return Some((eligible_bronze_targets(core, catalog, owner, kind), 1, 1));
    }
    match name {
        "gwmaoxian_yioufeisi" => Some((others_with_cards(false), 2, 2)),
        "gwmaoxian_fulisi" => {
            let opts = others_with_cards(true);
            let max = opts.len().min(3);
            Some((opts, 1, max))
        }
        "gwmaoxian_kaerweite" => {
            let opts = others_with_cards(true);
            let max = opts.len().min(2);
            Some((opts, 1, max))
        }
        "gwmaoxian_fuertaisite" => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len().min(2);
            Some((opts, 1, max))
        }
        "gw_zhuoshao" => {
            let mhp = core.alive_players().map(|p| core.player(p).hp).max().unwrap_or(0);
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && core.player(p).hp == mhp)
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len();
            Some((opts, 1, max))
        }
        "gw_baishuang" => {
            let opts = eligible_bronze_targets(core, catalog, owner, CardKind::Ciguhanshuang);
            let max = opts.len().min(3);
            Some((opts, 1, max))
        }
        "gw_baobaoshu" => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && !super::has_named_skill(core, catalog, p, "gw_baobaoshu"))
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len().min(2);
            Some((opts, 1, max))
        }
        "gw_youer" | "gw_tongdi" => Some((others_with_cards(true), 1, 1)),
        "gw_nuhaifengbao" => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && !super::has_named_skill(core, catalog, p, "gw_nuhaifengbao"))
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_enxier" => {
            let h0 = core.player(owner).hand.len() as i32;
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && (core.player(p).hand.len() as i32 - h0).abs() <= 1)
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_kuite" => {
            let h0 = core.player(owner).hand.len();
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && core.player(p).alive && core.player(p).hand.len() >= h0)
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_dagong" => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    core.player(p).alive
                        && (!super::has_named_skill(core, catalog, p, GW_CIGU_STATE_KEY)
                            || !super::has_named_skill(core, catalog, p, GW_QINPEN_STATE_KEY)
                            || !super::has_named_skill(core, catalog, p, GW_BIRIN_STATE_KEY))
                })
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gwmaoxian_luoqi" => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        "gw_butianshu" | "gwmaoxian_jieluote" | "gwmaoxian_laduoweide" | "gwmaoxian_gaier" => {
            let opts = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            Some((opts, 1, 1))
        }
        _ => None,
    }
}

fn is_target_hidden(core: &GameStateCore, p: PlayerId) -> bool {
    core.player(p).jidong_active
        || core.player(p).yinshen_active
        || core.player(p).jileng_active
        || core.player(p).yinmi_active
}

pub(in crate::content::host::step) fn jiedao_victim_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    holder: PlayerId,
) -> Vec<ChoiceOption> {
    let range = core.attack_range(catalog, holder);
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&v| {
            v != owner
                && v != holder
                && core.player(v).alive
                && !crate::kernel::sha_user_or_target_blocked(core, catalog, holder, v)
                && (core.computed_distance(catalog, holder, v) <= range
                    || core.turn.zhufangshenshi_target == Some(v))
        })
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn jiedao_holder_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: CardId,
) -> Vec<ChoiceOption> {
    let is_black = matches!(
        catalog.effective_card_suit(core, owner, card),
        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
    );
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&h| {
            h != owner
                && core.player(h).alive
                && !is_target_hidden(core, h)
                && !(is_black && has_weimu_skill(core, catalog, h))
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core,
                    catalog,
                    owner,
                    h,
                    card,
                    CardKind::Jiedao,
                    crate::event::DamageNature::Normal,
                )
                && core
                    .player(h)
                    .equips
                    .iter()
                    .any(|c| catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Weapon))
                && !jiedao_victim_options(core, catalog, owner, h).is_empty()
        })
        .map(ChoiceOption::Player)
        .collect()
}
