//! O2 全量合并第二批（2026-10-14）：家族 chumei 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_chumei_machine {
    //! 第二百六十三批：dchumei 狐魅状态机行为测试（ABI 713 / tag 736 /
    //! Cont 314）。链路：owner 造成伤害（累计 dealt）→ ActivateSkill → 选体力
    //! ≤ dealt 的目标 → 三选一：目标摸 1／目标交 owner 一张手牌／目标回 1
    //! （受伤时）。体力和 owner 伤害计数由机内权威判定。

    use engine_api::{
        CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::ZoneKind;
    use engine_core::state::{CardKind, GameState};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const CHUMEI: u32 = 3_905_424_632;
    const CHUMEI_SRC: &str = include_str!("../../../content/original/ported/sp2/dchumei.skill");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled = content_compiler::compile_source_named("chumei-b263", CHUMEI_SRC).unwrap();
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
            skill_def: SkillDefId::new(CHUMEI),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.phase = engine_core::state::Phase::Play;
        (tx.commit().state, owner, target)
    }

    /// owner 对目标造成 2 点伤害（累计 dealt=2），并把目标体力压至 2（≤2 满足
    /// 门控，且受伤 → recover 选项可用）。
    fn deal_and_wound(mut state: GameState, owner: PlayerId, target: PlayerId) -> GameState {
        // 目标体力压至 3（≤上限），随后两次各 1 点伤害：dealt=2，目标存活且受伤。
        {
            let max_hp = state.players.get(target).unwrap().max_hp as i32;
            let p = state.players.get_mut(target).unwrap();
            p.hp = 3.min(max_hp);
        }
        for _ in 0..2 {
            let mut tx = Transaction::begin(state);
            let _ = run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source: owner,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                }],
            )
            .expect("damage pipeline");
            state = tx.commit().state;
        }
        state
    }

    fn activate(state: GameState, owner: PlayerId) -> GameState {
        let sid = state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.def.raw() == CHUMEI)
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

    #[test]
    fn chumei_give_arm_target_gives_owner_card() {
        let (mut state, owner, target) = setup(20_260_913_41);
        let state = deal_and_wound(state, owner, target);
        let o_hand = state.hand_count(owner) as usize;
        let t_hand = state.hand_count(target) as usize;

        let state = activate(state, owner);
        assert!(state.has_pending_choice(), "target selection opens");
        let state = resolve(state, owner, vec![target.raw()], 51);

        // 三选一：选 2（目标交一张牌）。
        let state = resolve(state, owner, vec![2], 52);
        assert!(state.has_pending_choice(), "give pick opens for target");
        let given = state.hand_cards(target).first().copied().unwrap();
        let state = resolve(state, target, vec![given.raw()], 53);
        assert!(!state.has_pending_choice());

        assert!(state.hand_cards(owner).contains(&given));
        assert_eq!(state.hand_count(owner) as usize, o_hand + 1);
        assert_eq!(state.hand_count(target) as usize, t_hand - 1);
    }

    #[test]
    fn chumei_draw_arm_target_draws() {
        let (mut state, owner, target) = setup(20_260_913_42);
        let state = deal_and_wound(state, owner, target);
        let t_hand = state.hand_count(target) as usize;

        let state = activate(state, owner);
        let state = resolve(state, owner, vec![target.raw()], 61);
        let state = resolve(state, owner, vec![1], 62);
        assert!(!state.has_pending_choice());
        assert_eq!(state.hand_count(target) as usize, t_hand + 1);
    }

    #[test]
    fn chumei_recover_arm_target_heals() {
        let (mut state, owner, target) = setup(20_260_913_43);
        let state = deal_and_wound(state, owner, target);
        let hp_target = state.players.get(target).unwrap().hp;

        let state = activate(state, owner);
        let state = resolve(state, owner, vec![target.raw()], 71);
        let state = resolve(state, owner, vec![3], 72);
        assert!(!state.has_pending_choice());
        assert_eq!(state.players.get(target).unwrap().hp, hp_target + 1);
    }

    #[test]
    fn chumei_zero_dealt_gate_blocks_activation() {
        // 未造成伤害：门控（dealt=0 → 无体力≤0 的存活目标）→ 发动后无选择打开。
        let (state, owner, _target) = setup(20_260_913_44);
        let state = activate(state, owner);
        assert!(!state.has_pending_choice(), "zero dealt must stay silent");
        let _ = CardKind::Sha.tag();
    }
}
