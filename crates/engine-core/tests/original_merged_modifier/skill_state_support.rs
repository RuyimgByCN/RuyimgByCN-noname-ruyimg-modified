use super::*;

pub(super) fn yijiang_source() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/yijiang");
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "skill"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect()
}

pub(super) fn loaded(package: &[u8], seed: u64) -> (Transaction, PlayerId) {
    loaded_players(package, seed, 2)
}

pub(super) fn loaded_players(
    package: &[u8],
    seed: u64,
    player_count: u8,
) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    (Transaction::begin(engine.state().clone()), owner)
}

pub(super) fn strip_skills(tx: &mut Transaction, owner: PlayerId) {
    let skills = tx.state().players.get(owner).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

pub(super) fn add_skill(tx: &mut Transaction, owner: PlayerId, skill: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        }],
    )
    .unwrap();
}

pub(super) fn add_mark(tx: &mut Transaction, owner: PlayerId, mark: u32, count: i32) {
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(mark),
            delta: count,
        }],
    )
    .unwrap();
}

pub(super) fn hand_limit(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

pub(super) fn attack_range(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

pub(super) fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

pub(super) fn equip_defensive_horse(tx: &mut Transaction, owner: PlayerId) -> CardId {
    equip_mount(tx, owner, CardKind::HorseDef)
}

pub(super) fn equip_mount(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> CardId {
    assert!(matches!(kind, CardKind::HorseOff | CardKind::HorseDef));
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_999),
        kind,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

pub(super) fn place_rank_thirteen_field_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
) -> CardId {
    assert!(matches!(zone_kind, ZoneKind::Equip | ZoneKind::Judge));
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let kind = if zone_kind == ZoneKind::Equip {
        CardKind::Weapon
    } else {
        CardKind::TrickLe
    };
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_998),
        kind,
        suit: Suit::Club,
        rank: 13,
        weapon_range: u8::from(kind == CardKind::Weapon),
        zone,
        owner,
        face_up: true,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

pub(super) fn assign_four_groups(tx: &mut Transaction, owner: PlayerId) -> Vec<PlayerId> {
    let mut players = vec![owner];
    players.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != owner),
    );
    assert_eq!(players.len(), 4);
    for (player, character) in players.iter().zip([1, 8, 15, 23]) {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .character_def = character;
    }
    players
}
