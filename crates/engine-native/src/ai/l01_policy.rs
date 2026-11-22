//! L01 `AI优化` 全量公开信息集策略（对应 `AI优化/extension.js`）。
//!
//! 严守 N10 信息边界：所有评估仅读 `PlayerView` / `PolicyInput` 与本座可见信息，
//! 绝不偷看未明置对手身份牌或暗手牌。

use super::extension_choices::public_attitude;
use super::{BoundedPlanner, ContentTuning};
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::choice::ChoiceOption;
use crate::ids::CardId;
use crate::state::Role;
use crate::view::{ChoiceView, PlayerCellView, PlayerView, PolicyInput};
use crate::{Engine, GameAction};

#[path = "l01_reserved_shan.rs"]
mod l01_reserved_shan;
pub use l01_reserved_shan::sync_l01_public_runtime;

/// L01 策略家族（对应上游 `btAi` / `sfjAi`：`shyg` 山河永固、`gjcx` 官将重修、`off` 关闭）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum L01StrategyMode {
    Off,
    #[default]
    Shyg,
    Gjcx,
}

/// AI 不砍队友阈值（对应 `nhFriends`：`off` / `1..=4` / `hp`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NhFriendsMode {
    #[default]
    Off,
    MaxHp(i32),
    SelfHp,
}

/// 稀有度分级（用于 `chaofeng_*` 与 `applyCf = pj` 威胁度公式）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralRarity {
    Common,
    Rare,
    Epic,
    Legend,
    Junk,
}

/// L01 `AI优化` 规划器策略配置（全 27 项含规则配置中与规划评估相关的控制面）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct L01AiOptions {
    pub bt_ai: L01StrategyMode,
    pub sfj_ai: L01StrategyMode,
    pub zb_ai: bool,
    pub wj_ai: bool,
    pub nh_friends: NhFriendsMode,
    pub nt_aoe: bool,
    pub mj_ai: bool,
    pub hsyh: bool,
    pub xyyh: bool,
    pub find_zhong: bool,
    pub attceshi: bool,
    pub qz_cf: bool,
    pub apply_cf_auto: bool,
    /// `0=off`, `1=pj`, `2=cf`
    pub ck_qz_mode: u8,
    /// 仅当观察者为主公且开启 `findZhong` 时注入的一名已知忠臣座位。
    pub lord_known_loyalist: Option<u8>,
    /// 由公开事件流累计的座位态度修正 `[seat -> signed score]`。
    pub public_attitudes: Vec<(u8, i32)>,
    /// 武将稀有度提示 `[general_name -> rarity]`。
    pub general_rarities: Vec<(String, GeneralRarity)>,
    /// 场上活跃 `twzhian`（是否可能触发妨碍延时/装备）。
    pub twzhian_active_enemy: bool,
    /// 场上是否已觉醒 `dcwumei_wake`（全局抑制回血、压低伤害收益）。
    pub dcwumei_active: bool,
    /// 配置静态武将权重表（`qz`：`[general_name -> weight]`）。
    pub general_weights: Vec<(String, f64)>,
    /// 配置静态技能威胁度表（`cf`：`[skill_name -> threat]`）。
    pub skill_threats: Vec<(String, f64)>,
    /// 当前处于 `zhenlie_lose` 濒死增益窗的公开座位。
    pub zhenlie_lose_seats: Vec<u8>,
    /// 当前出牌阶段是否处于酒增益（`_aiyh_reserved_shan` 防致命闪判断）。
    pub jiu_active: bool,
    /// 当前响应闪询问是否来自【杀】（`_aiyh_reserved_shan` 仅对杀留闪）。
    pub respond_shan_is_sha: bool,
    /// 当前响应【杀】是否带火属性（防 `tengjia` 2 伤）。
    pub respond_shan_fire: bool,
    /// 当前响应【杀】攻击者座位（公开）。
    pub respond_shan_attacker: Option<u8>,
    /// 攻击者当前出牌阶段是否仍存在可用的剩余【杀】额度（`getCardUsable('sha') >= 1`）。
    pub attacker_sha_usable: bool,
}

