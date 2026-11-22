//! L11（太虚幻境）Batch L11-5 挂起询问的公开信息集 AI 启发式策略。
//!
//! 仅使用 `PolicyInput` / `PlayerView` 公开信息（身份、阵营、明牌装备/判定区、手牌数、血量与己方手牌），
//! 覆盖批 5 新增与重写的武器、系统、领主与武将技能询问，不读取任何未知隐藏牌或随机状态。

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
    let mut cards: Vec<u32> = options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
    cards.into_iter().take(count).map(ChoiceOption::Card).collect()
}

fn pick_highest_card(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
) -> Option<ChoiceOption> {
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .max_by_key(|&c| extension_card_value(catalog, input, c))
        .map(ChoiceOption::Card)
}

fn best_hostile_target(
    input: &PolicyInput,
    options: &[ChoiceOption],
    require_negative: bool,
) -> Option<u8> {
    let seat = input.view.seat;
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .filter(|&p| !require_negative || public_attitude(&input.view, seat, p) < 0)
        .min_by_key(|&p| {
            let att = public_attitude(&input.view, seat, p);
            let hp = input.view.players.get(p as usize).map_or(99, |pl| pl.hp);
            (att, hp, p)
        })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_b5_choice_action(
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
    let answer = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };
    let has_enemy = input
        .view
        .players
        .iter()
        .enumerate()
        .any(|(i, p)| i as u8 != seat && p.alive && public_attitude(&input.view, seat, i as u8) < 0);

    match prompt {
        // `extension_skill.js:378`: 优先选冲应神符 (Tag 2)
        "twsidao:choose_treasure" => {
            if options.contains(&ChoiceOption::Tag(2)) {
                answer(vec![ChoiceOption::Tag(2)])
            } else {
                options.first().copied().map(|o| vec![o]).and_then(answer)
            }
        }
        "hanbing_skill:confirm" => {
            let prefer = input.view.players.iter().enumerate().any(|(i, p)| {
                i as u8 != seat
                    && p.alive
                    && public_attitude(&input.view, seat, i as u8) < 0
                    && p.hp > 1
                    && (p.hand_count + p.equips.len()) >= 2
            });
            answer(vec![if prefer {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "hanbing_skill:pick_card" | "txhj_xuanlve:card" => {
            if let Some(opt) = pick_highest_card(catalog, input, options) {
                answer(vec![opt])
            } else {
                None
            }
        }
        "cixiong_skill:confirm"
        | "txhj_mouduan:confirm"
        | "olshanjia:confirm"
        | "boss_yushou:confirm"
        | "boss_mojian:confirm"
        | "boss_lianyu:confirm"
        | "boss_jueji:confirm" => answer(vec![if has_enemy {
            ChoiceOption::Confirm
        } else {
            ChoiceOption::Pass
        }]),
        "boss_lingfeng:confirm"
        | "boss_moyany:confirm"
        | "boss_danshu:confirm"
        | "txhj_zhuwei:confirm_gain" => answer(vec![ChoiceOption::Confirm]),
        "txhj_zhuwei:confirm_buff" => {
            let cur = input.view.turn;
            let good = public_attitude(&input.view, seat, cur) > 0;
            answer(vec![if good {
                ChoiceOption::Confirm
            } else {
                ChoiceOption::Pass
            }])
        }
        "cixiong_skill:target_discard" => {
            let need = min.max(1);
            let sel = pick_lowest_cards(catalog, input, options, need);
            let total_val: i64 = sel
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(extension_card_value(catalog, input, c)),
                    _ => None,
                })
                .sum();
            if sel.len() == need && total_val < 7 * (need as i64) {
                answer(sel)
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(sel)
            }
        }
        "guanshi_skill:discard" => {
            let need = min.max(1);
            let sel = pick_lowest_cards(catalog, input, options, need);
            if sel.len() == need {
                answer(sel)
            } else {
                answer(Vec::new())
            }
        }
        "sanjian_skill:discard" | "yitianjian:discard" | "pyzhuren_diamond:discard" => {
            let sel = pick_lowest_cards(catalog, input, options, 1);
            let val = sel.first().and_then(|o| match *o {
                ChoiceOption::Card(c) => Some(extension_card_value(catalog, input, c)),
                _ => None,
            });
            if val.is_some_and(|v| v <= 6) {
                answer(sel)
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(sel)
            }
        }
        "sanjian_skill:target"
        | "boss_lingfeng:target"
        | "boss_leili:target"
        | "boss_fengxing:target"
        | "boss_xixing:target"
        | "txhj_xuanlve:target"
        | "txhj_chuhai:target"
        | "olshanjia:sha_target" => {
            if let Some(p) = best_hostile_target(input, options, true) {
                answer(vec![ChoiceOption::Player(p)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                best_hostile_target(input, options, false)
                    .map(|p| vec![ChoiceOption::Player(p)])
                    .and_then(answer)
            }
        }
        "boss_moyany:target" => best_hostile_target(input, options, false)
            .map(|p| vec![ChoiceOption::Player(p)])
            .and_then(answer),
        "txhj_mouduan:move_from" => {
            let pick = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .max_by_key(|&p| {
                    let att = public_attitude(&input.view, seat, p);
                    let pl = input.view.players.get(p as usize);
                    let has_j = pl.is_some_and(|x| !x.judge.is_empty());
                    let has_e = pl.is_some_and(|x| !x.equips.is_empty());
                    if att > 0 && has_j {
                        300
                    } else if att < 0 && has_e {
                        200
                    } else {
                        -att
                    }
                });
            if let Some(p) = pick {
                answer(vec![ChoiceOption::Player(p)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                None
            }
        }
        "txhj_mouduan:move_card" => {
            let equip_first = options.iter().filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            }).max_by_key(|&c| extension_card_value(catalog, input, c));
            if let Some(c) = equip_first {
                answer(vec![ChoiceOption::Card(c)])
            } else {
                answer(Vec::new())
            }
        }
        "txhj_mouduan:move_to" => {
            let players: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            let friend_first = players
                .iter()
                .copied()
                .max_by_key(|&p| public_attitude(&input.view, seat, p));
            if let Some(p) = friend_first {
                answer(vec![ChoiceOption::Player(p)])
            } else {
                answer(Vec::new())
            }
        }
        "olshanjia:discard" => {
            let need = min.max(1);
            let mut cards: Vec<u32> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            // 优先弃置装备以触发不计次数的无视距离【杀】
            cards.sort_by_key(|&c| {
                let non_equip = u8::from(catalog.card_kind(c).equip_slot().is_none());
                (non_equip, extension_card_value(catalog, input, c))
            });
            let sel: Vec<ChoiceOption> = cards.into_iter().take(need).map(ChoiceOption::Card).collect();
            answer(sel)
        }
        "txhj_chuhai:pindian_card" | "txhj_chuhai:target_pindian_card" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .max_by_key(|&c| catalog.cards.get(c as usize).map_or(0, |cd| cd.rank));
            best.map(|c| vec![ChoiceOption::Card(c)]).and_then(answer)
        }
        "xueji:card" => {
            let sel = pick_lowest_cards(catalog, input, options, 1);
            let val = sel.first().and_then(|o| match *o {
                ChoiceOption::Card(c) => Some(extension_card_value(catalog, input, c)),
                _ => None,
            });
            if has_enemy && val.is_some_and(|v| v <= 70) {
                answer(sel)
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(sel)
            }
        }
        "xueji:target" => {
            let mut enemies: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .filter(|&p| public_attitude(&input.view, seat, p) < 0)
                .collect();
            if enemies.is_empty() {
                if allow_cancel {
                    return answer(Vec::new());
                }
                return best_hostile_target(input, options, false)
                    .map(|p| vec![ChoiceOption::Player(p)])
                    .and_then(answer);
            }
            enemies.sort_by_key(|&p| {
                let linked = input.view.players.get(p as usize).is_some_and(|pl| pl.linked);
                let hp = input.view.players.get(p as usize).map_or(99, |pl| pl.hp);
                (u8::from(linked), hp, p)
            });
            let sel: Vec<ChoiceOption> = enemies
                .into_iter()
                .take(max.max(1))
                .map(ChoiceOption::Player)
                .collect();
            answer(sel)
        }
        _ => None,
    }
}
