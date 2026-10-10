//! Golden for DIY 张宝「望风」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSWANGFENG: u32 = 2_410_227_371;
const SOURCE: &str = include_str!("../../../content/original/ported/diy/nswangfeng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let face_up = tx.state().zones.find(ZoneKind::Hand, owner) != Some(zone);
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_950 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn only_red_hand_equip_or_expansion_cards_can_rewrite_a_judgment() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 342);
    assert!(committed.manifest.rule_vm_abi_version <= engine_api::RULE_VM_ABI_VERSION);
    assert!(committed.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == NSWANGFENG
            && modifier.target == "judgment_rewrite_red_hand_equip_expansion"
    }));

    let package = content_compiler::compile_source_named("nswangfeng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let judged = players[0];
    let rewriter = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for player in &players {
        let hand = tx.state().zones.find(ZoneKind::Hand, *player).unwrap();
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: rewriter,
        skill_def: SkillDefId::new(NSWANGFENG),
    });
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, rewriter).unwrap();
    let black = inject(&mut tx, rewriter, hand, Suit::Spade, 4);
    let expansion = tx.state_mut().zones.alloc(Zone {
        id: engine_api::ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner: rewriter,
        cards: Vec::new(),
    });
    let red = inject(&mut tx, rewriter, expansion, Suit::Heart, 5);
    let judge = tx.state().zones.find(ZoneKind::Judge, judged).unwrap();
    let delayed = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_960),
        kind: CardKind::TrickLe,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: judge,
        owner: judged,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(judge)
        .unwrap()
        .cards
        .push(delayed);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(&mut tx, PlayerId::NONE, draw, Suit::Spade, 3);
    tx.state_mut().turn.current = judged;
    tx.state_mut().turn.phase = Phase::Judge;

    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor: judged,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.observer, rewriter);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![red.raw()]
    );
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == black.raw()));

    let (state, resolved) = apply_command(
        state,
        Command {
            actor: rewriter,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![red.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, discard);
}
