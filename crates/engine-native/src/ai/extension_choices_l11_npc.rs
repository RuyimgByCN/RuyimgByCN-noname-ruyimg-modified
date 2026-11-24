//! Batch L11-6a 太虚自有 NPC 技能（`HostBehavior::L11OwnNpc`）的 AI 公开信息决策。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_npc_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let me_hp = input.view.players.get(seat as usize).map_or(1, |p| p.hp);
    let pick_enemy = || -> Option<u8> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            })
            .filter(|&t| public_attitude(&input.view, seat, t) < 0)
            .min_by_key(|&t| {
                let p = input.view.players.get(t as usize);
                (
                    p.map(|x| x.hp).unwrap_or(99),
                    p.map(|x| x.hand_count).unwrap_or(99),
                    public_attitude(&input.view, seat, t),
                )
            })
    };
    let pick_ally = || -> Option<u8> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            })
            .filter(|&t| public_attitude(&input.view, seat, t) >= 0)
            .max_by_key(|&t| public_attitude(&input.view, seat, t))
    };
    match prompt {
        "txhj_huangjin:confirm" | "txqyyouji:confirm" => Some(GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Confirm],
        }),
        "txhj_tunshi:choose_target"
        | "txhj_chuanyun:choose_target"
        | "txhj_guihuo:choose_target"
        | "txhj_luolei:choose_target"
        | "txhj_guimen:choose_target"
        | "txhj_renfang:choose_target"
        | "txhj_guiyan:choose_target"
        | "txhj_zhouzu:choose_target"
        | "txhj_yaoshu:choose_target"
        | "txhj_zhoufa:choose_target" => {
            let selection = match pick_enemy() {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "txhj_zhuhun:choose_target" => {
            let selection = match pick_ally() {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "txhj_zhouzu:choose_duration"
        | "txhj_yaoshu:choose_duration"
        | "txhj_zhoufa:choose_duration"
        | "txhj_renfang:choose_duration" => {
            let pick = if prompt == "txhj_zhoufa:choose_duration" || prompt == "txhj_renfang:choose_duration" {
                if me_hp >= 4 {
                    options.last().copied()
                } else {
                    options.first().copied()
                }
            } else {
                options.first().copied()
            };
            let _ = ChoiceOption::Tag(0);
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick.into_iter().collect(),
            })
        }
        "txhj_zhouzu:discard_cards"
        | "txhj_xiefeng:discard_cards"
        | "txhj_dizhou:choose_card" => {
            let mut ranked: Vec<(i64, ChoiceOption)> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, c), *o)),
                    _ => None,
                })
                .collect();
            ranked.sort_by_key(|(v, _)| *v);
            let want = min.max(1);
            let selection = if ranked.len() >= want {
                ranked.into_iter().take(want).map(|(_, o)| o).collect()
            } else if allow_cancel {
                Vec::new()
            } else {
                ranked.into_iter().map(|(_, o)| o).collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        _ => None,
    }
}
