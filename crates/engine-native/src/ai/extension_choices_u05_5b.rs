//! U05-5b（杀海拾遗 · 昆特牌武将角色技）公开信息集 AI 选择策略。
//! 严格仅读取 `PolicyInput` 公开信息与自身可见手牌，不偷看他家暗手牌。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

fn pick_lowest_cards(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut scored: Vec<(i64, ChoiceOption)> = options
        .iter()
        .filter_map(|&opt| match opt {
            ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), opt)),
            _ => None,
        })
        .collect();
    scored.sort_by_key(|(v, _)| *v);
    scored.into_iter().take(count).map(|(_, o)| o).collect()
}

fn pick_highest_cards(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut scored: Vec<(i64, ChoiceOption)> = options
        .iter()
        .filter_map(|&opt| match opt {
            ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), opt)),
            _ => None,
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(count).map(|(_, o)| o).collect()
}

fn best_enemy_opt(input: &PolicyInput, seat: u8, options: &[ChoiceOption]) -> Option<ChoiceOption> {
    options
        .iter()
        .filter_map(|&opt| match opt {
            ChoiceOption::Player(p) => {
                let att = public_attitude(&input.view, seat, p);
                let hc = input.view.players.get(p as usize).map_or(0, |pv| pv.hand_count as i32);
                Some(((-att * 100) + hc, opt))
            }
            _ => None,
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, o)| o)
}

fn best_friend_opt(input: &PolicyInput, seat: u8, options: &[ChoiceOption]) -> Option<ChoiceOption> {
    options
        .iter()
        .filter_map(|&opt| match opt {
            ChoiceOption::Player(p) => {
                let att = public_attitude(&input.view, seat, p);
                Some((att * 100, opt))
            }
            _ => None,
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, o)| o)
}

