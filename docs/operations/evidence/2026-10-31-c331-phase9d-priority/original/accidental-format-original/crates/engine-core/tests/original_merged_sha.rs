//! O2 全量合并第二批（2026-10-14）：家族 sha 的 2 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_sha_hand_limit {
    //! Golden coverage for authoritative hand-limit modifiers and protected Sha cards.

    use engine_api::{
        CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
        SkillDefId, Suit,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::run_effects;
    use engine_core::state::{Card, CardKind, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::Engine;
    use rule_api::EffectRequest;

    const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
    const REFRESH: &[u8] =
        include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
    const YIJIANG: &[u8] =
        include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
    const CHANGANDAJIAN_EQUIP5: u32 = 3_727_522_753;
    const XINXIANZHEN3: u32 = 3_077_071_581;
    const NEW_JIANGCHI3: u32 = 3_349_403_784;
    const CANGZHUO2: u32 = 3_573_995_787;
    const QIAOSHUI4: u32 = 3_521_041_512;

    fn inject(
        tx: &mut Transaction,
        owner: PlayerId,
        kind: CardKind,
        rank: u8,
    ) -> engine_api::CardId {
        inject_with_suit(tx, owner, kind, Suit::Heart, rank)
    }

    fn inject_with_suit(
        tx: &mut Transaction,
        owner: PlayerId,
        kind: CardKind,
        suit: Suit,
        rank: u8,
    ) -> engine_api::CardId {
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = Card {
            id: engine_api::CardId::NONE,
            def: CardDefId::new(9_900 + kind.tag() as u32 + rank as u32),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        };
        let card = tx.state_mut().cards.alloc(card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        card
    }

    #[test]
    fn generic_card_selectors_compose_for_real_discard_options() {
        const KEEP_TRICKS: u32 = 9_910_001;
        const KEEP_HEARTS: u32 = 9_910_002;
        let package = content_compiler::compile_source_named(
            "selector-test",
            r#"
skill keep_tricks(9910001) {
  modifier card_ignored_hand_limit(trick) override 1;
}
skill keep_hearts(9910002) {
  modifier card_ignored_hand_limit(heart) override 1;
}
"#,
        )
        .unwrap();
        let bytes = package.to_json_bytes().unwrap();
        let (mut tx, owner) = discard_state(&bytes, 20_260_783);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 0;

        let trick = inject_with_suit(&mut tx, owner, CardKind::TrickGuohe, Suit::Spade, 1);
        let heart = inject_with_suit(&mut tx, owner, CardKind::Sha, Suit::Heart, 2);
        let ordinary = inject_with_suit(&mut tx, owner, CardKind::Other, Suit::Club, 3);
        run_effects(
            &mut tx,
            vec![
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(KEEP_TRICKS),
                },
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(KEEP_HEARTS),
                },
            ],
        )
        .unwrap();

        assert!(engine_core::query::card_ignored_for_hand_limit(
            tx.state(),
            owner,
            trick
        ));
        assert!(engine_core::query::card_ignored_for_hand_limit(
            tx.state(),
            owner,
            heart
        ));
        assert!(!engine_core::query::card_ignored_for_hand_limit(
            tx.state(),
            owner,
            ordinary
        ));
        assert_eq!(
            engine_core::query::discard_to_limit_tokens(tx.state(), owner),
            vec![ordinary]
        );

        let (_, pipe) = end_discard(tx.state().clone(), owner);
        let choice = pipe.choice.expect("one ordinary card must be discarded");
        assert_eq!(choice.options.len(), 1);
        assert_eq!(choice.options[0].token, ordinary.raw());
    }

    #[test]
    fn migrated_trick_selectors_protect_normal_and_delayed_tricks() {
        for (package, skill_def, seed) in [
            (REFRESH, CANGZHUO2, 20_260_784),
            (YIJIANG, QIAOSHUI4, 20_260_785),
        ] {
            let (mut tx, owner) = discard_state(package, seed);
            tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
            let normal = inject_with_suit(&mut tx, owner, CardKind::TrickGuohe, Suit::Spade, 1);
            let delayed = inject_with_suit(&mut tx, owner, CardKind::TrickLe, Suit::Diamond, 2);
            let ordinary = inject_with_suit(&mut tx, owner, CardKind::Sha, Suit::Club, 3);
            run_effects(
                &mut tx,
                vec![EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(skill_def),
                }],
            )
            .unwrap();

            assert!(engine_core::query::card_ignored_for_hand_limit(
                tx.state(),
                owner,
                normal
            ));
            assert!(engine_core::query::card_ignored_for_hand_limit(
                tx.state(),
                owner,
                delayed
            ));
            assert_eq!(
                engine_core::query::discard_to_limit_tokens(tx.state(), owner),
                vec![ordinary]
            );
            let (_, pipe) = end_discard(tx.state().clone(), owner);
            let choice = pipe.choice.expect("only the ordinary card is discardable");
            assert_eq!(choice.options.len(), 1);
            assert_eq!(choice.options[0].token, ordinary.raw());
        }
    }

    fn discard_state(package: &[u8], seed: u64) -> (Transaction, PlayerId) {
        let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
        let mut engine = Engine::new();
        engine.load_package(&package).unwrap();
        assert!(matches!(
            engine.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::StartGame {
                    player_count: 2,
                    rng_seed: seed,
                    ruleset: "identity-standard".into(),
                },
            }),
            CommandResult::Accepted { .. }
        ));
        let owner = engine.state().turn.current;
        let mut tx = Transaction::begin(engine.state().clone());
        tx.state_mut().revision = 1;
        tx.state_mut().turn.phase = Phase::Discard;
        tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
        let skills = tx.state().players.get(owner).unwrap().skills.clone();
        for skill in skills {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let cards = tx.state().hand_cards(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            }],
        )
        .unwrap();
        (tx, owner)
    }

    fn end_discard(
        state: engine_core::state::GameState,
        owner: PlayerId,
    ) -> (
        engine_core::state::GameState,
        engine_core::command::PipelineResult,
    ) {
        apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::EndPhase,
            },
        )
    }

    #[test]
    fn ordinary_hand_limit_modifier_controls_the_real_discard_phase() {
        let (mut tx, owner) = discard_state(EXTRA, 20_260_780);
        for rank in 1..=4 {
            inject(&mut tx, owner, CardKind::Other, rank);
        }
        let (_, ordinary) = end_discard(tx.state().clone(), owner);
        assert!(ordinary.choice.is_some(), "four cards over a limit of two");

        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(CHANGANDAJIAN_EQUIP5),
            }],
        )
        .unwrap();
        let (modified, modified_pipe) = end_discard(tx.state().clone(), owner);
        assert!(matches!(
            modified_pipe.result,
            CommandResult::Accepted { .. }
        ));
        assert!(modified_pipe.choice.is_none());
        assert_eq!(modified.turn.phase, Phase::End);
    }

    #[test]
    fn protected_sha_neither_counts_nor_can_be_selected_for_phase_discard() {
        let (mut tx, owner) = discard_state(YIJIANG, 20_260_781);
        let sha1 = inject(&mut tx, owner, CardKind::Sha, 1);
        let sha2 = inject(&mut tx, owner, CardKind::ShaFire, 2);
        let keep = [sha1.raw(), sha2.raw()];
        for rank in 3..=5 {
            inject(&mut tx, owner, CardKind::Other, rank);
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(XINXIANZHEN3),
            }],
        )
        .unwrap();

        assert_eq!(
            engine_core::query::hand_count_for_limit(tx.state(), owner),
            3
        );
        let (state, pipe) = end_discard(tx.state().clone(), owner);
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        let choice = pipe.choice.unwrap();
        assert_eq!(choice.min_count, 1);
        assert_eq!(choice.max_count, 1);
        assert!(choice
            .options
            .iter()
            .all(|option| !keep.contains(&option.token)));

        let (_, rejected) = apply_command(
            state.clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![sha1.raw()],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

        let valid = choice.options[0].token;
        let (_, accepted) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(3),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![valid],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    }

    #[test]
    fn standard_new_jiangchi3_combines_all_three_sha_restrictions() {
        let mut engine = Engine::new();
        assert!(matches!(
            engine.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::StartGame {
                    player_count: 2,
                    rng_seed: 20_260_782,
                    ruleset: "identity-standard".into(),
                },
            }),
            CommandResult::Accepted { .. }
        ));
        let owner = engine.state().turn.current;
        let mut tx = Transaction::begin(engine.state().clone());
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NEW_JIANGCHI3),
            }],
        )
        .unwrap();

        assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 0);
        assert!(engine_core::query::sha_response_cancelled_for(
            tx.state(),
            owner
        ));
        assert!(engine_core::query::sha_ignored_for_hand_limit(
            tx.state(),
            owner
        ));
    }
}

