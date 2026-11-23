//! Batch U05-1 万智牌地牌与询问表（`card/mtg/{card,skill}.js` 及 `character/mtg/skill.js` 询问重构）。

use super::super::*;
use super::l27_batch_a::{alive_all, alive_others, discard_cards, grant_skill_named, set_flag};
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::ChoiceOption;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(in crate::content::host) fn is_delayed_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui | CardKind::Guiyoujie
    )
}

fn is_huoshan_card(catalog: &Catalog, card: Option<CardId>) -> bool {
    card.and_then(|cid| catalog.cards.get(cid as usize))
        .is_some_and(|cd| {
            let base = cd
                .name
                .rsplit_once('_')
                .filter(|(_, tail)| !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()))
                .map(|(stem, _)| stem)
                .unwrap_or(cd.name);
            matches!(base, "yc_huoshan" | "huoshan")
        })
}

fn is_huogong_name(catalog: &Catalog, card: Option<CardId>) -> bool {
    card.and_then(|cid| catalog.cards.get(cid as usize))
        .is_some_and(|cd| cd.name == "huogong")
}

/// 上游 `lib.filter.targetEnabled2(card, player, target)` 等价判定（`mhuanyi`/`msuoling`/`g_chenhuodajie` 共用）。
#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn target_enabled2_for_card(
    core: &GameStateCore,
    catalog: &Catalog,
    card: Option<CardId>,
    kind: CardKind,
    suit_opt: Option<Suit>,
    caster: PlayerId,
    target: PlayerId,
) -> bool {
    let tp = core.player(target);
    if !tp.alive {
        return false;
    }
    if is_delayed_trick_kind(kind) {
        if is_huoshan_card(catalog, card) {
            if target != caster {
                return false;
            }
            if tp.judge.iter().any(|&jc| is_huoshan_card(catalog, Some(jc))) {
                return false;
            }
        } else if tp
            .judge
            .iter()
            .any(|&jc| catalog.card_kind(jc) == kind && !is_huoshan_card(catalog, Some(jc)))
        {
            return false;
        }
        if catalog.blocks_delayed_trick(core, target) {
            return false;
        }
        if target != caster
            && catalog
                .skill_by_name("swd_wuxie")
                .is_some_and(|d| core.has_skill(caster, d))
        {
            return false;
        }
        if kind == CardKind::Le
            && catalog
                .skill_by_name(crate::content::standard::QIAN_XUN)
                .is_some_and(|d| core.has_skill(target, d))
        {
            return false;
        }
    }
    let black = matches!(suit_opt, Some(Suit::Spade | Suit::Club));
    if black && kind.is_trick_kind() {
        if catalog
            .skill_by_name(crate::content::standard::BOSS_WEIMU)
            .is_some_and(|d| core.has_skill(target, d))
        {
            return false;
        }
    }
    if target != caster {
        if black
            && catalog
                .skill_by_name(crate::content::host::GJ_FOOD_MOLICHA_KEY)
                .is_some_and(|d| core.skills.contains_key(&(target, d)))
        {
            return false;
        }
        if tp.jidong_active || tp.yinshen_active || tp.jileng_active || tp.yinmi_active {
            return false;
        }
        if catalog
            .skill_by_name(crate::content::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
            .is_some_and(|d| core.has_skill(caster, d))
        {
            return false;
        }
    }
    if kind == CardKind::Snatch
        && catalog
            .skill_by_name(crate::content::standard::QIAN_XUN)
            .is_some_and(|d| core.has_skill(target, d))
    {
        return false;
    }
    if matches!(
        kind,
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Duel
    ) && catalog
        .skill_by_name(crate::content::standard::KONG_CHENG)
        .is_some_and(|d| core.has_skill(target, d))
        && tp.hand.is_empty()
    {
        return false;
    }
    if kind == CardKind::Sha
        && catalog
            .skill_by_name(crate::content::standard::BOSS_REMEN)
            .is_some_and(|d| core.has_skill(target, d))
        && !tp
            .equips
            .iter()
            .any(|&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Armor))
    {
        return false;
    }
    if target == caster
        && (kind.is_sha_like()
            || matches!(
                kind,
                CardKind::Dismantle
                    | CardKind::Snatch
                    | CardKind::Duel
                    | CardKind::Le
                    | CardKind::Bing
                    | CardKind::Guiyoujie
                    | CardKind::Chenhuodajie
                    | CardKind::Fudichouxin
                    | CardKind::Toulianghuanzhu
                    | CardKind::Caochuanjiejian
                    | CardKind::Shuigong
                    | CardKind::Zhaomingdan
                    | CardKind::Feibiao
                    | CardKind::Shihuifen
                    | CardKind::Liufengsan
                    | CardKind::Zhufangshenshi
                    | CardKind::Geanguanhuo
                    | CardKind::Jiedao
                    | CardKind::Bingpotong
                    | CardKind::Qiankunbiao
                    | CardKind::Shenhuofeiya
                    | CardKind::Zumoshoukao
                    | CardKind::Birinongwu
                    | CardKind::Qinpendayu
                    | CardKind::Ciguhanshuang
                    | CardKind::Caoyao
            ))
        && !is_huoshan_card(catalog, card)
    {
        return false;
    }
    match kind {
        CardKind::Dismantle | CardKind::Snatch => {
            !tp.hand.is_empty() || !tp.equips.is_empty() || !tp.judge.is_empty()
        }
        CardKind::Chenhuodajie | CardKind::Qiankunbiao => {
            !tp.hand.is_empty() || !tp.equips.is_empty()
        }
        CardKind::Shuigong => !tp.equips.is_empty(),
        CardKind::Fudichouxin
        | CardKind::Toulianghuanzhu
        | CardKind::Caochuanjiejian
        | CardKind::Shujinsan
        | CardKind::Bingpotong
        | CardKind::Geanguanhuo
        | CardKind::Zhaomingdan => !tp.hand.is_empty(),
        CardKind::Zhiliaobo => tp.hp < tp.max_hp as i32,
        CardKind::Xuejibingbao => tp.xuejibingbao == 0,
        CardKind::Mianlijinzhen => tp.hp >= core.player(caster).hp,
        _ if is_huogong_name(catalog, card) => !tp.hand.is_empty(),
        _ => true,
    }
}

