//! GameStateCore 的查询与变更方法（座位、区域、体力、技能实例等）。

use super::*;

impl GameStateCore {
    /// Pinned isEnemyOf/isFriendOf semantics for mode facts represented by this kernel.
    pub fn source_is_enemy_of(
        &self,
        catalog: &crate::catalog::Catalog,
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
            GameMode::Kingdoms => Some(!self.is_friend_of(catalog, a, b)),
            _ => None,
        }
    }

    fn jun_group(name: &str) -> Option<&'static str> {
        match name {
            "jun_liubei" | "gz_jun_liubei" | "pe_jun_liubei" | "jun_shu" | "君刘备" => Some("shu"),
            "jun_zhangjiao" | "gz_jun_zhangjiao" | "pe_jun_zhangjiao" | "jun_qun" | "君张角" => Some("qun"),
            "jun_sunquan" | "gz_jun_sunquan" | "pe_jun_sunquan" | "jun_wu" | "君孙权" => Some("wu"),
            "jun_caocao" | "gz_jun_caocao" | "pe_jun_caocao" | "jun_wei" | "君曹操" => Some("wei"),
            _ => None,
        }
    }

    fn faction_name(faction: crate::catalog::Faction) -> &'static str {
        match faction {
            crate::catalog::Faction::Wei => "wei",
            crate::catalog::Faction::Shu => "shu",
            crate::catalog::Faction::Wu => "wu",
            crate::catalog::Faction::Qun => "qun",
        }
    }

    pub fn kingdom_friends(&self, a: PlayerId, b: PlayerId) -> bool {
        if a == b
            || self.player(a).yexinjia_friends.contains(&b)
            || self.player(b).yexinjia_friends.contains(&a)
        {
            return true;
        }
        let resolve = |p: PlayerId| match &self.player(p).kingdom_identity {
            KingdomIdentity::Declared(n) => Some(Self::jun_group(n).unwrap_or(n)),
            KingdomIdentity::Ye => {
                let pl = self.player(p);
                let id = pl.revealed_generals[0].then_some(pl.general_slots[0]).flatten().or_else(|| pl.revealed_generals[1].then_some(pl.general_slots[1]).flatten()).or(pl.general_slots[0]).or(pl.general_slots[1])?;
                let g = Self::faction_name(crate::vocabulary::generals::STANDARD_GENERALS.get(id as usize)?.faction);
                self.players.iter().any(|q| q.alive && matches!(&q.kingdom_identity, KingdomIdentity::Declared(n) if Self::jun_group(n) == Some(g) || (n == g && q.revealed_generals[0] && q.general_slots[0].is_some_and(|j| j as usize >= crate::vocabulary::generals::STANDARD_GENERALS.len())))).then_some(g)
            }
            KingdomIdentity::Unknown => None,
        };
        matches!((resolve(a), resolve(b)), (Some(x), Some(y)) if x == y)
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
    fn is_revealed_jun(&self, catalog: &crate::catalog::Catalog, q: PlayerId) -> bool {
        let pl = self.player(q);
        pl.alive && pl.revealed_generals[0] && pl.general_slots[0].and_then(|g| catalog.general_meta(g)).is_some_and(|(n, ..)| Self::jun_group(n).is_some() || n.starts_with("jun_") || n.starts_with("gz_jun_"))
    }
    pub fn declare_revealed_kingdom(
        &mut self,
        catalog: &crate::catalog::Catalog,
        player: PlayerId,
    ) {
        let is_jun = self.is_revealed_jun(catalog, player);
        if !matches!(self.mode, GameMode::Kingdoms)
            || (self.player(player).kingdom_identity != KingdomIdentity::Unknown && !is_jun)
        {
            return;
        }
        let Some(faction) = self.declared_faction(catalog, player) else {
            return;
        };
        let name = Self::faction_name(faction);
        let has_jun = (0..self.players.len() as PlayerId)
            .any(|q| self.is_revealed_jun(catalog, q) && self.declared_faction(catalog, q) == Some(faction));
        let population = self
            .players
            .iter()
            .filter(|p| matches!(&p.kingdom_identity, KingdomIdentity::Declared(n) if Self::jun_group(n).unwrap_or(n) == name))
            .count();
        let identity = if has_jun || (population + 1) * 2 <= self.players.len() {
            KingdomIdentity::Declared(name.into())
        } else {
            KingdomIdentity::Ye
        };
        self.set_kingdom_identity(player, identity)
            .expect("native reveal identity context");
        if is_jun {
            for q in 0..self.players.len() as PlayerId {
                if self.player(q).kingdom_identity == KingdomIdentity::Ye
                    && self.declared_faction(catalog, q) == Some(faction)
                {
                    self.set_kingdom_identity(q, KingdomIdentity::Declared(name.into()))
                        .expect("native jun alliance upgrade");
                }
            }
        }
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
            if self.kingdom_friends(a, b) {
                return true;
            }
            let group_of = |p: PlayerId| match &self.player(p).kingdom_identity {
                KingdomIdentity::Declared(n) => Some(Self::jun_group(n).unwrap_or(n)),
                KingdomIdentity::Ye => {
                    let f = self.declared_faction(catalog, p)?;
                    (0..self.players.len() as PlayerId).any(|q| self.is_revealed_jun(catalog, q) && self.declared_faction(catalog, q) == Some(f)).then(|| Self::faction_name(f))
                }
                KingdomIdentity::Unknown => self.is_revealed_jun(catalog, p).then(|| Self::faction_name(self.declared_faction(catalog, p).unwrap())),
            };
            return matches!((group_of(a), group_of(b)), (Some(x), Some(y)) if x == y);
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

    /// 计算距离（含 ±1 马修饰，下限 1；座位距离只计存活玩家）。
    /// 上游/旧线一致性待核对，见覆盖清单行为差异表。
    pub fn computed_distance(
        &self,
        catalog: &crate::catalog::Catalog,
        a: PlayerId,
        b: PlayerId,
    ) -> u32 {
        use crate::catalog::CardKind;
        if a == self.turn.current && self.turn.liangou_hook == Some(b) {
            return 1;
        }
        if a == self.turn.current && self.turn.feitiao_target == Some(b) {
            return 1;
        }
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
        self.finish_computed_distance(catalog, a, b, base, plus, minus)
    }
}
