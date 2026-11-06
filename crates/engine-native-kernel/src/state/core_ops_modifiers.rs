//! `GameStateCore` 距离、攻击范围、性别与手牌上限计算（含 PA12 Batch 14 `VmModifierRule` 钩子）。

use super::*;

impl GameStateCore {
    pub(super) fn finish_computed_distance(
        &self,
        catalog: &crate::catalog::Catalog,
        a: PlayerId,
        b: PlayerId,
        base: u32,
        plus: u32,
        minus: u32,
    ) -> u32 {
        use crate::catalog::CardKind;
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
        crate::rvm_bridge::modifiers::eval_distance_total(self, catalog, a, b, total)
    }

    /// 攻击范围：武器射程（泛型 Weapon 走 CardDef.range，命名武器走
    /// 牌种固定射程），无武器为 1；叠算 `dcdichou_eff`（UP37 R01）标记加成及 `VmModifierRule`。
    pub fn attack_range(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> u32 {
        use crate::catalog::CardKind;
        let base = self
            .player(p)
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
            .unwrap_or(1);
        let dcdichou_eff_has_vm_mod = catalog
            .skill_by_name("dcdichou_eff")
            .is_some_and(|def| {
                self.has_skill(p, def)
                    && !catalog.skills[def as usize].behavior.vm_modifiers().is_empty()
            });
        let dcdichou_eff_bonus = if !dcdichou_eff_has_vm_mod
            && (catalog
                .skill_by_name("dcdichou_eff")
                .is_some_and(|def| self.has_skill(p, def))
                || self
                    .vm_aux
                    .flags
                    .get(&(p, "skill:2754298115".to_string()))
                    .copied()
                    .unwrap_or(0)
                    != 0)
        {
            self.vm_aux
                .marks
                .get(&(p, 2_754_298_115))
                .copied()
                .unwrap_or(0)
                .max(0) as u32
        } else {
            0
        };
        crate::rvm_bridge::modifiers::eval_attack_range(
            self,
            catalog,
            p,
            base + dcdichou_eff_bonus,
        )
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
                let b = &catalog.skills[*def as usize].behavior;
                b.hand_limit_base(self, p).or_else(|| {
                    b.vm_modifiers()
                        .iter()
                        .any(|r| {
                            r.target == crate::rvm_bridge::VmModifierTarget::HandLimitBaseMaxHp
                                && r.selector.map_or(true, |s| s.matches_owner(self, p))
                                && r.effective_value(self, catalog, p, *def) > 0
                        })
                        .then_some(self.player(p).max_hp as usize)
                })
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
        crate::rvm_bridge::modifiers::eval_hand_limit(self, catalog, p, limit as i32)
    }
}
