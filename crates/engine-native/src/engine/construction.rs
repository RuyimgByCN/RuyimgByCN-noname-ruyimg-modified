use super::*;

impl Engine {
    /// 开一局 `seats` 人原型对局（存活数 ≤ 1 终局），每人 4 点体力、摸 4 张牌；
    /// 种子决定牌库顺序。
    pub fn new(seed: u64, seats: usize) -> Self {
        Self::new_with_mode(seed, seats, GameMode::Prototype, OpeningPlan::Standard)
    }

    /// 开一局 `seats` 人身份局（4–8 人）。座位 0 为主公（体力上限 +1），
    /// 其余身份按人数表分配并以规则随机流洗序；主忠反内胜负与奖惩见
    /// `GameMode::Identity` 与覆盖清单。人数不合法返回 `Err`
    /// （外部建局接口不依赖 panic；`new_identity` 为断言便捷封装）。
    pub fn try_new_identity(seed: u64, seats: usize) -> Result<Self, String> {
        if !(4..=8).contains(&seats) {
            return Err(format!("identity mode supports 4-8 seats, got {seats}"));
        }
        Ok(Self::new_identity_unchecked(seed, seats))
    }

    /// `try_new_identity` 的断言封装（内部与测试便捷用）。
    pub fn new_identity(seed: u64, seats: usize) -> Self {
        assert!(
            (4..=8).contains(&seats),
            "identity mode supports 4-8 seats, got {seats}"
        );
        Self::new_identity_unchecked(seed, seats)
    }

    pub(super) fn new_identity_unchecked(seed: u64, seats: usize) -> Self {
        Self::new_with_mode(
            seed,
            seats,
            Self::identity_mode(seats),
            OpeningPlan::Standard,
        )
    }

    fn identity_mode(seats: usize) -> GameMode {
        let loyalists = match seats {
            4 | 5 | 6 => 1,
            _ => 2,
        };
        let mut roles = vec![crate::state::Role::Lord];
        for _ in 0..loyalists {
            roles.push(crate::state::Role::Loyalist);
        }
        while roles.len() < seats - 1 {
            roles.push(crate::state::Role::Rebel);
        }
        roles.push(crate::state::Role::Traitor);
        debug_assert_eq!(roles.len(), seats);
        GameMode::Identity { roles }
    }

    /// 身份局使用同一扩展目录和初始化钩子；非法人数返回错误。
    pub fn try_new_identity_with_extensions(
        seed: u64,
        seats: usize,
        packs: &[&crate::content::extension::ExtensionPack],
    ) -> Result<Self, String> {
        if !(4..=8).contains(&seats) {
            return Err(format!("identity mode supports 4-8 seats, got {seats}"));
        }
        Self::new_with_extensions_mode(seed, seats, packs, Self::identity_mode(seats))
            .map_err(|e| format!("{e:?}"))
    }

    /// 国战模式（N11）：4–8 人或通用座位数势力对抗局，初始双将槽暗置。
    pub fn new_kingdoms(seed: u64, seats: usize) -> Self {
        Self::new_with_mode(seed, seats, GameMode::Kingdoms, OpeningPlan::Standard)
    }

    /// 乱斗模式（Brawl）：多座位大乱斗（存活数 ≤ 1 终局），复用原型无阵营生存结算。
    pub fn new_brawl(seed: u64, seats: usize) -> Self {
        Self::new_with_mode(seed, seats, GameMode::Prototype, OpeningPlan::Standard)
    }

    /// 单挑模式（Single / 1v1 KOF）：2 人单挑，默认双方各 2 名候选将池。
    pub fn new_single(seed: u64) -> Self {
        Self::new_kof(seed, [vec![0, 1], vec![2, 3]], [0, 0], 0)
    }

    /// KOF 引擎层（P2/N08 模式入口批四）：2 人局，携带会话将池/比分/局数。
    pub fn new_kof(seed: u64, pools: [Vec<u16>; 2], wins: [u32; 2], games: u32) -> Self {
        Self::new_with_mode(
            seed,
            2,
            GameMode::Kof { pools, wins, games },
            OpeningPlan::Standard,
        )
    }