fn snatch_in_range(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    cid: CardId,
    target: PlayerId,
) -> bool {
    let qicai = catalog
        .skill_by_name(crate::content::standard::QI_CAI)
        .is_some_and(|d| core.has_skill(actor, d));
    let shenen = catalog
        .skill_by_name(crate::content::standard::BOSS_SHENEN)
        .is_some_and(|def| core.skills.keys().any(|(_, dd)| *dd == def))
        && core.is_boss_side(actor);
    let shouyi = catalog
        .skill_by_name(crate::content::standard::BOSS_SHOUYI)
        .is_some_and(|d| core.has_skill(actor, d));
    let fentian_red = matches!(
        catalog.effective_card_suit(core, actor, cid),
        Suit::Heart | Suit::Diamond
    ) && catalog
        .skill_by_name(crate::content::standard::BOSS_FENTIAN)
        .is_some_and(|d| core.has_skill(actor, d));
    qicai
        || shenen
        || shouyi
        || fentian_red
        || core.computed_distance(catalog, actor, target) <= 1
}

pub(super) fn jiedao_victims_for(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    _cid: CardId,
    holder: PlayerId,
) -> Vec<PlayerId> {
    alive_all(core, actor)
        .into_iter()
        .filter(|&v| {
            v != holder
                && core.computed_distance(catalog, holder, v) <= core.attack_range(catalog, holder)
                && target_enabled2_for_card(core, catalog, None, CardKind::Sha, None, holder, v)
        })
        .collect()
}

pub(super) fn msuoling_auto_targets(
    core: &GameStateCore,
    _catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
) -> Option<Vec<PlayerId>> {
    match kind {
        CardKind::Wuzhong
        | CardKind::Taoyuan
        | CardKind::Wugu
        | CardKind::Shezhanqunru
        | CardKind::Jihuocard
        | CardKind::Dunpaigedang
        | CardKind::Xingjiegoutong
        | CardKind::Chuansongmen
        | CardKind::Tanshezhiren
        | CardKind::Yuansuhuimie
        | CardKind::Shijieshu
        | CardKind::Heilonglinpian
        | CardKind::Yangpijuan
        | CardKind::Shencaojie => Some(Vec::new()),
        CardKind::SavageAttack | CardKind::ArcheryAttack | CardKind::Jingleishan | CardKind::Chiyuxi => {
            let mut aoe: Vec<PlayerId> = core
                .seat_order_from(core.next_alive(actor))
                .into_iter()
                .filter(|p| *p != actor)
                .collect();
            if matches!(kind, CardKind::Jingleishan | CardKind::Chiyuxi) {
                aoe.reverse();
            }
            Some(aoe)
        }
        CardKind::Xianluhui => {
            let mut aoe: Vec<PlayerId> = core
                .seat_order_from(core.next_alive(actor))
                .into_iter()
                .filter(|p| *p != actor && core.player(*p).alive && core.player(*p).hp < core.player(*p).max_hp as i32)
                .collect();
            if core.player(actor).hp < core.player(actor).max_hp as i32 {
                aoe.push(actor);
            }
            aoe.reverse();
            Some(aoe)
        }
        _ => None,
    }
}

