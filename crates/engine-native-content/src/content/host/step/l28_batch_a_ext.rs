//! L28 雷霆万钧 Batch A 规则对齐扩展步进（`l28_batch_a_ext.rs`）：
//! `th_buxu`、`th_zhengjing`、`th_tianshu`、`th_taipingyaoshu`。

use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceOption;
use crate::content::host::{HostBehavior, HostMachine, HostStage};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

/// 延时锦囊牌判定（`extension.js:6217, 6229` `get.type(card, false) == 'delay'`）。
pub(super) fn is_delayed_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::Guiyoujie
    )
}

/// 技能移除回调（`onremove`）：清理武将牌标记牌或联动目标签。
pub(crate) fn on_skill_removed_l28(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    match skill_name {
        "th_chengye" => {
            set_flag(core, owner, "th_chengye_liujing_count", 0);
            set_flag(core, owner, "th_chengye_mask", 0);
            core.vm_aux
                .piles
                .remove(&(owner, "th_liujing".to_string()))
                .unwrap_or_default()
        }
        "th_zhengjing2" => {
            set_flag(core, owner, "th_zhengjing_boom", 0);
            core.vm_aux
                .piles
                .remove(&(owner, "th_jing".to_string()))
                .unwrap_or_default()
        }
        "th_tiansuan2" => {
            super::super::l28_batch_b::clear_tiansuan_lots_core(core, catalog, owner);
            Vec::new()
        }
        "txzhaoshu_skill" => {
            let mut out = core
                .vm_aux
                .piles
                .remove(&(owner, "txzhaoshu_skill".to_string()))
                .unwrap_or_default();
            out.extend(
                core.vm_aux
                    .piles
                    .remove(&(owner, "txzhaoshu_cards".to_string()))
                    .unwrap_or_default(),
            );
            out
        }
        "ssr_jinfan" => {
            set_flag(core, owner, "l25_jinfan_ling", 0);
            set_flag(core, owner, "mark:ssr_jinfan", 0);
            core.vm_aux
                .piles
                .remove(&(owner, "ssr_jinfan".to_string()))
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

/// 收集他人在弃牌堆中的牌按六经缺失槽收入 `th_liujing`（`extension.js:6203-6272`），
/// 并联动 `th_shengong_destroy` 移出游戏及 `pro_qinyin` 同回合入弃牌堆红黑记录。
pub(crate) fn on_cards_to_discard_chengye(
    ctx: &mut Ctx<'_>,
    actor: PlayerId,
    cards: &[CardId],
    from_use_card: bool,
) {
    super::l28_batch_a_shengong::intercept_shengong_destroy(ctx, cards);
    let seats = ctx.core.players.len() as PlayerId;
    if let Some(chengye_def) = ctx.catalog.skill_by_name("th_chengye") {
        let mut pool: Vec<CardId> = cards
            .iter()
            .copied()
            .filter(|&c| {
                ctx.core.discard.contains(&c)
                    && ctx.catalog.cards.get(c as usize).is_some_and(|d| {
                        from_use_card
                            || is_delayed_trick_kind(d.kind)
                            || d.kind.equip_slot().is_some()
                    })
            })
            .collect();
        if !pool.is_empty() {
            for owner in 0..seats {
                if owner == actor
                    || !ctx.core.player(owner).alive
                    || !ctx.core.has_skill(owner, chengye_def)
                {
                    continue;
                }
                let mut mask = flag(ctx.core, owner, "th_chengye_mask");
                if (mask & 0x3F) == 0x3F {
                    continue;
                }
                let mut added = false;
                for ch in 0..6u32 {
                    if (mask & (1 << ch)) != 0 {
                        continue;
                    }
                    if let Some(pos) = pool.iter().position(|&c| liujing_matches(ctx.catalog, c, ch))
                    {
                        let card = pool.remove(pos);
                        mask |= 1 << ch;
                        ctx.core.discard.retain(|&x| x != card);
                        ctx.core
                            .vm_aux
                            .piles
                            .entry((owner, "th_liujing".to_string()))
                            .or_default()
                            .push(card);
                        added = true;
                    }
                }
                if added {
                    set_flag(ctx.core, owner, "th_chengye_mask", mask);
                    set_flag(ctx.core, owner, "th_chengye_liujing_count", mask.count_ones());
                }
                if pool.is_empty() {
                    break;
                }
            }
        }
    }
    for &c in cards {
        crate::rvm_bridge::modifiers::d34_l28_mods::note_pro_qinyin_discard(
            ctx.core,
            ctx.catalog,
            c,
        );
    }
}

/// `get.cardPile` 同口径：先自牌堆顶向下搜寻，未命中再搜弃牌堆。
pub(super) fn pop_pile_matching(
    core: &mut GameStateCore,
    pred: impl Fn(CardId) -> bool,
) -> Option<CardId> {
    if let Some(pos) = core.deck.iter().rposition(|&c| pred(c)) {
        return Some(core.deck.remove(pos));
    }
    if let Some(pos) = core.discard.iter().rposition(|&c| pred(c)) {
        return Some(core.discard.remove(pos));
    }
    None
}

/// 自 `detached_cards` 实例化尚未进入 `inpile` 的扩展专属牌（如 `th_taipingyaoshu`）。
pub(super) fn pop_detached_matching(
    core: &mut GameStateCore,
    pred: impl Fn(CardId) -> bool,
) -> Option<CardId> {
    core.detached_cards
        .iter()
        .position(|&(c, _)| pred(c))
        .map(|pos| core.detached_cards.remove(pos).0)
}

/// `雷霆万钧/extension.js:6186-6195` `th_chengye.liujing_filter` 六经类别判定。
pub(super) fn liujing_matches(catalog: &Catalog, c: CardId, ch: u32) -> bool {
    let Some(def) = catalog.cards.get(c as usize) else {
        return false;
    };
    let kind = def.kind;
    let name = def.name;
    match ch {
        // 0: 伤害类非延时锦囊 (`get.type(c, false) == 'trick' && get.tag(c, 'damage') > 0`)
        0 => {
            matches!(
                kind,
                CardKind::Duel
                    | CardKind::ArcheryAttack
                    | CardKind::SavageAttack
                    | CardKind::Linghunzhihuo
                    | CardKind::Jingleishan
                    | CardKind::Chiyuxi
                    | CardKind::Bingpotong
                    | CardKind::Qiankunbiao
                    | CardKind::Shenhuofeiya
                    | CardKind::Yuansuhuimie
                    | CardKind::Tanshezhiren
                    | CardKind::Aozuzhilei
                    | CardKind::Baoxueyaoshui
                    | CardKind::Qinpendayu
                    | CardKind::Birinongwu
                    | CardKind::Poxiao
            ) || matches!(name, "huogong" | "shuiyanqijun" | "shuiyanqijunx")
        }
        // 1: 基本牌 (`get.type(c, false) == 'basic'`)
        1 => kind.is_basic_kind(),
        // 2: 无懈可击 (`get.name(c, false) == 'wuxie'`)
        2 => kind == CardKind::Nullify || name == "wuxie",
        // 3: 无中生有 (`get.name(c, false) == 'wuzhong'`)
        3 => kind == CardKind::Wuzhong || name == "wuzhong",
        // 4: 乐不思蜀 (`get.name(c, false) == 'lebu'`)
        4 => kind == CardKind::Le || name == "lebu",
        // 5: 装备牌 (`get.type(c, false) == 'equip'`)
        5 => kind.equip_slot().is_some(),
        _ => false,
    }
}

/// 检查场上所有存活角色的手牌/装备/判定/武将牌上标记牌区是否已有【太平要术】。
pub(in crate::content::host) fn taipingyaoshu_in_hejsx(
    core: &GameStateCore,
    catalog: &Catalog,
) -> bool {
    for (pid, p) in core.players.iter().enumerate() {
        if !p.alive {
            continue;
        }
        if p.hand
            .iter()
            .chain(p.equips.iter())
            .chain(p.judge.iter())
            .any(|&c| is_taipingyaoshu(catalog, c))
        {
            return true;
        }
        let seat = pid as PlayerId;
        if core
            .vm_aux
            .piles
            .iter()
            .any(|((owner, _), cards)| *owner == seat && cards.iter().any(|&c| is_taipingyaoshu(catalog, c)))
        {
            return true;
        }
    }
    false
}

pub(super) fn equip_slot_usable(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    card: CardId,
) -> bool {
    catalog
        .card_slot_of(core, card)
        .is_some_and(|slot| crate::rvm_ops::has_enabled_equip_slot(core, p, slot))
}

/// `雷霆万钧/extension.js:6372-6514` (`th_buxu`)：按所选六经类别先后搜牌堆/弃牌堆，均无时退还本次次数。
pub(super) fn step_th_buxu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_BUXU_DISCARD),
        HostStage::SelectCard => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            if cards.is_empty() {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            discard_cards(ctx, owner, &cards);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_BUXU_CHAMBER)
        }
        HostStage::SelectNature => {
            let ch = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => u32::from(t) % 6,
                _ => 0,
            };
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let stamp = turn_stamp(ctx.core);
            let used = if flag(ctx.core, owner, "th_buxu_turn_stamp") == stamp {
                flag(ctx.core, owner, "th_buxu_used_turn")
            } else {
                0
            };
            if let Some(c) = pop_pile_matching(ctx.core, |cid| liujing_matches(ctx.catalog, cid, ch))
            {
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "th_liujing".to_string()))
                    .or_default()
                    .push(c);
                let mask = flag(ctx.core, owner, "th_chengye_mask") | (1 << ch);
                let cnt = mask.count_ones();
                set_flag(ctx.core, owner, "th_chengye_mask", mask);
                set_flag(ctx.core, owner, "th_chengye_liujing_count", cnt);
                set_flag(ctx.core, owner, "th_buxu_turn_stamp", stamp);
                set_flag(ctx.core, owner, "th_buxu_used_turn", used + 1);
                set_flag(ctx.core, owner, "th_buxu_added_classic", 1);
            } else {
                // 牌堆与弃牌堆均无对应经牌：不添典、不置位并退还本次发动次数。
                set_flag(ctx.core, owner, "th_buxu_turn_stamp", stamp);
                set_flag(ctx.core, owner, "th_buxu_used_turn", used);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `雷霆万钧/extension.js:475-549, 6018-6155` (`th_zhengjing`)：
/// `count == 0` / boom 时不添经不摸牌；`>0` 亮出不重复非毒牌，先选目标再自选作经子集。
pub(super) fn step_th_zhengjing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_ZHENGJING_COUNT),
        HostStage::SelectNature => {
            let count = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n),
                _ => 1,
            };
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            set_flag(ctx.core, owner, "th_zhengjing_sorted", 1);
            if count == 0 || flag(ctx.core, owner, "th_zhengjing_force_boom") > 0 {
                set_flag(ctx.core, owner, "th_zhengjing_boom", 1);
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "th_zhengjing_boom", 0);
            let mut excavated = Vec::new();
            let mut names: Vec<&'static str> = Vec::new();
            for _ in 0..count.clamp(1, 5) {
                let picked = pop_pile_matching(ctx.core, |cid| {
                    let Some(def) = ctx.catalog.cards.get(cid as usize) else {
                        return false;
                    };
                    def.name != "du" && !names.contains(&def.name)
                });
                let Some(c) = picked else {
                    break;
                };
                names.push(ctx.catalog.cards[c as usize].name);
                excavated.push(c);
            }
            if excavated.is_empty() {
                return Flow::Done;
            }
            ctx.core.table.extend(excavated.iter().copied());
            m.cards = excavated;
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHENGJING_TARGET)
        }
        HostStage::TargetChoice => {
            let t = match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(p)) => p,
                _ => owner,
            };
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            m.target = Some(t);
            if m.cards.len() <= 1 {
                let shown = std::mem::take(&mut m.cards);
                ctx.core.table.retain(|c| !shown.contains(c));
                if let Some(&only) = shown.first() {
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((t, "th_jing".to_string()))
                        .or_default()
                        .push(only);
                    grant_skill_named(ctx, t, "th_zhengjing2");
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHENGJING_CARDS)
        }
        HostStage::SelectCard => {
            let t = m.target.unwrap_or(owner);
            let shown = std::mem::take(&mut m.cards);
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            let mut chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) if shown.contains(&c) => Some(c),
                    _ => None,
                })
                .collect();
            if chosen.is_empty() {
                if let Some(&first) = shown.first() {
                    chosen.push(first);
                }
            }
            ctx.core.table.retain(|c| !shown.contains(c));
            if !chosen.is_empty() {
                ctx.core
                    .vm_aux
                    .piles
                    .entry((t, "th_jing".to_string()))
                    .or_default()
                    .extend(chosen.iter().copied());
                grant_skill_named(ctx, t, "th_zhengjing2");
            }
            for c in shown {
                if !chosen.contains(&c) {
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `雷霆万钧/extension.js:6914-6959` (`th_tianshu`)：
/// 先选目标再弃牌（皆可取消无损耗），弃后沿牌堆→弃牌堆寻实体【太平要术】，入目标手牌后若合法则使用装备。
pub(super) fn step_th_tianshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if taipingyaoshu_in_hejsx(ctx.core, ctx.catalog)
                || hand_and_equips(ctx.core, owner).is_empty()
            {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_TIANSHU_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_TIANSHU_DISCARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            discard_cards(ctx, owner, &[c]);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let t = m.target.unwrap_or(owner);
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            if let Some(tp_cid) = pop_pile_matching(ctx.core, |c| is_taipingyaoshu(ctx.catalog, c))
                .or_else(|| pop_detached_matching(ctx.core, |c| is_taipingyaoshu(ctx.catalog, c)))
            {
                ctx.core.player_mut(t).hand.push(tp_cid);
                set_flag(ctx.core, owner, "th_tianshu_granted_yaoshu", 1);
                if equip_slot_usable(ctx.core, ctx.catalog, t, tp_cid)
                    && ctx.core.player(t).hand.contains(&tp_cid)
                {
                    ctx.emit(EventData::Equipped {
                        player: t,
                        card: tp_cid,
                        replaced: None,
                    });
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `phaseUseBegin` 钩子（`extension.js:6914`）：拥有 `th_tianshu` 且场上无【太平要术】且有牌可弃时入队提问。
pub(crate) fn play_begin_tianshu(ctx: &mut Ctx<'_>, player: PlayerId) {
    let Some(def) = ctx.catalog.skill_by_name("th_tianshu") else {
        return;
    };
    if !ctx.core.has_skill(player, def) || !ctx.core.player(player).alive {
        return;
    }
    set_flag(ctx.core, player, "th_tianshu_entered_play", turn_stamp(ctx.core));
    if taipingyaoshu_in_hejsx(ctx.core, ctx.catalog) || hand_and_equips(ctx.core, player).is_empty()
    {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "th_tianshu".to_string(),
            behavior: HostBehavior::L28ThunderBatchA,
            owner: player,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
}

/// 外层事件结算完毕钩子（`extension.js:8988-9016` `th_taipingyaoshu` 与 `mobile.js:502` `rewrite_baiyin`）。
pub(crate) fn on_outer_settled_taipingyaoshu(ctx: &mut Ctx<'_>) {
    let seats = ctx.core.players.len() as PlayerId;
    let mut push_pending = |p: PlayerId, key: &'static str, def_key: &'static str| {
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(p, key.to_string()))
            .is_some()
            && ctx.core.player(p).alive
        {
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: def_key.to_string(),
                    behavior: HostBehavior::L28ThunderBatchA,
                    owner: p,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
            });
        }
    };
    for p in 0..seats {
        push_pending(p, "l28_taipingyaoshu_pending_lose", "th_taipingyaoshu");
        push_pending(p, "l28_rw_baiyin_pending_lose", "l28_rw_baiyin_lose");
        push_pending(p, "l28_tianjitu_pending_lose", "l28_tianjitu_lose");
    }
}

/// `rewrite_baiyin` 离开装备区（`mobile.js:502`）：回复 1 点体力，然后摸 2 张牌。
pub(super) fn step_rw_baiyin_lose(_m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = _m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
        ctx.recover(owner, 1, None);
    }
    ctx.start_draw(owner, 2);
    Flow::Done
}

/// `th_taipingyaoshu` `onLosex` 两段串行结算：先摸 2 牌并 Yield，待摸牌帧完成后再判断 `hp > 1` 失 1 血。
pub(super) fn step_th_taipingyaoshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "th_taipingyaoshu_lost_effect", 1);
            ctx.start_draw(owner, 2);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive && ctx.core.player(owner).hp > 1 {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
