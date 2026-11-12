//! 王朗 `th_gushe`（鼓舌，`extension.js:8219-8383`）/ `th_gushe2` / `rejici`（激词，
//! `character/sp.js:16971` 界王朗技能，雷霆万钧武将表直接引用）。
//!
//! 鼓舌：出牌阶段，若「饶舌」标记数与本回合鼓舌拼点胜利次数之和小于 7，可用一张手牌与
//! 1~3 名角色同时拼点，逐一结算：没赢的角色选择弃置一张牌或令对方摸一张牌；王朗没赢则
//! 获得 1 枚「饶舌」，第 7 枚时死亡。
//! 激词：①拼点牌亮出后若点数不大于 X，点数 +X 并获得此次拼点点数最大的拼点牌；②死亡时令
//! 凶手弃置 7-X 张牌并失去 1 点体力。X 为上游 `regushe` 标记数——雷霆版鼓舌记的是
//! `th_gushe` 标记，不增加 `regushe`，故 X 恒为 0（①不触发、②弃 7 张），此处按上游口径保留。

use super::*;

const RAOSUO: &str = "th_gushe_raosuo";
const REGUSHE: &str = "regushe";

fn wins_this_turn(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "th_gushe2_stamp") == turn_stamp(core) {
        flag(core, owner, "th_gushe2_marks")
    } else {
        0
    }
}

pub(super) fn gushe_targets(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| !core.player(p).hand.is_empty())
        .collect()
}

pub(super) fn gushe_can_activate(core: &GameStateCore, owner: PlayerId) -> bool {
    !core.player(owner).hand.is_empty()
        && !gushe_targets(core, owner).is_empty()
        && flag(core, owner, RAOSUO) + wins_this_turn(core, owner) < 7
}

fn raosuo_marks(core: &GameStateCore, p: PlayerId) -> u32 {
    flag(core, p, REGUSHE).max(flag(core, p, RAOSUO))
}

pub(super) fn rejici_discard_count(core: &GameStateCore, owner: PlayerId) -> u32 {
    7u32.saturating_sub(raosuo_marks(core, owner))
}

fn rejici_x(ctx: &Ctx<'_>, p: PlayerId) -> Option<u32> {
    has_skill(ctx.core, ctx.catalog, p, "rejici").then(|| raosuo_marks(ctx.core, p))
}

fn gain_from_discard(ctx: &mut Ctx<'_>, p: PlayerId, c: CardId) {
    if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
        ctx.core.discard.remove(pos);
        ctx.core.player_mut(p).hand.push(c);
    }
}

/// 第 `idx` 组拼点的点数（含激词①修正）；激词获得点数最大的拼点牌。
fn compare(m: &HostMachine, ctx: &mut Ctx<'_>, idx: usize) -> (u32, u32) {
    let (owner, t) = (m.owner, m.targets[idx]);
    let (c1, c2) = (m.card.unwrap_or_default(), m.cards[idx]);
    let (mut n1, mut n2) = (card_rank(ctx.catalog, c1), card_rank(ctx.catalog, c2));
    let mut gainers = Vec::new();
    // 发起者的激词只在多目标拼点的首组（`!trigger.iwhile`）修正点数。
    if let Some(x) = rejici_x(ctx, owner).filter(|&x| n1 <= x) {
        if idx == 0 {
            n1 += x;
        }
        gainers.push(owner);
    }
    if let Some(x) = rejici_x(ctx, t).filter(|&x| n2 <= x) {
        n2 += x;
        gainers.push(t);
    }
    for p in gainers {
        if n1 > n2 {
            gain_from_discard(ctx, p, c1);
        } else if n1 < n2 {
            gain_from_discard(ctx, p, c2);
        }
    }
    (n1, n2)
}

