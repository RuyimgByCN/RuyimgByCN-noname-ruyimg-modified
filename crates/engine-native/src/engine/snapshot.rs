use super::*;

impl Engine {
    pub fn save(&self) -> StateSnapshot {
        StateSnapshot {
            ruleset_version: self.state.core.ruleset_version,
            auth_generation: self.auth_generation,
            core: self.state.core.clone(),
            resolution: crate::snap::ResolutionSnapshot {
                next_event_id: self.state.resolution.next_event_id,
                next_choice_id: self.state.resolution.next_choice_id,
                stack: self.state.resolution.stack.clone(),
                parks: self
                    .state
                    .resolution
                    .parks
                    .iter()
                    .map(|p| crate::snap::ParkSnapshot {
                        resume_depth: p.resume_depth,
                        slot: p.slot,
                        machine: p.machine.snapshot(),
                    })
                    .collect(),
                open_choice: self.state.resolution.open_choice.clone(),
                answer: self.state.resolution.answer.clone(),
                budget: self.state.resolution.budget,
            },
            last_dedup: self.last_dedup,
            segment_open: self.segment_open,
            segment_checkpoint: self
                .segment_checkpoint
                .as_ref()
                .map(|(snap, _)| Box::new(snap.clone())),
        }
    }

    /// 恢复对局：版本不符或快照含越界 ID 时拒绝，不产出引擎。
    pub fn restore(snapshot: &StateSnapshot) -> Result<Self, SnapshotError> {
        let catalog: SharedCatalog = std::sync::Arc::new(content::standard::standard_catalog());
        Self::restore_with_catalog(snapshot, catalog)
    }

    /// 带扩展目录的恢复（E03 内容锁接线）：按启用扩展集合重建目录，
    /// 版本不一致经既有 RuleVersionMismatch 机制拒绝——存档只能用
    /// 同一扩展集合（名称+版本指纹）恢复。
    pub fn restore_with_extensions(
        snapshot: &StateSnapshot,
        packs: &[&crate::content::extension::ExtensionPack],
    ) -> Result<Self, SnapshotError> {
        let catalog = content::standard::standard_catalog_with_extensions(packs)
            .map_err(|_| SnapshotError::UnknownMachine)?;
        let catalog: SharedCatalog = std::sync::Arc::new(catalog);
        Self::restore_with_catalog(snapshot, catalog)
    }

