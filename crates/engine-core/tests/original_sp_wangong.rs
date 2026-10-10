//! Golden for SP 黄祖「挽弓」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WANGONG: u32 = 3_743_761_196;
const WANGONG2: u32 = 3_164_390_759;
const HUANGZU: u32 = 4_052_796_884;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/wangong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "card use rejected: {:?}",
        pipeline.result
    );
    assert!(pipeline.choice.is_none());
    state
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == skill_def)
        })
}

#[test]
fn package_publishes_wangong_and_huangzu_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(
        content_compiler::compile_source_named("wangong-test", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        99
    );
    assert_eq!(package.skill_index.len(), 728);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUANGZU)
            .unwrap()
            .skills,
        [WANGONG]
    );
    for (skill, filter) in [
        (WANGONG, "owner_source_basic"),
        (WANGONG2, "owner_source_sha"),
        (WANGONG2, "owner_source_non_basic"),
    ] {
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == skill
                && trigger.event == "card_used"
                && trigger.filter == filter
                && !trigger.optional
        }));
    }
}

#[test]
fn basic_primes_next_card_sha_gains_damage_and_non_basic_consumes() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_072_301, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &seats {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WANGONG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let player = tx.state_mut().players.get_mut(*player).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let tao = inject(tx.state_mut(), owner, CardKind::Tao, 99_001);
    let sha = inject(tx.state_mut(), owner, CardKind::Sha, 99_002);
    let wuzhong = inject(tx.state_mut(), owner, CardKind::TrickWuzhong, 99_003);
    let mut state = tx.commit().state;

    state = play(state, owner, tao, Vec::new(), 1);
    assert!(has_skill(&state, owner, WANGONG2));
    assert_eq!(
        engine_core::query::sha_use_limit_for(&state, owner),
        i32::MAX as u32
    );
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state, owner, sha, target
    ));

    state = play(state, owner, sha, vec![target], 2);
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(has_skill(&state, owner, WANGONG2));

    state = play(state, owner, wuzhong, Vec::new(), 3);
    assert!(!has_skill(&state, owner, WANGONG2));
    assert_eq!(engine_core::query::sha_use_limit_for(&state, owner), 1);
}
