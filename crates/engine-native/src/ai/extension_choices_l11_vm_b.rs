//! L11-E2 `taixu` VM Fix B（`P3-E2`）公开信息集 AI 选择策略（`extension_choices_l11_vm_b.rs`）：
//! 仅使用 `input.view` / `input.own_hand_kinds` / `options` 公开信息，不读取他家暗手牌。

use super::extension_choices_common::{extension_card_value, public_attitude};
use crate::catalog::CardKind;
use crate::choice::ChoiceOption;
use crate::ids::{CardId, PlayerId};
use crate::view::ChoiceView;
use crate::GameAction;

fn ans(id: u64, selection: Vec<ChoiceOption>) -> Option<GameAction> {
    Some(GameAction::Answer {
        choice_id: id,
        selection,
    })
}

fn own_card_rank(catalog: &crate::catalog::Catalog, card: CardId) -> u8 {
    catalog.cards.get(card as usize).map_or(0, |d| d.rank)
}

fn pick_lowest_value_cards(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut cards: Vec<CardId> = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| (extension_card_value(catalog, input, c), own_card_rank(catalog, c), c));
    cards.into_iter().take(count).map(ChoiceOption::Card).collect()
}

fn pick_highest_rank_card(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
) -> Vec<ChoiceOption> {
    options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .max_by_key(|&c| {
            (
                own_card_rank(catalog, c),
                -extension_card_value(catalog, input, c),
            )
        })
        .map(|c| vec![ChoiceOption::Card(c)])
        .unwrap_or_default()
}

fn pick_players_by_att(
    input: &crate::view::PolicyInput,
    options: &[ChoiceOption],
    prefer_enemy: bool,
    only_preferred_if_cancel: bool,
    allow_cancel: bool,
    count: usize,
) -> Vec<ChoiceOption> {
    let seat = input.view.seat;
    let mut cands: Vec<(i32, usize, PlayerId)> = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => {
                let att = public_attitude(&input.view, seat, *p);
                let hand_len = input
                    .view
                    .players
                    .get(*p as usize)
                    .map_or(0, |pl| pl.hand_count);
                let score = if prefer_enemy { -att } else { att };
                Some((score, hand_len, *p))
            }
            _ => None,
        })
        .collect();
    cands.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(&b.2)));
    if allow_cancel && only_preferred_if_cancel {
        cands.retain(|c| c.0 > 0);
    }
    cands
        .into_iter()
        .take(count)
        .map(|(_, _, p)| ChoiceOption::Player(p))
        .collect()
}

