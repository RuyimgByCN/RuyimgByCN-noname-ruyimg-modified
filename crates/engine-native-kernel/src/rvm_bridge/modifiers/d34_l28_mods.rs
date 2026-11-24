//! L28 雷霆万钧（`th_bushi`、`th_jingxie` 强化装备、`th_tiansuan2_3/4`）内核修饰器逻辑。
//! 自 `d34_flags.rs` 拆出以保持非测试源文件有效行数与职责边界清晰。

use crate::catalog::{CardKind, Catalog, Suit};
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
    if catalog.skills.len() <= 24 {
        return false;
    }
    catalog
        .skill_by_name(name)
        .is_some_and(|def| state.has_skill(player, def))
}

fn has_qinggang_or_zhadao(state: &GameStateCore, catalog: &Catalog, source: PlayerId) -> bool {
    state.has_active_equip_kind(catalog, source, CardKind::Qinggang)
        || has_ext_skill(state, catalog, source, crate::vocabulary::names::BOSS_ZHADAO)
}

pub fn effective_dmg_card_kind(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    card: Option<CardId>,
) -> Option<CardKind> {
    if let Some(c) = card {
        return Some(
            state
                .declared_kinds
                .get(&c)
                .copied()
                .unwrap_or_else(|| catalog.card_kind(c)),
        );
    }
    match flag_val(state, target, "l28_vdmg_kind") {
        1 => Some(CardKind::Sha),
        2 => Some(CardKind::SavageAttack),
        3 => Some(CardKind::ArcheryAttack),
        _ => None,
    }
}

pub fn source_ignores_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    kind: Option<CardKind>,
    nature: DamageNature,
) -> bool {
    if has_rewritten_armor(state, catalog, target, "lanyinjia") {
        return false;
    }
    let eff_kind = kind.or_else(|| effective_dmg_card_kind(state, catalog, target, card));
    let qinggang_applies =
        eff_kind.is_some_and(|k| k.is_sha_like()) && has_qinggang_or_zhadao(state, catalog, source);
    qinggang_applies || super::eval_ignore_armor(state, catalog, source, target, card, nature)
}

/// 上游 `th_tiansuan2_3`/`4`（`extension.js:8157-8215`）与 `rw_tengjia2` 均在 `damageBegin3` 结算，
/// 严格先于 `damageBegin4` 的伤害防止/封顶（如 `th_tiansuan2_1`/`2`、`rw_baiyin_skill`）。
pub fn apply_damage_begin3(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if flag_val(state, target, "th_tiansuan2_3_extra_damage") > 0
        || flag_val(state, target, "th_tiansuan2_4_extra_damage") > 0
    {
        cur += 1;
    }
    if nature == DamageNature::Fire
        && !super::is_target_unequip2(state, target)
        && has_rewritten_armor(state, catalog, target, "tengjia")
        && !source.is_some_and(|s| source_ignores_armor(state, catalog, s, target, card, None, nature))
    {
        cur += 1;
    }
    super::d34_q4_boss_mods::apply_damage_begin3(state, nature, target, cur)
}

/// 陆凯 `th_bushi`（`extension.js:4208, 4347-4352`）：初始/首槽花色（未重新分配前默认为黑桃）
/// 使用的任意牌均不计次数限制（含【杀】【酒】）。
pub fn bushi_unlimited_card(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
) -> bool {
    if !has_ext_skill(state, catalog, player, "th_bushi") {
        return false;
    }
    let Some(c) = card else {
        return false;
    };
    let s0 = if flag_val(state, player, "th_bushi_suits_assigned") > 0 {
        flag_val(state, player, "th_bushi_s0") % 4
    } else {
        0 // extension.js:4208 ['spade', 'diamond', 'club', 'heart']
    };
    let suit = catalog.effective_card_suit(state, player, c);
    let code = match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    };
    code == s0
}

#[inline]
pub fn is_jingxie_rewritten(state: &GameStateCore, card: CardId) -> bool {
    flag_val(state, 0, &format!("th_jingxie_rewritten:{card}")) > 0
}

pub fn has_rewritten_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    name: &str,
) -> bool {
    state.player(player).equips.iter().any(|&c| {
        !state.is_gw_dieyi_masked(catalog, player, c)
            && is_jingxie_rewritten(state, c)
            && catalog.cards[c as usize].name == name
    })
}

pub fn rewrite_bagua_judge_ok(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    suit: Suit,
) -> Option<bool> {
    if super::is_target_unequip2(state, target) {
        return None;
    }
    let has = state.player(target).equips.iter().any(|&c| {
        !state.is_gw_dieyi_masked(catalog, target, c)
            && is_jingxie_rewritten(state, c)
            && matches!(catalog.cards[c as usize].name, "armor" | "bagua")
    });
    has.then(|| !suit.is_spade())
}

