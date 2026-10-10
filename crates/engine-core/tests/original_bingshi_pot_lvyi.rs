//! Golden coverage for BINGSHI 势吕壹「隳律/颂谳/势施」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HUILV: u32 = 4_200_000_570;
const SONGYAN: u32 = 4_200_000_571;
const SHISHI: u32 = 4_200_000_572;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_lvyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 564);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([HUILV, SONGYAN, SHISHI].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_owner = if matches!(
        zone_kind,
        ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Expansion
    ) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(564_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn request(
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolvePotlvyi {
        player,
        target,
        card,
        action,
        event_cards,
        targets: Vec::new(),
        skill,
    }
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_pot_lvyi_at_abi564() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_lvyi")
            .unwrap()
            .skills,
        [HUILV, SONGYAN, SHISHI]
    );
}

#[test]
fn huilv_rejects_a_pending_choice_after_the_skill_is_removed() {
    let (state, owner, order) = setup(5640);
    let actor = order[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        0,
        CardKind::Sha,
        Suit::Heart,
    );
    let huilv = skill(tx.state(), owner, HUILV);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 0, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let mut pending = tx.commit().state;
    pending.skills.get_mut(huilv).unwrap().enabled = false;
    let (_, result) = resolve(pending, vec![1], false, 5641);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn huilv_tracks_the_exact_used_card_and_damages_the_phase_actor() {
    let (state, owner, order) = setup(5641);
    let actor = order[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let tracked = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        1,
        CardKind::Sha,
        Suit::Heart,
    );
    let huilv = skill(tx.state(), owner, HUILV);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 0, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![1], false, 5642);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(actor).contains(&tracked));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![request(owner, actor, tracked, 1, Vec::new(), huilv)],
    )
    .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![tracked],
            to: discard,
            from: Some(hand),
            reason: engine_api::CardMoveReason::Use,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(actor).unwrap().hp;
    run_effects(
        &mut tx,
        vec![request(owner, actor, CardId::NONE, 2, Vec::new(), huilv)],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(actor).unwrap().hp, hp_before - 1);
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}

#[test]
fn huilv_card_outside_discard_causes_unreal_damage_then_can_be_given_away() {
    let (state, owner, order) = setup(5643);
    let actor = order[1];
    let recipient = order[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let tracked = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        2,
        CardKind::Tao,
        Suit::Diamond,
    );
    let huilv = skill(tx.state(), owner, HUILV);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 0, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![1], false, 5644);
    let hp_before = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 2, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let pending_songyan = tx.commit().state;
    assert_eq!(
        pending_songyan.players.get(owner).unwrap().hp,
        hp_before - 1
    );
    let (pending_gift, result) = resolve(pending_songyan, Vec::new(), true, 5645);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_ne!(pending_gift.resolution.pending_choice_id, 0);
    let (state, result) = resolve(pending_gift, vec![recipient.raw()], false, 5646);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(recipient).contains(&tracked));
    assert!(!state.hand_cards(actor).contains(&tracked));
}

#[test]
fn songyan_allows_matching_round_discard_color_as_wuxie() {
    let (state, owner, order) = setup(5646);
    let responder = order[1];
    let mut tx = Transaction::begin(state);
    let discarded = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        3,
        CardKind::Sha,
        Suit::Spade,
    );
    let response = inject(
        &mut tx,
        responder,
        ZoneKind::Hand,
        4,
        CardKind::Tao,
        Suit::Club,
    );
    let songyan = skill(tx.state(), owner, SONGYAN);
    run_effects(
        &mut tx,
        vec![request(
            owner,
            responder,
            CardId::NONE,
            3,
            vec![discarded],
            songyan,
        )],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, owner, CardId::NONE, 4, Vec::new(), songyan)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        PlayerId::new(tx.state().resolution.pending_observer),
        responder
    );
    let (state, result) = resolve(tx.commit().state, vec![response.raw()], false, 5648);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(responder).contains(&response));
}

#[test]
fn shishi_freezes_each_card_then_stops_recording_after_first_response() {
    let (state, owner, order) = setup(5647);
    let source = order[1];
    let mut tx = Transaction::begin(state);
    let wuxie = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        5,
        CardKind::TrickWuxie,
        Suit::Spade,
    );
    let shishi = skill(tx.state(), owner, SHISHI);
    run_effects(
        &mut tx,
        vec![request(owner, source, CardId::NONE, 6, Vec::new(), shishi)],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::UseVirtualCardOn {
            player: owner,
            kind: CardKind::TrickWuzhong.tag(),
            target: owner,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(source).contains(&wuxie));
    assert_eq!(tx.state().resolution.pending_choice_id, 0);

    run_effects(
        &mut tx,
        vec![request(owner, source, wuxie, 8, Vec::new(), shishi)],
    )
    .unwrap();
    clear_hands(&mut tx);
    let wuxie = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        6,
        CardKind::TrickWuxie,
        Suit::Club,
    );
    run_effects(
        &mut tx,
        vec![request(owner, source, CardId::NONE, 6, Vec::new(), shishi)],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseVirtualCardOn {
                player: owner,
                kind: CardKind::TrickWuzhong.tag(),
                target: owner,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        PlayerId::new(tx.state().resolution.pending_observer),
        source
    );
    assert!(tx.state().hand_cards(source).contains(&wuxie));
}
