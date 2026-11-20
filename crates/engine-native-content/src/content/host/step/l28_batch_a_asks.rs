//! L28 雷霆万钧 Batch A 询问构造与武将 5–9 步进（`l28_batch_a_asks.rs`）。
//!
//! 覆盖武将 5–9：
//! 5. `th_puyuan`（蒲元）：`th_shengong`、`olqisi`
//! 6. `th_pangdegong`（庞德公）：`th_pingcai`、`xinfu_pdgyingshi`
//! 7. `th_zhengxuan`（郑玄）：`th_zhengjing`、`th_zhengjing2`
//! 8. `th_mamidi`（马日磾）：`th_chengye`、`th_buxu`
//! 9. `th_nanhualaoxian`（南华老仙）：`th_yufeng`、`th_tianshu`、`th_taipingyaoshu`

use super::*;
use crate::catalog::{Catalog, EquipSlot, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

fn spec_ask(
    chooser: PlayerId,
    kind: ChoiceKind,
    prompt: &'static str,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
) -> Option<(PlayerId, ChoiceSpec)> {
    (!options.is_empty() && options.len() >= min).then_some((
        chooser,
        ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel: false,
        },
    ))
}

fn tag_ask(
    chooser: PlayerId,
    prompt: &'static str,
    tags: impl IntoIterator<Item = u8>,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(chooser, ChoiceKind::PickTag, prompt, tags.into_iter().map(ChoiceOption::Tag).collect(), 1, 1)
}

fn player_ask(
    chooser: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(chooser, ChoiceKind::SelectPlayers, prompt, players.into_iter().map(ChoiceOption::Player).collect(), 1, 1)
}

fn card_ask(
    chooser: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
) -> Option<(PlayerId, ChoiceSpec)> {
    spec_ask(chooser, ChoiceKind::SelectCards, prompt, cards.into_iter().map(ChoiceOption::Card).collect(), 1, 1)
}

/// 主动/可选技能的首问：玩家可取消（不发动），取消时退还次数。
const FIRST_CANCELLABLE_ASKS: [u32; 11] = [
    ASK_DUNSHI_BASIC,
    ASK_CAIYI_TARGET,
    ASK_CHONGXU_ACTION,
    ASK_MIAOJIAN_TARGET,
    ASK_SHENGONG_DISCARD,
    ASK_PINGCAI_RELIC,
    ASK_ZHENGJING_COUNT,
    ASK_BUXU_DISCARD,
    ASK_YUFENG_SCORE,
    ASK_TIANSHU_TARGET,
    ASK_TIANSHU_DISCARD,
];

pub(super) fn is_first_cancellable(ask_id: u32) -> bool {
    FIRST_CANCELLABLE_ASKS.contains(&ask_id)
}

fn build_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let mut built = build_ask_spec(ask_id, owner, target, card, cards, core, catalog)?;
    built.1.allow_cancel =
        built.1.allow_cancel || is_first_cancellable(ask_id) || ask_id == ASK_SHENGONG_ASSIST;
    Some(built)
}

