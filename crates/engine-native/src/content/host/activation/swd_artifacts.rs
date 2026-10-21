use super::super::*;

impl HostSkillBehavior {
    pub(super) fn activate_swd_artifacts(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            // 鼠槌（equip5 窗口机批）：手中有杀且本回合未用过杀（上游
            // filter cardUsable 同口径——限击数提升时收窄登记差异）
            HostBehavior::SwdShuchuiUse => {
                let has_sha = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c).is_sha_like());
                if !has_sha || ctx.state.turn.sha_used_play_phase {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::SwdLangeguaiyiEquipSkill => {
                // 蓝格怪衣（swd 第十五批 v99）：判定技无前置条件（上游
                // 无 filter；onLose 清 getStat 为上游计数簿记，原生
                // once_per_turn 等价）
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::SwdYufuluEquipSkill => {
                // 御夫录（swd 第十二批 v96）：需有武器牌（手牌/装备区，
                // 上游 filter he+equip1 同口径）且存在其他存活角色
                let is_weapon = |c: CardId| {
                    ctx.catalog.card_slot_of(ctx.state, c)
                        == Some(crate::catalog::EquipSlot::Weapon)
                };
                let has_weapon = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .copied()
                    .any(is_weapon);
                let others = ctx.state.alive_players().any(|p| p != owner);
                if !has_weapon || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::SwdShentouEquipSkill => {
                // 潜偷（swd 第三批 v92）：需有可弃手牌，且存在手牌更多的
                // 其他角色（上游 filter 同口径）
                let nh = ctx.state.player(owner).hand.len();
                let has = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && ctx.state.player(p).hand.len() > nh);
                if nh == 0 || !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 收纳：有手牌且存在其他有手牌角色（装备炼妖壶在 ActivateSkill
            // 命令层由 equip 技能实例承载——实例随 grant_equip_skills 授予）
            HostBehavior::SwdLianyaohuShouna => {
                let has = !ctx.state.player(owner).hand.is_empty();
                let others = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).hand.is_empty());
                if !has || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 炼化：壶仓 ≥2
            HostBehavior::SwdLianyaohuLianhua => {
                if ctx.state.player(owner).lianyaohu_store.len() < 2 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 昆仑镜：牌堆 ≥3 且有手牌
            HostBehavior::SwdKunlunjingcCard => {
                if ctx.state.deck.len() < 3 || ctx.state.player(owner).hand.is_empty() {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 四非珍面：存在其他有手牌角色
            HostBehavior::SwdSifeizhenmianCard => {
                let others = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).hand.is_empty());
                if !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::TargetChoice,
                }))
            }
            // 逆攻发动：标记 ≥1；lv≥2 打伤需选目标，lv==1 仅摸 1（直发）
            HostBehavior::SwdNigongStrike => {
                let lv = ctx.state.player(owner).nigong_lv;
                if lv == 0 {
                    return None;
                }
                if lv == 1 {
                    Some(Box::new(HostMachine {
                        def_key: self.name.clone(),
                        behavior: self.behavior,
                        owner,
                        target: None,
                        card: None,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }))
                } else {
                    Some(Box::new(HostMachine {
                        def_key: self.name.clone(),
                        behavior: self.behavior,
                        owner,
                        target: None,
                        card: None,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::TargetChoice,
                    }))
                }
            }
            // 神农鼎：受伤且手牌 ≥2
            HostBehavior::SwdShennongdingCard => {
                let p = ctx.state.player(owner);
                if p.hp >= p.max_hp as i32 || p.hand.len() < 2 {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            _ => unreachable!("behavior routed to the wrong activate module"),
        }
    }
}