impl BoundedPlanner {
    /// 注入 L01 `AI优化` 规划策略选项（仅依赖观察者公开信息集）。
    pub fn with_l01_options(mut self, opts: L01AiOptions) -> Self {
        self.l01_options = Some(opts);
        self
    }
}

/// 按公开信息推导 `observer` 对 `target` 的态度（正=友方，0=未知/中立，负=敌方）。
pub fn l01_public_attitude(
    view: &PlayerView,
    opts: Option<&L01AiOptions>,
    seat: u8,
    target: u8,
) -> i32 {
    if target == seat {
        return 10;
    }
    if let (Some(my_team), Some(target_team)) = (
        view.players.get(seat as usize).and_then(|p| p.team),
        view.players.get(target as usize).and_then(|p| p.team),
    ) {
        return if my_team == target_team { 6 } else { -5 };
    }
    if let Some(o) = opts {
        if o.lord_known_loyalist == Some(target)
            && view.players.get(seat as usize).and_then(|p| p.role) == Some(Role::Lord)
        {
            return 6;
        }
    }
    let my_role = view.players.get(seat as usize).and_then(|p| p.role);
    let target_role = view.players.get(target as usize).and_then(|p| p.role);
    if let Some(tr) = target_role {
        return match my_role {
            Some(Role::Lord) | Some(Role::Loyalist) => match tr {
                Role::Lord | Role::Loyalist => 6,
                Role::Rebel | Role::Traitor => -5,
            },
            Some(Role::Rebel) => match tr {
                Role::Lord | Role::Loyalist => -6,
                Role::Rebel => 6,
                Role::Traitor => -3,
            },
            Some(Role::Traitor) => match tr {
                Role::Lord => 1,
                Role::Loyalist | Role::Rebel => -2,
                Role::Traitor => 0,
            },
            None => public_attitude(view, seat, target),
        };
    }
    if let Some(o) = opts {
        if let Some((_, score)) = o.public_attitudes.iter().find(|(s, _)| *s == target) {
            let signed = match my_role {
                Some(Role::Rebel) => -*score,
                _ => *score,
            };
            if signed > 100 {
                return 4;
            }
            if signed < -100 {
                return -4;
            }
        }
    }
    0
}

pub(super) fn has_equip_named(catalog: &Catalog, p: &PlayerCellView, key_substr: &str) -> bool {
    p.equips.iter().any(|c| {
        catalog
            .cards
            .get(*c as usize)
            .is_some_and(|d| d.name.contains(key_substr))
    })
}

fn has_armor_slot(catalog: &Catalog, p: &PlayerCellView) -> bool {
    p.equips.iter().any(|c| {
        catalog
            .cards
            .get(*c as usize)
            .and_then(|d| d.kind.equip_slot())
            == Some(EquipSlot::Armor)
    })
}

fn has_mount_slot(catalog: &Catalog, p: &PlayerCellView) -> bool {
    p.equips.iter().any(|c| {
        matches!(
            catalog
                .cards
                .get(*c as usize)
                .and_then(|d| d.kind.equip_slot()),
            Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
        )
    })
}

fn is_single_damage_play(kind: CardKind) -> bool {
    kind.is_sha_like() || matches!(kind, CardKind::Duel)
}

fn is_aoe_damage_play(kind: CardKind) -> bool {
    matches!(kind, CardKind::SavageAttack | CardKind::ArcheryAttack)
}

fn rarity_threat_all(rarity: GeneralRarity) -> Option<f64> {
    match rarity {
        GeneralRarity::Rare => Some(1.2),
        GeneralRarity::Epic => Some(2.0),
        GeneralRarity::Legend => Some(2.4),
        GeneralRarity::Junk => Some(0.8),
        GeneralRarity::Common => None,
    }
}

/// 上游 `applyCf == 'pj'` 威胁度自动分配（`AI优化/extension.js:23-56`）：
/// `th = Math.pow(all, 1 / skills.len())`。
pub fn l01_auto_skill_threat(rarity: GeneralRarity, skill_count: usize, is_maixie_defend: bool) -> f64 {
    let Some(all) = rarity_threat_all(rarity) else {
        return 1.0;
    };
    if skill_count == 0 {
        return 1.0;
    }
    let th = all.powf(1.0 / (skill_count as f64));
    if is_maixie_defend {
        0.8 * th
    } else {
        th
    }
}