mod original_sha_response_cancel {
    //! Golden for YIJIANG `jiangchi3`: a held Sha cannot answer Nanman.

    use engine_api::{
        CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
        SkillDefId, Suit,
    };
    use engine_core::command::{apply_command, PipelineResult};
    use engine_core::effect::run_effects;
    use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::Engine;
    use rule_api::EffectRequest;

    const YIJIANG: &[u8] =
        include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
    const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
    const JIANGCHI3: u32 = 3_940_214_578;
    const XIYING2: u32 = 3_555_033_084;

    fn inject(
        tx: &mut Transaction,
        owner: PlayerId,
        kind: CardKind,
        rank: u8,
    ) -> engine_api::CardId {
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = Card {
            id: engine_api::CardId::NONE,
            def: CardDefId::new(9_800 + kind.tag() as u32 + rank as u32),
            kind,
            suit: Suit::Spade,
            rank,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        };
        let card = tx.state_mut().cards.alloc(card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        card
    }

    fn pass_wuxie(mut state: GameState, mut pipe: PipelineResult) -> (GameState, PipelineResult) {
        let mut key = 10;
        while state.has_pending_choice()
            && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
        {
            let choice = pipe.choice.clone().expect("wuxie choice");
            let result = apply_command(
                state,
                Command {
                    actor: choice.observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: choice.choice_id,
                        selections: Vec::new(),
                        cancelled: true,
                    },
                },
            );
            state = result.0;
            pipe = result.1;
            key += 1;
        }
        (state, pipe)
    }

