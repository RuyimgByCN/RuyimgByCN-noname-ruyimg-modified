//! Batch U05-6b (`shahai-hearth` Part 2) 询问的公开信息集 AI 决策策略。
//! 严格仅读取 `PolicyInput` / `PlayerView` 公开视野字段，不窥探他家私密手牌或牌库序。

use super::{extension_card_value, public_attitude};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::engine::GameAction;
use crate::view::PolicyInput;

fn answer(id: u64, selection: Vec<ChoiceOption>) -> Option<GameAction> {
    Some(GameAction::Answer {
        choice_id: id,
        selection,
    })
}

fn best_player(
    input: &PolicyInput,
    options: &[ChoiceOption],
    allow_cancel: bool,
    hostile: bool,
) -> Vec<ChoiceOption> {
    let seat = input.view.seat;
    let picked = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .filter(|&p| {
            let att = public_attitude(&input.view, seat, p);
            if !allow_cancel {
                true
            } else if hostile {
                att < 0
            } else {
                att >= 0
            }
        })
        .max_by_key(|&p| {
            let att = public_attitude(&input.view, seat, p);
            let pv = input.view.players.get(p as usize);
            let hp = pv.map_or(3, |x| x.hp);
            let hc = pv.map_or(0, |x| x.hand_count as i32);
            if hostile {
                (-att * 10) + (6 - hp) + hc
            } else {
                (att * 10) + (4 - hp)
            }
        });
    match picked {
        Some(p) => vec![ChoiceOption::Player(p)],
        None if allow_cancel => Vec::new(),
        None => options.iter().take(1).copied().collect(),
    }
}

