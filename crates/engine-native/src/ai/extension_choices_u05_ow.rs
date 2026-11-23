//! U05-3 (`shaai-ow`) 守望先锋挂起询问的公开信息 AI 策略。

use super::{extension_card_value, public_attitude};
use crate::catalog::Suit;
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::ids::{CardId, PlayerId};
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_u05_ow_choice_action(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    id: u64,
    prompt: &str,
    options: &[ChoiceOption],
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<GameAction> {
    if !prompt.starts_with("ow_") {
        return None;
    }
    let seat = input.view.seat;
    let me = input.view.players.get(seat as usize);
    let my_hp = me.map(|p| p.hp).unwrap_or(0);
    let my_max_hp = me.map(|p| p.max_hp as i32).unwrap_or(0);

    let mut cards: Vec<CardId> = options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| extension_card_value(catalog, input, c));

    let players: Vec<PlayerId> = options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .collect();

    let pick_cards = |n: usize| -> Vec<ChoiceOption> {
        if cards.len() < n && allow_cancel {
            Vec::new()
        } else {
            cards
                .iter()
                .take(n.min(max))
                .copied()
                .map(ChoiceOption::Card)
                .collect()
        }
    };

    let pick_enemy = |want_hand_nonempty: bool| -> Vec<ChoiceOption> {
        let best = players
            .iter()
            .copied()
            .filter(|&t| public_attitude(&input.view, seat, t) < 0)
            .filter(|&t| {
                !want_hand_nonempty
                    || input
                        .view
                        .players
                        .get(t as usize)
                        .is_some_and(|p| p.hand_count > 0)
            })
            .max_by_key(|&t| {
                let hp = input.view.players.get(t as usize).map(|p| p.hp).unwrap_or(4);
                (-public_attitude(&input.view, seat, t), -hp)
            });
        match best {
            Some(t) => vec![ChoiceOption::Player(t)],
            None if allow_cancel => Vec::new(),
            None => players
                .first()
                .copied()
                .map(ChoiceOption::Player)
                .into_iter()
                .collect(),
        }
    };

    let pick_enemies = |count_cap: usize| -> Vec<ChoiceOption> {
        let mut enemies: Vec<PlayerId> = players
            .iter()
            .copied()
            .filter(|&t| public_attitude(&input.view, seat, t) < 0)
            .collect();
        enemies.sort_by_key(|&t| {
            let hp = input.view.players.get(t as usize).map(|p| p.hp).unwrap_or(4);
            (public_attitude(&input.view, seat, t), hp)
        });
        let take_n = enemies.len().min(count_cap).min(max);
        if take_n >= min && take_n > 0 {
            enemies.into_iter().take(take_n).map(ChoiceOption::Player).collect()
        } else if allow_cancel {
            Vec::new()
        } else {
            players
                .iter()
                .take(min)
                .copied()
                .map(ChoiceOption::Player)
                .collect()
        }
    };

    let pick_ally = |want_damaged: bool| -> Vec<ChoiceOption> {
        let best = players
            .iter()
            .copied()
            .filter(|&t| public_attitude(&input.view, seat, t) > 0)
            .filter(|&t| {
                !want_damaged
                    || input
                        .view
                        .players
                        .get(t as usize)
                        .is_some_and(|p| p.hp < p.max_hp as i32)
            })
            .max_by_key(|&t| {
                let lost = input
                    .view
                    .players
                    .get(t as usize)
                    .map(|p| p.max_hp as i32 - p.hp)
                    .unwrap_or(0);
                (public_attitude(&input.view, seat, t), lost)
            });
        match best {
            Some(t) => vec![ChoiceOption::Player(t)],
            None if allow_cancel => Vec::new(),
            None => players
                .first()
                .copied()
                .map(ChoiceOption::Player)
                .into_iter()
                .collect(),
        }
    };

    let answer = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };

    match prompt {
        // Confirm 类（公开信息启发）
        "ow_xiyang_confirm" | "ow_maoding_confirm" | "ow_liudan_confirm" | "ow_jidong_confirm"
        | "ow_feiren3_confirm" | "ow_qinru_confirm" | "ow_shanxian_confirm" | "ow_shanhui_confirm"
        | "ow_lianpo_confirm" => {
            let confirm = options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied();
            answer(confirm.into_iter().collect())
        }
        "ow_zhiyuan_confirm" => {
            // 默认若能救或不想白伤盟友则可按公开信息决定：无敌我区分上下文时按 Confirm
            let confirm = options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied();
            answer(confirm.into_iter().collect())
        }
        "ow_tiandan_confirm" => {
            let cap = my_hp.clamp(0, 5) as usize;
            let hand_n = me.map(|p| p.hand_count).unwrap_or(0);
            if cap.saturating_sub(hand_n) >= 2 {
                let confirm = options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied();
                answer(confirm.into_iter().collect())
            } else {
                answer(Vec::new())
            }
        }
        "ow_zhencha_mode" => {
            let confirm = options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied();
            answer(confirm.into_iter().collect())
        }
        "ow_zhanlong_confirm" => {
            let has_le = input
                .view
                .players
                .get(seat as usize)
                .is_some_and(|p| p.judge.iter().any(|c| catalog.card_kind(*c) == crate::catalog::CardKind::Le));
            if has_le {
                answer(Vec::new())
            } else {
                let confirm = options.iter().find(|o| matches!(o, ChoiceOption::Confirm)).copied();
                answer(confirm.into_iter().collect())
            }
        }

        // 摸牌/回血二选一
        "ow_zhencha_draw_recover" | "ow_lichang2_dr" | "ow_ziyu_dr" => {
            let has_recover = options.contains(&ChoiceOption::Tag(1));
            let has_draw = options.contains(&ChoiceOption::Tag(0));
            let sel = if has_recover && my_hp < my_max_hp && (my_hp <= 2 || !has_draw) {
                vec![ChoiceOption::Tag(1)]
            } else if has_draw {
                vec![ChoiceOption::Tag(0)]
            } else {
                Vec::new()
            };
            answer(sel)
        }

        // 选 1/2 牌成本类
        "ow_qianggu2_discard" | "ow_pingzhang_discard" | "ow_mianzhen_shan" => {
            if cards.is_empty() && allow_cancel {
                answer(Vec::new())
            } else {
                answer(pick_cards(1))
            }
        }
        "ow_dianji_card" | "ow_feitiao_card" | "ow_liangou_card" | "ow_zhongdun_card"
        | "ow_paotai_card" | "ow_yinbo_card" | "ow_shoujia_card" | "ow_shanguang_card"
        | "ow_mianzhen_card" | "ow_dulei_card" | "ow_mujing_card" | "ow_lichang_card"
        | "ow_yihun_card" | "ow_xie_card" | "ow_luan_card" | "ow_feidan_card"
        | "ow_guangshu_card" | "ow_shouhu_card" => answer(pick_cards(1)),

        "ow_qianggu_cards" | "ow_maoding2_cards" | "ow_huoyu_cards" | "ow_huhuan_cards" => {
            answer(pick_cards(2))
        }
        "ow_zihui_discard" | "ow_kuoyin_cards" => answer(pick_cards(min.max(1))),

        "ow_juji_cards" => {
            let mut chosen = Vec::new();
            let mut seen_suits: Vec<Suit> = Vec::new();
            for &c in &cards {
                let s = catalog.cards[c as usize].suit;
                if !seen_suits.contains(&s) {
                    seen_suits.push(s);
                    chosen.push(ChoiceOption::Card(c));
                    if chosen.len() >= max.min(2) {
                        break;
                    }
                }
            }
            answer(chosen)
        }
        "ow_bingqiang_cards" => {
            let reds: Vec<CardId> = cards
                .iter()
                .copied()
                .filter(|c| matches!(catalog.cards[*c as usize].suit, Suit::Heart | Suit::Diamond))
                .collect();
            let blacks: Vec<CardId> = cards
                .iter()
                .copied()
                .filter(|c| matches!(catalog.cards[*c as usize].suit, Suit::Spade | Suit::Club))
                .collect();
            let chosen_pool = if blacks.len() >= reds.len() { blacks } else { reds };
            let chosen: Vec<ChoiceOption> = chosen_pool
                .into_iter()
                .take(min.max(1).min(max))
                .map(ChoiceOption::Card)
                .collect();
            answer(chosen)
        }

        // 选目标：敌方优先
        "ow_woliu_targets" | "ow_baoxue_targets" => answer(pick_enemies(max)),
        "ow_feitiao_target" | "ow_liangou_target" | "ow_shoujia_target" | "ow_shanguang_target"
        | "ow_mianzhen_target" | "ow_tuijin_target" | "ow_mujing_target" | "ow_yihun_target"
        | "ow_luan_target" => answer(pick_enemy(false)),
        "ow_juji_target" => answer(pick_enemy(true)),
        "ow_guangshu_target" | "ow_bingqiang_target" => {
            let enemy = pick_enemy(false);
            if !enemy.is_empty() {
                answer(enemy)
            } else {
                answer(pick_ally(false))
            }
        }

        // 选目标：友方优先
        "ow_zhongdun_target" | "ow_maoding2_target" | "ow_aqianghua_target" | "ow_xie_target"
        | "ow_lichang2_target" => answer(pick_ally(false)),
        "ow_shouhu_target" => answer(pick_ally(true)),
        "ow_sheng_targets" | "ow_yuedong_targets" => {
            let mut allies: Vec<PlayerId> = players
                .iter()
                .copied()
                .filter(|&t| public_attitude(&input.view, seat, t) > 0)
                .collect();
            allies.sort_by_key(|&t| {
                input
                    .view
                    .players
                    .get(t as usize)
                    .map(|p| p.hp)
                    .unwrap_or(4)
            });
            let take_n = allies.len().min(max);
            if take_n >= min && take_n > 0 {
                answer(allies.into_iter().take(take_n).map(ChoiceOption::Player).collect())
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(
                    players
                        .iter()
                        .take(min)
                        .copied()
                        .map(ChoiceOption::Player)
                        .collect(),
                )
            }
        }
        _ => None,
    }
}
