//! O2 全量合并第二批（2026-10-14）：家族 chulao 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_chulao_active {
    //! 第二百七十四批：chulao 黜老状态机行为测试（ABI 723 / tag 747 /
    //! Cont 322）。链路：出牌阶段限一次，弃一张成本牌（黑桃 → owner 摸 1）→
    //! 对每个势力各异目标由 owner 选弃其一张牌（被弃牌黑桃 → 目标摸 1）。

    use engine_api::{
        CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::{apply_command, PipelineResult};
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const CHULAO: u32 = 3_352_443_727;
    const CHULAO_SRC: &str = include_str!("../../../content/original/ported/refresh/chulao.skill");
    const REFRESH: &[u8] =
        include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

    struct Setup {
        state: GameState,
        owner: PlayerId,
        t1: PlayerId,
        t2: PlayerId,
        cost: CardId,
    }

    fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
        let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(98_400 + u32::from(rank)),
            kind: CardKind::Other,
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

    fn setup() -> Setup {
        let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
        let compiled = content_compiler::compile_source_named("chulao-test", CHULAO_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        bootstrap_match(&mut tx, 3, 20_260_913_601, "identity-standard").unwrap();
        let players = tx.state().players.alive_by_seat();
        let (owner, t1, t2) = (players[0], players[1], players[2]);
        // 势力各异：wei / shu / wu。
        tx.state_mut().group_overrides.insert(owner, "wei".into());
        tx.state_mut().group_overrides.insert(t1, "shu".into());
        tx.state_mut().group_overrides.insert(t2, "wu".into());
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }));
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHULAO),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        let cost = inject(&mut tx, owner, Suit::Spade, 9);
        inject(&mut tx, t1, Suit::Heart, 5);
        inject(&mut tx, t2, Suit::Club, 6);
        Setup {
            state: tx.commit().state,
            owner,
            t1,
            t2,
            cost,
        }
    }

    fn submit(
        state: GameState,
        actor: PlayerId,
        kind: CommandKind,
        key: u64,
    ) -> (GameState, PipelineResult) {
        let revision = state.revision;
        apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind,
            },
        )
    }

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        state.hand_cards(player)
    }

    #[test]
    fn package_publishes_chulao_active_multi_target() {
        let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 723);
        let active = package
            .active_index
            .iter()
            .find(|active| active.skill_def_id == CHULAO)
            .expect("chulao active entry");
        let targets = active.targets.as_ref().unwrap();
        assert_eq!(targets.min, 1);
        assert!(targets.max >= 2);
        assert!(!targets.allow_self);
    }

    #[test]
    fn cost_and_target_spade_draws_flow_through_both_targets() {
        let setup = setup();
        let owner_hand_before = hand_of(&setup.state, setup.owner).len();
        let t1_hand_before = hand_of(&setup.state, setup.t1).len();
        let t2_hand_before = hand_of(&setup.state, setup.t2).len();
        let discard = setup
            .state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();

        // 激活对 t1、t2 → 第一步：owner 选成本牌。
        let (state, cost_choice) = submit(
            setup.state.clone(),
            setup.owner,
            CommandKind::ActivateSkill {
                skill: setup_skill(&setup),
                targets: vec![setup.t1, setup.t2],
                card_targets: vec![],
            },
            1,
        );
        eprintln!(
            "DBG activate: result={:?} choice={:?}",
            cost_choice.result,
            cost_choice
                .choice
                .as_ref()
                .map(|c| (c.observer, c.kind, &c.options))
        );
        let cost_choice = cost_choice.choice.unwrap();
        assert_eq!(cost_choice.observer, setup.owner);
        assert_eq!(cost_choice.kind, ChoiceKind::SelectCards);

        // 选黑桃成本牌 → 成本弃置 + owner 摸 1 → 对 t1 开选牌。
        let (state, pick1) = submit(
            state,
            setup.owner,
            CommandKind::ResolveChoice {
                choice_id: cost_choice.choice_id,
                selections: vec![setup.cost.raw()],
                cancelled: false,
            },
            2,
        );
        let pick1 = pick1.choice.unwrap();
        assert_eq!(pick1.observer, setup.owner);
        assert_eq!(pick1.kind, ChoiceKind::SelectCards);
        assert_eq!(
            pick1.options.len(),
            1,
            "t1 has exactly one discardable card"
        );
        // 成本黑桃：owner 摸 1（手牌 = 注入剩余 0 + 摸 1）。
        assert_eq!(
            hand_of(&state, setup.owner).len(),
            owner_hand_before - 1 + 1
        );
        // 成本牌入弃牌堆。
        assert!(state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .contains(&setup.cost));

        // 选弃 t1 的红桃（非黑桃）→ t1 不摸 → 对 t2 开选牌。
        let t1_pick = hand_of(&state, setup.t1)[0];
        let (state, pick2) = submit(
            state,
            setup.owner,
            CommandKind::ResolveChoice {
                choice_id: pick1.choice_id,
                selections: vec![t1_pick.raw()],
                cancelled: false,
            },
            3,
        );
        let pick2 = pick2.choice.unwrap();
        assert_eq!(pick2.observer, setup.owner);
        assert_eq!(
            pick2.options.len(),
            1,
            "t2 has exactly one discardable card"
        );

        // 选弃 t2 的梅花（非黑桃）→ t2 不摸 → 结束。
        let t2_pick = hand_of(&state, setup.t2)[0];
        let (state, done) = submit(
            state,
            setup.owner,
            CommandKind::ResolveChoice {
                choice_id: pick2.choice_id,
                selections: vec![t2_pick.raw()],
                cancelled: false,
            },
            4,
        );
        assert!(matches!(done.result, CommandResult::Accepted { .. }));
        assert_eq!(hand_of(&state, setup.t1).len(), t1_hand_before - 1);
        assert_eq!(hand_of(&state, setup.t2).len(), t2_hand_before - 1);
        assert!(state.zones.get(discard).unwrap().cards.contains(&t1_pick));
        assert!(state.zones.get(discard).unwrap().cards.contains(&t2_pick));
        assert!(!state.has_pending_choice());
    }

    #[test]
    fn duplicate_groups_and_cost_cancel_are_rejected_or_noop() {
        // 同势力双目标：激活被拒。
        let mut base = setup();
        base.state.group_overrides.insert(base.t2, "shu".into());
        let (_, result) = submit(
            base.state.clone(),
            base.owner,
            CommandKind::ActivateSkill {
                skill: setup_skill(&base),
                targets: vec![base.t1, base.t2],
                card_targets: vec![],
            },
            11,
        );
        assert!(
            matches!(result.result, CommandResult::Rejected { .. }),
            "duplicate groups must be rejected: {:?}",
            result.result
        );

        // 取消成本选择：无任何结算。
        let setup2 = setup();
        let discard = setup2
            .state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let (state, cost_choice) = submit(
            setup2.state.clone(),
            setup2.owner,
            CommandKind::ActivateSkill {
                skill: setup_skill(&setup2),
                targets: vec![setup2.t1],
                card_targets: vec![],
            },
            12,
        );
        let cost_choice = cost_choice.choice.unwrap();
        let (state, _) = submit(
            state,
            setup2.owner,
            CommandKind::ResolveChoice {
                choice_id: cost_choice.choice_id,
                selections: vec![],
                cancelled: true,
            },
            13,
        );
        assert!(!state.has_pending_choice());
        assert!(!state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .contains(&setup2.cost));
        assert!(state.hand_cards(setup2.owner).contains(&setup2.cost));
    }

    fn setup_skill(setup: &Setup) -> engine_api::SkillId {
        // 激活按 def 定位技能实例：遍历 owner 实例。
        setup
            .state
            .players
            .get(setup.owner)
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                setup
                    .state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == CHULAO)
                    .then_some(*skill)
            })
            .unwrap()
    }
}
