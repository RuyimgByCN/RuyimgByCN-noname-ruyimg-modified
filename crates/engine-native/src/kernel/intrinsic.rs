use super::*;

/// 帧固有流程机器（`Intrinsic` 阶段启动）。
pub fn intrinsic_machine(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let data = ctx.res.frame(frame_id).expect("frame").data.clone();
    match data {
        EventData::LoseHp { .. } => None,
        EventData::DyingSaved { .. } => None,
        EventData::Dying { player, source } => Some(Box::new(DyingSaveMachine {
            dying: player,
            source,
            responders: ctx.core.seat_order_from(ctx.core.turn.current),
            asked: false,
            round_used: false,
            guihan_asked: false,
        })),
        EventData::CardUsed(u) => {
            // 筑城守卫（U07 批三，规则 v113）等 AfterTriggers 机器写回的
            // 取消：跳过结算（杀已由守卫归还手牌）
            if u.cancelled {
                return None;
            }
            let effective_kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
            // 目标被结算窗全部移出（贞烈等「移出目标」类，swd 批 139）：
            // 需要「恰一名目标」的牌种在目标清空后不结算（避免机器对
            // 空目标越界）；无目标/群体牌种照常（其机器自行处理空列表）
            if u.targets.is_empty()
                && matches!(
                    effective_kind,
                    CardKind::Sha
                        | CardKind::FireSha
                        | CardKind::LeiSha
                        | CardKind::Duel
                        | CardKind::Dismantle
                        | CardKind::Snatch
                        | CardKind::Le
                        | CardKind::Bing
                        | CardKind::Jiedao
                        | CardKind::Chain
                )
            {
                return None;
            }
            match effective_kind {
                CardKind::Dismantle => {
                    let effect = DismantleMachine {
                        card: u.card,
                        source: u.player,
                        target: *u.targets.first().expect("dismantle has one target"),
                        asked: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                // 无中生有（P2/N08 缺牌种批①）：使用后摸 2（无懈窗口包装）。
                CardKind::Wuzhong => Some(Box::new(wrap_nullify_window(
                    ctx,
                    u.card,
                    u.player,
                    Box::new(WuzhongMachine {
                        owner: u.player,
                        card: Some(u.card),
                    }),
                ))),
                // 桃园结义（P2/N08 缺牌种批①）：所有其他存活角色回复 1 点
                // 体力（无懈窗口包装）。
                CardKind::Taoyuan => Some(Box::new(wrap_nullify_window(
                    ctx,
                    u.card,
                    u.player,
                    Box::new(TaoyuanMachine {
                        owner: u.player,
                        card: Some(u.card),
                    }),
                ))),
                // 五谷丰登（批①-2）：亮存活角色数的牌，从使用者起按座位序
                // 逐人择一（无懈窗口包装）。
                CardKind::Wugu => Some(Box::new(wrap_nullify_window(
                    ctx,
                    u.card,
                    u.player,
                    Box::new(WuguMachine {
                        owner: u.player,
                        cards: Vec::new(),
                        pickers: Vec::new(),
                        stage: WuguStage::Init,
                    }),
                ))),
                // 借刀杀人（批①-2）：武器持有者出杀（登记差异：直接伤害
                // 结算，无出闪窗口）或交出武器（无懈窗口包装）。
                CardKind::Jiedao => {
                    let holder = *u.targets.first().expect("jiedao two targets");
                    let victim = *u.targets.get(1).expect("jiedao two targets");
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(JiedaoMachine {
                            owner: u.player,
                            holder,
                            victim,
                            card: Some(u.card),
                            stage: JiedaoStage::Ask,
                        }),
                    )))
                }
                CardKind::Snatch => {
                    let effect = SnatchMachine {
                        card: u.card,
                        source: u.player,
                        target: *u.targets.first().expect("snatch has one target"),
                        asked: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
                    // 方天画戟（批③-b 第九把）：多目标队列（首目标即期结算，
                    // 其余入队列逐目标推进）。
                    let mut it = u.targets.iter().copied();
                    let first = it.next().expect("sha has one target");
                    let extra: Vec<PlayerId> = it.collect();
                    Some(Box::new(ShaResolveMachine {
                        card: u.card,
                        source: u.player,
                        target: first,
                        nature: u.nature,
                        asked: false,
                        armor_judged: false,
                        dodges_used: 0,
                        tieji_red: None,
                        liuli_done: false,
                        liuli_stage: 0,
                        liuli_card: None,
                        asking_qilin: false,
                        qilin_horse: None,
                        asking_hanbing: false,
                        asking_fengxueren: false,
                        asking_guanshi: false,
                        guanshi_cards: Vec::new(),
                        asking_qinglong: false,
                        asking_cixiong: false,
                        cixiong_done: false,
                        hanbing_pending: 0,
                        asking_xiandan: false,
                        xiandan_done: false,
                        xiandan_bonus: false,
                        asking_lueying: false,
                        lueying_victim: None,
                        lueying_declined: false,
                        asking_fuchou: false,
                        fuchou_card: None,
                        asking_sadeng: false,
                        asking_sadeng_shan: false,
                        shoulie_direct: false,
                        extra_targets: extra,
                        hujia_queue: None,
                        hujia_holder: None,
                        // 酒效果在杀派发（=使用）时捕获并消耗——「下一张杀」
                        jiu_bonus: std::mem::take(&mut ctx.core.turn.jiu_active),
                    }))
                }
                CardKind::Shuigong => {
                    // 水攻（G1 运筹帷幄，锦囊）：无懈窗口 + 多目标强制弃装备
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::WD_SHUIGONG_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::ShuigongCard,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: u.targets.clone(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Shezhanqunru => {
                    // 舌战群儒（G1 运筹帷幄，锦囊）：与所有有手牌的其他角色拼点
                    //（登记差异：确定性首轮拼点，简化多轮 Bool 链）
                    let targets: Vec<u8> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != u.player && !ctx.core.player(*p).hand.is_empty())
                        .collect();
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::WD_SHEZHAN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::ShezhanqunruCompare,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets,
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Shujinsan => {
                    // 舒筋散（G1 古剑，锦囊）：目标弃任意张手牌（可取消）摸等量
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::GJ_SHUJINSAN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::GujianShujinsanCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("shujinsan has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::SelectCard,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
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
                | CardKind::Ziyangdan
                | CardKind::Yunvyuanshen => {
                    // 古剑食物（G1）：授予目标监听技实例并弃置食物牌
                    //（上游食物无 wuxieable，无懈窗口不包）
                    let key = match effective_kind {
                        CardKind::Tanhuadong => crate::content::host::GJ_TANHUADONG_CARD_KEY,
                        CardKind::Qingtuan => crate::content::host::GJ_QINGTUAN_CARD_KEY,
                        CardKind::Yuanbaorou => crate::content::host::GJ_YUANBAOROU_CARD_KEY,
                        CardKind::Gudonggeng => crate::content::host::GJ_GUDONGGENG_CARD_KEY,
                        CardKind::Mapodoufu => crate::content::host::GJ_MAPODOUFU_CARD_KEY,
                        CardKind::Chunbing => crate::content::host::GJ_CHUNBING_CARD_KEY,
                        CardKind::Liyutang => crate::content::host::GJ_LIYUTANG_CARD_KEY,
                        CardKind::Molicha => crate::content::host::GJ_MOLICHA_CARD_KEY,
                        CardKind::Xiajiao => crate::content::host::GJ_XIAJIAO_CARD_KEY,
                        CardKind::Mizhilianou => crate::content::host::GJ_MIZHLIANOU_CARD_KEY,
                        CardKind::Luyugeng => crate::content::host::GJ_LUYUGENG_CARD_KEY,
                        CardKind::Bingpotong => crate::content::host::GJ_BINGPOTONG_CARD_KEY,
                        CardKind::Qiankunbiao => crate::content::host::GJ_QIANKUNBIAO_CARD_KEY,
                        CardKind::Longxugou => crate::content::host::GJ_LONGXUGOU_CARD_KEY,
                        CardKind::Shenhuofeiya => crate::content::host::GJ_SHENHUOFEIYA_CARD_KEY,
                        CardKind::Ziyangdan => crate::content::host::GJ_ZIYANGDAN_CARD_KEY,
                        CardKind::Yunvyuanshen => crate::content::host::GJ_YUNVYUANSHEN_CARD_KEY,
                        _ => crate::content::host::GJ_YOUGENG_CARD_KEY,
                    };
                    let behavior = crate::content::host::HostBehavior::GujianFoodGrant;
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("food has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Dinvxuanshuang => {
                    // 帝女玄霜（G1 古剑）：主动使用——回复 1 + 弃任意摸等量
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::GJ_DINVXUANSHUANG_CARD_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::JiguanDinvCard,
                        owner: u.player,
                        target: Some(u.player),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Qinpendayu | CardKind::Birinongwu | CardKind::Ciguhanshuang => {
                    // gwent 天气（G1）：目标 + 距离 1 邻座（座位序，已持有
                    // 者授予时跳过）；无懈窗口包装
                    let t = *u.targets.first().expect("weather has one target");
                    let mut q = vec![t];
                    for p in ctx.core.alive_players() {
                        if p != t && ctx.core.computed_distance(ctx.catalog, t, p) == 1 {
                            q.push(p);
                        }
                    }
                    let (key, behavior) = match effective_kind {
                        CardKind::Qinpendayu => (
                            crate::content::host::GW_QINPEN_CARD_KEY,
                            crate::content::host::HostBehavior::GwentWeatherGrant,
                        ),
                        CardKind::Birinongwu => (
                            crate::content::host::GW_BIRIN_CARD_KEY,
                            crate::content::host::HostBehavior::GwentWeatherGrant,
                        ),
                        _ => (
                            crate::content::host::GW_CIGU_CARD_KEY,
                            crate::content::host::HostBehavior::GwentWeatherGrant,
                        ),
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: q,
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Zumoshoukao => {
                    // 祖墨手铠（G1 gwent）：无懈窗口 + 封印授予机
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::GW_ZUMO_CARD_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::GwentZumoCard,
                        owner: u.player,
                        target: u.targets.first().copied(),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Kunenfayin | CardKind::Shanbengshu | CardKind::Poxiao => {
                    // gwent 收官（G1）：无懈窗口包装 + 各自机器
                    let (key, behavior) = match effective_kind {
                        CardKind::Kunenfayin => (
                            crate::content::host::GW_KUNEN_CARD_KEY,
                            crate::content::host::HostBehavior::GwentKunenCard,
                        ),
                        CardKind::Shanbengshu => (
                            crate::content::host::GW_SHANBENG_CARD_KEY,
                            crate::content::host::HostBehavior::GwentShanbengCard,
                        ),
                        _ => (
                            crate::content::host::GW_POXIAO_CARD_KEY,
                            crate::content::host::HostBehavior::GwentPoxiaoCard,
                        ),
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target: Some(u.player),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Yanziyaoshui
                | CardKind::Aozuzhilei
                | CardKind::Wenyi
                | CardKind::Zhihuanjun
                | CardKind::Baoxueyaoshui => {
                    // gwent 法术（G1）：无懈窗口包装 + 各自机器
                    let (key, behavior) = match effective_kind {
                        CardKind::Yanziyaoshui => (
                            crate::content::host::GW_YANZHI_CARD_KEY,
                            crate::content::host::HostBehavior::GwentYanzhiCard,
                        ),
                        CardKind::Aozuzhilei => (
                            crate::content::host::GW_AOZU_CARD_KEY,
                            crate::content::host::HostBehavior::GwentAozuCard,
                        ),
                        CardKind::Wenyi => (
                            crate::content::host::GW_WENYI_CARD_KEY,
                            crate::content::host::HostBehavior::GwentWenyiCard,
                        ),
                        CardKind::Zhihuanjun => (
                            crate::content::host::GW_ZIHUAN_CARD_KEY,
                            crate::content::host::HostBehavior::GwentZhihuanCard,
                        ),
                        _ => (
                            crate::content::host::GW_BAOXUE_CARD_KEY,
                            crate::content::host::HostBehavior::GwentBaoxueCard,
                        ),
                    };
                    // 暴雪药水先开目标选牌询问（逐技对照批五 v79），其余直接执行
                    let stage = if behavior == crate::content::host::HostBehavior::GwentBaoxueCard {
                        crate::content::host::HostStage::SelectCard
                    } else {
                        crate::content::host::HostStage::Act
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target: u.targets.first().copied(),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Shatang | CardKind::Heilonglinpian | CardKind::Liutouge => {
                    // 古剑最后机关/锦囊（G1）
                    let (key, behavior, target) = match effective_kind {
                        CardKind::Shatang => (
                            crate::content::host::GJ_SHATANG_CARD_KEY,
                            crate::content::host::HostBehavior::JiguanShatangCard,
                            u.targets.first().copied(),
                        ),
                        CardKind::Heilonglinpian => (
                            crate::content::host::GJ_HEILONGLINPIAN_CARD_KEY,
                            crate::content::host::HostBehavior::JiguanHeilongCard,
                            Some(u.player),
                        ),
                        _ => (
                            crate::content::host::GJ_LIUTOUGE_CARD_KEY,
                            crate::content::host::HostBehavior::JiguanLiutougeCard,
                            u.targets.first().copied(),
                        ),
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Dunpaigedang
                | CardKind::Zhiliaobo
                | CardKind::Yuansuhuimie
                | CardKind::Xingjiegoutong
                | CardKind::Chuansongmen
                | CardKind::Tanshezhiren
                | CardKind::Shijieshu => {
                    // 炉石余量（G1）：无懈窗口包装 + 各自机器
                    let (key, behavior, target, targets) = match effective_kind {
                        CardKind::Dunpaigedang => (
                            crate::content::host::HH_DUNPAI_CARD_KEY,
                            crate::content::host::HostBehavior::HearthDunpaiCard,
                            Some(u.player),
                            Vec::new(),
                        ),
                        CardKind::Zhiliaobo => (
                            crate::content::host::HH_ZHILIAOBO_CARD_KEY,
                            crate::content::host::HostBehavior::HearthZhiliaoboCard,
                            u.targets.first().copied(),
                            Vec::new(),
                        ),
                        CardKind::Yuansuhuimie => (
                            crate::content::host::HH_YUANSU_CARD_KEY,
                            crate::content::host::HostBehavior::HearthYuansuCard,
                            None,
                            ctx.core
                                .alive_players()
                                .filter(|p| *p != u.player)
                                .collect(),
                        ),
                        CardKind::Xingjiegoutong => (
                            crate::content::host::HH_XINGJIE_CARD_KEY,
                            crate::content::host::HostBehavior::HearthXingjieCard,
                            Some(u.player),
                            Vec::new(),
                        ),
                        CardKind::Chuansongmen => (
                            crate::content::host::HH_CHUANSONG_CARD_KEY,
                            crate::content::host::HostBehavior::HearthChuansongCard,
                            Some(u.player),
                            Vec::new(),
                        ),
                        CardKind::Shijieshu => (
                            crate::content::host::HH_SHIJIE_CARD_KEY,
                            crate::content::host::HostBehavior::HearthShijieshuCard,
                            Some(u.player),
                            Vec::new(),
                        ),
                        _ => (
                            crate::content::host::HH_TANSHE_CARD_KEY,
                            crate::content::host::HostBehavior::HearthTansheCard,
                            Some(u.player),
                            Vec::new(),
                        ),
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets,
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Shihuifen | CardKind::Liufengsan => {
                    // 古剑锦囊（G1）：石会粉（响应拒绝）/六风散（获得闪）
                    let (key, behavior) = match effective_kind {
                        CardKind::Shihuifen => (
                            crate::content::host::GJ_SHIHUIFEN_CARD_KEY,
                            crate::content::host::HostBehavior::GujianShihuifenCard,
                        ),
                        _ => (
                            crate::content::host::GJ_LIUFENGSAN_CARD_KEY,
                            crate::content::host::HostBehavior::GujianLiufengsanCard,
                        ),
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target: u.targets.first().copied(),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Bingpotong
                | CardKind::Qiankunbiao
                | CardKind::Longxugou
                | CardKind::Shenhuofeiya => {
                    // 古剑机关（G1）：无懈窗口包装 + 各自机器
                    let (key, behavior, targets) = match effective_kind {
                        CardKind::Bingpotong => (
                            crate::content::host::GJ_BINGPOTONG_CARD_KEY,
                            crate::content::host::HostBehavior::JiguanBingpoCard,
                            u.targets.clone(),
                        ),
                        CardKind::Qiankunbiao => (
                            crate::content::host::GJ_QIANKUNBIAO_CARD_KEY,
                            crate::content::host::HostBehavior::JiguanQiankunCard,
                            Vec::new(),
                        ),
                        CardKind::Longxugou => (
                            crate::content::host::GJ_LONGXUGOU_CARD_KEY,
                            crate::content::host::HostBehavior::JiguanLongxuCard,
                            Vec::new(),
                        ),
                        _ => {
                            // 神火飞鸦：目标 + 距离 1 邻座（座位序）响应队列
                            let t = *u.targets.first().expect("shenhuo has one target");
                            let mut q = vec![t];
                            for p in ctx.core.alive_players() {
                                if p != t
                                    && p != u.player
                                    && ctx.core.computed_distance(ctx.catalog, t, p) == 1
                                {
                                    q.push(p);
                                }
                            }
                            (
                                crate::content::host::GJ_SHENHUOFEIYA_CARD_KEY,
                                crate::content::host::HostBehavior::JiguanShenhuoCard,
                                q,
                            )
                        }
                    };
                    // 冰魄通先开使用者展示询问（逐技对照批五 v79），其余直接执行
                    let stage = if behavior == crate::content::host::HostBehavior::JiguanBingpoCard
                    {
                        crate::content::host::HostStage::ComparePlayerPick
                    } else if behavior == crate::content::host::HostBehavior::JiguanLongxuCard {
                        crate::content::host::HostStage::SelectCard
                    } else {
                        crate::content::host::HostStage::Act
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: key.to_string(),
                        behavior,
                        owner: u.player,
                        target: u.targets.first().copied(),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets,
                        stage,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Liuxinghuoyu => {
                    // 流星火雨（U05 swd 首批，规则 v86）：无懈窗口包装 + 目标
                    // 自选弃两张 he 或受 1 点火伤
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::SWD_LIUXING_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::SwdLiuxinghuoyuCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("liuxinghuoyu has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Hslingjian => {
                    // 灵剑（swd 机关术批，规则 v198）：七卡按名分流；无懈窗
                    // 包装共享（上游 wuxieable 同口径）。
                    let name = &ctx.catalog.cards[u.card as usize].name;
                    let (behavior, def_key, target) = if name.contains("xuanfengzhiren") {
                        (
                            crate::content::host::HostBehavior::SwdXuanfengzhirenCard,
                            crate::content::host::SWD_XUANFENG_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("zhongxinghujia") {
                        (
                            crate::content::host::HostBehavior::SwdZhongxinghujiaCard,
                            crate::content::host::SWD_ZHONGXING_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("xingtigaizao") {
                        (
                            crate::content::host::HostBehavior::SwdXingtigaizaoCard,
                            crate::content::host::SWD_XINGTIGAIZAO_KEY,
                            Some(u.player),
                        )
                    } else if name.contains("shijianhuisu") {
                        (
                            crate::content::host::HostBehavior::SwdShijianhuisuCard,
                            crate::content::host::SWD_SHIJIANHUISU_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("jinjilengdong") {
                        (
                            crate::content::host::HostBehavior::SwdJinjilengdongCard,
                            crate::content::host::SWD_JINJILENGDONG_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("shengxiuhaojiao") {
                        (
                            crate::content::host::HostBehavior::SwdShengxiuhaojiaoCard,
                            crate::content::host::SWD_SHENGXIUHAOJIAO_KEY,
                            u.targets.first().copied(),
                        )
                    } else {
                        (
                            crate::content::host::HostBehavior::SwdYinmilichangCard,
                            crate::content::host::SWD_YINMILICHANG_KEY,
                            u.targets.first().copied(),
                        )
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: def_key.to_string(),
                        behavior,
                        owner: u.player,
                        target,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Changshezhen | CardKind::Pozhenjue => {
                    // 长蛇阵/破阵诀（U05 zhenfa，规则 v201/T19b）：座位重排类
                    //（swapSeat 原语 ADR 0781）；无懈窗口包装（dizaizhen 同款）
                    let kind = ctx.catalog.card_kind(u.card);
                    let (behavior, def_key) = if kind == CardKind::Changshezhen {
                        (
                            crate::content::host::HostBehavior::SwdChangshezhenCard,
                            crate::content::host::SWD_CHANGSHEZHEN_KEY,
                        )
                    } else {
                        (
                            crate::content::host::HostBehavior::SwdPozhenjueCard,
                            crate::content::host::SWD_POZHENJUE_KEY,
                        )
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: def_key.to_string(),
                        behavior,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Tianfuzhen | CardKind::Qixingzhen | CardKind::Niaoxiangzhen => {
                    // 阵法三张（U05 zhenfa，规则 v200/T19a）：AOE/自目标机器
                    // + 无懈窗口包装（dizaizhen 同款）
                    let kind = ctx.catalog.card_kind(u.card);
                    let behavior = match kind {
                        CardKind::Tianfuzhen => {
                            crate::content::host::HostBehavior::SwdTianfuzhenCard
                        }
                        CardKind::Qixingzhen => {
                            crate::content::host::HostBehavior::SwdQixingzhenCard
                        }
                        _ => crate::content::host::HostBehavior::SwdNiaoxiangzhenCard,
                    };
                    let def_key = match kind {
                        CardKind::Tianfuzhen => crate::content::host::SWD_TIANFUZHEN_KEY,
                        CardKind::Qixingzhen => crate::content::host::SWD_QIXINGZHEN_KEY,
                        _ => crate::content::host::SWD_NIAOXIANGZHEN_KEY,
                    };
                    let mut phase2_queue: Vec<u32> = Vec::new();
                    let targets_for_machine = match kind {
                        CardKind::Tianfuzhen => {
                            // AOE：全部大势力且有牌者（engine 校验同口径）
                            ctx.core
                                .alive_players()
                                .filter(|p| {
                                    ctx.core.is_major_faction(ctx.catalog, *p)
                                        && (!ctx.core.player(*p).hand.is_empty()
                                            || !ctx.core.player(*p).equips.is_empty())
                                })
                                .collect()
                        }
                        CardKind::Qixingzhen => {
                            // 阶段一队列=全部围攻者（sieges(q, player)）；阶段二
                            // 队列同名单（存 cards，视为用杀）
                            let besiegers: Vec<u8> = ctx
                                .core
                                .alive_players()
                                .filter(|q| ctx.core.sieges(ctx.catalog, *q, u.player))
                                .collect();
                            phase2_queue = besiegers.iter().map(|q| u32::from(*q)).collect();
                            besiegers
                        }
                        _ => {
                            // 鸟翔阵 AOE：全部合法目标
                            ctx.core
                                .alive_players()
                                .filter(|t| {
                                    *t != u.player
                                        && !ctx.core.is_friend_of(ctx.catalog, u.player, *t)
                                        && {
                                            let (n, pv) = ctx.core.alive_neighbors(*t);
                                            n.is_some_and(|q| {
                                                ctx.core.effective_faction(ctx.catalog, *t)
                                                    == ctx.core.effective_faction(ctx.catalog, q)
                                            }) || pv.is_some_and(|q| {
                                                ctx.core.effective_faction(ctx.catalog, *t)
                                                    == ctx.core.effective_faction(ctx.catalog, q)
                                            })
                                        }
                                })
                                .collect()
                        }
                    };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: def_key.to_string(),
                        behavior,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: phase2_queue,
                        targets: targets_for_machine,
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Jiguan => {
                    // 机关牌（swd 第十六/十七批，规则 v189/v190）：同 kind
                    // 异行为按目录名分流（指路小虎=随机敌虚拟杀；木甲人=
                    // 弃非基本换随机机关牌）——无懈窗口包装共享。
                    let name = &ctx.catalog.cards[u.card as usize].name;
                    let (behavior, def_key, target) = if name.contains("mujiaren") {
                        (
                            crate::content::host::HostBehavior::SwdMujiarenCard,
                            crate::content::host::SWD_MUJIAREN_KEY,
                            None,
                        )
                    } else if name.contains("gouhunluo") {
                        (
                            crate::content::host::HostBehavior::SwdGouhunluoCard,
                            crate::content::host::SWD_GOUHUNLUO_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("jiguantong") {
                        (
                            crate::content::host::HostBehavior::SwdJiguantongCard,
                            crate::content::host::SWD_JIGUANTONG_KEY,
                            None,
                        )
                    } else if name.contains("jiguanfeng") {
                        (
                            crate::content::host::HostBehavior::SwdJiguanfengCard,
                            crate::content::host::SWD_JIGUANFENG_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("jiguanyuan") {
                        (
                            crate::content::host::HostBehavior::SwdJiguanyuanCard,
                            crate::content::host::SWD_JIGUANYUAN_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("jiguanyaoshu") {
                        (
                            crate::content::host::HostBehavior::SwdJiguanyaoshuCard,
                            crate::content::host::SWD_JIGUANYAOSHU_KEY,
                            u.targets.first().copied(),
                        )
                    } else if name.contains("jiguanshu") {
                        (
                            crate::content::host::HostBehavior::SwdJiguanshuCard,
                            crate::content::host::SWD_JIGUANSHU_KEY,
                            Some(u.player),
                        )
                    } else {
                        (
                            crate::content::host::HostBehavior::SwdZhiluxiaohuCard,
                            crate::content::host::SWD_ZHILUXIAOHU_KEY,
                            None,
                        )
                    };
                    let targets_for_machine =
                        if behavior == crate::content::host::HostBehavior::SwdJiguantongCard {
                            // 逆序结算（规则 v193，上游 reverseOrder）：目标表
                            // 反转为结算序——从使用者上家逆座位序逐个结算
                            let mut ts = u.targets.clone();
                            ts.reverse();
                            ts
                        } else {
                            Vec::new()
                        };
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: def_key.to_string(),
                        behavior,
                        owner: u.player,
                        target,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: targets_for_machine,
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Yangpijuan => {
                    // 羊皮卷（U05 swd 第十三批，规则 v97）：无懈窗口包装 +
                    // 选类别 → 牌堆随机亮 3 张 → 选一张入手
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::SWD_YANGPIJUAN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::SwdYangpijuanCard,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Longlushanfeng => {
                    // 龙颅山峰（U05 mtg 首切片，规则 v100）：换地 + 二选一
                    // 获得杀/闪（上游 createCard 凭空造牌→原生牌堆取，登记
                    // 差异）；地非锦囊，无无懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_LONGSHANFENG_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgLongshanfengLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Haidao => {
                    // 海岛（U05 mtg 第二批，规则 v101）：地机器——满血加护
                    // 甲/受伤弃牌回血或取消得护甲；地非锦囊，无无懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_HAIDAO_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgHaidaoLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Shuimomuxue => {
                    // 水没墓穴（U05 mtg 第三批，规则 v102）：地机器——可弃
                    // 一张摸二（可取消）；地非锦囊，无无懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_SHUIMOMUXUE_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgShuimomuxueLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Feixu => {
                    // 废墟（U05 mtg 第四批，规则 v103）：地机器——从弃牌堆
                    // 发现一张牌；地非锦囊，无无懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_FEIXU_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgFeixuLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Yixialan => {
                    // 依夏兰（U05 mtg 第五批，规则 v104）：地机器——选类型
                    // 随机获得基本/锦囊牌一张；地附主动技（重铸）经换地授予
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_YIXIALAN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgYixialanLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Shamolvzhou => {
                    // 沙漠绿洲（U05 mtg 第六批，规则 v105）：地机器——从牌堆
                    // 基本牌发现一张；地非锦囊，无无懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_SHAMOLVZHOU_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgShamolvzhouLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Linzhongjianta => {
                    // 林中尖塔（U05 mtg 第七批，规则 v106）：地机器——发现
                    // 武器并直接装备；地非锦囊，无无懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_LINZHONGJIANTA_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgLinzhongjiantaLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Bingheyaosai => {
                    // 冰河要塞（U05 mtg 第八批，规则 v107）：地机器——摸二
                    // 弃二（强制）；地非锦囊，无无懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_BINGHEYAOSAI_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgBingheyaosaiLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Youlin => {
                    // 幽林（U05 mtg 第九批，规则 v108）：地机器——随机使用
                    // 一张普通锦囊（池收窄登记见 catalog）；地非锦囊，无无
                    // 懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_YOULIN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgYoulinLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Lindixiliu => {
                    // 邻地溪流（U05 mtg 第十批，规则 v109）：地机器——♥入
                    // 手 + 任意张♥弃 → 每张随机敌方置延时；地非锦囊，无无
                    // 懈窗口
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::MTG_LINDIXILIU_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::MtgLindixiliuLand,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Dizaizhen => {
                    // 地载阵（U05 zhenfa 首切片，规则 v110）：无懈窗口包装
                    // + 所有非大势力存活角色各摸一张
                    let effect = Box::new(DizaizhenMachine { owner: u.player });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Mianlijinzhen => {
                    // 面里金针（G1 古剑，机关）：目标摸 1 然后失去 1 点体力
                    //（规则 v77 对齐上游 loseHp；上游无 wuxieable，不包无懈窗口）
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::GJ_MIANLI_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::GujianMianliCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("mianlijinzhen has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(effect)
                }
                CardKind::Feibiao => {
                    // 飞镖（G1 古剑，机关）：目标弃一张黑色手牌或受 1 点伤害
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::GJ_FEIBIAO_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::GujianFeibiaoCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("feibiao has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::SelectCard,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Chenhuodajie => {
                    // 趁火打劫（G1 运筹帷幄，锦囊）：获得目标一张手牌
                    //（登记差异：上游为目标任选可见牌，原生取首张确定性获得）
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::WD_CHENHUO_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::ChenhuodajieCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("chenhuodajie has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Fudichouxin => {
                    // 釜底抽薪（G1 运筹帷幄，锦囊）：无懈窗口 + 拼点（两段选择）
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::WD_FUDI_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::FudichouxinCompare,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("fudichouxin has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::CompareTargetPick,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::WangmeiZhike => {
                    // 望梅止渴（G1 玩点论杀，锦囊）：无懈窗口 + 目标最小体力/
                    // 最小手牌条件增益（宿主原语）
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::WD_WANGMEIZHIKE_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::WangmeiZhikeCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("wangmeizhike has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::ShouLiJian => {
                    // 手里剑（G1 欢乐卡牌）：目标弃 1 张装备或受 1 点伤害
                    //（嵌套选择恢复样本，ADR 0776 规则宿主）
                    Some(Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::HOST_SHOULIJIAN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::TargetDiscardEquipOrDamage,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("shoulijian has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::TargetChoice,
                    }))
                }
                CardKind::Guisheqi => {
                    // 鬼蛇旗（swd 补切批⑦）：目标 +1 护甲（原生 change_hujia=
                    // 受伤抵 1 伤语义，上游 changeHujia 同口径）
                    Some(Box::new(crate::content::host::HostMachine {
                        def_key: "guisheqi_card".to_string(),
                        behavior: crate::content::host::HostBehavior::GuisheqiCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("guisheqi target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    }))
                }
                CardKind::Xianluhui => {
                    // 仙露回（swd 补切批⑦）：全体受伤目标各 +1 护甲
                    Some(Box::new(crate::content::host::HostMachine {
                        def_key: "xianluhui_card".to_string(),
                        behavior: crate::content::host::HostBehavior::XianluhuiCard,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: u.targets.clone(),
                        stage: crate::content::host::HostStage::Act,
                    }))
                }
                kind if kind.aoe_need().is_some() => {
                    // AOE 的无懈窗口为**逐目标**（由 AoeResolveMachine 对每个目标
                    // 自行开窗），外层不再包整牌级窗口——旧包裹会吞掉应答并把
                    // 整张牌作废（本批修复，ADR 0705）。
                    Some(Box::new(AoeResolveMachine {
                        card: u.card,
                        source: u.player,
                        need: kind.aoe_need().expect("checked"),
                        targets: u.targets,
                        idx: 0,
                        inner: None,
                        nature: kind.aoe_nature(),
                    }))
                }
                CardKind::Chain => {
                    // 铁索的逐目标无懈窗口由 ChainMachine 自身对每个目标开启
                    // （对齐上游 useCardToBegin 逐目标窗口），外层不再包整牌级
                    // 窗口——旧包裹在任一无懈生效时令**全部**目标不横置。
                    Some(Box::new(ChainMachine::dispatcher(
                        u.card, u.player, u.targets,
                    )))
                }
                CardKind::Duel => {
                    let effect = DuelMachine {
                        card: u.card,
                        source: u.player,
                        target: *u.targets.first().expect("duel has one target"),
                        on_target: true,
                        asked: false,
                        second_pending: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::Zhufangshenshi => {
                    // 贴房绅视（U05 swd 第十批，规则 v94）：他人锦囊——本回合
                    // 无视距离指定该目标并摸一张牌。效果在 intrinsic 提交段
                    // 同步执行（无需机器）；此处仅占位以透过穷尽 match。
                    None
                }
                CardKind::Guiyoujie => {
                    // 归有界（U05 swd，规则 v91）：他人延时锦囊——判定为黑色
                    // 则失去 1 点体力 + 随机弃一张牌（上游 loseHp+randomDiscard）
                    let target = *u.targets.first().expect("guiyoujie has one target");
                    let effect = PlaceDelayedMachine {
                        card: u.card,
                        target,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::Le | CardKind::Bing => {
                    // 延时锦囊使用时即开无懈窗口（对齐旧线：牌先入桌面，
                    // 无懈后入判定区、被无懈则弃置），窗口未抵消由
                    // PlaceDelayedMachine 置入目标判定区。
                    let target = *u.targets.first().expect("delayed trick has one target");
                    let effect = PlaceDelayedMachine {
                        card: u.card,
                        target,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::ShanDian => {
                    let effect = PlaceDelayedMachine {
                        card: u.card,
                        target: u.player,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::HongShui => {
                    // 洪水：自-target 延时锦囊（判定：梅花 2–9 → 3 点雷伤）
                    let effect = PlaceDelayedMachine {
                        card: u.card,
                        target: u.player,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        u.card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::Caochuanjiejian => {
                    // 草船借箭（G1 运筹帷幄，锦囊）：无懈窗口 + 目标分支选择
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::WD_CAOCHUAN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::CaochuanCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("caochuan has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::SelectNature,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Geanguanhuo => {
                    // 隔岸观火（G1 运筹帷幄，锦囊）：双目标拼点链
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::YC_GEAN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::GeanguanhuoCompare,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: u.targets.clone(),
                        stage: crate::content::host::HostStage::CompareTargetPick,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Linghunzhihuo => {
                    // 灵魂之火（G1 炉石，锦囊）：目标 1 点火焰伤害，使用者弃一张手牌
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::HT_LINGHUN_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::LinghunzhihuoCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("linghunzhihuo has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Zhaomingdan => {
                    // 照明弹（G1 炉石，锦囊）：弃目标一张手牌，双方各摸 1
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::HT_ZHAOMING_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::ZhaomingdanCard,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("zhaomingdan has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Jihuocard => {
                    // 急耗卡（G1 炉石，锦囊）：自-target 摸 1（临时技暂缓）
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::HT_JIHUO_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::JihuocardDraw,
                        owner: u.player,
                        target: None,
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::Act,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                CardKind::Toulianghuanzhu => {
                    // 偷梁换柱（G1 运筹帷幄，锦囊）：无懈窗口 + 换牌双选
                    let effect = Box::new(crate::content::host::HostMachine {
                        def_key: crate::content::host::WD_TOULIANG_KEY.to_string(),
                        behavior: crate::content::host::HostBehavior::TouliangSwap,
                        owner: u.player,
                        target: Some(*u.targets.first().expect("touliang has one target")),
                        card: Some(u.card),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: crate::content::host::HostStage::SelectCard,
                    });
                    Some(Box::new(wrap_nullify_window(ctx, u.card, u.player, effect)))
                }
                _ => None,
            }
        }
        EventData::Phase {
            player,
            phase: Phase::Judge,
            begin: true,
        } => {
            let judge = std::mem::take(&mut ctx.core.player_mut(player).judge);
            if judge.is_empty() {
                None
            } else {
                // 判定排序分组（N06-a，对齐旧线 pending_delayed_cards）：
                // 非闪电延时锦囊按放置序在前、闪电在后，队首先消费。
                let mut queue: Vec<crate::ids::CardId> = judge
                    .iter()
                    .copied()
                    .filter(|c| ctx.catalog.card_kind(*c) != CardKind::ShanDian)
                    .collect();
                queue.extend(
                    judge
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::ShanDian),
                );
                Some(Box::new(JudgePhaseMachine {
                    owner: player,
                    queue,
                    current: None,
                }))
            }
        }
        EventData::Phase {
            player,
            phase: Phase::Discard,
            begin: true,
        } => {
            // 贪欲（v215）/庆祝（v245）：跳弃牌族——弃牌机器不生成（上游
            // phaseDiscardBefore trigger.cancel 同口径；此前贪欲面被测试
            // drain 掩盖，本臂补真实闸门）
            let mut skip_discard = [
                crate::content::standard::BOSS_TANYU,
                crate::content::standard::BOSS_QINGZHU,
            ]
            .iter()
            .any(|name| {
                ctx.catalog
                    .skill_by_name(name)
                    .map(|d| ctx.core.has_skill(player, d))
                    .unwrap_or(false)
            });
            // 猛态（Boss 年兽，规则 v248）：本回合未进入出牌阶段 → 跳过
            // 弃牌阶段（上游 boss_mengtai_discard trigger.cancel 同口径）
            if !skip_discard && ctx.core.turn.mengtai_use_pending {
                if let Some(mtdef) = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::BOSS_MENGTAI)
                {
                    if ctx.core.has_skill(player, mtdef) {
                        skip_discard = true;
                    }
                }
            }
            if skip_discard {
                ctx.core.turn.discard_phase_discarded = 0;
                None
            }
            // 克己（吕蒙）：本回合未使用杀则无需弃牌（手牌上限视为无限）
            else {
                let keji = ctx
                    .catalog
                    .skill_by_name(crate::content::standard::KE_JI)
                    .map(|d| ctx.core.has_skill(player, d))
                    .unwrap_or(false);
                if keji && !ctx.core.turn.sha_used_play_phase {
                    None
                } else {
                    let mut limit = ctx.core.hand_limit(ctx.catalog, player);
                    // 机甲（U05 ow D.Va，规则 v141）：机甲血>0 时手牌上限 +4
                    //（上游 mod maxHandcard storage.jijia 同口径）
                    if ctx
                        .catalog
                        .skill_by_name(crate::content::host::WUDIAN_JIJIA_KEY)
                        .map(|def| ctx.core.has_skill(player, def))
                        .unwrap_or(false)
                        && ctx.core.player(player).jijia_hp > 0
                    {
                        limit += 4;
                    }
                    // 蛊舞·缚（U05 xianjian 阿奴，规则 v154）：蛊舞生效中手牌
                    // 上限 -1（上游 anwugu2 mod maxHandcard 同口径）
                    if ctx.core.player(player).anwugu_count > 0 {
                        limit = limit.saturating_sub(1);
                    }
                    // 春饼（G1 古剑食物）：手牌上限 +1（持有状态即生效）
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::content::host::GJ_FOOD_CHUNBING_KEY)
                    {
                        if ctx
                            .core
                            .skills
                            .get(&(player, def))
                            .map(|i| i.enabled)
                            .unwrap_or(false)
                        {
                            limit += 1;
                        }
                    }
                    // 壶境（壶中仙，swd 卡牌面补切批③）：装备炼妖壶时手牌上限
                    // +2（上游 hujing mod maxHandcard getEquip("lianyaohu") 同口径）
                    if ctx
                        .core
                        .player(player)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Lianyaohu)
                    {
                        limit += 2;
                    }
                    // 乾坤袋（swd 卡牌面补切批④）：装备时手牌上限 +1（上游
                    // qiankundai mod maxHandcard 同口径）
                    if ctx
                        .core
                        .player(player)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Qiankundai)
                    {
                        limit += 1;
                    }
                    // 倾盆大雨（gwent 天气）：手牌上限 -1
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::content::host::GW_QINPEN_STATE_KEY)
                    {
                        if ctx
                            .core
                            .skills
                            .get(&(player, def))
                            .map(|i| i.enabled)
                            .unwrap_or(false)
                        {
                            limit = limit.saturating_sub(1);
                        }
                    }
                    // 妄尊印（袁术妄尊授予的回合级印记）：手牌上限 -1
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::WANG_ZUN_MARK)
                    {
                        if ctx
                            .core
                            .skills
                            .get(&(player, def))
                            .map(|i| i.enabled)
                            .unwrap_or(false)
                        {
                            limit = limit.saturating_sub(1);
                        }
                    }
                    // 水没墓穴地图效果（mtg 第三批，规则 v102）：锁定全局——
                    // 手牌上限 -1（current_land 状态直读，同倾盆大雨口径）
                    if ctx.core.current_land == Some(crate::catalog::CardKind::Shuimomuxue) {
                        limit = limit.saturating_sub(1);
                    }
                    // 沙漠绿洲地图效果（mtg 第六批，规则 v105）：基本牌不
                    // 计入手牌上限——需弃数按非基本手牌数计（上游
                    // ignoredHandcard basic 同口径）
                    let shamolvzhou =
                        ctx.core.current_land == Some(crate::catalog::CardKind::Shamolvzhou);
                    // 红莲（Boss 毕方，规则 v228）：红色牌不计入手牌上限
                    //（上游 ignoredHandcard red 同口径）
                    let honglianx = ctx
                        .catalog
                        .skill_by_name(crate::content::standard::BOSS_HONGLIANX)
                        .map(|def| ctx.core.has_skill(player, def))
                        .unwrap_or(false);
                    let hand = if shamolvzhou {
                        ctx.core
                            .player(player)
                            .hand
                            .iter()
                            .filter(|c| {
                                crate::content::host::yangpijuan_class(ctx.catalog.card_kind(**c))
                                    != Some(0)
                            })
                            .count()
                    } else if honglianx {
                        ctx.core
                            .player(player)
                            .hand
                            .iter()
                            .filter(|c| {
                                !matches!(
                                    ctx.catalog.cards[**c as usize].suit,
                                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                                )
                            })
                            .count()
                    } else {
                        ctx.core.player(player).hand.len()
                    };
                    if hand > limit {
                        Some(Box::new(DiscardToLimitMachine {
                            player,
                            need: hand - limit,
                            asked: false,
                        }))
                    } else {
                        None
                    }
                }
            }
        }
        EventData::Death { player, killer } => Some(Box::new(DeathMachine { player, killer })),
        _ => None,
    }
}
