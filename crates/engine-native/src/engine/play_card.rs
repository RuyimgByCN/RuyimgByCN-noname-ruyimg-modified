use super::*;

pub(super) fn play_card(
    state: &mut GameState,
    catalog: &SharedCatalog,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<(), RejectReason> {
    let core = &state.core;
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
    if !core.player(core.turn.current).hand.contains(&card) {
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
                for t in &targets {
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
        for t in &targets {
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
    if kind.is_equip() {
        if !targets.is_empty() {
            return Err(RejectReason::IllegalTargets);
        }
        let player = core.turn.current;
        crate::resolution::push_frame(
            &mut state.resolution,
            EventData::Equipped {
                player,
                card,
                replaced: None,
            },
        );
        return Ok(());
    }
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
            for t in &targets {
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
        for t in &targets {
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
    match kind {
                CardKind::Xuanyuanjian => {
                    // 轩辕剑（swd 批 146）：无目标；装备者须持有「轩辕」或
                    // 体力 >2（上游 enable 同口径）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let has_xuanyuan = catalog
                        .skill_by_name(crate::content::host::SWD_XUANYUAN_KEY)
                        .map(|d| core.has_skill(core.turn.current, d))
                        .unwrap_or(false);
                    if !has_xuanyuan && core.player(core.turn.current).hp <= 2 {
                        return Err(RejectReason::CardLimit);
                    }
                }
                CardKind::Xuejibingbao => {
                    // 雪霁冰雹（swd 卡牌面补切批②）：恰 1 名存活目标且未
                    // 处于标记中（上游 filterTarget 无同标记者同口径）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).xuejibingbao > 0
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shenmiguo => {
                    // 神秘果不可直接使用（经用牌后窗口消耗——窗口机登记
                    // 结构性暂缓，见补切批②证据）
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Lianyaohu
                | CardKind::Qiankundai
                | CardKind::Nvwashi
                | CardKind::Shennongding
                | CardKind::Kunlunjingc
                | CardKind::Donghuangzhong
                | CardKind::Kongdongyin
                | CardKind::Guilingzhitao
                | CardKind::Sifeizhenmian
                | CardKind::Shuchui
                | CardKind::Sadengjinhuan
                | CardKind::Haotianta
                | CardKind::Yuruyi
                | CardKind::Yuchanqian
                | CardKind::Yuchankun
                | CardKind::Yuchanzhen
                | CardKind::Yuchanxun
                | CardKind::Yuchankan
                | CardKind::Yuchanli
                | CardKind::Yuchangen
                | CardKind::Yuchandui
                | CardKind::Shencaojie
                // 器族五件：手持牌（非装备路径；效果经手持触发技）
                | CardKind::Qinglongzhigui
                | CardKind::Zhuquezhizhang
                | CardKind::Baishouzhihu
                | CardKind::Xuanwuzhihuang
                | CardKind::Huanglinzhicong => {
                    // 炼妖壶/乾坤袋/女娲石/神农鼎：装备牌走 equip 校验
                    //（同栏替换由 commit_equip 处理；神农鼎/女娲石效果经
                    // 装备技实例）
                }

                CardKind::Fengyinzhidan => {
                    // 封印之丹：对自身使用（无目标或目标=自己）
                    if targets.len() > 1 || targets.first().map(|t| *t != core.turn.current).unwrap_or(false) {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => { return play_sha(state, catalog, card, targets, kind, mark_mutoumianju); }
                CardKind::Chain => {
                    if targets.is_empty() || targets.len() > 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if targets.iter().any(|t| !core.player(*t).alive) {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 铁索为结算型锦囊：走 CardUsed → 无懈窗口 → ChainMachine
                    // 翻转（旧实现只推 Linked 帧，牌从未离手，ADR 0705 修复）。
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Duel => {
                    // 决斗：恰好 1 名其他存活目标，无距离/射程限制。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 空城（诸葛亮）：同杀
                    let kongcheng = catalog
                        .skill_by_name(crate::content::standard::KONG_CHENG)
                        .map(|d| core.has_skill(target, d))
                        .unwrap_or(false);
                    if kongcheng && core.player(target).hand.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Dismantle => {
                    // 过河拆桥：弃置一名其他存活玩家的一张牌
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            Some(CardKind::Dismantle),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Snatch => {
                    // 顺手牵羊：获得距离 1 的一名其他存活角色的一张牌
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 谦逊（陆逊）：不能成为顺手牵羊目标（锁定门控）
                    let qianxun = catalog
                        .skill_by_name(crate::content::standard::QIAN_XUN)
                        .map(|d| core.has_skill(target, d))
                        .unwrap_or(false);
                    if qianxun {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let from = core.turn.current;
                    // 奇才（黄月英，锁定）：使用锦囊无距离限制
                    let qicai = catalog
                        .skill_by_name(crate::content::standard::QI_CAI)
                        .map(|d| core.has_skill(from, d))
                        .unwrap_or(false);
                    // 神恩（Boss 太昊/炎帝 光环，规则 v238）：boss 方用
                    // 牌无视距离
                    let shenen_boss_free = {
                        let active = catalog
                            .skill_by_name(crate::content::standard::BOSS_SHENEN)
                            .map(|def| core.skills.keys().any(|(_, dd)| *dd == def))
                            .unwrap_or(false);
                        active && core.is_boss_side(from)
                    };
                    // 守翼（年兽守翼形态，规则 v251）：所有牌无视距离
                    let shouyi_free = catalog
                        .skill_by_name(crate::content::standard::BOSS_SHOUYI)
                        .map(|def| core.has_skill(from, def))
                        .unwrap_or(false);
                    // 焚天（Boss 朱雀，规则 v236）：红色牌无视距离
                    let fentian_red_free = {
                        let red = matches!(
                            catalog.cards[card as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        );
                        red
                            && catalog
                                .skill_by_name(crate::content::standard::BOSS_FENTIAN)
                                .map(|def| core.has_skill(from, def))
                                .unwrap_or(false)
                    };
                    // 探囊（U05 xianjian 韩菱纱，规则 v148）：视为的顺手
                    // 牵羊无距离限制（上游 mod targetInRange 同口径；仅
                    // 声明转化命中——实体牌非顺牵即视为路径）
                    let tannang_free = catalog.card_kind(card) != CardKind::Snatch
                        && catalog
                            .skill_by_name(crate::content::host::XJ_TANNANG_KEY)
                            .map(|def| core.has_skill(from, def))
                            .unwrap_or(false);
                    if !qicai
                        && !tannang_free
                        && !fentian_red_free
                        && !shenen_boss_free
                        && !shouyi_free
                        && core.computed_distance(catalog, from, target) > 1
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let t = core.player(target);
                    if t.hand.is_empty() && t.equips.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            Some(CardKind::Snatch),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Le | CardKind::Bing => {
                    // 延时锦囊：置于 1 名其他存活玩家的判定区（不立即生效）；
                    // 判定区不能有同名牌（本批接入的同名限制）。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 无心（Boss 张春华，规则 v260）：锁定——你不能使用延时
                    // 锦囊指定其他角色为目标（上游 mod targetEnabled delay 同
                    // 口径；原生乐/兵路由本就只能置他人→等价持有者不可用乐/
                    // 兵；闪电自置不受限）
                    if let Some(wxdef) = catalog
                        .skill_by_name(crate::content::standard::BOSS_WUXIN)
                    {
                        if core.has_skill(core.turn.current, wxdef) {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    // 谦逊（陆逊）：不能成为乐不思蜀目标（锁定门控；
                    // 上游无顺手牵羊，见覆盖清单）。
                    // G1 墨守宿主原语：延时锦囊目标免疫（乐/兵均免疫）
                    for t in &targets {
                        if catalog.blocks_delayed_trick(core, *t) {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    if kind == CardKind::Le {
                        let qianxun = catalog
                            .skill_by_name(crate::content::standard::QIAN_XUN)
                            .map(|d| core.has_skill(target, d))
                            .unwrap_or(false);
                        if qianxun {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    if core
                        .player(target)
                        .judge
                        .iter()
                        .any(|c| catalog.card_kind(*c) == kind)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            Some(kind),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Longlushanfeng
                | CardKind::Haidao
                | CardKind::Shuimomuxue
                | CardKind::Feixu
                | CardKind::Yixialan
                | CardKind::Shamolvzhou
                | CardKind::Linzhongjianta
                | CardKind::Bingheyaosai
                | CardKind::Youlin
                | CardKind::Lindixiliu => {
                    // 地（mtg 首切片 v100 龙颅山峰/第二批 v101 海岛/第三批
                    // v102 水没墓穴）：无目标（notarget），每回合限一张
                    //（上游 land_used 临时技同口径）；结算在 CardUsed 帧
                    // 内核机器
                    if !targets.is_empty() || core.turn.land_used {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card, player, targets,
                        )),
                    );
                    return Ok(());
                }
                // 无中生有/桃园结义/五谷丰登（P2/N08 缺牌种批①）：无
                // 目标锦囊，结算在 CardUsed 帧内核机器；羊皮卷（swd 第
                // 十三批 v97）对自己使用，同路；地载阵（zhenfa 首切片
                // v110）自动展开非大势力目标，同路。
                | CardKind::Wuzhong
                | CardKind::Taoyuan
                | CardKind::Wugu
                | CardKind::Yangpijuan
                | CardKind::Dizaizhen
                | CardKind::Changshezhen
                | CardKind::Pozhenjue
                | CardKind::ShanDian
                | CardKind::HongShui => {
                    // 长蛇阵：须处于队列或存在友方（上游 enable 同口径）；
                    // 破阵诀：恒可用（enable true）
                    if kind == CardKind::Changshezhen
                        && !core.inline(catalog, core.turn.current)
                        && !core
                            .alive_players()
                            .any(|p| {
                                p != core.turn.current
                                    && core.is_friend_of(catalog, core.turn.current, p)
                            })
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 地载阵：须存在非大势力存活角色（上游 enable 同口径）
                    if kind == CardKind::Dizaizhen
                        && !core
                            .alive_players()
                            .any(|p| !core.is_major_faction(catalog, p))
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 闪电/洪水：只能置入自己判定区；判定区不能有同名牌
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    if core
                        .player(player)
                        .judge
                        .iter()
                        .any(|c| catalog.card_kind(*c) == kind)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 良策转化（玩点论杀批 15）：基本牌当五谷——use_as 声明
                    // 必须随帧（否则机器工厂回退目录种）。
                    let use_as =
                        (kind != catalog.card_kind(card)).then_some(kind);
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card, player, targets, use_as,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Tianfuzhen | CardKind::Qixingzhen | CardKind::Niaoxiangzhen => {
                    // 阵法三张（U05 zhenfa，规则 v200/T19a；上游
                    // guozhanZhenfaEnabled=存活≥4 同口径；派生查询见
                    // state.rs）。天覆阵=AOE 全部大势力且有牌者；七星阵
                    // =自目标（须围攻/被围攻）；鸟翔阵=AOE 全部合法目
                    // 标（异势力、已知、与邻座同势力）。
                    if core.players.len() < 4 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    match kind {
                        CardKind::Tianfuzhen => {
                            if !core.alive_players().any(|p| core.is_major_faction(catalog, p)) {
                                return Err(RejectReason::IllegalTargets);
                            }
                            let expected: Vec<u8> = core
                                .alive_players()
                                .filter(|p| {
                                    core.is_major_faction(catalog, *p)
                                        && (!core.player(*p).hand.is_empty()
                                            || !core.player(*p).equips.is_empty())
                                })
                                .collect();
                            if targets != expected {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                        CardKind::Qixingzhen => {
                            let cur = core.turn.current;
                            let under_siege = core.alive_players().any(|q| {
                                core.sieges(catalog, q, cur)
                            });
                            let besieging = core.alive_neighbors(cur).0.is_some_and(|n| {
                                core.sieged(catalog, n)
                            }) || core.alive_neighbors(cur).1.is_some_and(|pv| {
                                core.sieged(catalog, pv)
                            });
                            if !(under_siege || besieging) || targets != [cur] {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                        _ => {
                            // 鸟翔阵：targets=全部合法目标（异势力、已知、
                            // 与自身邻座同势力）
                            let expected: Vec<u8> = core
                                .alive_players()
                                .filter(|t| {
                                    *t != core.turn.current
                                        && !core.is_friend_of(catalog, core.turn.current, *t)
                                        && {
                                            let (n, pv) = core.alive_neighbors(*t);
                                            n.is_some_and(|q| {
                                                core.effective_faction(catalog, *t)
                                                    == core.effective_faction(catalog, q)
                                            }) || pv.is_some_and(|q| {
                                                core.effective_faction(catalog, *t)
                                                    == core.effective_faction(catalog, q)
                                            })
                                        }
                                })
                                .collect();
                            if targets != expected {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                    }
                }
                CardKind::Jiedao => {
                    // 持有者装备区须有武器，另一角色在其攻击范围内。
                    if targets.len() != 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let (holder, victim) = (targets[0], targets[1]);
                    if holder == core.turn.current
                        || victim == core.turn.current
                        || holder == victim
                        || !core.player(holder).alive
                        || !core.player(victim).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let has_weapon = core
                        .player(holder)
                        .equips
                        .iter()
                        .any(|c| catalog.card_kind(*c) == CardKind::Weapon);
                    let in_range = core.computed_distance(catalog, holder, victim)
                        <= core.attack_range(catalog, holder);
                    if !has_weapon || !in_range {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::SavageAttack
                | CardKind::ArcheryAttack
                | CardKind::Jingleishan
                | CardKind::Chiyuxi => {
                    // AOE：目标 = 出牌时刻的全部其他存活玩家（自下家起按座位
                    // 顺序冻结），命令显式 targets 必须为空。惊雷山/炽羽袭
                    // reverseOrder：结算序反转为从自己上家起逆座位序。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    let mut aoe_targets: Vec<PlayerId> = core
                        .seat_order_from(core.next_alive(player))
                        .into_iter()
                        .filter(|p| *p != player)
                        .collect();
                    if matches!(kind, CardKind::Jingleishan | CardKind::Chiyuxi) {
                        aoe_targets.reverse();
                    }
                    // viewAs 转化（万钧装备牌当南蛮等）须随帧声明——工厂按
                    // 声明种路由 AOE 机，实体种会把机路由丢空
                    let declared =
                        if kind != catalog.card_kind(card) {
                            Some(kind)
                        } else {
                            None
                        };
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card, player, aoe_targets, declared,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Guisheqi => {
                    // 鬼蛇旗：恰 1 名任意存活目标（可含自己）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Xianluhui => {
                    // 仙露回：AOE 全体受伤角色（含自己，reverseOrder 逆序近似
                    // 为逆座位序），命令显式 targets 必须为空。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    let mut aoe_targets: Vec<PlayerId> = core
                        .seat_order_from(core.next_alive(player))
                        .into_iter()
                        .filter(|p| {
                            *p != player && core.player(*p).alive && core.player(*p).hp < core.player(*p).max_hp as i32
                        })
                        .collect();
                    if core.player(player).hp < core.player(player).max_hp as i32 {
                        aoe_targets.push(player);
                    }
                    aoe_targets.reverse();
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card, player, aoe_targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::HorsePlus
                | CardKind::HorseMinus
                | CardKind::Weapon
                | CardKind::Armor
                | CardKind::Suolianjia
                | CardKind::WuxingPan
                | CardKind::Sanjian
                // 猴子/写轮眼/酒尾（E03）：equip5 宝物，equip 路径按 is_equip
                // 提前处理。
                | CardKind::Monkey
                | CardKind::Xuelunyang
                | CardKind::Jiuwei
                | CardKind::Mianju
                | CardKind::Renwang
                | CardKind::Qinggang
                | CardKind::Qilin
                | CardKind::Hanbing
                | CardKind::Zhuge
                | CardKind::Guanshi
                | CardKind::Qinglong
                | CardKind::Zhangba
                | CardKind::Cixiong
                | CardKind::Fangtian
                | CardKind::Chilongya
                | CardKind::Xixueguizhihuan
                | CardKind::Shentoumianju
                | CardKind::Xingjunyan
                | CardKind::Qinglonglingzhu => {
                    // 占栏装备：equip 路径在上方按 is_equip 提前处理；
                    // 此臂仅为穷尽性——正常不会到达
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Guiyoujie => {
                    // 归有界（U05 swd，规则 v91）：他人延时锦囊——恰 1 名
                    // 其他存活目标，且其判定区无同名延时
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || targets[0] == core.turn.current
                        || core.player(targets[0])
                            .judge
                            .iter()
                            .any(|c| catalog.card_kind(*c) == CardKind::Guiyoujie)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zhufangshenshi => {
                    // 贴房绅视（U05 swd 第十批，规则 v94）：恰 1 名其他
                    // 存活目标（上游 filterTarget target !== player）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || targets[0] == core.turn.current
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Fengxueren
                | CardKind::Yufulu
                | CardKind::Daihuofenglun
                | CardKind::Xiayuncailing
                | CardKind::Langeguaiyi => {
                    // 装备牌（风雪刃 equip1/御夫录·蓝格怪衣 equip5/带火
                    // 风轮 equip4/霞云彩绫 equip3）：走装配路径（is_equip
                    // 提前处理），出牌区使用为穷尽性占位拒绝
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Caoyao => {
                    // 草药（U05 swd 第三批，规则 v87）：为距离 1 以内的
                    // 受伤其他目标回复 1 点体力（上游 range.global 1 +
                    // filterTarget hp<maxHp + recover）。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let t = targets[0];
                    let tp = core.player(t);
                    if !tp.alive || tp.hp >= tp.max_hp as i32 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let user = core.turn.current;
                    if core.computed_distance(catalog, user, t) > 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Tao | CardKind::Hufu => {
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let user = core.turn.current;
                    // 覆泰2（Boss 穷奇，规则 v214）：全场禁桃时非穷奇玩
                    // 家非穷奇回合不能用桃（上游 cardEnabled 同口径；护
                    // 符濒死专用不受此 mod）
                    if kind == CardKind::Tao
                        && crate::kernel::tao_futai2_banned(core, catalog, user)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let p = core.player(user);
                    // 护符（U05 swd 首切片，规则 v85）：仅濒死（自己进入
                    // 濒死流程）时可用；非濒死拒绝（上游 savable 守卫）
                    if kind == CardKind::Hufu && !p.dying {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if p.hp >= p.max_hp as i32 {
                        return Err(RejectReason::HpFull);
                    }
                    // E03 猴子偷桃：他人装备猴子时开「使用前取消窗口」
                    // （命令层挂起机器，ActivateSkill 槽模式）——确认者弃
                    // 猴子把桃收为手牌（使用不发生，无 CardUsed 帧）；
                    // 全部放弃则按普通桃结算。
                    let monkey_def = catalog
                        .skill_by_name(crate::content::host::HUANLE_MONKEY_KEY);
                    let holders: Vec<u8> = monkey_def
                        .map(|d| {
                            core.skills
                                .range(..)
                                .filter(|((o, idef), inst)| {
                                    *idef == d && inst.enabled && core.player(*o).alive && *o != user
                                })
                                .map(|((o, _), _)| *o)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if !holders.is_empty() {
                        state.resolution.parks.push(crate::resolution::Park {
                            resume_depth: state.resolution.stack.len(),
                            slot: crate::resolution::Slot::Intrinsic {
                                frame: crate::resolution::NO_FRAME,
                            },
                            machine: Box::new(crate::kernel::MonkeyStealMachine::new(
                                card, user, holders,
                            )),
                        });
                        return Ok(());
                    }
                }
                CardKind::Jiu | CardKind::Tianxianjiu => {
                    // 酒/天仙酒（swd 变体，规则 v89；登记差异：与酒共享
                    // 每回合一次计数）：出牌阶段对自身使用，每回合限一次
                    // ——本回合使用的下一张杀伤害 +1；濒死使用走濒死救援
                    // 通道（不经此命令）。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 焚天（Boss 朱雀，规则 v236）：红色酒不受每回合一次
                    // 限制（上游 mod cardUsable Infinity 同口径）
                    let fentian_red_jiu = {
                        let red = matches!(
                            catalog.cards[card as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        );
                        red
                            && catalog
                                .skill_by_name(crate::content::standard::BOSS_FENTIAN)
                                .map(|def| core.has_skill(core.turn.current, def))
                                .unwrap_or(false)
                    };
                    if core.turn.jiu_used && !fentian_red_jiu {
                        return Err(RejectReason::CardLimit);
                    }
                }
                CardKind::ShouLiJian => {
                    // 手里剑（G1 欢乐卡牌）：恰 1 名其他存活目标（无距离限制，
                    // 上游 outrange global）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                // 闪/无懈可击：仅在响应窗口打出，不可主动使用
                CardKind::Shan | CardKind::Nullify => return Err(RejectReason::WrongPhase),
                CardKind::Shuigong => {
                    // 水攻：所有有装备的存活角色（多目标）
                    for t in &targets {
                        if !core.player(*t).alive || core.player(*t).equips.is_empty() {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
                CardKind::Fudichouxin => {
                    // 釜底抽薪：恰 1 名其他存活目标，双方均有手牌（可拼点）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                        || core.player(core.turn.current).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Chenhuodajie => {
                    // 趁火打劫（G1 运筹帷幄）：恰 1 名其他存活且有手牌的目标
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Linghunzhihuo => {
                    // 灵魂之火：恰 1 名存活目标（可含自己）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zhaomingdan => {
                    // 照明弹：恰 1 名其他存活且有手牌的目标
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Jihuocard => {
                    // 急耗卡：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Youdishenru => {
                    // 诱敌深入：仅作为杀的响应打出（响应窗口内），不可主动使用
                    return Err(RejectReason::WrongPhase);
                }
                CardKind::Caochuanjiejian => {
                    // 草船借箭：恰 1 名其他存活且有手牌的目标
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shezhanqunru => {
                    // 舌战群儒：自-target；目标=所有有手牌的其他角色
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Geanguanhuo => {
                    // 隔岸观火：恰 2 名其他存活且有手牌的目标
                    if targets.len() != 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    for t in &targets {
                        if *t == core.turn.current
                            || !core.player(*t).alive
                            || core.player(*t).hand.is_empty()
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
                CardKind::Toulianghuanzhu => {
                    // 偷梁换柱（G1 运筹帷幄）：恰 1 名其他存活且有手牌的目标；
                    // 己方也需有手牌（各选一张交换）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.player(target).hand.is_empty()
                        || core.player(core.turn.current).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::WangmeiZhike => {
                    // 望梅止渴（G1 玩点论杀）：恰 1 名存活目标（可含自己）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shujinsan => {
                    // 舒筋散（G1 古剑）：恰 1 名存活且有手牌的目标（可含自己；
                    // 登记差异：上游含装备区，原生仅手牌）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).hand.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Mianlijinzhen => {
                    // 面里金针（G1 古剑）：恰 1 名体力值不小于使用者的存活目标
                    //（可含自己）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).hp < core.player(core.turn.current).hp
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Feibiao => {
                    // 飞镖（G1 古剑）：恰 1 名其他存活目标（无距离限制）
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Jinlianzhu => {
                    // 金莲烛（G1 古剑）：响应牌，不能主动使用——仅经
                    // 伤害前置监听技打出
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Ziyangdan => {
                    // 紫阳丹：恰 1 名存活目标（可含自己；可叠加）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shihuifen | CardKind::Liufengsan => {
                    // 石会粉/六风散：恰 1 名其他存活目标
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shatang | CardKind::Liutouge => {
                    // 砂糖/硫头阁：恰 1 名存活目标（可含自己）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Heilonglinpian => {
                    // 黑龙鳞片：自-target（对自己使用）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Mutoumianju => {
                    // 木头面具：装备牌，无目标（走装备路径）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Dinvxuanshuang => {
                    // 帝女玄霜：无目标、对自己使用、需已受伤（桃同款校验）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let p = core.player(core.turn.current);
                    if p.hp >= p.max_hp as i32 {
                        return Err(RejectReason::HpFull);
                    }
                }
                CardKind::Gjyuheng => {
                    // 玉衡：装备牌，无目标（走装备路径）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Dunpaigedang
                | CardKind::Xingjiegoutong
                | CardKind::Chuansongmen
                | CardKind::Tanshezhiren => {
                    // 炉石自-target 锦囊：无目标
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zhiliaobo => {
                    // 治疗波：恰 1 名已受伤的存活目标（可含自己）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).hp >= core.player(targets[0]).max_hp as i32
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Yuansuhuimie => {
                    // 元素毁灭：自-target（AOE：所有其他存活角色）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shijieshu => {
                    // 世界树：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Kunenfayin | CardKind::Poxiao => {
                    // 昆恩/破晓：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zumoshoukao => {
                    // 祖墨手铠：恰 1 名其他存活目标
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shanbengshu => {
                    // 山崩术：自-target
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Yanziyaoshui
                | CardKind::Aozuzhilei
                | CardKind::Wenyi
                | CardKind::Zhihuanjun
                | CardKind::Baoxueyaoshui
                | CardKind::Liuxinghuoyu => {
                    // gwent 法术：恰 1 名存活目标
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Jiguan => {
                    // 机关牌（swd 第十六~十八批，规则 v189/v190/v191）：
                    // 指路小虎/木甲人=自目标（上游 selectTarget -1+
                    // filterTarget target===player 同口径）；木甲人
                    // usable:1 每回合一次；勾魂锣=恰 1 名无挂附目标
                    // （上游 filterTarget !hasSkill("gouhunluo") 同口径）
                    if catalog.cards[card as usize].name.contains("jiguantong") {
                        // 机关桶：AOE=全部其他存活角色（上游 selectTarget
                        // -1 target!==player 同口径；结算序=座位序登记差异）
                        let expected: Vec<u8> = core
                            .seat_order_from(core.turn.current)
                            .into_iter()
                            .filter(|p| *p != core.turn.current && core.player(*p).alive)
                            .collect();
                        if targets != expected {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("gouhunluo") {
                        if targets.len() != 1 || !core.player(targets[0]).alive {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if core.player(targets[0]).gouhunluo_rounds > 0 {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanfeng") {
                        // 机关凤（swd 第二十批，规则 v195）：恰 1 名其他
                        // 角色（上游 filterTarget target!==player 同口径）
                        if targets.len() != 1
                            || targets[0] == core.turn.current
                            || !core.player(targets[0]).alive
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanyaoshu") {
                        // 机关妖术（swd 第二十二批，规则 v197）：恰 1 名其
                        // 他存活角色（range.global 1 无距离约束）；目标有空
                        // 装备栏（上游 filterTarget 空 subtype 槽同口径）、
                        // 无防重标记（上游 !hasSkill 同口径）、非 isMin
                        //（手牌数>体力，上游 !isMin 同口径）
                        if targets.len() != 1 || !core.player(targets[0]).alive {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let t = targets[0];
                        let marker = catalog
                            .skill_by_name(crate::content::host::SWD_JIGUANYAOSHU_STATE_KEY);
                        if marker.is_some_and(|d| core.skills.contains_key(&(t, d))) {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let hand = core.player(t).hand.len() as i32;
                        if hand <= core.player(t).hp {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let occupied: Vec<Option<crate::catalog::EquipSlot>> = core
                            .player(t)
                            .equips
                            .iter()
                            .map(|c| catalog.card_slot_of(core, *c))
                            .collect();
                        if occupied.len() >= 5 {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanshu") {
                        // 机关术（swd 机关术批，规则 v199/ADR 0780）：自目
                        // 标（上游 filterTarget target===player 同口径）；
                        // 至少一张有效装备
                        if targets.len() != 1 || targets[0] != core.turn.current {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if core.player(core.turn.current).equips.is_empty() {
                            return Err(RejectReason::IllegalTargets);
                        }
                    } else if catalog.cards[card as usize].name.contains("jiguanyuan") {
                        // 机关怨（swd 第二十一批，规则 v196）：恰 1 名其他
                        // 角色（上游 filterTarget target!==player+
                        // !hasSkill("jiguanyuan") 同口径）；enable 要求除本
                        // 体至少还有一张手牌（上游 enable 同口径）
                        if targets.len() != 1
                            || targets[0] == core.turn.current
                            || !core.player(targets[0]).alive
                            || !core.player(targets[0]).jiguanyuan_cards.is_empty()
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let hand = core.player(core.turn.current).hand.clone();
                        if hand.is_empty()
                            || (hand.len() == 1 && hand[0] == card)
                        {
                            return Err(RejectReason::CardLimit);
                        }
                    } else {
                        if targets != [core.turn.current] {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if catalog.cards[card as usize].name.contains("mujiaren")
                            && core.turn.mujiaren_used
                        {
                            return Err(RejectReason::CardLimit);
                        }
                    }
                }
                CardKind::Hslingjian => {
                    // 灵剑（swd 机关术批，规则 v198）：逐卡目标规则（上游
                    // 各卡 filterTarget 同口径）
                    let cname = &catalog.cards[card as usize].name;
                    let t_ok = if cname.contains("xingtigaizao") {
                        targets.len() == 1 && targets[0] == core.turn.current
                    } else if cname.contains("shijianhuisu") {
                        targets.len() == 1
                            && targets[0] != core.turn.current
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).equips.is_empty()
                    } else if cname.contains("jinjilengdong") {
                        targets.len() == 1
                            && targets[0] != core.turn.current
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).face_down
                    } else if cname.contains("yinmilichang") {
                        targets.len() == 1
                            && targets[0] != core.turn.current
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).yinmi_active
                            && !core.player(targets[0]).yinshen_active
                    } else if cname.contains("shengxiuhaojiao") {
                        targets.len() == 1
                            && core.player(targets[0]).alive
                            && !core.player(targets[0]).chaofeng_active
                    } else if cname.contains("zhongxinghujia") {
                        targets.len() == 1
                            && core.player(targets[0]).alive
                            && core.player(targets[0]).hand.len() as i32
                                > core.player(targets[0]).hp
                    } else {
                        targets.len() == 1
                            && core.player(targets[0]).alive
                            && (!core.player(targets[0]).hand.is_empty()
                                || !core.player(targets[0]).equips.is_empty())
                    };
                    if !t_ok {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Qinpendayu | CardKind::Birinongwu | CardKind::Ciguhanshuang => {
                    // gwent 天气：恰 1 名其他存活目标（邻座结算侧展开；
                    // 已持有状态的目标由授予机跳过）
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Bingpotong => {
                    // 冰魄通：1–3 名其他存活且有手牌的目标
                    if targets.is_empty() || targets.len() > 3 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    for t in &targets {
                        if *t == core.turn.current
                            || !core.player(*t).alive
                            || core.player(*t).hand.is_empty()
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
                CardKind::Qiankunbiao | CardKind::Shenhuofeiya => {
                    // 乾坤镖/神火飞鸦：恰 1 名其他存活目标（邻座由结算
                    // 侧按距离展开）
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }

                CardKind::Longxugou => {
                    // 龙须钩：恰 1 名其他存活且有装备的目标
                    if targets.len() != 1
                        || targets[0] == core.turn.current
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).equips.is_empty()
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Tanhuadong
                | CardKind::Yougeng
                | CardKind::Qingtuan
                | CardKind::Yuanbaorou
                | CardKind::Gudonggeng
                | CardKind::Mapodoufu
                | CardKind::Chunbing
                | CardKind::Liyutang
                | CardKind::Molicha
                | CardKind::Xiajiao
                | CardKind::Mizhilianou
                | CardKind::Luyugeng
                | CardKind::Yunvyuanshen => {
                    // 古剑食物：恰 1 名存活目标（可含自己），且目标尚
                    // 未持有同名状态（登记差异：上游按技能名过滤）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let listener = match kind {
                        CardKind::Tanhuadong => crate::content::host::GJ_FOOD_TANHUADONG_KEY,
                        CardKind::Qingtuan => crate::content::host::GJ_FOOD_QINGTUAN_KEY,
                        CardKind::Yuanbaorou => crate::content::host::GJ_FOOD_YUANBAOROU_KEY,
                        CardKind::Gudonggeng => crate::content::host::GJ_FOOD_GUDONGGENG_KEY,
                        CardKind::Mapodoufu => crate::content::host::GJ_FOOD_MAPODOUFU_KEY,
                        CardKind::Chunbing => crate::content::host::GJ_FOOD_CHUNBING_KEY,
                        CardKind::Liyutang => crate::content::host::GJ_FOOD_LIYUTANG_KEY,
                        CardKind::Molicha => crate::content::host::GJ_FOOD_MOLICHA_KEY,
                        CardKind::Xiajiao => crate::content::host::GJ_FOOD_XIAJIAO_KEY,
                        CardKind::Mizhilianou => crate::content::host::GJ_FOOD_MIZHLIANOU_KEY,
                        CardKind::Luyugeng => crate::content::host::GJ_FOOD_LUYUGENG_KEY,
                        CardKind::Yunvyuanshen => crate::content::host::GJ_YUNVYUANSHEN_LISTENER_KEY,
                        _ => crate::content::host::GJ_FOOD_YOUGENG_KEY,
                    };
                    if let Some(def) = catalog.skill_by_name(listener) {
                        if core.skills.contains_key(&(targets[0], def)) {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                }
            }
    let player = core.turn.current;
    // view-as 声明（蜜链藕红桃当桃等）：转化后按声明牌种结算
    let use_as = if kind != catalog.card_kind(card) {
        Some(kind)
    } else {
        None
    };
    // 还侠跟踪（swd 屈娴，批 27）：还侠转化的杀入跟踪列（是否造
    // 成伤害由 commit_damage 置位；阶段末回收未造成伤害者）
    if use_as == Some(CardKind::Sha)
        && catalog
            .skill_by_name("huanxia")
            .map(|d| state.core.has_skill(state.core.turn.current, d))
            .unwrap_or(false)
    {
        state.core.turn.huanxia_track.push((card, false));
    }
    crate::resolution::push_frame(
        &mut state.resolution,
        EventData::CardUsed(crate::machine::card_use_as(card, player, targets, use_as)),
    );
    Ok(())
}
