//! O2 全量合并第二批（2026-10-14）：家族 view 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_view_as_equipment_filter {
//! Golden for equipment-card view-as skills with independent card-type and zone checks.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRGSHILI: u32 = 2_444_554_979;
const STDYUNJI: u32 = 2_318_160_114;
const JSRG: &[u8] = include_bytes!("../../../content/original/ported/jsrg/jsrg.npack.json");
const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: (kind == CardKind::Weapon).then_some(3).unwrap_or(0),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(package_bytes: &[u8], player_count: u8, skill_def: u32) -> (GameState, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(package_bytes).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, player_count, 20_260_716, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    for player in tx.state().players.alive_by_seat() {
        clear_hand(&mut tx, player);
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.state().clone(), owner, skill)
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    source: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![source],
            },
        },
    )
}

#[test]
fn packages_expose_the_legacy_equipment_view_as_contracts() {
    let jsrg = content_schema::ContentPackage::from_json_bytes(JSRG).unwrap();
    assert_eq!(jsrg.manifest.rule_vm_abi_version, 730);
    let shili = jsrg
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JSRGSHILI)
        .expect("恃力 view_as entry");
    assert_eq!(shili.as_kind, "juedou");
    assert_eq!(shili.window, "play_phase");
    assert_eq!(shili.filter, "card_equip");
    assert_eq!(shili.source_zone, None);
    assert_eq!(shili.uses_per_turn, Some(1));

    let sixiang = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert!(sixiang.manifest.rule_vm_abi_version >= 5);
    let yunji = sixiang
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == STDYUNJI)
        .expect("运机 view_as entry");
    assert_eq!(yunji.as_kind, "jiedao");
    assert_eq!(yunji.window, "play_phase");
    assert_eq!(yunji.filter, "card_equip");
    assert_eq!(yunji.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(yunji.uses_per_turn, None);
}

#[test]
fn jsrgshili_only_uses_hand_equipment_as_duel_once_per_turn() {
    let (state, owner, skill) = setup(JSRG, 2, JSRGSHILI);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let non_equip = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        1,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (state, pipe) = activate(state, owner, skill, vec![target], non_equip, 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let mut tx = Transaction::begin(state);
    let equipped = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        2,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (state, pipe) = activate(state, owner, skill, vec![target], equipped, 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let mut tx = Transaction::begin(state);
    let source = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Diamond,
        3,
    );
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![target], source, 3);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(source).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    let second = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Spade,
        4,
    );
    let state = tx.state().clone();
    let before = canonical_state_hash(&state);
    let (state, pipe) = activate(state, owner, skill, vec![target], second, 4);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}

#[test]
fn stdyunji_uses_equipment_cards_from_hand_or_equipment_zone_as_jiedao() {
    for (index, zone) in [ZoneKind::Hand, ZoneKind::Equip].into_iter().enumerate() {
        let (state, owner, skill) = setup(SIXIANG, 3, STDYUNJI);
        let others: Vec<_> = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != owner)
            .collect();
        let (holder, victim) = (others[0], others[1]);
        let mut tx = Transaction::begin(state);
        let non_equip = inject(
            &mut tx,
            owner,
            ZoneKind::Hand,
            CardKind::Tao,
            Suit::Heart,
            5,
        );
        inject(
            &mut tx,
            holder,
            ZoneKind::Equip,
            CardKind::Weapon,
            Suit::Spade,
            6,
        );
        let state = tx.state().clone();
        let before = canonical_state_hash(&state);
        let (state, pipe) = activate(
            state,
            owner,
            skill,
            vec![holder, victim],
            non_equip,
            10 + index as u64,
        );
        assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
        assert_eq!(canonical_state_hash(&state), before);

        let mut tx = Transaction::begin(state);
        let source = inject(&mut tx, owner, zone, CardKind::Armor, Suit::Diamond, 7);
        let (state, pipe) = activate(
            tx.state().clone(),
            owner,
            skill,
            vec![holder, victim],
            source,
            20 + index as u64,
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        let choice = pipe.choice.expect("借刀持刀者选择");
        assert_eq!(choice.observer, holder);
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(source).unwrap().zone, discard);

        let (_, pipe) = apply_command(
            state,
            Command {
                actor: holder,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(30 + index as u64),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![0],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    }
}
}
