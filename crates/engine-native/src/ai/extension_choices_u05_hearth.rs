//! Batch U05-6a (`shahai-hearth` Part 1) 询问的公开信息集 AI 决策策略。

use super::{extension_card_value, public_attitude};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::engine::GameAction;
use crate::view::PolicyInput;

pub(super) fn plan_wzhanyi_trigger_confirm(input: &PolicyInput, id: u64) -> Option<GameAction> {
    let me = input.view.players.get(input.view.seat as usize);
    let confirm = me.is_some_and(|m| m.hand_count + 2 <= m.hp.max(0) as usize);
    let selection = if confirm {
        vec![ChoiceOption::Confirm]
    } else {
        vec![ChoiceOption::Pass]
    };
    Some(GameAction::Answer {
        choice_id: id,
        selection,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_hearth_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    kind: ChoiceKind,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    match prompt {
        "bolan_g:choose_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(t) => Some(*t),
                    _ => None,
                })
                .filter(|&t| !allow_cancel || public_attitude(&input.view, seat, t) >= 0)
                .max_by_key(|&t| public_attitude(&input.view, seat, t));
            let selection = match best {
                Some(t) => vec![ChoiceOption::Player(t)],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(1).copied().collect(),
            };
            return Some(GameAction::Answer {
                choice_id: id,
                selection,
            });
        }
        "sbqingzheng:choose_suits" => {
            let need = min.max(1);
            let selection = if options.len() < need && allow_cancel {
                Vec::new()
            } else {
                options.iter().take(need).copied().collect()
            };
            return Some(GameAction::Answer {
                choice_id: id,
                selection,
            });
        }
        "bolan:choose_skill" => {
            let selection = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(_)))
                .copied()
                .into_iter()
                .collect();
            return Some(GameAction::Answer {
                choice_id: id,
                selection,
            });
        }
        _ => {}
    }
    if !prompt.starts_with("hs_") {
        return None;
    }
    let pick_low_cards = |n: usize| -> Vec<ChoiceOption> {
        let mut cards: Vec<_> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            })
            .collect();
        cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
        cards.into_iter().take(n).map(ChoiceOption::Card).collect()
    };
    let pick_high_cards = |n: usize| -> Vec<ChoiceOption> {
        let mut cards: Vec<_> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            })
            .collect();
        cards.sort_by_key(|&c| std::cmp::Reverse(extension_card_value(catalog, input, c)));
        cards.into_iter().take(n).map(ChoiceOption::Card).collect()
    };
    let pick_best_player = |want_ally: bool| -> Option<u8> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(p) => Some(p),
                _ => None,
            })
            .max_by_key(|&p| {
                let att = public_attitude(&input.view, seat, p);
                if want_ally {
                    att
                } else {
                    -att
                }
            })
    };
    let answer = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };
    match prompt {
        "hs_zhaomingdan_discard" => {
            // 优先选任意可弃牌；若全是己方且可取消则跳过
            return answer(pick_high_cards(1));
        }
        "hs_shandianjian_discard" => {
            let best = pick_low_cards(1);
            if best.is_empty() && allow_cancel {
                return answer(Vec::new());
            }
            return answer(best);
        }
        "hs_shandianjian_show"
        | "hs_dunpaigedang_discard"
        | "hs_shijieshu_discard"
        | "hs_buwendingyibian_recast"
        | "hs_nvwudeganguo_discard"
        | "hs_guhuo_give"
        | "hs_hshuanling_discard"
        | "hs_hspuzhao_card"
        | "hs_hstianqi_card"
        | "hs_yindan_card"
        | "hs_hllingxi_discard"
        | "hs_tansuo_card"
        | "hs_zhuilie_discard"
        | "hs_fuhua_card"
        | "hs_fuhua2_give"
        | "hs_qianhou_discard"
        | "hs_zhongji_discard"
        | "hs_midian_card"
        | "hs_xingluo_discard"
        | "hs_yuelu_card"
        | "hs_yushou_card"
        | "hs_aoshu_card"
        | "hs_lianjin_card"
        | "hs_yingxi_card"
        | "hs_nuyan_card"
        | "hs_xunbao2_discard" => {
            let n = min.max(1);
            return answer(pick_low_cards(n));
        }
        "hs_chuansongmen_use"
        | "hs_asyouzhang_use"
        | "hs_xiyong_use" => {
            if kind == ChoiceKind::SelectPlayers {
                if let Some(t) = pick_best_player(false) {
                    return answer(vec![ChoiceOption::Player(t)]);
                }
                return answer(Vec::new());
            }
            let n = min.max(1);
            return answer(pick_high_cards(n));
        }
        "hs_morijingxiang_gain"
        | "hs_hshuanyu_discover"
        | "hs_tansuo_discover"
        | "hs_oldhuanjue_pick"
        | "hs_lieqi_guess"
        | "hs_xingluo_pick"
        | "hs_zhuizong_pick"
        | "hs_nsbeiming_order"
        | "hs_tanmi_use"
        | "hs_wzhanyi_use" => {
            let n = min.max(1);
            return answer(pick_high_cards(n));
        }
        "hs_mobao_cards" | "hs_qianghuax_cards" | "hs_zhuizong_cards" => {
            let sel = pick_low_cards(max.min(2).max(min.max(1)));
            return answer(sel);
        }
        "hs_xianji2_discard" => {
            // 有献祭友方时弃 2 张低价值牌，否则取消
            if allow_cancel {
                let any_ally = input.view.players.iter().any(|p| {
                    p.seat != seat && p.alive && public_attitude(&input.view, seat, p.seat) > 0
                });
                if !any_ally {
                    return answer(Vec::new());
                }
            }
            return answer(pick_low_cards(1));
        }
        "hs_hualing_target"
        | "hs_mengye_target"
        | "hs_kuixin_target"
        | "hs_fuhua_target"
        | "hs_jinzhou_target"
        | "hs_yingxi_target"
        | "hs_wzhanyi_target"
        | "hs_nuyan_target"
        | "hs_nuyan2_target"
        | "hs_azaowu_target"
        | "hs_tanmi_target" => {
            if let Some(t) = pick_best_player(false) {
                if allow_cancel && public_attitude(&input.view, seat, t) > 0 {
                    return answer(Vec::new());
                }
                return answer(vec![ChoiceOption::Player(t)]);
            }
            return answer(Vec::new());
        }
        "hs_lieqi_target" => {
            if let Some(t) = pick_best_player(false) {
                return answer(vec![ChoiceOption::Player(t)]);
            }
            return answer(Vec::new());
        }
        "hs_xingluo_targets" => {
            let enemies: Vec<ChoiceOption> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) if public_attitude(&input.view, seat, p) <= 0 => Some(*o),
                    _ => None,
                })
                .take(max.max(1))
                .collect();
            if !enemies.is_empty() {
                return answer(enemies);
            }
            if allow_cancel {
                return answer(Vec::new());
            }
        }
        "hs_hllingxi_target" | "hs_peiyu_target" | "hs_zengli_target" => {
            let want_ally = prompt != "hs_zengli_target";
            if let Some(t) = pick_best_player(want_ally) {
                return answer(vec![ChoiceOption::Player(t)]);
            }
            return answer(Vec::new());
        }
        "hs_szbianshen_confirm"
        | "hs_bzhuiji_confirm"
        | "hs_wzhanyi_confirm"
        | "hs_yuansu_confirm"
        | "hs_xueren_confirm" => {
            let me = input.view.players.get(seat as usize);
            let should_confirm = match prompt {
                "hs_xueren_confirm" => me.is_some_and(|m| m.hp > 1),
                "hs_wzhanyi_confirm" => me.is_some_and(|m| m.hand_count + 2 <= m.hp.max(0) as usize),
                _ => true,
            };
            if should_confirm && options.contains(&ChoiceOption::Confirm) {
                return answer(vec![ChoiceOption::Confirm]);
            }
            return answer(Vec::new());
        }
        "hs_fuhua_choice" => {
            // Tag 1 = Duel if healthy, Tag 0 if low HP
            let hp = input.view.players.get(seat as usize).map_or(2, |m| m.hp);
            let want = if hp <= 1 { 0 } else { 1 };
            let chosen = options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(t) if *t == want))
                .or_else(|| options.first())
                .copied();
            return answer(chosen.into_iter().collect());
        }
        "hs_hualing_skill"
        | "hs_hsnitai_skill"
        | "hs_szbianshen_general"
        | "hs_kekao_pick"
        | "hs_jinhua_skill"
        | "hs_kqizhou_pick"
        | "hs_azaowu_kind"
        | "hs_nuyan_kind"
        | "hs_nuyan2_kind"
        | "hs_maoxian_skill" => {
            let first = options.iter().find(|o| matches!(o, ChoiceOption::Tag(_))).copied();
            return answer(first.into_iter().collect());
        }
        _ => {}
    }
    None
}
