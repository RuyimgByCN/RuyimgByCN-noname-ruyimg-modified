//! Golden coverage for MOBILE 来敏「乱群」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LUANQUN: u32 = 3_119_764_480;
const LUANQUN_EFFECT: u32 = 3_973_186_966;
const LUANQUN_DIRECT_HIT: u32 = 3_537_894_821;
const LAISHOU: u32 = 2_235_144_457;
const LAIMIN: u32 = 4_094_388_327;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    mismatch: PlayerId,
    matching: PlayerId,
    skill: engine_api::SkillId,
    owner_card: CardId,
    mismatch_card: CardId,
    mismatch_second_sha: CardId,
    matching_card: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let source = [
        include_str!("../../../../content/original/ported/mobile/luanqun.skill"),
        include_str!("../../../../content/original/ported/mobile/luanqun_effect.skill"),
        include_str!("../../../../content/original/ported/mobile/luanqun_directHit.skill"),
    ]
    .join("\n");
    let package = content_compiler::compile_source_named("luanqun-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_303, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mismatch = players[1];
    let matching = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LUANQUN),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_card = inject(&mut tx, owner, CardKind::Shan, Suit::Heart, 99_801);
    let mismatch_card = inject(&mut tx, mismatch, CardKind::Sha, Suit::Club, 99_802);
    let mismatch_second_sha = inject(&mut tx, mismatch, CardKind::Sha, Suit::Spade, 99_803);
    let matching_card = inject(&mut tx, matching, CardKind::Tao, Suit::Diamond, 99_804);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == LUANQUN)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        mismatch,
        matching,
        skill,
        owner_card,
        mismatch_card,
        mismatch_second_sha,
        matching_card,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_laimin_and_both_hidden_helpers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 329);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LAIMIN)
            .unwrap()
            .skills,
        [LAISHOU, LUANQUN]
    );
    for skill in [LUANQUN, LUANQUN_EFFECT, LUANQUN_DIRECT_HIT] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn selections_stay_private_then_mismatch_restricts_first_sha_and_direct_hits_owner() {
    let setup = setup();
    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(
        activated
            .choice
            .as_ref()
            .unwrap_or_else(|| panic!("activation did not open a choice: {activated:?}"))
            .observer,
        setup.owner
    );
    assert!(!activated
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, first) = choose(state, setup.owner, vec![setup.owner_card.raw()], false, 2);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.mismatch);
    assert!(!first
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, second) = choose(
        state,
        setup.mismatch,
        vec![setup.mismatch_card.raw()],
        false,
        3,
    );
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.matching);
    assert!(!second
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));

    let (state, revealed) = choose(
        state,
        setup.matching,
        vec![setup.matching_card.raw()],
        false,
        4,
    );
    assert!(revealed.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::CardsRevealed { cards, to }
                if cards == &[setup.owner_card, setup.mismatch_card, setup.matching_card]
                    && to.is_empty()
        )
    }));
    assert_eq!(revealed.choice.as_ref().unwrap().observer, setup.owner);
    assert_eq!(
        revealed.choice.as_ref().unwrap().options[0].token,
        setup.matching_card.raw()
    );
    assert!(has_skill(&state, setup.mismatch, LUANQUN_EFFECT));
    assert!(has_skill(&state, setup.mismatch, LUANQUN_DIRECT_HIT));
    assert!(!has_skill(&state, setup.matching, LUANQUN_EFFECT));

    let (mut state, gained) = choose(
        state,
        setup.owner,
        vec![setup.matching_card.raw()],
        false,
        5,
    );
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.cards.get(setup.matching_card).unwrap().owner,
        setup.owner
    );

    state.turn.current = setup.mismatch;
    state.turn.phase = Phase::Play;
    let (next, rejected) = submit(
        state,
        setup.mismatch,
        CommandKind::PlayCard {
            card: setup.mismatch_card,
            targets: vec![setup.matching],
            card_targets: Vec::new(),
        },
        6,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let owner_hp = next.players.get(setup.owner).unwrap().hp;
    let (mut next, direct_hit) = submit(
        next,
        setup.mismatch,
        CommandKind::PlayCard {
            card: setup.mismatch_card,
            targets: vec![setup.owner],
            card_targets: Vec::new(),
        },
        7,
    );
    assert!(matches!(direct_hit.result, CommandResult::Accepted { .. }));
    assert!(direct_hit.choice.is_none());
    assert_eq!(next.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert!(!has_skill(&next, setup.mismatch, LUANQUN_EFFECT));
    assert!(has_skill(&next, setup.mismatch, LUANQUN_DIRECT_HIT));

    next.turn.set_flag(turn_flag::SHA_PLAYED, 0);
    next.turn.set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let (mut next, unrestricted) = submit(
        next,
        setup.mismatch,
        CommandKind::PlayCard {
            card: setup.mismatch_second_sha,
            targets: vec![setup.matching],
            card_targets: Vec::new(),
        },
        8,
    );
    assert!(matches!(
        unrestricted.result,
        CommandResult::Accepted { .. }
    ));

    next.turn.phase = Phase::End;
    let (next, ended) = submit(next, setup.mismatch, CommandKind::EndPhase, 9);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&next, setup.mismatch, LUANQUN_DIRECT_HIT));
}
