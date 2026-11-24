//! L11-E2 `taixu` VM Fix B 交互询问码、选项构造与保存恢复 `ExpectedAsk` 同步表。

use super::ask_l11_vm_fix_b;
use super::super::super::*;
use super::super::l11_common::*;
use super::super::l11_common_b2::{
    ask_txyingyang_mode, ask_txyonglve_confirm, has_txyonglve_ready, in_attack_range,
};
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::rvm_bridge::modifiers::{
    eval_card_distance_unlimited_to, eval_card_target_canceled, eval_card_target_canceled_opt,
    eval_card_use_canceled, eval_sha_unlimited_for_targets, eval_sha_use_limit,
};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(super) const ASK_PINDIAN_SELF_CARD: CardId = 1;
pub(super) const ASK_PINDIAN_TARGET_CARD: CardId = 2;
pub(super) const ASK_TXYONGLVE_OWNER: CardId = 3;
pub(super) const ASK_TXYONGLVE_TARGET: CardId = 4;
pub(super) const ASK_TXYINGYANG_OWNER: CardId = 5;
pub(super) const ASK_TXYINGYANG_TARGET: CardId = 6;

pub(super) const ASK_QUHU_TARGET: CardId = 7;
pub(super) const ASK_QUHU_DAMAGE_TARGET: CardId = 8;
pub(super) const ASK_REXIANZHEN_TARGET: CardId = 9;
pub(super) const ASK_TIANYI_TARGET: CardId = 10;
pub(super) const ASK_REFANJIAN_CARD: CardId = 11;
pub(super) const ASK_REFANJIAN_TARGET: CardId = 12;
pub(super) const ASK_REFANJIAN_MODE: CardId = 13;
pub(super) const ASK_RELUANWU_USE_SHA: CardId = 14;
pub(super) const ASK_RELUANWU_SHA_TARGET: CardId = 15;
pub(super) const ASK_RELUANWU_AFTER_SHA: CardId = 16;
pub(super) const ASK_TIAOXIN_TARGET: CardId = 17;
pub(super) const ASK_TIAOXIN_USE_SHA: CardId = 18;
pub(super) const ASK_TIAOXIN_DISCARD: CardId = 19;

pub(super) const ASK_QINGXI_DISCARD: CardId = 20;
pub(super) const ASK_REBEIGE_DISCARD: CardId = 21;
pub(super) const ASK_REBEIGE_SOURCE_DISCARD: CardId = 22;
pub(super) const ASK_REJIEMING_TARGET: CardId = 23;
pub(super) const ASK_RELEIJI_TARGET: CardId = 24;
pub(super) const ASK_RESHEJIAN_DISCARD: CardId = 25;
pub(super) const ASK_RESHEJIAN_MODE: CardId = 26;
pub(super) const ASK_RESHEJIAN_PICK: CardId = 27;
pub(super) const ASK_SPHUANGEN_TARGETS: CardId = 28;
pub(super) const ASK_ZHUANDUI_CONFIRM: CardId = 29;

pub(super) const ASK_QIANGZHI_TARGET: CardId = 30;
pub(super) const ASK_QIANGZHI_CARD: CardId = 31;
pub(super) const ASK_REJIANGCHI_MODE: CardId = 32;
pub(super) const ASK_REJIANGCHI_DISCARD: CardId = 33;
pub(super) const ASK_REPOLU_TARGETS: CardId = 34;
pub(super) const ASK_REZHIYI_KIND: CardId = 35;
pub(super) const ASK_REZHIYI_TARGET: CardId = 36;
pub(super) const ASK_TWSHEGXI_ZHUNBEI: CardId = 37;
pub(super) const ASK_TWSHEGXI_ZHINANG: CardId = 38;
pub(super) const ASK_XINFU_QINGUO_RECOVER: CardId = 39;
pub(super) const ASK_XINFU_QINGUO_SHA: CardId = 40;
pub(super) const ASK_YINGHUN_TARGET: CardId = 41;
pub(super) const ASK_YINGHUN_MODE: CardId = 42;
pub(super) const ASK_YINGHUN_DISCARD: CardId = 43;
pub(super) const ASK_ZHENGJIAN_TARGET: CardId = 44;
pub(super) const ASK_ZHIYI_MODE: CardId = 45;
pub(super) const ASK_ZHIYI_TARGET: CardId = 46;
pub(super) const ASK_TIAOJIYANMEI_DISCARD: CardId = 47;
pub(super) const ASK_TIAOJIYANMEI_GAIN_TARGET: CardId = 48;
pub(super) const ASK_SHENFEN_DISCARD: CardId = 49;

