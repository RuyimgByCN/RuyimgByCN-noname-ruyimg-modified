//! U05 `杀海拾遗 · swd` 卡牌与卡牌技（第二半 U05-7b）询问的 AI 公开信息决策策略。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_u05_7b_choice_action(
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
    let pick_cards_lowest = |need: usize, threshold: Option<i64>| -> Vec<ChoiceOption> {
        let mut ranked: Vec<(i64, ChoiceOption)> = options
            .iter()
            .filter_map(|&opt| match opt {
                ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), opt)),
                _ => None,
            })
            .collect();
        ranked.sort_by_key(|(v, _)| *v);
        if allow_cancel
            && threshold.is_some_and(|lim| ranked.first().is_none_or(|(v, _)| *v > lim))
        {
            return Vec::new();
        }
        ranked.into_iter().take(need).map(|(_, o)| o).collect()
    };
    let pick_cards_highest = || -> Vec<ChoiceOption> {
        options
            .iter()
            .filter_map(|&opt| match opt {
                ChoiceOption::Card(c) => Some((catalog.cards[c as usize].rank, opt)),
                _ => None,
            })
            .max_by_key(|(r, _)| *r)
            .map(|(_, o)| vec![o])
            .unwrap_or_default()
    };
    let pick_enemy = || -> Vec<ChoiceOption> {
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
    let pick_friend = || -> Vec<ChoiceOption> {
        let best = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            })
            .filter(|&t| !allow_cancel || public_attitude(&input.view, seat, t) > 0)
            .max_by_key(|&t| public_attitude(&input.view, seat, t));
        match best {
            Some(t) => vec![ChoiceOption::Player(t)],
            None if allow_cancel => Vec::new(),
            None => options.iter().take(1).copied().collect(),
        }
    };
    match prompt {
        "u05_xiangyuye_discard" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_lowest(min.max(1), Some(75)),
        }),
        "u05_kongxin_pindian_own" | "u05_kongxin_pindian_target" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_highest(),
        }),
        "u05_kongxin_target"
        | "u05_kongxin_sha_target"
        | "u05_lingjian_e5_target"
        | "u05_old_longfan_target"
        | "u05_jileng_equip5_target" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_enemy(),
        }),
        "u05_yinmi_equip1_target" | "u05_duanzao_target" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_friend(),
        }),
        "u05_kongxin_lose_discard"
        | "u05_lingjian_e5_cost"
        | "u05_duanzao_equip"
        | "u05_duanzao_mat" => Some(GameAction::Answer {
            choice_id: id,
            selection: pick_cards_lowest(min.max(1), None),
        }),
        "u05_qinglonglingzhu_gain" => Some(GameAction::Answer {
            choice_id: id,
            selection: options
                .iter()
                .filter_map(|&o| match o {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), o)),
                    _ => None,
                })
                .max_by_key(|(v, _)| *v)
                .map(|(_, o)| vec![o])
                .unwrap_or_default(),
        }),
        "u05_zhongxing_equip1_confirm"
        | "u05_zhongxing_equip2_confirm"
        | "u05_baihupifeng_recover"
        | "u05_jileng_equip1_confirm"
        | "u05_jileng_equip2_confirm" => Some(GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Confirm],
        }),
        "u05_fengxueren_confirm" | "u05_guiyanfadao_confirm" => Some(GameAction::Answer {
            choice_id: id,
            selection: Vec::new(),
        }),
        _ => None,
    }
}