pub(super) fn trick_targets_for(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    cid: CardId,
) -> (Vec<ChoiceOption>, usize, usize) {
    let kind = catalog.card_kind(cid);
    if crate::kernel::jinyan_blocks_black_trick(core, catalog, actor, cid) {
        return (Vec::new(), 1, 1);
    }
    let suit_opt = Some(catalog.effective_card_suit(core, actor, cid));
    if is_huogong_name(catalog, Some(cid)) {
        let opts: Vec<ChoiceOption> = alive_all(core, actor)
            .into_iter()
            .filter(|&p| !core.player(p).hand.is_empty() && target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, p))
            .map(ChoiceOption::Player)
            .collect();
        return (opts, 1, 1);
    }
    match kind {
        CardKind::Jiedao => {
            let opts: Vec<ChoiceOption> = alive_others(core, actor)
                .into_iter()
                .filter(|&h| {
                    core.player(h).equips.iter().any(|&eq| catalog.card_slot_of(core, eq) == Some(EquipSlot::Weapon))
                        && target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, h)
                        && !jiedao_victims_for(core, catalog, actor, cid, h).is_empty()
                })
                .map(ChoiceOption::Player)
                .collect();
            (opts, 1, 1)
        }
        CardKind::Geanguanhuo => {
            let opts: Vec<ChoiceOption> = alive_others(core, actor)
                .into_iter()
                .filter(|&p| target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, p))
                .map(ChoiceOption::Player)
                .collect();
            if opts.len() < 2 {
                (Vec::new(), 2, 2)
            } else {
                (opts, 2, 2)
            }
        }
        CardKind::Bingpotong => {
            let opts: Vec<ChoiceOption> = alive_others(core, actor)
                .into_iter()
                .filter(|&p| target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, p))
                .map(ChoiceOption::Player)
                .collect();
            let max = 3.min(opts.len());
            (opts, 1, max.max(1))
        }
        CardKind::Chain => {
            let opts: Vec<ChoiceOption> = alive_all(core, actor)
                .into_iter()
                .filter(|&p| target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, p))
                .map(ChoiceOption::Player)
                .collect();
            let max = 2.min(opts.len());
            (opts, 1, max.max(1))
        }
        CardKind::Snatch => {
            let opts: Vec<ChoiceOption> = alive_others(core, actor)
                .into_iter()
                .filter(|&p| {
                    snatch_in_range(core, catalog, actor, cid, p)
                        && target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, p)
                })
                .map(ChoiceOption::Player)
                .collect();
            (opts, 1, 1)
        }
        CardKind::Fudichouxin => {
            if core.player(actor).hand.is_empty() {
                return (Vec::new(), 1, 1);
            }
            let opts: Vec<ChoiceOption> = alive_others(core, actor)
                .into_iter()
                .filter(|&p| target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, p))
                .map(ChoiceOption::Player)
                .collect();
            (opts, 1, 1)
        }
        _ => {
            let pool = if matches!(
                kind,
                CardKind::Shujinsan
                    | CardKind::Mianlijinzhen
                    | CardKind::Zhiliaobo
                    | CardKind::Linghunzhihuo
                    | CardKind::WangmeiZhike
                    | CardKind::Guisheqi
                    | CardKind::Shatang
                    | CardKind::Liuxinghuoyu
            ) {
                alive_all(core, actor)
            } else {
                alive_others(core, actor)
            };
            let opts: Vec<ChoiceOption> = pool
                .into_iter()
                .filter(|&p| target_enabled2_for_card(core, catalog, Some(cid), kind, suit_opt, actor, p))
                .map(ChoiceOption::Player)
                .collect();
            (opts, 1, 1)
        }
    }
}

