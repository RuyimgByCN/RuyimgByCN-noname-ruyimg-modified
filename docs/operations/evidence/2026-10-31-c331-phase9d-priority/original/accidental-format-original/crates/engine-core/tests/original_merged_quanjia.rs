//! O2 全量合并第二批（2026-10-14）：家族 quanjia 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_quanjia_machine {
    //! 第二百三十七批：quanjia 拳甲专属状态机行为测试（ABI 694 / tag 717，单发无续体）。
    //! 全链：damage 事件（来源=当前回合角色）→ optional 确认 → RNG 取样 →
    //! 有【杀】→ 对来源使用虚拟杀；无【杀】→ 加标记。

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

    const QUANJIA: u32 = 3_913_289_765;
    const QUANJIA_SRC: &str = include_str!("../../../content/original/ported/collab/quanjia.skill");

    fn inject_hand_card(
        state: GameState,
        owner: PlayerId,
        kind: CardKind,
        suit: Suit,
        def: u32,
    ) -> (GameState, CardId) {
        let mut tx = Transaction::begin(state);
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(def),
            kind,
            suit,
            rank: 5,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        (tx.commit().state, card)
    }

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
        let compiled = content_compiler::compile_source_named("quanjia-b237", QUANJIA_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        let players = tx.state().players.alive_by_seat();
        let owner = players[0];
        let source = players[1];
        let target = players[2];
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QUANJIA),
        });
        run_effects(&mut tx, effects).unwrap();
        (tx.commit().state, owner, source, target)
    }

    fn fire_damage(state: GameState, source: PlayerId, target: PlayerId) -> (GameState, bool) {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source,
                target,
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

    fn mark_count(state: &GameState, player: PlayerId) -> i32 {
        state
            .players
            .get(player)
            .and_then(|p| p.marks.iter().find(|(m, _)| *m == 0x51A_0001))
            .map_or(0, |(_, c)| *c)
    }

    #[test]
    fn quanjia_no_sha_gains_mark() {
        // 来源手牌只有【桃】（非杀）→ 确认后机内取样无杀 → 加标记。
        let (state, owner, source, target) = setup(20_261_004_01);
        // 清空来源手牌并注入 2 张桃。
        let mut tx = Transaction::begin(state);
        let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let drained: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        tx.state_mut().zones.get_mut(hand).unwrap().cards.clear();
        for card in &drained {
            tx.state_mut()
                .zones
                .get_mut(discard)
                .unwrap()
                .cards
                .push(*card);
            if let Some(card_state) = tx.state_mut().cards.get_mut(*card) {
                card_state.zone = discard;
                card_state.owner = PlayerId::NONE;
            }
        }
        let state = tx.commit().state;
        let (state, _t1) = inject_hand_card(state, source, CardKind::Tao, Suit::Heart, 9_980);
        let (state, _t2) = inject_hand_card(state, source, CardKind::Tao, Suit::Diamond, 9_981);
        // 来源须为当前回合角色。
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.current = source;
        let state = tx.commit().state;

        let (state, opened) = fire_damage(state, source, target);
        assert!(opened, "optional confirm opens");
        let state = resolve(state, owner, vec![1], 20_261_004_11);
        assert!(!state.has_pending_choice(), "single-shot machine");
        assert_eq!(
            mark_count(&state, owner),
            1,
            "no sha in sample → mark gained"
        );
        let hp_after = state.players.get(target).unwrap().hp;
        let _ = hp_after;
    }

    #[test]
    fn quanjia_source_is_owner_silent() {
        // 来源=owner → 机内校验拒绝 → 确认后无效果、无标记。
        let (state, owner, _source, target) = setup(20_261_004_02);
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.current = owner;
        let state = tx.commit().state;

        let (state, opened) = fire_damage(state, owner, target);
        assert!(opened, "confirm opens (filter is owner_alive only)");
        let state = resolve(state, owner, vec![1], 20_261_004_21);
        assert!(!state.has_pending_choice());
        assert_eq!(mark_count(&state, owner), 0, "source==owner rejected");
    }
}
