//! Batch U05-6b (`shahai-hearth` Part 2): 询问构造与恢复期望校验。

use super::{
    b6b_flag, can_jingxiang_card_respond, is_b6b_red_suit, sha_target_options, tuteng_owned,
    tuteng_pool,
};
use super::super::u05_hearth::{alive_others, flag, has_any_skill, in_attack_range_hearth, set_flag};
use super::super::{HostBehavior, HostMachine, HostStage};
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(crate) const PROMPT_HS_XIENENG_PICK: &str = "hs_xieneng_pick";
pub(crate) const PROMPT_HS_FBEIFA_TARGET: &str = "hs_fbeifa_target";
pub(crate) const PROMPT_HS_OLDFBEIFA_TARGET: &str = "hs_oldfbeifa_target";
pub(crate) const PROMPT_HS_YUFA_TARGET: &str = "hs_yufa_target";
pub(crate) const PROMPT_HS_BINGYAN_CARD: &str = "hs_bingyan_card";
pub(crate) const PROMPT_HS_SHIFA_DRAW_CONFIRM: &str = "hs_shifa_draw_confirm";
pub(crate) const PROMPT_HS_OLDSHIFA_DRAW_CONFIRM: &str = "hs_oldshifa_draw_confirm";
#[allow(dead_code)]
pub(crate) const PROMPT_HS_YUANZHENG_CONFIRM: &str = "hs_yuanzheng_confirm";
pub(crate) const PROMPT_HS_YUANZHENG_CARD: &str = "hs_yuanzheng_card";
pub(crate) const PROMPT_HS_YULU_TARGETS: &str = "hs_yulu_targets";
pub(crate) const PROMPT_HS_YULU_DISCARD: &str = "hs_yulu_discard";
pub(crate) const PROMPT_HS_DUZHANG_TARGET: &str = "hs_duzhang_target";
pub(crate) const PROMPT_HS_QIANGHUA_CONFIRM: &str = "hs_qianghua_confirm";
pub(crate) const PROMPT_HS_BIRI_DISCARD: &str = "hs_biri_discard";
pub(crate) const PROMPT_HS_JIAOHUI_TARGET: &str = "hs_jiaohui_target";
pub(crate) const PROMPT_HS_JIAOHUI_CHOICE: &str = "hs_jiaohui_choice";
pub(crate) const PROMPT_HS_BIMENG_CONFIRM: &str = "hs_bimeng_confirm";
pub(crate) const PROMPT_HS_MALYMOWANG_DISCOVER: &str = "hs_malymowang_discover";
pub(crate) const PROMPT_HS_LINGZHOU_TARGET: &str = "hs_lingzhou_target";
pub(crate) const PROMPT_HS_LINGZHOU_CHOICE: &str = "hs_lingzhou_choice";
pub(crate) const PROMPT_HS_XMOJIAN_TARGET: &str = "hs_xmojian_target";
pub(crate) const PROMPT_HS_FUSHI_TARGET: &str = "hs_fushi_target";
pub(crate) const PROMPT_HS_OLDFUSHI_TARGET: &str = "hs_oldfushi_target";
pub(crate) const PROMPT_HS_ENZE_TARGET: &str = "hs_enze_target";
pub(crate) const PROMPT_HS_OLDENZE_TARGET: &str = "hs_oldenze_target";
pub(crate) const PROMPT_HS_GUOZAI2_DISCARD: &str = "hs_guozai2_discard";
pub(crate) const PROMPT_HS_HUANWU_TARGET: &str = "hs_huanwu_target";
pub(crate) const PROMPT_HS_JINGMENG_CONFIRM: &str = "hs_jingmeng_confirm";
pub(crate) const PROMPT_HS_HSWUJI_CONFIRM: &str = "hs_hswuji_confirm";
pub(crate) const PROMPT_HS_BINGSHUANG_CONFIRM: &str = "hs_bingshuang_confirm";
pub(crate) const PROMPT_HS_YANSHU_CONFIRM: &str = "hs_yanshu_confirm";
pub(crate) const PROMPT_HS_OLDYANSHU_CONFIRM: &str = "hs_oldyanshu_confirm";
pub(crate) const PROMPT_HS_SHENGYAN_CONFIRM: &str = "hs_shengyan_confirm";
pub(crate) const PROMPT_HS_DUNJI_TARGETS: &str = "hs_dunji_targets";
pub(crate) const PROMPT_HS_FENGXING_DISCARD: &str = "hs_fengxing_discard";
pub(crate) const PROMPT_HS_FENGXIAN_DISCARD: &str = "hs_fengxian_discard";
pub(crate) const PROMPT_HS_ZHANHOU_CARD: &str = "hs_zhanhou_card";
pub(crate) const PROMPT_HS_SHIJIE_TARGET: &str = "hs_shijie_target";
pub(crate) const PROMPT_HS_SHENGGUANG_CARD: &str = "hs_shengguang_card";
pub(crate) const PROMPT_HS_SHENGGUANG_TARGET: &str = "hs_shengguang_target";
pub(crate) const PROMPT_HS_XINCI_CARD: &str = "hs_xinci_card";
pub(crate) const PROMPT_HS_XINCI_TARGET: &str = "hs_xinci_target";
pub(crate) const PROMPT_HS_ANYING_CARD: &str = "hs_anying_card";
pub(crate) const PROMPT_HS_HUANJUE_PICK: &str = "hs_huanjue_pick";
pub(crate) const PROMPT_HS_BINGJIA_CARD: &str = "hs_bingjia_card";
pub(crate) const PROMPT_HS_MDZHOUFU_CARD: &str = "hs_mdzhoufu_card";
pub(crate) const PROMPT_HS_MDZHOUFU_TARGET: &str = "hs_mdzhoufu_target";
pub(crate) const PROMPT_HS_XIANZHI_CHOICE: &str = "hs_xianzhi_choice";
pub(crate) const PROMPT_HS_JINGXIANG_TARGET: &str = "hs_jingxiang_target";
pub(crate) const PROMPT_HS_JINGXIANG_CARD: &str = "hs_jingxiang_card";
pub(crate) const PROMPT_HS_WLIANJI_CONFIRM: &str = "hs_wlianji_confirm";
pub(crate) const PROMPT_HS_MENGUN_DISCARD: &str = "hs_mengun_discard";
pub(crate) const PROMPT_HS_JIHUO_DISCARD: &str = "hs_jihuo_discard";
pub(crate) const PROMPT_HS_TUTENG_PICK: &str = "hs_tuteng_pick";
pub(crate) const PROMPT_HS_TUTENG_REPLACE: &str = "hs_tuteng_replace";
pub(crate) const PROMPT_HS_TUTENG7_TARGET: &str = "hs_tuteng7_target";
pub(crate) const PROMPT_HS_SHIXU_DISCARD: &str = "hs_shixu_discard";
pub(crate) const PROMPT_HS_ENZE_DISCARD: &str = "hs_enze_discard";
pub(crate) const PROMPT_HS_REJIEMING_TARGET: &str = "hs_rejieming_target";
pub(crate) const PROMPT_HS_XINLIEGONG_CONFIRM: &str = "hs_xinliegong_confirm";

