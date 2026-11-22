//! L11（太虚幻境）Batch 4 挂起询问的公开信息集 AI 启发式策略。
//!
//! - `txhj_luanjifengshi:discard_card`（`extension_skill.js:2532`）：仅看自身可见牌，
//!   无【闪】或仅有一张唯一【闪】且无其余可弃牌时放弃弃牌（避免白丢唯一【闪】或白丢牌）；
//!   若有其余低价值手牌/装备则弃置最低价值非【闪】牌或富余【闪】以抵消万箭齐发直击。
//! - `txhj_suikuiliejia:discard_armor`（`extension_skill.js:2754`）：仅依据公开身份/阵营与桌面装备，
//!   对敌方最高价值防具选择弃置，对友方防具选择取消。
//! - `txhj_renigongdeyouhu:confirm`（`extension_buff.js:799`）：确认发动额外摸牌。

use super::{extension_card_value, public_attitude};
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_b4_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    _min: usize,
    _max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let answer = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };

    match prompt {
        "txhj_luanjifengshi:discard_card" => {
            let candidate_cards: Vec<u32> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            let shan_cards: Vec<u32> = input
                .own_hand_kinds
                .iter()
                .filter(|(c, k)| *k == CardKind::Shan && candidate_cards.contains(c))
                .map(|(c, _)| *c)
                .collect();
            let shan_count = shan_cards.len();
            if shan_count == 0 {
                return answer(Vec::new());
            }
            if shan_count == 1 {
                let only_shan = shan_cards[0];
                if let Some(non_shan) = candidate_cards
                    .iter()
                    .copied()
                    .filter(|&c| c != only_shan)
                    .min_by_key(|&c| extension_card_value(catalog, input, c))
                {
                    return answer(vec![ChoiceOption::Card(non_shan)]);
                }
                return answer(Vec::new());
            }
            let pick = candidate_cards.iter().copied().min_by_key(|&c| {
                let base = extension_card_value(catalog, input, c);
                if shan_cards.contains(&c) {
                    base + 200
                } else {
                    base
                }
            });
            if let Some(c) = pick {
                answer(vec![ChoiceOption::Card(c)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                None
            }
        }
        "txhj_suikuiliejia:discard_armor" => {
            let pick = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .filter(|&c| {
                    input.view.players.iter().any(|p| {
                        p.equips.contains(&c) && public_attitude(&input.view, seat, p.seat) < 0
                    })
                })
                .max_by_key(|&c| extension_card_value(catalog, input, c));
            if let Some(c) = pick {
                answer(vec![ChoiceOption::Card(c)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                None
            }
        }
        "txhj_renigongdeyouhu:confirm" => {
            if options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                None
            }
        }
        _ => None,
    }
}
