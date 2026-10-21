//! Golden coverage for 标刘表「聚进」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_LIUBIAO: u32 = 2_470_672_462;
const STD_QUYI: u32 = 2_888_834_865;
const STDZISHOU: u32 = 2_683_293_404;
const ZONGSHI: u32 = 2_639_072_445;
const STDJUJIN: u32 = 3_408_182_272;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdjujin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    (state, output)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_and_runtime_preserve_exact_discard_then_recover_order() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 134);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBIAO)
        .unwrap();
    assert_eq!(character.name_key, "std_liubiao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDZISHOU, ZONGSHI, STDJUJIN]);
    let compiled = content_compiler::compile_source_named("stdjujin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 134);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_074_402, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardCards {
        player: *player,
        cards: tx.state().hand_cards(*player),
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDJUJIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = STD_LIUBIAO;
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = STD_QUYI;
    {
        let participant = tx.state_mut().players.get_mut(owner).unwrap();
        participant.hp = 2;
        participant.max_hp = 3;
    }
    let cards = [
        inject(&mut tx, owner, 134_001),
        inject(&mut tx, owner, 134_002),
        inject(&mut tx, owner, 134_003),
    ];

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (selecting, select_offer) = resolve(committed.state, owner, vec![1], 1);
    let choice = select_offer.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));

    let (done, completed) = resolve(selecting, owner, vec![cards[0].raw(), cards[2].raw()], 2);
    assert_eq!(done.hand_count(owner), 1);
    assert_eq!(done.players.get(owner).unwrap().hp, 3);
    let moved = completed
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsMoved { .. }))
        .unwrap();
    let recovered = completed
        .events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::HpRecovered { player, .. } if *player == owner),
        )
        .unwrap();
    assert!(moved < recovered);
}
