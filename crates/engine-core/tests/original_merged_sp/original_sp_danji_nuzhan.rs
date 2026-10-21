//! Golden coverage for SP 关羽「单骑」and its derived「怒斩」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag, RULE_VM_ABI_VERSION,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DANJI: u32 = 2_150_258_599;
const NUZHAN: u32 = 2_974_830_679;
const NEW_REWUSHENG: u32 = 3_038_912_803;
const JSP_GUANYU: u32 = 3_737_388_953;
const LIUBEI: u32 = 8;
const SP: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");
const DANJI_SOURCE: &str = include_str!("../../../../content/original/ported/sp/danji.skill");
const NUZHAN_SOURCE: &str = include_str!("../../../../content/original/ported/sp/nuzhan.skill");
const WUSHENG_SOURCE: &str =
    include_str!("../../../../content/original/ported/refresh/new_rewusheng.skill");

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn skill(state: &GameState, player: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let source = format!("{DANJI_SOURCE}\n{NUZHAN_SOURCE}\n{WUSHENG_SOURCE}");
    let package = content_compiler::compile_source_named("danji-nuzhan-test", &source).unwrap();
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&standard)
        .unwrap();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_073_103, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, seats[0], seats[1])
}

fn add_skills(mut state: GameState, owner: PlayerId, defs: &[u32]) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        defs.iter()
            .map(|def| EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*def),
            })
            .collect(),
    )
    .unwrap();
    state = tx.commit().state;
    state
}

fn use_as_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    source: CardId,
    key: u64,
) -> GameState {
    let view_as = skill(&state, owner, NEW_REWUSHENG);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: view_as,
                targets: vec![target],
                card_targets: vec![source],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    state
}

#[test]
fn package_closes_jsp_guanyu_and_preserves_both_legacy_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.character_index.iter().any(|character| {
        character.id == JSP_GUANYU
            && character.name_key == "jsp_guanyu"
            && character.skills == [NEW_REWUSHENG, DANJI]
    }));
    assert!(package
        .skill_index
        .iter()
        .any(|entry| entry.skill_def_id == NUZHAN));

    let (state, owner, target) = setup();
    let mut state = add_skills(state, owner, &[NEW_REWUSHENG, NUZHAN]);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    state.players.get_mut(target).unwrap().hp = 10;
    state.players.get_mut(target).unwrap().max_hp = 10;
    let trick_a = inject(&mut state, owner, CardKind::TrickJuedou, 345_001);
    state = use_as_sha(state, owner, target, trick_a, 1);
    let trick_b = inject(&mut state, owner, CardKind::TrickGuohe, 345_002);
    state = use_as_sha(state, owner, target, trick_b, 2);
    let equipment = inject(&mut state, owner, CardKind::Weapon, 345_003);
    state = use_as_sha(state, owner, target, equipment, 3);
    assert_eq!(state.players.get(target).unwrap().hp, 6);

    let (state, lord, owner) = setup();
    let mut state = add_skills(state, owner, &[DANJI]);
    state.players.get_mut(lord).unwrap().character_def = LIUBEI;
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(owner).unwrap().max_hp = 4;
    for def in 345_010..345_014 {
        inject(&mut state, owner, CardKind::Other, def);
    }
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(has_skill(tx.state(), owner, DANJI));
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 4);

    tx.state_mut().players.get_mut(lord).unwrap().character_def = 0;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(!has_skill(tx.state(), owner, DANJI));
    assert!(has_skill(tx.state(), owner, 2));
    assert!(has_skill(tx.state(), owner, NUZHAN));
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 3);
}
