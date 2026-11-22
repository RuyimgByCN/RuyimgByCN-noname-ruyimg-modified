//! L11（太虚幻境）Batch 1 挂起询问的公开信息集 AI 启发式策略。

use super::{extension_card_value, public_attitude};
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    _max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let self_cell = input.view.players.iter().find(|p| p.seat == seat);
    let answer = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };
    let best_player = |want_enemy: bool| -> Option<u8> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(p) => Some(p),
                _ => None,
            })
            .max_by_key(|&p| {
                let att = public_attitude(&input.view, seat, p);
                if want_enemy {
                    -att
                } else {
                    att
                }
            })
    };
    let best_cards_low = |count: usize| -> Vec<ChoiceOption> {
        let mut cards: Vec<_> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            })
            .collect();
        cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
        cards.into_iter().take(count).map(ChoiceOption::Card).collect()
    };

    match prompt {
        // Unconditional confirmations
        "txhj_zhijizhibi:confirm" => answer(vec![ChoiceOption::Confirm]),
        "txhj_piliche:confirm"
        | "txhj_leijimu:confirm"
        | "txhj_feilong:confirm"
        | "txhj_feilong_skill:confirm"
        | "txayboji:confirm"
        | "tx_mojian:confirm"
        | "tx_yushou:confirm"
        | "diyzhuiling:confirm"
        | "txshixin:confirm"
        | "txmouduan:confirm"
        | "txfeiyan:confirm"
        | "cxy_BaoYing:confirm"
        | "cxyJieLve:confirm"
        | "txkaikang:choose_equip" => {
            if options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else {
                answer(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "txhj_qilingong:discard_horse" | "qilin_skill:discard_horse" => {
            if options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else if let Some(c) = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
            {
                answer(vec![c])
            } else {
                answer(Vec::new())
            }
        }
        "txkaikang:confirm" => {
            if options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else {
                answer(Vec::new())
            }
        }
        // Targeted choices against enemies
        "txhj_fangtianhuaji:extra_targets" => {
            let mut enemies: Vec<_> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) if public_attitude(&input.view, seat, p) < 0 => Some(p),
                    _ => None,
                })
                .collect();
            if enemies.is_empty() && allow_cancel {
                answer(Vec::new())
            } else {
                enemies.truncate(2);
                let sel: Vec<_> = enemies.into_iter().map(ChoiceOption::Player).collect();
                if sel.is_empty() {
                    answer(options.iter().take(min.max(1)).copied().collect())
                } else {
                    answer(sel)
                }
            }
        }
        "tx_moyany:choose_target"
        | "txleili:choose_target"
        | "cxyKuangXi:choose_target"
        | "tx_dqtianzi:choose_target" => {
            if let Some(p) = best_player(true) {
                if allow_cancel
                    && prompt == "cxyKuangXi:choose_target"
                    && self_cell.is_some_and(|c| c.hp <= 1)
                {
                    return answer(Vec::new());
                }
                answer(vec![ChoiceOption::Player(p)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(min.max(1)).copied().collect())
            }
        }
        // Friendly targets
        "txhj_daiyuan:choose_target"
        | "txzhaoshu_global:choose_target"
        | "txyirang:choose_target"
        | "xinmingjian:choose_target" => {
            if let Some(p) = best_player(false) {
                answer(vec![ChoiceOption::Player(p)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "cxyYingHun:choose_target" => {
            if let Some(p) = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .max_by_key(|&p| public_attitude(&input.view, seat, p).abs())
            {
                answer(vec![ChoiceOption::Player(p)])
            } else {
                answer(Vec::new())
            }
        }
        "cxyYingHun:choose_mode" => {
            let any_friend = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) > 0);
            let tag = if any_friend { 0 } else { 1 };
            if options.contains(&ChoiceOption::Tag(tag)) {
                answer(vec![ChoiceOption::Tag(tag)])
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "cxyYaoWu:choose_mode" => {
            let want = if self_cell.is_some_and(|c| c.hp < c.max_hp as i32) {
                0
            } else {
                1
            };
            if options.contains(&ChoiceOption::Tag(want)) {
                answer(vec![ChoiceOption::Tag(want)])
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "txhj_daiyuan:choose_mode" => answer(options.iter().take(1).copied().collect()),
        "txhj_daiyuan:choose_cards" => {
            let cards: Vec<u32> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            for i in 0..cards.len() {
                for j in (i + 1)..cards.len() {
                    let s1 = catalog.cards.get(cards[i] as usize).map(|c| c.suit);
                    let s2 = catalog.cards.get(cards[j] as usize).map(|c| c.suit);
                    if s1 != s2 {
                        return answer(vec![
                            ChoiceOption::Card(cards[i]),
                            ChoiceOption::Card(cards[j]),
                        ]);
                    }
                }
            }
            if allow_cancel {
                answer(Vec::new())
            } else {
                answer(best_cards_low(min.max(2)))
            }
        }
        "txhj_shuiyan:discard_equips" => {
            let mut eqs: Vec<(u32, u8, i64)> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => {
                        let rank = catalog.cards.get(c as usize).map_or(0, |cd| cd.rank);
                        Some((c, rank, extension_card_value(catalog, input, c)))
                    }
                    _ => None,
                })
                .collect();
            eqs.sort_by_key(|x| x.2);
            let mut picked = Vec::new();
            let mut sum = 0u32;
            for (c, r, _) in eqs {
                picked.push(ChoiceOption::Card(c));
                sum += u32::from(r);
                if sum >= 6 {
                    return answer(picked);
                }
            }
            if allow_cancel {
                answer(Vec::new())
            } else {
                answer(picked)
            }
        }
        "txhj_taijifuchen:target_discard" | "gx_taijifuchen:target_discard" => {
            let has_shan = input
                .own_hand_kinds
                .iter()
                .any(|&(_, k)| k == CardKind::Shan);
            if has_shan {
                let best = options
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) if !input.own_hand_kinds.iter().any(|&(hc, hk)| hc == c && hk == CardKind::Shan) => Some(c),
                        _ => None,
                    })
                    .min_by_key(|&c| extension_card_value(catalog, input, c));
                if let Some(c) = best {
                    return answer(vec![ChoiceOption::Card(c)]);
                }
            }
            if options.contains(&ChoiceOption::Pass) {
                answer(vec![ChoiceOption::Pass])
            } else {
                answer(Vec::new())
            }
        }
        "txhj_jiedao:choose_sha" | "cxyFanGong:choose_sha" | "txhj_jiuchiroulin:respond_jiu" => {
            let best_card = options.iter().find_map(|o| match *o {
                ChoiceOption::Card(c) => Some(ChoiceOption::Card(c)),
                _ => None,
            });
            if let Some(c) = best_card {
                answer(vec![c])
            } else if options.contains(&ChoiceOption::Pass) {
                answer(vec![ChoiceOption::Pass])
            } else {
                answer(Vec::new())
            }
        }
        "txhj_dinglanyemingzhu:discard_cards"
        | "txhj_xingtianpojunfu:discard_cards"
        | "txhj_noda_axe:discard_cards"
        | "txhj_yiyi:discard_cards"
        | "txhj_caomu:discard_card"
        | "txhj_shengdong:give_card1"
        | "txhj_shengdong:give_card2"
        | "huogong:show_card"
        | "huogong:discard_card"
        | "txzhaoshu_global:give_cards"
        | "txayboji:discard_card"
        | "txclanxieshu:discard_cards"
        | "cxyMoQu:discard_card"
        | "cxyYingHun:discard_cards" => answer(best_cards_low(min.max(1))),
        "xinmingjian:choose_cards"
        | "tx_dqtianzi:choose_cards"
        | "txfeiyan:choose_sha"
        | "txkaikang:choose_card"
        | "txhj_qilinSkill2:choose_card"
        | "cxyJieLve:choose_h"
        | "cxyJieLve:choose_e"
        | "cxyJieLve:choose_j"
        | "txmouduan:move_card" => answer(options.iter().filter(|o| matches!(o, ChoiceOption::Card(_))).take(min.max(1)).copied().collect()),
        "txmouduan:move_from" | "txmouduan:move_to" => {
            answer(options.iter().take(min.max(1)).copied().collect())
        }
        _ => None,
    }
}
