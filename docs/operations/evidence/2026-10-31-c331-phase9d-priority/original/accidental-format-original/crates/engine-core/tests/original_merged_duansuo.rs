//! O2 全量合并第二批（2026-10-14）：家族 duansuo 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_duansuo_active {
    //! 第二百七十批：duansuo 断索主动原语行为测试（ABI 719 / tag 743，无续体）。
    //! 链路：出牌阶段限一次 → 选 1..N 名连环状态角色（linked 过滤器）→ 先全体
    //! 重置（移除 MARK_CHAIN）→ 再对全体各造成 1 点火焰伤害（重置后不经链传导）。

    use engine_api::{
        ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MarkDefId,
        PlayerId, SkillDefId,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{GameState, Phase};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const DUANSUO: u32 = 2_917_815_938;
    const DUANSUO_SRC: &str = include_str!("../../../content/original/ported/mobile/duansuo.skill");
    const MOBILE: &[u8] =
        include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

    struct Setup {
        state: GameState,
        owner: PlayerId,
        skill: engine_api::SkillId,
    }

    fn setup(seed: u64) -> Setup {
        let compiled = content_compiler::compile_source_named("duansuo-test", DUANSUO_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(DUANSUO),
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
                    .is_some_and(|instance| instance.def.raw() == DUANSUO)
                    .then_some(*skill)
            })
            .unwrap();
        Setup {
            state: tx.commit().state,
            owner,
            skill,
        }
    }

    fn link(state: GameState, player: PlayerId) -> GameState {
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddMark {
                player,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            }],
        )
        .unwrap();
        tx.commit().state
    }

    fn is_linked(state: &GameState, player: PlayerId) -> bool {
        state
            .players
            .get(player)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
    }

    fn hp_of(state: &GameState, player: PlayerId) -> i32 {
        state.players.get(player).unwrap().hp
    }

    fn submit(
        state: GameState,
        actor: PlayerId,
        kind: CommandKind,
        key: u64,
    ) -> (GameState, CommandResult) {
        let revision = state.revision;
        let (state, result) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind,
            },
        );
        (state, result.result)
    }

    fn activate(setup: &Setup, targets: Vec<PlayerId>, key: u64) -> (GameState, CommandResult) {
        submit(
            setup.state.clone(),
            setup.owner,
            CommandKind::ActivateSkill {
                skill: setup.skill,
                targets,
                card_targets: vec![],
            },
            key,
        )
    }

    #[test]
    fn package_publishes_duansuo_active_linked() {
        let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 734);
        let active = package
            .active_index
            .iter()
            .find(|active| active.skill_def_id == DUANSUO)
            .expect("duansuo active entry");
        let targets = active.targets.as_ref().unwrap();
        assert!(targets.linked);
        assert!(!targets.unlinked);
        assert_eq!(targets.min, 1);
    }

    #[test]
    fn resets_all_then_fire_damages_each_without_chain_propagation() {
        let mut setup = setup(2_026_091_301);
        let players = setup.state.players.alive_by_seat();
        let (a, b, c) = (players[1], players[2], players[3]);
        for player in [a, b, c] {
            setup.state = link(setup.state, player);
        }
        let (hp_a, hp_b, hp_c) = (
            hp_of(&setup.state, a),
            hp_of(&setup.state, b),
            hp_of(&setup.state, c),
        );

        // 只选 a、b；c 连环但未被选。
        let (state, result) = activate(&setup, vec![a, b], 1);
        assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        // 全体重置。
        assert!(!is_linked(&state, a));
        assert!(!is_linked(&state, b));
        assert!(is_linked(&state, c), "unselected player stays linked");
        // 所选各 -1 火伤；c 因重置在伤害前完成而不经链传导。
        assert_eq!(hp_of(&state, a), hp_a - 1);
        assert_eq!(hp_of(&state, b), hp_b - 1);
        assert_eq!(hp_of(&state, c), hp_c);
        // 无待决选择（原语无续体）。
        assert!(!state.has_pending_choice());
    }

    #[test]
    fn once_per_turn_and_linked_filter_enforced() {
        let mut setup = setup(2_026_091_302);
        let players = setup.state.players.alive_by_seat();
        let (a, b) = (players[1], players[2]);
        setup.state = link(setup.state, a);

        // 未连环目标被过滤器拒绝。
        let (_, result) = activate(&setup, vec![b], 11);
        assert!(matches!(result, CommandResult::Rejected { .. }));

        // 零目标（min 1）拒绝。
        let (_, result) = activate(&setup, vec![], 12);
        assert!(matches!(result, CommandResult::Rejected { .. }));

        // 合法发动一次。
        let (state, result) = activate(&setup, vec![a], 13);
        assert!(matches!(result, CommandResult::Accepted { .. }));
        assert!(!is_linked(&state, a));

        // 每回合限一次：同回合二次发动拒绝。
        let setup2 = Setup { state, ..setup };
        let (_, result) = activate(&setup2, vec![a], 14);
        assert!(matches!(result, CommandResult::Rejected { .. }));
        let _ = ChoiceId::new(0);
    }
}
