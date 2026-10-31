//! O2 全量合并第二批（2026-10-14）：家族 qianlong 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_qianlong_machine {
    //! 第二百四十一批：qianlong 潜龙专属状态机行为测试（ABI 697 / tag 720 / Cont 303）。
    //! 全链：damage 事件 → optional 确认 → 顶三张 SelectCards 0..min(3,已损失)
    //! 可取消 → 获得所选 + 余者置牌堆底。债：frequent 口径、拖拽 UI 近似。

    use engine_api::{
        CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
        ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{Card, CardKind, GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const QIANLONG: u32 = 4_122_694_957;
    const QIANLONG_SRC: &str =
        include_str!("../../../content/original/ported/xianding/qianlong.skill");

    fn setup(seed: u64, damage: bool) -> (GameState, PlayerId) {
        let compiled =
            content_compiler::compile_source_named("qianlong-b241", QIANLONG_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        let players = tx.state().players.alive_by_seat();
        let owner = players[0];
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIANLONG),
        });
        run_effects(&mut tx, effects).unwrap();
        if damage {
            // 制造已损失体力（max_hp=4，hp=4 → 打 2 伤 → 已损失 2）。
            let max_hp = tx.state().players.get(owner).unwrap().max_hp;
            let p = tx.state_mut().players.get_mut(owner).unwrap();
            p.hp = max_hp as i32 - 2;
        }
        (tx.commit().state, owner)
    }

    fn fire_damage(state: GameState, owner: PlayerId) -> (GameState, bool) {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source: PlayerId::NONE,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
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

    fn resolve(state: GameState, owner: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor: owner,
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

    fn draw_top3(state: &GameState) -> Vec<CardId> {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let cards = state.zones.get(draw).unwrap().cards.clone();
        cards[cards.len() - 3..].to_vec()
    }

    #[test]
    fn qianlong_gain_up_to_lost_hp_rest_to_bottom() {
        // 已损失 2 → 至多得 2 张；选 1 张获得，其余 2 张置牌堆底。
        let (state, owner) = setup(20_261_007_01, true);
        let top3 = draw_top3(&state);
        assert_eq!(top3.len(), 3);
        let hand_before = state.hand_count(owner);

        let (state, opened) = fire_damage(state, owner);
        assert!(opened, "confirm opens");
        let state = resolve(state, owner, vec![1], 11);
        assert!(state.has_pending_choice(), "top-3 selection opens");

        // 选顶三张中的第一张获得。
        let state = resolve(state, owner, vec![top3[0].raw()], 12);
        assert!(!state.has_pending_choice());

        // 获得 1 张。
        assert_eq!(state.hand_count(owner), hand_before + 1);
        let hand_zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
        assert!(
            state.zones.get(hand_zone).unwrap().cards.contains(&top3[0]),
            "gained card in hand"
        );
        // 余下两张置牌堆底（向量头部前两位，次序保持）。
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let bottom = &state.zones.get(draw).unwrap().cards[..2];
        assert_eq!(bottom[0], top3[1], "second-top card at bottom first");
        assert_eq!(bottom[1], top3[2], "third-top card below it");
    }

    #[test]
    fn qianlong_selection_cap_enforced() {
        // 已损失 2 → 选择 3 张被拒（超出 cap）。
        let (state, owner) = setup(20_261_007_02, true);
        let top3 = draw_top3(&state);
        let (state, _) = fire_damage(state, owner);
        let state = resolve(state, owner, vec![1], 21);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (_, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(22),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: top3.iter().map(|c| c.raw()).collect(),
                    cancelled: false,
                },
            },
        );
        assert!(
            !matches!(result.result, CommandResult::Accepted { .. }),
            "3 selections beyond cap rejected"
        );
    }
}
