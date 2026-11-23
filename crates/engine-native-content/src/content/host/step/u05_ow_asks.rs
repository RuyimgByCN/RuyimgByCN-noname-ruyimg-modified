//! U05-3 (`shaai-ow`) 询问构造与恢复期望校验（单一事实源，保证 Runtime 与 Save/Restore `ExpectedAsk` 完全一致）。

use super::u05_ow::{
    alive_others, bingqiang_valid_targets, has_skill_u05, hand_and_equips, is_black_card,
    is_red_card, mujing_max_targets, mujing_valid_targets, prev_next_targets_clear_of,
    qinru_valid_card_suit, suit_of, flag,
};
use super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::ids::PlayerId;
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(crate) const PROMPT_OW_WOLIU_TARGETS: &str = "ow_woliu_targets";
pub(crate) const PROMPT_OW_QIANGGU_CARDS: &str = "ow_qianggu_cards";
pub(crate) const PROMPT_OW_QIANGGU2_DISCARD: &str = "ow_qianggu2_discard";
pub(crate) const PROMPT_OW_DIANJI_CARD: &str = "ow_dianji_card";
pub(crate) const PROMPT_OW_FEITIAO_CARD: &str = "ow_feitiao_card";
pub(crate) const PROMPT_OW_FEITIAO_TARGET: &str = "ow_feitiao_target";
pub(crate) const PROMPT_OW_ZHENCHA_MODE: &str = "ow_zhencha_mode";
pub(crate) const PROMPT_OW_ZHENCHA_DR: &str = "ow_zhencha_draw_recover";
pub(crate) const PROMPT_OW_PINGZHANG_CARD: &str = "ow_pingzhang_discard";
pub(crate) const PROMPT_OW_LIANGOU_CARD: &str = "ow_liangou_card";
pub(crate) const PROMPT_OW_LIANGOU_TARGET: &str = "ow_liangou_target";
pub(crate) const PROMPT_OW_XIYANG_CONFIRM: &str = "ow_xiyang_confirm";
pub(crate) const PROMPT_OW_QINRU_CONFIRM: &str = "ow_qinru_confirm";
pub(crate) const PROMPT_OW_ZHONGDUN_CARD: &str = "ow_zhongdun_card";
pub(crate) const PROMPT_OW_ZHONGDUN_TARGET: &str = "ow_zhongdun_target";
pub(crate) const PROMPT_OW_MAODING_CONFIRM: &str = "ow_maoding_confirm";
pub(crate) const PROMPT_OW_MAODING2_CARDS: &str = "ow_maoding2_cards";
pub(crate) const PROMPT_OW_MAODING2_TARGET: &str = "ow_maoding2_target";
pub(crate) const PROMPT_OW_PAOTAI_CARD: &str = "ow_paotai_card";
pub(crate) const PROMPT_OW_YINBO_CARD: &str = "ow_yinbo_card";
pub(crate) const PROMPT_OW_AQIANGHUA_TARGET: &str = "ow_aqianghua_target";
pub(crate) const PROMPT_OW_SHOUJIA_CARD: &str = "ow_shoujia_card";
pub(crate) const PROMPT_OW_SHOUJIA_TARGET: &str = "ow_shoujia_target";
pub(crate) const PROMPT_OW_LIUDAN_CONFIRM: &str = "ow_liudan_confirm";
pub(crate) const PROMPT_OW_TIANDAN_CONFIRM: &str = "ow_tiandan_confirm";
pub(crate) const PROMPT_OW_SHANGUANG_CARD: &str = "ow_shanguang_card";
pub(crate) const PROMPT_OW_SHANGUANG_TARGET: &str = "ow_shanguang_target";
pub(crate) const PROMPT_OW_BAOXUE_TARGETS: &str = "ow_baoxue_targets";
pub(crate) const PROMPT_OW_MIANZHEN_CARD: &str = "ow_mianzhen_card";
pub(crate) const PROMPT_OW_MIANZHEN_TARGET: &str = "ow_mianzhen_target";
pub(crate) const PROMPT_OW_MIANZHEN_SHAN: &str = "ow_mianzhen_shan";
pub(crate) const PROMPT_OW_ZHIYUAN_CONFIRM: &str = "ow_zhiyuan_confirm";
pub(crate) const PROMPT_OW_JUJI_CARDS: &str = "ow_juji_cards";
pub(crate) const PROMPT_OW_JUJI_TARGET: &str = "ow_juji_target";
pub(crate) const PROMPT_OW_DULEI_CARD: &str = "ow_dulei_card";
pub(crate) const PROMPT_OW_BINGQIANG_CARDS: &str = "ow_bingqiang_cards";
pub(crate) const PROMPT_OW_BINGQIANG_TARGET: &str = "ow_bingqiang_target";
pub(crate) const PROMPT_OW_JIDONG_CONFIRM: &str = "ow_jidong_confirm";
pub(crate) const PROMPT_OW_TUIJIN_TARGET: &str = "ow_tuijin_target";
pub(crate) const PROMPT_OW_ZIHUI_DISCARD: &str = "ow_zihui_discard";
pub(crate) const PROMPT_OW_MUJING_CARD: &str = "ow_mujing_card";
pub(crate) const PROMPT_OW_MUJING_TARGET: &str = "ow_mujing_target";
pub(crate) const PROMPT_OW_LICHANG_CARD: &str = "ow_lichang_card";
pub(crate) const PROMPT_OW_LICHANG2_TARGET: &str = "ow_lichang2_target";
pub(crate) const PROMPT_OW_LICHANG2_DR: &str = "ow_lichang2_dr";
pub(crate) const PROMPT_OW_ZHANLONG_CONFIRM: &str = "ow_zhanlong_confirm";
pub(crate) const PROMPT_OW_FEIREN3_CONFIRM: &str = "ow_feiren3_confirm";
pub(crate) const PROMPT_OW_YIHUN_CARD: &str = "ow_yihun_card";
pub(crate) const PROMPT_OW_YIHUN_TARGET: &str = "ow_yihun_target";
pub(crate) const PROMPT_OW_XIE_CARD: &str = "ow_xie_card";
pub(crate) const PROMPT_OW_XIE_TARGET: &str = "ow_xie_target";
pub(crate) const PROMPT_OW_LUAN_CARD: &str = "ow_luan_card";
pub(crate) const PROMPT_OW_LUAN_TARGET: &str = "ow_luan_target";
pub(crate) const PROMPT_OW_SHENG_TARGETS: &str = "ow_sheng_targets";
pub(crate) const PROMPT_OW_HUOYU_CARDS: &str = "ow_huoyu_cards";
pub(crate) const PROMPT_OW_FEIDAN_CARD: &str = "ow_feidan_card";
pub(crate) const PROMPT_OW_YUEDONG_TARGETS: &str = "ow_yuedong_targets";
pub(crate) const PROMPT_OW_HUHUAN_CARDS: &str = "ow_huhuan_cards";
pub(crate) const PROMPT_OW_KUOYIN_CARDS: &str = "ow_kuoyin_cards";
pub(crate) const PROMPT_OW_GUANGSHU_CARD: &str = "ow_guangshu_card";
pub(crate) const PROMPT_OW_GUANGSHU_TARGET: &str = "ow_guangshu_target";
pub(crate) const PROMPT_OW_ZIYU_DR: &str = "ow_ziyu_dr";
pub(crate) const PROMPT_OW_SHOUHU_CARD: &str = "ow_shouhu_card";
pub(crate) const PROMPT_OW_SHOUHU_TARGET: &str = "ow_shouhu_target";
pub(crate) const PROMPT_OW_SHANXIAN_CONFIRM: &str = "ow_shanxian_confirm";
pub(crate) const PROMPT_OW_SHANHUI_CONFIRM: &str = "ow_shanhui_confirm";
pub(crate) const PROMPT_OW_LIANPO_CONFIRM: &str = "ow_lianpo_confirm";