    fn play_nanman(
        package: &[u8],
        blocked_skill: Option<u32>,
    ) -> (GameState, PipelineResult, PlayerId, u32, i32) {
        let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
        let mut engine = Engine::new();
        engine.load_package(&package).unwrap();
        assert!(matches!(
            engine.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::StartGame {
                    player_count: 2,
                    rng_seed: 20_260_770,
                    ruleset: "identity-standard".into(),
                },
            }),
            CommandResult::Accepted { .. }
        ));
        let actor = engine.state().turn.current;
        let target = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != actor)
            .unwrap()
            .id;
        let mut tx = Transaction::begin(engine.state().clone());
        tx.state_mut().revision = 1;
        tx.state_mut().turn.phase = Phase::Play;
        // This Golden isolates the injected response modifier from whichever
        // character skills the growing package roster assigned at StartGame.
        let initial_skills: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| skill.id)
            .collect();
        run_effects(
            &mut tx,
            initial_skills
                .into_iter()
                .map(|skill| EffectRequest::RemoveSkill { skill })
                .collect(),
        )
        .unwrap();
        if let Some(skill) = blocked_skill {
            run_effects(
                &mut tx,
                vec![EffectRequest::AddSkill {
                    player: target,
                    skill_def: SkillDefId::new(skill),
                }],
            )
            .unwrap();
        }
        let sha = inject(&mut tx, target, CardKind::Sha, 6);
        let nanman = inject(&mut tx, actor, CardKind::TrickNanman, 7);
        let hp = tx.state().players.get(target).unwrap().hp;
        let (state, pipe) = apply_command(
            tx.state().clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::PlayCard {
                    card: nanman,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        let (state, pipe) = pass_wuxie(state, pipe);
        (state, pipe, target, sha.raw(), hp)
    }

    #[test]
    fn jiangchi3_removes_held_sha_from_nanman_response_options() {
        let (ordinary, ordinary_pipe, target, sha, hp) = play_nanman(YIJIANG, None);
        assert_eq!(
            ordinary.resolution.continuation_kind,
            ContinuationKind::RespondAoeQueue
        );
        assert_eq!(ordinary.players.get(target).unwrap().hp, hp);
        assert!(ordinary_pipe
            .choice
            .unwrap()
            .options
            .iter()
            .any(|option| option.token == sha));

        let (blocked, blocked_pipe, target, _, hp) = play_nanman(YIJIANG, Some(JIANGCHI3));
        assert!(blocked_pipe.choice.is_none());
        assert_eq!(blocked.players.get(target).unwrap().hp, hp - 1);
    }

    #[test]
    fn xiying2_removes_all_card_response_options() {
        let (blocked, blocked_pipe, target, _, hp) = play_nanman(SP2, Some(XIYING2));
        assert!(blocked_pipe.choice.is_none());
        assert_eq!(blocked.players.get(target).unwrap().hp, hp - 1);
    }
}
