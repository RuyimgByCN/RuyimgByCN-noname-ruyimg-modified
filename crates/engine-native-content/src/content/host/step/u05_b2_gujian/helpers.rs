use super::super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data, Ctx, Flow, LogEvent};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(super) fn answer_first_or_pass(ctx: &Ctx<'_>) -> ChoiceOption {
    ctx.answer().first().copied().unwrap_or(ChoiceOption::Pass)
}

pub(super) fn card_use_virtual_with_nature(
    player: PlayerId,
    targets: Vec<PlayerId>,
    nature: DamageNature,
    kind: CardKind,
) -> CardUseData {
    CardUseData {
        card: None,
        player,
        targets,
        nature,
        use_as: Some(kind),
        cancelled: false,
    }
}

pub(super) fn general_id_by_name(catalog: &Catalog, name: &str) -> Option<u16> {
    if let Some(i) = catalog.standard_generals.iter().position(|g| g.name == name) {
        return u16::try_from(i).ok();
    }
    let base = catalog.standard_generals.len();
    if let Some(i) = catalog.extension_generals.iter().position(|g| g.name == name) {
        return u16::try_from(base + i).ok();
    }
    let boss_base = base + catalog.extension_generals.len();
    catalog
        .boss_generals
        .iter()
        .position(|g| g.name == name)
        .and_then(|i| u16::try_from(boss_base + i).ok())
}

pub(super) fn is_jiguan_kind(k: CardKind) -> bool {
    matches!(
        k,
        CardKind::Jiguan
            | CardKind::Bingpotong
            | CardKind::Feibiao
            | CardKind::Longxugou
            | CardKind::Qiankunbiao
            | CardKind::Shenhuofeiya
            | CardKind::Mianlijinzhen
            | CardKind::Liutouge
    )
}

pub(super) const ASK_YANJIADAN_VCARD: u32 = 1;
pub(super) const ASK_YANJIADAN_TARGET: u32 = 2;
pub(super) const ASK_G_SHIHUIFEN_USE: u32 = 3;
pub(super) const ASK_YUHENG_COST: u32 = 4;

pub(super) const LAND_KINDS: [CardKind; 10] = [
    CardKind::Longlushanfeng,
    CardKind::Haidao,
    CardKind::Shuimomuxue,
    CardKind::Feixu,
    CardKind::Yixialan,
    CardKind::Shamolvzhou,
    CardKind::Linzhongjianta,
    CardKind::Bingheyaosai,
    CardKind::Youlin,
    CardKind::Lindixiliu,
];

pub(super) fn flag(core: &GameStateCore, p: PlayerId, k: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, k.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, k: &str, v: u32) {
    if v == 0 {
        core.vm_aux.flags.remove(&(p, k.to_string()));
    } else {
        core.vm_aux.flags.insert((p, k.to_string()), v);
    }
}

pub(super) fn has_skill(core: &GameStateCore, catalog: &Catalog, p: PlayerId, k: &str) -> bool {
    catalog
        .skill_by_name(k)
        .is_some_and(|d| core.has_skill(p, d))
}

pub(super) fn ask_key(beh: HostBehavior) -> String {
    format!("u05_b2_ask:{}", beh.tag())
}

pub(super) fn obs_key(beh: HostBehavior) -> String {
    format!("u05_b2_obs:{}", beh.tag())
}

pub(super) fn wait_ask(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    observer: PlayerId,
    ask_id: u32,
    spec: ChoiceSpec,
) -> Flow {
    machine.stage = stage;
    set_flag(ctx.core, machine.owner, &ask_key(machine.behavior), ask_id);
    set_flag(
        ctx.core,
        machine.owner,
        &obs_key(machine.behavior),
        u32::from(observer),
    );
    Flow::WaitChoice { observer, spec }
}

pub(super) fn current_ask_id(core: &GameStateCore, owner: PlayerId, beh: HostBehavior) -> u32 {
    flag(core, owner, &ask_key(beh))
}

pub(super) fn current_ask_obs(core: &GameStateCore, owner: PlayerId, beh: HostBehavior) -> PlayerId {
    flag(core, owner, &obs_key(beh)) as PlayerId
}

pub(super) fn is_red(catalog: &Catalog, card: CardId) -> bool {
    matches!(catalog.cards[card as usize].suit, Suit::Heart | Suit::Diamond)
}

