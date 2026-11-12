use super::*;

pub(super) fn u_matches_lw2(event: &EventData, owner: u8, ctx: &QueryCtx<'_>) -> bool {
    matches!(event,
        EventData::CardUsed(u)
        if u.player == owner
            && u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog))
                == crate::catalog::CardKind::Shuigong
            && !u.targets.is_empty())
}

/// 诈降条件（上游 `player.countUsed() < player.getDamagedHp()`）：本阶段已
/// 用牌数 < 已损失体力（`max_hp - hp`，下限 0）。
pub fn sbzhaxiang_active(core: &crate::state::GameStateCore, p: PlayerId) -> bool {
    let lost = (core.player(p).max_hp as i64 - core.player(p).hp as i64).max(0) as u32;
    core.turn.cards_used_play_phase < lost
}

/// 蛊惑声明选项（Tag=下标）。
pub(super) fn reguhuo_kind_options() -> Vec<ChoiceOption> {
    (0..REGUHUO_KINDS.len())
        .map(|i| ChoiceOption::Tag(i as u8))
        .collect()
}

/// 蛊惑下标 → 牌种。
pub(super) fn reguhuo_kind_at(idx: u8) -> Option<crate::catalog::CardKind> {
    REGUHUO_KINDS.get(idx as usize).copied()
}

/// 疾风范围扩展适用牌种（上游：`card.name=='sha'` 或 `selectTarget` 为
/// [1,1] 的锦囊——原生按「恰一名目标」的既有牌种集合判定）。
pub fn jifeng_single_target_trick(kind: crate::catalog::CardKind) -> bool {
    use crate::catalog::CardKind as K;
    matches!(
        kind,
        K::Duel | K::Snatch | K::Dismantle | K::Le | K::Bing | K::Jiedao
    )
}

/// 昆仑镜回滚：把 `card` 从当前所在任意区移到 `player` 的第 `zone` 区
/// （0=手牌 1=装备 2=判定区）；装备区变更同步授予/撤销装备技。
pub(super) fn kunlunjing_move_to(ctx: &mut Ctx<'_>, card: CardId, player: PlayerId, zone: u8) {
    // 先自任意区移除（含装备技撤销）
    for p in 0..ctx.core.players.len() as PlayerId {
        if ctx.core.player(p).equips.contains(&card) {
            ctx.core.player_mut(p).equips.retain(|x| *x != card);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, card);
        }
        ctx.core.player_mut(p).hand.retain(|x| *x != card);
        ctx.core.player_mut(p).judge.retain(|x| *x != card);
    }
    ctx.core.table.retain(|x| *x != card);
    ctx.core.deck.retain(|x| *x != card);
    ctx.core.discard.retain(|x| *x != card);
    match zone {
        0 => ctx.core.player_mut(player).hand.push(card),
        1 => {
            ctx.core.player_mut(player).equips.push(card);
            crate::kernel::grant_equip_skills(ctx, player, card);
        }
        _ => ctx.core.player_mut(player).judge.push(card),
    }
}

/// 神弓借用候选（swd 批 148）：持有神弓的 `responder` 需要打出【闪】
/// （want_shan=true，借他人**防具**）或【杀】（借他人**武器**）时，返回
/// 其他角色对应装备栏的牌作为候选；否则空。
pub fn shengong_borrow_options(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    want_shan: bool,
) -> Vec<ChoiceOption> {
    let Some(def) = ctx.catalog.skill_by_name(SWD_SHENGONG_KEY) else {
        return Vec::new();
    };
    if !ctx.core.has_skill(responder, def) {
        return Vec::new();
    }
    let want = if want_shan {
        crate::catalog::EquipSlot::Armor
    } else {
        crate::catalog::EquipSlot::Weapon
    };
    let mut out = Vec::new();
    for p in ctx.core.alive_players() {
        if p == responder {
            continue;
        }
        for c in ctx.core.player(p).equips.iter().copied() {
            if ctx.catalog.card_slot_of(ctx.core, c) == Some(want) {
                out.push(ChoiceOption::Card(c));
            }
        }
    }
    out
}

/// 借用的装备是否属于 `responder` 之外的角色（若是则返回其所有者）。
pub fn shengong_borrowed_owner(
    ctx: &Ctx<'_>,
    responder: PlayerId,
    card: CardId,
) -> Option<PlayerId> {
    ctx.core
        .alive_players()
        .find(|p| *p != responder && ctx.core.player(*p).equips.contains(&card))
}

