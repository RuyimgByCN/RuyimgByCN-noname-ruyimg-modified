//! O2 全量合并第二批（2026-10-14）：家族 qiangyong 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_qiangyong_machine {
    //! 第二百七十八批：mbqiangyong 羌勇状态机行为测试（ABI 727 / tag 751 /
    //! Cont 325）。链路：【杀】造成伤害（damage_before）→ owner 从目标 he 区
    //! 选 X 张（X=本回合用杀数）强制弃置 → 目标无手牌则此伤害 +1。

    use engine_api::{
        CardDefId, CardId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
        ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::{apply_command, PipelineResult};
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const QIANGYONG: u32 = 3_316_930_446;
    const QIANGYONG_SRC: &str =
        include_str!("../../../content/original/ported/mobile/mbqiangyong.skill");
    const MOBILE: &[u8] =
        include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

    struct Setup {
        state: GameState,
        owner: PlayerId,
        target: PlayerId,
        sha: CardId,
    }

    fn inject(
        tx: &mut Transaction,
        owner: PlayerId,
        kind: CardKind,
        suit: Suit,
        rank: u8,
    ) -> CardId {
        let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(98_800 + u32::from(rank)),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        card
    }

    fn setup(seed: u64) -> Setup {
        let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
        let compiled =
            content_compiler::compile_source_named("qiangyong-test", QIANGYONG_SRC).unwrap();
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
            skill_def: SkillDefId::new(QIANGYONG),
        });
        run_effects(&mut tx, effects).unwrap();
        for player in [owner, target] {
            let state = tx.state_mut().players.get_mut(player).unwrap();
            state.hp = 10;
            state.max_hp = 10;
        }
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 7);
        Setup {
            state: tx.commit().state,
            owner,
            target,
            sha,
        }
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

    fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
        state.hand_cards(player)
    }

    fn play_sha(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
        submit(
            setup.state.clone(),
            setup.owner,
            CommandKind::PlayCard {
                card: setup.sha,
                targets: vec![setup.target],
                card_targets: vec![],
            },
            key,
        )
    }

    #[test]
    fn package_publishes_qiangyong_locked_damage_before() {
        let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 734);
        let trigger = package
            .trigger_index
            .iter()
            .find(|trigger| trigger.skill_def_id == QIANGYONG)
            .expect("qiangyong trigger");
        assert_eq!(trigger.event, "damage_before");
        assert_eq!(trigger.filter, "owner_is_source_sha_card_damage_before");
    }

    #[test]
    fn sha_damage_prompts_discard_of_sha_count_cards_with_bonus() {
        // 首次用杀：X=1。target 手牌注入 3 张，弃 1 后仍有手牌 → 伤害不 +1。
        let mut setup = setup(20_260_914_101);
        let mut tx = Transaction::begin(setup.state);
        for rank in [5u8, 6, 7] {
            inject(&mut tx, setup.target, CardKind::Other, Suit::Heart, rank);
        }
        setup.state = tx.commit().state;
        let target_hp = hp_of(&setup.state, setup.target);

        let (state, discard_choice) = play_sha(&setup, 1);
        // 目标先被询问是否出【闪】？空手牌直接进入羌勇弃牌选择。
        let discard_choice: &ChoiceRequest = discard_choice
            .choice
            .as_ref()
            .expect("discard prompt opens");
        assert_eq!(discard_choice.observer, setup.owner);
        assert_eq!(discard_choice.kind, ChoiceKind::SelectCards);
        assert_eq!(discard_choice.min_count, 1);
        assert_eq!(discard_choice.max_count, 1);

        // 弃 1 张（点数 5）→ 目标仍剩 2 张手牌 → 无 +1。
        let pick = state
            .zones
            .find(ZoneKind::Hand, setup.target)
            .and_then(|zone| state.zones.get(zone).unwrap().cards.first().copied())
            .unwrap();
        let (state, outcome2) = submit(
            state,
            setup.owner,
            CommandKind::ResolveChoice {
                choice_id: discard_choice.choice_id,
                selections: vec![pick.raw()],
                cancelled: false,
            },
            2,
        );
        eprintln!(
            "DBG qy: hp={} pending={:?}",
            hp_of(&state, setup.target),
            outcome2
                .choice
                .as_ref()
                .map(|c| (c.observer, c.kind, &c.options))
        );
        assert_eq!(hp_of(&state, setup.target), target_hp - 1);
        let _ = discard_choice;
    }

    #[test]
    fn empty_hand_after_discard_increases_damage() {
        // target 仅 1 张手牌：X=1 弃光 → 伤害 +1。
        let mut setup = setup(20_260_914_102);
        let mut tx = Transaction::begin(setup.state);
        inject(&mut tx, setup.target, CardKind::Other, Suit::Heart, 5);
        setup.state = tx.commit().state;
        let target_hp = hp_of(&setup.state, setup.target);

        let (state, discard_choice) = play_sha(&setup, 1);
        let discard_choice = discard_choice
            .choice
            .as_ref()
            .expect("discard prompt opens");

        let pick = state
            .zones
            .find(ZoneKind::Hand, setup.target)
            .and_then(|zone| state.zones.get(zone).unwrap().cards.first().copied())
            .unwrap();
        let (state, _) = submit(
            state,
            setup.owner,
            CommandKind::ResolveChoice {
                choice_id: discard_choice.choice_id,
                selections: vec![pick.raw()],
                cancelled: false,
            },
            2,
        );
        // 弃光手牌 → 伤害 +1：10 - 1(杀) - 1(羌勇) = 8。
        assert_eq!(hp_of(&state, setup.target), target_hp - 2);
    }

    fn hp_of(state: &GameState, player: PlayerId) -> i32 {
        state.players.get(player).unwrap().hp
    }
}
