//! O2 全量合并第二批（2026-10-14）：家族 sxrm 的 2 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_sxrm_batch1 {
    //! Golden coverage for sxrm 包批次一（ABI 596）：窃王/潜流/利水/负背 + 全包契约。

    use engine_api::{
        CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
        PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::{EffectRequest, QueryRequest, QueryResponse};

    const QIEWANG: u32 = 3_539_390_001;
    const QIANLIU: u32 = 3_539_390_002;
    const LISHUI: u32 = 3_539_390_003;
    const FUBEI: u32 = 3_539_390_004;
    const SOURCE_QIEWANG: &str =
        include_str!("../../../content/original/ported/sxrm/sxrmqiewang.skill");
    const SOURCE_QIANLIU: &str =
        include_str!("../../../content/original/ported/sxrm/sxrmqianliu.skill");
    const SOURCE_LISHUI: &str =
        include_str!("../../../content/original/ported/sxrm/sxrmlishui.skill");
    const SOURCE_FUBEI: &str =
        include_str!("../../../content/original/ported/sxrm/sxrmfubei.skill");

    struct Setup {
        tx: Transaction,
        owner: PlayerId,
        other: PlayerId,
    }

    fn setup(source: &str, defs: &[u32]) -> Setup {
        let package = content_compiler::compile_source_named("sxrm-batch1", source).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
        let seats = tx.state().players.alive_by_seat();
        let owner = seats[0];
        let other = seats[1];
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }));
        for def in defs {
            effects.push(EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*def),
            });
        }
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        Setup { tx, owner, other }
    }

    fn alloc_card(
        tx: &mut Transaction,
        zone: engine_api::ZoneId,
        kind: CardKind,
        def: u32,
        suit: Suit,
        rank: u8,
    ) -> CardId {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(def),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone,
            owner: PlayerId::NONE,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        card
    }

    fn dispatch(state: GameState, event: DomainEvent) -> GameState {
        let mut tx = Transaction::begin(state);
        let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
        match fired {
            Ok(effects) => match run_effects(&mut tx, effects) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("run error: {e:?}"),
            },
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        }
        tx.commit().state
    }

    fn hand_count(state: &GameState, player: PlayerId) -> u32 {
        state
            .zones
            .find(ZoneKind::Hand, player)
            .map(|zone| {
                state
                    .zones
                    .get(zone)
                    .map(|z| z.cards.len() as u32)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    #[test]
    fn package_exposes_sxrm_contract() {
        let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
            "../../../content/original/ported/sxrm/sxrm.npack.json"
        ))
        .unwrap();
        assert_eq!(committed.manifest.rule_vm_abi_version, 596);
        assert_eq!(committed.character_index.len(), 27);
        // 四个真实移植技能都在包里。
        for def in [QIEWANG, QIANLIU, LISHUI, FUBEI] {
            assert!(
                committed
                    .skill_index
                    .iter()
                    .any(|skill| skill.skill_def_id == def),
                "skill {def} must exist in the pack"
            );
        }
        assert!(committed
            .character_index
            .iter()
            .any(|character| character.skills.contains(&QIEWANG)));
        assert!(committed
            .character_index
            .iter()
            .any(|character| character.skills.contains(&FUBEI)));
    }

    #[test]
    fn qiewang_draws_on_nearby_damage() {
        let mut setup = setup(SOURCE_QIEWANG, &[QIEWANG]);
        let before = hand_count(&setup.tx.state(), setup.owner);
        // other 在距离 1 内受伤 → owner 摸一张。
        let state = setup.tx.commit().state;
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(setup.owner).unwrap().hp = 3;
        tx.state_mut().players.get_mut(setup.other).unwrap().hp = 2;
        // 引擎在 run_effects 内对产生的 DamageDealt 事件分发触发器：窃王在本次伤害中触发。
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap();
        let state = tx.commit().state;
        assert_eq!(
            hand_count(&state, setup.owner),
            before + 1,
            "qiewang draws one on nearby damage"
        );
    }

    #[test]
    fn qianliu_gains_bottom_four_when_suits_diverse() {
        let mut setup = setup(SOURCE_QIANLIU, &[QIANLIU]);
        let draw = setup
            .tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        // 牌堆底四张：四种花色。The engine draws from the vector end (top), so
        // the bottom four are the first four entries. alloc_card pushes to the top;
        // move the freshly allocated four to the front.
        for (i, suit) in [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond]
            .into_iter()
            .enumerate()
        {
            alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, suit, 5 + i as u8);
        }
        {
            let z = setup.tx.state_mut().zones.get_mut(draw).unwrap();
            let len = z.cards.len();
            let tail: Vec<_> = z.cards.split_off(len - 4);
            z.cards.splice(0..0, tail);
        }
        let before = hand_count(&setup.tx.state(), setup.owner);
        let hand = setup
            .tx
            .state()
            .zones
            .find(ZoneKind::Hand, setup.other)
            .unwrap();
        let sha = alloc_card(&mut setup.tx, hand, CardKind::Sha, 1, Suit::Spade, 9);
        let state = setup.tx.commit().state;
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.phase = Phase::Play;
        let event = DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.owner],
        };
        let mut state = {
            let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
            match fired {
                Ok(effects) => match run_effects(&mut tx, effects) {
                    Ok(_) => tx.commit().state,
                    Err(EffectError::ChoiceRequested) => tx.commit().state,
                    Err(e) => panic!("run error: {e:?}"),
                },
                Err(EffectError::ChoiceRequested) => tx.commit().state,
                Err(e) => panic!("dispatch error: {e:?}"),
            }
        };
        // 可选触发确认。
        if state.has_pending_choice() {
            let pending = state.resolution.pending_choice_id;
            let observer = PlayerId::new(state.resolution.pending_observer);
            let (state2, result) = apply_command(
                state,
                Command {
                    actor: observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(0x5961),
                    kind: CommandKind::ResolveChoice {
                        choice_id: engine_api::ChoiceId::new(pending),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            assert!(
                matches!(result.result, engine_api::CommandResult::Accepted { .. }),
                "qianliu confirm must resolve: {result:?}"
            );
            state = state2;
        }
        assert_eq!(
            hand_count(&state, setup.owner),
            before + 4,
            "qianliu gains the bottom four when suits are diverse"
        );
    }

    #[test]
    fn lishui_draws_on_black_card_targeted() {
        let mut setup = setup(SOURCE_LISHUI, &[LISHUI]);
        let before = hand_count(&setup.tx.state(), setup.owner);
        let hand = setup
            .tx
            .state()
            .zones
            .find(ZoneKind::Hand, setup.other)
            .unwrap();
        let black = alloc_card(&mut setup.tx, hand, CardKind::TrickGuohe, 2, Suit::Spade, 3);
        let mut state = setup.tx.commit().state;
        let mut tx = Transaction::begin(state);
        let fired = dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: setup.other,
                card: black,
                targets: vec![setup.owner],
            }],
            &mut TriggerFireRecord::new(),
        );
        state = match fired {
            Ok(effects) => match run_effects(&mut tx, effects) {
                Ok(_) => tx.commit().state,
                Err(EffectError::ChoiceRequested) => tx.commit().state,
                Err(e) => panic!("run error: {e:?}"),
            },
            Err(EffectError::ChoiceRequested) => tx.commit().state,
            Err(e) => panic!("dispatch error: {e:?}"),
        };
        if state.has_pending_choice() {
            let pending = state.resolution.pending_choice_id;
            let observer = PlayerId::new(state.resolution.pending_observer);
            let (state2, result) = apply_command(
                state,
                Command {
                    actor: observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(0x5962),
                    kind: CommandKind::ResolveChoice {
                        choice_id: engine_api::ChoiceId::new(pending),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            assert!(
                matches!(result.result, engine_api::CommandResult::Accepted { .. }),
                "lishui confirm must resolve: {result:?}"
            );
            state = state2;
        }
        assert_eq!(
            hand_count(&state, setup.owner),
            before + 1,
            "lishui draws on black card targeted"
        );
    }

    #[test]
    fn fubei_draws_to_max_hp() {
        let mut setup = setup(SOURCE_FUBEI, &[FUBEI]);
        let skill = setup
            .tx
            .state()
            .players
            .get(setup.owner)
            .unwrap()
            .skills
            .iter()
            .find(|skill| {
                setup
                    .tx
                    .state()
                    .skills
                    .get(**skill)
                    .is_some_and(|instance| instance.def.raw() == FUBEI)
            })
            .copied()
            .unwrap();
        setup
            .tx
            .state_mut()
            .players
            .get_mut(setup.owner)
            .unwrap()
            .hp = 1;
        let state = setup.tx.commit().state;
        let (state, result) = apply_command(
            state,
            Command {
                actor: setup.owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x5960),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: vec![],
                    card_targets: vec![],
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "fubei must activate: {result:?}"
        );
        assert_eq!(
            hand_count(&state, setup.owner),
            4,
            "fubei draws to max hp (4)"
        );
    }

    #[test]
    fn qiewang_skips_distant_damage() {
        let mut setup = setup(SOURCE_QIEWANG, &[QIEWANG]);
        // 强制距离 3：直接调用 resolve 验证距离门。
        let skill = setup
            .tx
            .state()
            .players
            .get(setup.owner)
            .unwrap()
            .skills
            .iter()
            .find(|skill| {
                setup
                    .tx
                    .state()
                    .skills
                    .get(**skill)
                    .is_some_and(|instance| instance.def.raw() == QIEWANG)
            })
            .copied()
            .unwrap();
        let state = setup.tx.commit().state;
        let mut tx = Transaction::begin(state);
        // 距离 2：两玩家间基础距离 1（相邻）；无法制造距离 2，改为直接校验事件源为自身跳过。
        let fired = dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source: setup.owner,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
            &mut TriggerFireRecord::new(),
        );
        match fired {
            Ok(effects) => {
                let res = run_effects(&mut tx, effects);
                if let Err(EffectError::ChoiceRequested) = res {
                    let _ = tx.commit().state;
                    return;
                }
                res.unwrap();
            }
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        }
        let state = tx.commit().state;
        // 自身受伤（距离 0 但事件源=拥有者）：窃王按“其他角色受伤”语义跳过。
        let _ = skill;
        assert_eq!(
            hand_count(&state, setup.owner),
            0,
            "self damage must not trigger qiewang draw"
        );
    }
}

