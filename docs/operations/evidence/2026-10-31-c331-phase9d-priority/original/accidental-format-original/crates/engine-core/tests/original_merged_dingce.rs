//! O2 全量合并第二批（2026-10-14）：家族 dingce 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_dingce_machine {
    //! 第二百八十一批：jsrgdingce 定策状态机行为测试（ABI 730 / tag 752 /
    //! Cont 326）。链路：受到伤害后 → 弃自己一张手牌（可取消）→ owner 代伤害
    //! 来源选弃一张手牌（强制）；同色分支（虚拟【洞烛先机】）因卡牌不在牌池
    //! 登记债，暂不结算。

    use engine_api::{
        CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
        ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
    };
    use engine_core::command::{apply_command, PipelineResult};
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const DINGCE: u32 = 2_518_986_694;
    const DINGCE_SRC: &str = include_str!("../../../content/original/ported/jsrg/jsrgdingce.skill");
    const JSRG: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        state.hand_cards(player)
    }

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let package = content_schema::ContentPackage::from_json_bytes(JSRG).unwrap();
        let compiled = content_compiler::compile_source_named("dingce-test", DINGCE_SRC).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&compiled)
            .unwrap();
        bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
        let players = tx.state().players.alive_by_seat();
        let (owner, source) = (players[0], players[1]);
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.extend(
            [owner, source]
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        );
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DINGCE),
        });
        run_effects(&mut tx, effects).unwrap();
        for player in [owner, source] {
            let state = tx.state_mut().players.get_mut(player).unwrap();
            state.hp = 10;
            state.max_hp = 10;
        }
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        (tx.commit().state, owner, source)
    }

    fn inject_hand(
        state: &GameState,
        player: PlayerId,
        suit: Suit,
        rank: u8,
    ) -> (GameState, CardId) {
        let mut tx = Transaction::begin(state.clone());
        let zone = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(99_000 + u32::from(rank)),
            kind: CardKind::Other,
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

    fn resolve(
        state: GameState,
        actor: PlayerId,
        selections: Vec<u32>,
        cancelled: bool,
        key: u64,
    ) -> (GameState, PipelineResult) {
        let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
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
        (next, result)
    }

    fn fire_damage(state: GameState, source: PlayerId, owner: PlayerId) -> (GameState, bool) {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
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

    #[test]
    fn package_publishes_dingce_damage_follow_up() {
        let package = content_schema::ContentPackage::from_json_bytes(JSRG).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 730);
        let trigger = package
            .trigger_index
            .iter()
            .find(|trigger| trigger.skill_def_id == DINGCE)
            .expect("dingce trigger");
        assert_eq!(trigger.event, "damage");
        assert_eq!(trigger.filter, "owner_is_target_source_alive");
    }

    #[test]
    fn both_players_discard_one_card_each() {
        let (state, owner, source) = setup(20_260_914_301);
        let (state, owner_card) = inject_hand(&state, owner, Suit::Heart, 5);
        let (state, source_card) = inject_hand(&state, source, Suit::Spade, 6);
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();

        let (state, opened) = fire_damage(state, source, owner);
        assert!(opened, "cost choice opens for owner");
        let state = resolve(state, owner, vec![owner_card.raw()], false, 1).0;
        let state = resolve(state, owner, vec![source_card.raw()], false, 2).0;
        assert!(!state.has_pending_choice());
        let pile = state.zones.get(discard).unwrap();
        assert!(pile.cards.contains(&owner_card));
        assert!(pile.cards.contains(&source_card));
        assert!(hand_of(&state, owner).is_empty());
        assert!(hand_of(&state, source).is_empty());
    }

    #[test]
    fn cancelling_or_empty_source_hand_limits_the_flow() {
        // 取消：无结算。
        let (state, owner, source) = setup(20_260_914_302);
        let (state, owner_card) = inject_hand(&state, owner, Suit::Heart, 5);
        let (state, _source_card) = inject_hand(&state, source, Suit::Spade, 6);
        let (state, opened) = fire_damage(state, source, owner);
        assert!(opened);
        let (mut state, mut outcome) = resolve(state, owner, vec![], true, 1);
        // 取消后残留的重复交付询问逐个取消（引擎已知形态，见批 271/276）。
        for key in 10..15 {
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
        assert!(!state.has_pending_choice());
        assert!(hand_of(&state, owner).contains(&owner_card));

        // 来源无手牌：弃自己的牌后直接结束。
        let (state, owner, source) = setup(20_260_914_303);
        let (state, owner_card) = inject_hand(&state, owner, Suit::Heart, 5);
        let (state, opened) = fire_damage(state, source, owner);
        assert!(opened);
        let state = resolve(state, owner, vec![owner_card.raw()], false, 1).0;
        assert!(!state.has_pending_choice());
        assert!(state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .and_then(|zone| state.zones.get(zone).unwrap().cards.first().copied())
            .is_some());
    }
}