/// 内部继续态标识（存 `m.card`，不暴露 Ask，数值 < 108 以兼容快照卡牌 ID 边界校验）。
pub(super) const STAGE_RELEIJI_RECOVER: CardId = 90;
pub(super) const STAGE_REBEIGE_SETTLE: CardId = 91;
pub(super) const STAGE_SPCHIJIE_SETTLE: CardId = 92;
pub(super) const STAGE_ZHUANDUI_PINDIAN: CardId = 93;

pub(super) fn wait_vm_b_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    code: CardId,
) -> Flow {
    m.stage = stage;
    m.card = Some(code);
    let Some(expected) = ask_l11_vm_fix_b(
        stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) else {
        m.stage = HostStage::Act;
        m.card = None;
        return Flow::Done;
    };
    Flow::WaitChoice {
        observer: expected.observer,
        spec: ChoiceSpec {
            kind: expected.kind,
            prompt: expected.prompt,
            options: expected.options,
            min: expected.min,
            max: expected.max,
            allow_cancel: expected.allow_cancel,
        },
    }
}

pub(super) fn take_preset_target(core: &mut GameStateCore, owner: PlayerId) -> Option<PlayerId> {
    let raw = core.vm_aux.selected_targets(owner);
    core.vm_aux.set_selected_targets(owner, &[]);
    raw.into_iter()
        .find_map(|s| s.checked_sub(1).and_then(|v| PlayerId::try_from(v).ok()))
}

pub(super) fn take_preset_card(core: &mut GameStateCore, owner: PlayerId) -> Option<CardId> {
    let cs = core.vm_aux.selected_cards(owner);
    core.vm_aux.set_selected_cards(owner, &[]);
    cs.into_iter().next()
}

pub(super) fn reveal_to_all(ctx: &mut Ctx<'_>, cards: Vec<CardId>) {
    if cards.is_empty() {
        return;
    }
    let to = (0..ctx.core.players.len() as PlayerId).collect();
    ctx.out.push(LogEvent::CardsRevealed { cards, to });
}

pub(super) fn decode_hidden_or_card(
    core: &GameStateCore,
    target: PlayerId,
    opt: ChoiceOption,
) -> Option<CardId> {
    match opt {
        ChoiceOption::Card(c) => Some(c),
        ChoiceOption::HiddenCard(i) => core.player(target).hand.get(i as usize).copied(),
        _ => None,
    }
}

pub(super) fn decode_hidden_or_cards(
    core: &GameStateCore,
    target: PlayerId,
    ans: &[ChoiceOption],
) -> Vec<CardId> {
    let mut out = Vec::with_capacity(ans.len());
    for &o in ans {
        if let Some(c) = decode_hidden_or_card(core, target, o) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

pub(super) fn nearest_alive_others(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
) -> Vec<PlayerId> {
    let others = alive_others(core, actor);
    let Some(min_d) = others
        .iter()
        .map(|&p| core.computed_distance(catalog, actor, p))
        .min()
    else {
        return Vec::new();
    };
    others
        .into_iter()
        .filter(|&p| core.computed_distance(catalog, actor, p) == min_d)
        .collect()
}

pub(super) fn valid_reluanwu_sha_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
) -> Vec<CardId> {
    let nearest = nearest_alive_others(core, catalog, actor);
    core.player(actor)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let kind = catalog.resolved_card_kind(core, c);
            let nat = match kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            kind.is_sha_like()
                && !eval_card_use_canceled(core, catalog, actor, c, kind, nat)
                && nearest
                    .iter()
                    .any(|&t| !eval_card_target_canceled(core, catalog, actor, t, c, kind, nat))
        })
        .collect()
}

