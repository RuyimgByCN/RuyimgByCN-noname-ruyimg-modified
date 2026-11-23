//! Batch U05-6a (`shahai-hearth` Part 1): 询问构造与恢复期望校验（单一事实源，保证
//! 运行时 `Flow::WaitChoice` 与快照恢复 `ExpectedAsk` 完全一致）。

use super::u05_hearth::{
    alive_others, exact_base_name, flag, has_any_skill, has_skill_hearth, hearth_card_type_tag,
    in_attack_range_hearth, is_du_card_hearth, set_flag,
};
use super::*;
use crate::catalog::{Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(crate) const PROMPT_HS_SHANDIANJIAN_SHOW: &str = "hs_shandianjian_show";
pub(crate) const PROMPT_HS_SHANDIANJIAN_DISCARD: &str = "hs_shandianjian_discard";
pub(crate) const PROMPT_HS_ZHAOMINGDAN_DISCARD: &str = "hs_zhaomingdan_discard";
pub(crate) const PROMPT_HS_DUNPAIGEDANG_DISCARD: &str = "hs_dunpaigedang_discard";
pub(crate) const PROMPT_HS_CHUANSONGMEN_USE: &str = "hs_chuansongmen_use";
pub(crate) const PROMPT_HS_SHIJIESHU_DISCARD: &str = "hs_shijieshu_discard";
pub(crate) const PROMPT_HS_BUWENDINGYIBIAN_RECAST: &str = "hs_buwendingyibian_recast";
pub(crate) const PROMPT_HS_MORIJINGXIANG_GAIN: &str = "hs_morijingxiang_gain";
pub(crate) const PROMPT_HS_NVWUDEGANGUO_DISCARD: &str = "hs_nvwudeganguo_discard";
pub(crate) const PROMPT_HS_GUHUO_GIVE: &str = "hs_guhuo_give";

pub(crate) const PROMPT_HS_HSHUANYU_DISCOVER: &str = "hs_hshuanyu_discover";
pub(crate) const PROMPT_HS_HUALING_TARGET: &str = "hs_hualing_target";
pub(crate) const PROMPT_HS_HUALING_SKILL: &str = "hs_hualing_skill";
pub(crate) const PROMPT_HS_OLDHUANJUE_PICK: &str = "hs_oldhuanjue_pick";
pub(crate) const PROMPT_HS_HSHUANLING_DISCARD: &str = "hs_hshuanling_discard";
pub(crate) const PROMPT_HS_ASYOUZHANG_USE: &str = "hs_asyouzhang_use";
pub(crate) const PROMPT_HS_NSBEIMING_ORDER: &str = "hs_nsbeiming_order";
pub(crate) const PROMPT_HS_HSNITAI_SKILL: &str = "hs_hsnitai_skill";
pub(crate) const PROMPT_HS_HSPUZHAO_CARD: &str = "hs_hspuzhao_card";
pub(crate) const PROMPT_HS_HSTIANQI_CARD: &str = "hs_hstianqi_card";

pub(crate) const PROMPT_HS_YINDAN_CARD: &str = "hs_yindan_card";
pub(crate) const PROMPT_HS_HLLINGXI_TARGET: &str = "hs_hllingxi_target";
pub(crate) const PROMPT_HS_HLLINGXI_DISCARD: &str = "hs_hllingxi_discard";
pub(crate) const PROMPT_HS_XIYONG_USE: &str = "hs_xiyong_use";
pub(crate) const PROMPT_HS_TANSUO_CARD: &str = "hs_tansuo_card";
pub(crate) const PROMPT_HS_TANSUO_DISCOVER: &str = "hs_tansuo_discover";
pub(crate) const PROMPT_HS_LIEQI_TARGET: &str = "hs_lieqi_target";
pub(crate) const PROMPT_HS_LIEQI_GUESS: &str = "hs_lieqi_guess";
pub(crate) const PROMPT_HS_AZAOWU_KIND: &str = "hs_azaowu_kind";
pub(crate) const PROMPT_HS_AZAOWU_CARD: &str = "hs_azaowu_card";
pub(crate) const PROMPT_HS_AZAOWU_TARGET: &str = "hs_azaowu_target";
pub(crate) const PROMPT_HS_ZHUILIE_DISCARD: &str = "hs_zhuilie_discard";

pub(crate) const PROMPT_HS_SZBIANSHEN_CONFIRM: &str = "hs_szbianshen_confirm";
pub(crate) const PROMPT_HS_SZBIANSHEN_GENERAL: &str = "hs_szbianshen_general";
pub(crate) const PROMPT_HS_KEKAO_PICK: &str = "hs_kekao_pick";
pub(crate) const PROMPT_HS_JINHUA_SKILL: &str = "hs_jinhua_skill";
pub(crate) const PROMPT_HS_KQIZHOU_PICK: &str = "hs_kqizhou_pick";
pub(crate) const PROMPT_HS_MENGYE_TARGET: &str = "hs_mengye_target";
pub(crate) const PROMPT_HS_KUIXIN_TARGET: &str = "hs_kuixin_target";
pub(crate) const PROMPT_HS_FUHUA_CARD: &str = "hs_fuhua_card";
pub(crate) const PROMPT_HS_FUHUA_TARGET: &str = "hs_fuhua_target";
pub(crate) const PROMPT_HS_FUHUA_CHOICE: &str = "hs_fuhua_choice";
pub(crate) const PROMPT_HS_FUHUA2_GIVE: &str = "hs_fuhua2_give";
pub(crate) const PROMPT_HS_QIANHOU_DISCARD: &str = "hs_qianhou_discard";
pub(crate) const PROMPT_HS_ZHONGJI_DISCARD: &str = "hs_zhongji_discard";
pub(crate) const PROMPT_HS_JINZHOU_TARGET: &str = "hs_jinzhou_target";

pub(crate) const PROMPT_HS_MIDIAN_CARD: &str = "hs_midian_card";
pub(crate) const PROMPT_HS_XINGLUO_TARGETS: &str = "hs_xingluo_targets";
pub(crate) const PROMPT_HS_XINGLUO_DISCARD: &str = "hs_xingluo_discard";
pub(crate) const PROMPT_HS_XINGLUO_PICK: &str = "hs_xingluo_pick";
pub(crate) const PROMPT_HS_YUELU_CARD: &str = "hs_yuelu_card";
pub(crate) const PROMPT_HS_YUSHOU_CARD: &str = "hs_yushou_card";
pub(crate) const PROMPT_HS_AOSHU_CARD: &str = "hs_aoshu_card";
pub(crate) const PROMPT_HS_BZHUIJI_CONFIRM: &str = "hs_bzhuiji_confirm";
pub(crate) const PROMPT_HS_LIANJIN_CARD: &str = "hs_lianjin_card";
pub(crate) const PROMPT_HS_YINGXI_CARD: &str = "hs_yingxi_card";
pub(crate) const PROMPT_HS_YINGXI_TARGET: &str = "hs_yingxi_target";
pub(crate) const PROMPT_HS_PEIYU_TARGET: &str = "hs_peiyu_target";
pub(crate) const PROMPT_HS_WZHANYI_CONFIRM: &str = "hs_wzhanyi_confirm";
pub(crate) const PROMPT_HS_WZHANYI_USE: &str = "hs_wzhanyi_use";
pub(crate) const PROMPT_HS_WZHANYI_TARGET: &str = "hs_wzhanyi_target";

pub(crate) const PROMPT_HS_YUANSU_CONFIRM: &str = "hs_yuansu_confirm";
pub(crate) const PROMPT_HS_NUYAN2_KIND: &str = "hs_nuyan2_kind";
pub(crate) const PROMPT_HS_NUYAN2_TARGET: &str = "hs_nuyan2_target";
pub(crate) const PROMPT_HS_NUYAN_KIND: &str = "hs_nuyan_kind";
pub(crate) const PROMPT_HS_NUYAN_CARD: &str = "hs_nuyan_card";
pub(crate) const PROMPT_HS_NUYAN_TARGET: &str = "hs_nuyan_target";
pub(crate) const PROMPT_HS_ZENGLI_TARGET: &str = "hs_zengli_target";
pub(crate) const PROMPT_HS_MOBAO_CARDS: &str = "hs_mobao_cards";
pub(crate) const PROMPT_HS_XIANJI2_DISCARD: &str = "hs_xianji2_discard";
pub(crate) const PROMPT_HS_TANMI_USE: &str = "hs_tanmi_use";
pub(crate) const PROMPT_HS_TANMI_TARGET: &str = "hs_tanmi_target";
pub(crate) const PROMPT_HS_XUEREN_CONFIRM: &str = "hs_xueren_confirm";
pub(crate) const PROMPT_HS_MAOXIAN_SKILL: &str = "hs_maoxian_skill";
pub(crate) const PROMPT_HS_QIANGHUAX_CARDS: &str = "hs_qianghuax_cards";
pub(crate) const PROMPT_HS_ZHUIZONG_CARDS: &str = "hs_zhuizong_cards";
pub(crate) const PROMPT_HS_ZHUIZONG_PICK: &str = "hs_zhuizong_pick";
pub(crate) const PROMPT_HS_XUNBAO2_DISCARD: &str = "hs_xunbao2_discard";

// Compact numeric ask IDs saved into `vm_aux.flags` keyed by `(owner, hs_ask_key(behavior, stage))`.
pub(super) const ASK_HS_SHANDIANJIAN_SHOW: u32 = 1;
pub(super) const ASK_HS_SHANDIANJIAN_DISCARD: u32 = 2;
pub(super) const ASK_HS_ZHAOMINGDAN_DISCARD: u32 = 3;
pub(super) const ASK_HS_DUNPAIGEDANG_DISCARD: u32 = 4;
pub(super) const ASK_HS_CHUANSONGMEN_USE: u32 = 5;
pub(super) const ASK_HS_SHIJIESHU_DISCARD: u32 = 6;
pub(super) const ASK_HS_BUWENDINGYIBIAN_RECAST: u32 = 7;
pub(super) const ASK_HS_MORIJINGXIANG_GAIN: u32 = 8;
pub(super) const ASK_HS_NVWUDEGANGUO_DISCARD: u32 = 9;
pub(super) const ASK_HS_GUHUO_GIVE: u32 = 10;
pub(super) const ASK_HS_NITAI_SKILL: u32 = 11;

pub(super) const ASK_HS_HSHUANYU_DISCOVER: u32 = 20;
pub(super) const ASK_HS_HUALING_TARGET: u32 = 21;
pub(super) const ASK_HS_HUALING_SKILL: u32 = 22;
pub(super) const ASK_HS_OLDHUANJUE_PICK: u32 = 23;
pub(super) const ASK_HS_HSHUANLING_DISCARD: u32 = 24;
pub(super) const ASK_HS_ASYOUZHANG_USE: u32 = 25;
pub(super) const ASK_HS_NSBEIMING_ORDER: u32 = 26;
pub(super) const ASK_HS_HSPUZHAO_CARD: u32 = 27;
pub(super) const ASK_HS_HSTIANQI_CARD: u32 = 28;

pub(super) const ASK_HS_YINDAN_CARD: u32 = 30;
pub(super) const ASK_HS_HLLINGXI_TARGET: u32 = 31;
pub(super) const ASK_HS_HLLINGXI_DISCARD: u32 = 32;
pub(super) const ASK_HS_XIYONG_USE: u32 = 33;
pub(super) const ASK_HS_TANSUO_CARD: u32 = 34;
pub(super) const ASK_HS_TANSUO_DISCOVER: u32 = 35;
pub(super) const ASK_HS_LIEQI_TARGET: u32 = 36;
pub(super) const ASK_HS_LIEQI_GUESS: u32 = 37;
pub(super) const ASK_HS_AZAOWU_KIND: u32 = 38;
pub(super) const ASK_HS_AZAOWU_CARD: u32 = 39;
pub(super) const ASK_HS_AZAOWU_TARGET: u32 = 40;
pub(super) const ASK_HS_ZHUILIE_DISCARD: u32 = 41;

pub(super) const ASK_HS_SZBIANSHEN_CONFIRM: u32 = 50;
pub(super) const ASK_HS_SZBIANSHEN_GENERAL: u32 = 51;
pub(super) const ASK_HS_KEKAO_PICK: u32 = 52;
pub(super) const ASK_HS_JINHUA_SKILL: u32 = 53;
pub(super) const ASK_HS_KQIZHOU_PICK: u32 = 54;
pub(super) const ASK_HS_MENGYE_TARGET: u32 = 55;
pub(super) const ASK_HS_KUIXIN_TARGET: u32 = 56;
pub(super) const ASK_HS_FUHUA_CARD: u32 = 57;
pub(super) const ASK_HS_FUHUA_TARGET: u32 = 58;
pub(super) const ASK_HS_FUHUA_CHOICE: u32 = 59;
pub(super) const ASK_HS_FUHUA2_GIVE: u32 = 60;
pub(super) const ASK_HS_QIANHOU_DISCARD: u32 = 61;
pub(super) const ASK_HS_ZHONGJI_DISCARD: u32 = 62;
pub(super) const ASK_HS_JINZHOU_TARGET: u32 = 63;

pub(super) const ASK_HS_MIDIAN_CARD: u32 = 70;
pub(super) const ASK_HS_XINGLUO_TARGETS: u32 = 71;
pub(super) const ASK_HS_XINGLUO_DISCARD: u32 = 72;
pub(super) const ASK_HS_XINGLUO_PICK: u32 = 73;
pub(super) const ASK_HS_YUSHOU_CARD: u32 = 74;
pub(super) const ASK_HS_AOSHU_CARD: u32 = 75;
pub(super) const ASK_HS_BZHUIJI_CONFIRM: u32 = 76;
pub(super) const ASK_HS_LIANJIN_CARD: u32 = 77;
pub(super) const ASK_HS_YINGXI_CARD: u32 = 78;
pub(super) const ASK_HS_YINGXI_TARGET: u32 = 79;
pub(super) const ASK_HS_PEIYU_TARGET: u32 = 80;
#[allow(dead_code)]
pub(super) const ASK_HS_WZHANYI_CONFIRM: u32 = 81;
pub(super) const ASK_HS_WZHANYI_USE: u32 = 82;
pub(super) const ASK_HS_WZHANYI_TARGET: u32 = 83;

pub(super) const ASK_HS_YUANSU_CONFIRM: u32 = 90;
pub(super) const ASK_HS_NUYAN2_KIND: u32 = 91;
pub(super) const ASK_HS_NUYAN2_TARGET: u32 = 92;
pub(super) const ASK_HS_NUYAN_KIND: u32 = 93;
pub(super) const ASK_HS_NUYAN_CARD: u32 = 94;
pub(super) const ASK_HS_NUYAN_TARGET: u32 = 95;
pub(super) const ASK_HS_ZENGLI_TARGET: u32 = 96;
pub(super) const ASK_HS_MOBAO_CARDS: u32 = 97;
pub(super) const ASK_HS_XIANJI2_DISCARD: u32 = 98;
pub(super) const ASK_HS_TANMI_USE: u32 = 99;
pub(super) const ASK_HS_TANMI_TARGET: u32 = 100;
pub(super) const ASK_HS_XUEREN_CONFIRM: u32 = 101;
pub(super) const ASK_HS_MAOXIAN_SKILL: u32 = 102;
pub(super) const ASK_HS_QIANGHUAX_CARDS: u32 = 103;
pub(super) const ASK_HS_ZHUIZONG_CARDS: u32 = 104;
pub(super) const ASK_HS_ZHUIZONG_PICK: u32 = 105;
pub(super) const ASK_HS_XUNBAO2_DISCARD: u32 = 106;
pub(super) const ASK_HS_CHUANSONGMEN_TARGET: u32 = 107;
pub(super) const ASK_HS_ASYOUZHANG_TARGET: u32 = 108;
pub(super) const ASK_HS_XIYONG_TARGET: u32 = 109;
pub(super) const ASK_HS_YUSHOU_HUOFU_CARD: u32 = 110;
pub(super) const ASK_HS_YUSHOU_HUOFU_TARGET: u32 = 111;
pub(super) const ASK_HS_YUELU_CARD: u32 = 112;

pub(super) fn ask_flag_key(behavior: HostBehavior, stage: HostStage) -> String {
    format!("hs_ask:{}:{}", behavior.tag(), stage.tag())
}

pub(super) fn set_active_ask(
    core: &mut GameStateCore,
    behavior: HostBehavior,
    owner: PlayerId,
    stage: HostStage,
    ask_id: u32,
) {
    set_flag(core, owner, &ask_flag_key(behavior, stage), ask_id);
}

pub(super) fn get_active_ask(
    core: &GameStateCore,
    behavior: HostBehavior,
    owner: PlayerId,
    stage: HostStage,
) -> u32 {
    flag(core, owner, &ask_flag_key(behavior, stage))
}

pub(super) fn wait_hs(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.stage = stage;
    set_active_ask(ctx.core, m.behavior, m.owner, stage, ask_id);
    let exp = build_hearth_expected_ask(
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

fn hand_eq(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand.iter().chain(pl.equips.iter()).copied().collect()
}

fn hej(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand
        .iter()
        .chain(pl.equips.iter())
        .chain(pl.judge.iter())
        .copied()
        .collect()
}

pub(super) use super::u05_hearth_a_tail::{
    collect_player_answers, effective_hearth_card, hearth_card_needs_target,
    hearth_target_choice_max, is_tanmi_card_usable, single_target_options,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn build_hearth_expected_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> ExpectedAsk {
    match ask_id {
        ASK_HS_SHANDIANJIAN_SHOW => {
            let t = target.unwrap_or(owner);
            let opts = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(t, PROMPT_HS_SHANDIANJIAN_SHOW, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_SHANDIANJIAN_DISCARD => {
            let shown = cards.first().copied();
            let suit = shown.map(|c| catalog.effective_card_suit(core, target.unwrap_or(owner), c));
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| suit.is_none_or(|s| catalog.effective_card_suit(core, owner, c) == s))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_SHANDIANJIAN_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_ZHAOMINGDAN_DISCARD => {
            let t = target.unwrap_or(owner);
            let opts = hej(core, t).into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_ZHAOMINGDAN_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_DUNPAIGEDANG_DISCARD => {
            let opts: Vec<ChoiceOption> =
                hand_eq(core, owner).into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_DUNPAIGEDANG_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_CHUANSONGMEN_USE => {
            let opts: Vec<ChoiceOption> = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_CHUANSONGMEN_USE, ChoiceKind::SelectCards, opts, 0, 1, true)
        }
        ASK_HS_CHUANSONGMEN_TARGET => {
            let cid = cards.first().copied().unwrap_or(0);
            let opts = single_target_options(core, catalog, owner, cid);
            let max_n = hearth_target_choice_max(core, catalog, owner, cid, opts.len());
            build_ask(owner, PROMPT_HS_CHUANSONGMEN_USE, ChoiceKind::SelectPlayers, opts, 1, max_n, true)
        }
        ASK_HS_SHIJIESHU_DISCARD => {
            let t = target.unwrap_or(owner);
            let opts = hand_eq(core, t).into_iter().map(ChoiceOption::Card).collect();
            build_ask(t, PROMPT_HS_SHIJIESHU_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_BUWENDINGYIBIAN_RECAST => {
            let t = target.unwrap_or(owner);
            let opts = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(t, PROMPT_HS_BUWENDINGYIBIAN_RECAST, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_MORIJINGXIANG_GAIN => {
            let t = target.unwrap_or(owner);
            let opts = hej(core, t).into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_MORIJINGXIANG_GAIN, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_NVWUDEGANGUO_DISCARD => {
            let t = target.unwrap_or(owner);
            let opts = hand_eq(core, t).into_iter().map(ChoiceOption::Card).collect();
            build_ask(t, PROMPT_HS_NVWUDEGANGUO_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_GUHUO_GIVE => {
            let t = target.unwrap_or(owner);
            let opts = hand_eq(core, t).into_iter().map(ChoiceOption::Card).collect();
            build_ask(t, PROMPT_HS_GUHUO_GIVE, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_NITAI_SKILL => {
            let cnt = flag(core, owner, "hs_choice_count").max(1) as u8;
            let opts: Vec<ChoiceOption> = (0..cnt).map(ChoiceOption::Tag).collect();
            build_ask(owner, PROMPT_HS_HSNITAI_SKILL, ChoiceKind::PickTag, opts, 1, 1, false)
        }
        ASK_HS_HSHUANYU_DISCOVER | ASK_HS_TANSUO_DISCOVER => {
            let prompt = if ask_id == ASK_HS_HSHUANYU_DISCOVER {
                PROMPT_HS_HSHUANYU_DISCOVER
            } else {
                PROMPT_HS_TANSUO_DISCOVER
            };
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, prompt, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_HUALING_TARGET => {
            let opts = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            build_ask(owner, PROMPT_HS_HUALING_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, false)
        }
        ASK_HS_HUALING_SKILL => {
            let cnt = flag(core, owner, "hs_choice_count").max(1) as u8;
            let opts: Vec<ChoiceOption> = (0..cnt).map(ChoiceOption::Tag).collect();
            build_ask(owner, PROMPT_HS_HUALING_SKILL, ChoiceKind::PickTag, opts, 1, 1, true)
        }
        ASK_HS_OLDHUANJUE_PICK => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_OLDHUANJUE_PICK, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_HSHUANLING_DISCARD => {
            let stored_n = flag(core, owner, "hs_hshuanling_cnt") as usize;
            let max_n = core.player(owner).equips.len().max(1).min(stored_n).max(1);
            let opts: Vec<ChoiceOption> =
                hand_eq(core, owner).into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_HSHUANLING_DISCARD, ChoiceKind::SelectCards, opts, 1, max_n, true)
        }
        ASK_HS_ASYOUZHANG_USE => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_ASYOUZHANG_USE, ChoiceKind::SelectCards, opts, 0, 1, true)
        }
        ASK_HS_ASYOUZHANG_TARGET => {
            let cid = cards.first().copied().unwrap_or(0);
            let opts = single_target_options(core, catalog, owner, cid);
            let max_n = hearth_target_choice_max(core, catalog, owner, cid, opts.len());
            build_ask(owner, PROMPT_HS_ASYOUZHANG_USE, ChoiceKind::SelectPlayers, opts, 1, max_n, true)
        }
        ASK_HS_NSBEIMING_ORDER => {
            let opts: Vec<ChoiceOption> = core
                .vm_aux
                .piles
                .get(&(owner, "hs_nsaiqi".to_string()))
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let n = opts.len().max(1);
            build_ask(owner, PROMPT_HS_NSBEIMING_ORDER, ChoiceKind::SelectCards, opts, n, n, false)
        }
        ASK_HS_HSPUZHAO_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_HSPUZHAO_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_HSTIANQI_CARD => {
            let all_4 = (1..=4).all(|_| false);
            let pl = core.player(owner);
            let has_slot = |slot| pl.equips.iter().any(|&e| catalog.card_kind(e).equip_slot() == Some(slot));
            let equips_full = has_slot(crate::catalog::EquipSlot::Weapon)
                && has_slot(crate::catalog::EquipSlot::Armor)
                && has_slot(crate::catalog::EquipSlot::HorsePlus)
                && has_slot(crate::catalog::EquipSlot::HorseMinus);
            let _ = all_4;
            let mut opts: Vec<ChoiceOption> = pl.equips.iter().copied().map(ChoiceOption::Card).collect();
            if !equips_full {
                opts.extend(pl.hand.iter().copied().map(ChoiceOption::Card));
            }
            build_ask(owner, PROMPT_HS_HSTIANQI_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_YINDAN_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == Suit::Spade)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_YINDAN_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_HLLINGXI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    let pl = core.player(p);
                    pl.hp < pl.max_hp as i32
                        && (pl.hand.len() + pl.equips.len()) >= 2
                        && flag(core, p, "hs_hllingxi_used_turn") == 0
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_HLLINGXI_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_HLLINGXI_DISCARD => {
            let t = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = hand_eq(core, t).into_iter().map(ChoiceOption::Card).collect();
            build_ask(t, PROMPT_HS_HLLINGXI_DISCARD, ChoiceKind::SelectCards, opts, 2, 2, false)
        }
        ASK_HS_XIYONG_USE => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_XIYONG_USE, ChoiceKind::SelectCards, opts, 0, 1, true)
        }
        ASK_HS_XIYONG_TARGET => {
            let cid = cards.first().copied().unwrap_or(0);
            let opts = single_target_options(core, catalog, owner, cid);
            let max_n = hearth_target_choice_max(core, catalog, owner, cid, opts.len());
            build_ask(owner, PROMPT_HS_XIYONG_USE, ChoiceKind::SelectPlayers, opts, 1, max_n, true)
        }
        ASK_HS_TANSUO_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner).into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_TANSUO_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_LIEQI_TARGET => {
            let last = flag(core, owner, "hs_lieqi_last");
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty() && u32::from(p) + 1 != last)
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_LIEQI_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_LIEQI_GUESS => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_LIEQI_GUESS, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_AZAOWU_KIND => {
            // Tag: 0=sha, 1=fire_sha, 2=thunder_sha, 3=jiu, 4=tao
            let mut opts = Vec::new();
            if super::u05_hearth::hearth_sha_can_play(core, catalog, owner) {
                opts.push(ChoiceOption::Tag(0));
                opts.push(ChoiceOption::Tag(1));
                opts.push(ChoiceOption::Tag(2));
            }
            if super::u05_hearth::hearth_jiu_can_play(core, catalog, owner) {
                opts.push(ChoiceOption::Tag(3));
            }
            if super::u05_hearth::hearth_tao_can_play(core, catalog, owner) {
                opts.push(ChoiceOption::Tag(4));
            }
            build_ask(owner, PROMPT_HS_AZAOWU_KIND, ChoiceKind::PickTag, opts, 1, 1, true)
        }
        ASK_HS_AZAOWU_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    catalog.card_kind(c).is_basic_kind()
                        && exact_base_name(catalog.cards[c as usize].name) != "du"
                })
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_AZAOWU_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_AZAOWU_TARGET => {
            let kind_tag = flag(core, owner, "hs_azaowu_picked_kind");
            let opts: Vec<ChoiceOption> = if kind_tag == 3 {
                vec![ChoiceOption::Player(owner)]
            } else if kind_tag == 4 {
                core.alive_players()
                    .filter(|&p| p == owner && core.player(p).hp < core.player(p).max_hp as i32)
                    .map(ChoiceOption::Player)
                    .collect()
            } else {
                alive_others(core, owner)
                    .into_iter()
                    .filter(|&p| in_attack_range_hearth(core, catalog, owner, p))
                    .map(ChoiceOption::Player)
                    .collect()
            };
            build_ask(owner, PROMPT_HS_AZAOWU_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_ZHUILIE_DISCARD => {
            let opts: Vec<ChoiceOption> =
                hand_eq(core, owner).into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_ZHUILIE_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_SZBIANSHEN_CONFIRM => {
            build_ask(owner, PROMPT_HS_SZBIANSHEN_CONFIRM, ChoiceKind::Confirm, vec![ChoiceOption::Confirm], 0, 1, true)
        }
        ASK_HS_SZBIANSHEN_GENERAL => {
            let cnt = flag(core, owner, "hs_choice_count").max(1) as u8;
            let opts: Vec<ChoiceOption> = (0..cnt).map(ChoiceOption::Tag).collect();
            build_ask(owner, PROMPT_HS_SZBIANSHEN_GENERAL, ChoiceKind::PickTag, opts, 1, 1, false)
        }
        ASK_HS_KEKAO_PICK => {
            // Delay trick pool: Tag 0=lebu, 1=bingliang, 2=shandian
            let opts = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1), ChoiceOption::Tag(2)];
            build_ask(owner, PROMPT_HS_KEKAO_PICK, ChoiceKind::PickTag, opts, 1, 1, true)
        }
        ASK_HS_JINHUA_SKILL | ASK_HS_MAOXIAN_SKILL => {
            let prompt = if ask_id == ASK_HS_JINHUA_SKILL {
                PROMPT_HS_JINHUA_SKILL
            } else {
                PROMPT_HS_MAOXIAN_SKILL
            };
            let cnt = flag(core, owner, "hs_choice_count").clamp(1, 3) as u8;
            let opts: Vec<ChoiceOption> = (0..cnt).map(ChoiceOption::Tag).collect();
            build_ask(owner, prompt, ChoiceKind::PickTag, opts, 1, 1, ask_id == ASK_HS_MAOXIAN_SKILL)
        }
        ASK_HS_KQIZHOU_PICK => {
            let opts = vec![
                ChoiceOption::Tag(0), // feng
                ChoiceOption::Tag(1), // shui
                ChoiceOption::Tag(2), // huo
                ChoiceOption::Tag(3), // tu
            ];
            build_ask(owner, PROMPT_HS_KQIZHOU_PICK, ChoiceKind::PickTag, opts, 1, 1, true)
        }
        ASK_HS_MENGYE_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_MENGYE_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_KUIXIN_TARGET => {
            let nh = core.player(owner).hand.len();
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    let h = core.player(p).hand.len();
                    h > 0 && h >= nh
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_KUIXIN_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_FUHUA_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_du_card_hearth(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_FUHUA_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_FUHUA_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    !has_skill_hearth(core, catalog, p, "moxie")
                        && !has_skill_hearth(core, catalog, p, "fuhua2")
                        && flag(core, p, "hs_fuhua_immune") == 0
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_FUHUA_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_FUHUA_CHOICE => {
            let t = target.unwrap_or(owner);
            let opts = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)];
            build_ask(t, PROMPT_HS_FUHUA_CHOICE, ChoiceKind::PickTag, opts, 1, 1, false)
        }
        ASK_HS_FUHUA2_GIVE => {
            let opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_FUHUA2_GIVE, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_QIANHOU_DISCARD => {
            let opts = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_QIANHOU_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_ZHONGJI_DISCARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| matches!(catalog.effective_card_suit(core, owner, c), Suit::Spade | Suit::Club))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_ZHONGJI_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_JINZHOU_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    flag(core, p, "hs_jinzhou_sealed") == 0
                        && flag(core, p, "hs_fengyin_xuetu") == 0
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_JINZHOU_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_MIDIAN_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    let k = catalog.card_kind(c);
                    !k.is_basic_kind() && k.equip_slot().is_none()
                })
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_MIDIAN_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_XINGLUO_TARGETS => {
            let nh = core.player(owner).hand.len();
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).hand.len() > nh)
                .map(ChoiceOption::Player)
                .collect();
            let max_n = opts.len().max(1);
            build_ask(owner, PROMPT_HS_XINGLUO_TARGETS, ChoiceKind::SelectPlayers, opts, 1, max_n, true)
        }
        ASK_HS_XINGLUO_DISCARD => {
            let t = target.unwrap_or(owner);
            let opts = core.player(t).hand.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(t, PROMPT_HS_XINGLUO_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_XINGLUO_PICK => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_XINGLUO_PICK, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_YUSHOU_CARD => {
            let opts: Vec<ChoiceOption> =
                hand_eq(core, owner).into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_YUSHOU_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_YUSHOU_HUOFU_CARD | ASK_HS_YUELU_CARD => {
            let prompt = if ask_id == ASK_HS_YUELU_CARD {
                PROMPT_HS_YUELU_CARD
            } else {
                PROMPT_HS_YUSHOU_CARD
            };
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| matches!(catalog.effective_card_suit(core, owner, c), Suit::Spade | Suit::Club))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, prompt, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_YUSHOU_HUOFU_TARGET => {
            let opts = alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect();
            build_ask(owner, PROMPT_HS_NUYAN_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_AOSHU_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| catalog.effective_card_suit(core, owner, c) == Suit::Spade)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_AOSHU_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_BZHUIJI_CONFIRM => {
            build_ask(owner, PROMPT_HS_BZHUIJI_CONFIRM, ChoiceKind::Confirm, vec![ChoiceOption::Confirm], 0, 1, true)
        }
        ASK_HS_LIANJIN_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| exact_base_name(catalog.cards[c as usize].name) != "hsyaoshui")
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_LIANJIN_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_YINGXI_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| matches!(catalog.effective_card_suit(core, owner, c), Suit::Spade | Suit::Club))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_YINGXI_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_YINGXI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| in_attack_range_hearth(core, catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_YINGXI_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_PEIYU_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| {
                    !(1..=8).any(|i| has_any_skill(core, catalog, p, &format!("tuteng{i}")))
                })
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_PEIYU_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_WZHANYI_CONFIRM => {
            build_ask(owner, PROMPT_HS_WZHANYI_CONFIRM, ChoiceKind::Confirm, vec![ChoiceOption::Confirm], 0, 1, true)
        }
        ASK_HS_WZHANYI_USE => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_WZHANYI_USE, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_WZHANYI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| in_attack_range_hearth(core, catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_WZHANYI_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_YUANSU_CONFIRM => {
            build_ask(owner, PROMPT_HS_YUANSU_CONFIRM, ChoiceKind::Confirm, vec![ChoiceOption::Confirm], 0, 1, true)
        }
        ASK_HS_NUYAN2_KIND => {
            // Damage cards pool: 0=sha, 1=juedou, 2=nanman, 3=wanjian, 4=huosha, 5=yuansuhuimie
            let opts = (0..=5u8).map(ChoiceOption::Tag).collect();
            build_ask(owner, PROMPT_HS_NUYAN2_KIND, ChoiceKind::PickTag, opts, 1, 1, true)
        }
        ASK_HS_NUYAN2_TARGET | ASK_HS_NUYAN_TARGET => {
            let prompt = if ask_id == ASK_HS_NUYAN2_TARGET {
                PROMPT_HS_NUYAN2_TARGET
            } else {
                PROMPT_HS_NUYAN_TARGET
            };
            let kind_tag = flag(core, owner, "hs_nuyan_picked_kind");
            let opts: Vec<ChoiceOption> = if kind_tag == 0 || kind_tag == 4 {
                alive_others(core, owner)
                    .into_iter()
                    .filter(|&p| in_attack_range_hearth(core, catalog, owner, p))
                    .map(ChoiceOption::Player)
                    .collect()
            } else {
                alive_others(core, owner).into_iter().map(ChoiceOption::Player).collect()
            };
            build_ask(owner, prompt, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_NUYAN_KIND => {
            let used_mask = flag(core, owner, "hs_nuyan_used_mask");
            let opts: Vec<ChoiceOption> = (0..=5u8)
                .filter(|&tag| (used_mask & (1 << tag)) == 0)
                .map(ChoiceOption::Tag)
                .collect();
            build_ask(owner, PROMPT_HS_NUYAN_KIND, ChoiceKind::PickTag, opts, 1, 1, true)
        }
        ASK_HS_NUYAN_CARD => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| matches!(catalog.effective_card_suit(core, owner, c), Suit::Heart | Suit::Diamond))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_NUYAN_CARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_ZENGLI_TARGET => {
            let opts: Vec<ChoiceOption> = alive_others(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            build_ask(owner, PROMPT_HS_ZENGLI_TARGET, ChoiceKind::SelectPlayers, opts, 1, 1, true)
        }
        ASK_HS_MOBAO_CARDS => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| matches!(catalog.effective_card_suit(core, owner, c), Suit::Spade | Suit::Club))
                .map(ChoiceOption::Card)
                .collect();
            let max_n = opts.len().min(3).max(1);
            build_ask(owner, PROMPT_HS_MOBAO_CARDS, ChoiceKind::SelectCards, opts, 1, max_n, true)
        }
        ASK_HS_XIANJI2_DISCARD => {
            let actor = target.unwrap_or(owner);
            let opts: Vec<ChoiceOption> = core
                .player(actor)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            let max_n = opts.len().min(2).max(1);
            build_ask(actor, PROMPT_HS_XIANJI2_DISCARD, ChoiceKind::SelectCards, opts, 1, max_n, true)
        }
        ASK_HS_TANMI_USE => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_tanmi_card_usable(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_TANMI_USE, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        ASK_HS_TANMI_TARGET => {
            let cid = card.or_else(|| cards.first().copied()).unwrap_or(0);
            let opts = single_target_options(core, catalog, owner, cid);
            let max_n = hearth_target_choice_max(core, catalog, owner, cid, opts.len());
            build_ask(owner, PROMPT_HS_TANMI_TARGET, ChoiceKind::SelectPlayers, opts, 1, max_n, true)
        }
        ASK_HS_XUEREN_CONFIRM => {
            build_ask(owner, PROMPT_HS_XUEREN_CONFIRM, ChoiceKind::Confirm, vec![ChoiceOption::Confirm], 0, 1, true)
        }
        ASK_HS_QIANGHUAX_CARDS => {
            let opts: Vec<ChoiceOption> = hand_eq(core, owner).into_iter().map(ChoiceOption::Card).collect();
            let mut type_bits = 0u8;
            for c in hand_eq(core, owner) {
                type_bits |= 1 << hearth_card_type_tag(catalog, c);
            }
            let max_n = (type_bits.count_ones() as usize).max(1);
            build_ask(owner, PROMPT_HS_QIANGHUAX_CARDS, ChoiceKind::SelectCards, opts, 1, max_n, true)
        }
        ASK_HS_ZHUIZONG_CARDS => {
            let opts: Vec<ChoiceOption> =
                hand_eq(core, owner).into_iter().map(ChoiceOption::Card).collect();
            let max_n = opts.len().max(1);
            build_ask(owner, PROMPT_HS_ZHUIZONG_CARDS, ChoiceKind::SelectCards, opts, 1, max_n, true)
        }
        ASK_HS_ZHUIZONG_PICK => {
            let held = core
                .vm_aux
                .piles
                .get(&(owner, "hs_zhuizong_holding".to_string()))
                .cloned()
                .unwrap_or_default();
            let opts: Vec<ChoiceOption> = held.into_iter().map(ChoiceOption::Card).collect();
            build_ask(owner, PROMPT_HS_ZHUIZONG_PICK, ChoiceKind::SelectCards, opts, 1, 1, false)
        }
        ASK_HS_XUNBAO2_DISCARD => {
            let want_rank = flag(core, owner, "hs_xunbao_rank") as u8;
            let opts: Vec<ChoiceOption> = hand_eq(core, owner)
                .into_iter()
                .filter(|&c| catalog.cards[c as usize].rank == want_rank)
                .map(ChoiceOption::Card)
                .collect();
            build_ask(owner, PROMPT_HS_XUNBAO2_DISCARD, ChoiceKind::SelectCards, opts, 1, 1, true)
        }
        _ => {
            let _ = targets;
            build_ask(owner, PROMPT_HS_SZBIANSHEN_CONFIRM, ChoiceKind::Confirm, vec![ChoiceOption::Confirm], 0, 1, true)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_u05_hearth(
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
    let ask_id = get_active_ask(core, behavior, owner, stage);
    if ask_id == 0 {
        return None;
    }
    Some(build_hearth_expected_ask(
        ask_id, owner, target, targets, card, cards, core, catalog,
    ))
}
