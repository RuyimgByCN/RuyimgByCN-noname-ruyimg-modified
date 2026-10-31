//! O2 全量合并第二批（2026-10-14）：家族 inherited 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_inherited_skill_rewrites {
    //! Golden coverage for three legacy skills rewritten entirely through existing rule surfaces.

    use content_compiler::compile_source_named;
    use content_schema::ContentPackage;
    use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
    use engine_core::effect::{bootstrap_match, run_effects};
    use engine_core::state::{Card, CardKind, GameState, ZoneKind};
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;

    const REJIANXIONG: u32 = 2_280_204_341;
    const NSLIEGONG: u32 = 4_133_432_161;
    const LIULI_DAXIAOQIAO: u32 = 3_661_036_669;
    const REJIANXIONG_SOURCE: &str =
        include_str!("../../../content/original/ported/refresh/rejianxiong.skill");
    const NSLIEGONG_SOURCE: &str =
        include_str!("../../../content/original/ported/diy/nsliegong.skill");
    const XINLIEGONG_SOURCE: &str =
        include_str!("../../../content/original/ported/shenhua/xinliegong.skill");
    const LIULI_DAXIAOQIAO_SOURCE: &str =
        include_str!("../../../content/original/ported/tw/liuli_daxiaoqiao.skill");
    const LIULI_SOURCE: &str = include_str!("../../../content/standard/liuli.skill");
    const REFRESH: &[u8] =
        include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
    const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");
    const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

    fn setup_rejianxiong() -> (GameState, PlayerId, PlayerId) {
        let package = compile_source_named("rejianxiong-test", REJIANXIONG_SOURCE).unwrap();
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 2_026_073_002, "identity-standard").unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&package)
            .unwrap();
        let players = tx.state().players.alive_by_seat();
        let (owner, source) = (players[0], players[1]);
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REJIANXIONG),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state_mut().players.get_mut(owner).unwrap().hp = 20;
        tx.state_mut().players.get_mut(owner).unwrap().max_hp = 20;
        (tx.commit().state, owner, source)
    }

    fn inject_processing_sha(state: &mut GameState) -> CardId {
        let processing = state
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(99_601),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: 7,
            weapon_range: 0,
            zone: processing,
            owner: PlayerId::NONE,
            face_up: true,
        });
        state.zones.get_mut(processing).unwrap().cards.push(card);
        card
    }

    fn deal_damage(
        state: GameState,
        source: PlayerId,
        target: PlayerId,
        card: CardId,
    ) -> GameState {
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::DealCardDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
                card,
                card_kind: CardKind::Sha.tag(),
                card_flags: 0,
            }],
        )
        .unwrap();
        tx.commit().state
    }

    fn assert_same_rule_shape(child_source: &str, parent_source: &str) {
        let child = compile_source_named("inherited-child", child_source).unwrap();
        let parent = compile_source_named("inherited-parent", parent_source).unwrap();
        assert_eq!(child.modules.len(), 1);
        assert_eq!(parent.modules.len(), 1);
        assert_eq!(child.modules[0].code, parent.modules[0].code);
        assert_eq!(child.modules[0].local_count, parent.modules[0].local_count);
        assert_eq!(child.modules[0].max_stack, parent.modules[0].max_stack);
        assert_eq!(child.modules[0].caps_mask, parent.modules[0].caps_mask);

        let child_trigger = &child.trigger_index[0];
        let parent_trigger = &parent.trigger_index[0];
        assert_eq!(child_trigger.event, parent_trigger.event);
        assert_eq!(child_trigger.filter, parent_trigger.filter);
        assert_eq!(child_trigger.optional, parent_trigger.optional);
        assert_eq!(child_trigger.choice_actor, parent_trigger.choice_actor);

        let child_modifiers: Vec<_> = child
            .modifier_index
            .iter()
            .map(|modifier| {
                (
                    &modifier.target,
                    &modifier.selector,
                    &modifier.mode,
                    &modifier.value_source,
                    modifier.value,
                )
            })
            .collect();
        let parent_modifiers: Vec<_> = parent
            .modifier_index
            .iter()
            .map(|modifier| {
                (
                    &modifier.target,
                    &modifier.selector,
                    &modifier.mode,
                    &modifier.value_source,
                    modifier.value,
                )
            })
            .collect();
        assert_eq!(child_modifiers, parent_modifiers);
    }

    #[test]
    fn pure_inherits_compile_to_the_same_rule_handlers_as_their_parents() {
        assert_same_rule_shape(NSLIEGONG_SOURCE, XINLIEGONG_SOURCE);
        assert_same_rule_shape(LIULI_DAXIAOQIAO_SOURCE, LIULI_SOURCE);
    }

    #[test]
    fn rejianxiong_forcedly_gains_the_processing_card_then_draws() {
        let (mut state, owner, source) = setup_rejianxiong();
        let card = inject_processing_sha(&mut state);
        let hand_before = state.hand_count(owner);
        let state = deal_damage(state, source, owner, card);
        assert_eq!(state.hand_count(owner), hand_before + 2);
        assert!(state.hand_cards(owner).contains(&card));

        let (state, owner, source) = setup_rejianxiong();
        let hand_before = state.hand_count(owner);
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand_before);
    }

    #[test]
    fn committed_packages_publish_all_three_rewritten_skill_ids() {
        for (bytes, skill_def_id, name) in [
            (REFRESH, REJIANXIONG, "rejianxiong"),
            (DIY, NSLIEGONG, "nsliegong"),
            (TW, LIULI_DAXIAOQIAO, "liuli_daxiaoqiao"),
        ] {
            let package = ContentPackage::from_json_bytes(bytes).unwrap();
            assert!(package
                .skill_index
                .iter()
                .any(|skill| { skill.skill_def_id == skill_def_id && skill.name == name }));
        }
    }
}
