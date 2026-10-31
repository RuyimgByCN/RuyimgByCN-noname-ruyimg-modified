//! O2 全量合并第二批（2026-10-14）：家族 round 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_round_history_primitive {
    //! Coverage for the round-history foundation primitive: `cards_used_this_round` query, backed
    //! by `Turn::round_card_use_count`. Foundation for upstream `player.getRoundHistory("useCard")`
    //! used by clanyuzhi / clandingan / clanxsyingxiang.

    use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
    use engine_core::query::{query, QueryContext};
    use engine_core::transaction::Transaction;
    use engine_core::Engine;
    use rule_api::{QueryRequest, QueryResponse};

    const SOURCE: &str = concat!(
        "skill round_hist_probe(",
        "3222222222",
        ") locked {\n",
        "  trigger card_used always {\n",
        "    effect storage_set(self_skill, 1, cards_used_this_round(self));\n",
        "  }\n",
        "}\n",
    );

    #[test]
    fn cards_used_this_round_compiles_and_reports_round_scoped_count() {
        let pkg = content_compiler::compile_source(SOURCE).unwrap();
        // The query is referenced from DSL and the package compiles.
        assert!(pkg.skill_index.iter().any(|s| s.name == "round_hist_probe"));

        let mut engine = Engine::new();
        engine.load_package(&pkg).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_809,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine.state().turn.current;

        // Initially the round-scoped counter is zero.
        let tx = Transaction::begin(engine.state().clone());
        let q0 = query(
            tx.state(),
            &QueryRequest::CardsUsedThisRound { player: owner },
            &mut QueryContext::new(),
        );
        assert!(matches!(q0, QueryResponse::Int(0)));

        // Record three proactive card uses this round via the Turn primitive.
        let mut tx = tx;
        for _ in 0..3 {
            tx.state_mut().turn.record_round_card_use(owner);
        }
        let q1 = query(
            tx.state(),
            &QueryRequest::CardsUsedThisRound { player: owner },
            &mut QueryContext::new(),
        );
        assert!(matches!(q1, QueryResponse::Int(3)));

        // Round rollover clears the counter.
        tx.state_mut().turn.clear_round_card_use_counts();
        let q2 = query(
            tx.state(),
            &QueryRequest::CardsUsedThisRound { player: owner },
            &mut QueryContext::new(),
        );
        assert!(matches!(q2, QueryResponse::Int(0)));
    }
}
