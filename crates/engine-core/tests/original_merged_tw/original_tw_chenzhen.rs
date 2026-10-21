//! Golden coverage for TW 陈震「睦约/察异」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWMUYUE: u32 = 2_544_676_263;
const TWMUYUE_EFFECT: u32 = 3_469_371_151;
const TWCHAYI: u32 = 3_796_108_342;
const TWCHAYI_EFFECT: u32 = 3_469_371_153;
const TWCHAYI_RE: u32 = 3_469_371_154;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twchenzhen.skill");

fn inject(
    tx: &mut Transaction,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_378_000 + u32::from(rank)),
        kind,
        suit: Suit::Diamond,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn muyue_matching_discard_grants_free_use_and_chayi_switches_branch() {
    let package = content_compiler::compile_source_named("tw-chenzhen-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 378);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_119, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([TWMUYUE, TWCHAYI].map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();

    let cost = inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 1);
    let reward = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Shan,
        2,
    );
    let muyue = skill(&tx, owner, TWMUYUE);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwchenzhen {
                player: owner,
                target: PlayerId::NONE,
                helper_skill_def: TWMUYUE_EFFECT,
                record_skill_def: 0,
                action: 0,
                stage: 2,
                card_kind: CardKind::Shan.tag(),
                card: cost,
                branch: 0,
                skill: muyue,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], 1);
    assert!(state.hand_cards(target).contains(&reward));
    let mut tx = Transaction::begin(state);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|id| {
            tx.state()
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == TWMUYUE_EFFECT)
        }));

    let chayi = skill(&tx, owner, TWCHAYI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwchenzhen {
                player: owner,
                target: owner,
                helper_skill_def: TWCHAYI_EFFECT,
                record_skill_def: TWCHAYI_RE,
                action: 1,
                stage: 0,
                card_kind: 0,
                card: CardId::NONE,
                branch: 0,
                skill: chayi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], 2);
    let state = choose(state, target, vec![0], 3);
    let mut tx = Transaction::begin(state);
    inject(&mut tx, ZoneKind::Hand, target, CardKind::Sha, 3);
    let record = skill(&tx, target, TWCHAYI_RE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwchenzhen {
            player: target,
            target,
            helper_skill_def: TWCHAYI_EFFECT,
            record_skill_def: 0,
            action: 3,
            stage: 0,
            card_kind: 0,
            card: CardId::NONE,
            branch: 0,
            skill: record,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .contains(&(TWCHAYI_EFFECT, 1)));
    assert!(tx.state().skills.get(record).is_none());
}
