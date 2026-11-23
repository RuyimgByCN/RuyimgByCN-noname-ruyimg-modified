//! U05-5a（杀海拾遗 · 昆特牌）选择的 AI 公开信息决策策略。

use super::{extension_card_value, public_attitude};
use crate::choice::ChoiceOption;
use crate::engine::GameAction;
use crate::view::PolicyInput;

const WEATHER_DEBUFFS: &[&str] = &[
    "gw_qinpendayu",
    "gw_birinongwu",
    "gw_ciguhanshuang",
    "gw_nuhaifengbao",
    "gw_baobaoshu",
    "gw_qinpen_state",
    "gw_birin_state",
    "gw_cigu_state",
];

fn has_harmful_state(view: &crate::view::PlayerView, seat: u8) -> bool {
    view.players.get(seat as usize).is_some_and(|p| {
        !p.judge.is_empty()
            || p.skills
                .iter()
                .any(|s| WEATHER_DEBUFFS.contains(s))
    })
}

fn gold_spell_tag_priority(tag: u8) -> i32 {
    // 顺序对应 step/u05_b5a_gwent/mod.rs GOLD_SPELL_PICK_NAMES
    match tag {
        9 => 95,  // gw_tunshi
        3 => 90,  // gw_aerdeyin
        4 => 85,  // gw_ansha
        7 => 80,  // gw_zhongmozhizhan
        8 => 78,  // gw_ganhan
        1 => 75,  // gw_yigeniyin
        0 => 70,  // gw_hudiewu
        2 => 65,  // gw_leizhoushu
        6 => 60,  // gw_niuquzhijing
        10 => 55, // gw_chongci
        5 => 45,  // gw_xinsheng
        _ => 10,
    }
}

fn silver_spell_tag_priority(tag: u8) -> i32 {
    // 顺序对应 step/u05_b5a_gwent/mod.rs SILVER_SPELL_PICK_NAMES
    match tag {
        3 => 92,  // gw_zhuoshao
        0 => 88,  // gw_youer
        1 => 86,  // gw_tongdi
        6 => 82,  // gw_zuihouyuanwang
        7 => 78,  // gw_zirankuizeng
        2 => 75,  // gw_fuyuan
        11 => 72, // gw_guaiwuchaoxue
        9 => 68,  // gw_baishuang
        8 => 65,  // gw_nuhaifengbao
        10 => 62, // gw_baobaoshu
        4 => 58,  // gw_butianshu
        5 => 55,  // gw_shizizhaohuan
        _ => 10,
    }
}

fn bronze_spell_tag_priority(tag: u8) -> i32 {
    // 优先护盾与回复（与目标选择正向收益一致），其次伤害与天气
    match tag {
        8 => 95,  // gw_kunenfayin
        9 => 90,  // gw_yanziyaoshui
        6 => 80,  // gw_aozuzhilei
        3 => 72,  // gw_baoxueyaoshui
        10 => 68, // gw_wenyi
        11 => 65, // gw_shanbengshu
        2 => 60,  // gw_ciguhanshuang
        1 => 56,  // gw_birinongwu
        0 => 54,  // gw_qinpendayu
        5 => 50,  // gw_zumoshoukao
        4 => 45,  // gw_zhihuanjun
        7 => 40,  // gw_poxiao
        _ => 10,
    }
}

fn pick_lowest_cards(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut cards: Vec<_> = options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| extension_card_value(catalog, input, c));
    cards.into_iter().take(count).map(ChoiceOption::Card).collect()
}

fn pick_highest_cards(
    catalog: &crate::catalog::Catalog,
    input: &PolicyInput,
    options: &[ChoiceOption],
    count: usize,
) -> Vec<ChoiceOption> {
    let mut cards: Vec<_> = options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) => Some(c),
            _ => None,
        })
        .collect();
    cards.sort_by_key(|&c| -extension_card_value(catalog, input, c));
    cards.into_iter().take(count).map(ChoiceOption::Card).collect()
}

