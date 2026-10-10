//! Golden coverage for old 关银屏's rewritten skill closure.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XUEJI_OLD: u32 = 3_528_622_023;
const OLDHUXIAO: u32 = 3_615_190_640;
const OLDWUJI: u32 = 3_438_545_283;

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    weapon_range: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(76_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    card: CardId,
    key: u64,
) -> (GameState, CommandResult) {
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![card],
            },
        },
    );
    (state, result.result)
}

#[test]
fn xueji_enforces_red_lost_hp_cap_and_oldwuji_awakens_at_three_damage() {
    let source = [
        include_str!("../../../content/original/ported/sp/xueji_old.skill"),
        include_str!("../../../content/original/ported/sp/oldhuxiao.skill"),
        include_str!("../../../content/original/ported/sp/oldwuji.skill"),
    ]
    .join("\n");
    let package = content_compiler::compile_source_named("old-guanyinping-test", &source).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 279);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_944, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = &players[1..];
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
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.extend(
        [XUEJI_OLD, OLDHUXIAO, OLDWUJI].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 1;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let red = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        0,
    );
    let black_weapon = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        8,
    );
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(XUEJI_OLD))
        .unwrap()
        .id;
    let state = tx.commit().state;

    let (_, result) = activate(state.clone(), owner, skill, targets.to_vec(), red, 1);
    assert!(matches!(result, CommandResult::Rejected { .. }));
    let (_, result) = activate(
        state.clone(),
        owner,
        skill,
        vec![targets[0]],
        black_weapon,
        2,
    );
    assert!(matches!(result, CommandResult::Rejected { .. }));

    let (state, result) = activate(state, owner, skill, vec![targets[0], targets[1]], red, 3);
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.turn.flag(turn_flag::DAMAGE_DEALT), 2);
    assert_eq!(state.hand_count(targets[0]), 1);
    assert_eq!(state.hand_count(targets[1]), 1);
    assert!(state.zones.get(discard).unwrap().cards.contains(&red));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: targets[2],
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (2, 4));
    let defs: Vec<_> = player
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(defs.contains(&XUEJI_OLD));
    assert!(!defs.contains(&OLDHUXIAO));
    assert!(!defs.contains(&OLDWUJI));
}
