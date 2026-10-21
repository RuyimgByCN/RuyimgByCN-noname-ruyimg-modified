//! Golden for xianding 朝献 dchaoxian (ABI 747, tag 764):
//! 限定技（once_per_game）——弃牌堆全部 3 点牌洗入牌堆底；收齐其他角色
//! 手牌中的 3 点牌。active 全管道驱动。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCHAOXIAN: u32 = 3_403_528_397;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
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

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, rank: u8, tag: u32) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(101_000 + tag),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_dchaoxian_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCHAOXIAN)
        .expect("dchaoxian active entry");
    assert!(active.once_per_game);
}

#[test]
fn dchaoxian_shuffles_threes_to_deck_bottom_and_collects_from_others() {
    let (mut state, players) = started(2_026_103_001);
    let owner = players[0];
    let other1 = players[1];
    let other2 = players[2];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, DCHAOXIAN);
    // 弃牌堆：两张 3 点 + 一张干扰。
    let d3a = inject(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, 3, 1);
    let d3b = inject(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, 3, 2);
    let _d5 = inject(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, 5, 3);
    // 其他角色手牌：other1 一张 3 点；other2 无。
    let h3 = inject(&mut state, other1, ZoneKind::Hand, 3, 4);
    // owner 自身手牌的 3 点不收（上游只收其他角色）——预置干扰。
    let _own3 = inject(&mut state, owner, ZoneKind::Hand, 3, 5);
    let owner_before = state.hand_count(owner);
    let draw_before = {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        state.zones.get(draw).unwrap().cards.len()
    };
    // bootstrap 弃牌堆可能自带 3 点牌残留——预记数量供动态断言。
    let pre_threes = {
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .iter()
            .filter(|card| state.cards.get(**card).unwrap().rank == 3)
            .count()
    };

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));

    // 弃牌堆 3 点牌全部进牌堆（bootstrap 弃牌堆可能含残留 3 点牌，
    // 一并收走——动态断言总数与「弃牌堆无 3 点牌」）。
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = state.zones.get(draw).unwrap().cards.clone();
    assert!(draw_cards.contains(&d3a) && draw_cards.contains(&d3b));
    // pre_threes 测量于注入之后，已含注入的两张。
    assert_eq!(draw_cards.len(), draw_before + pre_threes);
    // 注入的两张在牌堆底部区域（头部（2+残留）张内）。
    assert!(draw_cards[..2 + pre_threes].contains(&d3a));
    assert!(draw_cards[..2 + pre_threes].contains(&d3b));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let discard_cards = state.zones.get(discard).unwrap().cards.clone();
    assert!(discard_cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().rank != 3));
    // other1 的 3 点被收走；owner 手牌 +1（净 0：自持 3 点未动）。
    assert!(!state.hand_cards(other1).contains(&h3));
    assert!(state.hand_cards(owner).contains(&h3));
    assert_eq!(state.hand_count(owner), owner_before + 1);
    assert_eq!(state.hand_count(other2), 0);
    let _ = draw_before;

    // once_per_game：二次发动被拒。
    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