pub(super) fn valid_tiaoxin_sha_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    victim: PlayerId,
) -> Vec<CardId> {
    core.player(actor)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let kind = catalog.resolved_card_kind(core, c);
            let nat = match kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            kind.is_sha_like()
                && !eval_card_use_canceled(core, catalog, actor, c, kind, nat)
                && !eval_card_target_canceled(core, catalog, actor, victim, c, kind, nat)
        })
        .collect()
}

pub(super) fn tag_to_basic_kind(tag: u8) -> Option<(CardKind, DamageNature)> {
    match tag {
        0 => Some((CardKind::Sha, DamageNature::Normal)),
        1 => Some((CardKind::FireSha, DamageNature::Fire)),
        2 => Some((CardKind::LeiSha, DamageNature::Thunder)),
        3 => Some((CardKind::Tao, DamageNature::Normal)),
        4 => Some((CardKind::Jiu, DamageNature::Normal)),
        _ => None,
    }
}

pub(super) fn basic_kind_to_tag(kind: CardKind) -> Option<u8> {
    match kind {
        CardKind::Sha => Some(0),
        CardKind::FireSha => Some(1),
        CardKind::LeiSha => Some(2),
        CardKind::Tao => Some(3),
        CardKind::Jiu => Some(4),
        CardKind::Shan => Some(5),
        _ => None,
    }
}

pub(super) fn has_sha_count_remaining(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
    nature: DamageNature,
    target: PlayerId,
) -> bool {
    if core.turn.phase != Phase::Play || core.turn.current != actor {
        return true;
    }
    if has_named_skill(core, catalog, actor, "paoxiao")
        || has_named_skill(core, catalog, actor, "咆哮")
        || core.has_active_equip_kind(catalog, actor, CardKind::Zhuge)
        || eval_sha_unlimited_for_targets(core, catalog, actor, &[target])
    {
        return true;
    }
    let (limit, unlimited) = eval_sha_use_limit(core, catalog, actor, None, kind, nature, 1);
    if unlimited {
        return true;
    }
    let extra = flag(core, actor, "vm_sha_extra_used") as i32;
    let used = if core.turn.sha_used_play_phase {
        1 + extra
    } else {
        0
    };
    used < limit as i32
}

pub(super) fn valid_virtual_sha_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
    nature: DamageNature,
    check_distance: bool,
    check_count: bool,
) -> Vec<PlayerId> {
    if eval_card_use_canceled(core, catalog, actor, CardId::MAX, kind, nature) {
        return Vec::new();
    }
    alive_others(core, actor)
        .into_iter()
        .filter(|&t| {
            !eval_card_target_canceled_opt(core, catalog, actor, t, None, kind, nature)
                && (!check_distance
                    || eval_card_distance_unlimited_to(
                        core,
                        catalog,
                        actor,
                        Some(t),
                        CardId::MAX,
                        kind,
                        nature,
                    )
                    || in_attack_range(core, catalog, actor, t))
                && (!check_count || has_sha_count_remaining(core, catalog, actor, kind, nature, t))
        })
        .collect()
}

