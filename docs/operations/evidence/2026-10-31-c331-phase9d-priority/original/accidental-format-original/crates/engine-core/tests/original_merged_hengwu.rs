//! O2 全量合并第二批（2026-10-14）：家族 hengwu 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_hengwu_machine {
    //! 第二百五十六批：hengwu 横骛花色计数状态机行为测试（ABI 706 / tag 729）。
    //! 锁定技：使用/打出牌后，若手牌无此花色 → 摸「场上该花色装备数」张。
    //! 测试：注入已知花色手牌 + 装备区注入同花色/异花色装备，直派 CardUsed /
    //! CardResponded 事件验证计数摸牌与门控。

    use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, Suit};
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::CardKind;
    use engine_core::state::{GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const HENGWU: u32 = 3_527_268_887;
    const HENGWU_SRC: &str = include_str!("../../../content/original/ported/xianding/hengwu.skill");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled = content_compiler::compile_source_named("hengwu-b256", HENGWU_SRC).unwrap();
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
            skill_def: SkillDefId::new(HENGWU),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.phase = engine_core::state::Phase::Play;
        (tx.commit().state, owner, other)
    }

    /// 清空手牌后注入指定花色牌，返回牌 id 列表。
    fn set_hand(state: &mut GameState, player: PlayerId, suits: &[Suit]) -> Vec<CardId> {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        let existing = state.zones.get(hand).unwrap().cards.clone();
        for card in existing {
            state
                .zones
                .get_mut(hand)
                .unwrap()
                .cards
                .retain(|c| *c != card);
            state.zones.get_mut(draw).unwrap().cards.push(card);
            if let Some(card_state) = state.cards.get_mut(card) {
                card_state.zone = draw;
                card_state.owner = PlayerId::NONE;
            }
        }
        let mut out = Vec::new();
        for suit in suits {
            let card = *state
                .zones
                .get(draw)
                .unwrap()
                .cards
                .iter()
                .rev()
                .find(|card| state.cards.get(**card).is_some_and(|c| c.suit == *suit))
                .unwrap_or_else(|| panic!("no {suit:?} card in draw pile"));
            state
                .zones
                .get_mut(draw)
                .unwrap()
                .cards
                .retain(|c| *c != card);
            state.zones.get_mut(hand).unwrap().cards.push(card);
            if let Some(card_state) = state.cards.get_mut(card) {
                card_state.zone = hand;
                card_state.owner = player;
            }
            out.push(card);
        }
        out
    }

    /// 从牌堆取一张指定花色且 kind 匹配的牌（留在原处，仅引用）。
    fn take_card(state: &GameState, suit: Suit, kind: CardKind) -> CardId {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        *state
            .zones
            .get(draw)
            .unwrap()
            .cards
            .iter()
            .rev()
            .find(|card| {
                state
                    .cards
                    .get(**card)
                    .is_some_and(|c| c.suit == suit && c.kind == kind)
            })
            .unwrap_or_else(|| panic!("no {suit:?} {kind:?} in draw pile"))
    }

    /// 从牌堆取一张指定花色的装备置入 player 装备区。
    fn add_equip_of_suit(state: &mut GameState, player: PlayerId, suit: Suit) -> CardId {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let equip = state.zones.find(ZoneKind::Equip, player).unwrap();
        let draw_cards = state.zones.get(draw).unwrap().cards.clone();
        let card = *draw_cards
            .iter()
            .rev()
            .find(|card| {
                state
                    .cards
                    .get(**card)
                    .is_some_and(|c| c.suit == suit && c.kind.equip_slot().is_some())
            })
            .unwrap_or_else(|| panic!("no {suit:?} equipment in draw pile"));
        state
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        state.zones.get_mut(equip).unwrap().cards.push(card);
        if let Some(card_state) = state.cards.get_mut(card) {
            card_state.zone = equip;
            card_state.owner = player;
        }
        card
    }

    fn hand_count(state: &GameState, player: PlayerId) -> usize {
        state.hand_cards(player).len()
    }

    /// 直派 CardUsed 事件（card_used 触发由此驱动），提交后状态返回。
    fn dispatch_card_used(state: GameState, player: PlayerId, card: CardId) -> GameState {
        let mut tx = Transaction::begin(state);
        use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
        match dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player,
                card,
                targets: Vec::new(),
            }],
            &mut TriggerFireRecord::new(),
        ) {
            Err(engine_core::effect::EffectError::ChoiceRequested) => {}
            Ok(effects) => {
                engine_core::effect::run_effects(&mut tx, effects).expect("effects apply");
            }
            Err(other) => panic!("unexpected: {other:?}"),
        }
        tx.commit().state
    }

    #[test]
    fn hengwu_card_used_counts_field_equips_of_suit() {
        // owner 手牌全红（无黑桃）：使用黑桃牌后 → 摸场上黑桃装备数（2）张。
        let (mut state, owner, other) = setup(20_260_912_111);
        let _ = set_hand(&mut state, owner, &[Suit::Heart, Suit::Diamond]);
        let _ = add_equip_of_suit(&mut state, owner, Suit::Spade);
        let _ = add_equip_of_suit(&mut state, other, Suit::Spade);
        let used = take_card(&state, Suit::Spade, CardKind::Sha);
        let hand_before = state.hand_count(owner);

        let state = dispatch_card_used(state, owner, used);
        assert_eq!(
            state.hand_count(owner),
            hand_before + 2,
            "draws per matching field equipment"
        );
        let _ = hand_count(&state, other);
    }

    #[test]
    fn hengwu_skips_when_hand_has_suit() {
        // 手牌含同花色（红桃）→ 门控拦截，不摸牌。
        let (mut state, owner, _other) = setup(20_260_912_112);
        let hand = set_hand(&mut state, owner, &[Suit::Heart, Suit::Heart]);
        let hand_before = state.hand_count(owner);
        let _ = add_equip_of_suit(&mut state, owner, Suit::Heart);
        let used = take_card(&state, Suit::Heart, CardKind::Sha);
        let _ = hand;
        let state = dispatch_card_used(state, owner, used);
        assert!(!state.has_pending_choice());
        assert_eq!(state.hand_count(owner), hand_before);
    }
}
