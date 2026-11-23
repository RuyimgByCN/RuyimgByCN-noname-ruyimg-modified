//! Batch U05-1 公开信息集 AI 选择策略（严禁读取对手私密手牌内容）。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

fn pick_hostile_player(input: &PolicyInput, options: &[ChoiceOption], seat: u8) -> Option<ChoiceOption> {
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .min_by_key(|&p| {
            let att = public_attitude(&input.view, seat, p);
            let hp = input.view.players.get(p as usize).map(|pl| pl.hp).unwrap_or(9);
            (att, hp)
        })
        .map(ChoiceOption::Player)
}

fn pick_friendly_player(input: &PolicyInput, options: &[ChoiceOption], seat: u8) -> Option<ChoiceOption> {
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .max_by_key(|&p| public_attitude(&input.view, seat, p))
        .map(ChoiceOption::Player)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_u05_b1_choice_action(
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
    let selection = match prompt {
        "u05_mtg_duzhao_target"
        | "u05_mhuanyi_target"
        | "u05_shaoying_target"
        | "u05_luweiyan2_target"
        | "u05_msuoling_target" => pick_hostile_player(input, options, seat)
            .into_iter()
            .collect(),
        "u05_qiaosi_target" => pick_friendly_player(input, options, seat)
            .into_iter()
            .collect(),
        "u05_qiaosi_mode" => {
            let has_ally = input
                .view
                .players
                .iter()
                .any(|p| p.seat != seat && p.alive && public_attitude(&input.view, seat, p.seat) > 0);
            let want = if has_ally && options.contains(&ChoiceOption::Tag(0)) {
                ChoiceOption::Tag(0)
            } else {
                *options.last()?
            };
            vec![want]
        }
        "u05_mhuanyi_kind" | "u05_mtg_cangbaohaiwan_pack" | "u05_lingyong_summon" => {
            options.first().copied().into_iter().collect()
        }
        "u05_msilian_confirm" | "u05_shezhan_respond" => {
            if options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else {
                vec![ChoiceOption::Pass]
            }
        }
        "u05_zhenfa_recast_card"
        | "u05_mtg_lindixiliu_heart"
        | "u05_mtg_lindixiliu_judge"
        | "u05_mtg_bingheyaosai_sha_discard"
        | "u05_msuoling_discard"
        | "u05_jieyan_red"
        | "u05_zonghuo_discard"
        | "u05_diyguhuo_discard"
        | "u05_qiaosi_cards"
        | "u05_qiaosi_discard"
        | "u05_xinfu_jingxie_card"
        | "u05_beige_club_discard"
        | "beige_discard"
        | "u05_shezhan_own_pick"
        | "u05_shezhan_target_pick"
        | "u05_shuigong_discard"
        | "u05_zhenfa_huyizhen_card" => {
            let mut ranked: Vec<crate::ids::CardId> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            ranked.sort_by_key(|&c| extension_card_value(catalog, input, c));
            let need = min.max(1).min(max.max(1));
            ranked
                .into_iter()
                .take(need)
                .map(ChoiceOption::Card)
                .collect()
        }
        "u05_zhenfa_longfei_discard"
        | "swd_qixingzhen_discard"
        | "u05_chenhuodajie_gain"
        | "g_chenhuodajie_use"
        | "u05_caochuan_visible_discard"
        | "u05_gean_winner_gain"
        | "u05_gean_tie_discard" => {
            let need = min.max(1);
            options.iter().take(need).copied().collect()
        }
        _ => return None,
    };
    if selection.is_empty() && !allow_cancel && !options.is_empty() {
        return Some(GameAction::Answer {
            choice_id: id,
            selection: options.iter().take(min.max(1)).copied().collect(),
        });
    }
    Some(GameAction::Answer {
        choice_id: id,
        selection,
    })
}