pub(super) fn can_use_virtual_basic(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
    nature: DamageNature,
    check_distance: bool,
    check_count: bool,
) -> bool {
    if eval_card_use_canceled(core, catalog, actor, CardId::MAX, kind, nature) {
        return false;
    }
    match kind {
        CardKind::Tao => {
            core.player(actor).alive
                && core.player(actor).hp < core.player(actor).max_hp as i32
                && !eval_card_target_canceled_opt(
                    core,
                    catalog,
                    actor,
                    actor,
                    None,
                    CardKind::Tao,
                    DamageNature::Normal,
                )
        }
        CardKind::Jiu => {
            core.player(actor).alive
                && !core.turn.jiu_used
                && !core.turn.jiu_active
                && !eval_card_target_canceled_opt(
                    core,
                    catalog,
                    actor,
                    actor,
                    None,
                    CardKind::Jiu,
                    DamageNature::Normal,
                )
        }
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
            !valid_virtual_sha_targets(core, catalog, actor, kind, nature, check_distance, check_count)
                .is_empty()
        }
        _ => false,
    }
}

pub(super) fn weapon_range_sum(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> usize {
    core.player(owner)
        .equips
        .iter()
        .copied()
        .filter(|&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Weapon))
        .map(|c| {
            let Some(def) = catalog.resolved_card_def(core, c) else {
                return 1;
            };
            if let Some(r) = crate::rvm_bridge::modifiers::d34_l28_mods::rewrite_zhuge_range(core, catalog, c) {
                r as usize
            } else if def.kind == CardKind::Weapon {
                def.range as usize
            } else {
                def.kind.named_weapon_range().unwrap_or(1) as usize
            }
        })
        .sum()
}

