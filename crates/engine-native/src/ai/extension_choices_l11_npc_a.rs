//! L11（太虚幻境）Part A Cross NPC 挂起询问的公开信息集 AI 启发式策略。
//!
//! 仅消费 `PolicyInput` / `PlayerView` 公开信息，不读取未公开私密牌或随机状态。

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

fn best_player_by_att(
    input: &PolicyInput,
    options: &[ChoiceOption],
    want_enemy: bool,
) -> Option<u8> {
    let seat = input.view.seat;
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .max_by_key(|&p| {
            let att = public_attitude(&input.view, seat, p);
            if want_enemy { -att } else { att }
        })
}

fn is_negative_target(input: &PolicyInput, p: u8) -> bool {
    public_attitude(&input.view, input.view.seat, p) < 0
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_npc_a_choice_action(
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
    let pass_or_first = || {
        if allow_cancel {
            answer(Vec::new())
        } else {
            answer(options.iter().take(min.max(1)).copied().collect())
        }
    };

    match prompt {
        // 明确有益己方的确认
        "jici:confirm"
        | "new_yijue:confirm_recover"
        | "rezhijian:confirm_draw"
        | "bmcanshi:cancel_confirm"
        | "chouce:confirm"
        | "nzry_shicai:confirm"
        | "rexingluan:confirm"
        | "reyajiao:confirm"
        | "diancai:confirm"
        | "jigong:confirm"
        | "piaoling:confirm"
        | "qingzhongx:confirm"
        | "reguanxing:confirm"
        | "reluoyi:confirm"
        | "reshuangxiong:confirm"
        | "reshuangxiong:gain_shas_confirm"
        | "olniepan:confirm"
        | "requanji:confirm"
        | "retuntian:confirm" => {
            if options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else {
                answer(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "nzry_kuizhu:choose_mode" => {
            if options.contains(&ChoiceOption::Tag(0)) {
                answer(vec![ChoiceOption::Tag(0)])
            } else {
                pass_or_first()
            }
        }
        "nzry_kuizhu:damage_targets" => pass_or_first(),
        "jsrgshichong:confirm" | "fenji:confirm" | "nsxianhai:confirm" => {
            let want = match prompt {
                "fenji:confirm" => {
                    input.view.players.get(seat as usize).is_some_and(|p| p.hp > 1)
                }
                "nsxianhai:confirm" => {
                    input.view.players.get(seat as usize).is_some_and(|p| p.max_hp > 1)
                }
                _ => true,
            };
            if want && options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else {
                pass_or_first()
            }
        }
        // 选择敌人单目标/多目标
        "duwu:choose_target"
        | "fenxun:choose_target"
        | "guanzong:choose_target"
        | "gushe:choose_targets"
        | "jinglve:choose_target"
        | "lihun:choose_target"
        | "mouzhu:choose_target"
        | "new_yijue:choose_target"
        | "nzry_feijun:choose_target"
        | "refenxun:choose_target"
        | "rejunxing:choose_target"
        | "rewurong:choose_target"
        | "kyou_zhidian:choose_target"
        | "olhuoji:choose_target"
        | "reshuangxiong:choose_target"
        | "bmcanshi:extra_targets"
        | "chouce:discard_target"
        | "leiji:choose_target"
        | "liyu:choose_duel_target"
        | "qingxian:choose_target"
        | "rexingluan:choose_target"
        | "rejuece:choose_target"
        | "retuxi:choose_targets"
        | "rexuanfeng:discard_target"
        | "rexuanfeng:move_from"
        | "renshe:faction_target" => {
            let enemies: Vec<ChoiceOption> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) if is_negative_target(input, p) => Some(*o),
                    _ => None,
                })
                .take(max.max(1))
                .collect();
            if enemies.len() >= min {
                if !enemies.is_empty() {
                    return answer(enemies);
                }
                return pass_or_first();
            }
            if let Some(p) = best_player_by_att(input, options, true)
                .filter(|&p| is_negative_target(input, p) || !allow_cancel)
            {
                answer(vec![ChoiceOption::Player(p)])
            } else {
                pass_or_first()
            }
        }
        // 选择友方目标
        "jujian:choose_target"
        | "lianzhu:choose_target"
        | "mingjian:choose_target"
        | "mizhao:choose_target1"
        | "new_reqingnang:choose_target"
        | "quji:choose_targets"
        | "rezhijian:choose_target"
        | "rongbei:choose_target"
        | "chouce:draw_target"
        | "reyajiao:give_target"
        | "nzry_kuizhu:draw_targets"
        | "piaoling:choose_target"
        | "rezaiqi:choose_targets"
        | "chenqing:choose_target"
        | "lirang:choose_target"
        | "relianying:choose_targets"
        | "renshe:draw_target"
        | "rexuanfeng:move_to" => {
            let allies: Vec<ChoiceOption> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) if !is_negative_target(input, p) => Some(*o),
                    _ => None,
                })
                .take(max.max(1))
                .collect();
            if allies.len() >= min && !allies.is_empty() {
                return answer(allies);
            }
            if let Some(p) = best_player_by_att(input, options, false)
                .filter(|&p| !is_negative_target(input, p) || !allow_cancel)
            {
                answer(vec![ChoiceOption::Player(p)])
            } else {
                pass_or_first()
            }
        }
        "reguose:choose_target" => {
            let has_le = |p: u8| {
                input.view.players.get(p as usize).is_some_and(|pv| {
                    pv.judge_kinds
                        .contains(&crate::catalog::CardKind::Le)
                })
            };
            if let Some(p) = options.iter().find_map(|o| match *o {
                ChoiceOption::Player(p) if !is_negative_target(input, p) && has_le(p) => Some(p),
                _ => None,
            }) {
                return answer(vec![ChoiceOption::Player(p)]);
            }
            if let Some(p) = options.iter().find_map(|o| match *o {
                ChoiceOption::Player(p) if is_negative_target(input, p) && !has_le(p) => Some(p),
                _ => None,
            }) {
                return answer(vec![ChoiceOption::Player(p)]);
            }
            pass_or_first()
        }
        // 其他角色或双目标混合类
        "baiyi:choose_targets"
        | "guanzong:choose_source"
        | "mizhao:choose_target2"
        | "reganlu:choose_target1"
        | "reganlu:choose_target2"
        | "gaoyuan:choose_target"
        | "retianxiang:choose_target"
        | "mozhi:choose_target"
        | "qingzhongx:choose_target" => {
            if let Some(p) = best_player_by_att(input, options, true) {
                if max == 1 {
                    return answer(vec![ChoiceOption::Player(p)]);
                }
            }
            let picked: Vec<ChoiceOption> = options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Player(_)))
                .take(min.max(1))
                .copied()
                .collect();
            if picked.len() >= min {
                answer(picked)
            } else {
                pass_or_first()
            }
        }
        // 拼点选牌 / 主动选最大或最小牌
        "gushe:owner_pindian"
        | "gushe:target_pindian"
        | "mizhao:pindian_card1"
        | "mizhao:pindian_card2" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .max_by_key(|&c| {
                    catalog
                        .cards
                        .get(c as usize)
                        .map_or(0, |d| d.rank)
                })
                .map(ChoiceOption::Card);
            if let Some(c) = best {
                answer(vec![c])
            } else {
                pass_or_first()
            }
        }
        // 观星全放牌堆底或选高价值留顶；傲才/无牌弃选按公开牌低值或首个合法项
        "reguanxing:to_top" => answer(Vec::new()),
        "aocai:choose_card" => {
            let best = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
                .copied();
            if let Some(c) = best {
                answer(vec![c])
            } else {
                pass_or_first()
            }
        }
        // 模式 Tag 选择
        "mouzhu:choose_mode"
        | "nzry_feijun:choose_mode"
        | "rejunxing:target_mode"
        | "benyu:choose_mode"
        | "dcmoukui:choose_mode"
        | "kyou_zhidian:choose_effect"
        | "nsxianhai:choose_slot"
        | "olzhuiji:choose_mode"
        | "qingxian:choose_mode"
        | "retianxiang:choose_mode"
        | "reyajiao:diff_mode"
        | "lixia:choose_mode"
        | "rezaiqi:target_mode"
        | "olniepan:choose_skill"
        | "rexingshang:choose_mode"
        | "liangzhu:choose_mode"
        | "renshe:choose_mode"
        | "rexuanfeng:mode" => {
            if let Some(opt) = options.first().copied() {
                answer(vec![opt])
            } else {
                pass_or_first()
            }
        }
        // 其余牌选择（按公开信息最低价值消耗或选定 `min` 张）
        "duwu:discard_cards"
        | "fenxun:discard_card"
        | "gushe:owner_discard_or_draw"
        | "gushe:target_discard_or_draw"
        | "jinglve:choose_card"
        | "jujian:discard_cards"
        | "lianzhu:give_card"
        | "lianzhu:target_discard"
        | "lihun:discard_card"
        | "lihun:return_cards"
        | "mouzhu:target_give"
        | "new_reqingnang:discard_card"
        | "new_yijue:discard_card"
        | "new_yijue:target_show"
        | "nzry_chenglve:discard_cards"
        | "nzry_feijun:discard_card"
        | "nzry_feijun:target_card"
        | "quji:discard_cards"
        | "refenxun2:discard_card"
        | "reguose:choose_card"
        | "rejunxing:discard_cards"
        | "rejunxing:target_discard"
        | "reluanji:choose_cards"
        | "rewurong:owner_show"
        | "rewurong:target_show"
        | "rewurong:gain_card"
        | "rezhijian:choose_card"
        | "kyou_zhidian:choose_card"
        | "dclbjiuxian:choose_card"
        | "olhuoji:choose_card"
        | "reshuangxiong:choose_card"
        | "benyu:discard_cards"
        | "chouce:discard_card"
        | "dcmoukui:target_discard_owner"
        | "gaoyuan:discard_card"
        | "jieyuan:more_card"
        | "jieyuan:less_card"
        | "jsrgshichong:steal_card"
        | "jsrgshichong:give_card"
        | "liyu:gain_card"
        | "olzhuiji:discard_card"
        | "qingxian:discard_equip"
        | "reduodao:discard_card"
        | "repojun:choose_cards"
        | "retianxiang:discard_card"
        | "rexingluan:choose_card"
        | "gnsheque:choose_sha"
        | "mozhi:choose_card"
        | "piaoling:discard_card"
        | "reshuangxiong:pick_card"
        | "chenqing:discard_cards"
        | "lirang:choose_cards"
        | "requanji:choose_card"
        | "rexuanfeng:card"
        | "rexuanfeng:move_card" => {
            if prompt == "gushe:owner_discard_or_draw" || prompt == "gushe:target_discard_or_draw"
            {
                if allow_cancel {
                    return answer(Vec::new());
                }
            }
            let picked = pick_lowest_cards(catalog, input, options, min.max(1));
            if picked.len() >= min && !picked.is_empty() {
                answer(picked)
            } else {
                pass_or_first()
            }
        }
        _ => None,
    }
}
