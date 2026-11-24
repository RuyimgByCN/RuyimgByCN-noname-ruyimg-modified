//! Public-information-set AI heuristics for `L11` Part B (`l11_npc_b`) prompts.
//!
//! Strictly uses only `PolicyInput` / `PlayerView` public information (seat, public attitudes,
//! HP, max HP, equip/judge zones, hand counts, revealed/own cards) — never accesses hidden state.

use super::{extension_card_value, public_attitude};
use crate::catalog::Catalog;
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

fn player_opts(options: &[ChoiceOption]) -> Vec<u8> {
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .collect()
}

fn card_opts(options: &[ChoiceOption]) -> Vec<u32> {
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect()
}

fn pick_cards_sorted(
    catalog: &Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
    count: usize,
    highest_first: bool,
) -> Vec<ChoiceOption> {
    let mut cs = card_opts(options);
    cs.sort_by_key(|&c| {
        let v = extension_card_value(catalog, input, c);
        if highest_first { -v } else { v }
    });
    cs.into_iter().take(count).map(ChoiceOption::Card).collect()
}

fn pick_best_hostile(
    input: &PolicyInput,
    options: &[ChoiceOption],
    require_negative: bool,
) -> Option<u8> {
    let seat = input.view.seat;
    player_opts(options)
        .into_iter()
        .filter(|&p| !require_negative || public_attitude(&input.view, seat, p) < 0)
        .min_by_key(|&p| {
            let att = public_attitude(&input.view, seat, p);
            let hp = input.view.players.get(p as usize).map_or(99, |pl| pl.hp);
            (att, hp, p)
        })
}

fn pick_best_ally(
    input: &PolicyInput,
    options: &[ChoiceOption],
    require_positive: bool,
) -> Option<u8> {
    let seat = input.view.seat;
    player_opts(options)
        .into_iter()
        .filter(|&p| !require_positive || public_attitude(&input.view, seat, p) > 0)
        .max_by_key(|&p| {
            let att = public_attitude(&input.view, seat, p);
            let hp = input.view.players.get(p as usize).map_or(0, |pl| pl.hp);
            (att, -hp)
        })
}

