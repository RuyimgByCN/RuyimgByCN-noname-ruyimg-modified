use super::super::*;

/// 判定锦囊是否由内核 `NullifyWindowMachine` 包裹：此类牌的 `_shengke` 须在无懈窗口
/// 确认未抵消后再触发（与上游 `_wuxie` `priority: 5` 先于 `_shengke` `priority: 0` 一致）。
fn is_nullify_wrapped_trick(
    catalog: &crate::catalog::Catalog,
    card: crate::ids::CardId,
    effective: CardKind,
    targets: &[PlayerId],
) -> bool {
    if effective.aoe_need().is_some() {
        return true;
    }
    match effective {
        CardKind::Wuzhong => !super::super::intrinsic::is_named_wuzhong_unwrapped(catalog, card),
        CardKind::Jiguan => {
            let raw = catalog.cards.get(card as usize).map(|c| c.name).unwrap_or("");
            !super::super::intrinsic::is_named_jiguan_unwrapped(raw)
        }
        CardKind::Taoyuan
        | CardKind::Wugu
        | CardKind::Dizaizhen
        | CardKind::ShanDian
        | CardKind::HongShui
        | CardKind::Shuigong
        | CardKind::Shezhanqunru
        | CardKind::Shujinsan
        | CardKind::Qinpendayu
        | CardKind::Birinongwu
        | CardKind::Ciguhanshuang
        | CardKind::Zumoshoukao
        | CardKind::Kunenfayin
        | CardKind::Shanbengshu
        | CardKind::Poxiao
        | CardKind::Yanziyaoshui
        | CardKind::Aozuzhilei
        | CardKind::Wenyi
        | CardKind::Zhihuanjun
        | CardKind::Baoxueyaoshui
        | CardKind::Shatang
        | CardKind::Heilonglinpian
        | CardKind::Liutouge
        | CardKind::Dunpaigedang
        | CardKind::Zhiliaobo
        | CardKind::Yuansuhuimie
        | CardKind::Xingjiegoutong
        | CardKind::Chuansongmen
        | CardKind::Tanshezhiren
        | CardKind::Shijieshu
        | CardKind::Shihuifen
        | CardKind::Liufengsan
        | CardKind::Bingpotong
        | CardKind::Qiankunbiao
        | CardKind::Longxugou
        | CardKind::Shenhuofeiya
        | CardKind::Liuxinghuoyu
        | CardKind::Hslingjian
        | CardKind::Changshezhen
        | CardKind::Pozhenjue
        | CardKind::Tianfuzhen
        | CardKind::Qixingzhen
        | CardKind::Niaoxiangzhen
        | CardKind::Yangpijuan
        | CardKind::Feibiao
        | CardKind::Chenhuodajie
        | CardKind::Fudichouxin
        | CardKind::WangmeiZhike
        | CardKind::Caochuanjiejian
        | CardKind::Geanguanhuo
        | CardKind::Linghunzhihuo
        | CardKind::Zhaomingdan
        | CardKind::Jihuocard
        | CardKind::Toulianghuanzhu => true,
        CardKind::Dismantle
        | CardKind::Snatch
        | CardKind::Duel
        | CardKind::Jiedao
        | CardKind::Chain
        | CardKind::Guiyoujie
        | CardKind::Le
        | CardKind::Bing => !targets.is_empty(),
        _ => false,
    }
}

/// 带帧 id 的 `_shengke` 入口：`EventData::Equipped` 仅认 `PlayCard` 使用装备，排除直接 `equip()`。
pub(super) fn shengke_matches_in_frame(
    ctx: &QueryCtx<'_>,
    frame_id: crate::ids::EventId,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if let EventData::Equipped { player, card, .. } = event {
        return ctx
            .state
            .vm_aux
            .flags
            .contains_key(&(*player, format!("wuxing_equip_use:{frame_id}")))
            && *player == owner
            && ctx.state.card_natures.contains_key(card)
            && ctx
                .state
                .vm_aux
                .flags
                .get(&(0, "connect_mode".to_string()))
                .copied()
                .unwrap_or(0)
                == 0
            && ctx.state.player(owner).alive
            && ctx.state.player(owner).wunature.is_some();
    }
    shengke_matches(ctx, owner, event)
}

