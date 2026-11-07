//! 出牌校验规则：公共前缀、view-as 牌种转化与目标禁指（`play_card` 的只读前段）。

use super::*;

/// 出牌公共校验前缀（PA05 抽取）：授权/阶段/封锁/限用/手牌持有/目标
/// 界与目标禁指——**纯只读**。`play_card` 与合法动作快拒
/// （`legal_actions::certainly_illegal`）共用同一段代码，语义不可能
/// 漂移；此后牌种分派校验（杀限/距离/满血/装备空栏等）仍由
/// `play_card` 各分支负责。
pub(super) fn common_play_validation(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    // 授权：只有当前回合玩家可以出牌。
    if actor != core.turn.current {
        return Err(RejectReason::NotYourTurn);
    }
    if core.turn.phase != Phase::Play {
        return Err(RejectReason::WrongPhase);
    }
    // 冰封·封（swd 批 134，规则 v314 同批）：出牌封锁者不能使用牌
    if catalog.blocks_all_cards(core, actor) {
        return Err(RejectReason::CardBlocked);
    }
    // 蛊舞·缚（U05 xianjian 阿奴，规则 v154）：蛊舞生效中每回合
    // 最多使用 3 张牌（上游 anwugu2 mod cardEnabled countUsed>=
    // storage 同口径；countUsed=本出牌阶段已用数——登记差异：
    // 上游跨阶段计 countUsed，原生以出牌阶段计数近似）
    if core.player(actor).anwugu_count > 0 && core.turn.cards_used_play_phase >= 3 {
        return Err(RejectReason::CardLimit);
    }
    // 往事封禁（十殿阎罗孟婆，规则 v270）：本回合不能使用或打出
    // 基本牌（上游 mod cardEnabled2 类型禁用→恒定基本牌近似登记；
    // kind 基础集=杀族/闪/桃/酒族/手里剑）；打出面（响应窗）未
    // 落=登记差异（仅使用位 PlayCard 校验）
    if core.player(actor).wangshi_ban && catalog.card_kind(card).is_basic_kind() {
        return Err(RejectReason::CardLimit);
    }
    // 愚钝·福面（swd 武将面批 82）：锁定——无法使用锦囊牌（上游
    // mod cardEnabled/cardRespondable/cardSavable 全禁；登记差异：
    // 原生仅使用位 PlayCard 校验，打出/响应面同往事封禁口径）
    if catalog
        .skill_by_name("yudun")
        .map(|d| core.has_skill(actor, d))
        .unwrap_or(false)
    {
        let kind = catalog.card_kind(card);
        let is_trick =
            !kind.is_basic_kind() && kind.equip_slot().is_none() && kind.is_basic_trick_or_equip();
        if is_trick {
            return Err(RejectReason::CardLimit);
        }
    }
    // 禁言（玩点论杀西真夕红，玩点论杀续批）：体力 ≤2 时黑色锦囊
    // 禁用（查询面，往事封禁先例同位；登记差异：上游四面
    // cardEnabled/Usable/Respondable/Savable，原生仅使用面）
    if core.player(actor).hp <= 2 {
        let jinyan = catalog
            .skill_by_name(crate::content::host::WUDIAN_JINYAN_KEY)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false);
        let kind = catalog.card_kind(card);
        let black_trick = !kind.is_basic_kind()
            && kind.equip_slot().is_none()
            && kind.is_basic_trick_or_equip()
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            );
        if jinyan && black_trick {
            return Err(RejectReason::CardLimit);
        }
    }
    let actor_p = core.player(core.turn.current);
    let in_hand = actor_p.hand.contains(&card);
    let in_equip_va = !in_hand
        && catalog
            .player_vm_view_as_extra_cards(core, core.turn.current, true, None)
            .contains(&card);
    if !in_hand && !in_equip_va {
        return Err(RejectReason::CardNotInHand);
    }
    // 目标 ID 越界必须显式拒绝（不得进入 player() 索引）。
    if targets.iter().any(|t| *t as usize >= core.players.len()) {
        return Err(RejectReason::IllegalTargets);
    }
    // 封印（Boss 应昭封冻，规则 v228）：被封印者本回合无法使用手
    // 牌（上游 fengyin mod cardEnabled2 position=h 同口径；装备区
    // 牌不受限）
    if core.player(core.turn.current).fengyin_active
        && core.player(core.turn.current).hand.contains(&card)
    {
        return Err(RejectReason::IllegalTargets);
    }
    // 冥顽（Boss 梼杌，规则 v219）：回合内锁定集已激活时，出牌
    // 目标只能是自己或已伤害过的角色（上游 boss_minwan mod
    // playerEnabled 同口径）
    {
        let user = core.turn.current;
        if !core.turn.minwan_targets.is_empty() {
            if let Some(mdef) = catalog.skill_by_name(crate::content::standard::BOSS_MINWAN) {
                if core.has_skill(user, mdef)
                    && targets
                        .iter()
                        .any(|t| *t != user && !core.turn.minwan_targets.contains(t))
                {
                    return Err(RejectReason::IllegalTargets);
                }
            }
        }
    }
    Ok(())
}

