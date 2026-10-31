//! O2 全量合并第二批（2026-10-14）：家族 shuaiyan 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_shuaiyan_machine {
    //! 第二百六十六批：shuaiyan 率言状态机行为测试（ABI 715 / tag 738 /
    //! Cont 315）。链路：弃牌阶段开始（手牌>1）→ optional 确认 → SelectTargets
    //! 其他有牌角色 → 目标从手牌/装备区选一张交给 owner。

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

    const SHUAIYAN: u32 = 2_856_434_937;
    const SHUAIYAN_SRC: &str =
        include_str!("../../../content/original/ported/mobile/shuaiyan.skill");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled =
            content_compiler::compile_source_named("shuaiyan-b266", SHUAIYAN_SRC).unwrap();
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
            skill_def: SkillDefId::new(SHUAIYAN),
        });
        run_effects(&mut tx, effects).unwrap();
        (tx.commit().state, owner, other)
    }

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        state.zones.get(hand).unwrap().cards.clone()
    }

    fn fire_discard(state: GameState, owner: PlayerId) -> (GameState, bool) {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Discard,
            }],
            &mut TriggerFireRecord::new(),
        ) {
            Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
            Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
            Err(other) => panic!("unexpected: {other:?}"),
        };
        let opened = matches!(outcome, Err(EffectError::ChoiceRequested));
        (tx.commit().state, opened)
    }

    fn resolve(
        state: GameState,
        actor: PlayerId,
        selections: Vec<u32>,
        cancelled: bool,
        key: u64,
    ) -> GameState {
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
                    cancelled,
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

    #[test]
    fn shuaiyan_target_gives_owner_one_card() {
        let (state, owner, other) = setup(20_260_913_101);
        let o_hand = state.hand_count(owner) as usize;
        let t_hand = hand_of(&state, other);
        assert!(t_hand.len() >= 1);
        let t_pick = t_hand.first().copied().unwrap();

        let (state, opened) = fire_discard(state, owner);
        assert!(opened, "confirm opens");
        let state = resolve(state, owner, vec![1], false, 11);
        assert!(state.has_pending_choice(), "target selection opens");
        let state = resolve(state, owner, vec![other.raw()], false, 12);
        assert!(state.has_pending_choice(), "give pick opens for target");
        let state = resolve(state, other, vec![t_pick.raw()], false, 13);
        assert!(!state.has_pending_choice());

        // 目标手牌 -1，owner 手牌 +1 且为所给之牌。
        assert_eq!(state.hand_count(other) as usize, t_hand.len() - 1);
        assert_eq!(state.hand_count(owner) as usize, o_hand + 1);
        assert!(hand_of(&state, owner).contains(&t_pick));
    }

    #[test]
    fn shuaiyan_cancel_places_nothing() {
        let (state, owner, other) = setup(20_260_913_102);
        let o_hand = state.hand_count(owner) as usize;
        let t_hand = state.hand_count(other) as usize;

        let (state, opened) = fire_discard(state, owner);
        assert!(opened);
        let state = resolve(state, owner, vec![], true, 21);
        assert!(!state.has_pending_choice());
        assert_eq!(state.hand_count(owner) as usize, o_hand);
        assert_eq!(state.hand_count(other) as usize, t_hand);
    }
}
