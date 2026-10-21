//! Golden coverage for BINGSHI 势于吉「道转/符济」.

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

const DAOZHUAN: u32 = 4_200_000_577;
const FUJI: u32 = 4_200_000_578;
const FUJI_EFFECT: u32 = 4_200_000_579;
const FUJI_SHA: u32 = 4_200_000_580;
const FUJI_SHAN: u32 = 4_200_000_581;
const SOURCE: &str = include_str!("../../../../content/original/ported/bingshi/pot_yuji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 570);
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
    effects.extend([DAOZHUAN, FUJI].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    tx.state_mut().turn.round = 1;
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::Hand | ZoneKind::Equip) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(570_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
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

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == def)
}

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_complete_pot_yuji_at_abi570() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 37);
    assert_eq!(package.skill_index.len(), 105);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_yuji")
            .unwrap()
            .skills,
        [FUJI, DAOZHUAN]
    );
}

#[test]
fn daozhuan_discards_cost_and_enforces_turn_and_round_names() {
    let (state, owner, order) = setup(5700);
    let target = order[1];
    let mut tx = Transaction::begin(state);
    let first = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        1,
        CardKind::Tao,
        Suit::Heart,
    );
    let second = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        2,
        CardKind::Shan,
        Suit::Club,
    );
    let daozhuan = skill(tx.state(), owner, DAOZHUAN);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5701),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::Sha.tag(),
                targets: vec![target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, output) = choose(state, vec![first.raw()], 5702);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(!state.hand_cards(owner).contains(&first));

    let (mut state, repeated) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5703),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    state.skills.get_mut(daozhuan).unwrap().turn_fire_count = 0;
    let (_, repeated_name) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5704),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::ShaFire.tag(),
                targets: vec![target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(
        repeated_name.result,
        CommandResult::Rejected { .. }
    ));
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5705),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let (_, output) = choose(state, vec![second.raw()], 5706);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
}

#[test]
fn fuji_pairs_cards_rewards_minimum_and_tagged_sha_gains_and_deals_extra_damage() {
    let (state, owner, order) = setup(5710);
    let first = order[1];
    let second = order[2];
    let mut tx = Transaction::begin(state);
    let first_gift = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        10,
        CardKind::Sha,
        Suit::Heart,
    );
    let second_gift = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        11,
        CardKind::Tao,
        Suit::Club,
    );
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        12,
        CardKind::Tao,
        Suit::Heart,
    );
    let fuji = skill(tx.state(), owner, FUJI);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5711),
            kind: CommandKind::ActivateSkill {
                skill: fuji,
                targets: vec![first, second],
                card_targets: vec![first_gift, second_gift],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(first_gift).unwrap().owner, first);
    assert_eq!(state.cards.get(second_gift).unwrap().owner, second);
    assert!(state.card_has_content_tag(first_gift, FUJI));
    assert!(state.card_has_content_tag(second_gift, FUJI));
    assert!(has_skill(&state, first, FUJI_EFFECT));
    assert!(has_skill(&state, second, FUJI_EFFECT));
    assert!(has_skill(&state, owner, FUJI_SHA));
    assert!(has_skill(&state, owner, FUJI_SHAN));
    assert_eq!(state.hand_count(owner), 2);

    let mut state = state;
    state.turn.current = first;
    state.turn.phase = Phase::Play;
    let hp = state.players.get(second).unwrap().hp;
    let before_hand = state.hand_count(first);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: first,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5712),
            kind: CommandKind::PlayCard {
                card: first_gift,
                targets: vec![second],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(second).unwrap().hp, hp - 2);
    assert_eq!(state.hand_count(first), before_hand);
    assert!(!state.card_has_content_tag(first_gift, FUJI));
}

#[test]
fn fuji_one_time_shan_reward_draws_then_expires() {
    let (state, owner, _) = setup(5720);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FUJI_SHAN),
        }],
    )
    .unwrap();
    let shan = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        20,
        CardKind::Shan,
        Suit::Diamond,
    );
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        21,
        CardKind::Tao,
        Suit::Diamond,
    );
    let helper = skill(tx.state(), owner, FUJI_SHAN);
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolvePotyuji {
                player: owner,
                target: owner,
                card: shan,
                action: 5,
                cards: vec![shan],
                targets: vec![],
                skill: helper,
            },
            EffectRequest::ResolvePotyuji {
                player: owner,
                target: owner,
                card: shan,
                action: 7,
                cards: vec![shan],
                targets: vec![],
                skill: helper,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
    assert!(!has_skill(tx.state(), owner, FUJI_SHAN));
}