pub(super) fn is_black(catalog: &Catalog, card: CardId) -> bool {
    matches!(catalog.cards[card as usize].suit, Suit::Spade | Suit::Club)
}

pub(super) fn hand_and_equips(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let mut out = core.player(p).hand.clone();
    out.extend(core.player(p).equips.iter().copied());
    out
}

pub(super) fn discard_card_from_he(ctx: &mut Ctx<'_>, p: PlayerId, c: CardId) -> bool {
    if ctx.core.player(p).hand.contains(&c) {
        ctx.discard_from_hand(p, c);
        ctx.log(LogEvent::CardDiscarded {
            player: p,
            cards: vec![c],
        });
        true
    } else if ctx.core.player(p).equips.contains(&c) {
        ctx.core.player_mut(p).equips.retain(|x| *x != c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, c);
        ctx.to_discard(c);
        ctx.log(LogEvent::CardDiscarded {
            player: p,
            cards: vec![c],
        });
        true
    } else {
        false
    }
}

pub(super) fn discard_cards_from_he(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut logged = Vec::new();
    for &c in cards {
        if ctx.core.player(p).hand.contains(&c) {
            ctx.core.player_mut(p).hand.retain(|x| *x != c);
            ctx.to_discard(c);
            logged.push(c);
        } else if ctx.core.player(p).equips.contains(&c) {
            ctx.core.player_mut(p).equips.retain(|x| *x != c);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, c);
            ctx.to_discard(c);
            logged.push(c);
        }
    }
    if !logged.is_empty() {
        ctx.log(LogEvent::CardDiscarded {
            player: p,
            cards: logged,
        });
    }
}

pub(super) fn give_card(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId, c: CardId) -> bool {
    let moved = if ctx.core.player(from).hand.contains(&c) {
        ctx.core.player_mut(from).hand.retain(|x| *x != c);
        true
    } else if ctx.core.player(from).equips.contains(&c) {
        ctx.core.player_mut(from).equips.retain(|x| *x != c);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, c);
        true
    } else if ctx.core.player(from).judge.contains(&c) {
        ctx.core.player_mut(from).judge.retain(|x| *x != c);
        true
    } else {
        false
    };
    if moved {
        ctx.core.player_mut(to).hand.push(c);
        ctx.log(LogEvent::CardsGiven {
            from,
            to,
            cards: vec![c],
        });
    }
    moved
}

pub(super) fn can_target_card(
    core: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
    kind: CardKind,
) -> bool {
    if !core.player(to).alive {
        return false;
    }
    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core,
        catalog,
        from,
        to,
        0,
        kind,
        DamageNature::Normal,
    ) && !core.player(to).jidong_active
        && !core.player(to).yinshen_active
        && !core.player(to).jileng_active
        && !core.player(to).yinmi_active
}

pub(super) fn spawn_virtual_sha(
    ctx: &mut Ctx<'_>,
    user: PlayerId,
    targets: Vec<PlayerId>,
    nature: DamageNature,
) {
    let alive_targets: Vec<PlayerId> = targets
        .into_iter()
        .filter(|&t| {
            ctx.core.player(t).alive
                && t != user
                && can_target_card(ctx.core, ctx.catalog, user, t, CardKind::Sha)
        })
        .collect();
    if alive_targets.is_empty() {
        return;
    }
    let kind = match nature {
        DamageNature::Fire => CardKind::FireSha,
        DamageNature::Thunder => CardKind::LeiSha,
        DamageNature::Normal => CardKind::Sha,
    };
    ctx.emit(EventData::CardUsed(card_use_virtual_with_nature(
        user,
        alive_targets,
        nature,
        kind,
    )));
}

