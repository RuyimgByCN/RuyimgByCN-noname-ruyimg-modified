//! Golden for CLAN「还阴」「清励」「八龙」的补牌语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANBALONG: u32 = 2_722_267_819;
const CLANHUANYIN: u32 = 2_722_285_495;
const CLANQINGLI: u32 = 2_992_461_968;
const CHENLIUWUSHI: u32 = 3_882_274_053;
const CLANDANDAO: u32 = 2_693_545_350;
/// Concatenate every committed clan source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn clan_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/clan");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "skill")
        })
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, skill_def: u32) {
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
    run_effects(tx, effects).unwrap();
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn committed_package_matches_source_and_draw_to_semantics() {
    // The dying pipeline nests run_effects several levels deep with large frames;
    // run on a dedicated thread with a bigger stack to stay clear of the default.
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(|| run_clan_draw_to_semantics())
        .unwrap()
        .join()
        .unwrap();
}

fn run_clan_draw_to_semantics() {
    let committed = package();
    let mut compiled = content_compiler::compile_source_named("clan", &clan_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    for def in [
        CLANHUANYIN,
        CLANQINGLI,
        CHENLIUWUSHI,
        CLANBALONG,
        CLANDANDAO,
    ] {
        assert!(
            committed
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == def),
            "clan pack must contain {def}"
        );
    }

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut dying = Transaction::begin(engine.state().clone());
    add_skill(&mut dying, owner, CLANHUANYIN);
    run_effects(
        &mut dying,
        vec![
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
            EffectRequest::EnterDying { player: owner },
        ],
    )
    .unwrap();
    assert_eq!(dying.state().hand_count(owner), 4);

    let mut phase_end = Transaction::begin(engine.state().clone());
    add_skill(&mut phase_end, owner, CLANQINGLI);
    run_effects(
        &mut phase_end,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    phase_end.state_mut().players.get_mut(owner).unwrap().max_hp = 7;
    run_effects(
        &mut phase_end,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: engine_core::state::Phase::End.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(phase_end.state().hand_count(owner), 5);
}

#[test]
fn balong_only_uses_the_first_hp_change_and_requires_a_unique_trick_majority() {
    let committed = package();
    assert_eq!(committed.manifest.rule_vm_abi_version, 649);
    assert_eq!(
        committed
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == CLANBALONG)
            .map(|trigger| trigger.event.as_str())
            .collect::<Vec<_>>(),
        ["damage", "hp_lost", "hp_recovered", "max_hp_changed"]
    );

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_731,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut first = Transaction::begin(engine.state().clone());
    add_skill(&mut first, owner, CLANBALONG);
    run_effects(
        &mut first,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let trick = inject_hand(first.state_mut(), owner, CardKind::TrickWuzhong, 336_001);
    let events = run_effects(
        &mut first,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to } if cards == &[trick] && to.is_empty()
    )));
    assert_eq!(first.state().hand_count(owner), 3);

    let second_events = run_effects(
        &mut first,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!second_events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
    assert_eq!(first.state().hand_count(owner), 3);

    let mut tied = Transaction::begin(engine.state().clone());
    add_skill(&mut tied, owner, CLANBALONG);
    run_effects(
        &mut tied,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    inject_hand(tied.state_mut(), owner, CardKind::TrickWuzhong, 336_002);
    inject_hand(tied.state_mut(), owner, CardKind::Sha, 336_003);
    let tied_events = run_effects(
        &mut tied,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!tied_events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsRevealed { .. })));
    assert_eq!(tied.state().hand_count(owner), 2);
}