fn best_player_option(
    input: &PolicyInput,
    options: &[ChoiceOption],
    prefer_friendly: bool,
) -> Option<ChoiceOption> {
    let seat = input.view.seat;
    options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) => Some(p),
            _ => None,
        })
        .max_by_key(|&p| {
            let att = public_attitude(&input.view, seat, p) as i64;
            let hp = input
                .view
                .players
                .get(p as usize)
                .map(|pl| pl.hp as i64)
                .unwrap_or(0);
            if prefer_friendly {
                (att, -hp)
            } else {
                (-att, -hp)
            }
        })
        .map(ChoiceOption::Player)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_u05_5a_choice_action(
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
    match prompt {
        "gw_poxiao_branch" => {
            let friend_needs_cleanse = input.view.players.iter().any(|p| {
                p.alive
                    && public_attitude(&input.view, seat, p.seat) >= 0
                    && has_harmful_state(&input.view, p.seat)
            });
            let preferred = if friend_needs_cleanse && options.contains(&ChoiceOption::Tag(0)) {
                ChoiceOption::Tag(0)
            } else if options.contains(&ChoiceOption::Tag(1)) {
                ChoiceOption::Tag(1)
            } else {
                *options.first()?
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection: vec![preferred],
            })
        }
        "gw_poxiao_targets" => {
            let mut players: Vec<_> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            players.sort_by_key(|&p| {
                let att = public_attitude(&input.view, seat, p);
                let harmful = i32::from(has_harmful_state(&input.view, p));
                -(att * 10 + harmful * 20)
            });
            let mut picked: Vec<_> = players
                .iter()
                .copied()
                .filter(|&p| public_attitude(&input.view, seat, p) >= 0 && has_harmful_state(&input.view, p))
                .take(max)
                .map(ChoiceOption::Player)
                .collect();
            if picked.len() < min {
                picked = players.into_iter().take(min).map(ChoiceOption::Player).collect();
            }
            Some(GameAction::Answer {
                choice_id: id,
                selection: picked,
            })
        }
        "gw_xinsheng_general" => {
            let best = *options.first()?;
            Some(GameAction::Answer {
                choice_id: id,
                selection: vec![best],
            })
        }
        "gw_xinsheng_target" | "gwmaoxian_airuiting_target" | "gwmaoxian_huoge_target" => {
            let selection = best_player_option(input, options, false).into_iter().collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "gw_tongdi_pick" | "gwmaoxian_kaerweite_pick" => {
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick_highest_cards(catalog, input, options, min.max(1)),
            })
        }
        "gw_butianshu_delay" => {
            let mut tags: Vec<u8> = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Tag(t) => Some(t),
                    _ => None,
                })
                .collect();
            tags.sort_by_key(|&t| match t {
                0 => 0, // Le
                1 => 1, // Bing
                3 => 2, // Guiyoujie
                _ => 3, // ShanDian
            });
            let sel = tags.first().copied().map(ChoiceOption::Tag).into_iter().collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection: sel,
            })
        }
        "gw_zuihouyuanwang_discard" | "gw_baobaoshu_discard" => {
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick_lowest_cards(catalog, input, options, min.max(1)),
            })
        }
        "gwmaoxian_bulanwang_discard" => {
            let cap = max.min(options.len());
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick_lowest_cards(catalog, input, options, cap.max(min)),
            })
        }
        "gw_zirankuizeng_spell" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Tag(t) => Some(t),
                    _ => None,
                })
                .max_by_key(|&t| bronze_spell_tag_priority(t))
                .map(ChoiceOption::Tag)
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection: best,
            })
        }
        "gw_zirankuizeng_target" => {
            let has_friendly_injured_or_unshielded = options.iter().any(|o| match *o {
                ChoiceOption::Player(p) if public_attitude(&input.view, seat, p) >= 0 => input
                    .view
                    .players
                    .get(p as usize)
                    .is_some_and(|pl| {
                        pl.hp < pl.max_hp as i32
                            || (!pl.skills.contains(&"gw_kunenfayin")
                                && !pl.skills.contains(&"gw_kunen_state"))
                    }),
                _ => false,
            });
            let sel = best_player_option(input, options, has_friendly_injured_or_unshielded)
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection: sel,
            })
        }
        "gwmaoxian_luoqi_sha" => {
            let has_enemy = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) < 0);
            let selection = if has_enemy {
                pick_lowest_cards(catalog, input, options, 1)
            } else if allow_cancel {
                Vec::new()
            } else {
                options.iter().take(min).copied().collect()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "gwmaoxian_airuiting_sha" => {
            // 不打杀会直接失去 1 点体力，优先出最低价值的杀（无可出或自血充足且仅能砍友时亦避免无益自损）
            let my_hp = input
                .view
                .players
                .get(seat as usize)
                .map(|p| p.hp)
                .unwrap_or(0);
            let has_enemy = input
                .view
                .players
                .iter()
                .any(|p| p.alive && p.seat != seat && public_attitude(&input.view, seat, p.seat) < 0);
            let selection = if has_enemy || my_hp <= 1 || !allow_cancel {
                pick_lowest_cards(catalog, input, options, 1)
            } else {
                Vec::new()
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection,
            })
        }
        "gwmaoxian_fulisi_discard" | "gwmaoxian_huoge_pick" => {
            let count = max.min(options.len()).max(min);
            Some(GameAction::Answer {
                choice_id: id,
                selection: pick_highest_cards(catalog, input, options, count),
            })
        }
        "gwmaoxian_gaier_branch" => {
            let self_injured = input
                .view
                .players
                .get(seat as usize)
                .is_some_and(|p| p.hp < p.max_hp as i32);
            let preferred = if self_injured && options.contains(&ChoiceOption::Tag(0)) {
                ChoiceOption::Tag(0)
            } else if options.contains(&ChoiceOption::Tag(1)) {
                ChoiceOption::Tag(1)
            } else {
                *options.first()?
            };
            Some(GameAction::Answer {
                choice_id: id,
                selection: vec![preferred],
            })
        }
        "gwmaoxian_aisinie_pick" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Tag(t) => Some(t),
                    _ => None,
                })
                .max_by_key(|&t| silver_spell_tag_priority(t))
                .map(ChoiceOption::Tag)
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection: best,
            })
        }
        "gw_huangjiashenpan_pick" | "gwmaoxian_falanxisika_pick" => {
            let best = options
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Tag(t) => Some(t),
                    _ => None,
                })
                .max_by_key(|&t| gold_spell_tag_priority(t))
                .map(ChoiceOption::Tag)
                .into_iter()
                .collect();
            Some(GameAction::Answer {
                choice_id: id,
                selection: best,
            })
        }
        _ => None,
    }
}