/// 天书候选：目标武将技能中自己未持有者（登记差异：上游排除 unique
/// 限定技，原生无 unique 元数据，故一并列出）。
pub(super) fn swdtianshu_skills(
    ctx: &Ctx<'_>,
    target: PlayerId,
    owner: PlayerId,
) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for slot in ctx.core.player(target).general_slots.iter().flatten() {
        let Some((_, _, skills, _, _)) = ctx.catalog.general_meta(*slot) else {
            continue;
        };
        for name in skills {
            if out.contains(&name) {
                continue;
            }
            let Some(def) = ctx.catalog.skill_by_name(name) else {
                continue;
            };
            if ctx.core.skills.contains_key(&(owner, def)) {
                continue;
            }
            out.push(name);
        }
    }
    out
}

pub(super) fn cyqiaoxie_rule(name: &str) -> Option<CyqiaoxieRule> {
    if name.contains("jiguantong") {
        Some(CyqiaoxieRule::AllOthers)
    } else if name.contains("gouhunluo")
        || name.contains("jiguanfeng")
        || name.contains("jiguanyuan")
        || name.contains("jiguanyaoshu")
    {
        Some(CyqiaoxieRule::OneOther)
    } else if name.contains("zhiluxiaohu")
        || name.contains("mujiaren")
        || name.contains("jiguanshu")
    {
        Some(CyqiaoxieRule::SelfTarget)
    } else {
        None
    }
}

/// 单目标机关牌的目标合法性（近似：勾魂锣未挂附 / 机关凤存活 /
/// 机关妖术 手牌>体力且无标记 / 机关怨 无标记）。
pub(super) fn cyqiaoxie_target_ok(ctx: &Ctx<'_>, target: PlayerId, name: &str) -> bool {
    if !ctx.core.player(target).alive {
        return false;
    }
    if name.contains("gouhunluo") || name.contains("jiguanyuan") {
        return ctx.core.player(target).gouhunluo_rounds == 0
            && ctx.core.player(target).jiguanyuan_cards.is_empty();
    }
    if name.contains("jiguanyaoshu") {
        let hand = ctx.core.player(target).hand.len() as i32;
        return hand > ctx.core.player(target).hp && ctx.core.player(target).equips.len() < 5;
    }
    true
}

/// 机关牌候选：牌堆/弃牌堆中可得的机关牌（各牌名取首张代表；自目标类
/// 检查自身前置、单目标类检查存在合法目标；至多 3 张——上游
/// `randomGets(3)` 的确定性近似）。
pub(super) fn cyqiaoxie_candidates(ctx: &Ctx<'_>, owner: PlayerId) -> Vec<ChoiceOption> {
    let mut seen: Vec<&str> = Vec::new();
    let mut out: Vec<ChoiceOption> = Vec::new();
    for c in ctx.core.deck.iter().chain(ctx.core.discard.iter()).copied() {
        let def = &ctx.catalog.cards[c as usize];
        if def.kind != crate::catalog::CardKind::Jiguan {
            continue;
        }
        let name: &str = def.name;
        if seen.contains(&name) {
            continue;
        }
        let Some(rule) = cyqiaoxie_rule(&name) else {
            continue;
        };
        let ok = match rule {
            CyqiaoxieRule::SelfTarget => {
                if name.contains("mujiaren") {
                    !ctx.core.turn.mujiaren_used
                } else if name.contains("jiguanshu") {
                    !ctx.core.player(owner).equips.is_empty()
                } else {
                    true
                }
            }
            CyqiaoxieRule::OneOther => ctx
                .core
                .alive_players()
                .any(|p| p != owner && cyqiaoxie_target_ok(ctx, p, &name)),
            CyqiaoxieRule::AllOthers => ctx.core.alive_players().any(|p| p != owner),
        };
        if !ok {
            continue;
        }
        seen.push(name);
        out.push(ChoiceOption::Card(c));
        if out.len() >= 3 {
            break;
        }
    }
    out
}

/// 使用所选机关牌：实体从所在区入手 → `CardUsed` 帧结算（机关牌按名路由）。
pub(super) fn cyqiaoxie_use(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) {
    if let Some(pos) = ctx.core.deck.iter().position(|c| *c == card) {
        ctx.core.deck.remove(pos);
    } else if let Some(pos) = ctx.core.discard.iter().position(|c| *c == card) {
        ctx.core.discard.remove(pos);
    } else {
        return;
    }
    ctx.core.player_mut(owner).hand.push(card);
    ctx.log(LogEvent::SkillTriggered {
        skill: "此桥邪",
        owner, presentation: None });
    ctx.emit(crate::event::EventData::CardUsed(crate::machine::card_use(
        card, owner, targets,
    )));
}

/// 颂词记录（按座位序插入，去重）。
pub(super) fn songci_record(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId) {
    let p = ctx.core.player_mut(owner);
    if !p.songci_targets.contains(&target) {
        p.songci_targets.push(target);
        p.songci_targets.sort_unstable();
    }
}

