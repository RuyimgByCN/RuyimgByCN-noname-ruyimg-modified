//! O2 全量合并第二批（2026-10-14）：家族 yuheng 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_yuheng_dependencies {
    //! Golden coverage for the first cross-package skills granted by EXTRA「驭衡」.

    use engine_api::{
        CardMoveReason, ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{turn_flag, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const HONGDE: u32 = 3_138_095_197;
    const REZHIHENG: u32 = 2_455_708_599;
    const GZYINGHUN: u32 = 2_790_748_306;
    const BIZHENG: u32 = 3_859_813_014;
    const XIASHU: u32 = 2_873_874_515;
    const GUANCHAO: u32 = 3_040_931_439;
    const GUANWEI: u32 = 3_285_976_176;
    const LANJIANG: u32 = 3_473_961_755;
    const REBINGYI: u32 = 2_238_473_770;
    const REBOTU: u32 = 2_231_312_450;
    const OLDIMENG: u32 = 2_509_094_453;
    const REJIEYIN: u32 = 3_295_441_821;
    const REANXU: u32 = 2_398_589_569;
    const DRLT_JUEYAN: u32 = 3_834_102_895;
    const DRLT_JUEYAN_SHA: u32 = 2_863_335_974;
    const DRLT_JUEYAN1: u32 = 3_194_432_315;
    const DRLT_JUEYAN3: u32 = 4_232_768_410;
    const DRLT_JUEYAN_REJIZHI: u32 = 2_811_682_656;
    const JIEXUN: u32 = 2_662_349_862;
    const FUNAN_JIEXUN: u32 = 4_079_819_056;
    const WENGUA: u32 = 4_068_014_439;
    const WENGUA2: u32 = 2_716_743_244;
    const WENGUA3: u32 = 4_005_759_674;
    const XINANGUO: u32 = 3_075_552_814;
    const SOURCE: &str = concat!(
        include_str!("../../../content/original/ported/sp/hongde.skill"),
        include_str!("../../../content/original/ported/sp/bizheng.skill"),
        include_str!("../../../content/original/ported/sp/xiashu.skill"),
        include_str!("../../../content/original/ported/sp/xinfu_guanchao.skill"),
        include_str!("../../../content/original/ported/sp/xinfu_guanwei.skill"),
        include_str!("../../../content/original/ported/sp/lanjiang.skill"),
        include_str!("../../../content/original/ported/refresh/rebingyi.skill"),
        include_str!("../../../content/original/ported/refresh/rebotu.skill"),
        include_str!("../../../content/original/ported/refresh/oldimeng.skill"),
        include_str!("../../../content/original/ported/refresh/rejieyin.skill"),
        include_str!("../../../content/original/ported/refresh/rezhiheng.skill"),
        include_str!("../../../content/original/ported/shenhua/gzyinghun.skill"),
        include_str!("../../../content/original/ported/mobile/reanxu.skill"),
        include_str!("../../../content/original/ported/shenhua/drlt_jueyan.skill"),
        include_str!("../../../content/original/ported/shenhua/drlt_jueyan1.skill"),
        include_str!("../../../content/original/ported/shenhua/drlt_jueyan2.skill"),
        include_str!("../../../content/original/ported/shenhua/drlt_jueyan3.skill"),
        include_str!("../../../content/original/ported/shenhua/drlt_jueyan_rejizhi.skill"),
        include_str!("../../../content/original/ported/yijiang/jiexun.skill"),
        include_str!("../../../content/original/ported/yijiang/wengua.skill"),
        include_str!("../../../content/original/ported/yijiang/xinanguo.skill"),
    );

    fn setup(skill_def: u32) -> (GameState, PlayerId, SkillId) {
        let package =
            content_compiler::compile_source_named("yuheng-dependencies-test", SOURCE).unwrap();
        let mut state = GameState::empty();
        std::sync::Arc::make_mut(&mut state.content)
            .load_package(&package)
            .unwrap();
        let mut tx = Transaction::begin(state);
        bootstrap_match(&mut tx, 3, 241, "identity-standard").unwrap();
        let owner = tx.state().players.alive_by_seat()[0];
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        });
        run_effects(&mut tx, effects).unwrap();
        let skill = tx
            .state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
            .unwrap();
        (tx.commit().state, owner, skill)
    }

    fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
        let revision = state.revision;
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let (state, result) = apply_command(
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
            !matches!(result.result, engine_api::CommandResult::Rejected { .. }),
            "choice key {key}: {:?}",
            result.result
        );
        state
    }

    #[test]
    fn hongde_requires_one_gain_or_loss_event_of_at_least_two_cards() {
        let (state, owner, _) = setup(HONGDE);
        let hand = state.hand_cards(owner);
        let from = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let to = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();

        let mut one = Transaction::begin(state.clone());
        assert!(dispatch_for_events(
            &mut one,
            &[DomainEvent::CardsMoved {
                cards: hand[..1].to_vec(),
                from,
                to,
                reason: CardMoveReason::Discard,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap()
        .is_empty());

        let mut lost_two = Transaction::begin(state.clone());
        assert!(matches!(
            dispatch_for_events(
                &mut lost_two,
                &[DomainEvent::CardsMoved {
                    cards: hand[..2].to_vec(),
                    from,
                    to,
                    reason: CardMoveReason::Discard,
                }],
                &mut TriggerFireRecord::new(),
            ),
            Err(EffectError::ChoiceRequested)
        ));
        assert!(lost_two.state().has_pending_choice());

        let mut gained_two = Transaction::begin(state);
        assert!(matches!(
            dispatch_for_events(
                &mut gained_two,
                &[DomainEvent::CardsDrawn {
                    player: owner,
                    cards: hand[..2].to_vec(),
                }],
                &mut TriggerFireRecord::new(),
            ),
            Err(EffectError::ChoiceRequested)
        ));
    }

    #[test]
    fn rezhiheng_draws_one_extra_only_when_all_hand_cards_were_selected() {
        let (state, owner, skill) = setup(REZHIHENG);
        let hand = state.hand_cards(owner);

        let mut all = Transaction::begin(state.clone());
        run_effects(
            &mut all,
            vec![EffectRequest::DiscardSelectedDrawAllHandBonus {
                player: owner,
                cards: hand.clone(),
                skill,
            }],
        )
        .unwrap();
        assert_eq!(all.state().hand_count(owner), hand.len() as u32 + 1);

        let mut partial = Transaction::begin(state);
        run_effects(
            &mut partial,
            vec![EffectRequest::DiscardSelectedDrawAllHandBonus {
                player: owner,
                cards: hand[..1].to_vec(),
                skill,
            }],
        )
        .unwrap();
        assert_eq!(partial.state().hand_count(owner), hand.len() as u32);
    }

    #[test]
    fn gzyinghun_preserves_both_lost_hp_draw_discard_branches() {
        for (branch, expected_delta) in [(1_u32, 1_i32), (2_u32, -1_i32)] {
            let (mut state, owner, skill) = setup(GZYINGHUN);
            let target = state
                .players
                .alive_by_seat()
                .into_iter()
                .find(|target| *target != owner)
                .unwrap();
            state.players.get_mut(owner).unwrap().hp -= 2;
            let before = state.hand_count(target) as i32;
            let mut tx = Transaction::begin(state);
            assert!(matches!(
                run_effects(
                    &mut tx,
                    vec![EffectRequest::ChooseOtherYinghun {
                        player: owner,
                        target: PlayerId::NONE,
                        count: 0,
                        branch: 0,
                        ready_to_discard: false,
                        skill,
                    }],
                ),
                Err(EffectError::ChoiceRequested)
            ));
            let state = choose(
                tx.commit().state,
                owner,
                vec![target.raw()],
                10 + u64::from(branch),
            );
            let state = choose(state, owner, vec![branch], 20 + u64::from(branch));
            let discard: Vec<_> = state
                .hand_cards(target)
                .into_iter()
                .take(branch as usize)
                .map(engine_api::CardId::raw)
                .collect();
            let state = choose(state, target, discard, 30 + u64::from(branch));
            assert_eq!(state.hand_count(target) as i32, before + expected_delta);
        }
    }

    #[test]
    fn bizheng_draws_first_then_checks_and_discards_both_players_in_order() {
        let (mut state, owner, skill) = setup(BIZHENG);
        let target = state
            .players
            .alive_by_seat()
            .into_iter()
            .find(|target| *target != owner)
            .unwrap();
        for player in [owner, target] {
            state.players.get_mut(player).unwrap().max_hp = 2;
            state.players.get_mut(player).unwrap().hp = 2;
        }
        let owner_before = state.hand_count(owner);
        let target_before = state.hand_count(target);
        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ChooseOtherBizheng {
                    player: owner,
                    target: PlayerId::NONE,
                    stage: 0,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![target.raw()], 40);
        let owner_discard: Vec<_> = state
            .hand_cards(owner)
            .into_iter()
            .take(2)
            .map(engine_api::CardId::raw)
            .collect();
        let state = choose(state, owner, owner_discard, 41);
        let target_discard: Vec<_> = state
            .hand_cards(target)
            .into_iter()
            .take(2)
            .map(engine_api::CardId::raw)
            .collect();
        let state = choose(state, target, target_discard, 42);
        assert_eq!(state.hand_count(owner), owner_before - 2);
        assert_eq!(state.hand_count(target), target_before);
    }

    #[test]
    fn xiashu_gives_all_hand_then_gains_exactly_the_chosen_partition() {
        for branch in [1_u32, 2_u32] {
            let (state, owner, skill) = setup(XIASHU);
            let target = state
                .players
                .alive_by_seat()
                .into_iter()
                .find(|target| *target != owner)
                .unwrap();
            let owner_before = state.hand_count(owner);
            let target_before = state.hand_count(target);
            let mut tx = Transaction::begin(state);
            assert!(matches!(
                run_effects(
                    &mut tx,
                    vec![EffectRequest::ChooseOtherXiashu {
                        player: owner,
                        target: PlayerId::NONE,
                        stage: 0,
                        cards: Vec::new(),
                        skill,
                    }],
                ),
                Err(EffectError::ChoiceRequested)
            ));
            let state = choose(
                tx.commit().state,
                owner,
                vec![target.raw()],
                50 + u64::from(branch),
            );
            assert_eq!(state.hand_count(owner), 0);
            assert_eq!(
                state.hand_count(target),
                owner_before.saturating_add(target_before)
            );
            let revealed: Vec<_> = state
                .hand_cards(target)
                .into_iter()
                .take(3)
                .map(engine_api::CardId::raw)
                .collect();
            let state = choose(state, target, revealed.clone(), 60 + u64::from(branch));
            let state = choose(state, owner, vec![branch], 70 + u64::from(branch));
            let expected = if branch == 1 {
                revealed.len() as u32
            } else {
                owner_before + target_before - revealed.len() as u32
            };
            assert_eq!(state.hand_count(owner), expected);
            assert_eq!(
                state.hand_count(target),
                owner_before + target_before - expected
            );
        }
    }

    #[test]
    fn guanchao_draws_for_strict_progression_and_stays_invalid_after_one_violation() {
        let (state, owner, skill) = setup(GUANCHAO);
        let card = state.hand_cards(owner)[0];
        let mut tx = Transaction::begin(state);
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        tx.state_mut().cards.get_mut(card).unwrap().rank = 3;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(GUANCHAO),
                delta: 1,
            }],
        )
        .unwrap();

        let before = tx.state().hand_count(owner);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_PLAY_CARD_META, 2 << 8);
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveGuanchaoRank {
                player: owner,
                card,
                skill,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), before + 1);

        tx.state_mut().cards.get_mut(card).unwrap().rank = 1;
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_PLAY_CARD_META, 3 << 8);
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveGuanchaoRank {
                player: owner,
                card,
                skill,
            }],
        )
        .unwrap();
        let mark = tx
            .state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == GUANCHAO)
            .map(|(_, count)| *count)
            .unwrap_or(0);
        assert_eq!(mark, 3);

        tx.state_mut().cards.get_mut(card).unwrap().rank = 5;
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_PLAY_CARD_META, 1 << 8);
        let invalid_before = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveGuanchaoRank {
                player: owner,
                card,
                skill,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), invalid_before);
    }

    #[test]
    fn guanwei_discards_then_draws_and_inserts_an_immediate_play_phase() {
        let (mut state, owner, skill) = setup(GUANWEI);
        let target = state
            .players
            .alive_by_seat()
            .into_iter()
            .find(|target| *target != owner)
            .unwrap();
        state.turn.current = target;
        state.turn.phase = Phase::Discard;
        let discard = state.hand_cards(owner)[0];
        let owner_before = state.hand_count(owner);
        let target_before = state.hand_count(target);

        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveGuanwei {
                    player: owner,
                    target,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![discard.raw()], 80);
        assert_eq!(state.hand_count(owner), owner_before - 1);
        assert_eq!(state.hand_count(target), target_before + 2);
        assert_eq!(state.turn.current, target);
        assert_eq!(state.turn.phase, Phase::Play);
    }

    #[test]
    fn lanjiang_keeps_entry_snapshot_and_orders_draw_damage_then_forced_draw() {
        let (state, owner, skill) = setup(LANJIANG);
        let others: Vec<_> = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != owner)
            .collect();
        let equal_after_owner_draw = others[0];
        let lower_after_owner_draw = others[1];
        let mut tx = Transaction::begin(state);
        let owner_before = tx.state().hand_count(owner);
        let equal_need = owner_before
            .saturating_add(1)
            .saturating_sub(tx.state().hand_count(equal_after_owner_draw));
        let lower_need = owner_before.saturating_sub(tx.state().hand_count(lower_after_owner_draw));
        run_effects(
            &mut tx,
            vec![
                EffectRequest::DrawCards {
                    player: equal_after_owner_draw,
                    count: equal_need,
                },
                EffectRequest::DrawCards {
                    player: lower_after_owner_draw,
                    count: lower_need,
                },
            ],
        )
        .unwrap();
        let lower_before = tx.state().hand_count(lower_after_owner_draw);
        let hp_before = tx.state().players.get(equal_after_owner_draw).unwrap().hp;
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveLanjiang {
                    player: owner,
                    stage: 0,
                    index: 0,
                    target: PlayerId::NONE,
                    targets: Vec::new(),
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));

        let state = choose(tx.commit().state, owner, vec![1], 90);
        let state = choose(state, equal_after_owner_draw, vec![0], 91);
        let state = choose(state, lower_after_owner_draw, vec![0], 92);
        let state = choose(state, owner, vec![equal_after_owner_draw.raw()], 93);
        let state = choose(state, owner, vec![lower_after_owner_draw.raw()], 94);

        assert_eq!(state.hand_count(owner), owner_before + 1);
        assert_eq!(
            state.players.get(equal_after_owner_draw).unwrap().hp,
            hp_before - 1
        );
        assert_eq!(state.hand_count(lower_after_owner_draw), lower_before + 1);
    }

    #[test]
    fn rebingyi_reveals_uniform_hand_draws_chosen_players_and_adds_same_rank_self_draw() {
        let (mut state, owner, skill) = setup(REBINGYI);
        let targets: Vec<_> = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != owner)
            .collect();
        for card in state.hand_cards(owner) {
            let card = state.cards.get_mut(card).unwrap();
            card.suit = Suit::Heart;
            card.rank = 7;
        }
        let owner_before = state.hand_count(owner);
        let target_before: Vec<_> = targets
            .iter()
            .map(|target| state.hand_count(*target))
            .collect();
        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveRebingyi {
                    player: owner,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(
            tx.commit().state,
            owner,
            targets.iter().map(|target| target.raw()).collect(),
            100,
        );
        assert_eq!(state.hand_count(owner), owner_before + 1);
        for (target, before) in targets.into_iter().zip(target_before) {
            assert_eq!(state.hand_count(target), before + 1);
        }
    }

    #[test]
    fn rebotu_tracks_four_discard_suits_and_caps_extra_turns_per_round() {
        let (mut state, owner, _) = setup(REBOTU);
        let cards = state.hand_cards(owner);
        for (card, suit) in
            cards
                .iter()
                .take(4)
                .zip([Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond])
        {
            state.cards.get_mut(*card).unwrap().suit = suit;
        }
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let mut tx = Transaction::begin(state);
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardsMoved {
                cards: cards[..4].to_vec(),
                from: hand,
                to: discard,
                reason: CardMoveReason::Discard,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap();
        assert_eq!(
            tx.state().turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK),
            0b1111
        );
        assert!(matches!(
            dispatch_for_events(
                &mut tx,
                &[DomainEvent::TurnStateChanged {
                    current: owner,
                    phase: TurnPhaseTag::End,
                }],
                &mut TriggerFireRecord::new(),
            ),
            Err(EffectError::ChoiceRequested)
        ));

        let (state, owner, skill) = setup(REBOTU);
        let mut tx = Transaction::begin(state);
        for _ in 0..4 {
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveRebotu {
                    player: owner,
                    skill,
                }],
            )
            .unwrap();
        }
        assert_eq!(tx.state().turn.extra_turn_queue, vec![owner, owner, owner]);
    }

    #[test]
    fn oldimeng_swaps_both_hands_atomically_then_discards_the_play_end_difference() {
        let (state, owner, skill) = setup(OLDIMENG);
        let targets: Vec<_> = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != owner)
            .collect();
        let first = targets[0];
        let second = targets[1];
        let mut first_cards = state.hand_cards(first);
        let mut second_cards = state.hand_cards(second);
        first_cards.sort_by_key(|card| card.raw());
        second_cards.sort_by_key(|card| card.raw());
        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveOldimeng {
                    player: owner,
                    stage: 0,
                    first: PlayerId::NONE,
                    second: PlayerId::NONE,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![first.raw()], 110);
        let mut state = choose(state, owner, vec![second.raw()], 111);
        let mut current_first = state.hand_cards(first);
        let mut current_second = state.hand_cards(second);
        current_first.sort_by_key(|card| card.raw());
        current_second.sort_by_key(|card| card.raw());
        assert_eq!(current_first, second_cards);
        assert_eq!(current_second, first_cards);

        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: first,
                count: 1,
            }],
        )
        .unwrap();
        let owner_before = tx.state().hand_count(owner);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveOldimeng {
                    player: owner,
                    stage: 2,
                    first: PlayerId::NONE,
                    second: PlayerId::NONE,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let discard = tx.state().hand_cards(owner)[0];
        state = choose(tx.commit().state, owner, vec![discard.raw()], 112);
        assert_eq!(state.hand_count(owner), owner_before - 1);
    }

    #[test]
    fn rejieyin_discards_non_equipment_then_draws_high_hp_and_recovers_low_hp() {
        let (mut state, owner, skill) = setup(REJIEYIN);
        let target = state
            .players
            .alive_by_seat()
            .into_iter()
            .find(|target| {
                *target != owner
                    && state
                        .players
                        .get(*target)
                        .and_then(|target| state.content.character(target.character_def))
                        .is_some_and(|character| {
                            matches!(character.sex.as_str(), "male" | "double")
                        })
            })
            .unwrap();
        state.players.get_mut(owner).unwrap().hp = 4;
        state.players.get_mut(target).unwrap().hp = 2;
        state.players.get_mut(target).unwrap().max_hp = 4;
        let card = state.hand_cards(owner)[0];
        state.cards.get_mut(card).unwrap().kind = CardKind::Sha;
        let owner_before = state.hand_count(owner);
        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveRejieyin {
                    player: owner,
                    target: PlayerId::NONE,
                    stage: 0,
                    card,
                    equip: false,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![target.raw()], 120);
        assert_eq!(state.hand_count(owner), owner_before);
        assert_eq!(state.players.get(target).unwrap().hp, 3);
    }

    #[test]
    fn reanxu_transfers_first_targets_hand_card_then_runs_both_draw_checks() {
        let (state, owner, skill) = setup(REANXU);
        let targets: Vec<_> = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != owner)
            .collect();
        let first = targets[0];
        let second = targets[1];
        let card = state.hand_cards(first)[0];
        let owner_before = state.hand_count(owner);
        let first_before = state.hand_count(first);
        let second_before = state.hand_count(second);
        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveReanxu {
                    player: owner,
                    stage: 0,
                    first: PlayerId::NONE,
                    second: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    from_hand: false,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![first.raw()], 130);
        let state = choose(state, owner, vec![second.raw()], 131);
        let state = choose(state, second, vec![card.raw()], 132);
        let state = choose(state, owner, vec![1], 133);
        assert_eq!(state.cards.get(card).unwrap().owner, second);
        assert_eq!(state.hand_count(owner), owner_before + 1);
        assert_eq!(state.hand_count(first), first_before);
        assert_eq!(state.hand_count(second), second_before + 1);
    }

    #[test]
    fn drlt_jueyan_makes_first_branch_permanent_and_later_branch_temporary() {
        let (state, owner, skill) = setup(DRLT_JUEYAN);
        let before = state.hand_count(owner);
        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveDrltJueyan {
                    player: owner,
                    stage: 0,
                    slot: 0,
                    first: false,
                    sha_skill_def: SkillDefId::new(DRLT_JUEYAN_SHA),
                    weapon_skill_def: SkillDefId::new(DRLT_JUEYAN1),
                    mount_skill_def: SkillDefId::new(3_629_461_497),
                    armor_skill_def: SkillDefId::new(DRLT_JUEYAN3),
                    treasure_skill_def: SkillDefId::new(DRLT_JUEYAN_REJIZHI),
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![2], 140);
        assert_eq!(state.hand_count(owner), before + 3);
        assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[1], 1);
        let armor = state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                state
                    .skills
                    .get(*skill)
                    .filter(|instance| instance.def.raw() == DRLT_JUEYAN3)
            })
            .unwrap();
        assert!(armor.temporary_until_turn_end_of.is_none());

        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveDrltJueyan {
                    player: owner,
                    stage: 0,
                    slot: 0,
                    first: false,
                    sha_skill_def: SkillDefId::new(DRLT_JUEYAN_SHA),
                    weapon_skill_def: SkillDefId::new(DRLT_JUEYAN1),
                    mount_skill_def: SkillDefId::new(3_629_461_497),
                    armor_skill_def: SkillDefId::new(DRLT_JUEYAN3),
                    treasure_skill_def: SkillDefId::new(DRLT_JUEYAN_REJIZHI),
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![1], 141);
        let weapon = state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                state
                    .skills
                    .get(*skill)
                    .filter(|instance| instance.def.raw() == DRLT_JUEYAN1)
            })
            .unwrap();
        assert!(weapon.temporary_until_turn_end_of.is_some());
        assert!(!state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == DRLT_JUEYAN_SHA)
            }));
    }

    #[test]
    fn rejizhi_can_discard_the_drawn_basic_card_for_one_turn_hand_limit_mark() {
        let (mut state, owner, skill) = setup(DRLT_JUEYAN_REJIZHI);
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let card = *state.zones.get(draw).unwrap().cards.last().unwrap();
        state.cards.get_mut(card).unwrap().kind = CardKind::Sha;
        let before = state.hand_count(owner);
        let mut tx = Transaction::begin(state);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveRejizhi {
                    player: owner,
                    stage: 0,
                    card: engine_api::CardId::NONE,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![1], 150);
        assert_eq!(state.hand_count(owner), before);
        assert_eq!(
            state
                .players
                .get(owner)
                .unwrap()
                .marks
                .iter()
                .find(|(mark, _)| *mark == DRLT_JUEYAN_REJIZHI)
                .map(|(_, count)| *count),
            Some(1)
        );
    }

    #[test]
    fn jiexun_uses_pre_increment_mark_and_replaces_itself_after_target_discards_all() {
        let (state, owner, skill) = setup(JIEXUN);
        let target = state
            .players
            .alive_by_seat()
            .into_iter()
            .find(|target| *target != owner)
            .unwrap();
        let hand = state.hand_cards(target);
        let from = state.zones.find(ZoneKind::Hand, target).unwrap();
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![
                EffectRequest::MoveCards {
                    cards: hand[..hand.len() - 1].to_vec(),
                    to: discard,
                    from: Some(from),
                },
                EffectRequest::AddMark {
                    player: owner,
                    mark: engine_api::MarkDefId::new(JIEXUN),
                    delta: 1,
                },
            ],
        )
        .unwrap();
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveJiexun {
                    player: owner,
                    stage: 0,
                    target: PlayerId::NONE,
                    draw_count: 0,
                    discard_count: 0,
                    funan_helper_skill_def: SkillDefId::new(FUNAN_JIEXUN),
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![target.raw()], 160);
        assert_eq!(state.hand_count(target), 0);
        assert!(!state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == JIEXUN)
            }));
        assert!(state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == FUNAN_JIEXUN)
            }));
        assert!(!state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, _)| *mark == JIEXUN));
    }

    #[test]
    fn wengua_grants_global_helper_and_bottom_branch_draws_from_the_top() {
        let (state, owner, skill) = setup(WENGUA);
        let actor = state
            .players
            .alive_by_seat()
            .into_iter()
            .find(|target| *target != owner)
            .unwrap();
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveWenguaSetup {
                player: owner,
                helper_skill_def: SkillDefId::new(WENGUA2),
                skill,
            }],
        )
        .unwrap();
        let helper = tx
            .state()
            .players
            .get(actor)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == WENGUA2)
            .unwrap();
        let card = tx.state().hand_cards(actor)[0];
        let actor_before = tx.state().hand_count(actor);
        let owner_before = tx.state().hand_count(owner);
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveWengua {
                    player: actor,
                    stage: 0,
                    target: PlayerId::NONE,
                    card,
                    branch: 0,
                    main_skill_def: SkillDefId::new(WENGUA),
                    marker_skill_def: SkillDefId::new(WENGUA3),
                    skill: helper,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let state = choose(tx.commit().state, owner, vec![1], 170);
        assert_eq!(state.hand_count(actor), actor_before);
        assert_eq!(state.hand_count(owner), owner_before + 1);
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.zones.get(draw).unwrap().cards.first(), Some(&card));
        assert!(state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == WENGUA3)
            }));
    }

    #[test]
    fn xinanguo_runs_target_minimum_actions_then_only_the_owners_remaining_actions() {
        let (mut state, owner, skill) = setup(XINANGUO);
        let others: Vec<_> = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != owner)
            .collect();
        let target = others[0];
        let third = others[1];
        state.players.get_mut(owner).unwrap().hp = 2;
        state.players.get_mut(owner).unwrap().max_hp = 4;
        state.players.get_mut(target).unwrap().hp = 4;
        state.players.get_mut(target).unwrap().max_hp = 4;
        state.players.get_mut(third).unwrap().hp = 3;
        state.players.get_mut(third).unwrap().max_hp = 4;
        let cards = state.hand_cards(target);
        state.cards.get_mut(cards[0]).unwrap().kind = CardKind::Weapon;
        let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
        let target_equip = state.zones.find(ZoneKind::Equip, target).unwrap();
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![
                EffectRequest::MoveCards {
                    cards: vec![cards[0]],
                    to: target_equip,
                    from: Some(target_hand),
                },
                EffectRequest::MoveCards {
                    cards: cards[1..3].to_vec(),
                    to: discard,
                    from: Some(target_hand),
                },
            ],
        )
        .unwrap();
        let target_before = tx.state().hand_count(target);
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveXinanguo {
                player: owner,
                target,
                stage: 0,
                index: 0,
                remaining: 0b111,
                skill,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(target), target_before + 1);
        assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
        assert_eq!(
            tx.state()
                .zones
                .get(tx.state().zones.find(ZoneKind::Equip, owner).unwrap())
                .unwrap()
                .cards
                .len(),
            1
        );
        assert_eq!(tx.state().zones.get(target_equip).unwrap().cards.len(), 1);
    }
}