pub(super) fn plan_u05_5b_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let ans = |sel: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection: sel,
        })
    };
    match prompt {
        // 公开手牌数最大猜想：禁止偷看他家手牌（`gwjingshi_guess` 只读 `hand_count`）
        "gwjingshi_guess" => {
            let pick = options
                .iter()
                .filter_map(|&opt| match opt {
                    ChoiceOption::Player(p) => {
                        let hc = input.view.players.get(p as usize).map_or(0, |pv| pv.hand_count);
                        Some((hc, opt))
                    }
                    _ => None,
                })
                .max_by_key(|(hc, _)| *hc)
                .map(|(_, o)| o)
                .or_else(|| options.first().copied())?;
            ans(vec![pick])
        }
        // `gwtianbian`: 阵营公开计数决定正向或负向 AOE
        "gwtianbian_mode" => {
            let mut pos = 0i32;
            let mut neg = 0i32;
            for p in &input.view.players {
                if !p.alive {
                    continue;
                }
                if public_attitude(&input.view, seat, p.seat) > 0 {
                    pos += 1;
                } else {
                    neg += 1;
                }
            }
            let pref = if pos > neg {
                ChoiceOption::Tag(0)
            } else {
                ChoiceOption::Tag(1)
            };
            let pick = if options.contains(&pref) {
                pref
            } else {
                *options.first()?
            };
            ans(vec![pick])
        }
        // `gwfengshi`: 低血或少敌优先 2 buff，否则 3 debuff
        "gwfengshi_choice" => {
            let my_hp = input.view.players.get(seat as usize).map_or(0, |p| p.hp);
            let enemies = input
                .view
                .players
                .iter()
                .filter(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) < 0)
                .count();
            let pref = if my_hp <= 2 || enemies < 2 {
                ChoiceOption::Tag(1)
            } else {
                ChoiceOption::Tag(0)
            };
            let pick = if options.contains(&pref) {
                pref
            } else {
                *options.first()?
            };
            ans(vec![pick])
        }
        // `gwgouhun`: 优先交牌/弃杂而非变疯，牌极少或过痛则选末位
        "gwgouhun_reply" => {
            let pick = *options.first()?;
            ans(vec![pick])
        }
        // 伤害/负效指敌类（有敌人必挑，全友可取消则取消）
        "sqlongnu_damage" | "gwlangshi_target" | "gwfutian_target" | "gwgouhun_target"
        | "gwxiaoshou_target" | "fengjian_target" | "hunmo_target" | "gwjushi_target"
        | "gwbaquan_target" | "shuangxi_targets" => {
            let enemies: Vec<ChoiceOption> = options
                .iter()
                .copied()
                .filter(|opt| {
                    matches!(opt, ChoiceOption::Player(p) if public_attitude(&input.view, seat, *p) < 0)
                })
                .collect();
            if enemies.is_empty() && allow_cancel {
                return ans(Vec::new());
            }
            if max > 1 && !enemies.is_empty() {
                return ans(enemies.into_iter().take(max).collect());
            }
            let pick = best_enemy_opt(input, seat, options)?;
            ans(vec![pick])
        }
        "gwminxiang_targets" | "gwjiquan_targets" => {
            let enemies: Vec<ChoiceOption> = options
                .iter()
                .copied()
                .filter(|opt| {
                    matches!(opt, ChoiceOption::Player(p) if public_attitude(&input.view, seat, *p) < 0)
                })
                .take(max)
                .collect();
            if enemies.len() >= min {
                ans(enemies)
            } else if allow_cancel {
                ans(Vec::new())
            } else {
                ans(options.iter().take(min).copied().collect())
            }
        }
        "gwjinli_target" | "gwqinwu_target" | "gwshenyu_target" | "hupeng_target" => {
            let friends: Vec<ChoiceOption> = options
                .iter()
                .copied()
                .filter(|opt| {
                    matches!(opt, ChoiceOption::Player(p) if public_attitude(&input.view, seat, *p) > 0)
                })
                .take(max)
                .collect();
            if max > 1 && !friends.is_empty() {
                return ans(friends);
            }
            if let Some(pick) = best_friend_opt(input, seat, options) {
                ans(vec![pick])
            } else if allow_cancel {
                ans(Vec::new())
            } else {
                ans(options.iter().take(min).copied().collect())
            }
        }
        "huandie_targets" => {
            let friends: Vec<ChoiceOption> = options
                .iter()
                .copied()
                .filter(|opt| {
                    matches!(opt, ChoiceOption::Player(p) if public_attitude(&input.view, seat, *p) > 0)
                })
                .take(max)
                .collect();
            ans(friends)
        }
        "sqlongnu_pick" | "lanquan_pick" | "zhengjun_pick" | "bolang_gain"
        | "gwshenyu_pick" | "hunmo_pick" | "gwjingshi_steal" | "gwjiquan_steal" => {
            ans(pick_highest_cards(catalog, input, options, min.max(1)))
        }
        "gwjinli_card" | "gwhuanbi_cost" | "gwminxiang_card" | "gwgouhun_card"
        | "gwyewu_cost" | "huanshu_card" | "lingji_discard" | "hupeng_card"
        | "hunmo_discard_self" | "shuijian_cost" | "yinzhang_cost" | "gwxiaoshou_cost"
        | "nuhou_cost" | "shewu_cost" | "gwzhanjiang_cost" | "fayin_cost"
        | "huandie_discard_pick" | "gwbaquan_return" | "gwhuanbi_give" => {
            ans(pick_lowest_cards(catalog, input, options, min.max(1)))
        }
        "sqlongwu_use_card" | "junchi_use" | "hupeng4_confirm" | "gwzhanjiang_sha"
        | "gwchuanxin_confirm" | "bolang_pick" | "gwhuanshuang_pick" | "gwhuanbi_pick"
        | "gwjieyin_pick" | "yinzhang_pick" | "gwshenyu_mode" | "hupeng_pick"
        | "xuezhou_mode" | "gwfengchi_pick" => ans(options.iter().take(min.max(1)).copied().collect()),
        "sqlongwu_use_target" | "jielue_target" | "gwhuanshuang_target"
        | "gwjieyin_target" | "lanquan_target" | "gwfutian_discard" | "hunmo_discard_other" => {
            if let Some(pick) = best_enemy_opt(input, seat, options) {
                ans(vec![pick])
            } else {
                ans(options.iter().take(min.max(1)).copied().collect())
            }
        }
        _ => None,
    }
}
