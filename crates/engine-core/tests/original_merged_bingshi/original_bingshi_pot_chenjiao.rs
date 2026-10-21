//! Golden coverage for BINGSHI 势陈矫「清严」「策断」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTQINGYAN: u32 = 4_232_147_498;
const POTCEDUAN: u32 = 4_257_587_443;
const SOURCE: &str = include_str!("../../../../content/original/ported/bingshi/pot_chenjiao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(def: u32, seed: u64, players: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: players,
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects = tx
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
    range: u8,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(544_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: range,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(state: GameState, selection: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    submit(
        state.clone(),
        actor,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections: selection,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_chenjiao_at_abi544() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 37);
    assert_eq!(package.skill_index.len(), 105);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_chenjiao")
            .unwrap()
            .skills,
        [POTQINGYAN, POTCEDUAN]
    );
}

#[test]
fn qingyan_keeps_shown_cards_locks_then_increases_round_cost() {
    let (state, _first, order) = setup(POTQINGYAN, 5441, 3);
    let attacker = order[0];
    let owner = order[1];
    let mut tx = Transaction::begin(state);
    let old_skill = skill(tx.state(), attacker, POTQINGYAN);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: old_skill },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(POTQINGYAN),
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let attack = inject(
        &mut tx,
        attacker,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        0,
        1,
    );
    for serial in 2..=4 {
        inject(
            &mut tx,
            owner,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            0,
            serial,
        );
    }
    let state = tx.commit().state;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        5442,
    );
    let hand: std::collections::BTreeSet<_> = state
        .hand_cards(owner)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !hand.contains(token) && !matches!(*token, 0 | 1))
        .unwrap();
    let (state, sources) = resolve(state, vec![conversion], 5443);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let choice = sources.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let shown = choice.options[0].token;
    let (state, result) = resolve(state, vec![shown], 5444);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&CardId::new(shown)));

    let mut tx = Transaction::begin(state);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![CardId::new(shown)],
            to: discard,
            from: hand,
        }],
    )
    .unwrap();
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let attack = inject(
        &mut tx,
        attacker,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        0,
        5,
    );
    let (state, response) = submit(
        tx.commit().state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        5445,
    );
    let hand: std::collections::BTreeSet<_> = state
        .hand_cards(owner)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !hand.contains(token) && !matches!(*token, 0 | 1))
        .unwrap();
    let (state, sources) = resolve(state, vec![conversion], 5446);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    assert_eq!(
        (
            sources.choice.as_ref().unwrap().min_count,
            sources.choice.unwrap().max_count
        ),
        (2, 2)
    );
}

#[test]
fn ceduan_reveals_range_players_uses_all_majority_cards_and_draws_on_damage() {
    let (state, owner, order) = setup(POTCEDUAN, 5447, 4);
    let target = order[2];
    let mut tx = Transaction::begin(state);
    inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        3,
        10,
    );
    for serial in 11..=12 {
        inject(
            &mut tx,
            owner,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            0,
            serial,
        );
    }
    inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Club,
        0,
        13,
    );
    for (index, candidate) in order
        .iter()
        .copied()
        .filter(|candidate| *candidate != owner)
        .enumerate()
    {
        inject(
            &mut tx,
            candidate,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            0,
            20 + index as u32,
        );
    }
    let ceduan = skill(tx.state(), owner, POTCEDUAN);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (mut state, mut output) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill: ceduan,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        5448,
    );
    let mut key = 5449;
    while state.resolution.continuation_kind == ContinuationKind::Custom {
        let choice = output.choice.take().unwrap();
        let selected = choice
            .options
            .iter()
            .map(|option| CardId::new(option.token))
            .find(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| matches!(card.suit, Suit::Heart | Suit::Diamond))
            })
            .unwrap();
        (state, output) = resolve(state, vec![selected.raw()], key);
        key += 1;
    }
    while state.has_pending_choice() {
        let choice = output.choice.take().unwrap();
        let pass = choice
            .options
            .iter()
            .find(|option| option.token == 0)
            .map_or(choice.options[0].token, |option| option.token);
        (state, output) = resolve(state, vec![pass], key);
        key += 1;
    }
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.hand_count(owner),
        2,
        "两张红牌作为杀，黑牌保留，造成伤害后摸一"
    );
}