pub(super) const ASK_OW_WOLIU_TARGETS: u32 = 1;
pub(super) const ASK_OW_QIANGGU_CARDS: u32 = 2;
pub(super) const ASK_OW_QIANGGU2_DISCARD: u32 = 3;
pub(super) const ASK_OW_DIANJI_CARD: u32 = 4;
pub(super) const ASK_OW_FEITIAO_CARD: u32 = 5;
pub(super) const ASK_OW_FEITIAO_TARGET: u32 = 6;
pub(super) const ASK_OW_ZHENCHA_MODE: u32 = 7;
pub(super) const ASK_OW_ZHENCHA_DR: u32 = 8;
pub(super) const ASK_OW_PINGZHANG_CARD: u32 = 9;
pub(super) const ASK_OW_LIANGOU_CARD: u32 = 10;
pub(super) const ASK_OW_LIANGOU_TARGET: u32 = 11;
pub(super) const ASK_OW_XIYANG_CONFIRM: u32 = 12;
pub(super) const ASK_OW_QINRU_CONFIRM: u32 = 13;
pub(super) const ASK_OW_ZHONGDUN_CARD: u32 = 14;
pub(super) const ASK_OW_ZHONGDUN_TARGET: u32 = 15;
pub(super) const ASK_OW_MAODING_CONFIRM: u32 = 16;
pub(super) const ASK_OW_MAODING2_CARDS: u32 = 17;
pub(super) const ASK_OW_MAODING2_TARGET: u32 = 18;
pub(super) const ASK_OW_PAOTAI_CARD: u32 = 19;
pub(super) const ASK_OW_YINBO_CARD: u32 = 20;
pub(super) const ASK_OW_AQIANGHUA_TARGET: u32 = 21;
pub(super) const ASK_OW_SHOUJIA_CARD: u32 = 22;
pub(super) const ASK_OW_SHOUJIA_TARGET: u32 = 23;
pub(super) const ASK_OW_LIUDAN_CONFIRM: u32 = 24;
pub(super) const ASK_OW_TIANDAN_CONFIRM: u32 = 25;
pub(super) const ASK_OW_SHANGUANG_CARD: u32 = 26;
pub(super) const ASK_OW_SHANGUANG_TARGET: u32 = 27;
pub(super) const ASK_OW_BAOXUE_TARGETS: u32 = 28;
pub(super) const ASK_OW_MIANZHEN_CARD: u32 = 29;
pub(super) const ASK_OW_MIANZHEN_TARGET: u32 = 30;
pub(super) const ASK_OW_MIANZHEN_SHAN: u32 = 31;
pub(super) const ASK_OW_ZHIYUAN_CONFIRM: u32 = 32;
pub(super) const ASK_OW_JUJI_CARDS: u32 = 33;
pub(super) const ASK_OW_JUJI_TARGET: u32 = 34;
pub(super) const ASK_OW_DULEI_CARD: u32 = 35;
pub(super) const ASK_OW_BINGQIANG_CARDS: u32 = 36;
pub(super) const ASK_OW_BINGQIANG_TARGET: u32 = 37;
pub(super) const ASK_OW_JIDONG_CONFIRM: u32 = 38;
pub(super) const ASK_OW_TUIJIN_TARGET: u32 = 39;
pub(super) const ASK_OW_ZIHUI_DISCARD: u32 = 40;
pub(super) const ASK_OW_MUJING_CARD: u32 = 41;
pub(super) const ASK_OW_MUJING_TARGET: u32 = 42;
pub(super) const ASK_OW_LICHANG_CARD: u32 = 43;
pub(super) const ASK_OW_LICHANG2_TARGET: u32 = 44;
pub(super) const ASK_OW_LICHANG2_DR: u32 = 45;
pub(super) const ASK_OW_ZHANLONG_CONFIRM: u32 = 46;
pub(super) const ASK_OW_FEIREN3_CONFIRM: u32 = 47;
pub(super) const ASK_OW_YIHUN_CARD: u32 = 48;
pub(super) const ASK_OW_YIHUN_TARGET: u32 = 49;
pub(super) const ASK_OW_XIE_CARD: u32 = 50;
pub(super) const ASK_OW_XIE_TARGET: u32 = 51;
pub(super) const ASK_OW_LUAN_CARD: u32 = 52;
pub(super) const ASK_OW_LUAN_TARGET: u32 = 53;
pub(super) const ASK_OW_SHENG_TARGETS: u32 = 54;
pub(super) const ASK_OW_HUOYU_CARDS: u32 = 55;
pub(super) const ASK_OW_FEIDAN_CARD: u32 = 56;
pub(super) const ASK_OW_YUEDONG_TARGETS: u32 = 57;
pub(super) const ASK_OW_HUHUAN_CARDS: u32 = 58;
pub(super) const ASK_OW_KUOYIN_CARDS: u32 = 59;
pub(super) const ASK_OW_GUANGSHU_CARD: u32 = 60;
pub(super) const ASK_OW_GUANGSHU_TARGET: u32 = 61;
pub(super) const ASK_OW_ZIYU_DR: u32 = 62;
pub(super) const ASK_OW_SHOUHU_CARD: u32 = 63;
pub(super) const ASK_OW_SHOUHU_TARGET: u32 = 64;
pub(super) const ASK_OW_SHANXIAN_CONFIRM: u32 = 65;
pub(super) const ASK_OW_SHANHUI_CONFIRM: u32 = 66;
pub(super) const ASK_OW_LIANPO_CONFIRM: u32 = 67;