/// 存活查询（守印收束用）。
pub(super) fn pl_alive(ctx: &Ctx<'_>, p: PlayerId) -> bool {
    ctx.core.player(p).alive
}

/// 义山可回收判定（查询态）：牌存在且不属于持有者（上游 filter：
/// `get.owner(card)!=player && get.position(card)!='s'`——自己手里的与特殊
/// 区的不收）。
pub(super) fn yishan_recoverable_state(
    state: &crate::state::GameStateCore,
    owner: PlayerId,
    card: CardId,
) -> bool {
    let own = state.player(owner);
    if own.hand.contains(&card) || own.equips.contains(&card) || own.judge.contains(&card) {
        return false;
    }
    state.discard.contains(&card)
        || state.table.contains(&card)
        || state
            .players
            .iter()
            .any(|p| p.hand.contains(&card) || p.equips.contains(&card) || p.judge.contains(&card))
}

pub(super) fn yishan_recoverable(ctx: &Ctx<'_>, owner: PlayerId, card: CardId) -> bool {
    yishan_recoverable_state(ctx.core, owner, card)
}

/// 义山取回：从所在区取出并入手（弃牌堆/桌面/他人手牌·装备·判定区）。
pub(super) fn yishan_take(ctx: &mut Ctx<'_>, owner: PlayerId, card: CardId) -> bool {
    if let Some(pos) = ctx.core.discard.iter().position(|c| *c == card) {
        ctx.core.discard.remove(pos);
    } else if let Some(pos) = ctx.core.table.iter().position(|c| *c == card) {
        ctx.core.table.remove(pos);
    } else {
        let holder = ctx.core.players.iter().position(|p| {
            p.hand.contains(&card) || p.equips.contains(&card) || p.judge.contains(&card)
        });
        let Some(h) = holder else {
            return false;
        };
        let h = h as PlayerId;
        if ctx.core.player(h).equips.contains(&card) {
            ctx.core.player_mut(h).equips.retain(|c| *c != card);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, h, card);
        } else if ctx.core.player(h).judge.contains(&card) {
            ctx.core.player_mut(h).judge.retain(|c| *c != card);
        } else {
            ctx.core.player_mut(h).hand.retain(|c| *c != card);
        }
    }
    ctx.core.player_mut(owner).hand.push(card);
    ctx.core
        .player_mut(owner)
        .yishan_lost
        .retain(|c| *c != card);
    ctx.log(LogEvent::CardsGiven {
        from: owner,
        to: owner,
        cards: vec![card],
    });
    true
}

/// 清正候选花色：手牌中非空的花色（上游 filterButton：该花色手牌数 >0 且
/// 全部可弃置；原生手牌默认可弃置，登记差异）。
pub(super) fn mbcm_qingzheng_suits(ctx: &Ctx<'_>, player: PlayerId) -> Vec<u8> {
    let mut suits: Vec<u8> = Vec::new();
    for c in ctx.core.player(player).hand.iter().copied() {
        if let Some(s) = mbcm_suit_of(ctx, c) {
            if !suits.contains(&s) {
                suits.push(s);
            }
        }
    }
    suits.sort_unstable();
    suits
}

/// 牌花色编码（0 黑桃 / 1 红桃 / 2 梅花 / 3 方块；与选择项 Tag 同口径）。
pub(super) fn mbcm_suit_of(ctx: &Ctx<'_>, card: CardId) -> Option<u8> {
    Some(match ctx.catalog.cards[card as usize].suit {
        crate::catalog::Suit::Spade => 0,
        crate::catalog::Suit::Heart => 1,
        crate::catalog::Suit::Club => 2,
        crate::catalog::Suit::Diamond => 3,
    })
}

