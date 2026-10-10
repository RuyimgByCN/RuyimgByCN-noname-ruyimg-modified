//! Golden coverage for SHENHUA「驱虎」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XUNYU: u32 = 4_288_100_271;
const RE_XUNYU: u32 = 3_554_276_229;
const QUHU: u32 = 4_267_019_884;
const QUHU_WIN: u32 = 2_288_140_413;
const QUHU_LOSE: u32 = 4_142_452_028;
const JIEMING: u32 = 3_579_069_878;
const REJIEMING: u32 = 4_088_661_605;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/shenhua/quhu.skill"),
    include_str!("../../../content/original/ported/shenhua/quhu_win.skill"),
    include_str!("../../../content/original/ported/shenhua/quhu_lose.skill")
);
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    victim: PlayerId,
    far: PlayerId,
    owner_card: CardId,
    source_card: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(owner_rank: u8, source_rank: u8) -> Setup {
    let package = content_compiler::compile_source_named("quhu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_411, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, source, victim, far) = (seats[0], seats[1], seats[2], seats[3]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QUHU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 6;
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let owner_card = inject(tx.state_mut(), owner, 99_401, Suit::Spade, owner_rank);
    let source_card = inject(tx.state_mut(), source, 99_402, Suit::Heart, source_rank);
    Setup {
        state: tx.commit().state,
        owner,
        source,
        victim,
        far,
        owner_card,
        source_card,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn command(
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state.clone(),
        actor,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled: false,
        },
        key,
    )
}

fn compare(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    let active = skill_id(&setup.state, setup.owner, QUHU).unwrap();
    let (state, choose_owner_card) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.source],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(
        choose_owner_card.result,
        CommandResult::Accepted { .. }
    ));
    let (state, choose_source_card) =
        resolve(state, setup.owner, vec![setup.owner_card.raw()], key + 1);
    assert!(matches!(
        choose_source_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    resolve(state, setup.source, vec![setup.source_card.raw()], key + 2)
}

#[test]
fn packages_publish_abi159_quhu_and_both_complete_xunyu_variants() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    assert_eq!(shenhua.skill_index.len(), 201);
    assert_eq!(shenhua.character_index.len(), 48);
    let active = shenhua
        .active_index
        .iter()
        .find(|active| active.skill_def_id == QUHU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().unwrap().hp_gt_owner);
    assert!(active.targets.as_ref().unwrap().can_compare);
    assert_eq!(
        shenhua
            .character_index
            .iter()
            .find(|character| character.id == XUNYU)
            .unwrap()
            .skills,
        [QUHU, JIEMING]
    );

    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(mobile.manifest.rule_vm_abi_version, 734);
    assert_eq!(mobile.skill_index.len(), 440);
    assert_eq!(mobile.character_index.len(), 204);
    assert_eq!(
        mobile
            .character_index
            .iter()
            .find(|character| character.id == RE_XUNYU)
            .unwrap()
            .skills,
        [QUHU, REJIEMING]
    );

    let compiled = content_compiler::compile_source_named("quhu", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
    for expected in &compiled.modules {
        let actual = shenhua
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn activation_rejects_equal_or_lower_hp_targets() {
    for target_hp in [3, 2] {
        let mut setup = setup(13, 1);
        setup.state.players.get_mut(setup.source).unwrap().hp = target_hp;
        let active = skill_id(&setup.state, setup.owner, QUHU).unwrap();
        let (_, result) = command(
            setup.state,
            setup.owner,
            CommandKind::ActivateSkill {
                skill: active,
                targets: vec![setup.source],
                card_targets: Vec::new(),
            },
            10 + target_hp as u64,
        );
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
    }
}

#[test]
fn strict_win_lets_owner_choose_in_bound_sources_range_and_uses_that_source() {
    let setup = setup(13, 1);
    let (state, compared) = compare(&setup, 20);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseBoundSourceAttackRangeDamage
    );
    let options: Vec<_> = compared
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&setup.owner.raw()));
    assert!(options.contains(&setup.victim.raw()));
    assert!(!options.contains(&setup.source.raw()));
    assert!(!options.contains(&setup.far.raw()));
    assert!(skill_id(&state, setup.owner, QUHU_WIN).is_some());

    let victim_hp = state.players.get(setup.victim).unwrap().hp;
    let (state, damaged) = resolve(state, setup.owner, vec![setup.victim.raw()], 23);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.victim).unwrap().hp, victim_hp - 1);
    assert!(skill_id(&state, setup.owner, QUHU_WIN).is_none());
    assert!(!state.has_pending_choice());
}

#[test]
fn loss_and_tie_both_damage_owner_from_compare_target() {
    for (owner_rank, source_rank, key) in [(1, 13, 30), (7, 7, 40)] {
        let setup = setup(owner_rank, source_rank);
        let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
        let (state, compared) = compare(&setup, key);
        assert!(matches!(compared.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
        assert!(skill_id(&state, setup.owner, QUHU_LOSE).is_none());
        assert!(!state.has_pending_choice());
    }
}
