//! Golden for XIANDING 朱佩兰【辞虑/痛悼】（ABI 600）。

use content_schema::PackageCharacterEntry;
use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, PendingCardResolution, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCCILV: u32 = 3_468_569_693;
const DCCILV_EFFECT: u32 = 3_697_675_750;
const DCTONGDAO: u32 = 3_625_586_472;
const BASE: u32 = 90_600_001;
const ACQUIRED: u32 = 90_600_002;
const INTERNAL: u32 = 90_600_003;
const CHARACTER: u32 = 90_600_004;
const SOURCE: &str = include_str!("../../../../content/original/ported/xianding/zhupeilan.skill");

fn setup() -> (
    GameState,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
    engine_api::SkillId,
) {
    let mut package = content_compiler::compile_source_named(
        "zhupeilan-test",
        &format!(
            "{SOURCE}\nskill base({BASE}) {{}}\nskill acquired({ACQUIRED}) {{}}\nskill internal({INTERNAL}) non_gainable {{}}"
        ),
    )
    .unwrap();
    package.character_index.push(PackageCharacterEntry {
        id: CHARACTER,
        name_key: "tongdao_target".into(),
        max_hp: 3,
        initial_hp: 0,
        skills: vec![BASE],
        sex: "male".into(),
        group: "wu".into(),
        is_lord: false,
        clans: vec![],
        tags: Vec::new(),
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: Vec::new(),
    });
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            effects.push(EffectRequest::DiscardCards {
                player: *player,
                cards,
            });
        }
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCCILV),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCTONGDAO),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(BASE),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(ACQUIRED),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(INTERNAL),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = CHARACTER;
    let find = |def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let cilv = find(DCCILV);
    let tongdao = find(DCTONGDAO);
    (tx.commit().state, owner, target, cilv, tongdao)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    rank: u8,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap_or_else(|| {
        tx.state()
            .zones
            .find(zone, PlayerId::NONE)
            .expect("shared zone")
    });
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_900 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn arm_trick(
    tx: &mut Transaction,
    source: PlayerId,
    owner: PlayerId,
    card: CardId,
    materials: Vec<CardId>,
) {
    tx.state_mut()
        .resolution
        .pending_card_resolutions
        .push(PendingCardResolution {
            source,
            card,
            kind: CardKind::TrickJuedou.tag(),
            targets: vec![owner],
            damaged_targets: Vec::new(),
            source_was_hand: true,
            all_sources_were_hand: true,
            response_color_lock_mask: 0,
            source_cards: materials,
            originating_view_as_skill_def: 0,
            skill_excluded_targets: Vec::new(),
        });
}

#[test]
fn cilv_draws_by_remaining_options_then_binds_prevent_and_gain_to_exact_card() {
    let (state, owner, source, cilv, _) = setup();
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    alloc(&mut tx, owner, ZoneKind::Hand, 2, CardKind::Other);
    for rank in 3..=5 {
        alloc(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            rank,
            CardKind::Other,
        );
    }
    let trick = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        6,
        CardKind::TrickJuedou,
    );
    arm_trick(&mut tx, source, owner, trick, vec![trick]);
    // The current port auto-resolves the 赐赂 draw (3 - used categories);
    // the optional acceptance lives at the trigger level, not in action 0.
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 0,
            event_cards: vec![trick],
            skill: cilv,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 5);
    assert_eq!(tx.state().resolution.pending_choice_id, 0);

    let (state, owner, source, cilv, _) = setup();
    let mut tx = Transaction::begin(state);
    let trick = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        6,
        CardKind::TrickJuedou,
    );
    arm_trick(&mut tx, source, owner, trick, vec![trick]);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 4,
            event_cards: vec![trick],
            skill: cilv,
        }],
    )
    .unwrap();
    let helper = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCCILV_EFFECT)
        .unwrap();
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill: cilv }]).unwrap();
    tx.state_mut().resolution.current_damage_id = 77;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 1,
            event_cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().resolution.prevented_damage_ids.contains(&77));
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 2,
            event_cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();

    let second = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        7,
        CardKind::TrickHuogong,
    );
    let extra = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        8,
        CardKind::Other,
    );
    arm_trick(&mut tx, source, owner, second, vec![second, extra]);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCCILV),
        }],
    )
    .unwrap();
    let cilv = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCCILV)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: second,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 5,
            event_cards: vec![second, extra],
            skill: cilv,
        }],
    )
    .unwrap();
    let helper = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCCILV_EFFECT)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: second,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 2,
            event_cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).contains(&second));
    assert!(tx.state().hand_cards(owner).contains(&extra));
}

#[test]
fn tongdao_replaces_non_internal_skills_with_fresh_starting_skills_and_recovers_owner() {
    let (state, owner, target, _, tongdao) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    tx.state_mut().players.get_mut(owner).unwrap().dying = true;
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    tx.state_mut()
        .skills
        .get_mut(tongdao)
        .unwrap()
        .lifetime_fire_count = 1;
    let before = tx.state().players.get(target).unwrap().skills.clone();
    let internal = before
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == INTERNAL)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target,
            card: CardId::NONE,
            helper_skill_def: SkillDefId::NONE,
            action: 11,
            event_cards: Vec::new(),
            skill: tongdao,
        }],
    )
    .unwrap();
    let skills = tx.state().players.get(target).unwrap().skills.clone();
    assert!(skills.contains(&internal));
    assert!(skills.iter().any(|skill| {
        tx.state().skills.get(*skill).unwrap().def.raw() == BASE && !before.contains(skill)
    }));
    assert!(!skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ACQUIRED));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}