/// 五行生克（`archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62-172`）：
/// 判定 `owner` 是否作为真实使用牌的目标触发 `_shengke`（`filter_common` 及无懈包裹跳过检查）。
pub(super) fn shengke_matches(ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
    if ctx
        .state
        .vm_aux
        .flags
        .get(&(0, "connect_mode".to_string()))
        .copied()
        .unwrap_or(0)
        != 0
        || !ctx.state.player(owner).alive
        || ctx.state.player(owner).wunature.is_none()
    {
        return false;
    }
    match event {
        EventData::CardUsed(u) => {
            if u.cancelled {
                return false;
            }
            let Some(card) = u.card else {
                return false;
            };
            if !ctx.state.card_natures.contains_key(&card) {
                return false;
            }
            let effective = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
            if is_nullify_wrapped_trick(ctx.catalog, card, effective, &u.targets) {
                // 由 `NullifyWindowMachine` 在无懈未抵消后补发 `_shengke`，保障 `_wuxie` (priority 5) 在先。
                return false;
            }
            if !u.targets.is_empty() {
                if effective == CardKind::Taoyuan
                    && ctx.state.player(owner).hp >= ctx.state.player(owner).max_hp as i32
                {
                    return false;
                }
                return u.targets.contains(&owner);
            }
            match effective {
                CardKind::Shezhanqunru => {
                    owner != u.player && !ctx.state.player(owner).hand.is_empty()
                }
                CardKind::Yuansuhuimie => owner != u.player,
                CardKind::Tao
                | CardKind::Wuzhong
                | CardKind::Jiguan
                | CardKind::Hufu
                | CardKind::Jiu
                | CardKind::Tianxianjiu
                | CardKind::Dinvxuanshuang
                | CardKind::Yangpijuan
                | CardKind::Qixingzhen
                | CardKind::Jihuocard
                | CardKind::Heilonglinpian
                | CardKind::Dunpaigedang
                | CardKind::Xingjiegoutong
                | CardKind::Chuansongmen
                | CardKind::Tanshezhiren
                | CardKind::Shijieshu
                | CardKind::Kunenfayin
                | CardKind::Poxiao
                | CardKind::Shanbengshu
                | CardKind::Xuanyuanjian => owner == u.player,
                _ => false,
            }
        }
        EventData::Equipped { .. } => false,
        _ => false,
    }
}

/// 生成 `_shengke` 相克强制弃置的 `he` 牌可选项：
/// 装备牌由 `EventData::Equipped` 的 `AfterTriggers` 触发时，刚刚装备入栏的那张牌在上游
/// `useCardToBegin` 时点尚在处理区（`ordering`）而非目标的 `he` 区，故从弃置候选中排除。
pub(in crate::content::host) fn shengke_discard_options(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    used_card: Option<crate::ids::CardId>,
) -> Vec<ChoiceOption> {
    if !core.player(owner).alive {
        return Vec::new();
    }
    let p = core.player(owner);
    p.hand
        .iter()
        .chain(p.equips.iter())
        .copied()
        .filter(|&c| !(Some(c) == used_card && catalog.card_kind(c).is_equip()))
        .map(ChoiceOption::Card)
        .collect()
}

/// 猴子偷桃（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:5-31`）：
/// 其他角色使用【桃】（不含已被取消或 0 实体虚拟桃）且处理区仍存实体载体、未曾在本帧问过时触发。
pub(super) fn monkey_matches_in_frame(
    ctx: &QueryCtx<'_>,
    frame_id: crate::ids::EventId,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let EventData::CardUsed(u) = event else {
        return false;
    };
    if u.cancelled
        || u.player == owner
        || !ctx.state.player(owner).alive
        || ctx
            .state
            .vm_aux
            .flags
            .contains_key(&(owner, format!("monkey_asked:{frame_id}")))
    {
        return false;
    }
    let effective = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
    if effective != CardKind::Tao {
        return false;
    }
    let has_monkey = ctx.state.player(owner).equips.iter().any(|c| {
        ctx.catalog.card_slot_of(ctx.state, *c) == Some(crate::catalog::EquipSlot::Treasure)
            && ctx.catalog.card_kind(*c) == CardKind::Monkey
    });
    if !has_monkey {
        return false;
    }
    u.card
        .into_iter()
        .chain(crate::kernel::peek_tao_extra_carriers(
            ctx.state, u.player, frame_id,
        ))
        .any(|c| ctx.state.table.contains(&c))
}
