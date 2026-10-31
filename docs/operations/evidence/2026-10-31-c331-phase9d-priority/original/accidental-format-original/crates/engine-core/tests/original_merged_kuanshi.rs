//! O2 全量合并第二批（2026-10-14）：家族 kuanshi 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_kuanshi_mark_protection {
    //! 第二百四十四批：kuanshi 宽释零铸造重写行为测试（sp 包，零 ABI）。
    //! 链路：结束阶段确认 → 选其他角色授「宽释」标记 → 目标受 >1 伤害时 forced
    //! 防止并消耗标记 → 1 点伤害不触发 → 目标回合开始过期移除。
    //! 债：授予者跳过摸牌阶段未建模；到期时点按目标回合收窄；不可选自己。

    use engine_api::{
        ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::GameState;
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::{damage_nature, EffectRequest};

    const KUANSHI: u32 = 2_742_461_307;
    const KUANSHI_MARK: u32 = 30_507_535;
    const KUANSHI_SRC: &str = include_str!("../../../content/original/ported/sp/kuanshi.skill");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled = content_compiler::compile_source_named("kuanshi-b244", KUANSHI_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        let players = tx.state().players.alive_by_seat();
        let owner = players[0];
        let target = players[1];
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(KUANSHI),
        });
        run_effects(&mut tx, effects).unwrap();
        (tx.commit().state, owner, target)
    }

    fn mark_count(state: &GameState, player: PlayerId) -> i32 {
        state
            .players
            .get(player)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == KUANSHI_MARK)
            .map_or(0, |(_, count)| *count)
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

    fn grant(state: GameState, owner: PlayerId, target: PlayerId, key: u64) -> GameState {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ) {
            Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
            Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
            Err(other) => panic!("unexpected: {other:?}"),
        };
        assert!(
            matches!(outcome, Err(EffectError::ChoiceRequested)),
            "grant confirm opens"
        );
        let state = tx.commit().state;
        let state = resolve(state, owner, vec![1], key);
        assert!(state.has_pending_choice(), "target selection opens");
        let state = resolve(state, owner, vec![target.raw()], key + 1);
        assert!(!state.has_pending_choice());
        assert_eq!(mark_count(&state, target), 1);
        state
    }

    fn damage(state: GameState, source: PlayerId, target: PlayerId, amount: u32) -> GameState {
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            }],
        )
        .expect("damage pipeline");
        tx.commit().state
    }

    #[test]
    fn kuanshi_prevents_damage_over_one_and_consumes_mark() {
        let (state, owner, target) = setup(20_260_912_21);
        let hp = state.players.get(target).unwrap().hp;

        // 授予保护。
        let state = grant(state, owner, target, 41);

        // 2 点伤害：forced 防止（无选择），标记消耗，体力不变。
        let state = damage(state, owner, target, 2);
        assert_eq!(state.players.get(target).unwrap().hp, hp);
        assert_eq!(mark_count(&state, target), 0);

        // 标记已耗：2 点伤害生效。
        let state = damage(state, owner, target, 2);
        assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    }

    #[test]
    fn kuanshi_does_not_trigger_on_single_damage() {
        let (state, owner, target) = setup(20_260_912_22);
        let hp = state.players.get(target).unwrap().hp;
        let state = grant(state, owner, target, 51);

        // 1 点伤害不满足 amount>1 门控：照常生效，标记保留。
        let state = damage(state, owner, target, 1);
        assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
        assert_eq!(mark_count(&state, target), 1);

        // 随后 2 点伤害仍被防止（保护未被小额伤害消耗）。
        let state = damage(state, owner, target, 2);
        assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
        assert_eq!(mark_count(&state, target), 0);
    }

    #[test]
    fn kuanshi_mark_expires_at_target_turn_start() {
        let (state, owner, target) = setup(20_260_912_23);
        let state = grant(state, owner, target, 61);

        // 目标回合开始（Prepare）：过期臂移除全部标记。
        let mut tx = Transaction::begin(state);
        let effects = dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: target,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        )
        .expect("forced expiry dispatch");
        engine_core::effect::run_effects(&mut tx, effects).expect("expiry applies");
        let state = tx.commit().state;
        assert_eq!(mark_count(&state, target), 0);

        // 过期后 2 点伤害照常生效。
        let state = damage(state, owner, target, 2);
        let hp = state.players.get(target).unwrap().hp;
        assert!(hp < 10);
    }
}