fn pick_hidden_or_equip(options: &[ChoiceOption], count: usize) -> Vec<ChoiceOption> {
    let mut out: Vec<ChoiceOption> = options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .chain(
            options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::HiddenCard(_)))
                .copied(),
        )
        .take(count)
        .collect();
    out.truncate(count);
    out
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_vm_fix_b_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    prompt: &'static str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let me = input.view.players.get(seat as usize);
    let my_hp = me.map_or(0, |p| p.hp);
    let my_max_hp = me.map_or(0, |p| p.max_hp as i32);

    match prompt {
        "trigger_confirm" => match input.choice.as_ref().and_then(|c| match c {
            ChoiceView::Own { source_skill, .. } => *source_skill,
            _ => None,
        }) {
            Some(
                "rebiyue"
                | "rejianxiong"
                | "qiangzhi_draw"
                | "spdengli"
                | "xinliegong"
                | "yiyong"
                | "zaiqi",
            ) => ans(id, vec![ChoiceOption::Confirm]),
            Some("qingxi") => {
                let confirm = public_attitude(&input.view, seat, input.view.turn) <= 0
                    || input
                        .view
                        .players
                        .iter()
                        .any(|p| p.seat != seat && p.alive && public_attitude(&input.view, seat, p.seat) <= 0);
                ans(
                    id,
                    vec![if confirm {
                        ChoiceOption::Confirm
                    } else {
                        ChoiceOption::Pass
                    }],
                )
            }
            Some("spchijie") => {
                let confirm = public_attitude(&input.view, seat, input.view.turn) <= 0;
                ans(
                    id,
                    vec![if confirm {
                        ChoiceOption::Confirm
                    } else {
                        ChoiceOption::Pass
                    }],
                )
            }
            _ => None,
        },
        "l11_vm_b:pindian_card" | "l11_vm_b:target_pindian_card" => {
            ans(id, pick_highest_rank_card(catalog, input, options))
        }
        "l11_vm_b:quhu_target"
        | "l11_vm_b:rexianzhen_target"
        | "l11_vm_b:tianyi_target"
        | "l11_vm_b:quhu_damage_target"
        | "l11_vm_b:refanjian_target"
        | "l11_vm_b:reluanwu_sha_target"
        | "l11_vm_b:reluanwu_after_sha"
        | "l11_vm_b:releiji_target"
        | "l11_vm_b:tiaoxin_target"
        | "l11_vm_b:xinfu_qinguo_sha"
        | "l11_vm_b:rezhiyi_target"
        | "l11_vm_b:zhiyi_target" => {
            ans(id, pick_players_by_att(input, options, true, true, allow_cancel, 1))
        }
        "l11_vm_b:rejieming_target"
        | "l11_vm_b:zhengjian_target"
        | "l11_vm_b:tiaojiyanmei_gain_target" => {
            ans(id, pick_players_by_att(input, options, false, true, allow_cancel, 1))
        }
        "l11_vm_b:repolu_targets" => {
            let sel = pick_players_by_att(input, options, false, true, allow_cancel, max);
            ans(id, sel)
        }
        "l11_vm_b:sphuangen_targets" => {
            let sel = pick_players_by_att(input, options, false, true, allow_cancel, max);
            ans(id, sel)
        }
        "l11_vm_b:qiangzhi_target" => {
            ans(id, pick_players_by_att(input, options, true, false, allow_cancel, 1))
        }
        "l11_vm_b:yinghun_target" => {
            let sel = pick_players_by_att(input, options, false, true, allow_cancel, 1);
            if !sel.is_empty() {
                ans(id, sel)
            } else {
                ans(id, pick_players_by_att(input, options, true, true, allow_cancel, 1))
            }
        }
        "l11_vm_b:yinghun_mode" => {
            let has_injured_ally = input
                .view
                .players
                .iter()
                .any(|p| p.seat != seat && p.alive && public_attitude(&input.view, seat, p.seat) > 0);
            let want = if has_injured_ally { 0 } else { 1 };
            let tag = if options.contains(&ChoiceOption::Tag(want)) {
                want
            } else {
                0
            };
            ans(id, vec![ChoiceOption::Tag(tag)])
        }
        "l11_vm_b:refanjian_card" => {
            ans(id, pick_lowest_value_cards(catalog, input, options, 1))
        }
        "l11_vm_b:refanjian_mode" => {
            let tag = if my_hp <= 1 || input.own_hand_kinds.len() <= 1 {
                0
            } else if my_hp >= 3 {
                1
            } else {
                0
            };
            ans(id, vec![ChoiceOption::Tag(tag)])
        }
        "l11_vm_b:reluanwu_use_sha" => {
            let has_enemy = input
                .view
                .players
                .iter()
                .any(|p| p.seat != seat && p.alive && public_attitude(&input.view, seat, p.seat) <= 0);
            if has_enemy || my_hp <= 1 || !allow_cancel {
                ans(id, pick_lowest_value_cards(catalog, input, options, 1))
            } else {
                ans(id, Vec::new())
            }
        }
        "l11_vm_b:tiaoxin_use_sha" => {
            let att = public_attitude(&input.view, seat, input.view.turn);
            if att <= 0 || !allow_cancel {
                ans(id, pick_lowest_value_cards(catalog, input, options, 1))
            } else {
                ans(id, Vec::new())
            }
        }
        "l11_vm_b:tiaoxin_discard"
        | "l11_vm_b:qiangzhi_card"
        | "l11_vm_b:reshejian_pick" => ans(id, pick_hidden_or_equip(options, min)),
        "l11_vm_b:qingxi_discard" => {
            if allow_cancel && my_hp >= 2 && options.len() <= min + 1 {
                ans(id, Vec::new())
            } else {
                ans(id, pick_lowest_value_cards(catalog, input, options, min))
            }
        }
        "l11_vm_b:rebeige_discard" => {
            if allow_cancel && options.len() <= 1 && my_hp <= 2 {
                ans(id, Vec::new())
            } else {
                ans(id, pick_lowest_value_cards(catalog, input, options, 1))
            }
        }
        "l11_vm_b:rebeige_source_discard"
        | "l11_vm_b:shenfen_discard"
        | "l11_vm_b:yinghun_discard"
        | "l11_vm_b:tiaojiyanmei_discard" => {
            ans(id, pick_lowest_value_cards(catalog, input, options, min))
        }
        "l11_vm_b:rejiangchi_discard" => {
            ans(id, pick_lowest_value_cards(catalog, input, options, 1))
        }
        "l11_vm_b:rejiangchi_mode" => {
            let has_sha = input.own_hand_kinds.iter().any(|(_, k)| k.is_sha_like());
            let want = if has_sha && options.contains(&ChoiceOption::Tag(0)) {
                0
            } else {
                1
            };
            if options.contains(&ChoiceOption::Tag(want)) {
                ans(id, vec![ChoiceOption::Tag(want)])
            } else {
                ans(id, options.first().copied().into_iter().collect())
            }
        }
        "l11_vm_b:reshejian_discard" => {
            if allow_cancel && input.own_hand_kinds.len() <= 2 {
                ans(id, Vec::new())
            } else {
                ans(id, pick_lowest_value_cards(catalog, input, options, min))
            }
        }
        "l11_vm_b:reshejian_mode" => {
            let tag = if options.contains(&ChoiceOption::Tag(1)) {
                1
            } else {
                0
            };
            ans(id, vec![ChoiceOption::Tag(tag)])
        }
        "l11_vm_b:rezhiyi_kind" => {
            let pref = if my_hp < my_max_hp {
                &[3u8, 0, 1, 2, 4][..]
            } else {
                &[0u8, 1, 2, 4, 3][..]
            };
            let pick = pref
                .iter()
                .find(|&&t| options.contains(&ChoiceOption::Tag(t)))
                .copied();
            ans(id, pick.map(|t| vec![ChoiceOption::Tag(t)]).unwrap_or_default())
        }
        "l11_vm_b:zhiyi_mode" => {
            let has_tao_inj = my_hp < my_max_hp
                && input
                    .own_hand_kinds
                    .iter()
                    .any(|(_, k)| *k == CardKind::Tao);
            let want = if has_tao_inj && options.contains(&ChoiceOption::Tag(1)) {
                1
            } else {
                0
            };
            ans(id, vec![ChoiceOption::Tag(want)])
        }
        "l11_vm_b:twshengxi_zhunbei"
        | "l11_vm_b:xinfu_qinguo_recover"
        | "l11_vm_b:zhuandui_confirm" => ans(id, vec![ChoiceOption::Confirm]),
        "l11_vm_b:twshengxi_zhinang" => {
            let pick = [0u8, 2, 1]
                .into_iter()
                .find(|t| options.contains(&ChoiceOption::Tag(*t)));
            ans(id, pick.map(|t| vec![ChoiceOption::Tag(t)]).unwrap_or_default())
        }
        _ => None,
    }
}