/// 出牌牌种的 view-as 转化：返回按声明结算的牌种，以及是否经木头面具转化为杀
///（转化后的路由与校验按声明牌种执行）。
pub(super) fn play_kind(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    targets: &[PlayerId],
) -> (CardKind, bool) {
    let actor = core.turn.current;
    if !core.player(actor).hand.contains(&card) {
        if let Some(spec) = catalog.find_player_vm_view_as(core, actor, card, true, None) {
            return (spec.as_kind, false);
        }
    }
    let mut kind = catalog.card_kind(card);
    // 良策（造之仁君，玩点论杀批 15）：基本手牌当【五谷丰登】
    //（viewAs 转化，独立于杀转化链——sha 牌可转化；登记差异：
    // 上游 usable:1，原生不限次——厉焰同款登记）
    let liangce_hit = catalog
        .skill_by_name("liangce")
        .map(|d| core.has_skill(core.turn.current, d))
        .unwrap_or(false);
    if liangce_hit && kind.is_basic_kind() {
        kind = CardKind::Wugu;
    }
    let mut mark_mutoumianju = false;
    // view-as 转化（N08 官方迁移）：
    // 国色（方块手牌当乐不思蜀，优先于武圣）/ 武圣（红色手牌当杀）/
    // 龙胆（闪当杀）——以此后的路由与校验按声明牌种执行。
    if kind != CardKind::Le && kind != CardKind::Sha {
        let actor = core.turn.current;
        let guose = catalog
            .skill_by_name(crate::content::standard::GUO_SE)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false);
        let diamond = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Diamond
        );
        if guose && diamond {
            kind = CardKind::Le;
        }
    }
    if kind != CardKind::Sha && kind != CardKind::Le {
        let actor = core.turn.current;
        let wusheng = catalog
            .skill_by_name(crate::content::standard::WU_SHENG)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false);
        let longdan = catalog
            .skill_by_name(crate::content::standard::LONG_DAN)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false);
        let qixi = catalog
            .skill_by_name(crate::content::standard::QI_XI)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false);
        let black = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        );
        let red = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
        if (wusheng && red) || (longdan && kind == CardKind::Shan) {
            kind = CardKind::Sha;
        }
        // 奇袭（甘宁）：黑色牌当过河拆桥（优先级低于杀类转化）
        else if qixi && black {
            kind = CardKind::Dismantle;
        }
        // 探囊（U05 xianjian 韩菱纱，规则 v148）：梅花手牌当顺手
        // 牵羊（优先级与奇袭同级并列）
        else if catalog
            .skill_by_name(crate::content::host::XJ_TANNANG_KEY)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Club
            )
        {
            kind = CardKind::Snatch;
        }
        // 良策（造之仁君，玩点论杀批 15）：基本手牌当【五谷丰登】
        //（viewAs 转化，厉焰同族；登记差异：上游 usable:1，原生
        // 不限次——厉焰同款登记）
        else if catalog
            .skill_by_name("liangce")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && catalog.card_kind(card).is_basic_kind()
        {
            kind = CardKind::Wugu;
        }
        // 厉焰（陆抗·DIY，运筹武将面批 6）：非基本手牌当【水攻】
        //（viewAs 转化，奇袭/探囊同族；优先级并列尾部落位——上游
        // 独立技无竞争链）
        else if catalog
            .skill_by_name("ycnluweiyan")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && !catalog.card_kind(card).is_basic_kind()
        {
            kind = CardKind::Shuigong;
        }
        // 酒池（Boss 董卓，规则 v258，群雄乱入）：黑桃手牌当【酒】
        //（viewAs 转化，武圣/奇袭同族；与奇袭上游同持择技——native
        // 链奇袭优先，登记）
        else if catalog
            .skill_by_name(crate::content::standard::BOSS_JIUCHI)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Spade
            )
        {
            kind = CardKind::Jiu;
        }
        // 还侠（屈娴，swd 武将面批 27）：红色牌当【杀】（viewAs
        // 转化，武圣同族；上游 he 区红牌仅使用面不含响应=登记）
        else if catalog
            .skill_by_name("huanxia")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && red
        {
            kind = CardKind::Sha;
        }
        // 玉蝉八卦（swd 补切批⑫）：手持 autoViewAs 对应基本牌
        //（你的行动时=PlayCard 自身回合门已限；登记差异随机换形态
        // 与煅造装备面挂后续批）
        else if let Some(vk) = catalog.cards[card as usize].kind.yuchan_auto_view_as() {
            kind = vk;
        }
        // 万钧（浊山铸，swd 武将面批㉖）：装备牌当南蛮入侵使用
        //（上游 viewAs nanman 同口径——出牌与响应双面，响应面
        // 天然收敛登记）
        else if catalog
            .skill_by_name("wanjun")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && catalog.card_kind(card).is_equip()
        {
            kind = CardKind::SavageAttack;
        }
        // 火扑（夜鸦熙，swd 武将面批㉖）：红桃牌当流星火雨使用
        //（上游 viewAs liuxinghuoyu 同口径）
        else if catalog
            .skill_by_name("huopu")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Heart
            )
        {
            kind = CardKind::Liuxinghuoyu;
        }
        // 夜影（小黄龙，swd 武将面批㉗）：黑色牌当乾坤镖使用
        else if catalog
            .skill_by_name("yeying")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            )
        {
            kind = CardKind::Qiankunbiao;
        }
        // 归影（虎魄，swd 武将面批㉗）：黑色牌当偷梁换柱使用
        else if catalog
            .skill_by_name("guiying")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            )
        {
            kind = CardKind::Toulianghuanzhu;
        }
        // 凤泽（司徒强，swd 武将面批㉗）：黑色牌当桃园结义使用
        else if catalog
            .skill_by_name("fengze")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            )
        {
            kind = CardKind::Taoyuan;
        }
        // 破晓（张烈，swd 武将面批㊞）：装备牌当杀使用（上游
        // viewAs sha 同口径）
        else if catalog
            .skill_by_name("poxiao")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && catalog.card_kind(card).is_equip()
        {
            kind = CardKind::Sha;
        }
        // 圣手（淳于恒，swd 武将面批 84）：黑色手牌当草药使用
        //（上游 viewAs caoyao 同口径）
        else if catalog
            .skill_by_name("shengshou")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && matches!(
                catalog.cards[card as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            )
        {
            kind = CardKind::Caoyao;
        }
        // 妖童（齐，swd deferred 消解批 93）：手牌数为奇数时任意
        // 手牌当杀使用（上游 viewAs sha 同口径；闪响应面/无懈
        // 偶数面登记 defer）
        else if catalog
            .skill_by_name("yaotong")
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
            && core.player(actor).hand.len() % 2 == 1
        {
            kind = CardKind::Sha;
        } else {
            // 木头面具（G1 古剑装备）：出牌阶段限一次，任意
            // 手牌当【杀】（once_used 随回合重置；标记延迟至
            // 借用结束后）
            let muto_active = catalog
                .skill_by_name(crate::content::host::GJ_MUTOU_SKILL_KEY)
                .map(|d| {
                    core.skills
                        .get(&(actor, d))
                        .map(|i| i.enabled && !i.once_used)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            if muto_active {
                kind = CardKind::Sha;
                mark_mutoumianju = true;
            } else {
                // 蜜链藕（G1 古剑食物）：红桃手牌当桃
                //（持有状态即生效）
                let mizhilianou = catalog
                    .skill_by_name(crate::content::host::GJ_FOOD_MIZHLIANOU_KEY)
                    .map(|d| core.has_skill(actor, d))
                    .unwrap_or(false);
                let is_heart = matches!(
                    catalog.cards[card as usize].suit,
                    crate::catalog::Suit::Heart
                );
                if mizhilianou && is_heart {
                    kind = CardKind::Tao;
                }
            }
        }
    }
    if targets.is_empty()
        && !mark_mutoumianju
        && matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart
        )
        && catalog
            .skill_by_name(crate::content::host::GJ_FOOD_MIZHLIANOU_KEY)
            .map(|d| core.has_skill(actor, d))
            .unwrap_or(false)
    {
        kind = CardKind::Tao;
    }
    if kind == catalog.card_kind(card) && !mark_mutoumianju {
        if let Some(spec) = catalog.find_player_vm_view_as(core, actor, card, true, None) {
            if (kind != CardKind::Sha && kind != CardKind::Le)
                || spec.filter == crate::rvm_bridge::VmViewAsFilter::Sha
                || spec.as_kind.is_sha_like()
                || targets.is_empty()
                || (kind == CardKind::Sha && core.turn.sha_used_play_phase)
            {
                kind = spec.as_kind;
            }
        }
    }
    (kind, mark_mutoumianju)
}

