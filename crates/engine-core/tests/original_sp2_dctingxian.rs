//! Golden coverage for 铤险（sp2 孙狼）摸牌 + 杀目标排除（ABI 464）.
//!
//! 使用【杀】指定目标后（每回合一次）摸装备数+1 张牌，并令此【杀】对至多等量目标无效。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCTINGXIAN: u32 = 3_933_341_702;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/dctingxian.skill");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).expect("target zone");
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .expect("zone")
        .cards
        .push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
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
fn draws_and_excludes_sha_target_from_damage() {
    let package = content_compiler::compile_source_named("dctingxian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 464);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCTINGXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // Strip the target's hand so it cannot respond with a Shan.
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .expect("target hand");
    let target_cards: Vec<_> = tx
        .state()
        .zones
        .get(target_hand)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .expect("discard pile");
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: target_cards,
            to: discard,
            from: Some(target_hand),
        }],
    )
    .unwrap();
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_000);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let owner_hand_before = tx.state().hand_count(owner);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    // Play the Sha: the trigger fires, draws equip+1, and opens the exclusion choice.
    let (state, result) = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected play result: {:?}",
        result.result
    );
    let choice = result.choice.expect("exclusion choice pending");
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(choice.observer, owner);
    let equip_count = 0; // no equipment injected
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before + equip_count,
        "owner draws equipment count + 1 (Sha leaves the hand)"
    );

    // Exclude the target from this Sha.
    let (state, result) = resolve(state, owner, vec![target.raw()], false, 2);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected exclusion result: {:?}",
        result.result
    );
    // The excluded Sha resolves with no response (target has no Shan); any further
    // engine choices are declined so the resolution completes.
    let mut state = state;
    let mut key = 3;
    loop {
        if state.resolution.pending_choice_id == 0 {
            break;
        }
        let (next, result) = resolve(state, target, Vec::new(), true, key);
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "unexpected sha resolution result: {:?}",
            result.result
        );
        state = next;
        key += 1;
        assert!(key < 10, "sha resolution did not settle");
    }
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before,
        "excluded Sha target takes no damage"
    );
}