fn pick_highest_rank(catalog: &Catalog, options: &[ChoiceOption]) -> Vec<ChoiceOption> {
    card_opts(options)
        .into_iter()
        .max_by_key(|&c| catalog.cards.get(c as usize).map_or(0, |cd| cd.rank))
        .into_iter()
        .map(ChoiceOption::Card)
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_npc_b_choice_action(
    catalog: &Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let ans = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };
    let my_hp = input.view.players.get(seat as usize).map_or(0, |p| p.hp);
    let my_max_hp = input.view.players.get(seat as usize).map_or(4, |p| p.max_hp as i32);

    match prompt {
        // ---- Confirms ----
        "sbliegong:confirm"
        | "spshiji:confirm"
        | "shefu2:confirm"
        | "twhengjiang:confirm"
        | "twliancai:draw_confirm"
        | "twlingfa:choose_mode"
        | "twqianxi:confirm"
        | "twyuhua:draw_confirm"
        | "xinfu_daigong:confirm"
        | "xinfu_zhaoxin:target_gain"
        | "xinfu_zhaoxin:damage_confirm"
        | "xinsheng:confirm"
        | "yise:recover_confirm"
        | "tianming:other_confirm" => {
            let want_yes = match prompt {
                "twqianxi:confirm" | "twyuhua:draw_confirm" | "twlingfa:choose_mode"
                | "xinsheng:confirm" | "xinfu_daigong:confirm" | "sbliegong:confirm"
                | "shefu2:confirm" | "spshiji:confirm" | "twhengjiang:confirm"
                | "twliancai:draw_confirm" => true,
                "tianming:other_confirm" => {
                    input.view.players.get(seat as usize).is_some_and(|p| p.hand_count <= 2 || p.hp <= 2)
                }
                "xinfu_zhaoxin:target_gain" => {
                    input.view.players.get(seat as usize).is_some_and(|p| p.hp > 1)
                }
                "xinfu_zhaoxin:damage_confirm" | "yise:recover_confirm" => true,
                _ => true,
            };
            let pick = if want_yes && options.contains(&ChoiceOption::Confirm) {
                ChoiceOption::Confirm
            } else if options.contains(&ChoiceOption::Pass) {
                ChoiceOption::Pass
            } else {
                *options.first()?
            };
            ans(vec![pick])
        }
        "twhongju:gain_saotao" => {
            let pick = if my_hp >= 3 && options.contains(&ChoiceOption::Confirm) {
                ChoiceOption::Confirm
            } else if options.contains(&ChoiceOption::Pass) {
                ChoiceOption::Pass
            } else {
                *options.first()?
            };
            ans(vec![pick])
        }

        // ---- Hostile single-target selectors ----
        "shangyi:choose_target"
        | "shensu1:choose_target"
        | "shensu2:choose_target"
        | "shizhi:choose_target"
        | "twwushen:choose_target"
        | "twmouli:choose_target"
        | "twqingce:choose_target"
        | "twfenming:choose_target"
        | "twliancai:choose_target"
        | "twqianxi:choose_target"
        | "twxiongxi:choose_target"
        | "twxuechang:choose_target"
        | "twxuanhuo:choose_duel_target"
        | "twzhengrong:choose_target"
        | "waishi:choose_target"
        | "weifeng:choose_target"
        | "weijing:choose_target"
        | "xinfenyue:choose_target"
        | "xinfu_kannan:choose_target"
        | "xinkuangfu:choose_target"
        | "xinkuangfu:use_sha_target"
        | "yechou:choose_target"
        | "twyuanhu:weapon_target"
        | "meiyingqiang:choose_target"
        | "zongkui:choose_target"
        | "spwenji:choose_target" => {
            if let Some(t) = pick_best_hostile(input, options, allow_cancel) {
                ans(vec![ChoiceOption::Player(t)])
            } else if allow_cancel {
                ans(Vec::new())
            } else {
                ans(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "twqirang:modify_target"
        | "twxiongzheng:choose_target"
        | "twlinglu:choose_target" => {
            if let Some(t) = pick_best_hostile(input, options, true)
                .or_else(|| (!allow_cancel).then(|| pick_best_hostile(input, options, false)).flatten())
            {
                ans(vec![ChoiceOption::Player(t)])
            } else if allow_cancel {
                ans(Vec::new())
            } else {
                ans(options.iter().take(min).copied().collect())
            }
        }

        // ---- Ally/support single-target selectors ----
        "shunshi:choose_target"
        | "tianshu:choose_target"
        | "twjiefan:choose_target"
        | "twjuntun:choose_target"
        | "twyingjia:choose_target"
        | "twyuanhu:choose_target"
        | "twxuanhuo:choose_target"
        | "xianfu:choose_target"
        | "xinfu_zengdao:choose_target"
        | "paiyi:choose_target"
        | "juedi:choose_target"
        | "zhiyan:choose_target"
        | "zhongyong:choose_target"
        | "zhuide:choose_target" => {
            if let Some(t) = pick_best_ally(input, options, allow_cancel) {
                ans(vec![ChoiceOption::Player(t)])
            } else if allow_cancel {
                ans(Vec::new())
            } else {
                ans(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "zhanyi_basic:choose_target" => {
            if let Some(t) = pick_best_hostile(input, options, false) {
                ans(vec![ChoiceOption::Player(t)])
            } else {
                ans(options.iter().take(min).copied().collect())
            }
        }

        // ---- Multi-player selectors ----
        "twwuhun:choose_targets" | "twxiongzheng:choose_sha_target" => {
            let mut enemies: Vec<ChoiceOption> = player_opts(options)
                .into_iter()
                .filter(|&p| public_attitude(&input.view, seat, p) < 0)
                .take(max)
                .map(ChoiceOption::Player)
                .collect();
            if enemies.len() < min {
                enemies = player_opts(options)
                    .into_iter()
                    .take(min)
                    .map(ChoiceOption::Player)
                    .collect();
            }
            ans(enemies)
        }
        "twxiongzheng:choose_draw_targets" | "wulie:choose_targets" => {
            let cap = if prompt == "wulie:choose_targets" {
                (my_hp - 1).max(0) as usize
            } else {
                max
            };
            let allies: Vec<ChoiceOption> = player_opts(options)
                .into_iter()
                .filter(|&p| public_attitude(&input.view, seat, p) > 0)
                .take(cap.min(max))
                .map(ChoiceOption::Player)
                .collect();
            if allies.len() < min {
                ans(options.iter().take(min).copied().collect())
            } else {
                ans(allies)
            }
        }
        "yjjianshu:choose_targets" => {
            let mut ps = player_opts(options);
            ps.sort_by_key(|&p| public_attitude(&input.view, seat, p));
            ans(ps.into_iter().take(2).map(ChoiceOption::Player).collect())
        }
        "xinjiewei_move:choose_src" | "yongjin:choose_src" => {
            let t = pick_best_hostile(input, options, false).or_else(|| player_opts(options).first().copied());
            ans(t.into_iter().take(max).map(ChoiceOption::Player).collect())
        }
        "xinjiewei_move:choose_dst" | "yongjin:choose_dst" => {
            let t = pick_best_ally(input, options, false).or_else(|| player_opts(options).first().copied());
            ans(t.into_iter().take(max).map(ChoiceOption::Player).collect())
        }

        // ---- Tag / Mode selectors ----
        "shefu:choose_name"
        | "twfenming:choose_mode"
        | "twzhian:choose_mode"
        | "twxiongzheng:choose_action"
        | "twyilie:choose_mode"
        | "weijing:choose_mode"
        | "zhanyi_basic:choose_kind" => ans(options.iter().take(1).copied().collect()),
        "zili:choose_mode" => {
            let pick = if my_hp == 1 && my_hp < my_max_hp && options.contains(&ChoiceOption::Tag(1)) {
                ChoiceOption::Tag(1)
            } else if options.contains(&ChoiceOption::Tag(0)) {
                ChoiceOption::Tag(0)
            } else {
                *options.first()?
            };
            ans(vec![pick])
        }
        "twxuhe:target_choose" => {
            let pick = if my_hp >= 3 && options.contains(&ChoiceOption::Tag(0)) {
                ChoiceOption::Tag(0)
            } else if options.contains(&ChoiceOption::Tag(1)) {
                ChoiceOption::Tag(1)
            } else {
                *options.first()?
            };
            ans(vec![pick])
        }
        "twxuanhuo:target_choose" => {
            let pick = options.first().copied()?;
            ans(vec![pick])
        }

        // ---- Pindian highest rank ----
        "twxuechang:pindian_owner"
        | "twxuechang:pindian_target"
        | "xinfenyue:pindian_target"
        | "xinfu_kannan:pindian_owner"
        | "xinfu_kannan:pindian_target"
        | "yjjianshu:pindian_first"
        | "yjjianshu:pindian_second" => ans(pick_highest_rank(catalog, options)),
        "xinfenyue:pindian_owner" => ans(pick_cards_sorted(catalog, input, options, 1, false)),

        // ---- Steal / discard opponent's best card ----
        "satomi_luodao:discard_shan"
        | "shangyi:discard_black"
        | "spyishi:choose_equip"
        | "twqingce:discard_card"
        | "twliancai:gain_equip"
        | "twzhengrong:choose_card"
        | "waishi:choose_target_cards"
        | "weifeng3:gain_card"
        | "twxuechang:gain_card"
        | "xinfenyue:gain_card"
        | "xinkuangfu:discard_equip"
        | "zhidao:choose_hand"
        | "zhidao:choose_equip"
        | "zhidao:choose_judge"
        | "zhiman:choose_card"
        | "twyuanhu:weapon_discard"
        | "xinjiewei_move:choose_card"
        | "yongjin:choose_card"
        | "twxuanhuo:take_cards" => {
            let take_n = max.min(options.len()).max(min);
            ans(pick_cards_sorted(catalog, input, options, take_n, true))
        }

        // ---- Own card discard / sacrifice / gift (lowest value) ----
        "satomi_luodao:discard_self"
        | "shefu:choose_card"
        | "shensu2:choose_card"
        | "shizhi:choose_card"
        | "shunshi:choose_card"
        | "tianming:discard"
        | "tianming:other_discard"
        | "tianshu:choose_card"
        | "tuifeng:choose_cards"
        | "twfenming:target_discard"
        | "twjiefan:discard_weapon"
        | "twlingfa:sha_discard"
        | "twlingfa:tao_give"
        | "twzhian:discard_hand"
        | "twqingce:choose_pile"
        | "twqianxi:discard"
        | "twxiongxi:discard"
        | "twxuanhuo:choose_cards"
        | "twyingjia:choose_card"
        | "twyuanhu:choose_equip"
        | "waishi:choose_cards"
        | "xinfu_daigong:give_card"
        | "xinfu_zengdao:choose_cards"
        | "xinfu_zhaoxin:choose_cards"
        | "xinfu_zhaoxin:choose_pile"
        | "xinjiewei_move:discard"
        | "xinjushou:choose_card"
        | "xinkuangfu:discard_two"
        | "yinbing:choose_cards"
        | "yongsi2:discard"
        | "yuce:show_card"
        | "yuce:source_discard"
        | "zhanyi:discard"
        | "zhanyi_basic:choose_card"
        | "zhanyi_equip:target_discard"
        | "yjjianshu:choose_card"
        | "spwenji:target_give"
        | "paiyi:choose_pile" => {
            let count = if min > 0 { min } else { 1.min(max).min(options.len()) };
            ans(pick_cards_sorted(catalog, input, options, count, false))
        }
        "zhendu:discard" => {
            let cur = input.view.turn;
            if cur != seat && public_attitude(&input.view, seat, cur) > 0 {
                ans(if allow_cancel {
                    Vec::new()
                } else {
                    options.iter().take(min).copied().collect()
                })
            } else {
                let count = if min > 0 { min } else { 1.min(max).min(options.len()) };
                ans(pick_cards_sorted(catalog, input, options, count, false))
            }
        }

        // ---- Offensive / useful card selection ----
        "twwushen:choose_card"
        | "twmouli:choose_card"
        | "meiyingqiang:choose_sha"
        | "xinkuangfu:use_sha_card"
        | "zhongyong:use_sha" => ans(pick_cards_sorted(catalog, input, options, 1.min(max), false)),

        "twhongju:swap_hand" => {
            let k = max.min(2).min(options.len());
            ans(pick_cards_sorted(catalog, input, options, k, false))
        }
        "twhongju:swap_pile" | "zhuide:choose_basics" => {
            let k = max.min(options.len()).max(min);
            ans(pick_cards_sorted(catalog, input, options, k, true))
        }
        "twyuhua:guanxing_top" => {
            let sel: Vec<ChoiceOption> = card_opts(options)
                .into_iter()
                .filter(|&c| extension_card_value(catalog, input, c) >= 4)
                .take(max)
                .map(ChoiceOption::Card)
                .collect();
            ans(sel)
        }
        _ => None,
    }
}