pub(super) const ASK_B6B_XIENENG_PICK: u32 = 201;
pub(super) const ASK_B6B_FBEIFA_TARGET: u32 = 202;
pub(super) const ASK_B6B_OLDFBEIFA_TARGET: u32 = 203;
pub(super) const ASK_B6B_YUFA_TARGET: u32 = 204;
pub(super) const ASK_B6B_BINGYAN_CARD: u32 = 205;
pub(super) const ASK_B6B_SHIFA_DRAW_CONFIRM: u32 = 206;
pub(super) const ASK_B6B_OLDSHIFA_DRAW_CONFIRM: u32 = 207;
#[allow(dead_code)]
pub(super) const ASK_B6B_YUANZHENG_CONFIRM: u32 = 208;
pub(super) const ASK_B6B_YULU_TARGETS: u32 = 209;
pub(super) const ASK_B6B_YULU_DISCARD: u32 = 210;
pub(super) const ASK_B6B_DUZHANG_TARGET: u32 = 211;
pub(super) const ASK_B6B_QIANGHUA_CONFIRM: u32 = 212;
pub(super) const ASK_B6B_BIRI_DISCARD: u32 = 213;
pub(super) const ASK_B6B_JIAOHUI_TARGET: u32 = 214;
pub(super) const ASK_B6B_JIAOHUI_CHOICE: u32 = 215;
pub(super) const ASK_B6B_BIMENG_CONFIRM: u32 = 216;
pub(super) const ASK_B6B_MALYMOWANG_DISCOVER: u32 = 217;
pub(super) const ASK_B6B_LINGZHOU_TARGET: u32 = 218;
pub(super) const ASK_B6B_LINGZHOU_CHOICE: u32 = 219;
pub(super) const ASK_B6B_XMOJIAN_TARGET: u32 = 220;
pub(super) const ASK_B6B_FUSHI_TARGET: u32 = 221;
pub(super) const ASK_B6B_OLDFUSHI_TARGET: u32 = 222;
pub(super) const ASK_B6B_ENZE_TARGET: u32 = 223;
pub(super) const ASK_B6B_OLDENZE_TARGET: u32 = 224;
pub(super) const ASK_B6B_GUOZAI2_DISCARD: u32 = 225;
pub(super) const ASK_B6B_HUANWU_TARGET: u32 = 226;
pub(super) const ASK_B6B_JINGMENG_CONFIRM: u32 = 227;
pub(super) const ASK_B6B_HSWUJI_CONFIRM: u32 = 228;
pub(super) const ASK_B6B_BINGSHUANG_CONFIRM: u32 = 229;
pub(super) const ASK_B6B_YANSHU_CONFIRM: u32 = 230;
pub(super) const ASK_B6B_OLDYANSHU_CONFIRM: u32 = 231;
pub(super) const ASK_B6B_SHENGYAN_CONFIRM: u32 = 232;
pub(super) const ASK_B6B_DUNJI_TARGETS: u32 = 233;
pub(super) const ASK_B6B_FENGXING_DISCARD: u32 = 234;
pub(super) const ASK_B6B_FENGXIAN_DISCARD: u32 = 235;
pub(super) const ASK_B6B_ZHANHOU_CARD: u32 = 236;
pub(super) const ASK_B6B_SHIJIE_TARGET: u32 = 237;
pub(super) const ASK_B6B_SHENGGUANG_CARD: u32 = 238;
pub(super) const ASK_B6B_SHENGGUANG_TARGET: u32 = 239;
pub(super) const ASK_B6B_XINCI_CARD: u32 = 240;
pub(super) const ASK_B6B_XINCI_TARGET: u32 = 241;
pub(super) const ASK_B6B_ANYING_CARD: u32 = 242;
pub(super) const ASK_B6B_HUANJUE_PICK: u32 = 243;
pub(super) const ASK_B6B_BINGJIA_CARD: u32 = 244;
pub(super) const ASK_B6B_MDZHOUFU_CARD: u32 = 245;
pub(super) const ASK_B6B_MDZHOUFU_TARGET: u32 = 246;
pub(super) const ASK_B6B_XIANZHI_CHOICE: u32 = 247;
pub(super) const ASK_B6B_JINGXIANG_TARGET: u32 = 248;
pub(super) const ASK_B6B_JINGXIANG_CARD: u32 = 249;
pub(super) const ASK_B6B_WLIANJI_CONFIRM: u32 = 250;
pub(super) const ASK_B6B_MENGUN_DISCARD: u32 = 251;
pub(super) const ASK_B6B_JIHUO_DISCARD: u32 = 252;
pub(super) const ASK_B6B_TUTENG_PICK: u32 = 253;
pub(super) const ASK_B6B_TUTENG_REPLACE: u32 = 254;
pub(super) const ASK_B6B_TUTENG7_TARGET: u32 = 255;
pub(super) const ASK_B6B_SHIXU_DISCARD: u32 = 256;
pub(super) const ASK_B6B_ENZE_DISCARD: u32 = 257;
pub(super) const ASK_B6B_YUANZHENG_CARD: u32 = 258;
pub(super) const ASK_B6B_REJIEMING_TARGET: u32 = 259;
pub(super) const ASK_B6B_XINLIEGONG_CONFIRM: u32 = 260;