/// 涅槃（T11 Boss 庞统族批 118，规则 v306）：限定技收束——置已发动、
/// 弃置全部牌（手牌/装备/判定区）、解除横置、翻回正面、摸 3、回复至
/// 3 点体力；随后若持浴火则授予看破/神威/朱羽（目录存在才授予，缺目录
/// 按上游 addSkill 无对应技能时不虚构空操作）并置关卡循环型 1。
pub(crate) fn niepan_effect(ctx: &mut Ctx<'_>, owner: PlayerId) {
    ctx.core.player_mut(owner).niepan_used = true;
    // 弃置全部牌（手/装/判）
    let hand: Vec<CardId> = ctx.core.player(owner).hand.clone();
    for c in hand {
        ctx.discard_from_hand(owner, c);
    }
    let equips: Vec<CardId> = ctx.core.player(owner).equips.clone();
    for c in equips {
        ctx.core.player_mut(owner).equips.retain(|x| *x != c);
        ctx.to_discard(c);
    }
    let judge: Vec<CardId> = ctx.core.player(owner).judge.clone();
    for c in judge {
        ctx.core.player_mut(owner).judge.retain(|x| *x != c);
        ctx.to_discard(c);
    }
    // 解除横置 + 翻回正面
    ctx.core.player_mut(owner).linked = false;
    ctx.core.player_mut(owner).face_down = false;
    // 摸 3 + 回复至 3
    ctx.start_draw(owner, 3);
    let hp = ctx.core.player(owner).hp;
    if hp < 3 {
        ctx.recover(owner, (3 - hp) as u32, None);
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "涅槃",
        owner, presentation: None });
    // 浴火：授予三技（存在才授予）+ 关卡循环型 1
    if let Some(def) = ctx.catalog.skill_by_name(BOSS_YUHOU_KEY) {
        if ctx.core.has_skill(owner, def) {
            // 上游 addSkill('kanpo'/'shenwei'/'zhuyu')；原生目录中神威为
            // 中名（看破/朱羽未转换 → 存在性守卫跳过）
            for name in ["kanpo", BOSS_SHENWEI_KEY, "zhuyu"] {
                let Some(sd) = ctx.catalog.skill_by_name(name) else {
                    continue;
                };
                if !ctx.core.skills.contains_key(&(owner, sd)) {
                    ctx.insert_skill(
                        (owner, sd),
                        crate::state::SkillInstance {
                            owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                    ctx.log(LogEvent::SkillToggled {
                        player: owner,
                        skill: ctx.catalog.skill_name(sd),
                        enabled: true,
                    });
                }
            }
            if let Some(stage) = ctx.core.boss_stage_mut() {
                stage.loop_type = 1;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "浴火",
                owner, presentation: None });
        }
    }
}

