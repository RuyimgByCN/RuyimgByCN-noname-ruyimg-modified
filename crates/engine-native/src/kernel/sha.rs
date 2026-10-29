use super::*;

mod liuli;
mod resume;
mod step;

#[derive(Clone)]
pub struct ShaResolveMachine {
    pub(super) card: crate::ids::CardId,
    pub(super) source: PlayerId,
    pub(super) target: PlayerId,
    pub(super) nature: crate::event::DamageNature,
    pub(super) asked: bool,
    /// 八卦阵判定是否已做（每次杀结算至多一次）。
    pub(super) armor_judged: bool,
    /// 已使用的闪数（无双：需要连续两张）。
    pub(super) dodges_used: u8,
    /// 铁骑判定结果：None=未判定/无铁骑；Some(true)=红色（不能出闪）。
    pub(super) tieji_red: Option<bool>,
    /// 流离（大乔）状态（入快照，ADR 0774：消除恢复分歧）。
    pub(super) liuli_done: bool,
    /// 流离两段询问（逐技对照批二 v76）：1=弃牌选择挂起，2=转移目标挂起。
    pub(super) liuli_stage: u8,
    /// 流离已选定的待弃牌（stage 2 期间暂存）。
    pub(super) liuli_card: Option<crate::ids::CardId>,
    /// 护驾（曹操）逐人询问队列（入快照）：`None` = 本响应窗口尚未
    /// 开始逐人询问；`Some(q)` = 剩余待问的魏势力角色（目标下家起
    /// 按座位序）；`Some(空)` = 已全部问完（拒绝/无人合格）。
    /// 不以暗手牌预筛。
    pub(super) hujia_queue: Option<Vec<PlayerId>>,
    /// 护驾当前被询问的持牌者（Some = 等待其 Confirm/Pass）。
    pub(super) hujia_holder: Option<PlayerId>,
    /// 酒效果（派发时从回合状态捕获）：本次杀伤害 +1。
    pub(super) jiu_bonus: bool,
    /// 麒麟弓（批③-b 第二把）：弃马询问挂起中。
    pub(super) asking_qilin: bool,
    pub(super) qilin_horse: Option<crate::ids::CardId>,
    /// 寒冰剑（批③-b 第三把）：防止伤害确认挂起中。
    pub(super) asking_hanbing: bool,
    /// 风雪刃（U05 swd 第十一批，规则 v95）：命中后翻面+摸牌询问挂起中。
    pub(super) asking_fengxueren: bool,
    /// 贯石斧（批③-b 第五把）：杀被闪后弃二强伤询问挂起中
    ///（Some = 待弃的牌；None = 询问未开）。
    pub(super) asking_guanshi: bool,
    pub(super) guanshi_cards: Vec<crate::ids::CardId>,
    /// 青龙偃月刀（批③-b 第六把）：被闪后「再杀」询问挂起中
    ///（Some = 选中的新杀牌，来自来源手牌）。
    pub(super) asking_qinglong: bool,
    /// 雌雄双股剑（批③-b 第八把）：弃牌/令摸询问挂起中。
    pub(super) asking_cixiong: bool,
    /// 雌雄双股剑：本杀已问过标志。
    pub(super) cixiong_done: bool,
    /// 寒冰剑（逐技对照批一，规则 v75）：目标自选弃牌剩余次数
    ///（来源确认防止后为 2，逐次扣减；上游逐张强制弃）。
    pub(super) hanbing_pending: u8,
    /// 方天画戟（批③-b 第九把）：当前目标之后的剩余目标队列。
    pub(super) extra_targets: Vec<PlayerId>,
    /// 霰弹（U05 ow 死神，规则 v140）：弃牌询问挂起中。
    pub(super) asking_xiandan: bool,
    /// 霰弹：本杀已问过标志（每杀至多一次）。
    pub(super) xiandan_done: bool,
    /// 霰弹：黑牌分支——本杀伤害 +1（上游 xiandan2 damageBegin num++
    /// 同口径，一次性随本杀）。
    pub(super) xiandan_bonus: bool,
    /// 掠影（U05 xianjian 慕长兰，规则 v154）：目标选人询问挂起中。
    pub(super) asking_lueying: bool,
    /// 掠影：目标已选定的受害者（等来源随机弃其一张）。
    pub(super) lueying_victim: Option<PlayerId>,
    /// 掠影：本杀已询问且被放弃（v194——放弃不消耗 usable，但同一杀
    /// 不得重复询问；阻止 step 重入钩子再问）。
    pub(super) lueying_declined: bool,
    /// 复仇（玩点论杀批 14，规则 v279）：目标弃牌取消询问挂起中。
    pub(super) asking_fuchou: bool,
    /// 复仇：目标所选取消牌（取消结算：杀回来源手、牌归来源、标记）。
    pub(super) fuchou_card: Option<crate::ids::CardId>,
    /// 萨登荆环（swd 卡牌面窗口机批）：被闪后判定确认询问挂起中。
    pub(super) asking_sadeng: bool,
    /// 萨登荆环：目标额外出闪询问挂起中。
    pub(super) asking_sadeng_shan: bool,
    /// 狩猎（swd deferred 消解批 子达书）：弃一张手牌令此杀不可闪避
    ///（确认后置位；整杀有效=机器实例生命周期）。
    pub(super) shoulie_direct: bool,
}