pub(super) fn step_th_gushe(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_GUSHE_TARGETS),
        HostStage::TargetChoice => {
            let picked: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            m.targets = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|p| picked.contains(p))
                .collect();
            // `player.addTempSkill('th_gushe2')`：本回合胜利计数载体。
            if let Some(def) = ctx.catalog.skill_by_name("th_gushe2") {
                ctx.core.skills.insert(
                    (owner, def),
                    crate::state::SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: true,
                        rounds: 0,
                    },
                );
            }
            if flag(ctx.core, owner, "th_gushe2_stamp") != turn_stamp(ctx.core) {
                set_flag(ctx.core, owner, "th_gushe2_stamp", turn_stamp(ctx.core));
                set_flag(ctx.core, owner, "th_gushe2_marks", 0);
            }
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_GUSHE_OWNER_CARD)
        }
        HostStage::ComparePlayerPick => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(c);
            m.cards.clear();
            m.target = m.targets.first().copied();
            wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_GUSHE_TARGET_CARD)
        }
        HostStage::CompareTargetPick => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards.push(c);
            if m.cards.len() < m.targets.len() {
                m.target = Some(m.targets[m.cards.len()]);
                return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_GUSHE_TARGET_CARD);
            }
            // 拼点牌同时亮出并进入弃牌堆。
            if let Some(c1) = m.card {
                discard_cards(ctx, owner, &[c1]);
            }
            for (i, &t) in m.targets.clone().iter().enumerate() {
                discard_cards(ctx, t, &[m.cards[i]]);
            }
            set_flag(ctx.core, owner, "th_gushe_idx", 0);
            resolve_next(m, ctx)
        }
        HostStage::CardsAsk => {
            let t = m.target.unwrap_or(owner);
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) => discard_cards(ctx, t, &[c]),
                _ => ctx.start_draw(owner, 1),
            }
            owner_part(m, ctx)
        }
        HostStage::JiefenOwnerDiscard => {
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) => discard_cards(ctx, owner, &[c]),
                _ => ctx.start_draw(owner, 1),
            }
            advance(m, ctx)
        }
        _ => Flow::Done,
    }
}

/// 结算第 `th_gushe_idx` 组：记饶舌/胜利，然后（未赢的一方）弃牌或令对方摸牌。
fn resolve_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let idx = flag(ctx.core, owner, "th_gushe_idx") as usize;
    if idx >= m.targets.len() || !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let (n1, n2) = compare(m, ctx, idx);
    let cmp = match n1.cmp(&n2) {
        std::cmp::Ordering::Less => 0,
        std::cmp::Ordering::Equal => 1,
        std::cmp::Ordering::Greater => 2,
    };
    set_flag(ctx.core, owner, "th_gushe_cmp", cmp);
    if n1 <= n2 {
        let raosuo = flag(ctx.core, owner, RAOSUO) + 1;
        set_flag(ctx.core, owner, RAOSUO, raosuo);
        if raosuo >= 7 {
            set_flag(ctx.core, owner, "th_gushe_died", 1);
            ctx.emit(EventData::Death {
                player: owner,
                killer: None,
            });
            return Flow::Done;
        }
    } else {
        let wins = wins_this_turn(ctx.core, owner) + 1;
        set_flag(ctx.core, owner, "th_gushe2_marks", wins);
    }
    let t = m.targets[idx];
    m.target = Some(t);
    if n1 >= n2 && ctx.core.player(t).alive {
        if hand_and_equips(ctx.core, t).is_empty() {
            ctx.start_draw(owner, 1);
        } else {
            return wait_ask(m, ctx, HostStage::CardsAsk, ASK_GUSHE_TARGET_DISCARD);
        }
    }
    owner_part(m, ctx)
}

fn owner_part(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    // th_gushe_cmp：0 = 王朗点数小，1 = 平局，2 = 王朗点数大。
    if flag(ctx.core, owner, "th_gushe_cmp") <= 1 {
        if hand_and_equips(ctx.core, owner).is_empty() {
            ctx.start_draw(owner, 1);
        } else {
            return wait_ask(m, ctx, HostStage::JiefenOwnerDiscard, ASK_GUSHE_OWNER_DISCARD);
        }
    }
    advance(m, ctx)
}

fn advance(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let idx = flag(ctx.core, owner, "th_gushe_idx") + 1;
    set_flag(ctx.core, owner, "th_gushe_idx", idx);
    resolve_next(m, ctx)
}

/// 激词②（`rejici2`）：王朗死亡时，凶手弃置 7-X 张牌并失去 1 点体力。
pub(super) fn step_rejici(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(killer) = m.target else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            if rejici_discard_count(ctx.core, m.owner) > 0
                && !hand_and_equips(ctx.core, killer).is_empty()
            {
                return wait_ask(m, ctx, HostStage::CardsAsk, ASK_REJICI_KILLER_DISCARD);
            }
        }
        HostStage::CardsAsk => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            discard_cards(ctx, killer, &cards);
        }
        _ => return Flow::Done,
    }
    ctx.emit(EventData::LoseHp {
        player: killer,
        amount: 1,
    });
    Flow::Done
}