/// 潜龙阈值解锁：按道心值授予清正（25）/酒诗（50）/放逐（75）/绝进（99）
/// ——先撤销四个（上游 removeAdditionalSkill），再按阈值授予；目录缺该
/// 技能时跳过（上游 `if(!lib.skill[s]) return` 同口径，外部包未转换时
/// 不虚构空操作）。
pub(super) fn mbqianlong_refresh(ctx: &mut Ctx<'_>, owner: PlayerId) {
    const THRESHOLDS: [(&str, u8); 4] = [
        ("mbcmqingzheng", 25),
        ("mbcmjiushi", 50),
        ("mbcmfangzhu", 75),
        ("mbjuejin", 99),
    ];
    let marks = ctx.core.player(owner).mbqianlong_marks;
    for (name, threshold) in THRESHOLDS {
        let Some(def) = ctx.catalog.skill_by_name(name) else {
            continue;
        };
        let granted = marks >= threshold;
        let has = ctx.core.skills.contains_key(&(owner, def));
        if granted && !has {
            ctx.insert_skill(
                (owner, def),
                crate::state::SkillInstance {
                    owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
            ctx.log(LogEvent::SkillToggled {
                player: owner,
                skill: ctx.catalog.skill_name(def),
                enabled: true,
            });
        } else if !granted && has {
            ctx.core.skills.remove(&(owner, def));
            ctx.log(LogEvent::SkillToggled {
                player: owner,
                skill: ctx.catalog.skill_name(def),
                enabled: false,
            });
        }
    }
    if marks >= 99 {
        if let Some(wt_def) = ctx.catalog.skill_by_name(HLSS_MBWEITONG_KEY) {
            ctx.core.remove_skill_and_r05_mark(ctx.catalog, owner, wt_def);
        }
    }
}

/// 起风候选：可用**非延时锦囊**牌种（未用过、有合法目标）。
/// 上游从 `lib.inpile` 取 `type == 'trick'` 且 `cardEnabled` 者；原生以
/// 目录固定表承载标准非延时锦囊（登记差异：扩展锦囊牌种不入候选、
/// 借刀杀人不入候选）。
pub(super) fn hlss_fengqi_candidates(
    ctx: &Ctx<'_>,
    owner: PlayerId,
) -> Vec<crate::catalog::CardKind> {
    use crate::catalog::CardKind::*;
    const TABLE: [crate::catalog::CardKind; 9] = [
        Wuzhong,
        Wugu,
        Taoyuan,
        SavageAttack,
        ArcheryAttack,
        Duel,
        Snatch,
        Dismantle,
        Chain,
    ];
    let used = &ctx.core.player(owner).hlss_fengqi_used;
    TABLE
        .into_iter()
        .filter(|k| {
            let code = crate::archive::kind_code(*k);
            if used.contains(&code) {
                return false;
            }
            let (min, max) = hlss_fengqi_targets(*k);
            if max == 0 {
                return true;
            }
            let count = ctx
                .core
                .alive_players()
                .filter(|p| hlss_fengqi_target_ok(ctx, owner, *p, *k))
                .count();
            count >= min as usize
        })
        .collect()
}

/// 起风目标数（min, max；max=0 表示无目标直接使用）。
pub(super) fn hlss_fengqi_targets(kind: crate::catalog::CardKind) -> (u8, u8) {
    use crate::catalog::CardKind::*;
    match kind {
        Wuzhong | Wugu | Taoyuan | SavageAttack | ArcheryAttack => (0, 0),
        Duel | Snatch | Dismantle => (1, 1),
        Chain => (1, 2),
        _ => (0, 0),
    }
}

/// 无目标锦囊的目标表：AOE 按出牌口径显式给出全部其他存活者（自下家
/// 起座位序冻结）；无中/五谷/桃园由各自机器自行取目标（空表）。
pub(super) fn hlss_fengqi_all_targets(
    ctx: &Ctx<'_>,
    owner: PlayerId,
    kind: crate::catalog::CardKind,
) -> Vec<PlayerId> {
    if kind.aoe_need().is_some() {
        ctx.core
            .seat_order_from(ctx.core.next_alive(owner))
            .into_iter()
            .filter(|p| *p != owner)
            .collect()
    } else {
        Vec::new()
    }
}

/// 起风目标合法性（近似：单目标锦囊按距离/持牌判；登记差异）。
pub(super) fn hlss_fengqi_target_ok(
    ctx: &Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
    kind: crate::catalog::CardKind,
) -> bool {
    use crate::catalog::CardKind::*;
    if target == owner {
        return matches!(kind, Chain);
    }
    if !ctx.core.player(target).alive {
        return false;
    }
    match kind {
        Duel => true,
        Snatch => {
            ctx.core.computed_distance(ctx.catalog, owner, target) <= 1
                && (!ctx.core.player(target).hand.is_empty()
                    || !ctx.core.player(target).equips.is_empty())
        }
        Dismantle => {
            ctx.core.computed_distance(ctx.catalog, owner, target) <= 1
                && (!ctx.core.player(target).hand.is_empty()
                    || !ctx.core.player(target).equips.is_empty()
                    || !ctx.core.player(target).judge.is_empty())
        }
        Chain => true,
        _ => true,
    }
}

/// 乱击·多鹏面（武将面批 81）：手牌选项列表（SelectCards min=max=2 读
/// 两张，花色一致性在消费段校验——上游 complexCard 逐张校验同口径）。
pub(super) fn same_suit_pair_options(
    core: &crate::state::GameStateCore,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .map(ChoiceOption::Card)
        .collect()
}

/// 乱击两卡花色一致性校验（上游 filterCard: get.suit(card) ==
/// get.suit(ui.selected.cards[0]) 同口径）。
pub(super) fn upstream_same_suit(ctx: &Ctx<'_>, cards: &[CardId]) -> bool {
    let Some(first) = cards.first() else {
        return false;
    };
    let suit = ctx.catalog.cards[*first as usize].suit;
    cards
        .iter()
        .all(|c| ctx.catalog.cards[*c as usize].suit == suit)
}

pub use crate::vocabulary::card_skills::yangpijuan_class;

pub(super) fn ctianfu_clear(ctx: &mut crate::machine::Ctx<'_>, owner: u8) {
    if let Some(cid) = ctx.core.player(owner).ctianfu_card {
        let cid = cid as u32;
        if let Some(pos) = ctx
            .core
            .player(owner)
            .off_court
            .iter()
            .position(|x| *x == cid)
        {
            ctx.core.player_mut(owner).off_court.remove(pos);
            ctx.to_discard(cid);
            ctx.log(LogEvent::CardDiscarded {
                player: owner,
                cards: vec![cid],
            });
        }
    }
    let pl = ctx.core.player_mut(owner);
    pl.ctianfu_source = None;
    pl.ctianfu_card = None;
}

impl HostMachine {
    /// 手里剑目标询问规格（机器与恢复派生共用同一形状来源）。
    pub(super) fn target_choice_spec(ctx: &Ctx<'_>, target: PlayerId) -> ChoiceSpec {
        let mut options = vec![ChoiceOption::Pass];
        options.extend(
            ctx.core
                .player(target)
                .hand
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c).is_some())
                .chain(ctx.core.player(target).equips.iter().copied())
                .map(ChoiceOption::Card),
        );
        ChoiceSpec {
            kind: ChoiceKind::RespondCard,
            prompt: HOST_TARGET_DISCARD_EQUIP,
            options,
            min: 1,
            max: 1,
            allow_cancel: true,
        }
    }

    /// 横刀被获得面：随机获得当前回合角色一张 he 牌。
    pub(super) fn hwendao_gain_random(&mut self, ctx: &mut Ctx<'_>, t: PlayerId) -> Flow {
        let mut pool: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
        pool.extend(ctx.core.player(t).equips.iter().copied());
        if pool.is_empty() || !ctx.core.player(self.owner).alive {
            return Flow::Done;
        }
        let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
        let c = pool[j];
        let in_hand = ctx.core.player(t).hand.contains(&c);
        if in_hand {
            ctx.take_hand_card(t, c);
        } else {
            ctx.core.player_mut(t).equips.retain(|x| *x != c);
            if ctx.catalog.card_kind(c).is_equip() {
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
            }
        }
        ctx.core.player_mut(self.owner).hand.push(c);
        ctx.log(LogEvent::CardsTaken {
            from: t,
            to: self.owner,
            cards: vec![c],
            public: !in_hand,
        });
        Flow::Done
    }

    /// 对诗获得面：随机获得目标一张牌（he；登记差异：上游弃牌者自选，
    /// 原生 RNG 代选——浴尘同款）。
    pub(super) fn duishi_gain_random(&mut self, ctx: &mut Ctx<'_>, t: PlayerId) -> Flow {
        let mut pool: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
        pool.extend(ctx.core.player(t).equips.iter().copied());
        if pool.is_empty() || !ctx.core.player(self.owner).alive {
            return Flow::Done;
        }
        let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
        let c = pool[j];
        let in_hand = ctx.core.player(t).hand.contains(&c);
        if in_hand {
            ctx.take_hand_card(t, c);
        } else {
            ctx.core.player_mut(t).equips.retain(|x| *x != c);
            if ctx.catalog.card_kind(c).is_equip() {
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
            }
        }
        ctx.core.player_mut(self.owner).hand.push(c);
        ctx.log(LogEvent::CardsTaken {
            from: t,
            to: self.owner,
            cards: vec![c],
            public: !in_hand,
        });
        Flow::Done
    }

    /// 鼠槌逐杀循环头（ShuchuiTargetLoop 臂与链头共用）：目标存活且
    /// 剩余次数未尽且手中有杀则用出（重扫首张=上游 getCards("h")[0]），
    /// Yield 等待本杀结算后重入；否则收束——链上任一杀造成伤害
    ///（commit_damage 钩置位）则摸 1，清武装旗。
    pub(super) fn shuchui_loop(&mut self, ctx: &mut Ctx<'_>, target: PlayerId) -> Flow {
        if !ctx.core.player(target).alive
            || self.cards.is_empty()
            || !ctx
                .core
                .player(self.owner)
                .hand
                .iter()
                .any(|c| ctx.catalog.card_kind(*c).is_sha_like())
        {
            ctx.core.turn.shuchui_armed = false;
            if ctx.core.turn.shuchui_hit {
                ctx.core.turn.shuchui_hit = false;
                if ctx.core.player(self.owner).alive {
                    ctx.start_draw(self.owner, 1);
                }
            }
            self.cards.clear();
            self.target = None;
            return Flow::Done;
        }
        self.cards.pop();
        let sha = ctx
            .core
            .player(self.owner)
            .hand
            .iter()
            .copied()
            .find(|c| ctx.catalog.card_kind(*c).is_sha_like())
            .expect("loop head checked sha presence");
        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
            card: Some(sha),
            player: self.owner,
            targets: vec![target],
            nature: match ctx.catalog.card_kind(sha) {
                CardKind::FireSha => crate::event::DamageNature::Fire,
                CardKind::LeiSha => crate::event::DamageNature::Thunder,
                _ => crate::event::DamageNature::Normal,
            },
            use_as: None,
            cancelled: false,
        }));
        Flow::Yield
    }
}

