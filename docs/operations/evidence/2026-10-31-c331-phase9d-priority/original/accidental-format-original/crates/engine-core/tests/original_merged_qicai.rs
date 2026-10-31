//! O2 全量合并第二批（2026-10-14）：家族 qicai 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_qicai {
    //! Exact legacy coverage for DIY/界限突破「奇才」.

    use engine_api::{
        CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
        PlayerId, SkillDefId, Suit,
    };
    use engine_core::command::apply_command;
    use engine_core::effect::run_effects;
    use engine_core::query::{card_discard_protected_from, card_distance_unlimited_for_card};
    use engine_core::state::{Card, CardKind, Phase, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::Engine;
    use rule_api::EffectRequest;

    const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");
    const REFRESH: &[u8] =
        include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
    const JUNKQICAI: u32 = 2_508_459_447;
    const REQICAI: u32 = 2_792_486_309;

    fn inject(
        tx: &mut Transaction,
        owner: PlayerId,
        zone: engine_api::ZoneId,
        kind: CardKind,
        rank: u8,
    ) -> CardId {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(98_000 + kind.tag() as u32 * 20 + rank as u32),
            kind,
            suit: Suit::Spade,
            rank,
            weapon_range: u8::from(kind == CardKind::Weapon) * 2,
            zone,
            owner,
            face_up: kind.is_equip(),
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        card
    }

    fn assert_qicai(package: &[u8], skill: u32, protected: &[CardKind], discardable: &[CardKind]) {
        let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
        let mut engine = Engine::new();
        engine.load_package(&package).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715 + skill as u64,
                ruleset: "identity-standard".into(),
            },
        });
        let actor = engine.state().turn.current;
        let target = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != actor)
            .unwrap()
            .id;
        let mut tx = Transaction::begin(engine.state().clone());
        tx.state_mut().revision = 1;
        tx.state_mut().turn.phase = Phase::Play;

        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        for player in [actor, target] {
            let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let cards = tx.state().hand_cards(player);
            if !cards.is_empty() {
                run_effects(
                    &mut tx,
                    vec![EffectRequest::MoveCards {
                        cards,
                        to: discard,
                        from: Some(hand),
                    }],
                )
                .unwrap();
            }
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(skill),
            }],
        )
        .unwrap();

        let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
        let trick = inject(&mut tx, target, target_hand, CardKind::TrickBingliang, 1);
        assert!(card_distance_unlimited_for_card(tx.state(), target, trick));
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![trick],
                to: discard,
                from: Some(target_hand),
            }],
        )
        .unwrap();

        let equip = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
        let protected_cards: Vec<_> = protected
            .iter()
            .enumerate()
            .map(|(index, kind)| inject(&mut tx, target, equip, *kind, index as u8 + 2))
            .collect();
        let discardable_cards: Vec<_> = discardable
            .iter()
            .enumerate()
            .map(|(index, kind)| inject(&mut tx, target, equip, *kind, index as u8 + 8))
            .collect();
        for card in &protected_cards {
            assert!(card_discard_protected_from(
                tx.state(),
                actor,
                target,
                *card
            ));
            assert!(!card_discard_protected_from(
                tx.state(),
                target,
                target,
                *card
            ));
        }
        for card in &discardable_cards {
            assert!(!card_discard_protected_from(
                tx.state(),
                actor,
                target,
                *card
            ));
        }

        let actor_hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
        let guohe = inject(&mut tx, actor, actor_hand, CardKind::TrickGuohe, 13);
        let (pending, play) = apply_command(
            tx.state().clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::PlayCard {
                    card: guohe,
                    targets: vec![target],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(play.result, CommandResult::Accepted { .. }));
        let choice = play.choice.unwrap();
        for card in &protected_cards {
            assert!(!choice
                .options
                .iter()
                .any(|option| option.token == card.raw()));
        }
        for card in &discardable_cards {
            assert!(choice
                .options
                .iter()
                .any(|option| option.token == card.raw()));
        }

        let (_, illegal) = apply_command(
            pending.clone(),
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![protected_cards[0].raw()],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(illegal.result, CommandResult::Rejected { .. }));

        let selected = discardable_cards[0];
        let (resolved, legal) = apply_command(
            pending,
            Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(3),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![selected.raw()],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(legal.result, CommandResult::Accepted { .. }));
        assert_eq!(resolved.cards.get(selected).unwrap().zone, discard);
        assert_eq!(resolved.cards.get(protected_cards[0]).unwrap().zone, equip);
    }

    #[test]
    fn legacy_qicai_variants_match_distance_and_equipment_protection() {
        assert_qicai(
            DIY,
            JUNKQICAI,
            &[CardKind::Weapon, CardKind::Armor, CardKind::Treasure],
            &[CardKind::HorseOff, CardKind::HorseDef],
        );
        assert_qicai(
            REFRESH,
            REQICAI,
            &[CardKind::Armor, CardKind::Treasure],
            &[CardKind::Weapon, CardKind::HorseOff, CardKind::HorseDef],
        );
    }
}