fn ask_key(behavior: HostBehavior, stage: HostStage) -> String {
    format!("hs_b6b_ask:{}:{}", behavior.tag(), stage.tag())
}

pub(super) fn set_b6b_ask(
    core: &mut GameStateCore,
    behavior: HostBehavior,
    owner: PlayerId,
    stage: HostStage,
    ask_id: u32,
) {
    set_flag(core, owner, &ask_key(behavior, stage), ask_id);
}

pub(super) fn clear_b6b_ask(m: &HostMachine, ctx: &mut Ctx<'_>, stage: HostStage) {
    set_b6b_ask(ctx.core, m.behavior, m.owner, stage, 0);
}

pub(super) fn wait_b6b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_b6b_ask(ctx.core, m.behavior, m.owner, stage, ask_id);
    m.stage = stage;
    let exp = build_b6b_expected_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    );
    Flow::WaitChoice {
        observer: exp.observer,
        spec: ChoiceSpec {
            kind: exp.kind,
            prompt: exp.prompt,
            options: exp.options,
            min: exp.min,
            max: exp.max,
            allow_cancel: exp.allow_cancel,
        },
    }
}

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

fn hand_eq(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .chain(core.player(owner).equips.iter())
        .copied()
        .collect()
}

pub(super) fn bingyan_aoe_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
    kind: CardKind,
) -> Vec<PlayerId> {
    let nature = kind.aoe_nature();
    let mut v: Vec<PlayerId> = core
        .seat_order_from(core.next_alive(owner))
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                    core, catalog, owner, p, cid, kind, nature,
                )
        })
        .collect();
    v.reverse();
    v
}