pub(super) fn ask_l11_vm_fix_b_inner(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle {
        return None;
    }
    let code = card?;
    let p_opts = |ps: Vec<PlayerId>| ps.into_iter().map(ChoiceOption::Player).collect();
    let c_opts = |cs: Vec<CardId>| cs.into_iter().map(ChoiceOption::Card).collect();
    match code {
        ASK_PINDIAN_SELF_CARD => Some(ask(
            owner,
            "l11_vm_b:pindian_card",
            ChoiceKind::SelectCards,
            c_opts(core.player(owner).hand.clone()),
            1,
            1,
            false,
        )),
        ASK_PINDIAN_TARGET_CARD => {
            let t = target?;
            Some(ask(
                t,
                "l11_vm_b:target_pindian_card",
                ChoiceKind::SelectCards,
                c_opts(core.player(t).hand.clone()),
                1,
                1,
                false,
            ))
        }
        ASK_TXYONGLVE_OWNER => Some(ask_txyonglve_confirm(owner)),
        ASK_TXYONGLVE_TARGET => Some(ask_txyonglve_confirm(target?)),
        ASK_TXYINGYANG_OWNER => Some(ask_txyingyang_mode(owner)),
        ASK_TXYINGYANG_TARGET => Some(ask_txyingyang_mode(target?)),
        ASK_QUHU_TARGET => {
            let o_hp = core.player(owner).hp;
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    core.player(p).hp > o_hp
                        && (!core.player(p).hand.is_empty()
                            || has_txyonglve_ready(core, catalog, p))
                })
                .collect();
            Some(ask(
                owner,
                "l11_vm_b:quhu_target",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                false,
            ))
        }
        ASK_QUHU_DAMAGE_TARGET => {
            let t = target?;
            let opts = alive_others(core, t)
                .into_iter()
                .filter(|&p| in_attack_range(core, catalog, t, p))
                .collect();
            Some(ask(
                owner,
                "l11_vm_b:quhu_damage_target",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                false,
            ))
        }
        ASK_REXIANZHEN_TARGET | ASK_TIANYI_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| {
                    !core.player(p).hand.is_empty() || has_txyonglve_ready(core, catalog, p)
                })
                .collect();
            let prompt = if code == ASK_REXIANZHEN_TARGET {
                "l11_vm_b:rexianzhen_target"
            } else {
                "l11_vm_b:tianyi_target"
            };
            Some(ask(
                owner,
                prompt,
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                false,
            ))
        }
        ASK_REFANJIAN_CARD => Some(ask(
            owner,
            "l11_vm_b:refanjian_card",
            ChoiceKind::SelectCards,
            c_opts(core.player(owner).hand.clone()),
            1,
            1,
            false,
        )),
        ASK_REFANJIAN_TARGET => Some(ask(
            owner,
            "l11_vm_b:refanjian_target",
            ChoiceKind::SelectPlayers,
            p_opts(alive_others(core, owner)),
            1,
            1,
            false,
        )),
        ASK_REFANJIAN_MODE => Some(ask(
            target?,
            "l11_vm_b:refanjian_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        ASK_RELUANWU_USE_SHA => {
            let t = target?;
            Some(ask(
                t,
                "l11_vm_b:reluanwu_use_sha",
                ChoiceKind::SelectCards,
                c_opts(valid_reluanwu_sha_cards(core, catalog, t)),
                1,
                1,
                true,
            ))
        }
        ASK_RELUANWU_SHA_TARGET => {
            let t = target?;
            let cid = cards.get(1).copied()?;
            let kind = catalog.resolved_card_kind(core, cid);
            let nat = match kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            let opts = nearest_alive_others(core, catalog, t)
                .into_iter()
                .filter(|&q| !eval_card_target_canceled(core, catalog, t, q, cid, kind, nat))
                .collect();
            Some(ask(
                t,
                "l11_vm_b:reluanwu_sha_target",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                false,
            ))
        }
        ASK_RELUANWU_AFTER_SHA => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&q| {
                    !eval_card_target_canceled_opt(
                        core,
                        catalog,
                        owner,
                        q,
                        None,
                        CardKind::Sha,
                        DamageNature::Normal,
                    )
                })
                .collect();
            Some(ask(
                owner,
                "l11_vm_b:reluanwu_after_sha",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                true,
            ))
        }
        ASK_TIAOXIN_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| in_attack_range(core, catalog, p, owner) && !he_cards(core, p).is_empty())
                .collect();
            Some(ask(
                owner,
                "l11_vm_b:tiaoxin_target",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                false,
            ))
        }
        ASK_TIAOXIN_USE_SHA => {
            let t = target?;
            Some(ask(
                t,
                "l11_vm_b:tiaoxin_use_sha",
                ChoiceKind::SelectCards,
                c_opts(valid_tiaoxin_sha_cards(core, catalog, t, owner)),
                1,
                1,
                true,
            ))
        }
        ASK_TIAOXIN_DISCARD => {
            let t = target?;
            let pl = core.player(t);
            Some(ask(
                owner,
                "l11_vm_b:tiaoxin_discard",
                ChoiceKind::SelectCards,
                crate::event::qinglong_options(owner, t, &pl.hand, &pl.equips),
                1,
                1,
                false,
            ))
        }
        ASK_QINGXI_DISCARD => {
            let t = target?;
            let n = weapon_range_sum(core, catalog, owner).max(1);
            Some(ask(
                t,
                "l11_vm_b:qingxi_discard",
                ChoiceKind::SelectCards,
                c_opts(core.player(t).hand.clone()),
                n,
                n,
                true,
            ))
        }
        ASK_REBEIGE_DISCARD => Some(ask(
            owner,
            "l11_vm_b:rebeige_discard",
            ChoiceKind::SelectCards,
            c_opts(he_cards(core, owner)),
            1,
            1,
            true,
        )),
        ASK_REBEIGE_SOURCE_DISCARD => {
            let src = *targets.first()?;
            let he = he_cards(core, src);
            let need = he.len().min(2);
            Some(ask(
                src,
                "l11_vm_b:rebeige_source_discard",
                ChoiceKind::SelectCards,
                c_opts(he),
                need,
                need,
                false,
            ))
        }
        ASK_REJIEMING_TARGET => Some(ask(
            owner,
            "l11_vm_b:rejieming_target",
            ChoiceKind::SelectPlayers,
            p_opts(alive_all(core, owner)),
            1,
            1,
            true,
        )),
        ASK_RELEIJI_TARGET => Some(ask(
            owner,
            "l11_vm_b:releiji_target",
            ChoiceKind::SelectPlayers,
            p_opts(alive_others(core, owner)),
            1,
            1,
            true,
        )),
        ASK_RESHEJIAN_DISCARD => {
            let he = he_cards(core, owner);
            Some(ask(
                owner,
                "l11_vm_b:reshejian_discard",
                ChoiceKind::SelectCards,
                c_opts(he.clone()),
                2,
                he.len().max(2),
                true,
            ))
        }
        ASK_RESHEJIAN_MODE => Some(ask(
            owner,
            "l11_vm_b:reshejian_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        ASK_RESHEJIAN_PICK => {
            let t = target?;
            let pl = core.player(t);
            let want = cards.first().copied().unwrap_or(1) as usize;
            let k = want.min(pl.hand.len() + pl.equips.len()).max(1);
            Some(ask(
                owner,
                "l11_vm_b:reshejian_pick",
                ChoiceKind::SelectCards,
                crate::event::qinglong_options(owner, t, &pl.hand, &pl.equips),
                k,
                k,
                false,
            ))
        }
        ASK_SPHUANGEN_TARGETS => {
            let opts: Vec<PlayerId> = targets.iter().copied().filter(|&p| core.player(p).alive).collect();
            let max_pick = (core.player(owner).hp.max(1) as usize).min(opts.len()).max(1);
            Some(ask(
                owner,
                "l11_vm_b:sphuangen_targets",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                max_pick,
                true,
            ))
        }
        ASK_ZHUANDUI_CONFIRM => Some(ask(
            owner,
            "l11_vm_b:zhuandui_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        ASK_QIANGZHI_TARGET => {
            let opts = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !core.player(p).hand.is_empty())
                .collect();
            Some(ask(
                owner,
                "l11_vm_b:qiangzhi_target",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                true,
            ))
        }
        ASK_QIANGZHI_CARD => {
            let t = target?;
            let pl = core.player(t);
            Some(ask(
                owner,
                "l11_vm_b:qiangzhi_card",
                ChoiceKind::SelectCards,
                crate::event::qinglong_options(owner, t, &pl.hand, &[]),
                1,
                1,
                false,
            ))
        }
        ASK_REJIANGCHI_MODE => {
            let mut tags = Vec::new();
            if !he_cards(core, owner).is_empty() {
                tags.push(ChoiceOption::Tag(0));
            }
            tags.push(ChoiceOption::Tag(1));
            Some(ask(
                owner,
                "l11_vm_b:rejiangchi_mode",
                ChoiceKind::PickTag,
                tags,
                1,
                1,
                true,
            ))
        }
        ASK_REJIANGCHI_DISCARD => Some(ask(
            owner,
            "l11_vm_b:rejiangchi_discard",
            ChoiceKind::SelectCards,
            c_opts(he_cards(core, owner)),
            1,
            1,
            true,
        )),
        ASK_REPOLU_TARGETS => {
            let opts = if cards.first().copied() == Some(1) {
                alive_others(core, owner)
            } else {
                alive_all(core, owner)
            };
            let max_pick = opts.len().max(1);
            Some(ask(
                owner,
                "l11_vm_b:repolu_targets",
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                max_pick,
                true,
            ))
        }
        ASK_REZHIYI_KIND => {
            let mask = cards.first().copied().unwrap_or(0);
            let mut tags = Vec::new();
            for t in 0..=4u8 {
                if (mask & (1u32 << t)) != 0 {
                    if let Some((k, nat)) = tag_to_basic_kind(t) {
                        if can_use_virtual_basic(core, catalog, owner, k, nat, true, true) {
                            tags.push(ChoiceOption::Tag(t));
                        }
                    }
                }
            }
            Some(ask(
                owner,
                "l11_vm_b:rezhiyi_kind",
                ChoiceKind::PickTag,
                tags,
                1,
                1,
                true,
            ))
        }
        ASK_REZHIYI_TARGET | ASK_ZHIYI_TARGET | ASK_XINFU_QINGUO_SHA => {
            let tag = cards.get(if code == ASK_REZHIYI_TARGET { 1 } else { 0 }).copied().unwrap_or(0) as u8;
            let (k, nat) = tag_to_basic_kind(tag).unwrap_or((CardKind::Sha, DamageNature::Normal));
            let opts = valid_virtual_sha_targets(core, catalog, owner, k, nat, true, false);
            let prompt = match code {
                ASK_REZHIYI_TARGET => "l11_vm_b:rezhiyi_target",
                ASK_ZHIYI_TARGET => "l11_vm_b:zhiyi_target",
                _ => "l11_vm_b:xinfu_qinguo_sha",
            };
            Some(ask(
                owner,
                prompt,
                ChoiceKind::SelectPlayers,
                p_opts(opts),
                1,
                1,
                true,
            ))
        }
        ASK_TWSHEGXI_ZHUNBEI => Some(ask(
            owner,
            "l11_vm_b:twshengxi_zhunbei",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        ASK_TWSHEGXI_ZHINANG => {
            let mask = cards.first().copied().unwrap_or(0);
            let mut tags = Vec::new();
            for t in 0..3u8 {
                if (mask & (1u32 << t)) != 0 {
                    tags.push(ChoiceOption::Tag(t));
                }
            }
            Some(ask(
                owner,
                "l11_vm_b:twshengxi_zhinang",
                ChoiceKind::PickTag,
                tags,
                1,
                1,
                true,
            ))
        }
        ASK_XINFU_QINGUO_RECOVER => Some(ask(
            owner,
            "l11_vm_b:xinfu_qinguo_recover",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        ASK_YINGHUN_TARGET => Some(ask(
            owner,
            "l11_vm_b:yinghun_target",
            ChoiceKind::SelectPlayers,
            p_opts(alive_others(core, owner)),
            1,
            1,
            true,
        )),
        ASK_YINGHUN_MODE => Some(ask(
            owner,
            "l11_vm_b:yinghun_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        ASK_YINGHUN_DISCARD => {
            let t = target?;
            let he = he_cards(core, t);
            let want = cards.first().copied().unwrap_or(1) as usize;
            let k = want.min(he.len()).max(1);
            Some(ask(
                t,
                "l11_vm_b:yinghun_discard",
                ChoiceKind::SelectCards,
                c_opts(he),
                k,
                k,
                false,
            ))
        }
        ASK_ZHENGJIAN_TARGET => Some(ask(
            owner,
            "l11_vm_b:zhengjian_target",
            ChoiceKind::SelectPlayers,
            p_opts(alive_all(core, owner)),
            1,
            1,
            true,
        )),
        ASK_ZHIYI_MODE => Some(ask(
            owner,
            "l11_vm_b:zhiyi_mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        ASK_TIAOJIYANMEI_DISCARD => {
            let t = target?;
            Some(ask(
                t,
                "l11_vm_b:tiaojiyanmei_discard",
                ChoiceKind::SelectCards,
                c_opts(he_cards(core, t)),
                1,
                1,
                false,
            ))
        }
        ASK_TIAOJIYANMEI_GAIN_TARGET => Some(ask(
            owner,
            "l11_vm_b:tiaojiyanmei_gain_target",
            ChoiceKind::SelectPlayers,
            p_opts(alive_all(core, owner)),
            1,
            1,
            true,
        )),
        ASK_SHENFEN_DISCARD => {
            let t = target?;
            let he = he_cards(core, t);
            let need = he.len().min(4);
            Some(ask(
                t,
                "l11_vm_b:shenfen_discard",
                ChoiceKind::SelectCards,
                c_opts(he),
                need,
                need,
                false,
            ))
        }
        _ => None,
    }
}