/// 出牌目标禁指（装备分派之前）：持有者技能与状态使某些牌不能指定特定目标（纯只读）。
pub(super) fn check_play_target_restrictions(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    // 麻茶（G1 古剑食物）：持有者不能成为其他角色黑色牌的
    // 目标（登记差异：仅出牌使用路径过滤，不含响应路径）
    {
        let black = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        );
        if black && !targets.is_empty() {
            let molicha = catalog.skill_by_name(crate::content::host::GJ_FOOD_MOLICHA_KEY);
            if let Some(def) = molicha {
                for t in targets {
                    if *t != core.turn.current && core.skills.contains_key(&(*t, def)) {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
            }
        }
    }
    // 石会粉禁用状态（G1 古剑）：本回合无法对其他角色使用
    // 卡牌（expires_end_of_turn 于其回合结束清除）
    {
        let ban = catalog
            .skill_by_name(crate::content::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
            .map(|def| {
                core.skills
                    .get(&(core.turn.current, def))
                    .map(|i| i.enabled)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        if ban && targets.iter().any(|t| *t != core.turn.current) {
            return Err(RejectReason::IllegalTargets);
        }
    }
    // 急冻冰封（U05 ow 美，规则 v127）：冰封角色不能被其他角色
    // 的牌指定为目标（上游 jidong2 mod targetEnabled 同口径；
    // AOE 类自动指向全体的牌不受影响——上游同口径；登记差异：
    // 仅出牌使用路径，响应路径同麻茶）
    {
        for t in targets {
            if *t != core.turn.current
                && (core.player(*t).jidong_active
                    || core.player(*t).yinshen_active
                    || core.player(*t).jileng_active
                    || core.player(*t).yinmi_active)
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
    }
    // 盈霸印记（UP37 R03 yingba_mark，哈希 3219705664）：
    // card_target_cancel_if_other_user_card_use_count_lt_owner_mark
    {
        const YINGBA_MARK_RAW: u32 = 3_219_705_664;
        const YINGBA_RAW: u32 = 2_187_983_153;
        let used = core.turn.cards_used_turn as i32;
        let user = core.turn.current;
        let has_mark_skill = |p: PlayerId| {
            catalog
                .skill_by_name("yingba_mark")
                .or_else(|| crate::rvm_ops::resolve_skill_def(catalog, YINGBA_MARK_RAW))
                .map(|def| core.has_skill(p, def))
                .unwrap_or(false)
                || core
                    .vm_aux
                    .flags
                    .get(&(p, format!("skill:{YINGBA_MARK_RAW}")))
                    .copied()
                    .unwrap_or(0)
                    != 0
        };
        let has_yingba_skill = |p: PlayerId| {
            catalog
                .skill_by_name("yingba")
                .or_else(|| crate::rvm_ops::resolve_skill_def(catalog, YINGBA_RAW))
                .map(|def| core.has_skill(p, def))
                .unwrap_or(false)
                || core
                    .vm_aux
                    .flags
                    .get(&(p, format!("skill:{YINGBA_RAW}")))
                    .copied()
                    .unwrap_or(0)
                    != 0
        };
        for &t in targets {
            if t != user {
                let target_mark = core
                    .vm_aux
                    .marks
                    .get(&(t, YINGBA_MARK_RAW))
                    .copied()
                    .unwrap_or(0);
                if has_mark_skill(t) && target_mark > used {
                    return Err(RejectReason::IllegalTargets);
                }
                let user_mark = core
                    .vm_aux
                    .marks
                    .get(&(user, YINGBA_MARK_RAW))
                    .copied()
                    .unwrap_or(0);
                if has_mark_skill(user) && has_yingba_skill(t) && user_mark > used {
                    return Err(RejectReason::IllegalTargets);
                }
            }
        }
    }
    // PA12 批⑭：通用 VM modifier 卡牌使用禁限与目标免疫校验
    {
        let (kind, _) = play_kind(core, catalog, card, targets);
        let nature = match kind {
            CardKind::FireSha => crate::event::DamageNature::Fire,
            CardKind::LeiSha => crate::event::DamageNature::Thunder,
            _ => crate::event::DamageNature::Normal,
        };
        let user = core.turn.current;
        if crate::rvm_bridge::modifiers::eval_card_use_canceled(
            core, catalog, user, card, kind, nature,
        ) {
            return Err(RejectReason::CardLimit);
        }
        for &t in targets {
            if crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core, catalog, user, t, card, kind, nature,
            ) {
                return Err(RejectReason::IllegalTargets);
            }
        }
    }
    Ok(())
}

/// 出牌使用禁限（装备牌已先行装备之后）：冻结者不能用牌、Boss 与持有者技能
/// 使某些牌对特定目标无效（纯只读）。
pub(super) fn check_play_use_restrictions(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    // 紧急冷冻（U05 swd 灵剑，规则 v198）：冻结者不能使用卡牌
    //（上游 hslingjian_jinjilengdong mod cardEnabled 同口径）
    if core.player(core.turn.current).jileng_active {
        return Err(RejectReason::CardLimit);
    }
    // 造异（Boss 颛顼，规则 v246）：玄冥在场 → 敌方不可为锦囊目
    // 标；共工在场 → 敌方不可为基本牌目标（上游 mod targetEnabled
    // 同口径；装备不受限）
    {
        let base = (catalog.general_count() - crate::content::standard::BOSS_GENERALS.len()) as u16;
        let named_alive = |gname: &str| -> bool {
            (base..catalog.general_count() as u16)
                .filter(|&d| {
                    catalog
                        .general_meta(d)
                        .map(|m| m.0 == gname)
                        .unwrap_or(false)
                })
                .any(|gd| {
                    (0..core.players.len() as u8).any(|t| {
                        core.player(t).alive && core.player(t).general_slots[0] == Some(gd)
                    })
                })
        };
        let xuanming_alive = named_alive("玄冥");
        let gonggong_alive = named_alive("共工");
        // 门控绑定造异持有者（上游 mod 挂 boss_zaoyi——非持有者
        // 不受限）
        let zaoyi_holder = catalog
            .skill_by_name(crate::content::standard::BOSS_ZAOYI)
            .map(|def| core.has_skill(core.turn.current, def))
            .unwrap_or(false);
        if (xuanming_alive || gonggong_alive) && zaoyi_holder {
            let is_basic = matches!(
                kind,
                CardKind::Sha
                    | CardKind::FireSha
                    | CardKind::LeiSha
                    | CardKind::Shan
                    | CardKind::Tao
                    | CardKind::Jiu
            );
            let is_equip = kind.is_equip();
            for t in targets {
                if !core.player(*t).alive {
                    continue;
                }
                if core.is_boss_side(*t) != core.is_boss_side(core.turn.current) {
                    if (xuanming_alive && !is_basic && !is_equip) || (gonggong_alive && is_basic) {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
            }
        }
    }
    // 热闷（Boss 都市王，规则 v273）：防具槽空时无属性【杀】
    // 对其无效（上游 useCardToBefore cancel→目标不可指定近似登记；
    // 南蛮/万箭面在 AOE 机窗口前跳过；属性杀不覆盖）
    if kind == CardKind::Sha {
        for t in targets {
            let remen = catalog
                .skill_by_name(crate::content::standard::BOSS_REMEN)
                .map(|def| {
                    core.has_skill(*t, def)
                        && !core.player(*t).equips.iter().any(|c| {
                            catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Armor)
                        })
                })
                .unwrap_or(false);
            if remen && core.player(*t).alive {
                return Err(RejectReason::IllegalTargets);
            }
        }
    }
    // 帷幕（楚江王，规则 v275）：黑色锦囊对持有者无效——显式
    // 目标锦囊（顺手/拆桥/决斗/乐）目标不可指定（上游 mod
    // targetEnabled 同族→目标不可指定近似登记；AOE 面在 AOE 机
    // 窗口前跳过；无指向锦囊无指向面天然不适用=登记）
    if matches!(
        kind,
        CardKind::Snatch | CardKind::Dismantle | CardKind::Duel | CardKind::Le
    ) {
        let black = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        );
        if black {
            let weimu = catalog
                .skill_by_name(crate::content::standard::BOSS_WEIMU)
                .map(|def| {
                    targets
                        .iter()
                        .any(|t| core.player(*t).alive && core.has_skill(*t, def))
                })
                .unwrap_or(false);
            if weimu {
                return Err(RejectReason::IllegalTargets);
            }
        }
    }
    // 判官（秦广王，规则 v276）：延时锦囊对持有者无效（上游 mod
    // targetEnabled delay→目标不可指定近似登记；乐/闪电——兵粮
    // 寸断不在原生目录）
    if matches!(kind, CardKind::Le | CardKind::ShanDian) {
        let pg = catalog
            .skill_by_name(crate::content::standard::BOSS_PANGUAN)
            .map(|def| {
                targets
                    .iter()
                    .any(|t| core.player(*t).alive && core.has_skill(*t, def))
            })
            .unwrap_or(false);
        if pg {
            return Err(RejectReason::IllegalTargets);
        }
        // 无邪（莉莲，swd 武将面批㉙）：锁定——延时锦囊只能以自己
        // 为目标（上游 mod targetEnabled delay target!=self false
        // 同口径）
        let wuxie = catalog
            .skill_by_name("swd_wuxie")
            .map(|def| core.has_skill(core.turn.current, def))
            .unwrap_or(false);
        if wuxie && targets.iter().any(|t| *t != core.turn.current) {
            return Err(RejectReason::IllegalTargets);
        }
    }
    Ok(())
}