pub(super) fn bingyan_eligible_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    hand_eq(core, owner)
        .into_iter()
        .filter(|&cid| {
            let kind = if is_b6b_red_suit(catalog.effective_card_suit(core, owner, cid)) {
                CardKind::Chiyuxi
            } else {
                CardKind::Jingleishan
            };
            !bingyan_aoe_targets(core, catalog, owner, cid, kind).is_empty()
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_b6b_expected_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> ExpectedAsk {
    match ask_id {
        ASK_B6B_XIENENG_PICK => build_ask(
            owner,
            PROMPT_HS_XIENENG_PICK,
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)],
            1,
            1,
            true,
        ),
        ASK_B6B_FBEIFA_TARGET | ASK_B6B_OLDFBEIFA_TARGET | ASK_B6B_XMOJIAN_TARGET => {
            let prompt = match ask_id {
                ASK_B6B_FBEIFA_TARGET => PROMPT_HS_FBEIFA_TARGET,
                ASK_B6B_OLDFBEIFA_TARGET => PROMPT_HS_OLDFBEIFA_TARGET,
                _ => PROMPT_HS_XMOJIAN_TARGET,
            };
            build_ask(
                owner,
                prompt,
                ChoiceKind::SelectPlayers,
                sha_target_options(core, catalog, owner),
                1,
                1,
                true,
            )
        }
        ASK_B6B_YUFA_TARGET => {
            let excl = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != excl && core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_YUFA_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_BINGYAN_CARD => {
            let opts: Vec<ChoiceOption> = bingyan_eligible_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_BINGYAN_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_SHIFA_DRAW_CONFIRM
        | ASK_B6B_OLDSHIFA_DRAW_CONFIRM
        | ASK_B6B_YUANZHENG_CONFIRM
        | ASK_B6B_QIANGHUA_CONFIRM
        | ASK_B6B_BIMENG_CONFIRM
        | ASK_B6B_JINGMENG_CONFIRM
        | ASK_B6B_HSWUJI_CONFIRM
        | ASK_B6B_BINGSHUANG_CONFIRM
        | ASK_B6B_YANSHU_CONFIRM
        | ASK_B6B_OLDYANSHU_CONFIRM
        | ASK_B6B_SHENGYAN_CONFIRM
        | ASK_B6B_XIANZHI_CHOICE
        | ASK_B6B_WLIANJI_CONFIRM
        | ASK_B6B_XINLIEGONG_CONFIRM => {
            let prompt = match ask_id {
                ASK_B6B_SHIFA_DRAW_CONFIRM => PROMPT_HS_SHIFA_DRAW_CONFIRM,
                ASK_B6B_OLDSHIFA_DRAW_CONFIRM => PROMPT_HS_OLDSHIFA_DRAW_CONFIRM,
                ASK_B6B_YUANZHENG_CONFIRM => PROMPT_HS_YUANZHENG_CONFIRM,
                ASK_B6B_QIANGHUA_CONFIRM => PROMPT_HS_QIANGHUA_CONFIRM,
                ASK_B6B_BIMENG_CONFIRM => PROMPT_HS_BIMENG_CONFIRM,
                ASK_B6B_JINGMENG_CONFIRM => PROMPT_HS_JINGMENG_CONFIRM,
                ASK_B6B_HSWUJI_CONFIRM => PROMPT_HS_HSWUJI_CONFIRM,
                ASK_B6B_BINGSHUANG_CONFIRM => PROMPT_HS_BINGSHUANG_CONFIRM,
                ASK_B6B_YANSHU_CONFIRM => PROMPT_HS_YANSHU_CONFIRM,
                ASK_B6B_OLDYANSHU_CONFIRM => PROMPT_HS_OLDYANSHU_CONFIRM,
                ASK_B6B_SHENGYAN_CONFIRM => PROMPT_HS_SHENGYAN_CONFIRM,
                ASK_B6B_XIANZHI_CHOICE => PROMPT_HS_XIANZHI_CHOICE,
                ASK_B6B_XINLIEGONG_CONFIRM => PROMPT_HS_XINLIEGONG_CONFIRM,
                _ => PROMPT_HS_WLIANJI_CONFIRM,
            };
            build_ask(
                owner,
                prompt,
                ChoiceKind::Confirm,
                vec![ChoiceOption::Confirm],
                0,
                1,
                true,
            )
        }
        ASK_B6B_YULU_TARGETS => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            let max_n = opts.len().max(1);
            build_ask(
                owner,
                PROMPT_HS_YULU_TARGETS,
                ChoiceKind::SelectPlayers,
                opts,
                1,
                max_n,
                true,
            )
        }
        ASK_B6B_YULU_DISCARD => {
            let actor = target.unwrap_or(owner);
            let pl = core.player(actor);
            let opts: Vec<ChoiceOption> = pl
                .hand
                .iter()
                .chain(pl.equips.iter())
                .chain(pl.judge.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let want = (card.unwrap_or(1) as usize).min(opts.len()).max(1);
            build_ask(
                actor,
                PROMPT_HS_YULU_DISCARD,
                ChoiceKind::SelectCards,
                opts,
                want,
                want,
                false,
            )
        }
        ASK_B6B_DUZHANG_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_DUZHANG_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_BIRI_DISCARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c) == CardKind::Shan)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_BIRI_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_JIAOHUI_TARGET | ASK_B6B_LINGZHOU_TARGET => {
            let prompt = if ask_id == ASK_B6B_JIAOHUI_TARGET {
                PROMPT_HS_JIAOHUI_TARGET
            } else {
                PROMPT_HS_LINGZHOU_TARGET
            };
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, prompt, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_JIAOHUI_CHOICE | ASK_B6B_LINGZHOU_CHOICE => {
            let actor = target.unwrap_or(owner);
            let prompt = if ask_id == ASK_B6B_JIAOHUI_CHOICE {
                PROMPT_HS_JIAOHUI_CHOICE
            } else {
                PROMPT_HS_LINGZHOU_CHOICE
            };
            let mut opts = vec![ChoiceOption::Tag(0)];
            if core.player(actor).hp < core.player(actor).max_hp as i32 {
                opts.push(ChoiceOption::Tag(1));
            }
            build_ask(actor, prompt, ChoiceKind::PickTag, opts, 1, 1, false)
        }
        ASK_B6B_MALYMOWANG_DISCOVER => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(
                owner,
                PROMPT_HS_MALYMOWANG_DISCOVER,
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            )
        }
        ASK_B6B_FUSHI_TARGET | ASK_B6B_OLDFUSHI_TARGET => {
            let prompt = if ask_id == ASK_B6B_FUSHI_TARGET {
                PROMPT_HS_FUSHI_TARGET
            } else {
                PROMPT_HS_OLDFUSHI_TARGET
            };
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    core.player(p).alive && core.player(p).hp < core.player(p).max_hp as i32
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, prompt, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_ENZE_TARGET | ASK_B6B_OLDENZE_TARGET => {
            let prompt = if ask_id == ASK_B6B_ENZE_TARGET {
                PROMPT_HS_ENZE_TARGET
            } else {
                PROMPT_HS_OLDENZE_TARGET
            };
            let my_h = core.player(owner).hand.len();
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).hand.len() != my_h)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, prompt, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_GUOZAI2_DISCARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let want = (card.unwrap_or(1) as usize).min(opts.len()).max(1);
            build_ask(
                owner,
                PROMPT_HS_GUOZAI2_DISCARD,
                ChoiceKind::SelectCards,
                opts,
                want,
                want,
                false,
            )
        }
        ASK_B6B_HUANWU_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && flag(core, p, "hs_huanwu_marked") == 0)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_HUANWU_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_DUNJI_TARGETS => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| in_attack_range_hearth(core, catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            let max_n = (core.player(owner).hujia as usize)
                .min(opts.len())
                .max(1);
            build_ask(
                owner,
                PROMPT_HS_DUNJI_TARGETS,
                ChoiceKind::SelectPlayers,
                opts,
                1,
                max_n,
                true,
            )
        }
        ASK_B6B_FENGXING_DISCARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_FENGXING_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_FENGXIAN_DISCARD => {
            let actor = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = core
                .player(actor)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build_ask(actor, PROMPT_HS_FENGXIAN_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_B6B_ZHANHOU_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| catalog.card_kind(c).equip_slot() == Some(EquipSlot::Armor))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_ZHANHOU_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_SHIJIE_TARGET | ASK_B6B_JINGXIANG_TARGET => {
            let prompt = if ask_id == ASK_B6B_SHIJIE_TARGET {
                PROMPT_HS_SHIJIE_TARGET
            } else {
                PROMPT_HS_JINGXIANG_TARGET
            };
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, prompt, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_SHENGGUANG_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| is_b6b_red_suit(catalog.effective_card_suit(core, owner, c)))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_SHENGGUANG_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_SHENGGUANG_TARGET => {
            let awakened = flag(core, owner, "hs_anying_awakened") > 0;
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    core.player(p).alive
                        && (awakened || core.player(p).hp < core.player(p).max_hp as i32)
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_SHENGGUANG_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_XINCI_CARD | ASK_B6B_ANYING_CARD => {
            let prompt = if ask_id == ASK_B6B_XINCI_CARD {
                PROMPT_HS_XINCI_CARD
            } else {
                PROMPT_HS_ANYING_CARD
            };
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| !is_b6b_red_suit(catalog.effective_card_suit(core, owner, c)))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, prompt, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_XINCI_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_XINCI_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_HUANJUE_PICK => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_HUANJUE_PICK, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_BINGJIA_CARD | ASK_B6B_JIHUO_DISCARD => {
            let prompt = if ask_id == ASK_B6B_BINGJIA_CARD {
                PROMPT_HS_BINGJIA_CARD
            } else {
                PROMPT_HS_JIHUO_DISCARD
            };
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, prompt, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_MDZHOUFU_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| !is_b6b_red_suit(catalog.effective_card_suit(core, owner, c)))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_MDZHOUFU_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_MDZHOUFU_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !has_any_skill(core, catalog, p, "mdzhoufu2"))
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_MDZHOUFU_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_JINGXIANG_CARD => {
            let tgt = target.unwrap_or(owner);
            let rctx = cards.first().copied().unwrap_or(1) as u8;
            let opts: Vec<ChoiceOption> = core
                .player(tgt)
                .hand
                .iter()
                .copied()
                .filter(|&c| can_jingxiang_card_respond(core, catalog, owner, c, rctx))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_JINGXIANG_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_MENGUN_DISCARD => {
            let want_code = card.unwrap_or(0);
            let want_suit = match want_code {
                0 => Suit::Spade,
                1 => Suit::Heart,
                2 => Suit::Club,
                _ => Suit::Diamond,
            };
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == want_suit)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_MENGUN_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_B6B_TUTENG_PICK => {
            let awake = b6b_flag(core, owner, "hs_tuteng_awake") > 0;
            let owned = tuteng_owned(core, catalog, owner);
            let avail: Vec<ChoiceOption> = tuteng_pool(awake)
                .into_iter()
                .filter(|idx| !owned.contains(idx))
                .map(ChoiceOption::Tag)
                .collect();
            build_ask(owner, PROMPT_HS_TUTENG_PICK, ChoiceKind::PickTag, avail, 1, 1, false)
        }
        ASK_B6B_TUTENG_REPLACE => {
            let owned: Vec<ChoiceOption> = tuteng_owned(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Tag)
                .collect();
            build_ask(
                owner,
                PROMPT_HS_TUTENG_REPLACE,
                ChoiceKind::PickTag,
                owned,
                1,
                1,
                false,
            )
        }
        ASK_B6B_TUTENG7_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).hp < core.player(p).max_hp as i32)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_TUTENG7_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_REJIEMING_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_REJIEMING_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_B6B_SHIXU_DISCARD => {
            let actor = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = hand_eq(core, actor)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let want = (card.unwrap_or(1) as usize).min(opts.len()).max(1);
            build_ask(actor, PROMPT_HS_SHIXU_DISCARD, ChoiceKind::SelectCards, opts, want, want, false)
        }
        ASK_B6B_ENZE_DISCARD => {
            let actor = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = core
                .player(actor)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let want = (card.unwrap_or(1) as usize).min(opts.len()).max(1);
            build_ask(actor, PROMPT_HS_ENZE_DISCARD, ChoiceKind::SelectCards, opts, want, want, false)
        }
        ASK_B6B_YUANZHENG_CARD => {
            let tgt = target.unwrap_or(owner);
            let pl = core.player(tgt);
            let opts: Vec<ChoiceOption> = pl
                .judge
                .iter()
                .chain(pl.equips.iter())
                .chain(pl.hand.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_YUANZHENG_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        _ => build_ask(
            owner,
            PROMPT_HS_BIMENG_CONFIRM,
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u05_b6b(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = flag(core, owner, &ask_key(behavior, stage));
    if ask_id == 0 {
        return None;
    }
    Some(build_b6b_expected_ask(
        ask_id, owner, target, targets, card, cards, core, catalog,
    ))
}