/// 上游 `_wanjunpiyi` 嘲讽威胁度等级（`AI优化/extension.js:3973-4033`）。
pub fn chaofeng_threat(rarity: GeneralRarity) -> f64 {
    match rarity {
        GeneralRarity::Common | GeneralRarity::Junk => 1.0,
        GeneralRarity::Rare => 2.0,
        GeneralRarity::Epic => 3.0,
        GeneralRarity::Legend => 4.0,
    }
}

/// 为当前规划候选动作计算 L01 启发式评分加减项（仅读观察者公开视角）。
#[allow(clippy::too_many_arguments)]
pub(super) fn score_l01_candidate(
    base: &PlayerView,
    after: &PlayerView,
    sim: &Engine,
    seat: u8,
    action: &GameAction,
    opts: &L01AiOptions,
    tuning: Option<&ContentTuning>,
) -> i64 {
    let catalog = sim.catalog();
    let mut delta: i64 = 0;
    let me = &base.players[seat as usize];
    let my_role = me.role;

    // 1. `_wanjunpiyi` 稀有度嘲讽权重、`applyCf = pj`、静态/局内微调（`_aiyh_fixEQz` / `_aiyh_fixECf`）
    let storage = &sim.state_core().vm_aux.storage;
    for (pb, pa) in base.players.iter().zip(after.players.iter()) {
        if pa.seat == seat {
            continue;
        }
        let hp_loss = (pb.hp - pa.hp).max(0) as f64;
        if hp_loss <= 0.0 {
            continue;
        }
        let att = l01_public_attitude(base, Some(opts), seat, pa.seat);
        let sign = if att <= 0 { 1.0 } else { -1.0 };
        let visible_gens: Vec<&str> = pa.generals.iter().flatten().map(|g| g.name).collect();
        for gname in &visible_gens {
            if let Some((_, rar)) = opts.general_rarities.iter().find(|(n, _)| n == gname) {
                let cf = chaofeng_threat(*rar);
                if opts.apply_cf_auto {
                    let auto_th = l01_auto_skill_threat(*rar, pa.skills.len(), false);
                    delta += (sign * hp_loss * (cf + auto_th) * 2.0).round() as i64;
                } else {
                    delta += (sign * hp_loss * cf).round() as i64;
                }
            }
        }
        let base_w = tuning
            .map(|t| t.weight_of(pa.seat))
            .unwrap_or_else(|| {
                visible_gens
                    .iter()
                    .filter_map(|gname| {
                        opts.general_weights
                            .iter()
                            .find(|(n, _)| n == gname)
                            .map(|(_, v)| *v)
                    })
                    .fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
                    .or_else(|| {
                        (opts.ck_qz_mode == 2).then(|| {
                            pa.skills
                                .iter()
                                .filter_map(|sk| {
                                    opts.skill_threats
                                        .iter()
                                        .find(|(k, _)| k == sk)
                                        .map(|(_, v)| *v)
                                })
                                .fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
                        })?
                    })
                    .unwrap_or(1.0)
            });
        let mut rt_qz_mult = 1.0f64;
        for gname in &visible_gens {
            if let Some(&pct) = storage.get(&format!("l01_rt_qz_pct:{gname}")) {
                rt_qz_mult *= (pct as f64) / 100.0;
            }
        }
        let eff_w = (base_w * rt_qz_mult).clamp(0.05, 100.0);
        if opts.qz_cf || (rt_qz_mult - 1.0).abs() > 1e-6 {
            delta += (sign * hp_loss * (eff_w - 1.0) * 6.0).round() as i64;
        }
        for sk in &pa.skills {
            let static_cf = opts
                .skill_threats
                .iter()
                .find(|(k, _)| k == sk)
                .map(|(_, v)| *v)
                .or_else(|| {
                    tuning.and_then(|t| {
                        t.skill_threat
                            .iter()
                            .find(|(k, _)| k == sk)
                            .map(|(_, v)| *v)
                    })
                });
            let rt_cf_pct = storage.get(&format!("l01_rt_cf_pct:{sk}")).copied();
            if static_cf.is_some() || rt_cf_pct.is_some() {
                let eff_cf = static_cf.unwrap_or(1.0) * (rt_cf_pct.unwrap_or(100) as f64 / 100.0);
                delta += (sign * hp_loss * (eff_cf - 1.0) * 6.0).round() as i64;
            }
        }
    }

    // 2. 出牌候选专用战略评估（`nhFriends` / `ntAoe` / `_shyg_qjAi` / `_gjcx_correct` / `_aiyh_wqjz` / `mjAi` / `sfjAi`）
    if let GameAction::PlayCard { card, targets } = action {
        let kind = catalog.card_kind(*card);
        let def_opt = catalog.cards.get(*card as usize);
        let suit = def_opt.map(|d| d.suit);

        // `_aiyh_wqjz`: 装备对位敌方弱点加权
        if opts.zb_ai && kind.is_equip() {
            if let Some(def) = def_opt {
                if def.name.contains("qinggang")
                    && base.players.iter().any(|p| {
                        p.alive
                            && p.seat != seat
                            && l01_public_attitude(base, Some(opts), seat, p.seat) < 0
                            && has_armor_slot(catalog, p)
                    })
                {
                    delta += 15;
                }
                if def.name.contains("qilin")
                    && base.players.iter().any(|p| {
                        p.alive
                            && p.seat != seat
                            && l01_public_attitude(base, Some(opts), seat, p.seat) < 0
                            && has_mount_slot(catalog, p)
                    })
                {
                    delta += 15;
                }
                if def.name.contains("zhuque")
                    && base.players.iter().any(|p| {
                        p.alive
                            && p.seat != seat
                            && l01_public_attitude(base, Some(opts), seat, p.seat) < 0
                            && has_equip_named(catalog, p, "tengjia")
                    })
                {
                    delta += 15;
                }
                if def.name.contains("guding")
                    && base.players.iter().any(|p| {
                        p.alive
                            && p.seat != seat
                            && l01_public_attitude(base, Some(opts), seat, p.seat) < 0
                            && p.hand_count == 0
                    })
                {
                    delta += 15;
                }
            }
        }

        // `twzhian_ai`: 敌方存在未使用的 `twzhian` 时降低装备/延时锦囊意愿
        if opts.twzhian_active_enemy
            && (kind.is_equip() || matches!(kind, CardKind::Le | CardKind::Bing | CardKind::ShanDian))
        {
            delta -= 12;
        }

        // `dcwumei_all`: 觉醒后抑制回复并压低盲目伤害收益
        if opts.dcwumei_active {
            if kind == CardKind::Tao {
                delta -= 20;
            } else if is_single_damage_play(kind) {
                delta -= 4;
            }
        }

        // `_gjcx_correct` / `_shyg_qjAi`: 防具无效提示与连环队友防误伤
        if opts.bt_ai != L01StrategyMode::Off {
            let ignore_armor = has_equip_named(catalog, me, "qinggang");
            let has_zhuque = has_equip_named(catalog, me, "zhuque");
            for &t in targets {
                let Some(tp) = base.players.get(t as usize) else {
                    continue;
                };
                if kind == CardKind::Sha && !ignore_armor {
                    if !has_zhuque && has_equip_named(catalog, tp, "tengjia") {
                        delta -= 40;
                    }
                    if matches!(
                        suit,
                        Some(crate::catalog::Suit::Spade | crate::catalog::Suit::Club)
                    ) && has_equip_named(catalog, tp, "renwang")
                    {
                        delta -= 40;
                    }
                }
                if matches!(kind, CardKind::FireSha | CardKind::LeiSha)
                    && !ignore_armor
                    && (has_equip_named(catalog, tp, "taipingyaoshu")
                        || has_equip_named(catalog, tp, "suolianjia"))
                {
                    delta -= 40;
                }
                // 横置防伤己方残血队友（`hzdy` / `cxhzdyf`）
                if matches!(kind, CardKind::FireSha | CardKind::LeiSha) && tp.linked {
                    let danger_allies = base
                        .players
                        .iter()
                        .filter(|p| {
                            p.alive
                                && p.linked
                                && p.hp <= 1
                                && l01_public_attitude(base, Some(opts), seat, p.seat) > 0
                        })
                        .count();
                    let linked_enemies = base
                        .players
                        .iter()
                        .filter(|p| {
                            p.alive
                                && p.linked
                                && l01_public_attitude(base, Some(opts), seat, p.seat) <= 0
                        })
                        .count();
                    if danger_allies > 0 && linked_enemies <= danger_allies {
                        delta -= 35;
                    }
                }
            }
            // 黑光铠多目标豁免（`heiguangkai` + targets.len() >= 2）
            if opts.bt_ai == L01StrategyMode::Gjcx && targets.len() >= 2 {
                if targets.iter().any(|&t| {
                    base.players
                        .get(t as usize)
                        .is_some_and(|tp| has_equip_named(catalog, tp, "heiguangkai"))
                }) {
                    delta -= 25;
                }
            }
        }

        // `_aiyh_nhFriends`: 不砍低血量队友（`off` / `1..=4` / `<= self.hp`）
        if opts.nh_friends != NhFriendsMode::Off {
            let block_aoe = !opts.nt_aoe && is_aoe_damage_play(kind);
            if is_single_damage_play(kind) || block_aoe {
                let threshold = match opts.nh_friends {
                    NhFriendsMode::Off => i32::MIN,
                    NhFriendsMode::MaxHp(v) => v,
                    NhFriendsMode::SelfHp => me.hp,
                };
                let checked: Vec<u8> = if block_aoe {
                    base.players
                        .iter()
                        .filter(|p| p.alive && p.seat != seat)
                        .map(|p| p.seat)
                        .collect()
                } else {
                    targets.clone()
                };
                for t in checked {
                    if let Some(tp) = base.players.get(t as usize) {
                        if t != seat
                            && l01_public_attitude(base, Some(opts), seat, t) > 0
                            && tp.hp <= threshold
                        {
                            delta -= 80;
                        }
                    }
                }
            }
        }

        // `_mjAiSkill` / `aiMangju` & `sfjAi`（身份局主忠盲狙 vs 保主残局）
        if opts.sfj_ai != L01StrategyMode::Off {
            let lord = base.players.iter().find(|p| p.role == Some(Role::Lord));
            if opts.mj_ai && matches!(my_role, Some(Role::Lord | Role::Loyalist)) {
                for &t in targets {
                    if t != seat && l01_public_attitude(base, Some(opts), seat, t) == 0 && is_single_damage_play(kind) {
                        delta += 6;
                    }
                }
            }
            if my_role == Some(Role::Loyalist) {
                if let Some(l) = lord {
                    if kind == CardKind::Tao && l.hp <= 1 && !targets.contains(&l.seat) {
                        delta -= 60;
                    }
                    if kind == CardKind::Taoyuan && l.hp <= 1 {
                        delta += 25;
                    }
                }
            }
            // 内奸平衡（`gjcx_neiZhong`/`gjcx_neiFan`/`gjcx_neiJiang` & `_aiyh_lianhe`）仅基于公开态度计算
            if my_role == Some(Role::Traitor) {
                let mut balance = 0i32;
                let mut alive_enemies_known = 0i32;
                for p in &base.players {
                    if !p.alive || p.seat == seat {
                        continue;
                    }
                    let att = l01_public_attitude(base, Some(opts), seat, p.seat);
                    if p.role == Some(Role::Lord) || att > 0 {
                        balance += p.hp;
                    } else if p.role == Some(Role::Rebel) {
                        balance -= p.hp;
                        alive_enemies_known += 1;
                    }
                }
                if let Some(l) = lord {
                    for &t in targets {
                        if t == l.seat && is_single_damage_play(kind) && (l.hp <= 1 || alive_enemies_known > 0) {
                            delta -= 60;
                        } else if t != seat && is_single_damage_play(kind) {
                            let is_known_rebel = base.players.get(t as usize).and_then(|p| p.role) == Some(Role::Rebel);
                            if balance < 0 && is_known_rebel {
                                delta += 8;
                            } else if balance > 1 && base.players.get(t as usize).and_then(|p| p.role) == Some(Role::Loyalist) {
                                delta += 6;
                            }
                        }
                    }
                }
            }
        }

        // `yiyi` AI 收益评估（使用公开可观测手牌数，不泄密）
        if def_opt.is_some_and(|d| d.name == "yiyi") {
            let mut sum = 0i64;
            for &t in targets {
                if let Some(tp) = base.players.get(t as usize) {
                    let att = l01_public_attitude(base, Some(opts), seat, t);
                    let sign = if t == seat || att >= 0 { 1 } else { -1 };
                    let count = if t == seat {
                        me.hand_count.saturating_sub(1)
                    } else {
                        tp.hand_count + tp.equips.len()
                    };
                    let val = (((count as f64) / 3.6).sqrt() * 5.0).round() as i64;
                    sum += sign * val;
                }
            }
            delta += sum;
        }
    }

    delta
}