pub(super) fn moyan_flag(ctx: &QueryCtx<'_>, owner: PlayerId, key: &str) -> Option<u32> {
    ctx.state
        .vm_aux
        .flags
        .get(&(owner, key.to_string()))
        .copied()
}

pub(super) fn moyan_set_flag(ctx: &mut Ctx<'_>, owner: PlayerId, key: &str, value: u32) {
    ctx.core
        .vm_aux
        .flags
        .insert((owner, key.to_string()), value);
}

pub(super) fn moyan_clear_flag(ctx: &mut Ctx<'_>, owner: PlayerId, key: &str) {
    ctx.core.vm_aux.flags.remove(&(owner, key.to_string()));
}

/// 末焱过滤：一轮内首次判定阶段开始（持有者存活且牌库或弃牌堆非空）；
/// 或持有者受伤且记录的获得者存活、其手牌数不大于获得张数。
pub(super) fn moyan_filter(ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match event {
        EventData::Phase {
            phase: crate::event::Phase::Judge,
            begin: true,
            ..
        } => {
            moyan_flag(ctx, owner, MOYAN_ROUND) != Some(ctx.state.turn.round)
                && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
        }
        EventData::Damage(damage) => {
            if damage.target != owner || damage.prevented || damage.amount == 0 {
                return false;
            }
            let Some(target) = moyan_flag(ctx, owner, MOYAN_TARGET) else {
                return false;
            };
            let Some(limit) = moyan_flag(ctx, owner, MOYAN_COUNT) else {
                return false;
            };
            let target = target as PlayerId;
            moyan_flag(ctx, owner, MOYAN_ROUND).map_or(true, |r| r == ctx.state.turn.round)
                && (target as usize) < ctx.state.players.len()
                && ctx.state.player(target).alive
                && ctx.state.player(target).hand.len() <= limit as usize
        }
        _ => false,
    }
}

