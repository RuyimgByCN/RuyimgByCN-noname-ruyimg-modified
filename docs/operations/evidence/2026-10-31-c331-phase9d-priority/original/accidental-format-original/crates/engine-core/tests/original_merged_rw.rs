//! O2 全量合并第二批（2026-10-14）：家族 rw 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_rw_baiyin_machine {
    //! 第二百八十/二批：rw_baiyin_skill 照月狮子盔行为测试（触发期部分减伤桥
    //! ABI 731）。链路：owner 受到 >1 伤害 → modify_current_damage 将伤害值改
    //! 为 1（锁定，无选择）；伤害量=1 时不触发。

    use engine_api::{
        CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
        MarkDefId, PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::{apply_command, PipelineResult};
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const BAIYIN: u32 = 3_014_252_790;
    const MOBILE: &[u8] =
        include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

    fn setup(seed: u64, with_skill: bool) -> (GameState, PlayerId, PlayerId) {
        let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
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
        if with_skill {
            effects.push(EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(BAIYIN),
            });
        }
        run_effects(&mut tx, effects).unwrap();
        for player in [owner, source] {
            let state = tx.state_mut().players.get_mut(player).unwrap();
            state.hp = 10;
            state.max_hp = 10;
        }
        tx.state_mut().turn.current = source;
        tx.state_mut().turn.phase = Phase::Play;
        (tx.commit().state, owner, source)
    }

    fn inject_card(
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
            def: CardDefId::new(98_900 + u32::from(rank)),
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

    #[test]
    fn package_publishes_baiyin_damage_cap_trigger() {
        let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
        assert_eq!(package.manifest.rule_vm_abi_version, 734);
        let trigger = package
            .trigger_index
            .iter()
            .find(|trigger| trigger.skill_def_id == BAIYIN)
            .expect("baiyin trigger");
        assert_eq!(trigger.event, "damage_before");
        assert_eq!(trigger.filter, "owner_is_target_damage_amount_gt_one");
    }

    #[test]
    fn damage_above_one_is_capped_to_one() {
        // 真实管道：source 饮酒（MARK_JIU）后用【杀】→ 伤害 2 → 封顶为 1。
        let (state, owner, source) = setup(20_260_914_201, true);
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddMark {
                player: source,
                mark: MarkDefId::new(engine_core::content::MARK_JIU),
                delta: 1,
            }],
        )
        .unwrap();
        let state = tx.commit().state;
        let (state, sha) = inject_card(&state, source, CardKind::Sha, Suit::Spade, 7);
        let (state, outcome) = submit(
            state,
            source,
            CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
            1,
        );
        assert!(
            matches!(outcome.result, CommandResult::Accepted { .. }),
            "sha play rejected: {:?}",
            outcome.result
        );
        assert!(!state.has_pending_choice());
        assert_eq!(state.players.get(owner).unwrap().hp, 10 - 1);
    }

    #[test]
    fn damage_of_one_is_unchanged_and_non_owner_unaffected() {
        let (state, owner, source) = setup(20_260_914_202, true);
        let (state, sha) = inject_card(&state, source, CardKind::Sha, Suit::Spade, 7);
        // 1 点伤害不满足 >1：正常结算。
        let (state, _) = submit(
            state,
            source,
            CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
            1,
        );
        assert_eq!(state.players.get(owner).unwrap().hp, 9);

        // 无技能的 source 受 3 点伤害：不封顶（10-3=7）。
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: source,
                amount: 3,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap();
        let state = tx.commit().state;
        assert_eq!(state.players.get(source).unwrap().hp, 7);
    }
}