pub(in crate::content::host) fn ask_u05_mtg(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let key = match (behavior, stage) {
        (HostBehavior::U05MtgCharBatch, HostStage::SelectNature) if target.is_none() => "lingyong",
        (HostBehavior::U05MtgCharBatch, HostStage::SelectNature) => "mhuanyi",
        (HostBehavior::U05MtgCharBatch, HostStage::TargetChoice) if card.is_none() => "mhuanyi",
        (HostBehavior::U05MtgCharBatch, HostStage::TargetChoice) => "msuoling",
        (HostBehavior::U05MtgCharBatch, HostStage::CardsAsk) if card.is_some() => "msuoling",
        (HostBehavior::U05MtgCharBatch, HostStage::SelectCard) if card.is_none() => "msilian",
        (HostBehavior::U05MtgCharBatch, HostStage::SelectCard) => "msuoling",
        (HostBehavior::U05MtgLandExtraBatch, HostStage::TargetChoice) => "mtg_duzhao",
        (HostBehavior::U05MtgLandExtraBatch, HostStage::SelectNature) => "mtg_cangbaohaiwan",
        (HostBehavior::U05MtgLandExtraBatch, HostStage::CardsAsk) => "mtg_lindixiliu_skill",
        (HostBehavior::U05MtgLandExtraBatch, HostStage::SelectCard) if card == Some(1) => {
            "mtg_bingheyaosai_skill"
        }
        (HostBehavior::U05MtgLandExtraBatch, HostStage::SelectCard) => "mtg_lindixiliu_skill",
        _ => return None,
    };
    match (key, stage) {
        ("lingyong", HostStage::SelectNature) => {
            let slots = super::u05_b1_mtg_subplayer::active_subplayer_slots(core, owner);
            (!slots.is_empty()).then(|| {
                ask(
                    owner,
                    "u05_lingyong_summon",
                    ChoiceKind::PickTag,
                    slots.into_iter().map(|i| ChoiceOption::Tag(i as u8)).collect(),
                    1,
                    1,
                    false,
                )
            })
        }
        ("msilian", HostStage::SelectCard) => Some(ask(
            owner,
            "u05_msilian_confirm",
            ChoiceKind::RespondCard,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        )),
        ("mhuanyi", HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(owner, "u05_mhuanyi_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        ("mhuanyi", HostStage::SelectNature) => Some(ask(
            owner,
            "u05_mhuanyi_kind",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            true,
        )),
        ("msuoling", HostStage::SelectCard) => {
            let cid = card?;
            let suit_ord = cards
                .first()
                .map(|&p| catalog.effective_card_suit(core, p as PlayerId, cid))
                .or_else(|| catalog.cards.get(cid as usize).map(|cd| cd.suit))?;
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == suit_ord)
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(owner, "u05_msuoling_discard", ChoiceKind::SelectCards, opts, 0, 1, true)
            })
        }
        ("msuoling", HostStage::TargetChoice) => {
            let cid = card?;
            let (opts, min, max) = trick_targets_for(core, catalog, owner, cid);
            (opts.len() >= min).then(|| {
                ask(owner, "u05_msuoling_target", ChoiceKind::SelectPlayers, opts, min, max, false)
            })
        }
        ("msuoling", HostStage::CardsAsk) => {
            let cid = card?;
            let holder = *targets.first()?;
            let opts: Vec<ChoiceOption> = jiedao_victims_for(core, catalog, owner, cid, holder)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(owner, "u05_msuoling_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)
            })
        }
        ("mtg_duzhao", HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = alive_all(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(owner, "u05_mtg_duzhao_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)
            })
        }
        ("mtg_cangbaohaiwan", HostStage::SelectNature) => Some(ask(
            owner,
            "u05_mtg_cangbaohaiwan_pack",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        ("mtg_lindixiliu_skill", HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(owner, "u05_mtg_lindixiliu_heart", ChoiceKind::SelectCards, opts, 0, 1, true)
            })
        }
        ("mtg_lindixiliu_skill", HostStage::CardsAsk) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .judge
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(owner, "u05_mtg_lindixiliu_judge", ChoiceKind::SelectCards, opts, 1, 1, false)
            })
        }
        ("mtg_bingheyaosai_skill", HostStage::SelectCard) => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!opts.is_empty()).then(|| {
                ask(
                    owner,
                    "u05_mtg_bingheyaosai_sha_discard",
                    ChoiceKind::SelectCards,
                    opts,
                    1,
                    1,
                    false,
                )
            })
        }
        _ => None,
    }
}