pub fn rewrite_renwang_blocks(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    suit: Option<Suit>,
) -> bool {
    !super::is_target_unequip2(state, target)
        && has_rewritten_armor(state, catalog, target, "renwang")
        && matches!(suit, Some(Suit::Spade | Suit::Club | Suit::Heart))
}

pub fn rewrite_zhuge_range(state: &GameStateCore, catalog: &Catalog, card: CardId) -> Option<u32> {
    (is_jingxie_rewritten(state, card) && catalog.equip_card_has_kind(state, card, CardKind::Zhuge)).then_some(3)
}

/// 精械升级防具 `damageBegin4` 减免：`rewrite_tengjia` 免疫南蛮/万箭/普杀，`rewrite_baiyin` 伤封顶 1。
pub fn rewrite_armor_damage_mod(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if super::is_target_unequip2(state, target)
        || source.is_some_and(|s| source_ignores_armor(state, catalog, s, target, card, None, nature))
    {
        return cur;
    }
    let ckind = effective_dmg_card_kind(state, catalog, target, card);
    if has_rewritten_armor(state, catalog, target, "tengjia")
        && (matches!(ckind, Some(CardKind::SavageAttack | CardKind::ArcheryAttack))
            || (ckind.is_some_and(|k| k.is_sha_like()) && nature == DamageNature::Normal))
    {
        return 0;
    }
    if has_rewritten_armor(state, catalog, target, "baiyin") && cur > 1 {
        cur = 1;
    }
    if matches!(nature, DamageNature::Fire | DamageNature::Thunder)
        && has_ext_skill(state, catalog, target, "hongmianbaihuapao_skill")
    {
        return 0;
    }
    cur
}

pub fn rewrite_tengjia_blocks_link(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
) -> bool {
    !state.player(target).linked
        && !super::is_target_unequip2(state, target)
        && has_rewritten_armor(state, catalog, target, "tengjia")
}

pub fn apply_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    target: PlayerId,
    ignore: bool,
) -> bool {
    if ignore && has_rewritten_armor(state, catalog, target, "lanyinjia") {
        false
    } else {
        ignore
    }
}

pub fn has_rewritten_baiyin_on_lose(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    is_jingxie_rewritten(state, card)
        && catalog.cards[card as usize].name == "baiyin"
        && state.player(player).alive
        && !super::is_target_unequip2(state, player)
}

fn is_non_bagua_armor_name(name: &str) -> bool {
    matches!(
        name,
        "taipingyaoshu"
            | "th_taipingyaoshu"
            | "linglongshimandai"
            | "hongmianbaihuapao"
            | "qimenbagua"
            | "guofengyupao"
            | "huxinjing"
            | "heiguangkai"
            | "lanyinjia"
            | "tengjia"
            | "baiyin"
            | "gw_dieyi_equip2"
    )
}

/// Static catalog check for Bagua Shield dodge-judge semantics (used by public-view AI without `GameStateCore`).
pub fn is_bagua_armor_card(catalog: &Catalog, card: CardId) -> bool {
    let Some(def) = catalog.cards.get(card as usize) else {
        return false;
    };
    def.kind == crate::catalog::CardKind::Armor && !is_non_bagua_armor_name(def.name)
}

/// Returns whether `card` (or a `yanjia`-merged / `yange`-cloned equip) carries Bagua Shield dodge-judge semantics.
pub fn is_bagua_armor(state: &GameStateCore, catalog: &Catalog, card: CardId) -> bool {
    let is_single_bagua = |cid: CardId| -> bool {
        let Some(def) = catalog.resolved_card_def(state, cid) else {
            return false;
        };
        let kind = catalog.resolved_card_kind(state, cid);
        kind == crate::catalog::CardKind::Armor && !is_non_bagua_armor_name(def.name)
    };
    is_single_bagua(card) || catalog.yanjia_merged_card_id(state, card).is_some_and(is_single_bagua)
}

#[inline]
pub fn l28_turn_stamp(core: &GameStateCore) -> u32 {
    (core.turn.round.max(1) << 8) | u32::from(core.turn.current)
}

/// Tracks every physical card entering `core.discard` during a turn when any alive seat holds
/// `pro_qinyin`, matching upstream `game.getGlobalHistory('cardMove')` (`extra.js:8735`).
pub fn note_pro_qinyin_discard(core: &mut GameStateCore, catalog: &Catalog, card: CardId) {
    if catalog.skills.len() <= 24 || !core.discard.contains(&card) {
        return;
    }
    let stamp = l28_turn_stamp(core);
    if let Some(pq) = catalog.skill_by_name("pro_qinyin") {
        let any_alive = (0..core.players.len() as PlayerId)
            .any(|p| core.player(p).alive && core.has_skill(p, pq));
        if any_alive {
            core.vm_aux
                .flags
                .insert((0, format!("pro_qinyin_d:{card}")), stamp);
        }
    }
    if catalog.card_kind(card).is_sha_like() {
        if let Some(zl) = catalog.skill_by_name("dddzhilian") {
            let any_alive = (0..core.players.len() as PlayerId)
                .any(|p| core.player(p).alive && core.has_skill(p, zl));
            if any_alive {
                core.vm_aux
                    .flags
                    .insert((0, format!("dddzhilian_sha_d:{card}")), stamp);
            }
        }
    }
}