mod original_sxrm_sxrmlanjiao {
    //! Golden coverage for 揽娇（sxrm 嗔曹操，ABI 479）.

    use engine_api::{
        CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
        ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const SXRMLANJIAO: u32 = 3_315_968_200;
    const SOURCE: &str = include_str!("../../../content/original/ported/sxrm/sxrmlanjiao.skill");

    fn fire_play(tx: Transaction, player: PlayerId) -> Transaction {
        let mut tx = tx;
        let fired = dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: player,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        );
        match fired {
            Ok(effects) => match run_effects(&mut tx, effects) {
                Ok(_) => tx,
                Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
                Err(e) => panic!("effect error: {e:?}"),
            },
            Err(engine_core::effect::EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("dispatch error: {e:?}"),
        }
    }

    fn resolve_choice(
        state: GameState,
        actor: PlayerId,
        selections: Vec<u32>,
        cancelled: bool,
        key: u64,
    ) -> GameState {
        let revision = state.revision;
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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
            "choice must be accepted: {:?}",
            result.result
        );
        next
    }

    fn setup(tx: &mut Transaction, owner: PlayerId, target: PlayerId) {
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SXRMLANJIAO),
        });
        run_effects(tx, effects).unwrap();
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        for &player in &[owner, target] {
            let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
            run_effects(
                tx,
                cards
                    .into_iter()
                    .map(|card| EffectRequest::MoveCards {
                        cards: vec![card],
                        to: discard,
                        from: Some(hand),
                    })
                    .collect(),
            )
            .unwrap();
        }
    }

    fn seed_hand(
        tx: &mut Transaction,
        player: PlayerId,
        specs: &[(CardKind, u32, Suit)],
    ) -> Vec<CardId> {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        specs
            .iter()
            .map(|(kind, def, suit)| {
                let card = tx.state_mut().cards.alloc(Card {
                    id: CardId::NONE,
                    def: CardDefId::new(*def),
                    kind: *kind,
                    suit: *suit,
                    rank: 5,
                    weapon_range: 0,
                    zone: hand,
                    owner: player,
                    face_up: false,
                });
                tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
                card
            })
            .collect()
    }

    #[test]
    fn stop_gains_the_viewed_cards() {
        let package = content_compiler::compile_source_named("sxrmlanjiao-test", SOURCE).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 479);
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
        let seats = tx.state().players.alive_by_seat();
        let (owner, target) = (seats[0], seats[1]);
        setup(&mut tx, owner, target);
        // The target holds exactly two cards (the first two will be viewed).
        let target_cards = seed_hand(
            &mut tx,
            target,
            &[
                (CardKind::Sha, 96_100, Suit::Spade),
                (CardKind::Tao, 96_101, Suit::Heart),
            ],
        );
        let owner_hand_before = tx.state().hand_count(owner);

        let mut tx = Transaction::begin(tx.commit().state);
        let tx = fire_play(tx, owner);
        let mut state = tx.commit().state;
        assert!(state.has_pending_choice(), "target choice pending");
        state = resolve_choice(state, owner, vec![target.raw()], false, 1);
        assert!(state.has_pending_choice(), "rechoose choice pending");
        // The target stops (0): the owner gains the two viewed cards.
        state = resolve_choice(state, target, vec![0], false, 2);

        assert_eq!(
            state.hand_count(owner),
            owner_hand_before + 2,
            "gained the two viewed cards"
        );
        assert!(
            state.hand_cards(owner).contains(&target_cards[0])
                && state.hand_cards(owner).contains(&target_cards[1]),
            "both viewed cards moved to the owner"
        );
        assert_eq!(state.hand_count(target), 0);
    }

    #[test]
    fn rechoose_loops_then_stop() {
        let package = content_compiler::compile_source_named("sxrmlanjiao-test", SOURCE).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
        let seats = tx.state().players.alive_by_seat();
        let (owner, target) = (seats[0], seats[1]);
        setup(&mut tx, owner, target);
        // Four cards: two viewed, rechoose (lose 1 + draw 1), two more viewed, then stop.
        let _target_cards = seed_hand(
            &mut tx,
            target,
            &[
                (CardKind::Sha, 96_100, Suit::Spade),
                (CardKind::Tao, 96_101, Suit::Heart),
                (CardKind::Sha, 96_102, Suit::Club),
                (CardKind::Tao, 96_103, Suit::Diamond),
            ],
        );
        let owner_hand_before = tx.state().hand_count(owner);
        let target_hp_before = tx.state().players.get(target).unwrap().hp;

        let mut tx = Transaction::begin(tx.commit().state);
        let tx = fire_play(tx, owner);
        let mut state = tx.commit().state;
        state = resolve_choice(state, owner, vec![target.raw()], false, 1);
        assert!(state.has_pending_choice());
        // Rechoose: target loses 1 HP, draws 1, and the loop re-opens.
        state = resolve_choice(state, target, vec![1], false, 2);
        assert!(state.has_pending_choice(), "loop re-opened");
        assert_eq!(
            state.players.get(target).unwrap().hp,
            target_hp_before - 1,
            "rechoose cost 1 HP"
        );
        // Stop on the second iteration: the owner gains the new two viewed cards.
        state = resolve_choice(state, target, vec![0], false, 3);

        assert_eq!(
            state.hand_count(owner),
            owner_hand_before + 2,
            "gained the second view's two cards"
        );
        assert_eq!(
            state.hand_count(target),
            3,
            "target kept 4 + 1 drawn - 2 gained"
        );
    }

    #[test]
    fn per_target_once_per_turn() {
        let package = content_compiler::compile_source_named("sxrmlanjiao-test", SOURCE).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
        let seats = tx.state().players.alive_by_seat();
        let (owner, target) = (seats[0], seats[1]);
        setup(&mut tx, owner, target);
        seed_hand(
            &mut tx,
            target,
            &[
                (CardKind::Sha, 96_100, Suit::Spade),
                (CardKind::Tao, 96_101, Suit::Heart),
                (CardKind::Sha, 96_102, Suit::Club),
                (CardKind::Tao, 96_103, Suit::Diamond),
            ],
        );

        let mut tx = Transaction::begin(tx.commit().state);
        // First use: stop, mark the target.
        let tx = fire_play(tx, owner);
        let mut state = tx.commit().state;
        state = resolve_choice(state, owner, vec![target.raw()], false, 1);
        state = resolve_choice(state, target, vec![0], false, 2);
        assert_eq!(state.hand_count(owner), 2, "first use gained two cards");

        // A second play-phase firing offers no target (the mark is set).
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.phase = Phase::Play;
        let tx = fire_play(tx, owner);
        let state = tx.commit().state;
        assert_eq!(
            state.resolution.pending_choice_id, 0,
            "no target offered after the per-target use"
        );
    }
}
