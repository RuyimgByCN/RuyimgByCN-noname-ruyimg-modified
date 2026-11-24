//! Batch Q4 U02/L03 boss/jiange 技能内核修饰器逻辑。
//!
//! 覆盖：`boss_biantian`（大雾/狂风）、`boss_yuhuojg`、`boss_chiying`、`boss_jingfan`、
//! `boss_aozhan`、`boss_xiaorui`、`boss_fengjian`、`boss_huchen`、`shenji`。

use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

#[inline]
fn flag_val(state: &GameStateCore, player: PlayerId, key: &str) -> u32 {
    if state.vm_aux.flags.is_empty() {
        return 0;
    }
    state
        .vm_aux
        .flags
        .get(&(player, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn has_ext_skill(state: &GameStateCore, catalog: &Catalog, player: PlayerId, name: &str) -> bool {
    if catalog.skills.len() <= 24 || catalog.is_tieji_sealed(state, player) {
        return false;
    }
    catalog
        .skill_by_name(name)
        .is_some_and(|def| state.has_skill(player, def))
}

/// 狂风（`boss_biantian3`）：来自 `boss_biantian` 红色判定，受到火焰伤害时 +1（`damageBegin3`）。
pub fn apply_damage_begin3(
    state: &GameStateCore,
    nature: DamageNature,
    target: PlayerId,
    mut cur: i32,
) -> i32 {
    if nature == DamageNature::Fire && flag_val(state, target, "q4_biantian_kuangfeng") > 0 {
        cur += 1;
    }
    cur
}

/// 大雾（`boss_biantian2`）、浴火（`boss_yuhuojg`）、持盈（`boss_chiying`）、鏖战防具（`boss_aozhan_fangju`）。
pub fn apply_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if cur <= 0 {
        return cur;
    }
    // 大雾：防止非雷电伤害（versus.js:7392）
    if flag_val(state, target, "q4_biantian_dawu") > 0 && nature != DamageNature::Thunder {
        return 0;
    }
    // 浴火：防止火焰伤害（versus.js:6730）
    if nature == DamageNature::Fire && has_ext_skill(state, catalog, target, "boss_yuhuojg") {
        return 0;
    }
    // 持盈：友方角色受到多于 1 点伤害时防止其余伤害（versus.js:6952）
    if cur > 1 {
        let chiying_active = (0..state.players.len() as PlayerId).any(|p| {
            state.player(p).alive
                && state.is_same_team_or_friend(catalog, p, target)
                && has_ext_skill(state, catalog, p, "boss_chiying")
        });
        if chiying_active {
            cur = 1;
        }
    }
    // 鏖战·防具：装备区有防具牌时至多受 1 点伤害（boss.js:4413）
    if cur > 1
        && state.player(target).equips.iter().any(|c| {
            catalog.card_slot_of(state, *c) == Some(EquipSlot::Armor)
        })
        && catalog
            .skill_by_name("boss_aozhan")
            .is_some_and(|def| state.has_skill(target, def))
    {
        cur = 1;
    }
    cur
}

/// 惊帆（`boss_jingfan2`）：对齐 `versus.js:6973-6983` `globalFrom` 距离修辞。
pub fn apply_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
    mut cur: i32,
) -> i32 {
    if from == to || state.is_hostile_to(catalog, to, from) {
        return cur;
    }
    let has_other_jingfan = (0..state.players.len() as PlayerId).any(|p| {
        p != from
            && state.player(p).alive
            && state.is_same_team_or_friend(catalog, p, from)
            && has_ext_skill(state, catalog, p, "boss_jingfan")
    });
    if has_other_jingfan {
        cur -= 1;
    }
    cur
}

/// 骁锐（`boss_xiaorui2`）、神戟（`shenji`）、鏖战·武器（`boss_aozhan_wuqi`）：出杀次数加成。
pub fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    cur += flag_val(state, player, "skill_turn:q4_xiaorui_sha_plus") as i32;
    if has_ext_skill(state, catalog, player, "shenji") {
        cur += 1;
    }
    let has_weapon = state
        .player(player)
        .equips
        .iter()
        .any(|c| catalog.card_slot_of(state, *c) == Some(EquipSlot::Weapon));
    if has_weapon
        && catalog
            .skill_by_name("boss_aozhan")
            .is_some_and(|def| state.has_skill(player, def))
    {
        cur += 1;
    }
    cur
}

/// 神戟（`shenji`）：【杀】目标上限 +2。
pub fn apply_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut extra: i32,
) -> i32 {
    if kind.is_sha_like() && has_ext_skill(state, catalog, player, "shenji") {
        extra += 2;
    }
    extra
}

/// 封缄（`boss_fengjian2`）：受到其伤害后于其下个回合结束前无法用牌指定来源为目标。
pub fn apply_card_target_canceled(
    state: &GameStateCore,
    actor: PlayerId,
    target: PlayerId,
    canceled: bool,
) -> bool {
    if canceled {
        return true;
    }
    flag_val(state, actor, &format!("q4_fengjian_ban:{target}")) > 0
}

/// 鏖战·宝物（`boss_aozhan_baowu`）：装备区有宝物牌时跳过判定阶段（boss.js:4435）。
pub fn skips_judge_phase(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    let has_treasure = state
        .player(player)
        .equips
        .iter()
        .any(|c| catalog.card_slot_of(state, *c) == Some(EquipSlot::Treasure));
    has_treasure
        && catalog
            .skill_by_name("boss_aozhan")
            .is_some_and(|def| state.has_skill(player, def))
}

/// 鏖战·坐骑（`boss_aozhan_zuoji`）与虎臣（`boss_huchen`）：摸牌阶段加摸数。
pub fn draw_phase_delta(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> i32 {
    let mut delta = 0;
    let has_mount = state.player(player).equips.iter().any(|c| {
        matches!(
            catalog.card_slot_of(state, *c),
            Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
        )
    });
    if has_mount
        && catalog
            .skill_by_name("boss_aozhan")
            .is_some_and(|def| state.has_skill(player, def))
    {
        delta += 1;
    }
    if has_ext_skill(state, catalog, player, "boss_huchen") {
        delta += flag_val(state, player, "q4_huchen_kills") as i32;
    }
    delta
}

/// `L26` `minsi2`（`character/sp2.js:11925-11934`）：本次敏思摸到的红色牌本回合不计入手牌上限且弃牌阶段不可弃。
pub fn is_minsi2_red_card(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    flag_val(state, player, &format!("q4_minsi2_card:{card}")) > 0
        && catalog
            .cards
            .get(card as usize)
            .is_some_and(|c| matches!(c.suit, Suit::Heart | Suit::Diamond))
}

/// `L26` `minsi2`（`character/sp2.js:11914-11924`）：本次敏思摸到的黑色牌本回合无距离限制。
pub fn apply_minsi2_distance_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    unlimited: bool,
) -> bool {
    if unlimited {
        return true;
    }
    flag_val(state, player, &format!("q4_minsi2_card:{card}")) > 0
        && catalog
            .cards
            .get(card as usize)
            .is_some_and(|c| matches!(c.suit, Suit::Spade | Suit::Club))
}
