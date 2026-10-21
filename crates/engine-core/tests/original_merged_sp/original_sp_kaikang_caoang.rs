//! Golden coverage for SP Cao Ang's 慷忾 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KAIKANG: u32 = 3_829_685_432;
const CAOANG: u32 = 3_627_114_828;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/kaikang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
    armor: CardId,
    sha: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("sp-kaikang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_704, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let source = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in [owner, target, source] {
        for kind in [ZoneKind::Hand, ZoneKind::Equip] {
            let zone = tx.state().zones.find(kind, player).unwrap();
            let cards = tx.state().zones.get(zone).unwrap().cards.clone();
            if !cards.is_empty() {
                effects.push(EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(zone),
                });
            }
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(KAIKANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let armor = inject(&mut tx, owner, CardKind::Armor, 2);
    let sha = inject(&mut tx, source, CardKind::Sha, 7);
    Setup {
        state: tx.commit().state,
        owner,
        source,
        target,
        armor,
        sha,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_kaikang_and_caoang_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == KAIKANG && skill.name == "kaikang"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOANG)
        .unwrap();
    assert_eq!(character.skills, [KAIKANG]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn draws_gives_then_recipient_optionally_uses_the_exact_equipment() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: setup.source,
                card: setup.sha,
                targets: vec![setup.target],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, invoked) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(invoked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    assert!(invoked
        .choice
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == setup.armor.raw()));

    let (state, given) = resolve(state, setup.owner, vec![setup.armor.raw()], false, 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::UseGivenEquipment
    );
    assert_eq!(state.cards.get(setup.armor).unwrap().owner, setup.target);

    let (state, used) = resolve(state, setup.target, vec![1], false, 3);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let equip = state.zones.find(ZoneKind::Equip, setup.target).unwrap();
    assert_eq!(state.cards.get(setup.armor).unwrap().zone, equip);
}
