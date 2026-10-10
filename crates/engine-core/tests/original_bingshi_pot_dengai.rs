//! Golden coverage for BINGSHI 势邓艾「屯田」「凿险」「急袭」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTTUNTIAN: u32 = 2_435_155_097;
const POTZAOXIAN: u32 = 2_551_307_508;
const POTJIXI: u32 = 4_126_984_108;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(defs: &[u32], seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
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
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(defs.iter().map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(543_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn charge(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == POTTUNTIAN)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_dengai_at_abi543() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_dengai")
            .unwrap()
            .skills,
        [POTTUNTIAN, POTZAOXIAN, POTJIXI]
    );
}

#[test]
fn tuntian_grows_charges_spends_and_applies_zaoxian() {
    let (state, owner, order) = setup(&[POTTUNTIAN, POTZAOXIAN], 5431);
    let targets: Vec<_> = order
        .into_iter()
        .filter(|target| *target != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    let tuntian = skill(tx.state(), owner, POTTUNTIAN);
    let zaoxian = skill(tx.state(), owner, POTZAOXIAN);
    let before_draw = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePottuntian {
            player: owner,
            action: 1,
            amount: 0,
            targets: Vec::new(),
            zaoxian_def: SkillDefId::new(POTZAOXIAN),
            skill: tuntian,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before_draw + 1);
    for action in [0, 1, 0] {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottuntian {
                player: owner,
                action,
                amount: 0,
                targets: Vec::new(),
                zaoxian_def: SkillDefId::new(POTZAOXIAN),
                skill: tuntian,
            }],
        )
        .unwrap();
    }
    assert_eq!(charge(tx.state(), owner), 2);
    let discard_owner = PlayerId::NONE;
    inject(
        &mut tx,
        discard_owner,
        ZoneKind::DiscardPile,
        CardKind::TrickWuzhong,
        Suit::Spade,
        1,
    );
    for (index, target) in targets.iter().enumerate() {
        inject(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Other,
            Suit::Heart,
            10 + index as u32,
        );
        assert!(tx.state().players.get(*target).unwrap().alive);
    }
    let target_counts: Vec<_> = targets
        .iter()
        .map(|target| tx.state().hand_count(*target))
        .collect();
    let wuzhong_before = tx
        .state()
        .hand_cards(owner)
        .into_iter()
        .filter(|card| tx.state().cards.get(*card).unwrap().kind == CardKind::TrickWuzhong)
        .count();
    assert!(tx.state().skills.get(zaoxian).is_some());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottuntian {
                player: owner,
                action: 2,
                amount: 0,
                targets: targets.clone(),
                zaoxian_def: SkillDefId::new(POTZAOXIAN),
                skill: tuntian,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![2], false, 5432);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(charge(&state, owner), 0);
    for (target, before) in targets.iter().zip(target_counts) {
        assert_eq!(state.hand_count(*target), before + 1);
    }
    assert_eq!(
        state
            .hand_cards(owner)
            .into_iter()
            .filter(|card| state.cards.get(*card).unwrap().kind == CardKind::TrickWuzhong)
            .count(),
        wuzhong_before + 1
    );
}

#[test]
fn jixi_records_targets_discards_then_starts_multi_target_shunshou() {
    let (state, owner, order) = setup(&[POTJIXI], 5433);
    let targets: Vec<_> = order
        .into_iter()
        .filter(|target| *target != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    let jixi = skill(tx.state(), owner, POTJIXI);
    let discard = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Club,
        20,
    );
    for (index, target) in targets.iter().enumerate() {
        inject(
            &mut tx,
            *target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            30 + index as u32,
        );
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjixi {
                player: owner,
                target: *target,
                action: 1,
                targets: Vec::new(),
                skill: jixi,
            }],
        )
        .unwrap();
    }
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjixi {
                player: owner,
                target: owner,
                action: 2,
                targets: Vec::new(),
                skill: jixi,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![discard.raw()], false, 5434);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, output) = resolve(
        state,
        targets.iter().map(|target| target.raw()).collect(),
        false,
        5435,
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(
        state.has_pending_choice(),
        "虚拟顺手牵羊应进入独立无懈/获得结算队列"
    );
}