fn build_ask_spec(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_DUNSHI_BASIC => {
            let mask = flag(core, owner, "th_dunshi_removed_mask");
            tag_ask(
                owner,
                "th_dunshi:choose_basic",
                (0..4u8).filter(|&i| (mask & (1 << i)) == 0),
            )
        }
        ASK_DUNSHI_SHA_TARGET => player_ask(
            owner,
            "th_dunshi:choose_sha_target",
            alive_others(core, owner),
        ),
        ASK_DUNSHI_RESPOND_SHAN => tag_ask(owner, "th_dunshi:respond_shan", 0..=1),
        ASK_DUNSHI_DAMAGE_CHOICE => {
            let first = flag(core, owner, "th_dunshi_first_opt");
            tag_ask(
                owner,
                "th_dunshi:damage_choice",
                (0..=2u8).filter(|&i| first == 0 || u32::from(i) + 1 != first),
            )
        }
        ASK_DUNSHI_SKILL => tag_ask(owner, "th_dunshi:choose_skill", 0..6),
        ASK_BUSHI_POSTURE => tag_ask(owner, "th_bushi:choose_posture", 0..24),
        ASK_CAIYI_TARGET => player_ask(owner, "th_caiyi:choose_target", alive_all(core, owner)),
        ASK_CAIYI_OPTION => {
            let chooser = target.unwrap_or(owner);
            let rem_key = if flag(core, owner, "th_caiyi_state") > 0 {
                "th_caiyi_yin_removed"
            } else {
                "th_caiyi_yang_removed"
            };
            let rem = flag(core, owner, rem_key) & 0b1111;
            tag_ask(
                chooser,
                "th_caiyi:choose_option",
                (0..4u8).filter(|&i| (rem & (1 << i)) == 0 && (i != 3 || (rem & 0b0111) != 0)),
            )
        }
        ASK_CAIYI_DISCARD => {
            let chooser = target.unwrap_or(owner);
            let he = hand_and_equips(core, chooser);
            let x = flag(core, owner, "th_caiyi_pending_discard").max(1) as usize;
            let cnt = x.min(he.len());
            let opts = he.into_iter().map(ChoiceOption::Card).collect();
            spec_ask(chooser, ChoiceKind::SelectCards, "th_caiyi:discard_cards", opts, cnt, cnt)
        }
        ASK_GUILI_TARGET => {
            player_ask(owner, "th_guili:choose_target", alive_others(core, owner))
        }
        ASK_CHONGXU_ACTION => tag_ask(owner, "th_chongxu:choose_action", 0..=2),
        ASK_MIAOJIAN_TARGET => {
            player_ask(owner, "miaojian:choose_target", alive_others(core, owner))
        }
        ASK_SHENGONG_DISCARD => card_ask(
            owner,
            "th_shengong:discard_card",
            super::l28_batch_a_shengong::shengong_discardable_equips(core, catalog, owner),
        ),
        ASK_SHENGONG_CARD => card_ask(owner, "th_shengong:choose_card", cards.to_vec()),
        ASK_SHENGONG_ASSIST => tag_ask(target.unwrap_or(owner), "th_shengong:assist", 0..=1),
        ASK_SHENGONG_TARGET => player_ask(
            owner,
            "th_shengong:choose_target",
            super::l28_batch_a_shengong::shengong_targets_for(core, catalog, owner, card),
        ),
        ASK_OLQISI_SLOT => tag_ask(owner, "olqisi:choose_slot", 0..4),
        ASK_PINGCAI_RELIC => tag_ask(owner, "th_pingcai:choose_relic", 0..4),
        ASK_PINGCAI_TARGET => {
            player_ask(owner, "th_pingcai:choose_target", alive_all(core, owner))
        }
        ASK_ZHENGJING_COUNT => tag_ask(owner, "th_zhengjing:choose_count", 0..=5),
        ASK_ZHENGJING_TARGET => {
            player_ask(owner, "th_zhengjing:choose_target", alive_all(core, owner))
        }
        ASK_ZHENGJING_CARDS => {
            let opts = cards.iter().copied().map(ChoiceOption::Card).collect();
            spec_ask(owner, ChoiceKind::SelectCards, "th_zhengjing:choose_cards", opts, 1, cards.len())
        }
        ASK_BUXU_DISCARD => {
            let cost = buxu_cost(core, owner);
            let opts = hand_and_equips(core, owner).into_iter().map(ChoiceOption::Card).collect();
            spec_ask(owner, ChoiceKind::SelectCards, "th_buxu:discard_cards", opts, cost, cost)
        }
        ASK_BUXU_CHAMBER => {
            let mask = flag(core, owner, "th_chengye_mask");
            let mut tags: Vec<u8> = (0..6u8).filter(|&i| (mask & (1 << i)) == 0).collect();
            if tags.is_empty() {
                tags.push(0);
            }
            tag_ask(owner, "th_buxu:choose_chamber", tags)
        }
        ASK_YUFENG_SCORE => tag_ask(owner, "th_yufeng:choose_score", 1..=3),
        ASK_YUFENG_TARGET => {
            let score = flag(core, owner, "th_yufeng_flight_score").clamp(1, 3) as usize;
            let others = alive_others(core, owner);
            let max = score.min(others.len()).max(1);
            let opts = others.into_iter().map(ChoiceOption::Player).collect();
            spec_ask(owner, ChoiceKind::SelectPlayers, "th_yufeng:choose_target", opts, 1, max)
        }
        ASK_TIANSHU_DISCARD => {
            card_ask(owner, "th_tianshu:discard_card", hand_and_equips(core, owner))
        }
        ASK_TIANSHU_TARGET => {
            player_ask(owner, "th_tianshu:choose_target", alive_all(core, owner))
        }
        id if id >= 100 => {
            super::l28_shengong_equips::build_equip_ask(id, owner, target, card, cards, core, catalog)
        }
        _ => None,
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l28_ask_id", ask_id);
    if let Some((observer, spec)) =
        build_ask(ask_id, m.owner, m.target, m.card, &m.cards, ctx.core, ctx.catalog)
    {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        Flow::Done
    }
}

