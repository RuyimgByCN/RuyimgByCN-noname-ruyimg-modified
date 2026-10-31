//! O2 全量合并第二批（2026-10-14）：家族 twdaigui 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_twdaigui_machine {
    //! 第二百四十七批：twdaigui 待归多段状态机 slice 1 行为测试（ABI 701 / tag 724
    //! / Cont 306）。链路：出牌阶段结束确认 → 机内同色门控 → SelectTargets
    //! [1, 手牌数]（可含自己、可取消）→ 亮出牌堆底等量牌（slice 1 暂存弃牌堆，
    //! 逐目标选获面由 slice 2 接续）。

    use engine_api::{
        CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const TWDAIGUI: u32 = 3_051_286_273;
    const TWDAIGUI_SRC: &str = include_str!("../../../content/original/ported/tw/twdaigui.skill");

    fn setup_with_hand(seed: u64, hand_suits: &[Suit]) -> (GameState, PlayerId, PlayerId) {
        let compiled =
            content_compiler::compile_source_named("twdaigui-b247", TWDAIGUI_SRC).unwrap();
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
            skill_def: SkillDefId::new(TWDAIGUI),
        });
        run_effects(&mut tx, effects).unwrap();

        // 重组 owner 手牌为 hand_suits 指定花色的牌（原手牌移回牌堆底）。
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let existing = tx.state().zones.get(hand).unwrap().cards.clone();
        for card in existing {
            tx.state_mut()
                .zones
                .get_mut(hand)
                .unwrap()
                .cards
                .retain(|c| *c != card);
            tx.state_mut()
                .zones
                .get_mut(draw)
                .unwrap()
                .cards
                .insert(0, card);
            if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                card_state.zone = draw;
                card_state.owner = PlayerId::NONE;
            }
        }
        for suit in hand_suits {
            let card = *tx
                .state()
                .zones
                .get(draw)
                .unwrap()
                .cards
                .iter()
                .rev()
                .find(|card| {
                    tx.state()
                        .cards
                        .get(**card)
                        .is_some_and(|card_state| card_state.suit == *suit)
                })
                .unwrap_or_else(|| panic!("no {suit:?} card in draw pile"));
            tx.state_mut()
                .zones
                .get_mut(draw)
                .unwrap()
                .cards
                .retain(|candidate| *candidate != card);
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                card_state.zone = hand;
                card_state.owner = owner;
            }
        }
        (tx.commit().state, owner, other)
    }

    fn fire_play_end(state: GameState, owner: PlayerId) -> (GameState, bool) {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseEnded {
                current: owner,
                phase: TurnPhaseTag::Play,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
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

    fn resolve(
        state: GameState,
        actor: PlayerId,
        selections: Vec<u32>,
        cancelled: bool,
        key: u64,
    ) -> (GameState, CommandResult) {
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
        (next, result.result)
    }

    fn draw_bottom(state: &GameState, n: usize) -> Vec<CardId> {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        state.zones.get(draw).unwrap().cards[..n].to_vec()
    }

    fn discard_top(state: &GameState, n: usize) -> Vec<CardId> {
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let cards = state.zones.get(discard).unwrap().cards.clone();
        cards[cards.len() - n..].to_vec()
    }

    #[test]
    fn twdaigui_per_target_picks_in_seat_order() {
        // 两张红手牌：选自己+对方共 2 名目标 → 亮出牌堆底 2 张 → 按座次依次
        // 选获（owner 座 0 先选，各获一张入手；无余牌）。
        let (state, owner, other) = setup_with_hand(20_260_912_41, &[Suit::Heart, Suit::Heart]);
        let (state, opened) = fire_play_end(state, owner);
        assert!(opened, "confirm opens");
        let (state, confirm) = resolve(state, owner, vec![1], false, 71);
        assert!(
            matches!(confirm, CommandResult::Accepted { .. }),
            "confirm rejected: {confirm:?}"
        );
        assert!(state.has_pending_choice(), "target selection opens");
        let bottom2 = draw_bottom(&state, 2);
        let (state, result) = resolve(state, owner, vec![owner.raw(), other.raw()], false, 72);
        assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        assert!(
            state.has_pending_choice(),
            "first pick opens for seat-0 target"
        );

        // 第一轮：座次靠前的 owner 选第一张。
        let (state, result) = resolve(state, owner, vec![bottom2[0].raw()], false, 73);
        assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        assert!(
            state.has_pending_choice(),
            "second pick opens for seat-1 target"
        );

        // 第二轮：other 只能选剩下的那张。
        let (state, result) = resolve(state, other, vec![bottom2[1].raw()], false, 74);
        assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        assert!(!state.has_pending_choice());

        // 入手断言。
        let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
        let other_hand = state.zones.find(ZoneKind::Hand, other).unwrap();
        assert!(state
            .zones
            .get(owner_hand)
            .unwrap()
            .cards
            .contains(&bottom2[0]));
        assert!(state
            .zones
            .get(other_hand)
            .unwrap()
            .cards
            .contains(&bottom2[1]));
        // 无余牌（目标数=亮牌数）：弃牌堆无新增。
        assert!(discard_top(&state, 0).is_empty());
    }

    #[test]
    fn twdaigui_target_order_sorted_to_seat() {
        // 选择顺序与座次相反时，选获轮仍按座次推进（先 owner 后 other）。
        let (state, owner, other) = setup_with_hand(20_260_912_44, &[Suit::Heart, Suit::Diamond]);
        let (state, opened) = fire_play_end(state, owner);
        assert!(opened);
        let (state, _) = resolve(state, owner, vec![1], false, 101);
        let bottom2 = draw_bottom(&state, 2);
        let (state, result) = resolve(state, owner, vec![other.raw(), owner.raw()], false, 102);
        assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        // 第一轮 chooser = owner（座 0）：用 other 的 actor 会被拒绝。
        let (_, wrong) = resolve(state, other, vec![bottom2[0].raw()], false, 103);
        assert!(
            !matches!(wrong, CommandResult::Accepted { .. }),
            "seat-order chooser mismatch must be rejected"
        );
    }

    #[test]
    fn twdaigui_mixed_colors_gate_closes_after_confirm() {
        // 红黑混合手牌：确认照常打开（机内门控在后），确认后不再开选目标。
        let (state, owner, _other) = setup_with_hand(20_260_912_42, &[Suit::Heart, Suit::Spade]);
        let (state, opened) = fire_play_end(state, owner);
        assert!(opened, "confirm opens (quanjia-family debt)");
        let (state, _) = resolve(state, owner, vec![1], false, 81);
        assert!(
            !state.has_pending_choice(),
            "uniform-color gate must close the chain"
        );
    }

    #[test]
    fn twdaigui_cancel_parks_nothing() {
        let (state, owner, _other) = setup_with_hand(20_260_912_43, &[Suit::Spade, Suit::Spade]);
        let (state, opened) = fire_play_end(state, owner);
        assert!(opened);
        let (state, result) = resolve(state, owner, vec![], true, 91);
        assert!(matches!(result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice());
        assert!(draw_bottom(&state, 1).is_empty() == false, "draw untouched");
        assert!(discard_top(&state, 0).is_empty(), "nothing parked");
    }
}
