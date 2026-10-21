//! Golden for 联动经典曹操「奸雄」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DCJIANXIONG: u32 = 3_695_091_838;
const DC_CAOCAO: u32 = 4_106_722_512;
const SOURCE: &str = include_str!("../../../../content/original/ported/collab/dcjianxiong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

fn inject(state: &mut GameState, zone_kind: ZoneKind, def: u32) -> CardId {
    let zone = state.zones.find(zone_kind, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == DCJIANXIONG).then_some(*count))
        .unwrap_or(0)
}

fn damage_and_accept(
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    card: CardId,
    key: u64,
) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealCardDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
                card,
                card_kind: CardKind::Sha.tag(),
                card_flags: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, output) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_and_runtime_match_the_legacy_growth_and_card_gain_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("dcjianxiong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 156);
    assert_eq!(package.character_index.len(), 90);
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == DC_CAOCAO && character.skills == [DCJIANXIONG]));
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash,
        compiled.modules[0].logic_hash
    );

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_722),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_722,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCJIANXIONG),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 100;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 100;
    let mut state = tx.commit().state;

    for index in 0..6 {
        let card = inject(&mut state, ZoneKind::Processing, 90_000 + index);
        state = damage_and_accept(state, owner, source, card, index as u64 + 1);
        assert!(state.hand_cards(owner).contains(&card));
        assert_eq!(mark_count(&state, owner), (index + 1).min(4) as i32);
    }
    assert_eq!(state.hand_count(owner), 26);

    let discarded = inject(&mut state, ZoneKind::DiscardPile, 91_000);
    state = damage_and_accept(state, owner, source, discarded, 7);
    assert_eq!(state.hand_count(owner), 31);
    assert!(!state.hand_cards(owner).contains(&discarded));
    assert_eq!(mark_count(&state, owner), 4);

    state = damage_and_accept(state, owner, source, CardId::NONE, 8);
    assert_eq!(state.hand_count(owner), 36);
}
