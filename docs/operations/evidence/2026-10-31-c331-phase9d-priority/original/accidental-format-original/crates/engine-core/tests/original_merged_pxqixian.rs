//! O2 全量合并第二批（2026-10-14）：家族 pxqixian 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_pxqixian_machine {
    //! 第二百三十六批：pxqixian 祁县专属状态机行为测试（ABI 693 / tag 716 / Cont 301）。
    //! 全链：active 发动 → 牌种双选（杀/拆）→ 重铸素材选择（装备牌）→ 目标选择 →
    //! 素材置入弃牌堆 + 物化虚拟牌使用。债：使用合法性宽口径、canRecast 校验。

    use engine_api::{
        CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const PXQIXIAN: u32 = 2_340_360_569;
    const PXQIXIAN_SRC: &str = include_str!("../../../content/original/ported/sp/pxqixian.skill");

    fn inject_equip_card(
        state: GameState,
        owner: PlayerId,
        kind: CardKind,
        def: u32,
    ) -> (GameState, CardId) {
        let mut tx = Transaction::begin(state);
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(def),
            kind,
            suit: Suit::Spade,
            rank: 5,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        (tx.commit().state, card)
    }

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled =
            content_compiler::compile_source_named("pxqixian-b236", PXQIXIAN_SRC).unwrap();
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
            skill_def: SkillDefId::new(PXQIXIAN),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().turn.phase = engine_core::state::Phase::Play;
        (tx.commit().state, owner, other)
    }

    fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
        state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == PXQIXIAN)
            .unwrap()
            .id
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
    fn pxqixian_no_equip_silent() {
        // 无装备牌 → 发动沉默（校验在机内，uses_per_turn 消耗一次为预期）。
        let (state, owner, _other) = setup(20_261_003_01);
        let sid = skill_id(&state, owner);
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
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice(), "no equip → silent");
    }

    #[test]
    fn pxqixian_full_chain_recast_and_virtual_use() {
        let (state, owner, other) = setup(20_261_003_02);

        // 注入装备牌 → 全链：牌种双选（拆）→ 素材 → 目标 → 素材入弃 + 虚拟拆使用。
        let (state, material) = inject_equip_card(state, owner, CardKind::Armor, 9_970);
        let sid = skill_id(&state, owner);
        let revision = state.revision;
        let (state, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ActivateSkill {
                    skill: sid,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        eprintln!(
            "DBG activate: {:?} pending={}",
            result.result,
            state.has_pending_choice()
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "activate 2 rejected"
        );
        assert!(state.has_pending_choice(), "kind choice opens");
        let state = resolve(state, owner, vec![2], 11); // 过河拆桥
        assert!(state.has_pending_choice(), "material selection opens");
        let state = resolve(state, owner, vec![material.raw()], 12);
        assert!(state.has_pending_choice(), "target selection opens");
        let state = resolve(state, owner, vec![other.raw()], 13);
        // 过河拆桥结算：选择弃置目标一张牌（owner 的后续选择）。
        assert!(state.has_pending_choice(), "guohe discard selection opens");
        let victim_card = state.hand_cards(other).first().copied().unwrap();
        let state = resolve(state, owner, vec![victim_card.raw()], 14);
        assert!(!state.has_pending_choice(), "chain completes");

        // 素材入弃牌堆。
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert!(
            state.zones.get(discard).unwrap().cards.contains(&material),
            "recast material in discard pile"
        );
    }
}