    /// Boss（P2/N08 模式入口批三-a）：boss 1 名对挑战者，chongzheng=重整阈值（0=无复活）。
    pub fn new_boss(seed: u64, seats: usize, boss: PlayerId, chongzheng: u8) -> Self {
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Boss {
                boss,
                chongzheng,
                fellows: Vec::new(),
                stage: crate::state::BossStageState::default_none(),
            },
            OpeningPlan::Standard,
        )
    }

    /// Boss 助战中途入队（批三-b，规则 v84；D4/B0 正式 API）：
    /// 座位表队尾追加新玩家（登记差异：上游 addFellow 可按位置插入，
    /// 原生固定队尾），直授 4 张手牌（上游 directgain(4) 同），登记入
    /// fellows，并同步 `seat_ring` 队尾（ADR 0781）。仅 Boss 模式；返回新座位。
    pub fn add_boss_fellow(&mut self, hp: u32) -> Option<PlayerId> {
        if !matches!(&self.state.core.mode, GameMode::Boss { .. }) {
            return None;
        }
        let pid = self.state.core.players.len() as u8;
        let mut p = crate::state::PlayerState::new(hp);
        let take = self.state.core.deck.len().min(4);
        p.hand.extend(self.state.core.deck.drain(..take));
        self.state.core.players.push(p);
        // 保持座位环与玩家表长度一致（ADR 0781）；历史 debug 入口曾漏写，
        // ring_* 退回恒等位——正式 API 显式追加。
        self.state.core.seat_ring.push(pid);
        if let GameMode::Boss { fellows, .. } = &mut self.state.core.mode {
            fellows.push(pid);
        }
        for &def in &self.catalog.extension_global_skills {
            if crate::kernel::is_l22_elo_global_skill(self.catalog.skills[def as usize].name) {
                self.state.core.skills.insert(
                    (pid, def),
                    crate::state::SkillInstance {
                        owner: pid,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }
        crate::kernel::elo_try_init_cards(&mut self.state.core, &self.catalog, pid, false);
        crate::resolution::sync_trigger_owners(&self.state.core, &mut self.state.resolution);
        Some(pid)
    }

    /// 换 Boss 将（D4/B0 内部原语）：在当前 `GameMode::Boss.boss` 座位上
    /// 清空技能、重挂目录将并更新装配；不改 boss 指针、不改座位环。
    /// `preserve_hp=true` 时保留当前体力（上限随新将裁剪，年兽 nbianshen
    /// 同构）；`false` 时回满。区域（手/装/判定）保留。默认对局路径不调用。
    pub fn change_boss(&mut self, general: u16, preserve_hp: bool) -> bool {
        let Some((boss, _)) = self.state.core.boss_mode() else {
            return false;
        };
        self.reinit_boss_seat(boss, general, preserve_hp)
    }

    /// 在指定座位重挂将（D4/B0 内部原语；不改 boss 指针）。
    /// 隐藏链侧座换将 / 年兽形态轮换可复用。非 Boss 模式拒绝。
    pub fn reinit_boss_seat(&mut self, seat: PlayerId, general: u16, preserve_hp: bool) -> bool {
        if !matches!(&self.state.core.mode, GameMode::Boss { .. }) {
            return false;
        }
        if seat as usize >= self.state.core.players.len() {
            return false;
        }
        if self.catalog.general_meta(general).is_none() {
            return false;
        }
        let old_hp = self.state.core.player(seat).hp;
        let removed: Vec<crate::ids::SkillDefId> = self
            .state
            .core
            .skills
            .keys()
            .filter(|(o, _)| *o == seat)
            .map(|(_, d)| *d)
            .collect();
        for d in removed {
            self.state
                .core
                .remove_skill_and_r05_mark(&self.catalog, seat, d);
        }
        self.state.core.player_mut(seat).general_slots[0] = Some(general);
        self.state.core.player_mut(seat).general_slots[1] = None;
        self.state.core.player_mut(seat).revealed_generals = [true, false];
        apply_general_assembly(&mut self.state.core, &self.catalog, seat, 0);
        let p = self.state.core.player_mut(seat);
        if preserve_hp {
            if p.hp > p.max_hp as i32 {
                p.hp = p.max_hp as i32;
            } else {
                p.hp = old_hp.min(p.max_hp as i32);
            }
        } else {
            p.hp = p.max_hp as i32;
        }
        p.alive = true;
        p.dying = false;
        true
    }

    /// 斗地主（P2/N08 模式入口批二）：地主 1 名，其余为农民。
    pub fn new_doudizhu(seed: u64, seats: usize, lord: PlayerId) -> Self {
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Doudizhu { lord },
            OpeningPlan::Standard,
        )
    }

    /// 斗地主 binglin 变体（T11 模式变体，规则 v205）：开局地主 5 张/
    /// 农民 4 张（上游 gameDraw num 覆盖同口径；replacePile 换牌堆登记
    /// 不建模，用标准牌堆）。变体仅影响初始发牌，不入模式编码。
    pub fn new_doudizhu_binglin(seed: u64, seats: usize, lord: PlayerId) -> Self {
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Doudizhu { lord },
            OpeningPlan::DoudizhuBinglin,
        )
    }

    /// Boss 关卡骨架（T11 模式变体，规则 v207）：按关卡配置开局——
    /// boss/挑战者分列发牌数（上游 bossinfo.gameDraw，如穷奇 boss 8/
    /// 玩家 4）与回合起点座位（上游 bossinfo.loopFirst 以显式座位承载，
    /// 越界回退 0）。变体为建局输入不入模式编码（同斗地主 v205 先例）。
    #[allow(clippy::too_many_arguments)]
    pub fn new_boss_stage(
        seed: u64,
        seats: usize,
        boss: PlayerId,
        chongzheng: u8,
        boss_cards: u32,
        player_cards: u32,
        first: PlayerId,
    ) -> Self {
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Boss {
                boss,
                chongzheng,
                fellows: Vec::new(),
                stage: crate::state::BossStageState::default_none(),
            },
            OpeningPlan::BossStage {
                boss_cards,
                player_cards,
                first,
            },
        )
    }

    /// 对决 2v2 基线（P2/N08 模式入口批一）：`teams` 两队互斥并集=全部座位。
    pub fn new_versus(seed: u64, teams: [Vec<PlayerId>; 2]) -> Self {
        let subs = vec![Vec::new(), Vec::new()];
        Self::new_versus_with_substitutes(seed, teams, subs)
    }

    /// 对决替补变体（T11 模式变体，规则 v206）：每队携带替补武将池
    /// （与队伍同队序）——死亡座位从本队池弹队首换将复活（上游
    /// revive+uninit/init+摸 4+濒死链取消同口径；奖惩不结算登记差异）。
    pub fn new_versus_with_substitutes(
        seed: u64,
        teams: [Vec<PlayerId>; 2],
        substitutes: Vec<Vec<u16>>,
    ) -> Self {
        let seats = teams[0].len() + teams[1].len();
        assert_eq!(substitutes.len(), 2, "替补池须与队伍同队序（每队一列）");
        Self::new_with_mode(
            seed,
            seats,
            GameMode::Versus {
                teams: teams.to_vec(),
                substitutes,
            },
            OpeningPlan::Standard,
        )
    }

    pub(super) fn new_with_mode(
        seed: u64,
        seats: usize,
        mode: GameMode,
        opening: OpeningPlan,
    ) -> Self {
        let catalog: SharedCatalog = std::sync::Arc::new(content::standard::standard_catalog());
        Self::new_with_catalog(seed, seats, mode, catalog, opening)
    }

    /// 带扩展目录的建局（E03 内容锁接线）：目录由调用方按启用扩展
    /// 合并构建（`standard_catalog_with_extensions`）。
    pub(super) fn new_with_catalog(
        seed: u64,
        seats: usize,
        mut mode: GameMode,
        catalog: SharedCatalog,
        opening: OpeningPlan,
    ) -> Self {
        let mut core = GameStateCore {
            revision: 0,
            ruleset_version: catalog.ruleset_version,
            mode: GameMode::Prototype,
            players: (0..seats).map(|_| PlayerState::new(4)).collect(),
            seat_ring: (0..seats).map(|i| i as PlayerId).collect(),
            deck: Vec::new(),
            discard: Vec::new(),
            table: Vec::new(),
detached_cards: Vec::new(),
            rng: RuleRng::from_seed(seed),
            turn: TurnState {
                current: 0,
                phase: Phase::Judge,
                round: 1,
                sha_used_play_phase: false,
                mufeng_used_basic: false,
                wangchen_discard_basic: false,
                shenmiguo_used: false,
                shuchui_armed: false,
                shuchui_hit: false,
                lianda_used: false,
                rende_given: 0,
                jiu_used: false,
                mujiaren_used: false,
                jiu_active: false,
                turn_opened: true,
                skip_draw_phase: false,
                skip_play_phase: false,
                guxing_armed: false,
                guxing_count: 0,
                mengtai_draw_pending: false,
                mengtai_use_pending: false,
                zhufangshenshi_target: None,
                yuedong_num: 0,
                yuedong_eff: 0,
                yuedong_recover: false,
                liangou_hook: None,
                liangou_bonus_used: false,
                feitiao_target: None,
                fanling_used: false,
                xjyufeng_used: 0,
                diewu_draw_used: false,
                lueying_used: false,
                txianqu_used: false,
                jtjubao_used: false,
                zhuyue_targets: Vec::new(),
                sheling_used: 0,
                sheying_used: false,
                zhangmu_used: false,
                yingfeng_chain: false,
                luanjian_card: None,
                minwan_targets: Vec::new(),
                yanyu_count: 0,
                qizuo_copy: None,
                jidian_chain: false,
                land_used: false,
                land_sha_used: false,
                discard_phase_discarded: 0,
                ciqiu_dying: Vec::new(),
                kangyin2: None,
                huanxia_track: Vec::new(),
                cards_used_play_phase: 0,
                cards_used_turn: 0,
                liaoyuan_count: 0,
                xuanyuanjian_marked: false,
                kunlunjing_snapshot: Vec::new(),
                kunlunjing_hp: 0,
                extra_phases: Vec::new(),
                extra_phase_lingwu: false,
                bte_used_play_phase: 0,
                phase_number: 0,
            },
            skills: Default::default(),
            declared_kinds: Default::default(),
            declared_slots: Default::default(),
            card_natures: Default::default(),
            current_land: None,
            outcome: None,
            vm_aux: crate::state::VmAux::default(),
        };
        // 身份局：其余身份洗序（种子决定），主公（座位 0）体力上限 +1。
        if let GameMode::Identity { roles } = &mut mode {
            core.rng.shuffle(&mut roles[1..]);
            let lord = roles
                .iter()
                .position(|r| *r == crate::state::Role::Lord)
                .expect("lord exists");
            let p = &mut core.players[lord];
            p.max_hp += 1;
            p.hp = p.max_hp as i32;
        }
        core.mode = mode;
        // 开局摸牌数表（v205 斗地主变体）：binglin 地主 5/农民 4（上游
        // gameDraw num 覆盖）；其余模式各 4 张。先于 core move 计算。
        let opening_cards: Vec<u32> = (0..seats as PlayerId)
            .map(|p| match (&core.mode, &opening) {
                (GameMode::Doudizhu { lord }, OpeningPlan::DoudizhuBinglin) => {
                    if p == *lord {
                        5
                    } else {
                        4
                    }
                }
                (
                    GameMode::Boss { boss, .. },
                    OpeningPlan::BossStage {
                        boss_cards,
                        player_cards,
                        ..
                    },
                ) if p == *boss => *boss_cards,
                (GameMode::Boss { .. }, OpeningPlan::BossStage { player_cards, .. }) => {
                    *player_cards
                }
                _ => 4,
            })
            .collect();
        // 牌库 = 目录牌序，开局用规则随机流洗牌。
        core.deck = (0..catalog.cards.len() as CardId).collect();
        core.rng.shuffle(&mut core.deck);
        let mut state = GameState {
            core,
            resolution: Resolution::new(DEFAULT_BUDGET),
        };
        // Boss 关卡回合起点（v207：loopFirst 以显式座位承载；越界回退 0）
        if let (GameMode::Boss { .. }, OpeningPlan::BossStage { first, .. }) =
            (&state.core.mode, opening)
        {
            state.core.turn.current = if (first as usize) < seats { first } else { 0 };
        }
        for (p, cards) in (0..seats as PlayerId).zip(opening_cards) {
            for _ in 0..cards {
                let card = state.core.deck.pop().expect("deck has cards");
                state.core.player_mut(p).hand.push(card);
            }
        }
        Self {
            catalog,
            state,
            log: Vec::new(),
            last_dedup: 0,
            time_slice: None,
            seat_quota: None,
            seat_cmd_counts: Default::default(),
            auth_tokens: Default::default(),
            auth_counter: 0,
            auth_secret: fresh_auth_secret(),
            auth_generation: 0,
            segment_open: false,
            debug_budget: None,
            segment_checkpoint: None,
            presentation_range: crate::presentation::PresentationRange::Idle,
            simulation: false,
        }
    }

    /// 带扩展的建局（E03）：启用扩展卡牌经冲突校验合并入目录——
    /// 牌库/发牌/牌类查询全部走合并后目录；目录版本绑定扩展集合。
    pub fn new_with_extensions(
        seed: u64,
        seats: usize,
        packs: &[&crate::content::extension::ExtensionPack],
    ) -> Result<Self, crate::content::extension::ExtensionError> {
        Self::new_with_extensions_mode(seed, seats, packs, GameMode::Prototype)
    }

    fn new_with_extensions_mode(
        seed: u64,
        seats: usize,
        packs: &[&crate::content::extension::ExtensionPack],
        mode: GameMode,
    ) -> Result<Self, crate::content::extension::ExtensionError> {
        let catalog = content::standard::standard_catalog_with_extensions(packs)?;
        let catalog: SharedCatalog = std::sync::Arc::new(catalog);
        let mut e = Self::new_with_catalog(seed, seats, mode, catalog, OpeningPlan::Standard);
        // G1 wuxing 初始化钩子：每名角色随机五行属性 + 牌堆按 1/3 比例
        // 随机获得属性（规则随机性使用引擎 RNG，保持可重放）。
        if packs
            .iter()
            .any(|p| p.enabled && p.init_hook == crate::content::extension::InitHook::WuxingNatures)
        {
            let core = &mut e.state.core;
            for pl in core.players.iter_mut() {
                if pl.alive {
                    pl.wunature = Some(core.rng.gen_range(5) as u8);
                }
            }
            let deck_len = core.deck.len();
            for i in 0..deck_len {
                if core.rng.gen_range(3) == 0 {
                    let card = core.deck[i];
                    let nature = core.rng.gen_range(5) as u8;
                    core.card_natures.insert(card, nature);
                }
            }
        }
        // Grants use the same derived declarations that selection and restore read.
        for &def in &e.catalog.extension_global_skills {
            for p in 0..seats as crate::ids::PlayerId {
                e.state.core.skills.insert(
                    (p, def),
                    crate::state::SkillInstance {
                        owner: p,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }
        crate::kernel::elo_on_game_start(&mut e.state.core, &e.catalog);
        Ok(e)
    }
}
