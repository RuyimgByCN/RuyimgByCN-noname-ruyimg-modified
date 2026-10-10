//! Golden coverage for mobile 曹性「劲镞」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBJINZU: u32 = 3_257_523_591;
const MBJINZU_EFFECT: u32 = 4_156_343_864;
const MBANXIAN: u32 = 2_708_680_147;
const MB_CAOXING: u32 = 2_827_969_473;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    owner_reveal: CardId,
    sha: CardId,
    target_low: CardId,
    target_high: CardId,
    target_shan: CardId,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
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

fn setup(owner_rank: u8, target_has_shan: bool) -> Setup {
    let source = [
        include_str!("../../../content/original/ported/mobile/mbjinzu.skill"),
        include_str!("../../../content/original/ported/mobile/mbjinzu_effect.skill"),
    ]
    .join("\n");
    let package = content_compiler::compile_source_named("mbjinzu-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_104, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(MBJINZU),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_reveal = inject(
        &mut tx,
        owner,
        CardKind::TrickWuzhong,
        Suit::Heart,
        owner_rank,
        99_901,
    );
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 6, 99_902);
    let target_low = inject(&mut tx, target, CardKind::Tao, Suit::Club, 3, 99_903);
    let target_high = inject(
        &mut tx,
        target,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        11,
        99_904,
    );
    let target_shan = if target_has_shan {
        inject(&mut tx, target, CardKind::Shan, Suit::Heart, 9, 99_905)
    } else {
        CardId::NONE
    };
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
                .is_some_and(|instance| instance.def.raw() == MBJINZU)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
        owner_reveal,
        sha,
        target_low,
        target_high,
        target_shan,
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
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
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
fn package_publishes_abi333_and_closes_caoxing() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == MB_CAOXING)
            .unwrap()
            .skills,
        [MBJINZU, MBANXIAN]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBJINZU_EFFECT && skill.locked));
}

#[test]
fn non_extreme_reveal_buffs_only_the_next_sha_against_that_target() {
    let setup = setup(7, true);
    let original_hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: vec![setup.owner_reveal],
        },
        1,
    );
    assert_eq!(activated.choice.as_ref().unwrap().observer, setup.target);
    assert_eq!(activated.choice.as_ref().unwrap().min_count, 2);
    let (state, revealed) = choose(
        state,
        setup.target,
        vec![setup.target_low.raw(), setup.target_high.raw()],
        2,
    );
    assert!(revealed.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to }
            if cards == &[setup.owner_reveal, setup.target_low, setup.target_high]
                && to.is_empty())
    }));
    assert!(has_skill(&state, setup.owner, MBJINZU_EFFECT));
    assert!(state.hand_cards(setup.target).contains(&setup.target_shan));

    let (state, played) = submit(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        3,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert!(played.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, original_hp - 2);
    assert!(!has_skill(&state, setup.owner, MBJINZU_EFFECT));
}

#[test]
fn unique_extreme_discards_all_reveals_and_refreshes_the_active_skill() {
    let setup = setup(13, false);
    let (state, activated) = submit(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![setup.target],
            card_targets: vec![setup.owner_reveal],
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.choice.is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [setup.owner_reveal, setup.target_low, setup.target_high] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 0);
    assert!(!has_skill(&state, setup.owner, MBJINZU_EFFECT));
}
