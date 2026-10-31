//! O2 全量合并第二批（2026-10-14）：家族 duwang 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_duwang_machine {
    //! 第二百三十九批：duwang 度望专属状态机行为测试（ABI 695 / tag 718，单发无续体）。
    //! 全链：game_started → 牌堆顶扫非【杀】取 5 张置入自身「刺」堆（跳过杀）。
    //! 债：距离修正面保守省略（上游有刺即 +1，+N 超集不建）。

    use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::GameState;
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const DUWANG: u32 = 3_777_977_845;
    const DUWANG_SRC: &str = include_str!("../../../content/original/ported/newjiang/duwang.skill");
    const NEWJIANG: &[u8] =
        include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled = content_compiler::compile_source_named("duwang-b239", DUWANG_SRC).unwrap();
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
            skill_def: SkillDefId::new(DUWANG),
        });
        run_effects(&mut tx, effects).unwrap();
        (tx.commit().state, owner, other)
    }

    #[test]
    fn duwang_contract_and_deck_scan() {
        // 契约：包内触发存在（game_started、locked 非可选）。
        let package = content_schema::ContentPackage::from_json_bytes(NEWJIANG).unwrap();
        assert!(package.manifest.rule_vm_abi_version >= 695);
        let trigger = package
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == DUWANG)
            .expect("duwang trigger");
        assert_eq!(trigger.event, "game_started");
        assert!(!trigger.optional);

        // 行为：game_started → 牌堆扫非杀取 5 置入「刺」堆。
        let (state, owner, _other) = setup(20_261_005_01);
        let mut tx = Transaction::begin(state);
        let effects = dispatch_for_events(
            &mut tx,
            &[DomainEvent::GameStarted { player_count: 2 }],
            &mut TriggerFireRecord::new(),
        )
        .expect("forced dispatch");
        engine_core::effect::run_effects(&mut tx, effects).expect("machine runs");
        let state = tx.commit().state;

        let ci = state.skill_pile_cards(owner, DUWANG);
        assert_eq!(ci.len(), 5, "five non-sha cards placed onto 刺 pile");
        // 置入的牌都不是【杀】。
        for card in &ci {
            let card_state = state.cards.get(*card).unwrap();
            assert_ne!(
                card_state.kind,
                engine_core::state::CardKind::Sha,
                "no sha in 刺 pile"
            );
        }
        // 牌堆减少（5 张入堆；洗牌等不发生——bootstrap 牌堆充足）。
    }
}