fn pick_cards(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
    min: usize,
    highest_first: bool,
) -> Vec<ChoiceOption> {
    let mut cards: Vec<crate::ids::CardId> = options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| {
        let v = extension_card_value(catalog, input, c);
        if highest_first { -v } else { v }
    });
    cards
        .into_iter()
        .take(min.max(1))
        .map(ChoiceOption::Card)
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_hearth_6b_choice_action(
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
    let me = input.view.players.get(seat as usize);
    match prompt {
        // 友方受益目标选择
        "hs_yufa_target"
        | "hs_jiaohui_target"
        | "hs_lingzhou_target"
        | "hs_huanwu_target"
        | "hs_tuteng7_target" => {
            answer(id, best_player(input, options, allow_cancel, false))
        }
        "hs_shengguang_target" => {
            let best_ally = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .filter(|&p| {
                    public_attitude(&input.view, seat, p) >= 0
                        && input
                            .view
                            .players
                            .get(p as usize)
                            .is_some_and(|pv| pv.hp < pv.max_hp as i32)
                })
                .max_by_key(|&p| public_attitude(&input.view, seat, p));
            if let Some(p) = best_ally {
                answer(id, vec![ChoiceOption::Player(p)])
            } else {
                answer(id, best_player(input, options, allow_cancel, true))
            }
        }
        "hs_fushi_target" | "hs_oldfushi_target" => {
            let picked = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .max_by_key(|&p| {
                    let att = public_attitude(&input.view, seat, p);
                    let pv = input.view.players.get(p as usize);
                    let wounds = pv.map_or(0, |x| x.max_hp as i32 - x.hp);
                    if wounds >= 2 && att >= 0 {
                        att * 10 + wounds
                    } else if wounds == 1 && att < 0 {
                        -att * 8
                    } else {
                        -5
                    }
                });
            match picked {
                Some(p) if !allow_cancel || public_attitude(&input.view, seat, p) != 0 => {
                    answer(id, vec![ChoiceOption::Player(p)])
                }
                _ => answer(id, Vec::new()),
            }
        }
        "hs_enze_target" | "hs_oldenze_target" => {
            let my_h = me.map_or(0, |m| m.hand_count as i32);
            let picked = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .max_by_key(|&p| {
                    let att = public_attitude(&input.view, seat, p);
                    let th = input
                        .view
                        .players
                        .get(p as usize)
                        .map_or(my_h, |x| x.hand_count as i32);
                    let delta = (my_h - th).clamp(-3, 3);
                    att * delta
                });
            match picked {
                Some(p) => answer(id, vec![ChoiceOption::Player(p)]),
                None if allow_cancel => answer(id, Vec::new()),
                None => answer(id, options.iter().take(1).copied().collect()),
            }
        }
        "hs_yulu_targets" => {
            let mut chosen: Vec<ChoiceOption> = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .filter(|&p| public_attitude(&input.view, seat, p) >= 0)
                .take(max.max(1))
                .map(ChoiceOption::Player)
                .collect();
            if chosen.is_empty() && !allow_cancel {
                chosen = options.iter().take(min.max(1)).copied().collect();
            }
            answer(id, chosen)
        }
        // 敌方负收益目标选择
        "hs_fbeifa_target"
        | "hs_oldfbeifa_target"
        | "hs_duzhang_target"
        | "hs_xmojian_target"
        | "hs_shijie_target"
        | "hs_xinci_target"
        | "hs_mdzhoufu_target"
        | "hs_jingxiang_target" => {
            answer(id, best_player(input, options, allow_cancel, true))
        }
        "hs_dunji_targets" => {
            let mut enemies: Vec<ChoiceOption> = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) if public_attitude(&input.view, seat, *p) < 0 => {
                        Some(ChoiceOption::Player(*p))
                    }
                    _ => None,
                })
                .take(max.max(1))
                .collect();
            if enemies.is_empty() && !allow_cancel {
                enemies = options.iter().take(min.max(1)).copied().collect();
            }
            answer(id, enemies)
        }
        // Tag 选择
        "hs_xieneng_pick" => answer(id, options.iter().take(1).copied().collect()),
        "hs_jiaohui_choice" | "hs_lingzhou_choice" => {
            let want_rec = me.is_some_and(|m| m.hp <= 2)
                && options.contains(&ChoiceOption::Tag(1));
            let opt = if want_rec {
                ChoiceOption::Tag(1)
            } else {
                ChoiceOption::Tag(0)
            };
            if options.contains(&opt) {
                answer(id, vec![opt])
            } else {
                answer(id, options.iter().take(1).copied().collect())
            }
        }
        "hs_tuteng_pick" => {
            let wounded = me.is_some_and(|m| m.hp < m.max_hp as i32);
            let pick = if wounded && options.contains(&ChoiceOption::Tag(1)) {
                ChoiceOption::Tag(1)
            } else {
                options.first().copied().unwrap_or(ChoiceOption::Tag(1))
            };
            answer(id, vec![pick])
        }
        "hs_tuteng_replace" => answer(id, options.iter().take(1).copied().collect()),
        // 发现 / 夺牌优先高价值
        "hs_huanjue_pick" | "hs_malymowang_discover" | "hs_jingxiang_card" => {
            answer(id, pick_cards(catalog, input, options, min.max(1), true))
        }
        // 弃牌 / 代价优先低价值
        "hs_bingyan_card"
        | "hs_yulu_discard"
        | "hs_biri_discard"
        | "hs_guozai2_discard"
        | "hs_fengxing_discard"
        | "hs_fengxian_discard"
        | "hs_zhanhou_card"
        | "hs_shengguang_card"
        | "hs_xinci_card"
        | "hs_anying_card"
        | "hs_bingjia_card"
        | "hs_mdzhoufu_card"
        | "hs_mengun_discard"
        | "hs_jihuo_discard"
        | "hs_shixu_discard"
        | "hs_enze_discard" => answer(id, pick_cards(catalog, input, options, min.max(1), false)),
        // `yuanzheng` 他家 `"hej"` 选牌弃置：严格只读 `PlayerView` 公开装备区/判定区与态度，不窥探他家私密手牌
        "hs_yuanzheng_card" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter_map(|c| {
                    let score = if let Some(pv) =
                        input.view.players.iter().find(|pv| pv.equips.contains(&c))
                    {
                        if public_attitude(&input.view, seat, pv.seat) >= 0 {
                            -100
                        } else {
                            100
                        }
                    } else if let Some(pv) =
                        input.view.players.iter().find(|pv| pv.judge.contains(&c))
                    {
                        if public_attitude(&input.view, seat, pv.seat) >= 0 {
                            90
                        } else {
                            -90
                        }
                    } else {
                        10
                    };
                    (score > 0).then_some((score, c))
                })
                .max_by_key(|&(s, _)| s)
                .map(|(_, c)| vec![ChoiceOption::Card(c)])
                .unwrap_or_else(|| {
                    if allow_cancel {
                        Vec::new()
                    } else {
                        options.iter().take(1).copied().collect()
                    }
                });
            answer(id, best)
        }
        // 确认弹窗
        "hs_shifa_draw_confirm"
        | "hs_oldshifa_draw_confirm"
        | "hs_yuanzheng_confirm"
        | "hs_qianghua_confirm"
        | "hs_bimeng_confirm"
        | "hs_jingmeng_confirm"
        | "hs_hswuji_confirm"
        | "hs_bingshuang_confirm"
        | "hs_yanshu_confirm"
        | "hs_oldyanshu_confirm"
        | "hs_shengyan_confirm"
        | "hs_wlianji_confirm"
        | "hs_xinliegong_confirm" => answer(id, vec![ChoiceOption::Confirm]),
        "hs_rejieming_target" => {
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => {
                        let p = *p;
                        let att = public_attitude(&input.view, seat, p);
                        if att <= 0 {
                            return None;
                        }
                        let pl = &input.view.players[p as usize];
                        let need = (pl.max_hp as i32) - (pl.hand_count as i32);
                        let score = if need > 2 { att * 20 } else { att * 10 } + need;
                        Some((p, score))
                    }
                    _ => None,
                })
                .max_by_key(|&(_, s)| s)
                .map(|(p, _)| ChoiceOption::Player(p));
            best.map(|o| answer(id, vec![o]))
                .unwrap_or_else(|| answer(id, Vec::new()))
        }
        "hs_xianzhi_choice" => {
            let _ = kind;
            answer(id, vec![ChoiceOption::Pass])
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choice::{ChoiceKind, ChoiceSpec};
    use crate::state::Role;
    use crate::view::{HandView, PlayerCellView, PlayerView};

    #[test]
    fn rejieming_decline_uses_empty_cancel_matching_select_players_accepts() {
        let spec = ChoiceSpec {
            kind: ChoiceKind::SelectPlayers,
            prompt: "hs_rejieming_target",
            options: vec![ChoiceOption::Player(1)],
            min: 1,
            max: 1,
            allow_cancel: true,
        };
        let cell = |seat: u8, role: Option<Role>, hand_count: usize| PlayerCellView {
            seat,
            alive: true,
            hp: 4,
            max_hp: 4,
            hand: HandView::Hidden,
            hand_count,
            equips: vec![],
            judge: vec![],
            judge_kinds: vec![],
            generals: [None, None],
            revealed_generals: [false, false],
            turned_over: false,
            linked: false,
            skills: vec![],
            r05_effects: vec![],
            role,
            team: None,
        };
        let input = PolicyInput {
            view: PlayerView {
                seat: 0,
                players: vec![cell(0, Some(Role::Lord), 0), cell(1, Some(Role::Rebel), 2)],
                cards: vec![],
                deck_count: 10,
                discard: vec![],
                table: vec![],
                turn: 1,
                phase: crate::event::Phase::Play,
                round: 1,
                outcome: None,
                ruleset_version: 333,
            },
            choice: None,
            revision: 0,
            quota_remaining: None,
            own_hand_kinds: vec![],
        };
        let e = crate::Engine::new(1, 2);
        let action = plan_hearth_6b_choice_action(
            e.catalog(),
            &input,
            77,
            ChoiceKind::SelectPlayers,
            "hs_rejieming_target",
            &spec.options,
            1,
            1,
            true,
        )
        .expect("rejieming plan action");
        match action {
            GameAction::Answer {
                choice_id: 77,
                selection,
            } => {
                assert!(selection.is_empty(), "must cancel with vec![], got {selection:?}");
                assert!(spec.accepts(&selection), "ChoiceSpec must accept cancel selection");
            }
            other => panic!("unexpected action: {other:?}"),
        }
    }
}