pub(in crate::content::host) fn ask_l28_batch_a(
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = flag(core, owner, "l28_ask_id");
    let (observer, spec) = build_ask(ask_id, owner, target, card, cards, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

pub(super) fn step_th_caiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_CAIYI_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_CAIYI_OPTION)
        }
        HostStage::SelectNature => {
            let opt = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => n.min(3),
                _ => 0,
            };
            let t = m.target.unwrap_or(owner);
            let is_yin = flag(ctx.core, owner, "th_caiyi_state") > 0;
            let rem_key = if is_yin {
                "th_caiyi_yin_removed"
            } else {
                "th_caiyi_yang_removed"
            };
            let removed = flag(ctx.core, owner, rem_key) & 0b1111;
            let x = (4 - removed.count_ones()).max(1);
            let eff = if opt == 3 {
                let rem_list: Vec<u8> = (0..3u8).filter(|&i| (removed & (1 << i)) != 0).collect();
                if rem_list.is_empty() {
                    return Flow::Done;
                }
                rem_list[ctx.core.rng.gen_range(rem_list.len() as u32) as usize]
            } else {
                opt
            };
            set_flag(ctx.core, owner, rem_key, removed | (1 << opt));
            if !is_yin {
                set_flag(ctx.core, owner, "th_caiyi_state", 1);
                set_flag(ctx.core, owner, "th_caiyi_yin", 1);
                set_flag(ctx.core, owner, "th_caiyi_yang", 1);
                set_flag(ctx.core, owner, "th_caiyi_yang_used_turn", 1);
                match eff {
                    0 if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 => {
                        ctx.recover(t, x, None);
                    }
                    0 => {}
                    1 => ctx.start_draw(t, x),
                    _ => {
                        if ctx.core.player(t).face_down {
                            ctx.emit(EventData::TurnOverBefore {
                                player: t,
                                cancelled: false,
                            });
                        }
                        ctx.core.player_mut(t).linked = false;
                    }
                }
            } else {
                set_flag(ctx.core, owner, "th_caiyi_state", 0);
                set_flag(ctx.core, owner, "th_caiyi_yin", 0);
                set_flag(ctx.core, owner, "th_caiyi_yang", 0);
                set_flag(ctx.core, owner, "th_caiyi_yin_used_turn", 1);
                match eff {
                    0 if ctx.core.player(t).alive => {
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(owner),
                            t,
                            x as i32,
                        )));
                    }
                    1 if ctx.core.player(t).alive && !hand_and_equips(ctx.core, t).is_empty() => {
                        set_flag(ctx.core, owner, "th_caiyi_pending_discard", x);
                        return wait_ask(m, ctx, HostStage::SelectCard, ASK_CAIYI_DISCARD);
                    }
                    _ if ctx.core.player(t).alive && eff >= 2 => {
                        if !ctx.core.player(t).face_down {
                            ctx.emit(EventData::TurnOverBefore {
                                player: t,
                                cancelled: false,
                            });
                        }
                        ctx.core.player_mut(t).linked = true;
                    }
                    _ => {}
                }
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            let t = m.target.unwrap_or(owner);
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) => Some(c),
                    _ => None,
                })
                .collect();
            set_flag(ctx.core, owner, "th_caiyi_pending_discard", 0);
            set_flag(ctx.core, owner, "l28_ask_id", 0);
            discard_cards(ctx, t, &cards);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_th_guili(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => match m.targets.first().copied() {
            Some(0) => {
                if m.target == Some(owner) && flag(ctx.core, owner, "th_guili_target") == 0 {
                    return wait_ask(m, ctx, HostStage::TargetChoice, ASK_GUILI_TARGET);
                }
                let r = ctx.core.turn.round.max(1);
                let first = u32::from(flag(ctx.core, owner, "th_guili_target_turn_round") != r);
                set_flag(ctx.core, owner, "th_guili_target_turn_round", r);
                set_flag(ctx.core, owner, "th_guili_target_first_turn", first);
                if first > 0 {
                    set_flag(ctx.core, owner, "th_guili_target_dealt_damage", 0);
                }
                Flow::Done
            }
            Some(1) => {
                set_flag(ctx.core, owner, "th_guili_target_dealt_damage", 1);
                Flow::Done
            }
            _ => {
                let r = ctx.core.turn.round.max(1);
                set_flag(ctx.core, owner, "th_guili_acted_round", r);
                set_flag(ctx.core, owner, "th_guili_target_first_turn", 0);
                set_flag(ctx.core, owner, "th_guili_extra_turn", 1);
                set_flag(ctx.core, owner, "extra_turn_insert_queued", 1);
                Flow::Done
            }
        },
        HostStage::TargetChoice => {
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, "th_guili_target", u32::from(t) + 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_th_chongxu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_CHONGXU_ACTION),
        HostStage::SelectNature => {
            let action = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => t,
                _ => 2,
            };
            let mut marks = flag(ctx.core, owner, "th_chongxu_marks") + 3;
            match action {
                0 if marks >= 2 && flag(ctx.core, owner, "th_miaojian_level") < 2 => {
                    marks -= 2;
                    let lv = flag(ctx.core, owner, "th_miaojian_level") + 1;
                    set_flag(ctx.core, owner, "th_miaojian_level", lv);
                    ctx.start_draw(owner, 1);
                }
                1 if marks >= 2 && flag(ctx.core, owner, "th_shhlianhua_level") < 2 => {
                    marks -= 2;
                    let lv = flag(ctx.core, owner, "th_shhlianhua_level") + 1;
                    set_flag(ctx.core, owner, "th_shhlianhua_level", lv);
                    ctx.start_draw(owner, 1);
                }
                _ => {
                    marks = marks.min(5);
                    ctx.start_draw(owner, 2);
                }
            }
            set_flag(ctx.core, owner, "th_chongxu_marks", marks);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_miaojian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_MIAOJIAN_TARGET),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if flag(ctx.core, owner, "th_miaojian_level") == 0 {
                if let Some(&c) = hand_and_equips(ctx.core, owner).first() {
                    discard_cards(ctx, owner, &[c]);
                }
            }
            set_flag(ctx.core, owner, "th_miaojian_pierce", 1);
            ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), t, 1)));
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_olqisi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "th_olqisi_init_done") == 0 && m.target != Some(2) {
                set_flag(ctx.core, owner, "th_olqisi_init_done", 1);
                let mut equipped_slots = Vec::new();
                for c in ctx.core.deck.clone() {
                    if let Some(slot) = ctx.catalog.card_kind(c).equip_slot() {
                        if !equipped_slots.contains(&slot) {
                            equipped_slots.push(slot);
                            equip_card_onto(ctx, owner, c);
                            if equipped_slots.len() >= 2 {
                                break;
                            }
                        }
                    }
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_OLQISI_SLOT)
        }
        HostStage::SelectNature => {
            let slot_tag = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => t,
                _ => 0,
            };
            set_flag(ctx.core, owner, "th_olqisi_converted", 1);
            let want_slot = match slot_tag {
                0 => EquipSlot::Weapon,
                1 => EquipSlot::Armor,
                2 => EquipSlot::HorsePlus,
                _ => EquipSlot::Treasure,
            };
            if let Some(&eq_card) = ctx.core.deck.iter().chain(&ctx.core.discard).find(|&&c| {
                let s = ctx.catalog.card_kind(c).equip_slot();
                s == Some(want_slot) || (slot_tag == 2 && s == Some(EquipSlot::HorseMinus)) || s.is_some()
            }) {
                ctx.core.deck.retain(|&c| c != eq_card);
                ctx.core.discard.retain(|&c| c != eq_card);
                ctx.core.player_mut(owner).hand.push(eq_card);
            } else {
                ctx.start_draw(owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_th_pingcai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_PINGCAI_RELIC),
        HostStage::SelectNature => {
            let relic = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(r)) => u32::from(r) % 4,
                _ => 0,
            };
            set_flag(ctx.core, owner, "th_pingcai_last_relic", relic + 1);
            set_flag(ctx.core, owner, "th_pingcai_relic_wiped", 1);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_PINGCAI_TARGET)
        }
        HostStage::TargetChoice => {
            let t = match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(p)) => p,
                _ => owner,
            };
            match flag(ctx.core, owner, "th_pingcai_last_relic").saturating_sub(1) {
                0 => ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    None,
                ))),
                1 => {
                    ctx.core.player_mut(t).linked = true;
                    ctx.start_draw(owner, 1);
                }
                2 => {
                    ctx.start_draw(t, 1);
                    ctx.start_draw(owner, 1);
                }
                _ => {
                    ctx.start_draw(t, 1);
                    if ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32 {
                        ctx.recover(t, 1, None);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_xinfu_pdgyingshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    set_flag(ctx.core, m.owner, "th_yingshi_hermit", 1);
    set_flag(ctx.core, m.owner, "th_yingshi_skipped_phase", 1);
    Flow::Done
}

pub(super) fn step_th_zhengjing2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(cards) = ctx.core.vm_aux.piles.remove(&(owner, "th_jing".to_string())) {
        ctx.core.player_mut(owner).hand.extend(cards);
    }
    set_flag(ctx.core, owner, "th_zhengjing_boom", 0);
    set_flag(ctx.core, owner, "th_zhengjing2_claimed_skip_judge_draw", 1);
    set_flag(ctx.core, owner, "skip_judge", 1);
    ctx.core.turn.skip_draw_phase = true;
    if let Some(def) = ctx.catalog.skill_by_name("th_zhengjing2") {
        ctx.core.skills.remove(&(owner, def));
    }
    Flow::Done
}

