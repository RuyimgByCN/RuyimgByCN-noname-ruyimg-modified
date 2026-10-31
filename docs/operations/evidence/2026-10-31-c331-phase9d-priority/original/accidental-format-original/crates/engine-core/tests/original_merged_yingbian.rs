//! O2 全量合并第二批（2026-10-14）：家族 yingbian 的 2 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_yingbian_batch1 {
    //! Golden coverage for yingbian 包批次一（ABI 567 门控）：才媛/崇信/新婉仪 + 全包契约。

    use engine_api::{
        CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
        PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const CAIYUAN: u32 = 4_141_410_001;
    const CHONGXIN: u32 = 4_141_410_002;
    const XINWANYI: u32 = 4_141_410_003;
    const QIANGSHOU: u32 = 3_061_185_460;
    const YBZHUIJI: u32 = 3_463_543_744;

    fn hand_count(state: &GameState, player: PlayerId) -> u32 {
        state
            .zones
            .find(ZoneKind::Hand, player)
            .map(|zone| {
                state
                    .zones
                    .get(zone)
                    .map(|z| z.cards.len() as u32)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    fn alloc_card(
        tx: &mut Transaction,
        owner: PlayerId,
        zone: engine_api::ZoneId,
        kind: CardKind,
        def: u32,
        suit: Suit,
        rank: u8,
    ) -> CardId {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(def),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        card
    }

    fn setup(source: &str, defs: &[u32]) -> (Transaction, PlayerId, PlayerId) {
        let package = content_compiler::compile_source_named("yingbian-batch1", source).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
        let seats = tx.state().players.alive_by_seat();
        let (owner, other) = (seats[0], seats[1]);
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }));
        for def in defs {
            effects.push(EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*def),
            });
        }
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        (tx, owner, other)
    }

    fn find_skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .find(|skill| {
                tx.state()
                    .skills
                    .get(**skill)
                    .is_some_and(|instance| instance.def.raw() == def)
            })
            .copied()
            .unwrap()
    }

    #[test]
    fn package_exposes_yingbian_contract() {
        let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
            "../../../content/original/ported/yingbian/yingbian.npack.json"
        ))
        .unwrap();
        assert_eq!(committed.character_index.len(), 23);
        for def in [CAIYUAN, CHONGXIN, XINWANYI, QIANGSHOU, YBZHUIJI] {
            assert!(
                committed
                    .skill_index
                    .iter()
                    .any(|skill| skill.skill_def_id == def),
                "skill {def} must exist"
            );
        }
        assert!(committed
            .character_index
            .iter()
            .any(|character| character.skills.contains(&CAIYUAN)));
        // 晋羊祜 lives in the SP package; its skills (huaiyuan/chongxin/dezhang)
        // are ported into yingbian and referenced cross-package.
        let sp = content_schema::ContentPackage::from_json_bytes(include_bytes!(
            "../../../content/original/ported/sp/sp.npack.json"
        ))
        .unwrap();
        assert!(sp
            .character_index
            .iter()
            .any(|character| character.skills.contains(&CHONGXIN)));
        assert!(committed.character_index.iter().any(|character| {
            character.name_key == "cheliji" && character.skills.contains(&QIANGSHOU)
        }));
        assert!(committed.character_index.iter().any(|character| {
            character.name_key == "xuangongzhu" && character.skills.contains(&YBZHUIJI)
        }));
    }

    #[test]
    fn caiyuan_draws_two_at_end_phase() {
        let source = include_str!("../../../content/original/ported/yingbian/caiyuan.skill");
        let (mut tx, owner, _other) = setup(source, &[CAIYUAN]);
        let before = hand_count(&tx.state(), owner);
        let state = tx.commit().state;
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.phase = Phase::End;
        let fired = dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: engine_api::TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        );
        let mut state = match fired {
            Ok(effects) => match run_effects(&mut tx, effects) {
                Ok(_) => tx.commit().state,
                Err(EffectError::ChoiceRequested) => tx.commit().state,
                Err(e) => panic!("run error: {e:?}"),
            },
            Err(EffectError::ChoiceRequested) => tx.commit().state,
            Err(e) => panic!("dispatch error: {e:?}"),
        };
        if state.has_pending_choice() {
            let pending = state.resolution.pending_choice_id;
            let observer = PlayerId::new(state.resolution.pending_observer);
            let (state2, result) = apply_command(
                state,
                Command {
                    actor: observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(0x5670),
                    kind: CommandKind::ResolveChoice {
                        choice_id: engine_api::ChoiceId::new(pending),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            assert!(
                matches!(result.result, engine_api::CommandResult::Accepted { .. }),
                "caiyuan confirm: {result:?}"
            );
            state = state2;
        }
        assert_eq!(hand_count(&state, owner), before + 2, "caiyuan draws two");
    }

    #[test]
    fn chongxin_activates_and_recasts_selected() {
        let source = include_str!("../../../content/original/ported/yingbian/chongxin.skill");
        let (mut tx, owner, _other) = setup(source, &[CHONGXIN]);
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = alloc_card(&mut tx, owner, hand, CardKind::Sha, 1, Suit::Club, 4);
        let skill = find_skill(&mut tx, owner, CHONGXIN);
        let state = tx.commit().state;
        let (state, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x5671),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: vec![],
                    card_targets: vec![card],
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "chongxin must activate: {result:?}"
        );
        // 重铸：牌进入弃牌堆（替换为摸一张）。
        assert!(
            state
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .is_some_and(|zone| state
                    .zones
                    .get(zone)
                    .is_some_and(|z| z.cards.contains(&card))),
            "recasted card goes to the discard pile"
        );
        assert_eq!(
            hand_count(&state, owner),
            1,
            "recast draws a replacement card"
        );
    }

    #[test]
    fn xinwanyi_stores_card_from_other() {
        let source = include_str!("../../../content/original/ported/yingbian/xinwanyi.skill");
        let (mut tx, owner, other) = setup(source, &[XINWANYI]);
        let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
        let card = alloc_card(&mut tx, other, other_hand, CardKind::Sha, 1, Suit::Spade, 5);
        let state = tx.commit().state;
        let mut tx = Transaction::begin(state);
        let fired = dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: other,
                card,
                targets: vec![owner],
            }],
            &mut TriggerFireRecord::new(),
        );
        let mut state = match fired {
            Ok(effects) => match run_effects(&mut tx, effects) {
                Ok(_) => tx.commit().state,
                Err(EffectError::ChoiceRequested) => tx.commit().state,
                Err(e) => panic!("run error: {e:?}"),
            },
            Err(EffectError::ChoiceRequested) => tx.commit().state,
            Err(e) => panic!("dispatch error: {e:?}"),
        };
        // 确认后进入选牌选择。
        if state.has_pending_choice() {
            let pending = state.resolution.pending_choice_id;
            let observer = PlayerId::new(state.resolution.pending_observer);
            let (state2, result) = apply_command(
                state,
                Command {
                    actor: observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(0x5672),
                    kind: CommandKind::ResolveChoice {
                        choice_id: engine_api::ChoiceId::new(pending),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            assert!(
                matches!(result.result, engine_api::CommandResult::Accepted { .. }),
                "xinwanyi confirm: {result:?}"
            );
            state = state2;
        }
        // 再处理选牌选择（储入技能牌堆）。
        if state.has_pending_choice() {
            let pending = state.resolution.pending_choice_id;
            let observer = PlayerId::new(state.resolution.pending_observer);
            let (state2, result) = apply_command(
                state,
                Command {
                    actor: observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(0x5673),
                    kind: CommandKind::ResolveChoice {
                        choice_id: engine_api::ChoiceId::new(pending),
                        selections: vec![card.raw()],
                        cancelled: false,
                    },
                },
            );
            assert!(
                matches!(result.result, engine_api::CommandResult::Accepted { .. }),
                "xinwanyi card choice: {result:?}"
            );
            state = state2;
        }
        // 牌已从 other 手牌移出并进入拥有者的技能牌堆。
        assert!(
            !state
                .zones
                .find(ZoneKind::Hand, other)
                .is_some_and(|zone| state
                    .zones
                    .get(zone)
                    .is_some_and(|z| z.cards.contains(&card))),
            "card leaves the other's hand"
        );
        assert!(
            state.skill_piles.iter().any(|pile| pile.owner == owner
                && state
                    .zones
                    .get(pile.zone)
                    .is_some_and(|z| z.cards.contains(&card))),
            "card enters the owner's skill pile"
        );
    }
}

mod original_yingbian_xiongshu {
    //! Golden for YINGBIAN 贾充「凶竖」语义（ABI 436）。
    //!
    //! 凶竖：其他角色出牌阶段开始时弃置 X 张牌（X=凶竖标记数，每轮递增）并展示其
    //! 一张手牌，预测其是否会使用该牌；该出牌阶段结束时预测对则其受 1 点伤害，
    //! 错则获得该牌。

    use engine_api::{
        CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{run_effects, EffectError};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::Engine;
    use rule_api::EffectRequest;

    const XIONGSHU: u32 = 3_731_458_110;
    const PACKAGE: &[u8] =
        include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json");

    fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
        let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
        let mut engine = Engine::new();
        engine.load_package(&package).unwrap();
        assert!(matches!(
            engine.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(XIONGSHU as u64),
                kind: CommandKind::StartGame {
                    player_count: 2,
                    rng_seed: 20_260_816,
                    ruleset: "identity-standard".into(),
                },
            }),
            CommandResult::Accepted { .. }
        ));
        let owner = engine.state().turn.current;
        let seats = engine.state().players.alive_by_seat();
        let mut tx = Transaction::begin(engine.state().clone());
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIONGSHU),
        });
        run_effects(&mut tx, effects).unwrap();
        (tx, owner, seats)
    }

    fn skill_instance(tx: &Transaction, owner: PlayerId) -> engine_api::SkillId {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|s| s.def.raw() == XIONGSHU && s.owner == owner)
            .map(|s| s.id)
            .unwrap()
    }

    fn resolve_choice(
        state: GameState,
        actor: PlayerId,
        key: u64,
        selections: Vec<u32>,
        cancelled: bool,
    ) -> GameState {
        let revision = state.revision;
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let (next, result) = apply_command(
            state.clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections,
                    cancelled,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "resolve failed: {:?}",
            result.result
        );
        next
    }

    fn xiongshu(
        player: PlayerId,
        target: PlayerId,
        stage: u8,
        skill: engine_api::SkillId,
    ) -> EffectRequest {
        EffectRequest::ResolveXiongshu {
            player,
            target,
            card: CardId::NONE,
            action: 0,
            stage,
            skill,
        }
    }

    /// 预测“会使用”且牌已离开目标区域 → 目标受 1 点伤害。
    #[test]
    fn xiongshu_correct_prediction_damages() {
        let (mut tx, owner, seats) = setup();
        let target = seats.into_iter().find(|p| *p != owner).unwrap();
        tx.state_mut().turn.current = target;
        tx.state_mut().turn.phase = Phase::Play;
        let skill = skill_instance(&tx, owner);
        // 目标手牌第一张作为展示牌。
        let shown = tx.state().hand_cards(target)[0];
        // stage 0（标记 0 → 直接推进）→ stage 1 开选牌。
        run_effects(&mut tx, vec![xiongshu(owner, target, 0, skill)]).unwrap_err();
        let state = resolve_choice(tx.commit().state, owner, 1, vec![shown.raw()], false);
        // stage 2 预测：0 会使用。
        let state = resolve_choice(state, owner, 2, vec![0], false);
        assert_eq!(state.resolution.pending_choice_id, 0, "预测后无进一步选择");
        let mut tx = Transaction::begin(state.clone());
        // 模拟该牌被使用：移入弃牌堆（离开目标区域）。
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        {
            let c = tx.state_mut().cards.get_mut(shown).unwrap();
            c.zone = discard;
        }
        // stage 3 结算：目标受 1 点伤害。
        let hp_before = tx.state().players.get(target).unwrap().hp;
        run_effects(&mut tx, vec![xiongshu(owner, target, 3, skill)]).unwrap();
        assert_eq!(
            tx.state().players.get(target).unwrap().hp,
            hp_before - 1,
            "预测正确应造成 1 点伤害"
        );
        // 预测状态已清空。
        assert!(
            tx.state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.state_blob.is_empty()),
            "预测状态应清空"
        );
    }

    /// 预测“会使用”但牌仍在目标手牌 → 预测错 → 拥有者获得该牌。
    #[test]
    fn xiongshu_wrong_prediction_gains_card() {
        let (mut tx, owner, seats) = setup();
        let target = seats.into_iter().find(|p| *p != owner).unwrap();
        tx.state_mut().turn.current = target;
        tx.state_mut().turn.phase = Phase::Play;
        let skill = skill_instance(&tx, owner);
        let shown = tx.state().hand_cards(target)[0];
        run_effects(&mut tx, vec![xiongshu(owner, target, 0, skill)]).unwrap_err();
        let state = resolve_choice(tx.commit().state, owner, 1, vec![shown.raw()], false);
        // 预测：0 会使用（但牌实际未用）。
        let state = resolve_choice(state, owner, 2, vec![0], false);
        let mut tx = Transaction::begin(state.clone());
        let hand_before = tx.state().hand_count(owner);
        // stage 3 结算：牌仍在目标手牌 → 预测错 → 获得该牌。
        run_effects(&mut tx, vec![xiongshu(owner, target, 3, skill)]).unwrap();
        assert_eq!(
            tx.state().hand_count(owner),
            hand_before + 1,
            "预测错应获得该牌"
        );
        assert!(
            tx.state().hand_cards(owner).contains(&shown),
            "应获得展示的那张牌"
        );
    }

    /// 奸回（ABI 437）：连续同一来源伤害 → 来源强制弃一张牌；连续同一受伤者 → 摸一张。
    #[test]
    fn jianhui_consecutive_source_discard_and_same_target_draw() {
        const JIANHUI: u32 = 4_269_285_584;
        let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
        let mut engine = Engine::new();
        engine.load_package(&package).unwrap();
        assert!(matches!(
            engine.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(JIANHUI as u64),
                kind: CommandKind::StartGame {
                    player_count: 3,
                    rng_seed: 20_260_817,
                    ruleset: "identity-standard".into(),
                },
            }),
            CommandResult::Accepted { .. }
        ));
        let owner = engine.state().turn.current;
        let seats = engine.state().players.alive_by_seat();
        let mut tx = Transaction::begin(engine.state().clone());
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIANHUI),
        });
        run_effects(&mut tx, effects).unwrap();
        let attacker = seats.iter().copied().find(|p| *p != owner).unwrap();
        let skill = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|s| s.def.raw() == JIANHUI && s.owner == owner)
            .map(|s| s.id)
            .unwrap();
        // 第一段：来源=attacker 对 owner 造成伤害 → 记录（无摸牌/弃牌）。
        let hand_before_first = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveJianhui {
                player: owner,
                source: attacker,
                target: owner,
                skill,
            }],
        )
        .unwrap();
        assert_eq!(
            tx.state().hand_count(owner),
            hand_before_first,
            "首次无奖励"
        );
        // 第二段：同一来源再次对 owner 造成伤害 → 来源强制弃一张（开 choice）。
        let err = run_effects(
            &mut tx,
            vec![EffectRequest::ResolveJianhui {
                player: owner,
                source: attacker,
                target: owner,
                skill,
            }],
        )
        .err();
        assert_eq!(
            err,
            Some(EffectError::ChoiceRequested),
            "连续同来源应开弃牌 choice"
        );
        // 解决来源的强制弃牌（选其一张手牌）。
        let attacker_card = tx.state().hand_cards(attacker)[0];
        let state = resolve_choice(
            tx.commit().state,
            attacker,
            3,
            vec![attacker_card.raw()],
            false,
        );
        let mut tx = Transaction::begin(state.clone());
        // 同一来源对另一目标（第三次）→ 受伤者不同，来源视角：连续同受伤者才摸。
        let other = seats
            .iter()
            .copied()
            .find(|p| *p != owner && *p != attacker)
            .unwrap();
        let hand_before = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveJianhui {
                player: owner,
                source: attacker,
                target: other,
                skill,
            }],
        )
        .unwrap();
        assert_eq!(
            tx.state().hand_count(owner),
            hand_before,
            "来源视角受伤者不同不摸牌"
        );
    }
}