pub(in super::super) fn apply_get_buff(ctx: &mut Ctx<'_>, t: PlayerId) {
    if !ctx.core.player(t).alive {
        return;
    }
    let mut list = vec![1u8, 2, 3, 4, 5, 6];
    let p = ctx.core.player(t);
    if p.hp >= p.max_hp as i32 {
        list.retain(|x| *x != 2);
    }
    if p.judge.is_empty() {
        list.retain(|x| *x != 5);
    }
    if !p.linked && !p.face_down {
        list.retain(|x| *x != 6);
    }
    if flag(ctx.core, t, "u05_mod:gj_qianxing") > 0 {
        list.retain(|x| *x != 4);
    }
    if list.is_empty() {
        return;
    }
    let idx = ctx.core.rng.gen_range(list.len() as u32) as usize;
    match list[idx] {
        1 => ctx.start_draw(t, 1),
        2 => {
            let _ = ctx.recover(t, 1, None);
        }
        3 => {
            let _ = ctx.change_hujia(t, 1);
        }
        4 => {
            set_flag(ctx.core, t, "u05_mod:gj_qianxing", 1);
            set_flag(ctx.core, t, "gj_buff_qianxing_temp", 1);
        }
        5 => {
            let judges: Vec<CardId> = ctx.core.player_mut(t).judge.drain(..).collect();
            for c in &judges {
                ctx.to_discard(*c);
            }
            if !judges.is_empty() {
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: judges,
                });
            }
        }
        6 => {
            if ctx.core.player(t).linked {
                ctx.core.player_mut(t).linked = false;
            }
            if ctx.core.player(t).face_down {
                ctx.core.player_mut(t).face_down = false;
            }
        }
        _ => {}
    }
}

pub(in super::super) fn apply_get_debuff(ctx: &mut Ctx<'_>, t: PlayerId, exclude6: bool) {
    if !ctx.core.player(t).alive {
        return;
    }
    let mut list = vec![1u8, 2, 3, 4, 5, 6];
    if exclude6 {
        list.retain(|x| *x != 6);
    }
    let p = ctx.core.player(t);
    if p.hand.is_empty() && p.equips.is_empty() {
        list.retain(|x| *x != 1);
    }
    if p.linked {
        list.retain(|x| *x != 4);
    }
    if p.fengyin_active {
        list.retain(|x| *x != 5);
    }
    if p.hp <= 1 {
        list.retain(|x| *x != 3);
        if list.len() > 1 {
            list.retain(|x| *x != 2);
        }
    }
    if list.is_empty() {
        return;
    }
    let idx = ctx.core.rng.gen_range(list.len() as u32) as usize;
    match list[idx] {
        1 => {
            let he = hand_and_equips(ctx.core, t);
            if !he.is_empty() {
                let ci = ctx.core.rng.gen_range(he.len() as u32) as usize;
                discard_card_from_he(ctx, t, he[ci]);
            }
        }
        2 => ctx.emit(EventData::LoseHp {
            player: t,
            amount: 1,
        }),
        3 => ctx.emit(EventData::Damage(damage_data(None, t, 1))),
        4 => ctx.core.player_mut(t).linked = true,
        5 => ctx.core.player_mut(t).fengyin_active = true,
        6 => {
            let has_kind_in_judge = |k: CardKind| {
                ctx.core
                    .player(t)
                    .judge
                    .iter()
                    .any(|&jc| ctx.catalog.card_kind(jc) == k)
            };
            let mut candidates: Vec<(bool, CardId)> = Vec::new();
            for &c in &ctx.core.deck {
                let k = ctx.catalog.card_kind(c);
                if matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian)
                    && !has_kind_in_judge(k)
                {
                    candidates.push((true, c));
                }
            }
            for &c in &ctx.core.discard {
                let k = ctx.catalog.card_kind(c);
                if matches!(k, CardKind::Le | CardKind::Bing | CardKind::ShanDian)
                    && !has_kind_in_judge(k)
                {
                    candidates.push((false, c));
                }
            }
            if candidates.is_empty() {
                if !exclude6 {
                    apply_get_debuff(ctx, t, true);
                }
                return;
            }
            let pick = ctx.core.rng.gen_range(candidates.len() as u32) as usize;
            let (from_deck, c) = candidates[pick];
            if from_deck {
                ctx.core.deck.retain(|x| *x != c);
            } else {
                ctx.core.discard.retain(|x| *x != c);
            }
            ctx.core.player_mut(t).judge.push(c);
        }
        _ => {}
    }
}

pub(super) fn land_kind_to_idx(kind: CardKind) -> Option<u32> {
    LAND_KINDS
        .iter()
        .position(|&k| k == kind)
        .map(|i| i as u32 + 1)
}

pub(super) fn idx_to_land_kind(idx: u32) -> Option<CardKind> {
    idx.checked_sub(1).and_then(|i| LAND_KINDS.get(i as usize)).copied()
}

