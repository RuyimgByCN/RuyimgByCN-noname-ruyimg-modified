//! U05 (`swd` 武将技前半) 扩展询问的 AI 公开信息决策策略。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_swd_choice_action(
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
    let is_public_equip = |c: crate::ids::CardId| -> bool {
        input
            .view
            .players
            .iter()
            .any(|p| p.equips.iter().any(|&ec| ec == c))
    };
    let own_or_equip_value = |c: crate::ids::CardId| -> Option<i64> {
        if input.own_hand_kinds.iter().any(|(hc, _)| *hc == c) || is_public_equip(c) {
            Some(extension_card_value(catalog, input, c))
        } else {
            None
        }
    };
    let pick_cards_low_own = |count: usize| -> Vec<ChoiceOption> {
        let mut cards: Vec<crate::ids::CardId> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            })
            .collect();
        cards.sort_by_key(|&c| own_or_equip_value(c).unwrap_or(30));
        cards
            .into_iter()
            .take(count)
            .map(ChoiceOption::Card)
            .collect()
    };
    let pick_player = |want_ally: bool| -> Option<ChoiceOption> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(t) => Some(t),
                _ => None,
            })
            .max_by_key(|&t| {
                let att = public_attitude(&input.view, seat, t);
                let hp = input.view.players.get(t as usize).map(|p| p.hp).unwrap_or(0);
                if want_ally {
                    (att, -hp)
                } else {
                    (-att, -hp)
                }
            })
            .map(ChoiceOption::Player)
    };
    let answer = |selection: Vec<ChoiceOption>| Some(GameAction::Answer { choice_id: id, selection });
    match prompt {
        "kbolan2_top" | "senluo_discard" | "guaili2_discard" | "lingyue_discard"
        | "swd_jiefen_give" | "swd_jiefen_give_card" => {
            answer(pick_cards_low_own(min.max(1)))
        }
        "touxi_discard" | "duishi_gain" | "yuchen_discard" => {
            // 仅看公开装备区，严禁读取对手暗手牌目录牌种。
            let mut public_equips: Vec<(i64, crate::ids::CardId)> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) if is_public_equip(c) => {
                        Some((extension_card_value(catalog, input, c), c))
                    }
                    _ => None,
                })
                .collect();
            public_equips.sort_by_key(|(v, _)| -*v);
            if let Some((_, c)) = public_equips.first() {
                answer(vec![ChoiceOption::Card(*c)])
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "tiebi_discard" => {
            let best = pick_cards_low_own(1);
            let low_enough = best
                .first()
                .and_then(|o| match o {
                    ChoiceOption::Card(c) => own_or_equip_value(*c),
                    _ => None,
                })
                .unwrap_or(50)
                <= 70;
            if allow_cancel && !low_enough {
                answer(Vec::new())
            } else {
                answer(best)
            }
        }
        "lingxian_target" => {
            let ally = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .filter(|&t| public_attitude(&input.view, seat, t) > 0)
                .max_by_key(|&t| {
                    let h = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hand_count as i32)
                        .unwrap_or(0);
                    (public_attitude(&input.view, seat, t), -h)
                });
            if let Some(t) = ally {
                answer(vec![ChoiceOption::Player(t)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "lingyue_confirm" => {
            let any_enemy = input
                .view
                .players
                .iter()
                .enumerate()
                .any(|(i, p)| p.alive && i as u8 != seat && public_attitude(&input.view, seat, i as u8) <= 0);
            if any_enemy {
                answer(vec![ChoiceOption::Confirm])
            } else {
                answer(Vec::new())
            }
        }
        "kbolan_confirm" => answer(vec![ChoiceOption::Confirm]),
        "linyun2_shan" => {
            let shan_count = input
                .own_hand_kinds
                .iter()
                .filter(|(_, k)| *k == crate::catalog::CardKind::Shan)
                .count();
            if shan_count > 1 || !allow_cancel {
                answer(pick_cards_low_own(1))
            } else {
                answer(Vec::new())
            }
        }
        "swd_jiefen_give_target" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) => Some(t),
                    _ => None,
                })
                .max_by_key(|&t| {
                    let h = input
                        .view
                        .players
                        .get(t as usize)
                        .map(|p| p.hand_count as i32)
                        .unwrap_or(0);
                    (public_attitude(&input.view, seat, t), -h)
                });
            answer(best.into_iter().map(ChoiceOption::Player).collect())
        }
        "nlianji_t1_pick" => {
            // t1 自选己方最高点数手牌以争拼点胜出（只读自己已知手牌）
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c)
                        if input.own_hand_kinds.iter().any(|(hc, _)| *hc == c) =>
                    {
                        let r = catalog.cards.get(c as usize).map(|d| d.rank).unwrap_or(0);
                        Some((r, c))
                    }
                    _ => None,
                })
                .max_by_key(|(r, _)| *r)
                .map(|(_, c)| ChoiceOption::Card(c));
            answer(best.into_iter().collect())
        }
        "pozhou_target" => {
            let mut enemies: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) if public_attitude(&input.view, seat, t) <= 0 => Some(t),
                    _ => None,
                })
                .collect();
            enemies.sort_by_key(|&t| public_attitude(&input.view, seat, t));
            let sel: Vec<ChoiceOption> = enemies
                .into_iter()
                .take(max.max(1))
                .map(ChoiceOption::Player)
                .collect();
            if sel.len() < min && allow_cancel {
                answer(Vec::new())
            } else {
                answer(sel)
            }
        }
        "aojian_targets" | "ningxian_targets" | "huodan_target" => {
            let mut enemies: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) if public_attitude(&input.view, seat, t) <= 0 => Some(t),
                    _ => None,
                })
                .collect();
            if enemies.len() < min && allow_cancel {
                return answer(Vec::new());
            }
            if enemies.len() < min {
                enemies = options
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Player(t) => Some(t),
                        _ => None,
                    })
                    .take(min)
                    .collect();
            }
            answer(
                enemies
                    .into_iter()
                    .take(max.max(min))
                    .map(ChoiceOption::Player)
                    .collect(),
            )
        }
        "swd_jqimou_use" => {
            use crate::catalog::CardKind;
            let injured = input
                .view
                .players
                .get(seat as usize)
                .is_some_and(|p| p.hp < p.max_hp as i32);
            let any_enemy = input.view.players.iter().enumerate().any(|(i, p)| {
                p.alive && i as u8 != seat && public_attitude(&input.view, seat, i as u8) <= 0
            });
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => {
                        let k = input
                            .own_hand_kinds
                            .iter()
                            .find(|(hc, _)| *hc == c)
                            .map(|(_, k)| *k)
                            .unwrap_or_else(|| catalog.card_kind(c));
                        let score = match k {
                            CardKind::Tao if injured => 100,
                            CardKind::Wuzhong => 90,
                            _ if k.is_equip() => 80,
                            CardKind::Dismantle
                            | CardKind::Snatch
                            | CardKind::Duel
                            | CardKind::Le
                            | CardKind::Bing
                            | CardKind::SavageAttack
                            | CardKind::ArcheryAttack
                            | CardKind::Sha
                            | CardKind::FireSha
                            | CardKind::LeiSha
                                if any_enemy =>
                            {
                                60
                            }
                            _ if any_enemy && !k.is_basic_kind() => 40,
                            _ => 0,
                        };
                        (score > 0).then_some((score, c))
                    }
                    _ => None,
                })
                .max_by_key(|(s, _)| *s)
                .map(|(_, c)| vec![ChoiceOption::Card(c)]);
            answer(best.unwrap_or_default())
        }
        "zhanxing_target"
        | "huanxing_target"
        | "mingfu_target"
        | "bingjian_target"
        | "shangxi_target"
        | "lmazui_target"
        | "jianji_target"
        | "linyun_sha_target"
        | "duishi_target"
        | "yuchen_target"
        | "swd_jiefen_target"
        | "swd_jqimou_target"
        | "nlianji_target0"
        | "nlianji_target1"
        | "xuanying_use_target"
        | "yeying_target"
        | "guiying_target"
        | "huanxia_target"
        | "yaotong_target"
        | "jilve_jiedao_victim" => {
            if let Some(t) = pick_player(false) {
                answer(vec![t])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "jilve_targets" => {
            if let Some(t) = pick_player(false) {
                answer(vec![t])
            } else {
                answer(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "jilve_card" => answer(options.iter().take(1).copied().collect()),
        "aojian_discard"
        | "ningxian_discard"
        | "huodan_pick"
        | "zhanxing_discard"
        | "huanxing_discard"
        | "mingfu_pick"
        | "bingjian_discard"
        | "shangxi_discard"
        | "lmazui_pick"
        | "jianji_pick"
        | "linyun_pick_two"
        | "duishi_discard"
        | "nlianji_throw"
        | "yeying_pick"
        | "guiying_pick"
        | "fengze_pick"
        | "huanxia_pick"
        | "yaotong_pick" => answer(pick_cards_low_own(min.max(1))),
        "yijue_target" => {
            let best_enemy = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) if public_attitude(&input.view, seat, t) < 0 => Some(t),
                    _ => None,
                })
                .max_by_key(|&t| {
                    let att = public_attitude(&input.view, seat, t);
                    let hp = input.view.players.get(t as usize).map(|p| p.hp).unwrap_or(0);
                    (-att, -hp)
                });
            let best_heal_ally = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(t) if public_attitude(&input.view, seat, t) > 0 => {
                        let p = input.view.players.get(t as usize)?;
                        (p.alive
                            && p.hp < p.max_hp as i32
                            && p.hand_count > p.hp.max(0) as usize + 1)
                            .then_some((p.max_hp as i32 - p.hp, t))
                    }
                    _ => None,
                })
                .max_by_key(|&(missing, _)| missing)
                .map(|(_, t)| t);
            if let Some(t) = best_enemy.or(best_heal_ally) {
                answer(vec![ChoiceOption::Player(t)])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "swd_yueren_confirm" => answer(vec![ChoiceOption::Confirm]),
        "swd_yijue_recover_confirm" => {
            let heal_target = options.iter().find_map(|o| match *o {
                ChoiceOption::Player(t)
                    if input.view.players.get(t as usize).is_some_and(|p| {
                        p.alive
                            && p.hp < p.max_hp as i32
                            && public_attitude(&input.view, seat, t) > 0
                    }) =>
                {
                    Some(ChoiceOption::Player(t))
                }
                _ => None,
            });
            if let Some(opt) = heal_target {
                answer(vec![opt])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        _ => None,
    }
}
