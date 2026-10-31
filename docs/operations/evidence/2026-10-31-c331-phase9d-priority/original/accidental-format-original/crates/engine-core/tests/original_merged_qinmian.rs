//! O2 全量合并第二批（2026-10-14）：家族 qinmian 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_qinmian_machine {
    //! 第二百六十五批：qinmian 勤勉状态机行为测试（ABI 714 / tag 737）。
    //! 锁定技三臂：准备阶段摸 1 打「勤勉」标；受伤后按点数摸牌打标；任意角色
    //! 结束阶段可弃全部标记牌并对当前回合角色造成等量（全手牌皆标记）或 1 点
    //! 伤害。测试注入已知手牌保证结算确定。

    use engine_api::{
        CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const QINMIAN: u32 = 4_117_221_194;
    const QINMIAN_TAG: u32 = 3_958_551_236;
    const QINMIAN_SRC: &str = include_str!("../../../content/original/ported/sp/qinmian.skill");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled = content_compiler::compile_source_named("qinmian-b265", QINMIAN_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        let players = tx.state().players.alive_by_seat();
        let owner = players[0];
        let other = players[1];
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QINMIAN),
        });
        run_effects(&mut tx, effects).unwrap();
        (tx.commit().state, owner, other)
    }

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        state.zones.get(hand).unwrap().cards.clone()
    }

    fn tagged_count(state: &GameState, player: PlayerId) -> usize {
        hand_of(state, player)
            .iter()
            .filter(|card| state.card_has_content_tag(**card, QINMIAN_TAG))
            .count()
    }

    fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections,
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "resolve rejected {key}: {:?}",
            result.result
        );
        next
    }

    fn fire_phase(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> (GameState, bool) {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged { current, phase }],
            &mut TriggerFireRecord::new(),
        ) {
            Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
            Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
            Err(other) => panic!("unexpected: {other:?}"),
        };
        let opened = matches!(outcome, Err(EffectError::ChoiceRequested));
        (tx.commit().state, opened)
    }

    #[test]
    fn qinmian_prepare_draws_one_tagged() {
        let (state, owner, _other) = setup(20_260_913_81);
        let hand_before = state.hand_count(owner);
        let (state, _opened) = fire_phase(state, owner, TurnPhaseTag::Prepare);
        // frequent 强制自动：直摸不打开确认。
        assert_eq!(state.hand_count(owner), hand_before + 1);
        assert_eq!(tagged_count(&state, owner), 1);
    }

    #[test]
    fn qinmian_damage_draws_per_point() {
        let (state, owner, other) = setup(20_260_913_82);
        let hand_before = state.hand_count(owner);
        let mut tx = Transaction::begin(state);
        let _ = run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .expect("damage");
        let state = tx.commit().state;
        // 2 点伤害 → 摸 2 张，均打标。
        assert_eq!(state.hand_count(owner), hand_before + 2);
        assert_eq!(tagged_count(&state, owner), 2);
    }

    #[test]
    fn qinmian_end_phase_settles_damage_equal_to_tags() {
        // 准备阶段摸 1 张标记牌；再普通摸 1 张（非标记，全手牌非皆标记 → 伤害
        // 1 点）；other 结束阶段确认结算。
        let (state, owner, other) = setup(20_260_913_83);
        let (state, _opened) = fire_phase(state, owner, TurnPhaseTag::Prepare);
        assert_eq!(tagged_count(&state, owner), 1);
        let mut tx = Transaction::begin(state);
        let _ = run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: owner,
                count: 1,
            }],
        );
        let state = tx.commit().state;
        let hp_other = state.players.get(other).unwrap().hp;

        // 显式推进回合到 other（直派事件不推进 turn 状态）。
        let state = {
            let mut st = state;
            st.turn.current = other;
            st
        };

        let (state, opened) = fire_phase(state, other, TurnPhaseTag::End);
        assert!(opened, "settlement opens on any player's end phase");
        let state = resolve(state, owner, vec![1], 41);
        assert!(!state.has_pending_choice());
        println!(
            "PROBE final tagged ids: {:?} (hand={:?})",
            hand_of(&state, owner)
                .iter()
                .filter(|c| state.card_has_content_tag(**c, QINMIAN_TAG))
                .map(|c| c.raw())
                .collect::<Vec<_>>(),
            hand_of(&state, owner)
                .iter()
                .map(|c| c.raw())
                .collect::<Vec<_>>()
        );
        assert_eq!(tagged_count(&state, owner), 0);
        assert_eq!(state.players.get(other).unwrap().hp, hp_other - 1);
    }
}
