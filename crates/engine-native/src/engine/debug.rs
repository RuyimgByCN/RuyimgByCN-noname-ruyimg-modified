use super::*;

impl Engine {
    /// 测试入口：等价于 [`Self::add_boss_fellow`]。
    pub fn debug_add_fellow(&mut self, hp: u32) -> Option<PlayerId> {
        self.add_boss_fellow(hp)
    }

    /// 测试/工具：从某玩家手牌/装备区取走一张（走 `take_owned_card`
    /// 语义：区移除、装备技撤销、义山记录钩），并**发射失去牌事件**
    /// （CardsLost，走完整结算——失去牌触发类技能照常开窗）。取走后牌
    /// 入弃牌堆。
    pub fn debug_take_owned_card(&mut self, player: PlayerId, card: CardId) -> CommandOutcome {
        if self.state.core.outcome.is_some() {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        if self.state.resolution.open_choice.is_some() {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        let in_hand = work
            .core
            .player(player)
            .hand
            .iter()
            .position(|c| *c == card);
        let in_equips = work
            .core
            .player(player)
            .equips
            .iter()
            .position(|c| *c == card);
        let origin = if let Some(pos) = in_hand {
            work.core.player_mut(player).hand.remove(pos);
            crate::event::LostOrigin::Hand
        } else if let Some(pos) = in_equips {
            work.core.player_mut(player).equips.remove(pos);
            crate::kernel::revoke_equip_skills(&mut work.core, &self.catalog, player, card);
            crate::event::LostOrigin::Equip
        } else {
            panic!("card {card} not in hand or equips of {player}");
        };
        crate::machine::note_cards_lost(&mut work.core, &self.catalog, player, &[card]);
        work.core.discard.push(card);
        crate::resolution::push_frame(
            &mut work.resolution,
            EventData::CardsLost {
                player,
                cards: vec![card],
                origins: vec![origin],
            },
        );
        self.finish_segment(work, log_mark)
    }

    /// 旧签名兼容（不发射事件；仅供无触发面测试布阵）。
    pub fn debug_take_owned_card_quiet(&mut self, player: PlayerId, card: CardId) {
        let in_hand = self
            .state
            .core
            .player(player)
            .hand
            .iter()
            .position(|c| *c == card);
        if let Some(pos) = in_hand {
            self.state.core.player_mut(player).hand.remove(pos);
            crate::machine::note_cards_lost(&mut self.state.core, &self.catalog, player, &[card]);
            self.state.core.discard.push(card);
            return;
        }
        let in_equips = self
            .state
            .core
            .player(player)
            .equips
            .iter()
            .position(|c| *c == card);
        if let Some(pos) = in_equips {
            self.state.core.player_mut(player).equips.remove(pos);
            crate::kernel::revoke_equip_skills(&mut self.state.core, &self.catalog, player, card);
            crate::machine::note_cards_lost(&mut self.state.core, &self.catalog, player, &[card]);
            self.state.core.discard.push(card);
            return;
        }
        panic!("card {card} not in hand or equips of {player}");
    }

    /// 测试/工具：从手牌弃置一张（走正常弃置管线：移出手牌、入弃堆、
    /// 义山记录钩、日志）。
    pub fn debug_discard_card(&mut self, player: PlayerId, card: CardId) -> CommandOutcome {
        if self.state.core.outcome.is_some() {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        if self.state.resolution.open_choice.is_some() {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        let pos = work
            .core
            .player(player)
            .hand
            .iter()
            .position(|c| *c == card)
            .expect("card in hand");
        work.core.player_mut(player).hand.remove(pos);
        work.core.discard.push(card);
        crate::machine::note_cards_lost(&mut work.core, &self.catalog, player, &[card]);
        crate::resolution::push_frame(
            &mut work.resolution,
            EventData::CardsLost {
                player,
                cards: vec![card],
                origins: vec![crate::event::LostOrigin::Hand],
            },
        );
        self.finish_segment(work, log_mark)
    }

    /// 测试布阵：把一张指定花色的牌放到某玩家手牌（从牌库或弃牌堆取；
    /// 用于花色类技能布阵——清正/反间等）。
    pub fn debug_place_suit(&mut self, player: PlayerId, suit: crate::catalog::Suit) -> CardId {
        let card = if let Some(pos) = self
            .state
            .core
            .deck
            .iter()
            .position(|c| self.catalog.cards[*c as usize].suit == suit)
        {
            self.state.core.deck.remove(pos)
        } else {
            let pos = self
                .state
                .core
                .discard
                .iter()
                .position(|c| self.catalog.cards[*c as usize].suit == suit)
                .expect("牌库/弃牌堆中有该花色");
            self.state.core.discard.remove(pos)
        };
        self.state.core.player_mut(player).hand.push(card);
        card
    }

    /// 测试布阵：把一张具名牌放到某玩家手牌（从牌库或弃牌堆取）。
    pub fn debug_place(&mut self, player: PlayerId, name: &str) -> CardId {
        let kind = self.card_kind_by_name(name);
        let from_deck = self
            .state
            .core
            .deck
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind);
        let card = if let Some(pos) = from_deck {
            self.state.core.deck.remove(pos)
        } else if let Some(pos) = self
            .state
            .core
            .discard
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
        {
            self.state.core.discard.remove(pos)
        } else {
            // 上帝视角布阵：最后从任意玩家手牌回收（仅测试用）
            let mut found = None;
            for (i, p) in self.state.core.players.iter_mut().enumerate() {
                if let Some(pos) = p
                    .hand
                    .iter()
                    .position(|c| self.catalog.cards[*c as usize].kind == kind)
                {
                    found = Some(p.hand.remove(pos));
                    break;
                }
                let _ = i;
            }
            found.unwrap_or_else(|| panic!("no {name} anywhere in deck/discard/hands"))
        };
        self.state.core.player_mut(player).hand.push(card);
        card
    }

    /// 上帝视角：按目录名把特定实体牌移入手牌（从牌库或弃牌堆取；
    /// 测试/负载布阵用——同 Kind 多牌种时按名字精确定位）。
    pub fn debug_move_named_card_to_hand(&mut self, player: PlayerId, name: &str) -> CardId {
        let id = self
            .catalog
            .cards
            .iter()
            .position(|d| d.name == name)
            .expect("目录中应存在该名") as CardId;
        let in_deck = self.state.core.deck.iter().position(|c| *c == id);
        match in_deck {
            Some(pos) => {
                self.state.core.deck.remove(pos);
            }
            None => {
                // 弃牌堆，最后从任意玩家手牌回收
                if let Some(pos) = self.state.core.discard.iter().position(|c| *c == id) {
                    self.state.core.discard.remove(pos);
                } else {
                    let mut found = false;
                    for p in self.state.core.players.iter_mut() {
                        if let Some(pos) = p.hand.iter().position(|c| *c == id) {
                            // 仅移出（尾部统一 push——此处重复 push 会产生
                            // 同 id 双份实体，区域守恒炸）
                            p.hand.remove(pos);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        panic!("{name} 不在牌库/弃牌堆/手牌");
                    }
                }
            }
        }
        self.state.core.player_mut(player).hand.push(id);
        id
    }

    /// 调试：来源玩家将一张手牌交给目标（真实 CardsMoved 事件面——
    /// 枳枉等 gain 类技能测试布阵用）。
    pub fn debug_give_hand_card(
        &mut self,
        from: PlayerId,
        to: PlayerId,
        card: CardId,
    ) -> CommandOutcome {
        if self.state.core.outcome.is_some() {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        if self.state.resolution.open_choice.is_some() {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        if !self.state.core.player(from).hand.contains(&card) {
            return CommandOutcome::Rejected(RejectReason::CardNotInHand);
        }
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        work.core.player_mut(from).hand.retain(|x| *x != card);
        work.core.player_mut(to).hand.push(card);
        crate::resolution::push_frame(
            &mut work.resolution,
            EventData::CardsMoved {
                cards: vec![card],
                from,
                to,
            },
        );
        self.finish_segment(work, log_mark)
    }

    /// 上帝视角：牌库全部移入弃牌堆（负载/测试布阵用，
    /// 下一次摸牌/翻判定牌即触发真实 DeckReshuffled 洗回）。
    pub fn debug_drain_deck_to_discard(&mut self) {
        let deck = std::mem::take(&mut self.state.core.deck);
        self.state.core.discard.extend(deck);
    }

    /// 上帝视角清场：全部玩家判定区移入弃牌堆（仅负载/测试布阵用，
    /// 防止闪电转移后同名锦囊滞留他区导致布阵无牌可回收）。
    pub fn debug_sweep_judge_zones(&mut self) {
        for i in 0..self.state.core.players.len() {
            let z = std::mem::take(&mut self.state.core.player_mut(i as u8).judge);
            self.state.core.discard.extend(z);
        }
    }

    /// 测试布阵：清空手牌（移入弃牌堆），保证确定性。
    pub fn debug_clear_hand(&mut self, player: PlayerId) {
        let hand = std::mem::take(&mut self.state.core.player_mut(player).hand);
        self.state.core.discard.extend(hand);
    }

    /// 测试布阵：直接设置体力。
    /// 测试/工具：设置护甲值。
    pub fn debug_set_hujia(&mut self, player: PlayerId, hujia: u8) {
        self.state.core.player_mut(player).hujia = hujia;
    }

    /// 测试/工具：设置机甲血（D.Va jijia，规则 v141）。
    pub fn debug_set_jijia(&mut self, player: PlayerId, hp: u8) {
        self.state.core.player_mut(player).jijia_hp = hp;
    }

    /// 测试入口：改当前回合座位、阶段与轮次（翼轮次门不经完整绕圈）。
    pub fn debug_set_turn(&mut self, current: PlayerId, phase: crate::event::Phase, round: u32) {
        self.state.core.turn.current = current;
        self.state.core.turn.phase = phase;
        self.state.core.turn.round = round;
    }

    /// 测试入口：直接写十殿进度（地藏王分支 / 孟婆旗）。
    pub fn debug_set_boss_progress(&mut self, level: u8, mengpo_spawned: bool, mengpo_died: bool) {
        if let GameMode::Boss { stage, .. } = &mut self.state.core.mode {
            stage.kind = crate::state::BossStageKind::ShidianYanluo;
            stage.level = level;
            stage.mengpo_spawned = mengpo_spawned;
            stage.mengpo_died = mengpo_died;
        }
    }

    /// 调试：覆写身份局身份分布（仅测试用；长度须与座位数一致）。
    pub fn debug_set_identity_roles(&mut self, roles: Vec<crate::state::Role>) {
        assert_eq!(
            roles.len(),
            self.state.core.players.len(),
            "身份数须等于座位数"
        );
        self.state.core.mode = crate::state::GameMode::Identity { roles };
    }

    pub fn debug_set_hp(&mut self, player: PlayerId, hp: i32) {
        self.state.core.player_mut(player).hp = hp;
    }

    /// 测试布阵：授予一名玩家一个永久技能实例。
    /// 测试布阵：按名找一张牌置入离场区（从手牌/牌堆/弃堆回收）。
    pub fn debug_off_court_place(&mut self, player: PlayerId, name: &str) -> CardId {
        let kind = self.card_kind_by_name(name);
        let mut found = None;
        let pos = self.state.core.players[player as usize]
            .hand
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind);
        if let Some(pos) = pos {
            found = Some(self.state.core.players[player as usize].hand.remove(pos));
        } else if let Some(pos) = self
            .state
            .core
            .deck
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
        {
            found = Some(self.state.core.deck.remove(pos));
        } else if let Some(pos) = self
            .state
            .core
            .discard
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
        {
            found = Some(self.state.core.discard.remove(pos));
        }
        let card = found.expect("card available somewhere");
        self.state.core.players[player as usize]
            .off_court
            .push(card);
        card
    }

    pub fn debug_grant(&mut self, player: PlayerId, name: &str) {
        let def = self
            .catalog
            .skill_by_name(name)
            .unwrap_or_else(|| panic!("unknown skill {name}"));
        self.state.core.skills.insert(
            (player, def),
            crate::state::SkillInstance {
                owner: player,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }

    /// 测试/工具：按武将名授予完整技能集（N08 完整武将授予）。
    pub fn debug_grant_general(&mut self, player: PlayerId, name: &str) {
        let skills = content::standard::STANDARD_GENERALS
            .iter()
            .find(|(g, _, _, _, _)| *g == name)
            .unwrap_or_else(|| panic!("unknown general {name}"))
            .2;
        for s in skills {
            self.debug_grant(player, s);
        }
    }

    /// 测试/工具：配置体力上限（N11 将领装配）。
    pub fn debug_set_max_hp(&mut self, player: PlayerId, hp: u32) {
        let p = self
            .state
            .core
            .players
            .get_mut(player as usize)
            .expect("player");
        p.max_hp = hp;
        if p.hp > hp as i32 {
            p.hp = hp as i32;
        }
    }

    /// 测试/工具：配置主将槽（势力/性别资格类测试用——明置状态不变）。
    /// 按目录 def 索引装配将（测试面；Boss 模式 boss 将经 BOSS_GENERALS
    /// 段索引——general_meta 统一解析，标准表之后为 boss 段）。
    pub fn debug_set_general_def(&mut self, player: PlayerId, def: u16) {
        self.state
            .core
            .players
            .get_mut(player as usize)
            .expect("player")
            .general_slots[0] = Some(def);
        apply_general_assembly(&mut self.state.core, &self.catalog, player, 0);
    }

    pub fn debug_set_general(&mut self, player: PlayerId, name: &str) {
        let idx = content::standard::STANDARD_GENERALS
            .iter()
            .position(|(g, _, _, _, _)| *g == name)
            .unwrap_or_else(|| panic!("unknown general {name}"));
        let p = self
            .state
            .core
            .players
            .get_mut(player as usize)
            .expect("player");
        p.general_slots[0] = Some(idx as u16);
    }

    /// 测试/工具：装配武将（P2/N08 装配授予原语）——配置主将槽并按
    /// 武将表授予技能集与体力上限，与国战 `RevealGeneral` 共用
    /// `apply_general_assembly`（授予语义不分叉）。原型模式无命令层
    /// 选将入口，装配类测试经此原语驱动。摘除见 `debug_remove_general`。
    pub fn debug_assemble_general(&mut self, player: PlayerId, name: &str) {
        let idx = content::standard::STANDARD_GENERALS
            .iter()
            .position(|(g, _, _, _, _)| *g == name)
            .unwrap_or_else(|| panic!("unknown general {name}"));
        self.state
            .core
            .players
            .get_mut(player as usize)
            .expect("player")
            .general_slots[0] = Some(idx as u16);
        apply_general_assembly(&mut self.state.core, &self.catalog, player, 0);
    }

    /// 测试/工具：按目录 def 装配主将（扩展/Boss 段）——与
    /// `debug_assemble_general` 同语义（`apply_general_assembly` + 体力按
    /// 上限设置），供扩展/Boss 武将测试驱动。
    pub fn debug_assemble_general_def(&mut self, player: PlayerId, def: u16) {
        self.state
            .core
            .players
            .get_mut(player as usize)
            .expect("player")
            .general_slots[0] = Some(def);
        apply_general_assembly(&mut self.state.core, &self.catalog, player, 0);
        let p = self.state.core.player_mut(player);
        p.hp = p.max_hp as i32;
        p.alive = true;
        p.dying = false;
    }

    /// 测试/工具：摘除主将（装配撤销）——撤销该武将授予的技能实例
    /// （装备离场撤销语义）并清空主将槽。体力上限不回退（装配授予的
    /// 差异登记：原型测试工具不做体力还原）。
    pub fn debug_remove_general(&mut self, player: PlayerId) {
        let p = self
            .state
            .core
            .players
            .get_mut(player as usize)
            .expect("player");
        let Some(g) = p.general_slots[0].take() else {
            return;
        };
        for skill in content::standard::general_skills(g) {
            let def = self
                .catalog
                .skill_by_name(skill)
                .unwrap_or_else(|| panic!("catalog skill {skill}"));
            self.state.core.skills.remove(&(player, def));
        }
    }

    /// 测试/工具：切换已授予技能的启停（N08 动态技能资格）。
    /// E03 翻面原语（测试驱动）：对 `player` 发起翻面——压入
    /// TurnOverBefore 帧并驱动调度到弹栈，前置取消窗口内锁定技
    /// （漩涡面具等）写回取消位则不翻转，返回翻面是否发生。
    pub fn debug_flip(&mut self, player: PlayerId) -> bool {
        let before_down = self.state.core.player(player).face_down;
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        let log_mark = self.log.len();
        let id = work.resolution.next_event_id;
        work.resolution.next_event_id += 1;
        work.resolution.stack.push(crate::event::Frame {
            id,
            parent: None,
            kind: crate::event::EventKind::TurnOverBefore,
            data: crate::event::EventData::TurnOverBefore {
                player,
                cancelled: false,
            },
            stage: crate::event::Stage::BeforeTriggers,
            queue: Vec::new(),
            planned: false,
            intrinsic_done: false,
        });
        match run(
            &mut work.core,
            &mut work.resolution,
            &self.catalog,
            &mut self.log,
            self.time_slice,
        ) {
            StepOutcome::Done => {
                work.core.revision += 1;
                self.state = work;
            }
            // 调试原语同步闭环契约：翻面窗口内不应开选择（当前目录中
            // TurnOverBefore 触发均为锁定技）；分片/预算/待决均视为未闭环。
            other => {
                self.log.truncate(log_mark);
                panic!("debug_flip: 翻面前置窗口未闭环: {other:?}");
            }
        }
        self.state.core.player(player).face_down != before_down
    }

    pub fn debug_set_skill_enabled(&mut self, player: PlayerId, name: &str, enabled: bool) {
        let def = self
            .catalog
            .skill_by_name(name)
            .unwrap_or_else(|| panic!("unknown skill {name}"));
        let inst = self
            .state
            .core
            .skills
            .get_mut(&(player, def))
            .unwrap_or_else(|| panic!("skill {name} not granted to player {player}"));
        inst.enabled = enabled;
    }

    /// 测试/工具：把一张牌移入某玩家判定区（按 ID；跨包无名牌用）。
    pub fn debug_place_judge_id(&mut self, player: PlayerId, card: CardId) {
        // 先从持有区域移除（手/牌库/弃牌堆）
        {
            let p = self.state.core.player_mut(player);
            p.hand.retain(|c| *c != card);
            p.equips.retain(|c| *c != card);
        }
        self.state.core.deck.retain(|c| *c != card);
        self.state.core.discard.retain(|c| *c != card);
        self.state.core.player_mut(player).judge.push(card);
    }

    /// 测试布阵：把一张具名延时锦囊放到某玩家判定区（从牌库或弃牌堆取）。
    pub fn debug_place_judge(&mut self, player: PlayerId, name: &str) -> CardId {
        let card = self.debug_place(player, name);
        let hand = &mut self.state.core.player_mut(player).hand;
        let pos = hand.iter().position(|c| *c == card).expect("just placed");
        hand.remove(pos);
        self.state.core.player_mut(player).judge.push(card);
        card
    }

    /// 测试布阵：把一张具名牌放到牌库顶（下一张摸/判定即该牌）。
    pub fn debug_set_deck_top(&mut self, name: &str) -> CardId {
        let kind = self.card_kind_by_name(name);
        let pos = self
            .state
            .core
            .deck
            .iter()
            .position(|c| self.catalog.cards[*c as usize].kind == kind)
            .or_else(|| {
                // 牌库没有则从弃牌堆回收
                let dpos = self
                    .state
                    .core
                    .discard
                    .iter()
                    .position(|c| self.catalog.cards[*c as usize].kind == kind)?;
                let card = self.state.core.discard.remove(dpos);
                self.state.core.deck.push(card);
                Some(self.state.core.deck.len() - 1)
            })
            .expect("card available somewhere");
        let card = self.state.core.deck.remove(pos);
        self.state.core.deck.push(card);
        card
    }

    /// 测试布阵：按谓词挑一张牌压到牌库顶。
    pub fn debug_set_deck_top_by(
        &mut self,
        pred: impl Fn(&crate::catalog::CardDef) -> bool,
    ) -> CardId {
        let find = |src: &GameState| -> Option<usize> {
            src.core
                .deck
                .iter()
                .position(|c| pred(&self.catalog.cards[*c as usize]))
        };
        let pos = match find(&self.state) {
            Some(pos) => pos,
            None => {
                // 牌库没有 → 弃牌堆 → 其他玩家手牌（上帝视角布阵）
                if let Some(dpos) = self
                    .state
                    .core
                    .discard
                    .iter()
                    .position(|c| pred(&self.catalog.cards[*c as usize]))
                {
                    let card = self.state.core.discard.remove(dpos);
                    self.state.core.deck.push(card);
                    return self.state.core.deck.pop().expect("just pushed");
                }
                let mut found = None;
                for p in self.state.core.players.iter_mut() {
                    if let Some(hpos) = p
                        .hand
                        .iter()
                        .position(|c| pred(&self.catalog.cards[*c as usize]))
                    {
                        found = Some(p.hand.remove(hpos));
                        break;
                    }
                }
                let card = found.expect("card matching predicate available in deck/discard/hands");
                self.state.core.deck.push(card);
                self.state.core.deck.len() - 1
            }
        };
        let card = self.state.core.deck.remove(pos);
        self.state.core.deck.push(card);
        card
    }

    /// 调试：读取判定区。
    pub fn debug_judge(&self, player: PlayerId) -> Vec<CardId> {
        self.state.core.player(player).judge.clone()
    }

    /// E03 翻面原语：读取武将牌朝向。
    pub fn debug_face_down(&self, player: PlayerId) -> bool {
        self.state.core.player(player).face_down
    }

    /// E03 测试原语：强制设置武将牌朝向（不经事件流，用于状态重置）。
    pub fn debug_set_face_down(&mut self, player: PlayerId, face_down: bool) {
        self.state.core.player_mut(player).face_down = face_down;
    }

    /// E03 测试辅助：移除玩家装备区的一张牌（离场撤销装备技）。
    pub fn debug_remove_equip(&mut self, player: PlayerId, card: CardId) {
        let equips = &mut self.state.core.players[player as usize].equips;
        equips.retain(|c| *c != card);
        crate::kernel::revoke_equip_skills(&mut self.state.core, &self.catalog, player, card);
    }

    pub fn debug_discard_len(&self) -> usize {
        self.state.core.discard.len()
    }

    /// 测试钩子：设置下一命令段的调度预算上限（默认 4096 步/段）。
    ///
    /// 预算是**每段**硬上限（ADR 0708）：段开始时重置，不跨段累计——
    /// 旧实现预算随状态全局递减，长对局必然耗尽。覆盖值不入状态/哈希。
    pub fn debug_set_budget(&mut self, budget: u32) {
        self.debug_budget = Some(budget);
    }

    pub fn debug_hand(&self, player: PlayerId) -> Vec<CardId> {
        self.state.core.player(player).hand.clone()
    }

    pub fn debug_equips(&self, player: PlayerId) -> Vec<CardId> {
        self.state.core.player(player).equips.clone()
    }

    pub fn debug_hp(&self, player: PlayerId) -> i32 {
        self.state.core.player(player).hp
    }

    /// 手牌上限查询（测试面：hand_limit 含扩展技修饰）。
    pub fn hand_limit_for_debug(&self, player: PlayerId) -> usize {
        self.state.core.hand_limit(&self.catalog, player)
    }

    pub fn debug_alive(&self, player: PlayerId) -> bool {
        self.state.core.player(player).alive
    }

    pub fn debug_has_skill(&self, player: PlayerId, name: &str) -> bool {
        match self.catalog.skill_by_name(name) {
            Some(def) => self.state.core.has_skill(player, def),
            None => false,
        }
    }

    /// 调试/后续内容迁移入口：直接注入一段属性伤害结算
    /// （走与命令相同的提交/回滚管线）。
    pub fn debug_deal_damage(
        &mut self,
        source: Option<PlayerId>,
        target: PlayerId,
        amount: u32,
        nature: crate::event::DamageNature,
    ) -> CommandOutcome {
        if self.state.core.outcome.is_some() {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        if self.state.resolution.open_choice.is_some() {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        let Ok(amount)=i32::try_from(amount) else {return CommandOutcome::Rejected(RejectReason::IllegalTargets);};
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        crate::resolution::push_frame(
            &mut work.resolution,
            EventData::Damage(crate::event::DamageData {
                    settlement: Default::default(),
                source,
                target,
                amount,
                nature,
                prevented: false,
                redirect_chain: Vec::new(),
                card: None,
                from_chain: false,
            }),
        );
        self.finish_segment(work, log_mark)
    }

    /// 调试：带卡伤害（T8 增量 16 filter 定向回归——"ordinary-trick
    /// damage"事件卡面；同 debug_deal_damage 管线，card 注入帧载荷）。
    pub fn debug_deal_damage_with_card(
        &mut self,
        source: Option<PlayerId>,
        target: PlayerId,
        amount: u32,
        nature: crate::event::DamageNature,
        card: crate::ids::CardId,
    ) -> CommandOutcome {
        if self.state.core.outcome.is_some() {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        if self.state.resolution.open_choice.is_some() {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        let Ok(amount)=i32::try_from(amount) else {return CommandOutcome::Rejected(RejectReason::IllegalTargets);};
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        crate::resolution::push_frame(
            &mut work.resolution,
            EventData::Damage(crate::event::DamageData {
                    settlement: Default::default(),
                source,
                target,
                amount,
                nature,
                prevented: false,
                redirect_chain: Vec::new(),
                card: Some(card),
                from_chain: false,
            }),
        );
        self.finish_segment(work, log_mark)
    }

    /// 调试：回复体力（发 Recover 事件走完整结算——积财等回复触发技
    /// 依赖此通道；机械回血经 Ctx.recover 同款语义：钳上限、实际回复量
    /// 入事件）。
    pub fn debug_recover(&mut self, player: PlayerId, amount: u32) -> CommandOutcome {
        if self.state.core.outcome.is_some() {
            return CommandOutcome::Rejected(RejectReason::MatchOver);
        }
        if self.state.resolution.open_choice.is_some() {
            return CommandOutcome::Rejected(RejectReason::ChoicePending);
        }
        let log_mark = self.log.len();
        let mut work = self.state.clone();
        work.resolution.budget = self.debug_budget.unwrap_or(DEFAULT_BUDGET);
        crate::resolution::push_frame(
            &mut work.resolution,
            EventData::Recover {
                player,
                amount,
                by_card: None,
            },
        );
        self.finish_segment(work, log_mark)
    }

    /// 调试：读取连环状态。
    pub fn debug_linked(&self, player: PlayerId) -> bool {
        self.state.core.player(player).linked
    }
    pub(super) fn card_kind_by_name(&self, name: &str) -> CardKind {
        self.catalog
            .cards
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.kind)
            .unwrap_or_else(|| panic!("unknown card name {name}"))
    }
}