/// `rewrite_lanyinjia`（「精银甲」，`mobile.js:19548`）：你可以将一张手牌当做【闪】使用或打出；
/// 且锁定技「【精银甲】不会无效」（不受 `unequip2` 或青釭剑限制）。
pub fn has_rewritten_lanyinjia(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    catalog.skills.len() > 24 && has_rewritten_armor(state, catalog, player, "lanyinjia")
}

/// 鬼龙斩月刀（`sp.js:7949`）：锁定技，你使用的红色【杀】不可被响应（跳过八卦判定与出闪）。
pub fn is_guilong_red_sha(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    card: Option<CardId>,
) -> bool {
    has_ext_skill(state, catalog, source, "guilongzhanyuedao")
        && card.is_some_and(|c| {
            !crate::kernel::is_linyun_colorless(state, source, c)
                && matches!(
                    catalog.effective_card_suit(state, source, c),
                    Suit::Heart | Suit::Diamond
                )
        })
}

/// 神工宝物手牌上限修正：`xuwangzhimian` `-1`、`sanlve_skill` `+1`。
pub fn apply_shengong_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    if catalog.skills.len() <= 24 {
        return limit;
    }
    if has_ext_skill(state, catalog, player, "xuwangzhimian") {
        limit -= 1;
    }
    if has_ext_skill(state, catalog, player, "sanlve_skill") {
        limit += 1;
    }
    limit
}

/// 神工宝物攻击范围修正：`sanlve_skill` `+1`。
pub fn apply_shengong_attack_range(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    if catalog.skills.len() > 24 && has_ext_skill(state, catalog, player, "sanlve_skill") {
        cur += 1;
    }
    cur
}

/// 神工武器/宝物【杀】次数上限修正：`sanlve_skill` `+1`、`bintieshuangji_skill` 回合内 `+N`。
pub fn apply_shengong_sha_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
) -> i32 {
    if catalog.skills.len() > 24 && has_ext_skill(state, catalog, player, "sanlve_skill") {
        cur += 1;
    }
    if flag_val(state, player, "bintieshuangji_sha_stamp") == l28_turn_stamp(state) {
        cur += flag_val(state, player, "bintieshuangji_sha_bonus") as i32;
    }
    cur
}

/// 普通锦囊牌判定（非基本、非装备、非延时锦囊）。
#[inline]
pub fn is_ordinary_trick(kind: CardKind) -> bool {
    kind.is_basic_trick_or_equip()
        && !kind.is_basic_kind()
        && !kind.is_equip()
        && !matches!(
            kind,
            CardKind::Le
                | CardKind::Bing
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Guiyoujie
        )
}

#[inline]
fn is_damage_trick(catalog: &Catalog, card: Option<CardId>, kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Duel
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
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
    ) || card.is_some_and(|c| {
        catalog
            .cards
            .get(c as usize)
            .is_some_and(|d| matches!(d.name, "huogong" | "shuiyanqijun" | "shuiyanqijunx"))
    })
}

/// 国风玉袍（`sp.js:7757`）：锁定技，你不是其他角色使用普通锦囊牌的合法目标。
pub fn guofengyupao_cancels_target(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
) -> bool {
    actor != target
        && is_ordinary_trick(kind)
        && has_ext_skill(state, catalog, target, "guofengyupao")
        && !super::is_target_unequip2(state, target)
        && !source_ignores_armor(state, catalog, actor, target, card, Some(kind), DamageNature::Normal)
}

/// 奇门八卦 / 国风玉袍 / 黑光铠：在使用前/指定目标后将该目标移除（对持有者无效）。
pub fn shengong_excludes_target(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    nature: DamageNature,
    target_count: usize,
) -> bool {
    if catalog.skills.len() <= 24
        || super::is_target_unequip2(state, target)
        || source_ignores_armor(state, catalog, actor, target, card, Some(kind), nature)
    {
        return false;
    }
    if kind.is_sha_like() && has_ext_skill(state, catalog, target, "qimenbagua") {
        return true;
    }
    if actor != target && is_ordinary_trick(kind) && has_ext_skill(state, catalog, target, "guofengyupao") {
        return true;
    }
    target_count >= 2
        && has_ext_skill(state, catalog, target, "heiguangkai_skill")
        && (kind.is_sha_like()
            || (is_ordinary_trick(kind)
                && (is_damage_trick(catalog, card, kind)
                    || card.is_some_and(|c| {
                        matches!(
                            catalog.effective_card_suit(state, actor, c),
                            Suit::Spade | Suit::Club
                        )
                    }))))
}
