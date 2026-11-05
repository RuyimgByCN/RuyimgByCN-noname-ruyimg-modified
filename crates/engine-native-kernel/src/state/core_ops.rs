//! GameStateCore 的查询与变更方法（座位、区域、体力、技能实例等）。

use super::*;

impl GameStateCore {
    /// Pinned isEnemyOf/isFriendOf semantics for mode facts represented by this kernel.
    pub fn source_is_enemy_of(
        &self,
        _catalog: &crate::catalog::Catalog,
        a: PlayerId,
        b: PlayerId,
    ) -> Option<bool> {
        if a == b {
            return Some(false);
        }
        match &self.mode {
            GameMode::Boss { .. } => Some(self.is_boss_side(a) != self.is_boss_side(b)),
            GameMode::Versus { teams, .. } if teams.len() == 2 => {
                let a = teams.iter().position(|t| t.contains(&a))?;
                let b = teams.iter().position(|t| t.contains(&b))?;
                Some(a != b)
            }
            GameMode::Doudizhu { lord } => Some((a == *lord) != (b == *lord)),
            // Source has no side boolean in standard Identity: friendship falls back to self.
            GameMode::Identity { .. } => Some(true),
            GameMode::Kingdoms => Some(!self.kingdom_friends(a, b)),
            _ => None,
        }
    }

