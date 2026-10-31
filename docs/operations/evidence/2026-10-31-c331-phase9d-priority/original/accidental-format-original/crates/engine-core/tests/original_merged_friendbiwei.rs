//! O2 全量合并第二批（2026-10-14）：家族 friendbiwei 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_friendbiwei_machine {
    //! 第二百七十五批：friendbiwei 鄙位状态机行为测试（ABI 724 / tag 748，
    //! 单发无续体）。链路：出牌阶段限一次，弃手牌中点数最大的一张 → 目标弃置
    //! 所有点数≥该牌的手牌；若目标未因此弃牌，重置技能使用计数。

    use engine_api::{
        CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
        PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::{apply_command, PipelineResult};
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const FRENDBIWEI: u32 = 2_799_409_601;
    const FRENDBIWEI_SRC: &str =
        include_str!("../../../content/original/ported/mobile/friendbiwei.skill");
    const MOBILE: &[u8] =
        include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

    struct Setup {
        state: GameState,
        owner: PlayerId,
        target: PlayerId,
        skill: engine_api::SkillId,
    }

    fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
        let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(98_500 + u32::from(rank)),
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

    fn setup(seed: u64) -> Setup {
        let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
        let compiled =
            content_compiler::compile_source_named("friendbiwei-test", FRENDBIWEI_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
        let players = tx.state().players.alive_by_seat();
        let (owner, target) = (players[0], players[1]);
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.extend(
            [owner, target]
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        );
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FRENDBIWEI),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        let skill = tx
            .state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == FRENDBIWEI)
                    .then_some(*skill)
            })
            .unwrap();
        Setup {
            state: tx.commit().state,
            owner,
            target,
            skill,
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

    fn discard_pile(state: &GameState) -> Vec<CardId> {
        let zone = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        state.zones.get(zone).unwrap().cards.clone()
    }

    #[test]
    fn target_discards_all_cards_of_equal_or_higher_rank() {
        let mut setup = setup(20_260_913_701);
        let mut tx = Transaction::begin(setup.state);
        // owner：3、5、9（9 为最大点数成本牌）。
        inject(&mut tx, setup.owner, Suit::Club, 3);
        inject(&mut tx, setup.owner, Suit::Heart, 5);
        let cost = inject(&mut tx, setup.owner, Suit::Spade, 9);
        // target：7、9、J(11)——全部 ≥9。
        inject(&mut tx, setup.target, Suit::Club, 7);
        inject(&mut tx, setup.target, Suit::Spade, 9);
        inject(&mut tx, setup.target, Suit::Heart, 11);
        setup.state = tx.commit().state;
        let t_hand_before = hand_of(&setup.state, setup.target);

        let (state, outcome) = submit(
            setup.state.clone(),
            setup.owner,
            CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![cost],
            },
            1,
        );
        assert!(
            matches!(outcome.result, CommandResult::Accepted { .. }),
            "activation rejected: {:?}",
            outcome.result
        );
        // owner 恰好弃成本牌；target 的 7(<9) 留在手上，9 与 11 被弃。
        assert_eq!(hand_of(&state, setup.owner).len(), 2);
        assert_eq!(hand_of(&state, setup.target).len(), 1);
        let remaining = hand_of(&state, setup.target)[0];
        assert_eq!(state.cards.get(remaining).unwrap().rank, 7);
        for card in &t_hand_before {
            if *card != remaining {
                assert!(discard_pile(&state).contains(card));
            }
        }
        assert!(discard_pile(&state).contains(&cost));
        assert!(!state.has_pending_choice());
    }

    #[test]
    fn no_discard_refunds_the_skill_use_for_same_turn_reuse() {
        let mut setup = setup(20_260_913_702);
        let mut tx = Transaction::begin(setup.state);
        // owner：4、9（9 为成本）。
        inject(&mut tx, setup.owner, Suit::Club, 4);
        let cost = inject(&mut tx, setup.owner, Suit::Spade, 9);
        // target：仅 3——无 ≥9 的牌。
        inject(&mut tx, setup.target, Suit::Heart, 3);
        setup.state = tx.commit().state;

        let (state, outcome) = submit(
            setup.state.clone(),
            setup.owner,
            CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![cost],
            },
            1,
        );
        assert!(matches!(outcome.result, CommandResult::Accepted { .. }));
        assert_eq!(
            hand_of(&state, setup.target).len(),
            1,
            "only the 3 (<9) stays"
        );
        assert!(discard_pile(&state).contains(&cost));

        // 使用计数已重置：同回合可再次发动。
        let instance_use = state.skills.get(setup.skill).unwrap().play_phase_use_count;
        assert_eq!(instance_use, 0, "use refunded for same-turn reuse");
        let mut setup2 = Setup { state, ..setup };
        let mut tx = Transaction::begin(setup2.state);
        let extra = inject(&mut tx, setup2.owner, Suit::Spade, 13);
        setup2.state = tx.commit().state;
        let (_, outcome2) = submit(
            setup2.state.clone(),
            setup2.owner,
            CommandKind::ActivateSkill {
                skill: setup2.skill,
                targets: vec![setup2.target],
                card_targets: vec![extra],
            },
            2,
        );
        assert!(
            matches!(outcome2.result, CommandResult::Accepted { .. }),
            "second activation allowed after refund: {:?}",
            outcome2.result
        );
    }

    #[test]
    fn non_highest_rank_card_is_rejected() {
        let mut setup = setup(20_260_913_703);
        let mut tx = Transaction::begin(setup.state);
        inject(&mut tx, setup.owner, Suit::Club, 3);
        inject(&mut tx, setup.owner, Suit::Spade, 9);
        inject(&mut tx, setup.target, Suit::Heart, 3);
        setup.state = tx.commit().state;

        // 选非最大点数（3）→ 拒绝。
        let hand = hand_of(&setup.state, setup.owner);
        let low = hand
            .iter()
            .copied()
            .find(|card| setup.state.cards.get(*card).unwrap().rank == 3)
            .unwrap();
        let (_, outcome) = submit(
            setup.state.clone(),
            setup.owner,
            CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![low],
            },
            1,
        );
        assert!(
            matches!(outcome.result, CommandResult::Rejected { .. }),
            "non-highest card must be rejected: {:?}",
            outcome.result
        );
    }
}
