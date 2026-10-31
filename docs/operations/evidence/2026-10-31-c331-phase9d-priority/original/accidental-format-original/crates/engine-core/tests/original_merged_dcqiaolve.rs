//! O2 全量合并第二批（2026-10-14）：家族 dcqiaolve 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_dcqiaolve_machine {
    //! 第二百八十六批：dcqiaolve 巧略弃牌代价补全行为测试（桥 ABI 732 /
    //! tag 753 / Cont 327）。链路：用牌指定目标后 → owner 选成本牌（强制）→
    //! 弃置 → 取事件目标一张随机手牌。

    use engine_api::{
        CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::{apply_command, PipelineResult};
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const DCQIAOLVE: u32 = 3_810_555_923;
    const DCQIAOLVE_SRC: &str =
        include_str!("../../../content/original/ported/newjiang/dcqiaolve.skill");
    const NEWJIANG: &[u8] =
        include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json");

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        state.hand_cards(player)
    }

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let package = content_schema::ContentPackage::from_json_bytes(NEWJIANG).unwrap();
        let compiled =
            content_compiler::compile_source_named("dcqiaolve-test", DCQIAOLVE_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
        let players = tx.state().players.alive_by_seat();
        let (owner, target) = (players[0], players[1]);
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.extend(
            [owner, target]
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        );
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCQIAOLVE),
        });
        run_effects(&mut tx, effects).unwrap();
        for player in [owner, target] {
            let state = tx.state_mut().players.get_mut(player).unwrap();
            state.hp = 10;
            state.max_hp = 10;
        }
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        (tx.commit().state, owner, target)
    }

    fn inject_hand(
        state: &GameState,
        player: PlayerId,
        kind: CardKind,
        suit: Suit,
        rank: u8,
    ) -> (GameState, CardId) {
        let mut tx = Transaction::begin(state.clone());
        let zone = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(99_100 + u32::from(rank)),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone,
            owner: player,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        let state = tx.commit().state;
        (state, card)
    }

    fn submit(
        state: GameState,
        actor: PlayerId,
        kind: CommandKind,
        key: u64,
    ) -> (GameState, PipelineResult) {
        let revision = state.revision;
        apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind,
            },
        )
    }

    fn play_trick_at(
        state: &GameState,
        source: PlayerId,
        card: CardId,
        targets: Vec<PlayerId>,
        key: u64,
    ) -> (GameState, PipelineResult) {
        submit(
            state.clone(),
            source,
            CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
            key,
        )
    }

    #[test]
    fn package_publishes_dcqiaolve_cost_machine() {
        let package = content_schema::ContentPackage::from_json_bytes(NEWJIANG).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 732);
        let trigger = package
            .trigger_index
            .iter()
            .find(|trigger| trigger.skill_def_id == DCQIAOLVE)
            .expect("dcqiaolve trigger");
        assert_eq!(trigger.event, "card_targeted");
        assert_eq!(trigger.filter, "owner_is_source");
    }

    #[test]
    fn discard_cost_paid_then_takes_target_hand_card() {
        // owner 使用一张【杀】指定 target（单目标即满足 card_targeted）。
        let (state, owner, target) = setup(20_260_915_101);
        let (state, trick) = inject_hand(&state, owner, CardKind::Other, Suit::Heart, 3);
        let (state, t_card) = inject_hand(&state, target, CardKind::Other, Suit::Spade, 8);
        let (state, cost) = inject_hand(&state, owner, CardKind::Other, Suit::Club, 4);
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();

        let (state, confirm) = play_trick_at(&state, owner, trick, vec![target], 1);
        let confirm = confirm.choice.expect("optional confirm opens");
        assert_eq!(confirm.kind, ChoiceKind::Confirm);
        let (state, cost_choice) = submit(
            state,
            owner,
            CommandKind::ResolveChoice {
                choice_id: confirm.choice_id,
                selections: vec![1],
                cancelled: false,
            },
            2,
        );
        let cost_choice = cost_choice.choice.expect("cost choice opens");
        assert_eq!(cost_choice.observer, owner);
        assert_eq!(cost_choice.kind, ChoiceKind::SelectCards);
        assert!(
            cost_choice
                .options
                .iter()
                .any(|option| option.token == cost.raw()),
            "cost card among options"
        );

        // 选成本牌 → 弃置 → TakeOneHandCard 开「owner 从 target 手牌选一张」选择。
        let t_hand_before = hand_of(&state, target).len();
        let (state, take_choice) = submit(
            state,
            owner,
            CommandKind::ResolveChoice {
                choice_id: cost_choice.choice_id,
                selections: vec![cost.raw()],
                cancelled: false,
            },
            3,
        );
        let take_choice = take_choice.choice.expect("take pick opens");
        assert_eq!(take_choice.observer, owner);
        assert_eq!(
            take_choice
                .options
                .iter()
                .map(|option| option.token)
                .collect::<Vec<_>>(),
            [t_card.raw()]
        );
        let (mut state, mut outcome) = submit(
            state,
            owner,
            CommandKind::ResolveChoice {
                choice_id: take_choice.choice_id,
                selections: vec![t_card.raw()],
                cancelled: false,
            },
            4,
        );
        // 残留重复交付逐个取消（双重加载假象已在单载下消除；防御性处理）。
        for key in 5..9 {
            match outcome.choice.clone() {
                Some(choice) => {
                    let revision = state.revision;
                    let (next, next_outcome) = apply_command(
                        state,
                        Command {
                            actor: choice.observer,
                            expected_revision: ExpectedRevision(revision),
                            idempotency_key: IdempotencyKey(key),
                            kind: CommandKind::ResolveChoice {
                                choice_id: choice.choice_id,
                                selections: vec![],
                                cancelled: true,
                            },
                        },
                    );
                    state = next;
                    outcome = next_outcome;
                }
                None => break,
            }
        }
        assert!(matches!(outcome.result, CommandResult::Accepted { .. }));
        assert!(state.zones.get(discard).unwrap().cards.contains(&cost));
        assert_eq!(hand_of(&state, target).len(), t_hand_before - 1);
        assert_eq!(
            hand_of(&state, owner).len(),
            1,
            "cost discarded, taken card in owner hand"
        );
        assert!(hand_of(&state, owner).contains(&t_card));
        assert!(!state.has_pending_choice());
    }
}