pub(super) fn land_skill_keys(kind: CardKind) -> &'static [&'static str] {
    match kind {
        CardKind::Longlushanfeng => &[MTG_LONGSHANFENG_KEY],
        CardKind::Haidao => &[MTG_HAIDAO_KEY],
        CardKind::Shuimomuxue => &[MTG_SHUIMOMUXUE_KEY],
        CardKind::Feixu => &[MTG_FEIXU_KEY],
        CardKind::Yixialan => &[MTG_YIXIALAN_KEY, MTG_YIXIALAN_RECAST_KEY],
        CardKind::Shamolvzhou => &[MTG_SHAMOLVZHOU_KEY],
        CardKind::Linzhongjianta => &[MTG_LINZHONGJIANTA_KEY, MTG_LINZHONGJIANTA_SKILL_KEY],
        CardKind::Bingheyaosai => &[MTG_BINGHEYAOSAI_KEY],
        CardKind::Youlin => &[MTG_YOULIN_KEY, MTG_YOULIN_DISCOVER_KEY],
        CardKind::Lindixiliu => &[MTG_LINDIXILIU_KEY],
        _ => &[],
    }
}

pub(super) fn build_ask(
    obs: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Option<ExpectedAsk> {
    if options.is_empty() {
        return None;
    }
    Some(ask(obs, prompt, kind, options, min, max, allow_cancel))
}

pub(super) fn find_yuheng_equip_card(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Option<CardId> {
    core.player(owner).equips.iter().copied().find(|&c| {
        catalog.cards[c as usize].kind == CardKind::Gjyuheng
            || catalog.cards[c as usize].name.starts_with("gjyuheng")
    })
}

pub(super) fn yuheng_tier(core: &GameStateCore, catalog: &Catalog, card: CardId) -> u8 {
    let cname = catalog.cards[card as usize].name;
    let flag_tier = flag(core, 0, &format!("gj_yuheng_tier:{card}"));
    if cname.contains("pro") || flag_tier >= 2 {
        2
    } else if cname.contains("plus") || flag_tier >= 1 {
        1
    } else {
        0
    }
}

pub(super) fn push_non_self_alive(core: &GameStateCore, owner: PlayerId) -> Vec<ChoiceOption> {
    core.alive_players()
        .filter(|&p| p != owner)
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn die_player(ctx: &mut Ctx<'_>, p: PlayerId, killer: Option<PlayerId>) {
    if ctx.core.player(p).alive {
        ctx.core.player_mut(p).hp = 0;
        ctx.emit(EventData::Death {
            player: p,
            killer,
        });
    }
}

pub(super) fn food_listener_key(kind: CardKind) -> Option<&'static str> {
    Some(match kind {
        CardKind::Tanhuadong => GJ_FOOD_TANHUADONG_KEY,
        CardKind::Yougeng => GJ_FOOD_YOUGENG_KEY,
        CardKind::Qingtuan => GJ_FOOD_QINGTUAN_KEY,
        CardKind::Yuanbaorou => GJ_FOOD_YUANBAOROU_KEY,
        CardKind::Gudonggeng => GJ_FOOD_GUDONGGENG_KEY,
        CardKind::Mapodoufu => GJ_FOOD_MAPODOUFU_KEY,
        CardKind::Chunbing => GJ_FOOD_CHUNBING_KEY,
        CardKind::Liyutang => GJ_FOOD_LIYUTANG_KEY,
        CardKind::Molicha => GJ_FOOD_MOLICHA_KEY,
        CardKind::Xiajiao => GJ_FOOD_XIAJIAO_KEY,
        CardKind::Mizhilianou => GJ_FOOD_MIZHLIANOU_KEY,
        CardKind::Luyugeng => GJ_FOOD_LUYUGENG_KEY,
        CardKind::Yunvyuanshen => GJ_YUNVYUANSHEN_LISTENER_KEY,
        CardKind::Ziyangdan => GJ_FOOD_ZIYANGDAN_KEY,
        _ => return None,
    })
}

pub(super) fn seat_order_enemies(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    let mut out: Vec<PlayerId> = core
        .alive_players()
        .filter(|&p| p != owner && !core.is_friend_of(catalog, owner, p))
        .collect();
    if out.is_empty() {
        out = core.alive_players().filter(|&p| p != owner).collect();
    }
    out
}
