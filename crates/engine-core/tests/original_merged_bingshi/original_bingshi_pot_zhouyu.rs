//! Golden coverage for BINGSHI 势周瑜「炽沄/焰洄/焚涛/雄姿」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CHIYUN: u32 = 4_200_000_566;
const YANHUI: u32 = 4_200_000_567;
const FENTAO: u32 = 4_200_000_568;
const XIONGZI: u32 = 4_200_000_569;
const SOURCE: &str = include_str!("../../../../content/original/ported/bingshi/pot_zhouyu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 561);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [CHIYUN, YANHUI, FENTAO, XIONGZI].map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, serial: u32, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(561_000 + serial),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn request(
    player: PlayerId,
    target: PlayerId,
    action: u8,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolvePotzhouyu {
        player,
        target,
        card: CardId::NONE,
        action,
        card_flags: 0,
        targets,
        skill,
    }
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_pot_zhouyu_at_abi561() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 37);
    assert_eq!(package.skill_index.len(), 105);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_zhouyu")
            .unwrap()
            .skills,
        [CHIYUN, YANHUI, FENTAO, XIONGZI]
    );
}

#[test]
fn chiyun_gives_cards_then_draws_two_and_links_target_once_per_phase() {
    let (state, owner, order) = setup(5611);
    let target = order[1];
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner, 1, Suit::Heart);
    let chiyun = skill(tx.state(), owner, CHIYUN);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 0, Vec::new(), chiyun)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![card.raw()], false, 5612);
    let (state, _) = resolve(state, vec![target.raw()], false, 5613);
    let (state, result) = resolve(state, vec![2], false, 5614);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == engine_core::content::MARK_CHAIN && *count > 0 }));

    let mut tx = Transaction::begin(state);
    inject_hand(&mut tx, owner, 2, Suit::Spade);
    run_effects(&mut tx, vec![request(owner, owner, 0, Vec::new(), chiyun)]).unwrap();
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}

#[test]
fn yanhui_phase_damage_is_limited_to_a_role_whose_card_was_discarded() {
    let (state, owner, order) = setup(5615);
    let shown = order[1];
    let other = order[2];
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, shown, 3, Suit::Club);
    let yanhui = skill(tx.state(), owner, YANHUI);

    for key in [5616, 5618] {
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![request(owner, shown, 1, vec![shown, other], yanhui)]
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let (state, _) = resolve(tx.commit().state, vec![shown.raw()], false, key);
        let (state, _) = resolve(state, vec![card.raw()], false, key + 1);
        tx = Transaction::begin(state);
    }
    assert!(!tx.state().hand_cards(shown).contains(&card));
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 2, Vec::new(), yanhui)]),
        Err(EffectError::ChoiceRequested)
    ));
    let pending = tx.commit().state;
    let (_, rejected) = resolve(pending.clone(), vec![other.raw()], false, 5620);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    let hp_before = pending.players.get(shown).unwrap().hp;
    let (state, accepted) = resolve(pending, vec![shown.raw()], false, 5621);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(shown).unwrap().hp, hp_before - 1);
}

#[test]
fn xiongzi_keeps_one_branch_for_all_three_skills_and_draws_two() {
    let (state, owner, _) = setup(5622);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let xiongzi = skill(tx.state(), owner, XIONGZI);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 6, Vec::new(), xiongzi)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![1], false, 5623);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.skills.get(xiongzi).unwrap().lifetime_fire_count, 1);
    for def in [CHIYUN, YANHUI, FENTAO] {
        assert_eq!(
            state
                .skills
                .get(skill(&state, owner, def))
                .unwrap()
                .content_flags
                & 4,
            4
        );
    }
}

#[test]
fn fentao_discard_branch_relinks_after_the_fire_damage() {
    let (state, owner, order) = setup(5624);
    let target = order[1];
    let mut tx = Transaction::begin(state);
    let first = inject_hand(&mut tx, target, 4, Suit::Diamond);
    inject_hand(&mut tx, target, 5, Suit::Spade);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
    let fentao = skill(tx.state(), owner, FENTAO);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, target, 3, Vec::new(), fentao)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![first.raw()], false, 5625);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .marks
        .retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    run_effects(&mut tx, vec![request(owner, target, 4, Vec::new(), fentao)]).unwrap();
    assert!(tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == engine_core::content::MARK_CHAIN && *count > 0 }));
}

#[test]
fn pending_choice_rejects_after_the_originating_skill_is_removed() {
    let (state, owner, order) = setup(5626);
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner, 6, Suit::Heart);
    let chiyun = skill(tx.state(), owner, CHIYUN);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 0, Vec::new(), chiyun)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![card.raw()], false, 5627);
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.remove(chiyun);
    let (_, result) = resolve(tx.commit().state, vec![order[1].raw()], false, 5628);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}
