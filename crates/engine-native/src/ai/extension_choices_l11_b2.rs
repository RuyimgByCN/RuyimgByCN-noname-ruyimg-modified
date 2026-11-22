//! L11（太虚幻境）Batch 2 挂起询问的公开信息集 AI 启发式策略。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_l11_b2_choice_action(
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
    let self_cell = input.view.players.iter().find(|p| p.seat == seat);
    let answer = |selection: Vec<ChoiceOption>| {
        Some(GameAction::Answer {
            choice_id: id,
            selection,
        })
    };
    let best_player_by = |want_enemy: bool| -> Option<(u8, i32)> {
        options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Player(p) => Some(p),
                _ => None,
            })
            .map(|p| {
                let att = public_attitude(&input.view, seat, p);
                (p, if want_enemy { -att } else { att })
            })
            .max_by_key(|&(_, score)| score)
    };
    let sorted_cards = |low_first: bool| -> Vec<u32> {
        let mut cards: Vec<_> = options
            .iter()
            .filter_map(|o| match *o {
                ChoiceOption::Card(c) => Some(c),
                _ => None,
            })
            .collect();
        cards.sort_by_key(|&c| {
            let v = extension_card_value(catalog, input, c);
            if low_first {
                v
            } else {
                -v
            }
        });
        cards
    };

    match prompt {
        "txhj_xiaoxiaoSkill1:confirm" => {
            if options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else {
                answer(vec![ChoiceOption::Pass])
            }
        }
        "txyicheng:confirm"
        | "txnangce:confirm"
        | "txkuangyi:confirm"
        | "mdqianxi7:confirm"
        | "mdqianxi:confirm"
        | "txyonglve:confirm"
        | "txjiang:confirm" => {
            let want = match prompt {
                "txnangce:confirm" => {
                    public_attitude(&input.view, seat, input.view.turn) > 0
                }
                "txduanzui:confirm" => false,
                "txkuangyi:confirm" => self_cell.is_some_and(|p| p.hp >= 2 || !p.linked),
                "txyonglve:confirm" => {
                    // `extension.js:6035`: prefer deck top if hand lacks junk / high non-heart cards
                    self_cell.is_some_and(|p| p.hand_count <= 2)
                }
                _ => true,
            };
            if want && options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else if options.contains(&ChoiceOption::Pass) {
                answer(vec![ChoiceOption::Pass])
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "txduanzui:confirm" => {
            let enemy_low = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .any(|p| public_attitude(&input.view, seat, p) < 0)
                || input.view.players.iter().any(|p| {
                    p.seat != seat
                        && p.alive
                        && p.hand_count > 0
                        && p.hand_count < 3
                        && public_attitude(&input.view, seat, p.seat) < 0
                });
            let ally_low = input.view.players.iter().any(|p| {
                p.seat != seat
                    && p.alive
                    && p.hand_count > 0
                    && p.hand_count < 3
                    && public_attitude(&input.view, seat, p.seat) > 0
            });
            if enemy_low && !ally_low && options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else if options.contains(&ChoiceOption::Pass) {
                answer(vec![ChoiceOption::Pass])
            } else {
                answer(Vec::new())
            }
        }
        "txmanzhen:confirm_damage" => {
            let hp_ok = self_cell.is_some_and(|p| p.hp > 1);
            if hp_ok && options.contains(&ChoiceOption::Confirm) {
                answer(vec![ChoiceOption::Confirm])
            } else if options.contains(&ChoiceOption::Pass) {
                answer(vec![ChoiceOption::Pass])
            } else {
                answer(Vec::new())
            }
        }
        // Negative/damage/attack target selection
        "txlveming:choose_target"
        | "txlangxi:choose_target"
        | "txtanbei:choose_target"
        | "mdqianxi7:choose_target"
        | "mdqianxi:choose_target"
        | "zylongdan:damage_target"
        | "txdaoji:choose_target"
        | "txhj_mouqiang:choose_target"
        | "txzhennu:choose_target"
        | "txchengyuan:choose_target"
        | "txbenji:choose_target" => {
            if let Some((p, score)) = best_player_by(true) {
                if !allow_cancel || score > 0 {
                    return answer(vec![ChoiceOption::Player(p)]);
                }
            }
            if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        // Supportive target selection
        "oldfuman:choose_target"
        | "gdsanchen:choose_target"
        | "zylongdan:recover_target"
        | "txjuxiang:choose_target" => {
            if let Some((p, score)) = best_player_by(false) {
                if !allow_cancel || score > 0 {
                    return answer(vec![ChoiceOption::Player(p)]);
                }
            }
            if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "txhj_suoming:choose_targets" => {
            let picked: Vec<ChoiceOption> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p)
                        if public_attitude(&input.view, seat, p) < 0 =>
                    {
                        Some(ChoiceOption::Player(p))
                    }
                    _ => None,
                })
                .take(max.max(1))
                .collect();
            if !picked.is_empty() {
                answer(picked)
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(options.iter().take(min.max(1)).copied().collect())
            }
        }
        "txmanzhen:choose_targets" => {
            let mut ranked: Vec<(u8, i32)> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some((p, -public_attitude(&input.view, seat, p))),
                    _ => None,
                })
                .collect();
            ranked.sort_by_key(|&(_, s)| -s);
            let picked: Vec<ChoiceOption> = ranked
                .into_iter()
                .take(min)
                .map(|(p, _)| ChoiceOption::Player(p))
                .collect();
            answer(picked)
        }
        // Steal high-value cards
        "txhj_yanSkill3:choose_card"
        | "txhj_manmanSkill2:choose_card"
        | "txdaoji:choose_he"
        | "txhj_suzhi:choose_he" => {
            let cards = sorted_cards(false);
            answer(
                cards
                    .into_iter()
                    .take(min.max(1))
                    .map(ChoiceOption::Card)
                    .collect(),
            )
        }
        "txhj_mouqiang:choose_he" => {
            let cards = sorted_cards(false);
            answer(
                cards
                    .into_iter()
                    .take(min.max(1))
                    .map(ChoiceOption::Card)
                    .collect(),
            )
        }
        "txyinli:choose_cards" => {
            let cards = sorted_cards(false);
            answer(cards.into_iter().map(ChoiceOption::Card).collect())
        }
        // Discard / give lowest value cards
        "oldfuman:choose_card"
        | "hgkurou:choose_card"
        | "txyicheng:discard_card"
        | "mdqianxi7:discard_card"
        | "txdaoji:discard_card"
        | "txchengyuan:choose_card" => {
            let cards = sorted_cards(true);
            if cards.len() >= min.max(1) {
                answer(
                    cards
                        .into_iter()
                        .take(min.max(1))
                        .map(ChoiceOption::Card)
                        .collect(),
                )
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(Vec::new())
            }
        }
        "gdsanchen:discard_cards" => {
            // `extension.js:4515`: prefer 3 distinct card categories (`Basic`, `Trick`, `Equip`)
            let cards = sorted_cards(true);
            let mut chosen = Vec::new();
            let mut seen_cats = [false; 3];
            for &c in &cards {
                if chosen.len() >= min.max(1) {
                    break;
                }
                let k = catalog.cards.get(c as usize).map(|cd| cd.kind);
                let cat = match k {
                    Some(kk) if kk.is_basic_kind() => 0,
                    Some(kk) if kk.is_equip() => 2,
                    _ => 1,
                };
                if !seen_cats[cat] {
                    seen_cats[cat] = true;
                    chosen.push(c);
                }
            }
            for &c in &cards {
                if chosen.len() >= min.max(1) {
                    break;
                }
                if !chosen.contains(&c) {
                    chosen.push(c);
                }
            }
            answer(chosen.into_iter().map(ChoiceOption::Card).collect())
        }
        "txmanzhen:choose_cards" => {
            let enemy_cnt = input
                .view
                .players
                .iter()
                .filter(|p| {
                    p.seat != seat && p.alive && public_attitude(&input.view, seat, p.seat) < 0
                })
                .count();
            let want = enemy_cnt.min(max).max(min);
            let cards = sorted_cards(true);
            if cards.len() >= want && want > 0 {
                answer(cards.into_iter().take(want).map(ChoiceOption::Card).collect())
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(Vec::new())
            }
        }
        "txyongguan_fate:discard" => {
            let cards = sorted_cards(true);
            let take_n = max.min(cards.len());
            if take_n >= min {
                answer(cards.into_iter().take(take_n).map(ChoiceOption::Card).collect())
            } else if allow_cancel {
                answer(Vec::new())
            } else {
                answer(Vec::new())
            }
        }
        // Modes & Rank tags
        "txtanbei:choose_mode" => {
            // `extension.js:4700`: target loses least by letting attacker give unlimited hits vs losing a card, or 0/1
            if options.contains(&ChoiceOption::Tag(1)) {
                answer(vec![ChoiceOption::Tag(1)])
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "txlveming:choose_rank" => {
            // Pick an uncommon rank to minimize chance of rank hit (or first valid)
            if let Some(&o) = options.first() {
                answer(vec![o])
            } else {
                None
            }
        }
        "txyingyang:choose_mode" => {
            // Boost (+3) by default unless cancel
            if options.contains(&ChoiceOption::Tag(0)) {
                answer(vec![ChoiceOption::Tag(0)])
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        "txhj_huangmen:choose_mode" => {
            if options.contains(&ChoiceOption::Tag(0)) {
                answer(vec![ChoiceOption::Tag(0)])
            } else {
                answer(options.iter().take(1).copied().collect())
            }
        }
        _ => None,
    }
}