impl ShaResolveMachine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        card: crate::ids::CardId,
        source: PlayerId,
        target: PlayerId,
        nature: crate::event::DamageNature,
        asked: bool,
        armor_judged: bool,
        dodges_used: u8,
        tieji_red: Option<bool>,
        liuli_done: bool,
        liuli_stage: u8,
        liuli_card: Option<crate::ids::CardId>,
        hujia_queue: Option<Vec<PlayerId>>,
        hujia_holder: Option<PlayerId>,
        jiu_bonus: bool,
        asking_qilin: bool,
        qilin_horse: Option<crate::ids::CardId>,
        asking_hanbing: bool,
        asking_fengxueren: bool,
        asking_guanshi: bool,
        guanshi_cards: Vec<crate::ids::CardId>,
        asking_qinglong: bool,
        asking_cixiong: bool,
        cixiong_done: bool,
        extra_targets: Vec<PlayerId>,
        hanbing_pending: u8,
        asking_xiandan: bool,
        xiandan_done: bool,
        xiandan_bonus: bool,
        asking_lueying: bool,
        lueying_victim: Option<PlayerId>,
        lueying_declined: bool,
        asking_fuchou: bool,
        fuchou_card: Option<crate::ids::CardId>,
        asking_sadeng: bool,
        asking_sadeng_shan: bool,
        shoulie_direct: bool,
    ) -> Self {
        Self {
            card,
            source,
            target,
            nature,
            asked,
            armor_judged,
            dodges_used,
            tieji_red,
            liuli_done,
            liuli_stage,
            liuli_card,
            hujia_queue,
            hujia_holder,
            jiu_bonus,
            asking_qilin,
            qilin_horse,
            asking_hanbing,
            asking_fengxueren,
            asking_guanshi,
            guanshi_cards,
            asking_qinglong,
            asking_cixiong,
            cixiong_done,
            extra_targets,
            hanbing_pending,
            asking_xiandan,
            xiandan_done,
            xiandan_bonus,
            asking_lueying,
            lueying_victim,
            lueying_declined,
            asking_fuchou,
            fuchou_card,
            asking_sadeng,
            asking_sadeng_shan,
            shoulie_direct,
        }
    }

    /// 流离转移候选：存活、非自己、非此杀使用者、在（被转移者）攻击
    /// 范围内且为该杀合法目标（空城不可）——上游 filter/filterTarget 同口径。
    fn liuli_candidates(&self, ctx: &Ctx<'_>) -> Vec<ChoiceOption> {
        ctx.core
            .alive_players()
            .filter(|p| self.liuli_candidate_ok(ctx, *p))
            .map(ChoiceOption::Player)
            .collect()
    }

    fn liuli_candidate_ok(&self, ctx: &Ctx<'_>, p: PlayerId) -> bool {
        if p == self.target || p == self.source || !ctx.core.player(p).alive {
            return false;
        }
        let kongcheng = ctx
            .catalog
            .skill_by_name(crate::content::standard::KONG_CHENG)
            .map(|d| ctx.core.has_skill(p, d))
            .unwrap_or(false);
        if kongcheng && ctx.core.player(p).hand.is_empty() {
            return false;
        }
        ctx.core.computed_distance(ctx.catalog, self.target, p)
            <= ctx.core.attack_range(ctx.catalog, self.target)
    }

    /// 方天画戟（批③-b 第九把）：当前目标结算完毕 → 队列取下一目标
    ///（逐目标状态复位，重开该目标的防具/出闪询问）；队列空 → 弃置杀
    /// 本体结束。
    fn advance_target(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if let Some(next) = self.extra_targets.first().copied() {
            self.extra_targets.remove(0);
            self.target = next;
            self.asked = false;
            self.armor_judged = false;
            self.dodges_used = 0;
            self.tieji_red = None;
            self.liuli_done = false;
            self.liuli_stage = 0;
            self.liuli_card = None;
            self.hujia_queue = None;
            self.hujia_holder = None;
            self.asking_hanbing = false;
            self.asking_qilin = false;
            self.asking_guanshi = false;
            self.asking_fengxueren = false;
            self.hanbing_pending = 0;
            return Flow::Yield;
        }
        let card = self.card;
        ctx.to_discard(card);
        ctx.core.table.retain(|c| *c != card);
        Flow::Done
    }

    /// 结算杀伤害（裸衣/酒/凶兽加成）并推进目标/弃置杀本体。
    /// 萨登荆环命中改写后的伤害续入（与正常命中路径同口径：寒冰/麒麟
    /// 窗先开，无窗则直接结算伤害）。
    fn sadeng_continue_to_damage(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.hanbing_can_fire(ctx) {
            self.asking_hanbing = true;
            return Flow::WaitChoice {
                observer: self.source,
                spec: ChoiceSpec {
                    kind: ChoiceKind::Confirm,
                    prompt: "hanbing_prevent_confirm",
                    options: vec![ChoiceOption::Confirm],
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        let horses = self.qilin_horses(ctx);
        if !horses.is_empty() {
            self.asking_qilin = true;
            return Flow::WaitChoice {
                observer: self.source,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "qilin_discard_horse",
                    options: horses,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            };
        }
        self.emit_sha_damage(ctx)
    }

    fn emit_sha_damage(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 燎原加伤（swd 批 145，规则 v319）：上游 liaoyuan2 damageBegin
        // `trigger.num += storage.liaoyuan` 同口径；计入后清零
        let liaoyuan = u32::from(ctx.core.turn.liaoyuan_count);
        ctx.core.turn.liaoyuan_count = 0;
        if ctx.core.player(self.target).alive {
            ctx.emit(EventData::Damage(damage_data_with(
                Some(self.source),
                self.target,
                1 + luoyi_bonus(ctx, Some(self.source))
                    + xiongshou_bonus(ctx, self.source, self.target)
                    + self.jiu_damage_bonus()
                    + u32::from(self.xiandan_bonus)
                    + liaoyuan,
                self.nature,
                Some(self.card),
            )));
        }
        self.xiandan_bonus = false;
        self.advance_target(ctx)
    }

    /// 来源是否持有无双（需要连续两张闪）。
    fn source_has_wushuang(&self, ctx: &Ctx<'_>) -> bool {
        ctx.catalog
            .skill_by_name(crate::content::standard::WU_SHUANG)
            .map(|d| ctx.core.has_skill(self.source, d))
            .unwrap_or(false)
    }

    /// 酒伤害加成（派发时已捕获）。
    fn jiu_damage_bonus(&self) -> u32 {
        if self.jiu_bonus {
            1
        } else {
            0
        }
    }

    /// 护驾是否可发动：目标持技 + 主公资格（身份局须主公）。
    fn hujia_active(&self, ctx: &Ctx<'_>) -> bool {
        let hujia = ctx
            .catalog
            .skill_by_name(crate::content::standard::HU_JIA)
            .map(|d| ctx.core.has_skill(self.target, d))
            .unwrap_or(false);
        hujia && ctx.core.is_lord(self.target)
    }

    /// 护驾询问队列：目标下家起按座位序的魏势力存活角色
    /// （不以暗手牌预筛——询问资格公开，应答私密）。
    fn build_hujia_queue(&self, ctx: &Ctx<'_>) -> Vec<PlayerId> {
        ctx.core
            .seat_order_from(ctx.core.next_alive(self.target))
            .into_iter()
            .filter(|p| {
                *p != self.target
                    && ctx.core.player(*p).alive
                    && ctx.core.effective_faction(ctx.catalog, *p)
                        == Some(crate::catalog::Faction::Wei)
            })
            .collect()
    }
}

impl ShaResolveMachine {
    /// 堡垒哨卫（U05 ow，规则 v125）：来源持 bshaowei 且哨卫模式——
    /// 杀无视距离/防具、无次数限制、不可闪避。
    fn baolei_sentinel(&self, ctx: &Ctx<'_>) -> bool {
        ctx.catalog
            .skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY)
            .map(|def| {
                ctx.core.has_skill(self.source, def)
                    && ctx.core.player(self.source).baolei_mode == 1
            })
            .unwrap_or(false)
    }

    /// 寒冰剑可发动（来源装备寒冰剑且目标有手牌/装备）。
    fn hanbing_can_fire(&self, ctx: &Ctx<'_>) -> bool {
        let has_hanbing = ctx
            .core
            .player(self.source)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Hanbing);
        has_hanbing
            && ctx.core.player(self.target).alive
            && (!ctx.core.player(self.target).hand.is_empty()
                || !ctx.core.player(self.target).equips.is_empty())
    }

    /// 目标「手牌+装备」是否仍有可弃牌（上游 hasDiscardableCards("he")）。
    fn target_has_he(&self, ctx: &Ctx<'_>) -> bool {
        !ctx.core.player(self.target).hand.is_empty()
            || !ctx.core.player(self.target).equips.is_empty()
    }

    /// 寒冰剑目标自选弃牌询问：手牌+装备逐张强选（min=1 不可取消）。
    fn open_hanbing_target_ask(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let mut options: Vec<ChoiceOption> = ctx
            .core
            .player(self.target)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        options.extend(
            ctx.core
                .player(self.target)
                .equips
                .iter()
                .copied()
                .map(ChoiceOption::Card),
        );
        Flow::WaitChoice {
            observer: self.target,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "hanbing_target_discard",
                options,
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }

    /// 麒麟弓可弃目标坐骑列表（来源装备麒麟弓且目标有马）。
    fn qilin_horses(&self, ctx: &Ctx<'_>) -> Vec<ChoiceOption> {
        let has_qilin = ctx
            .core
            .player(self.source)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qilin);
        if !has_qilin {
            return Vec::new();
        }
        ctx.core
            .player(self.target)
            .equips
            .iter()
            .filter(|c| {
                matches!(
                    ctx.catalog.card_kind(**c),
                    CardKind::HorsePlus | CardKind::HorseMinus
                )
            })
            .copied()
            .map(ChoiceOption::Card)
            .collect()
    }
}
