//! U05 `杀海拾遗 · swd` 卡牌与卡牌技（第一半 U05-7a）询问的 AI 公开信息决策策略。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_u05_7a_choice_action(
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
    let pick_lowest_card = |threshold: Option<i64>| -> Vec<ChoiceOption> {
        let mut ranked: Vec<(i64, ChoiceOption)> = options
            .iter()
            .filter_map(|&opt| match opt {
                ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), opt)),
                _ => None,
            })
            .collect();
        ranked.sort_by_key(|(v, _)| *v);
        match ranked.first().copied() {
            Some((v, _)) if allow_cancel && threshold.is_some_and(|lim| v > lim) => Vec::new(),
            Some((_, opt)) => vec![opt],
            None => Vec::new(),
        }
    };
    let pick_enemy_target = || -> Vec<ChoiceOption> {
        let best = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            })
            .filter(|&t| !allow_cancel || public_attitude(&input.view, seat, t) < 0)
            .min_by_key(|&t| public_attitude(&input.view, seat, t));
        match best {
            Some(t) => vec![ChoiceOption::Player(t)],
            None if allow_cancel => Vec::new(),
            None => options.iter().take(1).copied().collect(),
        }
    };
    match prompt {
        "u05_dujian_own_card" | "u05_dujian_target_card" | "u05_pangufu_discard" => {
            let need = min.max(1);
            let mut ranked: Vec<(i64, ChoiceOption)> = options
                .iter()
                .filter_map(|&opt| match opt {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), opt)),
                    _ => None,
                })
                .collect();
            ranked.sort_by_key(|(v, _)| *v);
            Some(GameAction::Answer {
                choice_id: id,
                selection: ranked.into_iter().take(need).map(|(_, o)| o).collect(),
            })
        }
        "u05_hufu_sha_card" | "u05_hufu_jiu_card" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_lowest_card(None),
        }),
        "u05_huanglinzhicong_forging_discard" | "u05_zhuquezhizhang_forging_card" => {
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick_lowest_card(Some(50)),
            })
        }
        "u05_xuanwuzhihuang_forging_discard" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_lowest_card(Some(75)),
        }),
        "u05_yuchan_equip_discard" => {
            let mut ranked: Vec<(i64, ChoiceOption)> = options
                .iter()
                .filter_map(|&opt| match opt {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), opt)),
                    _ => None,
                })
                .filter(|(v, _)| *v <= 50)
                .collect();
            ranked.sort_by_key(|(v, _)| *v);
            let selection: Vec<ChoiceOption> = if ranked.len() >= min {
                ranked.into_iter().take(max).map(|(_, o)| o).collect()
            } else if allow_cancel {
                Vec::new()
            } else {
                pick_lowest_card(None)
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "u05_hufu_sha_target"
        | "u05_zhuquezhizhang_forging_target"
        | "u05_baishouzhihu_forging_target"
        | "baishouzhihu_target" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_enemy_target(),
        }),
        "u05_cangchizhibi_forging_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter_map(|t| {
                    let linked = input.view.players.get(t as usize).is_some_and(|p| p.linked);
                    let att = public_attitude(&input.view, seat, t);
                    let gain = if linked && att > 0 {
                        att
                    } else if !linked && att < 0 {
                        -att
                    } else {
                        return None;
                    };
                    Some((gain, t))
                })
                .max_by_key(|(gain, _)| *gain);
            let selection = match best {
                Some((_, t)) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        _ => None,
    }
}