pub(super) fn moyan_take_shown(ctx: &mut Ctx<'_>, n: usize) -> Vec<CardId> {
    let mut cards = Vec::new();
    for _ in 0..n {
        if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
            break;
        }
        cards.push(ctx.draw_judge_card());
    }
    cards
}

pub(super) fn moyan_discard_shown(ctx: &mut Ctx<'_>, owner: PlayerId, cards: &[CardId]) {
    if cards.is_empty() {
        return;
    }
    for card in cards {
        ctx.to_discard(*card);
    }
    ctx.log(LogEvent::CardDiscarded {
        player: owner,
        cards: cards.to_vec(),
    });
}

pub(super) fn moyan_give_shown(ctx: &mut Ctx<'_>, owner: PlayerId, to: PlayerId, cards: &[CardId]) {
    for card in cards {
        ctx.core.player_mut(to).hand.push(*card);
    }
    ctx.log(LogEvent::CardsGivenPublic {
        from: owner,
        to,
        cards: cards.to_vec(),
    });
}

/// 末焱回合边界：展示至多 3 张，选一名角色（可取消），该角色至少获得一张。
/// 取消或未选中的牌立即弃置（登记差异：上游留在处理区）。
pub(super) fn moyan_round_step(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        HostStage::Act => {
            moyan_set_flag(ctx, machine.owner, MOYAN_ROUND, ctx.core.turn.round);
            moyan_clear_flag(ctx, machine.owner, MOYAN_TARGET);
            moyan_clear_flag(ctx, machine.owner, MOYAN_COUNT);
            machine.cards = moyan_take_shown(ctx, 3);
            if machine.cards.is_empty() {
                return Flow::Done;
            }
            ctx.log(LogEvent::CardsRevealed {
                cards: machine.cards.clone(),
                to: ctx.core.alive_players().collect(),
            });
            let options: Vec<ChoiceOption> =
                ctx.core.alive_players().map(ChoiceOption::Player).collect();
            if options.is_empty() {
                let cards = machine.cards.clone();
                moyan_discard_shown(ctx, machine.owner, &cards);
                machine.cards.clear();
                return Flow::Done;
            }
            machine.stage = HostStage::TargetChoice;
            Flow::WaitChoice {
                observer: machine.owner,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: D3_MOYAN_TARGET,
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            }
        }
        HostStage::TargetChoice => {
            let picked = ctx.answer().iter().find_map(|opt| match opt {
                ChoiceOption::Player(player) => Some(*player),
                _ => None,
            });
            let Some(target) = picked.filter(|player| ctx.core.player(*player).alive) else {
                let cards = machine.cards.clone();
                moyan_discard_shown(ctx, machine.owner, &cards);
                machine.cards.clear();
                return Flow::Done;
            };
            machine.target = Some(target);
            machine.stage = HostStage::SelectCard;
            let max = machine.cards.len();
            Flow::WaitChoice {
                observer: target,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: D3_MOYAN_GAIN,
                    options: machine
                        .cards
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect(),
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            }
        }
        HostStage::SelectCard => {
            let wanted: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|opt| match opt {
                    ChoiceOption::Card(card) => Some(*card),
                    _ => None,
                })
                .collect();
            let Some(target) = machine.target else {
                let cards = machine.cards.clone();
                moyan_discard_shown(ctx, machine.owner, &cards);
                machine.cards.clear();
                return Flow::Done;
            };
            let mut gain = Vec::new();
            let mut rest = Vec::new();
            for card in machine.cards.drain(..) {
                if wanted.contains(&card) {
                    gain.push(card);
                } else {
                    rest.push(card);
                }
            }
            if gain.is_empty() {
                moyan_discard_shown(ctx, machine.owner, &rest);
                return Flow::Done;
            }
            moyan_give_shown(ctx, machine.owner, target, &gain);
            moyan_discard_shown(ctx, machine.owner, &rest);
            moyan_set_flag(ctx, machine.owner, MOYAN_TARGET, u32::from(target));
            moyan_set_flag(ctx, machine.owner, MOYAN_COUNT, gain.len() as u32);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

impl HostMachine {
    /// 机关凤未响应路径（规则 v195）：目标全部技能实例禁用 + 授予封印
    /// 状态（其下一阶段开始由 SwdFengyinState 解除）+ 受 1 点伤害（上游
    /// addTempSkill fengyin + damage 同序；登记差异：原生无锁定技区分）。
    pub(super) fn seal_and_damage(&self, ctx: &mut Ctx<'_>) {
        let Some(t) = self.target else {
            return;
        };
        if let Some(fdef) = ctx.catalog.skill_by_name(SWD_FENGYIN_STATE_KEY) {
            let keys: Vec<(PlayerId, crate::ids::SkillDefId)> = ctx
                .core
                .skills
                .iter()
                .map(|((p, d), _)| (*p, *d))
                .filter(|(p, d)| *p == t && *d != fdef)
                .collect();
            for k in keys {
                if let Some(inst) = ctx.core.skills.get_mut(&k) {
                    inst.enabled = false;
                }
            }
            ctx.insert_skill(
                (t, fdef),
                crate::state::SkillInstance {
                    owner: t,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(self.owner),
            t,
            1,
            crate::event::DamageNature::Normal,
            self.card,
        )));
    }
    /// 石会粉：置目标禁用状态（本回合无法对其他角色用牌；
    /// expires_end_of_turn 于其回合结束清除）。
    pub(super) fn grant_shihuifen_ban(&mut self, ctx: &mut Ctx<'_>, target: PlayerId) {
        if let Some(def) = ctx.catalog.skill_by_name(GJ_FOOD_SHIHUIFEN_BAN_KEY) {
            ctx.insert_skill(
                (target, def),
                crate::state::SkillInstance {
                    owner: target,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: true,
                    rounds: 0,
                },
            );
        }
    }
}

/// 食物一次性状态：立即移除监听技实例（咕咚羹「下次」语义）。
pub(super) fn food_remove(ctx: &mut Ctx<'_>, owner: PlayerId, def_key: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(def_key) {
        ctx.core.skills.remove(&(owner, def));
    }
}