pub(super) fn wait_ask_mtg(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    next_stage: HostStage,
) -> Flow {
    let Some(exp) = ask_u05_mtg(
        m.behavior,
        next_stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) else {
        return Flow::Done;
    };
    m.stage = next_stage;
    Flow::WaitChoice {
        observer: exp.observer,
        spec: ChoiceSpec {
            kind: exp.kind,
            prompt: exp.prompt,
            options: exp.options,
            min: exp.min,
            max: exp.max,
            allow_cancel: exp.allow_cancel,
        },
    }
}

pub(in crate::content::host) fn pull_du_card_to_player(ctx: &mut Ctx<'_>, target: PlayerId) {
    let cid = ctx
        .core
        .vm_aux
        .piles
        .get_mut(&(0, "mtg_du_supply".to_string()))
        .and_then(|p| (!p.is_empty()).then(|| p.remove(0)))
        .or_else(|| {
            let pos = ctx.core.discard.iter().position(|&c| {
                ctx.catalog
                    .cards
                    .get(c as usize)
                    .is_some_and(|cd| cd.name.starts_with("mtg_du_"))
            });
            pos.map(|i| ctx.core.discard.remove(i))
        });
    if let Some(cid) = cid {
        ctx.core.player_mut(target).hand.push(cid);
        ctx.log(LogEvent::CardsDrawn {
            player: target,
            cards: vec![cid],
        });
    }
}

