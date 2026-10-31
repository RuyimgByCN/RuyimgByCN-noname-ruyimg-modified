//! O2 全量合并第二批（2026-10-14）：家族 zhongyan 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_zhongyan_machine {
    //! 第二百六十二批：starzhongyan 忠言状态机行为测试（ABI 712 / tag 735 /
    //! Cont 313）。链路：ActivateSkill → 选有手牌目标 → 亮牌堆顶 3 张 → 目标选
    //! 亮牌 1 张 → 目标选自己手牌 1 张 → 交换（亮牌入手，手牌回牌堆顶）。

    use engine_api::{
        CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
        PlayerId, SkillDefId,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{CardKind, GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const ZHONGYAN: u32 = 3_150_768_048;
    const ZHONGYAN_SRC: &str =
        include_str!("../../../content/original/ported/sp2/starzhongyan.skill");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled =
            content_compiler::compile_source_named("zhongyan-b262", ZHONGYAN_SRC).unwrap();
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
            skill_def: SkillDefId::new(ZHONGYAN),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.phase = engine_core::state::Phase::Play;
        (tx.commit().state, owner, other)
    }

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        state.zones.get(hand).unwrap().cards.clone()
    }

    fn draw_top(state: &GameState, n: usize) -> Vec<CardId> {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let cards = state.zones.get(draw).unwrap().cards.clone();
        cards[cards.len() - n.min(cards.len())..].to_vec()
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

    fn activate(state: GameState, owner: PlayerId) -> GameState {
        let sid = state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.def.raw() == ZHONGYAN)
            .unwrap()
            .id;
        let revision = state.revision;
        let (state, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::ActivateSkill {
                    skill: sid,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "activate rejected"
        );
        state
    }

    #[test]
    fn zhongyan_reveal_and_exchange_full_chain() {
        let (state, owner, target) = setup(20_260_913_31);
        let t_hand_before = state.hand_count(target);
        assert!(t_hand_before >= 1);
        let top3 = draw_top(&state, 3);

        // 发动 → 选目标（other）。
        let state = activate(state, owner);
        assert!(state.has_pending_choice(), "target selection opens");
        let state = resolve(state, owner, vec![target.raw()], false, 11);

        // 目标选一张亮牌（取第一张）。
        assert!(state.has_pending_choice(), "reveal pick opens");
        let state = resolve(state, target, vec![top3[0].raw()], false, 12);

        // 目标选自己一张手牌交换。
        let target_hand = hand_of(&state, target);
        let swapped_out = target_hand
            .iter()
            .copied()
            .find(|c| !top3.contains(c))
            .unwrap_or(target_hand[0]);
        let state = resolve(state, target, vec![swapped_out.raw()], false, 13);
        assert!(!state.has_pending_choice());

        // 交换断言：亮牌入手，换出的手牌回到牌堆顶。
        let t_hand_now = hand_of(&state, target);
        assert!(t_hand_now.contains(&top3[0]), "revealed card gained");
        assert!(!t_hand_now.contains(&swapped_out), "hand card swapped out");
        let new_top = draw_top(&state, 3);
        assert!(new_top.contains(&swapped_out), "hand card back to deck top");
        assert_eq!(state.hand_count(target), t_hand_before, "1:1 exchange");
        let _ = CardKind::Sha.tag();
    }

    #[test]
    fn zhongyan_cancellable_target_selection() {
        let (state, owner, _target) = setup(20_260_913_32);
        let state = activate(state, owner);
        assert!(state.has_pending_choice());
        let _ = resolve(state, owner, vec![], true, 21);
    }
}