/// 食物状态倒计时：效果触发点扣减，归零移除监听技实例。
pub(super) fn food_tick(ctx: &mut Ctx<'_>, owner: PlayerId, def_key: &str) {
    let Some(def) = ctx.catalog.skill_by_name(def_key) else {
        return;
    };
    let remove = match ctx.core.skills.get_mut(&(owner, def)) {
        Some(inst) => {
            inst.rounds = inst.rounds.saturating_sub(1);
            inst.rounds == 0
        }
        None => return,
    };
    if remove {
        ctx.core.skills.remove(&(owner, def));
        ctx.core.skills.remove(&(owner, def));
    }
}

/// 地附主动技同步（mtg 第五批 v104）：换地时撤销旧地附技、授予新地附
/// 技（当前仅依夏兰附重铸技；上游 addGlobalSkill+temp 簿记的实例化对
/// 应物——once_used 随拥有者回合开始重置为既有路径）。
pub(crate) fn sync_land_skill(ctx: &mut Ctx<'_>, owner: PlayerId, new_kind: CardKind) {
    // 地附技映射（当前三例）：依夏兰→重铸、林中尖塔→塔击、幽林→发现
    let key_of = |k: CardKind| match k {
        CardKind::Yixialan => Some(MTG_YIXIALAN_RECAST_KEY),
        CardKind::Linzhongjianta => Some(MTG_LINZHONGJIANTA_SKILL_KEY),
        CardKind::Youlin => Some(MTG_YOULIN_DISCOVER_KEY),
        _ => None,
    };
    let old_key = ctx.core.current_land.and_then(key_of);
    let new_key = key_of(new_kind);
    if let Some(k) = old_key {
        if old_key != new_key {
            if let Some(d) = ctx.catalog.skill_by_name(k) {
                ctx.core.skills.remove(&(owner, d));
            }
        }
    }
    if let Some(k) = new_key {
        if let Some(d) = ctx.catalog.skill_by_name(k) {
            ctx.insert_skill(
                (owner, d),
                crate::state::SkillInstance {
                    owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
    }
}

/// 黑色花色（♠/♣——浴尘 filter 共用）。
pub(super) fn is_black_suit(suit: &crate::catalog::Suit) -> bool {
    matches!(
        suit,
        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
    )
}

/// 器族手持门控（swd 卡牌面补切批⑩）：持有者手牌含对应 kind 实体。
pub(super) fn hand_has_kind(ctx: &crate::catalog::QueryCtx<'_>, owner: u8, kind: CardKind) -> bool {
    ctx.state
        .player(owner)
        .hand
        .iter()
        .any(|c| ctx.catalog.card_kind(*c) == kind)
}
