//! Golden for DIY 旧界黄月英「集智」的实体锦囊过滤与牌顶交换。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JUNKJIZHI: u32 = 2_783_324_283;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/diy/diy.npack.json");

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
        def: CardDefId::new(95_000_000 + u32::from(rank)),
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

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn started() -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.character_index.iter().any(|character| {
        character.name_key == "junk_huangyueying" && character.skills == [JUNKJIZHI, 2_508_459_447]
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUNKJIZHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

#[test]
fn non_basic_is_gained_and_basic_can_be_discarded_or_swapped() {
    let (mut tx, owner) = started();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let trick = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        Suit::Heart,
        1,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::RevealTopGainNonBasicOrSwapBasic { player: owner }],
    )
    .unwrap();
    assert_eq!(tx.state().cards.get(trick).unwrap().owner, owner);

    let basic_cancel = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        Suit::Diamond,
        2,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RevealTopGainNonBasicOrSwapBasic { player: owner }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::RevealTopGainNonBasicOrSwapBasic
    );
    let (state, result) = resolve(tx.commit().state, owner, Vec::new(), true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(basic_cancel).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    let replacement = tx.state().hand_cards(owner)[0];
    let basic_swap = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Club,
        3,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RevealTopGainNonBasicOrSwapBasic { player: owner }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(tx.commit().state, owner, vec![replacement.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(basic_swap).unwrap().owner, owner);
    assert_eq!(state.cards.get(replacement).unwrap().zone, draw);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last(),
        Some(&replacement)
    );
    assert!(!state.cards.get(replacement).unwrap().face_up);
}

#[test]
fn trigger_accepts_physical_trick_and_rejects_view_as_trick() {
    let (mut tx, owner) = started();
    let used = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        4,
    );
    let event = DomainEvent::CardUsed {
        player: owner,
        card: used,
        targets: Vec::new(),
    };
    assert_eq!(
        dispatch_for_events(&mut tx, &[event.clone()], &mut TriggerFireRecord::new()),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = resolve(tx.commit().state, owner, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let mut converted = Transaction::begin(state);
    converted
        .state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, used.raw() as i32);
    assert!(
        dispatch_for_events(&mut converted, &[event], &mut TriggerFireRecord::new(),)
            .unwrap()
            .is_empty()
    );
}