    pub fn kingdom_friends(&self, a: PlayerId, b: PlayerId) -> bool {
        if a == b
            || self.player(a).yexinjia_friends.contains(&b)
            || self.player(b).yexinjia_friends.contains(&a)
        {
            return true;
        }
        matches!((&self.player(a).kingdom_identity,&self.player(b).kingdom_identity),(KingdomIdentity::Declared(x),KingdomIdentity::Declared(y))if x==y)
    }
    pub fn set_kingdom_identity(
        &mut self,
        player: PlayerId,
        identity: KingdomIdentity,
    ) -> Result<(), &'static str> {
        if !matches!(self.mode, GameMode::Kingdoms) || player as usize >= self.players.len() {
            return Err("kingdom_identity_context");
        }
        if matches!(&identity,KingdomIdentity::Declared(name)if name.is_empty()||name=="unknown"||name=="ye")
        {
            return Err("kingdom_identity_name");
        }
        self.player_mut(player).kingdom_identity = identity;
        Ok(())
    }
    pub fn set_yexinjia_friends(
        &mut self,
        player: PlayerId,
        friends: Vec<PlayerId>,
    ) -> Result<(), &'static str> {
        if !matches!(self.mode, GameMode::Kingdoms) || player as usize >= self.players.len() {
            return Err("kingdom_friend_context");
        }
        let unique: std::collections::BTreeSet<_> = friends.iter().copied().collect();
        if unique.len() != friends.len()
            || friends.contains(&player)
            || friends.iter().any(|p| *p as usize >= self.players.len())
        {
            return Err("kingdom_friend_targets");
        }
        self.player_mut(player).yexinjia_friends = friends;
        Ok(())
    }
    pub fn declare_revealed_kingdom(
        &mut self,
        catalog: &crate::catalog::Catalog,
        player: PlayerId,
    ) {
        if !matches!(self.mode, GameMode::Kingdoms)
            || self.player(player).kingdom_identity != KingdomIdentity::Unknown
        {
            return;
        }
        let Some(faction) = self.declared_faction(catalog, player) else {
            return;
        };
        let name = match faction {
            crate::catalog::Faction::Wei => "wei",
            crate::catalog::Faction::Shu => "shu",
            crate::catalog::Faction::Wu => "wu",
            crate::catalog::Faction::Qun => "qun",
        };
        let population = self
            .players
            .iter()
            .filter(|p| matches!(&p.kingdom_identity,KingdomIdentity::Declared(n)if n==name))
            .count();
        let identity = if (population + 1) * 2 <= self.players.len() {
            KingdomIdentity::Declared(name.into())
        } else {
            KingdomIdentity::Ye
        };
        self.set_kingdom_identity(player, identity)
            .expect("native reveal identity context");
    }
    pub fn can_gain_card(
        &self,
        catalog: &crate::catalog::Catalog,
        source: PlayerId,
        target: PlayerId,
        card: crate::ids::CardId,
    ) -> bool {
        if source as usize >= self.players.len()
            || target as usize >= self.players.len()
            || card as usize >= catalog.cards.len()
        {
            return false;
        }
        let p = self.player(target);
        if !p.hand.contains(&card) && !p.equips.contains(&card) {
            return false;
        }
        let ctx = crate::catalog::QueryCtx {
            state: self,
            catalog,
        };
        !self.skills.iter().any(|((owner, def), inst)| {
            *owner == target
                && inst.enabled
                && catalog.skills[*def as usize]
                    .behavior
                    .can_be_gained(&ctx, *owner, card, source, target)
                    == Some(false)
        })
    }
    pub fn player(&self, p: PlayerId) -> &PlayerState {
        &self.players[p as usize]
    }

    pub fn player_mut(&mut self, p: PlayerId) -> &mut PlayerState {
        &mut self.players[p as usize]
    }

    pub fn alive_players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive)
            .map(|(i, _)| i as PlayerId)
    }

    pub fn alive_count(&self) -> usize {
        self.players.iter().filter(|p| p.alive).count()
    }

    /// 身份局的身份表（非身份局为 None）。
    pub fn identity_roles(&self) -> Option<&[Role]> {
        match &self.mode {
            GameMode::Identity { roles } => Some(roles),
            GameMode::Prototype
            | GameMode::Kingdoms
            | GameMode::Versus { .. }
            | GameMode::Doudizhu { .. }
            | GameMode::Boss { .. }
            | GameMode::Kof { .. } => None,
        }
    }

    /// KOF 会话状态（非 KOF 为 None）。
    pub fn kof_session(&self) -> Option<(&[Vec<u16>], &[u32; 2], u32)> {
        match &self.mode {
            GameMode::Kof { pools, wins, games } => Some((pools, wins, *games)),
            _ => None,
        }
    }

    /// Boss 模式参数：boss 座位与重整阈值（非 Boss 为 None）。
    pub fn boss_mode(&self) -> Option<(PlayerId, u8)> {
        match &self.mode {
            GameMode::Boss {
                boss, chongzheng, ..
            } => Some((*boss, *chongzheng)),
            _ => None,
        }
    }

    /// Boss 模式阵营：boss 座位与助战 fellow 为 boss 方，其余为挑战者
    /// 方（非 Boss 模式一律 false——神恩光环仅 Boss 场生效）。
    pub fn is_boss_side(&self, p: PlayerId) -> bool {
        match &self.mode {
            GameMode::Boss { boss, fellows, .. } => p == *boss || fellows.contains(&p),
            _ => false,
        }
    }

    /// Boss 模式助战座位表（非 Boss 为 None）。
    pub fn boss_fellows(&self) -> Option<&[PlayerId]> {
        match &self.mode {
            GameMode::Boss { fellows, .. } => Some(fellows),
            _ => None,
        }
    }

    /// Boss 关卡流程态（非 Boss 为 None）。
    pub fn boss_stage(&self) -> Option<&BossStageState> {
        match &self.mode {
            GameMode::Boss { stage, .. } => Some(stage),
            _ => None,
        }
    }

    /// 通用目标规则（swd 批 136 极略）：按牌种给出 (min, max, 合法目标)。
    /// 与 PlayCard 校验同口径的**近似镜像**（忽略逐技豁免/范围豁免等；
    /// 登记差异：仅供机器发起「非手牌用牌」时选目标，命令路径仍走原校验）。
    pub fn legal_targets(
        &self,
        catalog: &crate::catalog::Catalog,
        kind: crate::catalog::CardKind,
        player: PlayerId,
    ) -> (u8, u8, Vec<PlayerId>) {
        use crate::catalog::CardKind as K;
        let alive: Vec<PlayerId> = self.alive_players().collect();
        let others: Vec<PlayerId> = alive.iter().copied().filter(|p| *p != player).collect();
        let range = self.attack_range(catalog, player);
        let in_range = |t: PlayerId| self.computed_distance(catalog, player, t) <= range;
        match kind {
            K::Sha | K::FireSha | K::LeiSha => {
                let v: Vec<PlayerId> = others.iter().copied().filter(|t| in_range(*t)).collect();
                (if v.is_empty() { 1 } else { 1 }, 1, v)
            }
            K::Tao => {
                let wounded = self.player(player).hp < self.player(player).max_hp as i32;
                (1, 1, if wounded { vec![player] } else { Vec::new() })
            }
            K::Jiu => (1, 1, vec![player]),
            K::Wuzhong => (0, 0, Vec::new()),
            K::Wugu => (0, 0, alive.clone()),
            K::Taoyuan => (0, 0, alive.clone()),
            K::SavageAttack | K::ArcheryAttack => (0, 0, others.clone()),
            K::Duel => (1, 1, others.clone()),
            K::Snatch | K::Dismantle => {
                let v: Vec<PlayerId> = others
                    .iter()
                    .copied()
                    .filter(|t| {
                        in_range(*t)
                            && (!self.player(*t).hand.is_empty()
                                || !self.player(*t).equips.is_empty())
                    })
                    .collect();
                (1, 1, v)
            }
            K::Chain => (1, 2, alive.clone()),
            K::Le | K::Bing => {
                let v: Vec<PlayerId> = others
                    .iter()
                    .copied()
                    .filter(|t| {
                        !self
                            .player(*t)
                            .judge
                            .iter()
                            .any(|c| catalog.card_kind(*c) == kind)
                    })
                    .collect();
                (1, 1, v)
            }
            K::ShanDian => (1, 1, vec![player]),
            K::Jiedao => {
                let v: Vec<PlayerId> = others
                    .iter()
                    .copied()
                    .filter(|t| {
                        self.player(*t).equips.iter().any(|c| {
                            catalog.card_slot_of(self, *c)
                                == Some(crate::catalog::EquipSlot::Weapon)
                        })
                    })
                    .collect();
                (1, 1, v)
            }
            _ => (0, 0, Vec::new()),
        }
    }

    /// 关卡流程态可变访问（浴火置 loopType；非 Boss 返回 None）。
    pub fn boss_stage_mut(&mut self) -> Option<&mut BossStageState> {
        match &mut self.mode {
            GameMode::Boss { stage, .. } => Some(stage),
            _ => None,
        }
    }

    /// 禁将查询（B0 空实现：默认禁将表空 → 恒未禁；非 Boss 恒 false）。
    /// 写入与选将拦截留给 B3 `T11-BOSS-HUANREN`。
    pub fn is_boss_general_banned(&self, general: u16) -> bool {
        match &self.mode {
            GameMode::Boss { stage, .. } => stage.banned_generals.contains(&general),
            _ => false,
        }
    }

    /// 孟婆乱入旗只读（B0 默认 false；写入留给 B3）。
    pub fn boss_mengpo_spawned(&self) -> bool {
        match &self.mode {
            GameMode::Boss { stage, .. } => stage.mengpo_spawned,
            _ => false,
        }
    }

    /// 孟婆已死旗只读（B0 默认 false；写入留给 B3）。
    pub fn boss_mengpo_died(&self) -> bool {
        match &self.mode {
            GameMode::Boss { stage, .. } => stage.mengpo_died,
            _ => false,
        }
    }

    /// 斗地主模式的地主座位（非斗地主为 None）。
    pub fn doudizhu_lord(&self) -> Option<PlayerId> {
        match &self.mode {
            GameMode::Doudizhu { lord } => Some(*lord),
            _ => None,
        }
    }

    /// 对决模式的队伍表（非对决为 None）。队伍互斥且并集=全部座位。
    pub fn versus_teams(&self) -> Option<&[Vec<PlayerId>]> {
        match &self.mode {
            GameMode::Versus { teams, .. } => Some(teams),
            _ => None,
        }
    }

    /// 对决模式的替补武将池（非对决为 None；与队伍同队序，可为空池）。
    pub fn versus_substitutes(&self) -> Option<&[Vec<u16>]> {
        match &self.mode {
            GameMode::Versus { substitutes, .. } => Some(substitutes),
            _ => None,
        }
    }

    /// 对决模式：座位 `p` 的队伍下标（非对决/不在队中为 None）。
    pub fn versus_team_of(&self, p: PlayerId) -> Option<usize> {
        self.versus_teams()?.iter().position(|t| t.contains(&p))
    }

    /// 国战（N11 M3 门控）：座位 `p` 的**已声明**势力——
    /// 主将明置按主将势力；主将暗置而副将明置按副将势力
    /// （半明置）；双暗置或槽空 → None（势力未定）。
    pub fn declared_faction(
        &self,
        catalog: &crate::catalog::Catalog,
        p: PlayerId,
    ) -> Option<crate::catalog::Faction> {
        let pl = self.player(p);
        if pl.revealed_generals[0] {
            catalog
                .general_meta(pl.general_slots[0]?)
                .map(|(_, f, ..)| f)
        } else if pl.revealed_generals[1] {
            catalog
                .general_meta(pl.general_slots[1]?)
                .map(|(_, f, ..)| f)
        } else {
            None
        }
    }

    /// 对局内可见势力：国战取明置势力（暗置不出势力）；其余模式
    /// （身份/原型）主将势力直接可见——标准玩法无暗置概念，护驾/
    /// 激将等势力资格按此判定。
    pub fn effective_faction(
        &self,
        catalog: &crate::catalog::Catalog,
        p: PlayerId,
    ) -> Option<crate::catalog::Faction> {
        match self.mode {
            GameMode::Kingdoms => self.declared_faction(catalog, p),
            _ => {
                let pl = self.player(p);
                pl.general_slots[0].and_then(|g| catalog.general_meta(g).map(|(_, f, ..)| f))
            }
        }
    }

    /// 大势力判定（zhenfa 地载阵首卡，规则 v110；上游国战 isMajor 同
    /// 口径简化）：effective_faction 非空者按势力统计存活数，人数最大
    /// 的势力（并列同算）为大势力；未明置/无主将者不属大势力。上游
    /// forceMajor 技能标特例未建模（登记）。
    pub fn is_major_faction(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
        let mut counts: std::collections::BTreeMap<u8, u32> = Default::default();
        for q in self.alive_players() {
            if let Some(f) = self.effective_faction(catalog, q) {
                *counts.entry(f as u8).or_insert(0) += 1;
            }
        }
        let Some(max) = counts.values().copied().max() else {
            return false;
        };
        match self.effective_faction(catalog, p) {
            Some(f) => counts.get(&(f as u8)) == Some(&max) && max >= 1,
            None => false,
        }
    }

    /// 主公资格（zhu 技能：护驾/激将）：身份局要求主公身份；
    /// 无身份体系的模式持技即可。
    pub fn is_lord(&self, p: PlayerId) -> bool {
        match self.identity_roles() {
            Some(roles) => roles.get(p as usize) == Some(&Role::Lord),
            None => true,
        }
    }

    /// 座位**最短**距离（顺/逆时针取小；同座 0）。
    ///
    /// NC01 修正：旧实现用 `u8::wrapping_sub` 后在 256 上取模再 `% n`，
    /// 5 人局 `d(0,1)` 会因 `0u8.wrapping_sub(1)=255 → 255%5=0` 得到 **0**
    /// （审计反例）。现改为有符号取模（与 `seat_distance_alive` 同口径）。
    ///
    /// 语义边界（勿混用）：
    /// - **最短距离**（本函数）：判断相邻、射程、±1 范围等；
    /// - **顺座次序**（[`Self::seat_steps_from`]）：触发结算顺序、按座次轮转；
    /// - **存活座位环**（[`Self::seat_distance_alive`] / [`Self::ring_next`]）：
    ///   死亡玩家不占位的距离与轮转。
    pub fn seat_distance(&self, a: PlayerId, b: PlayerId) -> u8 {
        let n = self.players.len() as i16;
        if n <= 0 {
            return 0;
        }
        let a = (a as i16).rem_euclid(n);
        let b = (b as i16).rem_euclid(n);
        let cw = (b - a).rem_euclid(n) as u8;
        let ccw = (a - b).rem_euclid(n) as u8;
        cw.min(ccw)
    }

    /// **顺座次序**偏移：从 `from` 起沿座位环顺时针到 `p` 的步数（0..n-1）。
    ///
    /// 用于触发结算顺序、按座次轮转等**顺序**语义；不要用最短距离代替
    /// （4 人局最短距离序为 `[0,1,3,2]`，顺座次序为 `[0,1,2,3]`，审计反例）。
    pub fn seat_steps_from(&self, from: PlayerId, p: PlayerId) -> u8 {
        let n = self.players.len();
        if n == 0 {
            return 0;
        }
        // 以**座位环**位置为准（swapSeat 后座位次序随之改变）
        let pa = self.ring_pos(from) % n;
        let pb = self.ring_pos(p) % n;
        ((pb + n - pa) % n) as u8
    }

    // ---- 座位环（T19b/ADR 0781）----

    /// p 在座位环中的位置（环长度与玩家数不符时退回恒等位）。
    pub fn ring_pos(&self, p: PlayerId) -> usize {
        if self.seat_ring.len() == self.players.len() {
            self.seat_ring
                .iter()
                .position(|x| *x == p)
                .unwrap_or(p as usize)
        } else {
            p as usize
        }
    }

    /// 座位环中 p 的下一位（不计存活；出界退恒等）。
    pub fn ring_next(&self, p: PlayerId) -> PlayerId {
        let n = self.players.len() as PlayerId;
        let pos = self.ring_pos(p);
        self.seat_ring
            .get((pos + 1) % (n as usize))
            .copied()
            .unwrap_or((p + 1) % n)
    }

    /// 座位环中 p 的上一位（不计存活）。
    pub fn ring_prev(&self, p: PlayerId) -> PlayerId {
        let n = self.players.len() as PlayerId;
        let pos = self.ring_pos(p);
        if pos == 0 {
            *self.seat_ring.last().unwrap_or(&(n - 1))
        } else {
            self.seat_ring
                .get(pos - 1)
                .copied()
                .unwrap_or((p + n - 1) % n)
        }
    }

    /// swapSeat 座位重排（T19b，规则 v201）：交换 a/b 的环位置（PlayerId
    /// 身份不变，座位语义即时生效；上游 game.swapSeat 同口径）。
    pub fn swap_seat(&mut self, a: PlayerId, b: PlayerId) {
        if a == b || self.seat_ring.len() != self.players.len() {
            return;
        }
        let pa = self.ring_pos(a);
        let pb = self.ring_pos(b);
        self.seat_ring.swap(pa, pb);
    }

    /// 计算距离（含 ±1 马修饰，下限 1；座位距离只计存活玩家）。
    /// 上游/旧线一致性待核对，见覆盖清单行为差异表。
    pub fn computed_distance(
        &self,
        catalog: &crate::catalog::Catalog,
        a: PlayerId,
        b: PlayerId,
    ) -> u32 {
        use crate::catalog::CardKind;
        // 链钩（U05 ow 路霸，规则 v128）：本回合被钩者——路霸算到其
        // 距离恒 1（上游 liangou2 mod globalTo -Infinity 经 get.distance
        // 下限钳制同口径；仅路霸出牌阶段可钩住，换回合解除）
        if a == self.turn.current && self.turn.liangou_hook == Some(b) {
            return 1;
        }
        // 跳跃（U05 ow 温斯顿，规则 v137）：本回合跳跃目标——自己算到
        // 其距离恒 1（上游 feitiao2 mod globalFrom -Infinity 同口径；
        // 仅自己回合内置位，换回合解除）
        if a == self.turn.current && self.turn.feitiao_target == Some(b) {
            return 1;
        }
        // 月行（U05 xianjian 龙幽，规则 v159）：距离基数交换——持对偶
        // 者以配对者座位为基数（上游 yuexing2 mod globalFrom 的 pure 距
        // 离代换同口径；登记差异：代换仅座位基数，马匹等修饰照常叠加）
        let base = match self.player(a).yuexing_pair {
            Some(p) if p != a => self.seat_distance_alive(p, b) as u32,
            _ => self.seat_distance_alive(a, b) as u32,
        };
        let plus = self
            .player(b)
            .equips
            .iter()
            .filter(|c| catalog.cards[**c as usize].kind == CardKind::HorsePlus)
            .count() as u32;
        let minus = self
            .player(a)
            .equips
            .iter()
            .filter(|c| catalog.cards[**c as usize].kind == CardKind::HorseMinus)
            .count() as u32;
        // 马术（马超）：锁定技——计算与他人的距离时始终 -1（下限 1）
        let mashu = catalog
            .skill_by_name(crate::vocabulary::names::MA_SHU)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let minus = minus + u32::from(mashu);
        // 凶兽（Boss 模式共有技，规则 v208）：锁定技——你与其他角色
        // 距离 -1（上游 boss_xiongshou mod globalFrom distance-1 同口径）
        let xiongshou = catalog
            .skill_by_name(crate::vocabulary::names::BOSS_XIONGSHOU)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let minus = minus + u32::from(xiongshou);
        // 疾风（卢承轩，swd 武将面批㉘）：锁定技——计算与他人的距离 -X
        //（X=自己已损失体力，上游 attackFrom distance += hp-maxHp 同口径）
        let ljifeng = catalog
            .skill_by_name("ljifeng")
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let ljifeng_minus = if ljifeng {
            (self.player(a).max_hp as i32 - self.player(a).hp).max(0) as u32
        } else {
            0
        };
        // 破晓（张烈，swd 武将面批㊞）：锁定技——无武器时计算与他人的
        // 距离 -1（上游 attackFrom distance-1 同口径）
        let poxiao_minus = if catalog
            .skill_by_name("poxiao")
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false)
            && !self
                .player(a)
                .equips
                .iter()
                .any(|c| catalog.card_kind(*c) == crate::catalog::CardKind::Weapon)
        {
            1
        } else {
            0
        };
        // 妖兽（Boss 神兽共有技，规则 v221）：锁定技——你与其他角色
        // 距离 -2（上游 boss_yaoshou mod globalFrom distance-2 同口径）
        let yaoshou = catalog
            .skill_by_name(crate::vocabulary::names::BOSS_YAOSHOU)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let minus = minus + 2 * u32::from(yaoshou) + ljifeng_minus + poxiao_minus;
        // 义从（公孙瓒）：锁定技——你计算与其他角色的距离 -1；其他角色
        // 计算与你的距离 +1（你的体力值不大于 2 时）。
        let yicong_from = catalog
            .skill_by_name(crate::vocabulary::names::YI_CONG)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let yicong_to = catalog
            .skill_by_name(crate::vocabulary::names::YI_CONG)
            .map(|d| {
                self.skills.get(&(b, d)).map(|i| i.enabled).unwrap_or(false)
                    && self.player(b).hp <= 2
            })
            .unwrap_or(false);
        let mut minus = minus + u32::from(yicong_from);
        let mut plus = plus + u32::from(yicong_to);
        // 谛听（Boss 地藏王，规则 v274）：锁定技——你计算与其他角色的
        // 距离 -1；其他角色计算与你的距离 +1（上游 boss_diting mod
        // globalFrom/-1+globalTo/+1 同口径；义从同位）
        let diting_from = catalog
            .skill_by_name(crate::vocabulary::names::BOSS_DITING)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let diting_to = catalog
            .skill_by_name(crate::vocabulary::names::BOSS_DITING)
            .map(|d| self.skills.get(&(b, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        minus += u32::from(diting_from);
        plus += u32::from(diting_to);
        // 魂身（swd 胡月，规则 v296）：他人算向持有者距离 +计数
        //（上游 globalTo distance + to.storage.hyunshen 同口径）
        if let Some(def) = catalog.skill_by_name("hyunshen") {
            if self
                .skills
                .get(&(b, def))
                .map(|i| i.enabled)
                .unwrap_or(false)
            {
                plus += self.player(b).hyunshen_count as u32;
            }
        }
        // 平恤（swd 秦叔宝，swd 武将面批 23）：全局距离字段技——场上有
        // 任一存活持技者时，攻击方无武器则距离 -1、防守方无防具则距离
        // +1（上游 pingxu mod globalFrom/globalTo 全局字段同口径）
        let pingxu = catalog
            .skill_by_name("pingxu")
            .map(|d| {
                self.skills
                    .iter()
                    .any(|((o, dd), i)| *dd == d && i.enabled && self.player(*o).alive)
            })
            .unwrap_or(false);
        if pingxu {
            let a_no_weapon = !self.player(a).equips.iter().any(|c| {
                catalog.cards[*c as usize].kind.equip_slot()
                    == Some(crate::catalog::EquipSlot::Weapon)
            });
            let b_no_armor = !self.player(b).equips.iter().any(|c| {
                catalog.cards[*c as usize].kind.equip_slot()
                    == Some(crate::catalog::EquipSlot::Armor)
            });
            minus += u32::from(a_no_weapon);
            plus += u32::from(b_no_armor);
        }
        // 抗yin2（U07 玩点论杀留赞，规则 v114）：回合内临时——持有者算
        // 到其他角色距离 -X（上游 kangyin2 mod globalFrom 同口径）
        if let Some((o, x)) = self.turn.kangyin2 {
            if o == a {
                minus += x;
            }
        }
        // 突击（U07 ow 士兵76，规则 v118）：锁定——持有者回合内每已使
        // 用一张牌，进攻距离 -1（上游 tuji mod globalFrom countUsed 同
        // 口径）
        if let Some(def) = catalog.skill_by_name(crate::vocabulary::host::WUDIAN_TUJI_KEY) {
            let tuji = self
                .skills
                .get(&(a, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            if tuji && self.turn.current == a && self.turn.cards_used_play_phase > 0 {
                minus += self.turn.cards_used_play_phase;
            }
        }
        // 绝道（U07 玩点论杀刘焉，规则 v116）：锁定——**持技者**横置时
        // 双向距离 +1（上游 mod 挂在 juedao 技能上：globalFrom 查 from
        // 的技能、globalTo 查 to 的技能——非持技者的横置不影响距离）
        if let Some(def) = catalog.skill_by_name(crate::vocabulary::host::WUDIAN_JUEDAO_KEY) {
            let juedao_a = self
                .skills
                .get(&(a, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            let juedao_b = self
                .skills
                .get(&(b, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            if juedao_a && self.player(a).linked {
                plus += 1;
            }
            if juedao_b && self.player(b).linked {
                plus += 1;
            }
        }
        // 车阵（U07 玩点论杀田豫，规则 v115）：锁定——有装备时你算到
        // 他人 -1；目标无装备时其被算到 +1（上游 chezhen mod globalFrom/
        // globalTo 同口径）
        if let Some(def) = catalog.skill_by_name(crate::vocabulary::host::WUDIAN_CHEZHEN_KEY) {
            let chezhen_a = self
                .skills
                .get(&(a, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            let chezhen_b = self
                .skills
                .get(&(b, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            if chezhen_a && !self.player(a).equips.is_empty() {
                minus += 1;
            }
            if chezhen_b && self.player(b).equips.is_empty() {
                plus += 1;
            }
        }
        // 带火风轮/霞云彩绫（U05 swd 第十四批，规则 v98）：命名距离
        // 修饰（上游 distance globalFrom/globalTo 同口径）——
        // 带火风轮：持有者算到他人 -2、他人算到持有者 -1；
        // 霞云彩绫：持有者算到他人 +1、他人算到持有者 +2。
        let count_named = |equips: &[crate::ids::CardId], kind: CardKind| -> u32 {
            equips
                .iter()
                .filter(|c| catalog.cards[**c as usize].kind == kind)
                .count() as u32
        };
        let minus = minus
            + 2 * count_named(&self.player(a).equips, CardKind::Daihuofenglun)
            + count_named(&self.player(b).equips, CardKind::Daihuofenglun);
        let plus = plus
            + count_named(&self.player(a).equips, CardKind::Xiayuncailing)
            + 2 * count_named(&self.player(b).equips, CardKind::Xiayuncailing);
        // 灵矢（swd 计亭，swd deferred 消解批 113）：锁定——装备区每有一
        // 张牌，攻击范围 +2（上游 lingshi mod attackFrom distance -=
        // countCards("e")*2 同口径；攻击方一侧，非被算方；下限 1 由末尾
        // 钳制）
        let lingshi_minus = catalog
            .skill_by_name("lingshi")
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false);
        let minus = minus
            + if lingshi_minus {
                2 * self.player(a).equips.len() as u32
            } else {
                0
            };
        // 脉轮（swd 批 137，规则 v316）：7=双方向距离 -1、8=双方向 +1
        //（上游 mailun7/8 mod globalFrom/globalTo distance∓1——两端各自生效）
        let mailun_delta: i32 = [a, b]
            .iter()
            .map(|p| match self.player(*p).mailun_mode {
                7 => -1,
                8 => 1,
                _ => 0,
            })
            .sum();
        // 每步·距离（L27 批 3，`new_meibu_range`）：临时——到「每步」弃牌者
        // 的距离视为 1（上游 mod.globalFrom -Infinity → 下限钳制同口径；
        // 目标座位存于标记）
        if let Some(def) = catalog.skill_by_name(crate::vocabulary::host::L27_NEW_MEIBU_RANGE_KEY) {
            let enabled = self
                .skills
                .get(&(a, def))
                .map(|i| i.enabled)
                .unwrap_or(false);
            if enabled && self.vm_aux.marks.get(&(a, u32::from(def))).copied() == Some(i32::from(b))
            {
                return 1;
            }
        }
        // 撷芳（L27 选将美化批 1，`xiefang`）：锁定——你计算与其他角色的
        // 距离 -（场上存活**女性**角色数；上游 mod.globalFrom distance -
        // countPlayer(hasSex('female')) 同口径；下限 1 由末尾钳制）
        let xiefang_minus = if catalog
            .skill_by_name(crate::vocabulary::host::L27_XIEFANG_KEY)
            .map(|d| self.skills.get(&(a, d)).map(|i| i.enabled).unwrap_or(false))
            .unwrap_or(false)
        {
            (0..self.players.len() as u8)
                .filter(|p| {
                    self.player(*p).alive
                        && self.effective_gender(*p) == crate::vocabulary::generals::Gender::Female
                })
                .count() as u32
        } else {
            0
        };
        let minus = minus + xiefang_minus;
        let total = (base + plus).saturating_sub(minus) as i32 + mailun_delta;
        total.max(1) as u32
    }

    /// 攻击范围：武器射程（泛型 Weapon 走 CardDef.range，命名武器走
    /// 牌种固定射程），无武器为 1。
    pub fn attack_range(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> u32 {
        use crate::catalog::CardKind;
        self.player(p)
            .equips
            .iter()
            .filter_map(|c| {
                let def = &catalog.cards[*c as usize];
                if def.kind == CardKind::Weapon {
                    Some(def.range)
                } else {
                    def.kind.named_weapon_range()
                }
            })
            .max()
            .unwrap_or(1)
    }

    /// 存活玩家间的座位最短距离（死亡玩家的座位不计数）。
    pub fn seat_distance_alive(&self, a: PlayerId, b: PlayerId) -> u8 {
        let alive: Vec<PlayerId> = self.alive_players().collect();
        let n = alive.len() as i16;
        let ia = alive.iter().position(|p| *p == a).expect("a must be alive") as i16;
        let ib = alive.iter().position(|p| *p == b).expect("b must be alive") as i16;
        let cw = (ib - ia).rem_euclid(n) as u8;
        let ccw = (ia - ib).rem_euclid(n) as u8;
        cw.min(ccw)
    }

    /// 有效性别（批③-b 第八把 雌雄）：主将槽有武将时取武将性别，
    /// 无武将默认男性（「性别有槽时筛选、无槽默认」惯例）。
    pub fn effective_gender(&self, p: PlayerId) -> crate::vocabulary::generals::Gender {
        self.player(p).general_slots[0]
            .and_then(crate::vocabulary::generals::general_gender)
            .unwrap_or(crate::vocabulary::generals::Gender::Male)
    }

    // ---- 国战阵法派生判定（T19a，规则 v200；上游 guozhan patch
    // player.js 简化口径：构逆/野心家结盟/undist 不建模，登记差异）----

    /// 国战友方判定（上游 isFriendOf 简化）：双方明置势力已知且相同；
    /// 同一角色恒真。
    pub fn is_friend_of(
        &self,
        catalog: &crate::catalog::Catalog,
        a: PlayerId,
        b: PlayerId,
    ) -> bool {
        if a == b {
            return true;
        }
        if matches!(self.mode, GameMode::Kingdoms) {
            return self.kingdom_friends(a, b);
        }
        match (
            self.effective_faction(catalog, a),
            self.effective_faction(catalog, b),
        ) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// 存活左右邻居（座位环跳过死亡座位——登记差异：上游 getNext 含死
    /// 亡座位时语义不明，保守取存活）。
    pub fn alive_neighbors(&self, p: PlayerId) -> (Option<PlayerId>, Option<PlayerId>) {
        let n = self.players.len() as PlayerId;
        if n <= 1 {
            return (None, None);
        }
        let alive = |q: PlayerId| self.player(q).alive;
        let mut next = self.ring_next(p);
        let mut guard = 0;
        while next != p && !alive(next) && guard < n {
            next = self.ring_next(next);
            guard += 1;
        }
        let next = if next == p || !alive(next) {
            None
        } else {
            Some(next)
        };
        let mut prev = self.ring_prev(p);
        guard = 0;
        while prev != p && !alive(prev) && guard < n {
            prev = self.ring_prev(prev);
            guard += 1;
        }
        let prev = if prev == p || !alive(prev) {
            None
        } else {
            Some(prev)
        };
        (next, prev)
    }

    /// 被围攻（上游 sieged 简化）：己方明置势力已知，存活左右邻均已知
    /// 且互异、非己方、且互为友方（同势力夹击）。
    pub fn sieged(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
        if self.effective_faction(catalog, p).is_none() {
            return false;
        }
        let (Some(next), Some(prev)) = self.alive_neighbors(p) else {
            return false;
        };
        if next == prev {
            return false;
        }
        if self.effective_faction(catalog, next).is_none() {
            return false;
        }
        if self.is_friend_of(catalog, next, p) {
            return false;
        }
        self.is_friend_of(catalog, next, prev)
    }

    /// q 围攻 p（q 为 p 的存活邻且 p 处于被围攻）。
    pub fn sieges(&self, catalog: &crate::catalog::Catalog, q: PlayerId, p: PlayerId) -> bool {
        let (next, prev) = self.alive_neighbors(p);
        (next == Some(q) || prev == Some(q)) && self.sieged(catalog, p)
    }

    /// 队列判定（上游 inline）：自己回合外有 ≥1 连续同势力友邻。
    pub fn inline(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
        if self.effective_faction(catalog, p).is_none() {
            return false;
        }
        let (next, prev) = self.alive_neighbors(p);
        next.is_some_and(|q| self.is_friend_of(catalog, p, q))
            || prev.is_some_and(|q| self.is_friend_of(catalog, p, q))
    }

    /// q 是否处于 p 的连续同势力友方段内（或 q==p 且 p 处于队列）。
    pub fn inline_with(&self, catalog: &crate::catalog::Catalog, p: PlayerId, q: PlayerId) -> bool {
        if !self.inline(catalog, p) {
            return false;
        }
        if q == p {
            return true;
        }
        if !self.is_friend_of(catalog, p, q) {
            return false;
        }
        // forward/backward 沿座位环（T19b）
        let mut cur = p;
        loop {
            let nxt = self.ring_next(cur);
            if nxt == p
                || nxt == cur
                || !self.player(nxt).alive
                || !self.is_friend_of(catalog, p, nxt)
            {
                break;
            }
            if nxt == q {
                return true;
            }
            cur = nxt;
        }
        let mut cur = p;
        loop {
            let prv = self.ring_prev(cur);
            if prv == p
                || prv == cur
                || !self.player(prv).alive
                || !self.is_friend_of(catalog, p, prv)
            {
                break;
            }
            if prv == q {
                return true;
            }
            cur = prv;
        }
        false
    }

    pub fn hand_limit(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> usize {
        // 英姿·刷新面（swd 武将面批 81 reyingzi）：手牌上限基数为体力上限
        //（上游 reyingzi mod maxHandcardBase `return player.maxHp` 同口径
        //——覆盖基数；星体改造等 maxHandcard 修饰仍叠加）
        let base = if catalog
            .skill_by_name("reyingzi")
            .map(|d| self.has_skill(p, d))
            .unwrap_or(false)
        {
            self.player(p).max_hp as usize
        } else {
            self.player(p).hp.max(0) as usize
        };
        // 星体改造（U05 swd 灵剑，规则 v198）：手牌上限 -N 累加
        let base = self
            .skills
            .iter()
            .filter(|((owner, _), inst)| *owner == p && inst.enabled)
            .find_map(|((_, def), _)| {
                catalog.skills[*def as usize]
                    .behavior
                    .hand_limit_base(self, p)
            })
            .unwrap_or(base);
        let mut limit = base.saturating_sub(self.player(p).xingtigaizao_count as usize);
        // 无脏（Boss 混沌，规则 v209）：手牌上限基数 0——体力贡献清零
        // （上游 boss_wuzang mod maxHandcard `num - player.hp` 同口径：
        // 基数 hp-hp=0；其余加成修饰保留）
        if let Some(def) = catalog.skill_by_name(crate::vocabulary::names::BOSS_WUZANG) {
            if self.has_skill(p, def) {
                limit = limit.saturating_sub(self.player(p).hp.max(0) as usize);
            }
        }
        // 兽九（Boss 相柳，规则 v222）：手牌上限恒 9（上游 boss_jiushou
        // mod maxHandcard `num - hp + 9`：基数 hp-hp+9=9 同口径）
        if let Some(def) = catalog.skill_by_name(crate::vocabulary::names::BOSS_JIUSHOU) {
            if self.has_skill(p, def) {
                limit = 9;
            }
        }
        // 灵渠（Boss 玄武，规则 v237）：手牌上限 +受击计数（上游
        // boss_lingqu mod maxHandcard `num + storage` 同口径）
        limit += self.player(p).lingqu_count as usize;
        // 脉轮（swd 批 137，规则 v316）：5=上限 +1、6=上限 -1（上游
        // mailun5/6 mod maxHandcard num±1 同口径）
        match self.player(p).mailun_mode {
            5 => limit += 1,
            6 => limit = limit.saturating_sub(1),
            _ => {}
        }
        // 神威（Boss 庞统浴火授予，批 118 规则 v306）：手牌上限
        // +min(3, 其他角色数)（上游 shenwei mod maxHandcard 同口径）
        if let Some(def) = catalog.skill_by_name(crate::vocabulary::host::BOSS_SHENWEI_KEY) {
            if self.has_skill(p, def) {
                let others = self.alive_players().filter(|q| *q != p).count();
                limit += others.min(3);
            }
        }
        // 神恩（Boss 太昊/炎帝 光环，规则 v238）：挑战者侧手牌上限 +1
        //（上游 boss_shenen2 mod maxHandcard `!side → num+1` 同口径；
        // 光环激活=场上存在神恩持有者）
        {
            let shenen = catalog.skill_by_name(crate::vocabulary::names::BOSS_SHENEN);
            let active = shenen
                .map(|d| self.skills.keys().any(|(_, dd)| *dd == d))
                .unwrap_or(false);
            if active && !self.is_boss_side(p) {
                limit += 1;
            }
        }
        limit
    }

    /// 持有判定（**含未启用**实例）——隐贼激活门控用（装配即禁用，
    /// 体力条件达标时翻启用）。
    pub fn has_skill_any_state(&self, p: PlayerId, def: SkillDefId) -> bool {
        self.skills.contains_key(&(p, def))
    }

    pub fn has_skill(&self, p: PlayerId, def: SkillDefId) -> bool {
        self.skills
            .get(&(p, def))
            .map(|i| i.enabled)
            .unwrap_or(false)
    }

    /// R05-J（§6）：显式移除技能或换将清理时，同步清理对应 R05 helper 的
    /// `vm_aux.marks[(owner, def as u32)]`；若移除的是 `Limit` 槽，同逻辑
    /// 技能在同一座位上的 `Current` 槽（实例与 mark）一并移除。
    pub fn remove_skill_and_r05_mark(
        &mut self,
        catalog: &crate::catalog::Catalog,
        owner: PlayerId,
        def: SkillDefId,
    ) -> Option<SkillInstance> {
        let old = self.skills.remove(&(owner, def));
        if catalog.is_r05_helper_def(def) {
            self.vm_aux.marks.remove(&(owner, def as u32));
            let name = catalog.skill_name(def);
            if let Some(prefix) = name.strip_suffix("#r05:limit") {
                if let Some(cur_def) = catalog.skill_by_name(&format!("{prefix}#r05:current")) {
                    self.skills.remove(&(owner, cur_def));
                    self.vm_aux.marks.remove(&(owner, cur_def as u32));
                }
            }
        }
        old
    }

    /// R05-J（§6）：换将（uninit/init）时清除目标座位上的全部 R05 四角色
    /// helper 实例与对应 mark（含 Limit、Combs、Next、Current）。
    pub fn clear_r05_helpers_on_seat(
        &mut self,
        catalog: &crate::catalog::Catalog,
        seat: PlayerId,
    ) {
        let helpers: Vec<SkillDefId> = self
            .skills
            .keys()
            .filter(|(o, d)| *o == seat && catalog.is_r05_helper_def(*d))
            .map(|(_, d)| *d)
            .collect();
        for def in helpers {
            self.remove_skill_and_r05_mark(catalog, seat, def);
        }
    }

    /// 当前玩家的下一位存活玩家（座位环顺时针）。
    pub fn next_alive(&self, from: PlayerId) -> PlayerId {
        let n = self.players.len() as PlayerId;
        let mut i = self.ring_next(from);
        let mut guard = 0;
        while guard < n {
            if self.player(i).alive {
                return i;
            }
            i = self.ring_next(i);
            guard += 1;
        }
        i
    }

    /// 从 `start` 起按座位顺序（含 start，若存活）的存活玩家队列。
    /// 起点可为已死亡座位（问询链构造时 turn.current 可能已死）：
    /// 固定扫一圈座位数，避免「cur 只会落在存活座位、永不再等于
    /// 死亡起点」的死循环。
    pub fn seat_order_from(&self, start: PlayerId) -> Vec<PlayerId> {
        let n = self.players.len() as PlayerId;
        let mut out = Vec::new();
        let mut cur = self.ring_pos(start) as PlayerId;
        // 从 ring 位置起遍历（起点按环位置而非恒等位）
        let pos_start = self.ring_pos(start);
        let _ = cur;
        cur = *self.seat_ring.get(pos_start).unwrap_or(&start);
        for _ in 0..n {
            if self.player(cur).alive {
                out.push(cur);
            }
            cur = self.ring_next(cur);
        }
        out
    }
}
