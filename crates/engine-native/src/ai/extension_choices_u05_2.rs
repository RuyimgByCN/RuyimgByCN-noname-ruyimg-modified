use super::*;
use crate::choice::ChoiceOption;

pub(super) fn plan_u05_2_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    let seat = input.view.seat;
    let pick_lowest_cards = |count: usize| -> Vec<ChoiceOption> {
        let mut cards: Vec<(i64, ChoiceOption)> = options
            .iter()
            .filter_map(|o| match o {
                ChoiceOption::Card(c) => {
                    Some((extension_card_value(catalog, input, *c), *o))
                }
                _ => None,
            })
            .collect();
        cards.sort_by_key(|(v, _)| *v);
        cards.into_iter().take(count).map(|(_, o)| o).collect()
    };
    let pick_enemies = |count: usize| -> Vec<ChoiceOption> {
        let mut enemies: Vec<(i64, ChoiceOption)> = options
            .iter()
            .filter_map(|o| match o {
                ChoiceOption::Player(p) if public_attitude(&input.view, seat, *p) < 0 => {
                    let hp = input
                        .view
                        .players
                        .get(*p as usize)
                        .map(|pv| pv.hp as i64)
                        .unwrap_or(10);
                    Some((hp, *o))
                }
                _ => None,
            })
            .collect();
        enemies.sort_by_key(|(hp, _)| *hp);
        enemies.into_iter().take(count).map(|(_, o)| o).collect()
    };
    let pick_allies = |count: usize| -> Vec<ChoiceOption> {
        let mut allies: Vec<(i64, ChoiceOption)> = options
            .iter()
            .filter_map(|o| match o {
                ChoiceOption::Player(p) if public_attitude(&input.view, seat, *p) >= 0 => {
                    let hp = input
                        .view
                        .players
                        .get(*p as usize)
                        .map(|pv| pv.hp as i64)
                        .unwrap_or(10);
                    Some((hp, *o))
                }
                _ => None,
            })
            .collect();
        allies.sort_by_key(|(hp, _)| *hp);
        allies.into_iter().take(count).map(|(_, o)| o).collect()
    };

    let sel: Vec<ChoiceOption> = match prompt {
        "gj_luyugeng_discover" => {
            let mut best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some((extension_card_value(catalog, input, *c), *o)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            best.sort_by(|a, b| b.0.cmp(&a.0));
            best.into_iter().take(min.max(1)).map(|(_, o)| o).collect()
        }
        "gj_yuheng_cost" | "gj_zuiji_card" | "gj_meihu_give" | "gj_xidie_discard"
        | "gj_fumo_card1" | "gj_fumo_card2" | "gj_chizhen_discard" | "gj_dangping_card"
        | "gj_xuanning1_card" | "gj_yangming_card" | "gj_yunyin_card" | "gj_yuling3_discard"
        | "gj_lingyan_card" | "gj_zhenying_discard" | "gj_yanjia_card1" | "gj_yanjia_card2" => {
            pick_lowest_cards(min.max(1))
        }
        "gj_liuguang_respond" => {
            let hp = input
                .view
                .players
                .get(seat as usize)
                .map(|p| p.hp)
                .unwrap_or(2);
            let picked = pick_lowest_cards(1);
            let low_val = picked
                .first()
                .and_then(|o| match o {
                    ChoiceOption::Card(c) => Some(extension_card_value(catalog, input, *c)),
                    _ => None,
                })
                .unwrap_or(100);
            if hp <= 1 || low_val <= 60 {
                picked
            } else if allow_cancel {
                Vec::new()
            } else {
                picked
            }
        }
        "gj_shihuifen_use" => {
            let cur = input.view.turn;
            if public_attitude(&input.view, seat, cur) < 0 {
                if let Some(card_opt) = options.iter().find(|o| matches!(o, ChoiceOption::Card(_)))
                {
                    vec![*card_opt]
                } else {
                    vec![ChoiceOption::Pass]
                }
            } else {
                vec![ChoiceOption::Pass]
            }
        }
        "gj_xuelu_card" | "gj_xuanci_card" | "gj_qinglan_card" | "gj_meiying_discard"
        | "gj_cihong_red" => {
            let enemies_exist = !pick_enemies(1).is_empty()
                || input
                    .view
                    .players
                    .iter()
                    .enumerate()
                    .any(|(i, p)| p.alive && i as u8 != seat && public_attitude(&input.view, seat, i as u8) < 0);
            if enemies_exist || !allow_cancel {
                pick_lowest_cards(1)
            } else {
                Vec::new()
            }
        }
        "gj_xuelu_target" | "gj_xuanci_target" | "gj_chizhen_duel_target"
        | "gj_dangping_target"
        | "gj_liuying_target" | "gj_duhun_target" | "gj_cihong_target" | "gj_yunyin_target" => {
            let e = pick_enemies(1);
            if e.is_empty() && !allow_cancel {
                options.iter().take(min).copied().collect()
            } else {
                e
            }
        }
        "gj_jiehuo_target" => {
            let my_hp = input
                .view
                .players
                .get(seat as usize)
                .map(|p| p.hp)
                .unwrap_or(3);
            if my_hp <= 1 {
                pick_enemies(1)
            } else if allow_cancel {
                Vec::new()
            } else {
                pick_enemies(1)
            }
        }
        "gj_liuguang_targets" => {
            let e = pick_enemies(max);
            if e.is_empty() && !allow_cancel {
                options.iter().take(min).copied().collect()
            } else {
                e
            }
        }
        "gj_fanyin_target" | "gj_yangming2_targets" | "gj_qingshu_target"
        | "gj_lianjing_targets" => {
            let a = pick_allies(max);
            if a.is_empty() && !allow_cancel {
                options.iter().take(min).copied().collect()
            } else {
                a
            }
        }
        "gj_yange_target" => {
            let my_hand = input
                .view
                .players
                .get(seat as usize)
                .map(|p| p.hand_count)
                .unwrap_or(0);
            let best = options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => {
                        let hc = input
                            .view
                            .players
                            .get(*p as usize)
                            .map(|pv| pv.hand_count)
                            .unwrap_or(0);
                        if hc >= my_hand + 2 {
                            Some((hc, *o))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .max_by_key(|(hc, _)| *hc);
            match best {
                Some((_, o)) => vec![o],
                None if allow_cancel => Vec::new(),
                None => options.iter().take(min).copied().collect(),
            }
        }
        "gj_yanjiadan_target" => {
            let e = pick_enemies(max);
            if !e.is_empty() {
                e
            } else {
                let a = pick_allies(max);
                if a.is_empty() && !allow_cancel {
                    options.iter().take(min).copied().collect()
                } else {
                    a
                }
            }
        }
        "gj_yuehua_confirm"
        | "gj_meihu_confirm"
        | "gj_xiuhua_gain"
        | "gj_zuizhan_confirm"
        | "gj_xfanghua_confirm" => {
            if options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else {
                options.iter().take(min).copied().collect()
            }
        }
        "gj_xlqianhuan_confirm" => {
            let pv = input.view.players.get(seat as usize);
            let want = pv.is_some_and(|p| p.hp <= 1 || p.turned_over);
            if want && options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else if options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                options.iter().take(min).copied().collect()
            }
        }
        "gj_manwu_confirm" | "gj_woxue_confirm" | "gj_xuanzhen_confirm" => {
            let cur = input.view.turn;
            let att = public_attitude(&input.view, seat, cur);
            let want = if prompt == "gj_manwu_confirm" {
                att >= 0
            } else {
                att < 0
            };
            if want && options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else if options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                options.iter().take(min).copied().collect()
            }
        }
        "gj_cihong_losehp" => {
            let my_hp = input
                .view
                .players
                .get(seat as usize)
                .map(|p| p.hp)
                .unwrap_or(1);
            if my_hp > 2 && options.contains(&ChoiceOption::Confirm) {
                vec![ChoiceOption::Confirm]
            } else if options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                options.iter().take(min).copied().collect()
            }
        }
        "gj_cihong_turnover" => {
            if options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                options.iter().take(min).copied().collect()
            }
        }
        "gj_yanjiadan_vcard"
        | "gj_yunyin_vcard"
        | "gj_qingshu_vcard"
        | "gj_yunyou_discover"
        | "gj_xuanzhen_vcard" => options.iter().take(min.max(1)).copied().collect(),
        _ => return None,
    };
    Some(GameAction::Answer {
        choice_id: id,
        selection: sel,
    })
}