pub(super) fn step_th_chengye(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.target == Some(3) && m.cards.is_empty() {
        claim_all_liujing(ctx, owner);
        return Flow::Done;
    }
    if !m.cards.is_empty() {
        let actor = m.target.unwrap_or(PlayerId::MAX);
        super::l28_batch_a_ext::on_cards_to_discard_chengye(ctx, actor, &m.cards, false);
    }
    Flow::Done
}

fn claim_all_liujing(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let cards = ctx
        .core
        .vm_aux
        .piles
        .remove(&(owner, "th_liujing".to_string()))
        .unwrap_or_default();
    if !cards.is_empty() {
        ctx.core.player_mut(owner).hand.extend(cards);
    }
    set_flag(ctx.core, owner, "th_chengye_mask", 0);
    set_flag(ctx.core, owner, "th_chengye_liujing_count", 0);
    set_flag(ctx.core, owner, "th_chengye_completed", 1);
}

pub(super) fn step_th_yufeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if let Some(&p) = m.targets.first() {
                set_flag(ctx.core, p, "th_yufeng_locked", 0);
                set_flag(ctx.core, owner, "th_yufeng_locked_target", 0);
                let jc = ctx.draw_judge_card();
                let is_black = matches!(ctx.catalog.cards[jc as usize].suit, Suit::Spade | Suit::Club);
                ctx.to_discard(jc);
                if is_black {
                    ctx.core.turn.skip_play_phase = true;
                    ctx.core.player_mut(p).skip_discard_phase = true;
                    set_flag(ctx.core, p, "th_yufeng_skipped_play_discard", 1);
                } else {
                    ctx.core.turn.skip_draw_phase = true;
                    ctx.core.player_mut(p).skip_next_draw = true;
                    set_flag(ctx.core, p, "th_yufeng_skipped_draw", 1);
                }
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectNature, ASK_YUFENG_SCORE)
        }
        HostStage::SelectNature => {
            let score = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(s)) => u32::from(s).clamp(1, 3),
                _ => 2,
            };
            set_flag(ctx.core, owner, "th_yufeng_turn_stamp", turn_stamp(ctx.core));
            set_flag(ctx.core, owner, "th_yufeng_flight_score", score);
            set_flag(ctx.core, owner, "th_yufeng_marks", flag(ctx.core, owner, "th_yufeng_marks") + score);
            ctx.start_draw(owner, score);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_YUFENG_TARGET)
        }
        HostStage::TargetChoice => {
            for o in ctx.answer().clone() {
                if let ChoiceOption::Player(t) = o {
                    set_flag(ctx.core, t, "th_yufeng_locked", 1);
                    set_flag(ctx.core, owner, "th_yufeng_locked_target", u32::from(t) + 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