    pub(super) fn restore_with_catalog(
        snapshot: &StateSnapshot,
        catalog: SharedCatalog,
    ) -> Result<Self, SnapshotError> {
        if snapshot.ruleset_version != catalog.ruleset_version {
            return Err(SnapshotError::RuleVersionMismatch {
                snapshot: snapshot.ruleset_version,
                catalog: catalog.ruleset_version,
            });
        }
        let bounds = bounds_from(&catalog, snapshot.core.players.len());
        validate_core(&snapshot.core, &bounds)?;
        // N04-f：牌归属多重集——每张牌恰在一个牌区，机器持牌登记在界
        crate::snap::validate_card_ownership(&snapshot.core, &snapshot.resolution, &bounds)?;
        // 栈帧父子/帧号水位、挂起深度与槽位帧引用、选择/应答 ID 与机器树全部校验。
        validate_resolution(&snapshot.resolution, &bounds, &snapshot.core)?;
        // 段开放必须携带检查点（N04）：无检查点的开放段快照恢复后
        // 无法中止，属非法状态而非「恢复后再报错」。
        if snapshot.segment_open && snapshot.segment_checkpoint.is_none() {
            return Err(SnapshotError::IdOutOfBounds {
                what: "checkpoint_missing_for_open_segment",
                got: 0,
            });
        }
        // 嵌套段检查点同样校验（N07-b：恢复件必须保留可用的中止能力）。
        if let Some(ckpt) = &snapshot.segment_checkpoint {
            if ckpt.ruleset_version != catalog.ruleset_version {
                return Err(SnapshotError::RuleVersionMismatch {
                    snapshot: ckpt.ruleset_version,
                    catalog: catalog.ruleset_version,
                });
            }
            // 层级一致性（N04）：嵌套检查点自身不得再嵌套（禁止无限层级）。
            if ckpt.segment_checkpoint.is_some() {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_nested",
                    got: 0,
                });
            }
            // 检查点与当前段关联（N04）：检查点是段启动时的静止快照——
            // 座位数/模式必须与当前一致；revision 与去重高水位单调
            //（检查点不得超前当前）；自身不得处于段开放状态。
            if ckpt.core.players.len() != snapshot.core.players.len() {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_players_mismatch",
                    got: ckpt.core.players.len() as u64,
                });
            }
            if ckpt.core.mode != snapshot.core.mode {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_mode_mismatch",
                    got: 0,
                });
            }
            if ckpt.core.revision > snapshot.core.revision {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_revision_ahead",
                    got: ckpt.core.revision,
                });
            }
            if ckpt.last_dedup > snapshot.last_dedup {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_dedup_ahead",
                    got: ckpt.last_dedup,
                });
            }
            if ckpt.segment_open {
                return Err(SnapshotError::IdOutOfBounds {
                    what: "checkpoint_segment_open",
                    got: 1,
                });
            }
            validate_core(&ckpt.core, &bounds)?;
            validate_resolution(&ckpt.resolution, &bounds, &ckpt.core)?;
        }
        // 选择承接关系（N04-e）：开放选择必须恰好与某个续行机器的
        // 待答询问一致（观察者/提示/种类/选项含顺序/数量/放弃许可）；
        // 无开放选择时任何挂起机器都不得处于待询问状态。
        //
        // 命中的那台机器就是「提出询问者」：`run` 只把已校验应答交给它，
        // 同栈深多机时不会把应答转交给没有提问的机器。该身份是运行时
        // 标记（`Park::awaiting_answer`），不入快照/哈希——读档由本段
        // 唯一匹配重建（同深度多机因此不会在恢复后错配应答）。
        let asks: Vec<(usize, crate::snap::ExpectedAsk)> = snapshot
            .resolution
            .parks
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                crate::snap::active_ask(&p.machine, &snapshot.core, &catalog).map(|a| (i, a))
            })
            .collect();
        let mut asker: Option<usize> = None;
        let ask_refs: Vec<&crate::snap::ExpectedAsk> = asks.iter().map(|(_, a)| a).collect();
        match (&snapshot.resolution.open_choice, ask_refs.as_slice()) {
            (Some(open), [exp]) => {
                if open.observer != exp.observer
                    || open.spec.prompt != exp.prompt
                    || open.spec.kind != exp.kind
                    || open.spec.options != exp.options
                    || open.spec.min != exp.min
                    || open.spec.max != exp.max
                    || open.spec.allow_cancel != exp.allow_cancel
                {
                    return Err(SnapshotError::OpenChoiceMismatch {
                        observer: open.observer,
                    });
                }
                asker = Some(asks[0].0);
            }
            (Some(open), _) => {
                return Err(SnapshotError::OpenChoiceMismatch {
                    observer: open.observer,
                });
            }
            (None, []) => {}
            (None, asks) => {
                // 无开放选择却存在待答询问：承接关系不一致（N04-e）。
                let _ = asks;
                return Err(SnapshotError::OpenChoiceMismatch { observer: 0 });
            }
        }
        // G1 规则宿主绑定：扩展机器的定义键必须能在当前目录解析为
        // 宿主行为技能且行为标签一致——扩展集合不含该包（或版本不符）
        // 时恢复拒绝（UnknownMachine）。
        for park in &snapshot.resolution.parks {
            if let crate::snap::MachineSnapshot::Host {
                def_key, behavior, ..
            } = &park.machine
            {
                // 技能型宿主机器：定义键须解析为目录技能且行为一致；
                // 卡牌型（手里剑）：目录含该牌种 ⇔ 声明它的包在场。
                let skill_ok = catalog
                    .skill_by_name(def_key)
                    .and_then(|def| catalog.skills[def as usize].behavior.as_host_behavior())
                    .map(|hb| hb.tag() == *behavior)
                    .unwrap_or(false);
                let card_ok = match def_key.as_str() {
                    crate::content::host::HOST_SHOULIJIAN_KEY => {
                        *behavior
                            == crate::content::host::HostBehavior::TargetDiscardEquipOrDamage.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::ShouLiJian)
                    }
                    crate::content::host::WD_WANGMEIZHIKE_KEY => {
                        *behavior == crate::content::host::HostBehavior::WangmeiZhikeCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::WangmeiZhike)
                    }
                    crate::content::host::WD_SHUIGONG_KEY => {
                        *behavior == crate::content::host::HostBehavior::ShuigongCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shuigong)
                    }
                    crate::content::host::WD_TOULIANG_KEY => {
                        *behavior == crate::content::host::HostBehavior::TouliangSwap.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Toulianghuanzhu)
                    }
                    crate::content::host::HT_LINGHUN_KEY => {
                        *behavior == crate::content::host::HostBehavior::LinghunzhihuoCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Linghunzhihuo)
                    }
                    crate::content::host::HT_ZHAOMING_KEY => {
                        *behavior == crate::content::host::HostBehavior::ZhaomingdanCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Zhaomingdan)
                    }
                    crate::content::host::WD_CAOCHUAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::CaochuanCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Caochuanjiejian)
                    }
                    crate::content::host::WD_SHEZHAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::ShezhanqunruCompare.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Shezhanqunru)
                    }
                    crate::content::host::GJ_SHUJINSAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianShujinsanCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shujinsan)
                    }
                    crate::content::host::GJ_MIANLI_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianMianliCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Mianlijinzhen)
                    }
                    crate::content::host::GJ_FEIBIAO_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFeibiaoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Feibiao)
                    }
                    crate::content::host::GJ_TANHUADONG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Tanhuadong)
                    }
                    crate::content::host::GJ_YOUGENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Yougeng)
                    }
                    crate::content::host::GJ_QINGTUAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Qingtuan)
                    }
                    crate::content::host::GJ_YUANBAOROU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Yuanbaorou)
                    }
                    crate::content::host::GJ_GUDONGGENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Gudonggeng)
                    }
                    crate::content::host::GJ_MAPODOUFU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Mapodoufu)
                    }
                    crate::content::host::GJ_CHUNBING_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Chunbing)
                    }
                    crate::content::host::GJ_LIYUTANG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Liyutang)
                    }
                    crate::content::host::GJ_MOLICHA_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Molicha)
                    }
                    crate::content::host::GJ_XIAJIAO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Xiajiao)
                    }
                    crate::content::host::GJ_MIZHLIANOU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Mizhilianou)
                    }
                    crate::content::host::GJ_LUYUGENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Luyugeng)
                    }
                    crate::content::host::GJ_BINGPOTONG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanBingpoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Bingpotong)
                    }
                    crate::content::host::GJ_QIANKUNBIAO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanQiankunCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Qiankunbiao)
                    }
                    crate::content::host::GJ_LONGXUGOU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanLongxuCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Longxugou)
                    }
                    crate::content::host::GJ_SHENHUOFEIYA_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanShenhuoCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Shenhuofeiya)
                    }
                    crate::content::host::GJ_ZIYANGDAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Ziyangdan)
                    }
                    crate::content::host::GJ_YUNVYUANSHEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianFoodGrant.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Yunvyuanshen)
                    }
                    crate::content::host::GJ_SHIHUIFEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianShihuifenCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shihuifen)
                    }
                    crate::content::host::GJ_LIUFENGSAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GujianLiufengsanCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Liufengsan)
                    }
                    crate::content::host::GJ_SHATANG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanShatangCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shatang)
                    }
                    crate::content::host::GJ_HEILONGLINPIAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanHeilongCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Heilonglinpian)
                    }
                    crate::content::host::GJ_LIUTOUGE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanLiutougeCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Liutouge)
                    }
                    crate::content::host::GJ_DINVXUANSHUANG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::JiguanDinvCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Dinvxuanshuang)
                    }
                    crate::content::host::HH_DUNPAI_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthDunpaiCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Dunpaigedang)
                    }
                    crate::content::host::HH_ZHILIAOBO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthZhiliaoboCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Zhiliaobo)
                    }
                    crate::content::host::HH_YUANSU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthYuansuCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Yuansuhuimie)
                    }
                    crate::content::host::HH_XINGJIE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthXingjieCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Xingjiegoutong)
                    }
                    crate::content::host::HH_CHUANSONG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthChuansongCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Chuansongmen)
                    }
                    crate::content::host::HH_TANSHE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthTansheCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Tanshezhiren)
                    }
                    crate::content::host::HH_SHIJIE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::HearthShijieshuCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Shijieshu)
                    }
                    crate::content::host::GW_YANZHI_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentYanzhiCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Yanziyaoshui)
                    }
                    crate::content::host::GW_AOZU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentAozuCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Aozuzhilei)
                    }
                    crate::content::host::GW_WENYI_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWenyiCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Wenyi)
                    }
                    crate::content::host::GW_ZIHUAN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentZhihuanCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Zhihuanjun)
                    }
                    crate::content::host::GW_BAOXUE_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentBaoxueCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Baoxueyaoshui)
                    }
                    crate::content::host::GW_QINPEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWeatherGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Qinpendayu)
                    }
                    crate::content::host::GW_BIRIN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWeatherGrant.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Birinongwu)
                    }
                    crate::content::host::GW_CIGU_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentWeatherGrant.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Ciguhanshuang)
                    }
                    crate::content::host::GW_KUNEN_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentKunenCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Kunenfayin)
                    }
                    crate::content::host::GW_SHANBENG_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentShanbengCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Shanbengshu)
                    }
                    crate::content::host::GW_POXIAO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentPoxiaoCard.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Poxiao)
                    }
                    crate::content::host::GW_ZUMO_CARD_KEY => {
                        *behavior == crate::content::host::HostBehavior::GwentZumoCard.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Zumoshoukao)
                    }
                    crate::content::host::YC_GEAN_KEY => {
                        *behavior == crate::content::host::HostBehavior::GeanguanhuoCompare.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Geanguanhuo)
                    }
                    crate::content::host::HT_JIHUO_KEY => {
                        *behavior == crate::content::host::HostBehavior::JihuocardDraw.tag()
                            && catalog.cards.iter().any(|c| c.kind == CardKind::Jihuocard)
                    }
                    crate::content::host::WD_FUDI_KEY => {
                        *behavior == crate::content::host::HostBehavior::FudichouxinCompare.tag()
                            && catalog
                                .cards
                                .iter()
                                .any(|c| c.kind == CardKind::Fudichouxin)
                    }
                    _ => false,
                };
                if !skill_ok && !card_ok {
                    return Err(SnapshotError::UnknownMachine);
                }
            }
        }
        // G1 装备技能生命周期一致性：已装备牌声明的技能必须存在实例
        //（授予/撤销由装备/离场路径维护，快照不得出现「有装备无技能」）。
        for (pi, p) in snapshot.core.players.iter().enumerate() {
            for c in &p.equips {
                for name in &catalog.cards[*c as usize].equip_skills {
                    let ok = catalog
                        .skill_by_name(name)
                        .map(|def| snapshot.core.skills.contains_key(&(pi as u8, def)))
                        .unwrap_or(false);
                    if !ok {
                        return Err(SnapshotError::EquipSkillMismatch { player: pi as u8 });
                    }
                }
            }
        }
        let mut parks = Vec::with_capacity(snapshot.resolution.parks.len());
        for (index, park) in snapshot.resolution.parks.iter().enumerate() {
            parks.push(crate::resolution::Park {
                resume_depth: park.resume_depth,
                slot: park.slot,
                machine: park.machine.restore(&catalog)?,
                // 提出询问的机器才认领已校验应答（见上段唯一匹配）。
                awaiting_answer: asker == Some(index),
            });
        }
        Ok(Self {
            catalog,
            state: GameState {
                core: snapshot.core.clone(),
                resolution: Resolution {
                    next_event_id: snapshot.resolution.next_event_id,
                    next_choice_id: snapshot.resolution.next_choice_id,
                    stack: snapshot.resolution.stack.clone(),
                    parks,
                    open_choice: snapshot.resolution.open_choice.clone(),
                    answer: snapshot.resolution.answer.clone(),
                    fault: None,
                    budget: snapshot.resolution.budget,
                },
            },
            log: Vec::new(),
            last_dedup: snapshot.last_dedup,
            time_slice: None,
            seat_quota: None,
            seat_cmd_counts: Default::default(),
            auth_tokens: Default::default(),
            auth_counter: 0,
            auth_secret: fresh_auth_secret(),
            // N11-a：恢复轮换代次——恢复前签发的旧令牌全部失效
            auth_generation: snapshot.auth_generation + 1,
            segment_open: snapshot.segment_open,
            debug_budget: None,
            // 检查点随快照延续（N07-b）；宿主日志水位重置为 0——
            // 恢复件的日志从空开始，中止时截断到 0。
            segment_checkpoint: snapshot.segment_checkpoint.clone().map(|b| (*b, 0)),
            simulation: false,
        })
    }
}