pub(in crate::content::host) fn sync_global_land_skill(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    new_kind: CardKind,
) {
    let key_of = |k: CardKind| match k {
        CardKind::Yixialan => Some(MTG_YIXIALAN_RECAST_KEY),
        CardKind::Linzhongjianta => Some(MTG_LINZHONGJIANTA_SKILL_KEY),
        CardKind::Youlin => Some(MTG_YOULIN_DISCOVER_KEY),
        CardKind::Bingheyaosai => Some("mtg_bingheyaosai_skill"),
        CardKind::Lindixiliu => Some("mtg_lindixiliu_skill"),
        _ => None,
    };
    let is_global = |k: CardKind| matches!(k, CardKind::Bingheyaosai | CardKind::Lindixiliu);
    let old_kind = ctx.core.current_land;
    let old_key = old_kind.and_then(key_of);
    let new_key = key_of(new_kind);
    let seats = ctx.core.players.len() as PlayerId;
    for extra_k in ["mtg_duzhao_skill", "mtg_cangbaohaiwan_skill"] {
        if let Some(d) = ctx.catalog.skill_by_name(extra_k) {
            for p in 0..seats {
                ctx.core.skills.remove(&(p, d));
            }
        }
    }
    ctx.core.vm_aux.flags.remove(&(0, "u05_extra_land".to_string()));
    if let Some(k) = old_key {
        if old_key != new_key || old_kind.is_some_and(is_global) {
            if let Some(d) = ctx.catalog.skill_by_name(k) {
                for p in 0..seats {
                    ctx.core.skills.remove(&(p, d));
                }
            }
        }
    }
    if let Some(k) = new_key {
        if let Some(d) = ctx.catalog.skill_by_name(k) {
            if is_global(new_kind) {
                for p in alive_all(ctx.core, 0) {
                    grant_skill_named(ctx, p, k);
                }
            } else {
                ctx.insert_skill(
                    (owner, d),
                    crate::state::SkillInstance {
                        owner,
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

fn switch_extra_land(ctx: &mut Ctx<'_>, new_extra: u32) {
    let old_land = ctx.core.current_land.take();
    if let Some(old_k) = old_land {
        for decl in crate::vocabulary::card_skills::card_kind_host_skills(old_k) {
            if let Some(d) = ctx.catalog.skill_by_name(decl.name) {
                for seat in 0..ctx.core.players.len() as PlayerId {
                    ctx.core.skills.remove(&(seat, d));
                }
            }
        }
    }
    for key in ["mtg_duzhao_skill", "mtg_cangbaohaiwan_skill"] {
        if let Some(d) = ctx.catalog.skill_by_name(key) {
            for seat in 0..ctx.core.players.len() as PlayerId {
                ctx.core.skills.remove(&(seat, d));
            }
        }
    }
    set_flag(ctx.core, 0, "u05_extra_land", new_extra);
    let skill_name = match new_extra {
        1 => Some("mtg_duzhao_skill"),
        2 => Some("mtg_cangbaohaiwan_skill"),
        _ => None,
    };
    if let Some(sk) = skill_name {
        for seat in alive_all(ctx.core, 0) {
            grant_skill_named(ctx, seat, sk);
        }
    }
}

pub(in crate::content::host) fn handle_u05_mtg_land_extra(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "mtg_duzhao" | "g_mtg_duzhao" => match m.stage {
            HostStage::Act => {
                ctx.core.turn.land_used = true;
                set_flag(ctx.core, owner, "mtg_land_played_this_turn", 1);
                switch_extra_land(ctx, 1);
                wait_ask_mtg(m, ctx, HostStage::TargetChoice)
            }
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    pull_du_card_to_player(ctx, t);
                }
                if let Some(c) = m.card {
                    ctx.core.table.retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mtg_duzhao_skill" => {
            pull_du_card_to_player(ctx, owner);
            Flow::Done
        }
        "mtg_cangbaohaiwan" | "g_mtg_cangbaohaiwan" => match m.stage {
            HostStage::Act => {
                ctx.core.turn.land_used = true;
                set_flag(ctx.core, owner, "mtg_land_played_this_turn", 1);
                switch_extra_land(ctx, 2);
                wait_ask_mtg(m, ctx, HostStage::SelectNature)
            }
            HostStage::SelectNature => {
                let tag = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(t)) => t,
                    _ => 0,
                };
                let pick = if tag == 0 {
                    ctx.core
                        .deck
                        .iter()
                        .copied()
                        .find(|&c| ctx.catalog.card_kind(c).equip_slot().is_some())
                } else {
                    ctx.core
                        .deck
                        .iter()
                        .copied()
                        .find(|&c| ctx.catalog.card_kind(c).is_trick_kind())
                }
                .or_else(|| ctx.core.deck.last().copied());
                if let Some(cid) = pick {
                    ctx.core.deck.retain(|&x| x != cid);
                    ctx.core.player_mut(owner).hand.push(cid);
                    ctx.log(LogEvent::CardsDrawn {
                        player: owner,
                        cards: vec![cid],
                    });
                }
                if let Some(c) = m.card {
                    ctx.core.table.retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mtg_cangbaohaiwan_skill" => {
            if ctx.core.rng.gen_range(3) == 0 && ctx.core.deck.len() > 1 {
                if let Some(pos) = ctx.core.deck.iter().position(|&c| !ctx.catalog.card_kind(c).is_basic_kind()) {
                    let cid = ctx.core.deck.remove(pos);
                    ctx.core.deck.push(cid);
                }
            }
            Flow::Done
        }
        "mtg_lindixiliu_skill" => match m.stage {
            HostStage::Act => wait_ask_mtg(m, ctx, HostStage::SelectCard),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(heart_c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &[heart_c]);
                if ctx.core.player(owner).judge.is_empty() {
                    return Flow::Done;
                }
                if ctx.core.player(owner).judge.len() == 1 {
                    let jc = ctx.core.player_mut(owner).judge.remove(0);
                    ctx.to_discard(jc);
                    ctx.log(LogEvent::CardDiscarded {
                        player: owner,
                        cards: vec![jc],
                    });
                    return Flow::Done;
                }
                wait_ask_mtg(m, ctx, HostStage::CardsAsk)
            }
            HostStage::CardsAsk => {
                if let Some(ChoiceOption::Card(jc)) = ctx.answer().first().copied() {
                    if let Some(pos) = ctx.core.player(owner).judge.iter().position(|&x| x == jc) {
                        ctx.core.player_mut(owner).judge.remove(pos);
                        ctx.to_discard(jc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: owner,
                            cards: vec![jc],
                        });
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mtg_bingheyaosai_skill" => match m.stage {
            HostStage::Act => {
                m.card = Some(1);
                wait_ask_mtg(m, ctx, HostStage::SelectCard)
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_cards(ctx, owner, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
