//! L11（太虚幻境）Batch 3 挂起询问的公开信息集 AI 启发式策略。
//!
//! - `txhj_juxiangzhifeng:discard_card`（`extension_skill.js:2513`）：仅看自身可见牌，
//!   无【杀】或仅有一张唯一【杀】且无弃置保【杀】可能时放弃弃牌（避免白丢牌）；
//!   若有其余低价值手牌/装备则弃置最低价值非【杀】牌或富余【杀】以抵消南蛮入侵直击。
//! - `txhj_kapaikeshi:choose_card`（`extension_skill.js:3286` `viewHandcard`）：
//!   从当前视野中实际公开可见的目标手牌/装备/判定牌中拣选最高价值目标牌（不探测隐藏域）。

use super::extension_card_value;
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::{ChoiceView, HandView, PolicyInput};

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_b3_choice_action(
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
        "txhj_juxiangzhifeng:discard_card" => {
            let candidate_cards: Vec<u32> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            let sha_cards: Vec<u32> = input
                .own_hand_kinds
                .iter()
                .filter(|(c, k)| {
                    matches!(k, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha)
                        && candidate_cards.contains(c)
                })
                .map(|(c, _)| *c)
                .collect();
            let sha_count = sha_cards.len();
            if sha_count == 0 {
                return answer(Vec::new());
            }
            if sha_count == 1 {
                let only_sha = sha_cards[0];
                if let Some(non_sha) = candidate_cards
                    .iter()
                    .copied()
                    .filter(|&c| c != only_sha)
                    .min_by_key(|&c| extension_card_value(catalog, input, c))
                {
                    return answer(vec![ChoiceOption::Card(non_sha)]);
                }
                return answer(Vec::new());
            }
            // 手中有 2+ 张【杀】：先弃最低价值非【杀】牌，无非【杀】再弃余【杀】
            let pick = candidate_cards
                .iter()
                .copied()
                .min_by_key(|&c| {
                    let base = extension_card_value(catalog, input, c);
                    if sha_cards.contains(&c) {
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
        "txhj_kapaikeshi:choose_card" => {
            // 仅从视图可见集合（他家 `HandView::Own` 与公开装备/判定区）按公开信息计值
            let mut visible: Vec<u32> = Vec::new();
            for p in &input.view.players {
                if p.seat == seat {
                    continue;
                }
                if let HandView::Own(ref h) = p.hand {
                    visible.extend(h.iter().copied());
                }
                visible.extend(p.equips.iter().copied());
                visible.extend(p.judge.iter().copied());
            }
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .max_by_key(|&c| {
                    if visible.contains(&c) {
                        extension_card_value(catalog, input, c)
                    } else {
                        0
                    }
                });
            best.map(|c| answer(vec![ChoiceOption::Card(c)]))?
        }
        "trigger_confirm" => match input.choice.as_ref().and_then(|c| match c {
            ChoiceView::Own { source_skill, .. } => *source_skill,
            _ => None,
        }) {
            Some("buff_txhj_yingjicuoshijia" | "_buff_txhj_yingjichuoshijia") => {
                let injured = input
                    .view
                    .players
                    .iter()
                    .find(|p| p.seat == seat)
                    .is_some_and(|p| p.hp < p.max_hp as i32);
                if injured {
                    answer(vec![ChoiceOption::Confirm])
                } else {
                    answer(vec![ChoiceOption::Pass])
                }
            }
            Some("buff_txhj_yingjicuoshiyi" | "_buff_txhj_yingjichuoshiyi") => {
                answer(vec![ChoiceOption::Confirm])
            }
            _ => None,
        },
        _ => None,
    }
}
