//! Phase 3 P3-C1a `L11` 跨包 NPC Part-A 规则修饰符与内核查询辅助（自 `d34_l11_mods.rs` 拆分）。

use crate::catalog::{CardKind, Catalog, Suit};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

#[inline]
pub(super) fn flag_val(state: &GameStateCore, player: PlayerId, key: &str) -> u32 {
    state
        .vm_aux
        .flags
        .get(&(player, key.to_string()))
        .copied()
        .unwrap_or(0)
}

#[inline]
pub fn has_npc_a_skill_unsealed(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    name: &str,
) -> bool {
    if catalog.skills.len() <= 24 {
        return false;
    }
    let Some(def) = catalog.skill_by_name(name) else {
        return false;
    };
    if !state.has_skill(player, def) || catalog.is_skill_chanyuan_blocked(state, player, def) {
        return false;
    }
    if !Catalog::is_locked_skill_name(name, catalog.skills[def as usize].trigger.as_ref())
        && (catalog.is_tieji_sealed(state, player)
            || catalog.is_xj_fengyin_sealed(state, player))
    {
        return false;
    }
    true
}

#[inline]
pub fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1 << 0,
        Suit::Heart => 1 << 1,
        Suit::Club => 1 << 2,
        Suit::Diamond => 1 << 3,
    }
}

/// `refresh.js:9638` (`rejinjiu`): 禁酒持有者自身的【酒】恒为【杀】（不可作【酒】饮用/自救），且其回合内其他角色亦不能使用/自救【酒】。
pub fn rejinjiu_blocks_jiu(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    let has_rejinjiu = |p: PlayerId| {
        flag_val(state, p, "l11_mod:rejinjiu") > 0
            || has_npc_a_skill_unsealed(state, catalog, p, "rejinjiu")
    };
    has_rejinjiu(player) || has_rejinjiu(state.turn.current)
}

/// `collab.js:2492` (`dclbjiuxian`): 酒仙持有者使用【酒】无次数限制。
pub fn dclbjiuxian_jiu_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    has_npc_a_skill_unsealed(state, catalog, player, "dclbjiuxian")
}

/// `shenhua.js:2142` (`nzry_chenglve`): 本回合与所弃花色相同的牌无距离与次数限制。
pub fn chenglve_card_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
) -> bool {
    let mask = flag_val(state, player, "l11_nzry_chenglve_suits");
    if mask == 0 {
        return false;
    }
    let Some(cid) = card else {
        return false;
    };
    let suit = catalog.effective_card_suit(state, player, cid);
    (mask & suit_bit(suit)) != 0
}

/// `diy.js:3461` (`kyou_zhidian`) / `mobile.js:16944` (`gnsheque`): 逐帧特定【杀】无视防具。
pub fn card_ignore_armor(state: &GameStateCore, source: PlayerId, card: Option<CardId>) -> bool {
    card.is_some_and(|c| flag_val(state, source, &format!("l11_card_ignore_armor:{c}")) > 0)
        || flag_val(state, source, "l11_frame_ignore_armor") > 0
}

/// `refresh.js:11585` (`new_yijue`): 非锁定技失效且不能使用或打出手牌。
pub fn is_new_yijue_hand_locked(state: &GameStateCore, player: PlayerId) -> bool {
    flag_val(state, player, "l11_new_yijue_hand_ban") > 0
}

/// `mobile.js:11007` (`jinglve`): `死士` 标记手牌不可被合法结算（或拦截）。
pub fn is_jinglve_sishi_card_locked(
    state: &GameStateCore,
    actor: PlayerId,
    card: CardId,
) -> bool {
    flag_val(state, actor, &format!("l11_jinglve_sishi_card:{card}")) > 0
}

/// `collab.js:2498` (`dclbjiuxian`): 非 `notarget` 且 `selectTarget != 1` 的多目标锦囊/延时锦囊。
pub fn is_multi_target_trick_for_jiuxian(kind: CardKind, card_name: &str) -> bool {
    !kind.is_basic_kind()
        && !kind.is_equip()
        && (matches!(
            kind,
            CardKind::Wugu
                | CardKind::Taoyuan
                | CardKind::SavageAttack
                | CardKind::ArcheryAttack
                | CardKind::Chain
                | CardKind::Jiedao
                | CardKind::Geanguanhuo
                | CardKind::Xianluhui
                | CardKind::Chiyuxi
                | CardKind::Jingleishan
        ) || matches!(
            card_name,
            "txhj_jiuchiroulin" | "txhj_yiyi" | "txhj_shuiyan" | "shuiyanqijun" | "shuiyanqijunx"
        ))
}
