//! Golden coverage for 龙庞德「抬棺」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DRAGTAIGUAN: u32 = 3_898_790_253;
const DRAG_PANGDE: u32 = 3_996_963_952;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/dragtaiguan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("dragtaiguan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    for player in [owner, target] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
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
        skill_def: SkillDefId::new(DRAGTAIGUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DRAGTAIGUAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    cost: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![cost],
            },
        },
    )
}

fn use_once(mut setup: Setup, key: u64) -> Setup {
    let cost = inject(
        &mut setup.state,
        setup.owner,
        193_000 + key as u32,
        CardKind::TrickWuzhong,
    );
    let discarded = inject(
        &mut setup.state,
        setup.target,
        193_100 + key as u32,
        CardKind::Sha,
    );
    let (state, offered) = activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cost,
        key,
    );
    assert!(offered.choice.is_some());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, resolved) = apply_command(
        state,
        Command {
            actor: setup.target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key + 1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    setup.state = state;
    setup
}

fn rejected_activation(mut setup: Setup, key: u64) -> PipelineResult {
    let cost = inject(
        &mut setup.state,
        setup.owner,
        193_200 + key as u32,
        CardKind::TrickWuzhong,
    );
    inject(
        &mut setup.state,
        setup.target,
        193_300 + key as u32,
        CardKind::Sha,
    );
    activate(
        setup.state,
        setup.owner,
        setup.target,
        setup.skill,
        cost,
        key,
    )
    .1
}

#[test]
fn package_publishes_abi193_and_complete_drag_pangde() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DRAGTAIGUAN)
        .unwrap();
    assert_eq!(
        active.uses_per_turn_source.as_deref(),
        Some("owner_lost_hp_min_one")
    );
    assert_eq!(active.selection.as_ref().unwrap().zone, "hand_equip");
    let targets = active.targets.as_ref().unwrap();
    assert!(targets.in_attack_range && targets.hand_equip_nonempty && !targets.allow_self);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DRAG_PANGDE)
        .unwrap();
    assert_eq!(character.name_key, "drag_pangde");
    assert_eq!(character.skills, [DRAGTAIGUAN, 2]);

    let compiled = content_compiler::compile_source_named("dragtaiguan", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn active_limit_is_lost_hp_with_a_minimum_of_one() {
    let full = use_once(setup(2_026_072_701), 10);
    assert!(matches!(
        rejected_activation(full, 20).result,
        CommandResult::Rejected { .. }
    ));

    let mut wounded = setup(2_026_072_702);
    wounded.state.players.get_mut(wounded.owner).unwrap().hp = 2;
    let wounded = use_once(wounded, 30);
    let wounded = use_once(wounded, 40);
    assert!(matches!(
        rejected_activation(wounded, 50).result,
        CommandResult::Rejected { .. }
    ));
}