fn build_ask(
    observer: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> ExpectedAsk {
    ExpectedAsk {
        observer,
        prompt,
        kind,
        options,
        min,
        max,
        allow_cancel,
        source_skill: None,
    }
}

fn confirm_ask(observer: PlayerId, prompt: &'static str) -> ExpectedAsk {
    build_ask(
        observer,
        prompt,
        ChoiceKind::Confirm,
        vec![ChoiceOption::Confirm],
        0,
        1,
        true,
    )
}

fn draw_recover_options(core: &GameStateCore, p: PlayerId, allow_cancel: bool) -> (Vec<ChoiceOption>, usize) {
    let mut opts = vec![ChoiceOption::Tag(0)];
    if core.player(p).hp < core.player(p).max_hp as i32 {
        opts.push(ChoiceOption::Tag(1));
    }
    let min = if allow_cancel { 0 } else { 1 };
    (opts, min)
}

pub(in crate::content::host) fn build_ow_expected_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    cards: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> ExpectedAsk {
    let he = hand_and_equips(core, owner);
    let hand_opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
    let he_opts: Vec<ChoiceOption> = he.iter().copied().map(ChoiceOption::Card).collect();
    let other_opts: Vec<ChoiceOption> = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
    match ask_id {
        ASK_OW_WOLIU_TARGETS => {
            let max = other_opts.len().min(2);
            build_ask(owner, PROMPT_OW_WOLIU_TARGETS, ChoiceKind::SelectPlayers, other_opts, 1.min(max), max, true)
        }
        ASK_OW_QIANGGU_CARDS => build_ask(owner, PROMPT_OW_QIANGGU_CARDS, ChoiceKind::SelectCards, he_opts, 2, 2, true),
        ASK_OW_QIANGGU2_DISCARD => {
            let attacker = targets.first().copied().unwrap_or(owner);
            let opts: Vec<ChoiceOption> = core
                .player(attacker)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.card_kind(*c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            build_ask(attacker, PROMPT_OW_QIANGGU2_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_DIANJI_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    alive_others(core, owner).into_iter().any(|t| {
                        core.computed_distance(catalog, owner, t) <= 2
                            && catalog.ow_target_enabled(
                                core,
                                owner,
                                t,
                                Some(c),
                                CardKind::Jingleishan,
                                crate::event::DamageNature::Thunder,
                            )
                    })
                })
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_DIANJI_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_FEITIAO_CARD => build_ask(owner, PROMPT_OW_FEITIAO_CARD, ChoiceKind::SelectCards, he_opts, 1, 1, true),
        ASK_OW_FEITIAO_TARGET => build_ask(owner, PROMPT_OW_FEITIAO_TARGET, ChoiceKind::SelectPlayers, other_opts, 1, 1, true),
        ASK_OW_ZHENCHA_MODE => confirm_ask(owner, PROMPT_OW_ZHENCHA_MODE),
        ASK_OW_ZHENCHA_DR => {
            let (opts, min) = draw_recover_options(core, owner, true);
            build_ask(owner, PROMPT_OW_ZHENCHA_DR, ChoiceKind::PickTag, opts, min, 1, true)
        }
        ASK_OW_PINGZHANG_CARD => {
            let want = if target == Some(owner) { Suit::Heart } else { Suit::Spade };
            let opts: Vec<ChoiceOption> = he
                .into_iter()
                .filter(|c| suit_of(core, catalog, owner, *c) == want)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_PINGZHANG_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_LIANGOU_CARD => build_ask(owner, PROMPT_OW_LIANGOU_CARD, ChoiceKind::SelectCards, he_opts, 1, 1, true),
        ASK_OW_LIANGOU_TARGET => build_ask(owner, PROMPT_OW_LIANGOU_TARGET, ChoiceKind::SelectPlayers, other_opts, 1, 1, true),
        ASK_OW_XIYANG_CONFIRM => confirm_ask(owner, PROMPT_OW_XIYANG_CONFIRM),
        ASK_OW_QINRU_CONFIRM => confirm_ask(owner, PROMPT_OW_QINRU_CONFIRM),
        ASK_OW_ZHONGDUN_CARD => build_ask(owner, PROMPT_OW_ZHONGDUN_CARD, ChoiceKind::SelectCards, he_opts, 1, 1, true),
        ASK_OW_ZHONGDUN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| core.player(*p).hujia == 0)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_ZHONGDUN_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_MAODING_CONFIRM => confirm_ask(owner, PROMPT_OW_MAODING_CONFIRM),
        ASK_OW_MAODING2_CARDS => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.card_kind(*c) == CardKind::Hslingjian)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_MAODING2_CARDS, ChoiceKind::SelectCards, opts, 2, 2, true)
        }
        ASK_OW_MAODING2_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| core.player(*p).hujia == 0)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_MAODING2_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_PAOTAI_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.card_kind(*c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_PAOTAI_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_YINBO_CARD => {
            let opts: Vec<ChoiceOption> = he
                .into_iter()
                .filter(|c| suit_of(core, catalog, owner, *c) == Suit::Spade)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_YINBO_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_AQIANGHUA_TARGET => build_ask(owner, PROMPT_OW_AQIANGHUA_TARGET, ChoiceKind::SelectPlayers, other_opts, 1, 1, true),
        ASK_OW_SHOUJIA_CARD => build_ask(owner, PROMPT_OW_SHOUJIA_CARD, ChoiceKind::SelectCards, hand_opts, 1, 1, true),
        ASK_OW_SHOUJIA_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| !has_skill_u05(core, catalog, *p, "shoujia2"))
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_SHOUJIA_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_LIUDAN_CONFIRM => confirm_ask(owner, PROMPT_OW_LIUDAN_CONFIRM),
        ASK_OW_TIANDAN_CONFIRM => confirm_ask(owner, PROMPT_OW_TIANDAN_CONFIRM),
        ASK_OW_SHANGUANG_CARD => {
            let opts: Vec<ChoiceOption> = he
                .into_iter()
                .filter(|c| suit_of(core, catalog, owner, *c) == Suit::Diamond)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_SHANGUANG_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_SHANGUANG_TARGET => {
            let rng = core.attack_range(catalog, owner);
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| core.computed_distance(catalog, owner, *p) <= rng)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_SHANGUANG_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_BAOXUE_TARGETS => {
            let black_n = he.iter().filter(|c| is_black_card(core, catalog, owner, **c)).count();
            let max = black_n.min(other_opts.len());
            build_ask(owner, PROMPT_OW_BAOXUE_TARGETS, ChoiceKind::SelectPlayers, other_opts, 1.min(max), max, true)
        }
        ASK_OW_MIANZHEN_CARD => build_ask(owner, PROMPT_OW_MIANZHEN_CARD, ChoiceKind::SelectCards, he_opts, 1, 1, true),
        ASK_OW_MIANZHEN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| !has_skill_u05(core, catalog, *p, "mianzhen2"))
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_MIANZHEN_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_MIANZHEN_SHAN => {
            let obs = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = crate::kernel::response_kind_cards_core(core, catalog, obs, CardKind::Shan)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build_ask(obs, PROMPT_OW_MIANZHEN_SHAN, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_ZHIYUAN_CONFIRM => confirm_ask(owner, PROMPT_OW_ZHIYUAN_CONFIRM),
        ASK_OW_JUJI_CARDS => {
            let mut suits = 0u8;
            for &c in &he {
                suits |= match suit_of(core, catalog, owner, c) {
                    Suit::Spade => 1,
                    Suit::Heart => 2,
                    Suit::Club => 4,
                    Suit::Diamond => 8,
                };
            }
            let max = (suits.count_ones() as usize).clamp(1, 4).min(he_opts.len());
            build_ask(owner, PROMPT_OW_JUJI_CARDS, ChoiceKind::SelectCards, he_opts, 1, max, true)
        }
        ASK_OW_JUJI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| !core.player(*p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_JUJI_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_DULEI_CARD => build_ask(owner, PROMPT_OW_DULEI_CARD, ChoiceKind::SelectCards, hand_opts, 1, 1, true),
        ASK_OW_BINGQIANG_CARDS => {
            let max = he_opts.len().max(1);
            build_ask(owner, PROMPT_OW_BINGQIANG_CARDS, ChoiceKind::SelectCards, he_opts, 1, max, true)
        }
        ASK_OW_BINGQIANG_TARGET => {
            let opts: Vec<ChoiceOption> = bingqiang_valid_targets(core, catalog)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_BINGQIANG_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_JIDONG_CONFIRM => confirm_ask(owner, PROMPT_OW_JIDONG_CONFIRM),
        ASK_OW_TUIJIN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| core.computed_distance(catalog, owner, *p) > 1)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_TUIJIN_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_ZIHUI_DISCARD => {
            let obs = target.unwrap_or(owner);
            let n = cards.get(1).copied().unwrap_or(1) as usize;
            let opts: Vec<ChoiceOption> = hand_and_equips(core, obs).into_iter().map(ChoiceOption::Card).collect();
            build_ask(obs, PROMPT_OW_ZIHUI_DISCARD, ChoiceKind::SelectCards, opts, n, n, true)
        }
        ASK_OW_MUJING_CARD => {
            let opts: Vec<ChoiceOption> = he
                .into_iter()
                .filter(|c| is_black_card(core, catalog, owner, *c))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_MUJING_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_MUJING_TARGET => {
            let cid = cards.first().copied().unwrap_or(0);
            let opts: Vec<ChoiceOption> = mujing_valid_targets(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            let max = mujing_max_targets(core, catalog, owner, cid).min(opts.len()).max(1);
            build_ask(owner, PROMPT_OW_MUJING_TARGET, ChoiceKind::SelectPlayers, opts, 1, max, true)
        }
        ASK_OW_LICHANG_CARD => {
            let opts: Vec<ChoiceOption> = he
                .into_iter()
                .filter(|c| is_red_card(core, catalog, owner, *c))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_LICHANG_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_LICHANG2_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| core.computed_distance(catalog, owner, *p) <= 1)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_LICHANG2_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_LICHANG2_DR => {
            let obs = target.unwrap_or(owner);
            let (opts, min) = draw_recover_options(core, obs, false);
            build_ask(obs, PROMPT_OW_LICHANG2_DR, ChoiceKind::PickTag, opts, min, 1, false)
        }
        ASK_OW_ZHANLONG_CONFIRM => confirm_ask(owner, PROMPT_OW_ZHANLONG_CONFIRM),
        ASK_OW_FEIREN3_CONFIRM => confirm_ask(owner, PROMPT_OW_FEIREN3_CONFIRM),
        ASK_OW_YIHUN_CARD => {
            let opts: Vec<ChoiceOption> = he
                .into_iter()
                .filter(|c| is_black_card(core, catalog, owner, *c))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_YIHUN_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_YIHUN_TARGET => build_ask(owner, PROMPT_OW_YIHUN_TARGET, ChoiceKind::SelectPlayers, other_opts, 1, 1, true),
        ASK_OW_XIE_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| suit_of(core, catalog, owner, *c) == Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_XIE_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_XIE_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| !has_skill_u05(core, catalog, *p, "xie2"))
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_XIE_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_LUAN_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| suit_of(core, catalog, owner, *c) == Suit::Spade)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_LUAN_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_LUAN_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| !has_skill_u05(core, catalog, *p, "luan2"))
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_LUAN_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_SHENG_TARGETS => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| core.player(*p).hp < core.player(*p).max_hp as i32)
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len().max(1);
            build_ask(owner, PROMPT_OW_SHENG_TARGETS, ChoiceKind::SelectPlayers, opts, 1, max, true)
        }
        ASK_OW_HUOYU_CARDS => {
            let opts: Vec<ChoiceOption> = he
                .into_iter()
                .filter(|c| is_red_card(core, catalog, owner, *c))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_HUOYU_CARDS, ChoiceKind::SelectCards, opts, 2, 2, true)
        }
        ASK_OW_FEIDAN_CARD => build_ask(owner, PROMPT_OW_FEIDAN_CARD, ChoiceKind::SelectCards, he_opts, 1, 1, true),
        ASK_OW_YUEDONG_TARGETS => {
            let recover = flag(core, owner, "ow_yuedong_recover") > 0;
            let num = 1 + flag(core, owner, "ow_yuedong_num") as usize;
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| !recover || core.player(*p).hp < core.player(*p).max_hp as i32)
                .map(ChoiceOption::Player)
                .collect();
            let max = num.min(opts.len()).max(1);
            build_ask(owner, PROMPT_OW_YUEDONG_TARGETS, ChoiceKind::SelectPlayers, opts, 1, max, true)
        }
        ASK_OW_HUHUAN_CARDS => build_ask(owner, PROMPT_OW_HUHUAN_CARDS, ChoiceKind::SelectCards, he_opts, 2, 2, true),
        ASK_OW_KUOYIN_CARDS => {
            let has_num = flag(core, owner, "ow_yuedong_num") > 0;
            let has_eff = flag(core, owner, "ow_yuedong_eff") > 0;
            let min = if has_num { 2 } else { 1 };
            let max = if has_eff { 1 } else { 2 }.min(he_opts.len()).max(min);
            build_ask(owner, PROMPT_OW_KUOYIN_CARDS, ChoiceKind::SelectCards, he_opts, min, max, true)
        }
        ASK_OW_GUANGSHU_CARD => build_ask(owner, PROMPT_OW_GUANGSHU_CARD, ChoiceKind::SelectCards, he_opts, 1, 1, true),
        ASK_OW_GUANGSHU_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|p| {
                    !has_skill_u05(core, catalog, *p, "guangshu_heart")
                        && !has_skill_u05(core, catalog, *p, "guangshu_spade")
                        && !has_skill_u05(core, catalog, *p, "guangshu_club")
                        && !has_skill_u05(core, catalog, *p, "guangshu_diamond")
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_GUANGSHU_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_ZIYU_DR => {
            let (opts, min) = draw_recover_options(core, owner, true);
            build_ask(owner, PROMPT_OW_ZIYU_DR, ChoiceKind::PickTag, opts, min, 1, true)
        }
        ASK_OW_SHOUHU_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.card_kind(*c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_OW_SHOUHU_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_OW_SHOUHU_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|p| core.player(*p).hp < core.player(*p).max_hp as i32)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_OW_SHOUHU_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_OW_SHANXIAN_CONFIRM => confirm_ask(owner, PROMPT_OW_SHANXIAN_CONFIRM),
        ASK_OW_SHANHUI_CONFIRM => confirm_ask(owner, PROMPT_OW_SHANHUI_CONFIRM),
        ASK_OW_LIANPO_CONFIRM => confirm_ask(owner, PROMPT_OW_LIANPO_CONFIRM),
        _ => {
            let _ = (qinru_valid_card_suit, prev_next_targets_clear_of);
            confirm_ask(owner, PROMPT_OW_XIYANG_CONFIRM)
        }
    }
}

pub(in crate::content::host) fn ask_u05_ow(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> ExpectedAsk {
    build_ow_expected_ask(
        card.unwrap_or(ASK_OW_XIYANG_CONFIRM),
        owner,
        target,
        targets,
        cards,
        core,
        catalog,
    )
}
