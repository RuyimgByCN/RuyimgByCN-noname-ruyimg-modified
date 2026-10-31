//! O2 全量合并第二批（2026-10-14）：家族 mbbeiming 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_mbbeiming_machine {
    //! 第二百六十九批：mbbeiming 孛明状态机行为测试（ABI 719 / tag 742 /
    //! Cont 318）。链路：game_started → SelectTargets 至多两名任意角色（可取消）
    //! → 逐目标按手牌花色数从牌堆顶搜索 weapon_range 相等的武器牌置入其手牌区。

    use engine_api::{
        ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
        IdempotencyKey, PlayerId, SkillDefId,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::{bootstrap_match, run_effects, EffectError};
    use engine_core::state::{GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
    use rule_api::EffectRequest;

    const MBBEIMING: u32 = 3_072_978_888;
    const MBBEIMING_SRC: &str =
        include_str!("../../../content/original/ported/mobile/mbbeiming.skill");
    const MOBILE: &[u8] =
        include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

    fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
        let compiled =
            content_compiler::compile_source_named("mbbeiming-b269", MBBEIMING_SRC).unwrap();
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
            skill_def: SkillDefId::new(MBBEIMING),
        });
        run_effects(&mut tx, effects).unwrap();
        (tx.commit().state, owner, other)
    }

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<engine_api::CardId> {
        let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
        state.zones.get(hand).unwrap().cards.clone()
    }

    fn draw_pile_len(state: &GameState) -> usize {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        state.zones.get(draw).unwrap().cards.len()
    }

    fn fire_game_start(state: GameState) -> (GameState, bool) {
        let mut tx = Transaction::begin(state);
        let outcome = match dispatch_for_events(
            &mut tx,
            &[DomainEvent::GameStarted { player_count: 2 }],
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
    ) -> GameState {
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
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "resolve rejected {key}: {:?}",
            result.result
        );
        next
    }

    #[test]
    fn mbbeiming_contract_package_publishes_abi719() {
        let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 734);
        let trigger = package
            .trigger_index
            .iter()
            .find(|t| t.skill_def_id == MBBEIMING)
            .expect("mbbeiming trigger");
        assert_eq!(trigger.event, "game_started");
        assert!(!trigger.optional);
    }

    /// 目标手牌花色多样性（以选择时点为准）。
    fn suit_diversity(state: &GameState, before: &[engine_api::CardId]) -> u32 {
        let mut seen = [false; 4];
        for card in before {
            let card = state.cards.get(*card).unwrap();
            match card.suit {
                engine_api::Suit::Spade => seen[0] = true,
                engine_api::Suit::Heart => seen[1] = true,
                engine_api::Suit::Club => seen[2] = true,
                engine_api::Suit::Diamond => seen[3] = true,
                engine_api::Suit::None => {}
            }
        }
        seen.into_iter().filter(|s| *s).count() as u32
    }

    #[test]
    fn mbbeiming_grants_matching_weapons_to_both_targets() {
        // 种子 204：owner 花色 3、other 花色 3，牌堆内均有匹配范围武器。
        let (state, owner, other) = setup(204);
        let owner_hand_before = hand_of(&state, owner);
        let other_hand_before = hand_of(&state, other);
        let owner_suits = suit_diversity(&state, &owner_hand_before);
        let other_suits = suit_diversity(&state, &other_hand_before);
        let draw_before = draw_pile_len(&state);

        let (state, opened) = fire_game_start(state);
        assert!(opened, "target selection opens");
        let state = resolve(state, owner, vec![other.raw(), owner.raw()], false, 31);
        assert!(
            !state.has_pending_choice(),
            "single-stage grant, no re-choice"
        );

        let expectations = [
            (other, other_hand_before, other_suits),
            (owner, owner_hand_before, owner_suits),
        ];
        for (player, before, suits) in expectations {
            let gained: Vec<_> = hand_of(&state, player)
                .into_iter()
                .filter(|card| !before.contains(&card))
                .collect();
            assert_eq!(gained.len(), 1, "exactly one weapon granted");
            let granted = state.cards.get(gained[0]).unwrap();
            assert_eq!(
                granted.kind,
                engine_core::state::CardKind::Weapon,
                "granted card is a weapon"
            );
            assert_eq!(
                u32::from(granted.weapon_range.max(1)),
                suits,
                "weapon range equals hand suit diversity"
            );
        }
        assert_eq!(
            draw_pile_len(&state),
            draw_before - 2,
            "both from draw pile"
        );
    }

    #[test]
    fn mbbeiming_no_matching_weapon_skips_target() {
        // 种子 20_260_913_201：other 花色 2，而范围 2 的武器全在其手中——牌堆
        // 无匹配，按上游语义跳过（不发放、不报错）。
        let (state, owner, other) = setup(20_260_913_201);
        let t_hand_before = hand_of(&state, other);
        let draw_before = draw_pile_len(&state);

        let (state, opened) = fire_game_start(state);
        assert!(opened);
        let state = resolve(state, owner, vec![other.raw()], false, 32);
        assert!(!state.has_pending_choice());
        let gained: Vec<_> = hand_of(&state, other)
            .into_iter()
            .filter(|card| !t_hand_before.contains(card))
            .collect();
        assert!(gained.is_empty(), "no match → no grant");
        assert_eq!(draw_pile_len(&state), draw_before);
    }

    #[test]
    fn mbbeiming_cancel_places_nothing() {
        let (state, owner, other) = setup(20_260_913_202);
        let t_hand_before = hand_of(&state, other).len();
        let draw_before = draw_pile_len(&state);

        let (state, opened) = fire_game_start(state);
        assert!(opened);
        let state = resolve(state, owner, vec![], true, 41);
        assert!(!state.has_pending_choice());
        assert_eq!(hand_of(&state, other).len(), t_hand_before);
        assert_eq!(draw_pile_len(&state), draw_before);
    }
}
