//! O2 全量合并第二批（2026-10-14）：家族 yangming 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_yangming_machine {
    //! 第二百五十三批：jsrgyangming 养名循环拼点状态机行为测试（ABI 704 / tag 727
    //! / Cont 309）。链路：ActivateSkill → 选可拼点目标 → 循环【owner 选拼点牌 →
    //! 目标选拼点牌 → 比点】：目标赢 = 目标摸「已输次数」张 + owner 回复 1 结束；
    //! 否则计输 + owner 选择是否重复（不可再拼点自动结束）。拼点牌入弃牌堆。
    //! 测试用已知点数牌注入双手，保证比点结果确定。

    use engine_api::{
        CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
        PlayerId, SkillDefId,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const YANGMING: u32 = 2_355_602_394;
    const YANGMING_SRC: &str =
        include_str!("../../../content/original/ported/jsrg/jsrgyangming.skill");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled =
            content_compiler::compile_source_named("yangming-b253", YANGMING_SRC).unwrap();
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
            skill_def: SkillDefId::new(YANGMING),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.phase = engine_core::state::Phase::Play;
        (tx.commit().state, owner, target)
    }

    /// 清空 player 手牌（移回牌堆底），注入指定点数的牌各一张，按序返回。
    fn set_hand(state: &mut GameState, player: PlayerId, ranks: &[u8]) -> Vec<CardId> {
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
        for rank in ranks {
            let draw_cards = state.zones.get(draw).unwrap().cards.clone();
            let card = *draw_cards
                .iter()
                .rev()
                .find(|card| state.cards.get(**card).is_some_and(|c| c.rank == *rank))
                .unwrap_or_else(|| panic!("no rank {rank} card in draw pile"));
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

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        state.zones.get(hand).unwrap().cards.clone()
    }

    fn rank_of(state: &GameState, card: CardId) -> u8 {
        state.cards.get(card).map(|c| c.rank).unwrap_or(0)
    }

    fn hp_of(state: &GameState, player: PlayerId) -> i32 {
        state.players.get(player).unwrap().hp
    }

    fn discard_has(state: &GameState, card: CardId) -> bool {
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        state.zones.get(discard).unwrap().cards.contains(&card)
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

    fn activate(state: GameState, owner: PlayerId) -> GameState {
        let sid = state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.def.raw() == YANGMING)
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

    #[test]
    fn yangming_target_win_draws_losses_owner_recovers() {
        // owner=[7,5] hp 压 1；target=[6,11]。r1: 7 vs 6 owner 赢（计输 1）→
        // 重复；r2: 5 vs 11 目标赢 → 目标摸 1 张 + owner 回 1，结束。
        let (mut state, owner, target) = setup(20_260_912_91);
        {
            let hp = state.players.get(owner).unwrap().hp;
            state.players.get_mut(owner).unwrap().hp = (hp - 1).max(1);
        }
        let owner_start_hp = hp_of(&state, owner);
        let o_cards = set_hand(&mut state, owner, &[7, 5]);
        let target_cards = set_hand(&mut state, target, &[6, 11]);
        let state = state;

        let state = activate(state, owner);
        assert!(state.has_pending_choice(), "target selection opens");
        let state = resolve(state, owner, vec![target.raw()], 11);

        // r1: owner 7 vs target 6 → owner 赢，计输 1。
        let state = resolve(state, owner, vec![o_cards[0].raw()], 12);
        let state = resolve(state, target, vec![target_cards[0].raw()], 13);

        // 知情：重复。
        let state = resolve(state, owner, vec![1], 14);

        // r2: owner 5 vs target 11 → 目标赢 → 目标摸 1 张 + owner 回 1。
        let state = resolve(state, owner, vec![o_cards[1].raw()], 15);
        let state = resolve(state, target, vec![target_cards[1].raw()], 16);
        assert!(!state.has_pending_choice());

        assert_eq!(hp_of(&state, owner), owner_start_hp + 1, "owner recovers 1");
        // 目标手牌：6、11 均入弃牌堆，摸 1 张 → 1 张。
        assert_eq!(state.hand_count(target), 1);
        assert!(discard_has(&state, target_cards[0]));
        assert!(discard_has(&state, target_cards[1]));
    }

    #[test]
    fn yangming_decline_repeat_ends_chain() {
        // owner=[9,3] target=[5,10]。r1: 9 vs 5 owner 赢 → 计输 1 → 选择结束；
        // 目标不摸、owner 不回（目标赢才回）。
        let (mut state, owner, target) = setup(20_260_912_92);
        let hp_owner = hp_of(&state, owner);
        let o_cards = set_hand(&mut state, owner, &[9, 3]);
        let t_cards = set_hand(&mut state, target, &[5, 10]);
        let state = state;

        let state = activate(state, owner);
        let state = resolve(state, owner, vec![target.raw()], 21);

        // r1: owner 9 vs target 5 → owner 赢。
        let state = resolve(state, owner, vec![o_cards[0].raw()], 22);
        let state = resolve(state, target, vec![t_cards[0].raw()], 23);
        // 知情：结束（2）。
        let state = resolve(state, owner, vec![2], 24);
        assert!(!state.has_pending_choice());
        assert_eq!(hp_of(&state, owner), hp_owner);
    }

    #[test]
    fn yangming_auto_ends_when_owner_hand_exhausted() {
        // owner=[9] target=[5,10]：r1 owner 赢计输 1 → 重复但 owner 手牌已空
        // （canCompare 不满足）→ 自动结束。
        let (mut state, owner, target) = setup(20_260_912_93);
        let o_cards = set_hand(&mut state, owner, &[9]);
        let t_cards = set_hand(&mut state, target, &[5, 10]);
        let state = state;

        let state = activate(state, owner);
        let state = resolve(state, owner, vec![target.raw()], 41);
        let state = resolve(state, owner, vec![o_cards[0].raw()], 42);
        let state = resolve(state, target, vec![t_cards[0].raw()], 43);
        // owner 手牌已空（canCompare 不满足）→ 机器在知情前自动结束。
        assert!(!state.has_pending_choice());
    }
}