/// 询问面 L01 专用策略分支（五谷选牌、以逸待劳弃牌、内奸亮明抉择、拼点出牌与无懈判断）。
pub(super) fn plan_l01_choice(
    catalog: &Catalog,
    input: &PolicyInput,
    opts: &L01AiOptions,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let Some(ChoiceView::Own {
        id,
        prompt,
        ref options,
        min,
        max,
        allow_cancel,
        ..
    }) = input.choice
    else {
        return None;
    };
    let me = input.view.players.get(seat as usize)?;
    match prompt {
        "wugu_pick" if opts.bt_ai == L01StrategyMode::Shyg => {
            let damaged = me.hp < (me.max_hp as i32);
            let card_priority = |cid: CardId| -> i32 {
                let k = catalog.card_kind(cid);
                if damaged {
                    match k {
                        CardKind::Tao => 100,
                        CardKind::Wuzhong => 90,
                        CardKind::Jiu => 80,
                        CardKind::Nullify => 70,
                        CardKind::Snatch => 60,
                        CardKind::Dismantle => 50,
                        CardKind::Le => 40,
                        CardKind::Bing => 30,
                        _ => 10,
                    }
                } else {
                    match k {
                        CardKind::Wuzhong => 100,
                        CardKind::Nullify => 90,
                        CardKind::Snatch => 80,
                        CardKind::Tao => 70,
                        CardKind::Dismantle => 60,
                        CardKind::Le => 50,
                        CardKind::Bing => 40,
                        CardKind::Jiu => 30,
                        _ => 10,
                    }
                }
            };
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .max_by_key(|c| card_priority(*c))?;
            Some(GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Card(best)],
            })
        }
        "l01_yiyi_discard" | "discard" => {
            let need = min.max(1).min(max);
            let keep_value = |cid: CardId| -> i32 {
                let k = catalog.card_kind(cid);
                let mut v = match k {
                    CardKind::Tao => 90,
                    CardKind::Wuzhong => 70,
                    CardKind::Nullify => 65,
                    CardKind::Jiu => 55,
                    CardKind::Shan => 45,
                    CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => 35,
                    _ => 30,
                };
                if opts.bt_ai == L01StrategyMode::Shyg {
                    if k == CardKind::Wuzhong && me.hp > 2 && me.hand_count <= 2 {
                        v += 30;
                    }
                    if k == CardKind::Nullify && !me.judge.is_empty() {
                        v += 30;
                    }
                    if k == CardKind::Tao && me.hp <= 2 {
                        v += 30;
                    }
                    if k == CardKind::Jiu && me.hp <= 1 {
                        v += 30;
                    }
                }
                v
            };
            let mut ranked: Vec<(i32, CardId)> = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some((keep_value(*c), *c)),
                    _ => None,
                })
                .collect();
            if ranked.len() < need {
                return None;
            }
            ranked.sort_by_key(|(v, _)| *v);
            let selection: Vec<ChoiceOption> = ranked
                .into_iter()
                .take(need)
                .map(|(_, c)| ChoiceOption::Card(c))
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "l01_neikey_recover" => {
            // `AI优化/extension.js:253-262`: 主公满血、无忠、自己<=2血或主公<=1血时选择共同回复
            let lord_hp_and_full = input
                .view
                .players
                .iter()
                .find(|p| p.role == Some(Role::Lord))
                .map(|l| (l.hp, l.hp >= l.max_hp as i32))
                .unwrap_or((1, false));
            let any_public_loyalist = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.role == Some(Role::Loyalist));
            let want = lord_hp_and_full.1
                || !any_public_loyalist
                || me.hp <= 2
                || lord_hp_and_full.0 <= 1;
            let selection = if want {
                vec![ChoiceOption::Confirm]
            } else if options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if allow_cancel {
                Vec::new()
            } else {
                vec![ChoiceOption::Confirm]
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        p if p.contains("pindian") && opts.hsyh => {
            // `AI优化/extension.js:5424-5447`: 拼点优先取高点低价值手牌
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => {
                        let def = catalog.cards.get(*c as usize)?;
                        let rank = def.rank as i32;
                        let val_penalty = if matches!(def.kind, CardKind::Tao | CardKind::Nullify) {
                            6
                        } else {
                            2
                        };
                        Some((rank * 10 - val_penalty * 5, *c))
                    }
                    _ => None,
                })
                .max_by_key(|(s, _)| *s)?;
            Some(GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Card(best.1)],
            })
        }
        "respond_shan" if opts.bt_ai == L01StrategyMode::Shyg => {
            l01_reserved_shan::plan_l01_reserved_shan(catalog, input, me, opts, id, options)
        }
        "dying_save" => {
            // `zhenlie_lose` 濒死救援评估（严格仅读公开态度与公开 zhenlie 标记，不读未明身份）。
            let dying_seat = input
                .view
                .players
                .iter()
                .find(|p| p.alive && p.hp <= 0)
                .map(|p| p.seat)?;
            let best_save_opt = || -> Option<ChoiceOption> {
                options
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => {
                            let k = catalog.card_kind(*c);
                            let prio = if dying_seat == seat
                                && matches!(k, CardKind::Jiu | CardKind::Tianxianjiu | CardKind::Hufu)
                            {
                                100
                            } else if matches!(k, CardKind::Tao | CardKind::Dinvxuanshuang) {
                                80
                            } else {
                                60
                            };
                            Some((prio, *o))
                        }
                        ChoiceOption::Skill(_) => Some((50, *o)),
                        _ => None,
                    })
                    .max_by_key(|(p, _)| *p)
                    .map(|(_, o)| o)
            };
            if dying_seat == seat {
                return best_save_opt().map(|opt| GameAction::Answer {
                    choice_id: id,
                    selection: vec![opt],
                });
            }
            let att = l01_public_attitude(&input.view, Some(opts), seat, dying_seat);
            let zhenlie_boost = if opts.zhenlie_lose_seats.contains(&dying_seat) {
                1.2
            } else {
                1.0
            };
            let save_benefit = if att > 0 { 90.0 * zhenlie_boost } else { 0.0 };
            let keep_cost = if me.hp <= 2 { 100.0 } else { 50.0 };
            if save_benefit > keep_cost {
                if let Some(opt) = best_save_opt() {
                    return Some(GameAction::Answer {
                        choice_id: id,
                        selection: vec![opt],
                    });
                }
            }
            if options.contains(&ChoiceOption::Pass) {
                Some(GameAction::Answer {
                    choice_id: id,
                    selection: vec![ChoiceOption::Pass],
                })
            } else if allow_cancel {
                Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                })
            } else {
                None
            }
        }
        "nullify_window" | "nullify_query" | "wuxie" if opts.bt_ai == L01StrategyMode::Shyg => {
            // `_shyg_wuxie` / `yiyi.ai.wuxie == 0`: 如果桌面结算牌是 `yiyi` 且可选过，不无懈
            let is_yiyi = input
                .view
                .table
                .iter()
                .any(|c| {
                    catalog
                        .cards
                        .get(*c as usize)
                        .is_some_and(|d| d.name == "yiyi" || d.name.ends_with("__ref_yiyi"))
                });
            if is_yiyi && options.contains(&ChoiceOption::Pass) {
                Some(GameAction::Answer {
                    choice_id: id,
                    selection: vec![ChoiceOption::Pass],
                })
            } else if is_yiyi && allow_cancel {
                Some(GameAction::Answer {
                    choice_id: id,
                    selection: Vec::new(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}
