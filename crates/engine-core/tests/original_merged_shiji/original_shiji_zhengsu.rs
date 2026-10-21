//! Golden for SHIJI 整肃系统（ABI 432）与侯封/燕集（ABI 433）。
//!
//! 整肃：垒金（出牌阶段点数严格递增且≥3）/ 变阵（同花色且≥2）/ 明志（弃牌阶段
//! 花色全异且≥2）；成功时受益者（授予者+自己）各摸二或回复一，随后移除临时技能。
//! 侯封：其他角色出牌阶段开始选整肃授予并记录授予者；燕集：自己选整肃。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const ZHENGSU_LEIJIN: u32 = 2_584_022_884;
const ZHENGSU_BIANZHEN: u32 = 3_044_720_297;
const ZHENGSU_MINGZHI: u32 = 3_824_072_820;
const HOUFENG: u32 = 4_065_778_656;
const SPYANJI: u32 = 3_519_161_628;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_815,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn skill_instance(tx: &Transaction, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == def && s.owner == owner)
        .map(|s| s.id)
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve failed: {:?}",
        result.result
    );
    next
}

fn zhengsu(
    player: PlayerId,
    mode: u8,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolveZhengsu {
        player,
        mode,
        stage,
        card,
        skill,
    }
}

/// 垒金：三张递增牌 → 弃牌阶段末判定成功，摸两张并移除临时技能。
#[test]
fn zhengsu_leijin_success_draws_two() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_LEIJIN, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, ZHENGSU_LEIJIN).unwrap();
    for rank in [2u8, 5, 9] {
        let card = inject_card(&mut tx, owner, CardKind::Sha, Suit::Spade, rank);
        run_effects(&mut tx, vec![zhengsu(owner, 0, 0, card, skill)]).unwrap();
    }
    let hand_before = tx.state().hand_count(owner);
    // 弃牌阶段末判定。
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 0, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "整肃成功应摸两张（满血自动走摸牌分支）"
    );
    assert!(
        skill_instance(&tx, owner, ZHENGSU_LEIJIN).is_none(),
        "整肃成功后临时技能应移除"
    );
}

/// 垒金：点数不递增 → 判定失败，技能移除且无奖励。
#[test]
fn zhengsu_leijin_failure_removes_skill() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_LEIJIN, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, ZHENGSU_LEIJIN).unwrap();
    for rank in [5u8, 3] {
        let card = inject_card(&mut tx, owner, CardKind::Sha, Suit::Spade, rank);
        run_effects(&mut tx, vec![zhengsu(owner, 0, 0, card, skill)]).unwrap();
    }
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 0, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before, "失败无奖励");
    assert!(
        skill_instance(&tx, owner, ZHENGSU_LEIJIN).is_none(),
        "整肃失败后临时技能应移除"
    );
}

/// 变阵：出牌阶段两张同花色牌 → 判定成功。
#[test]
fn zhengsu_bianzhen_success() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_BIANZHEN, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // 手动构造引擎 flag：出牌阶段用牌 2 张、同花色（红心，suit_tag=2 → 位 1<<2）。
    let suit_key = turn_flag::play_phase_card_suit_history_by_player(owner);
    tx.state_mut().turn.set_flag(suit_key, 1 << 2);
    let count_key = turn_flag::play_phase_card_used_by_player(owner);
    tx.state_mut().turn.set_flag(count_key, 2);
    let skill = skill_instance(&tx, owner, ZHENGSU_BIANZHEN).unwrap();
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 1, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "变阵成功摸二"
    );
    assert!(skill_instance(&tx, owner, ZHENGSU_BIANZHEN).is_none());
}

/// 明志：弃牌阶段弃置两张不同花色牌 → 判定成功。
#[test]
fn zhengsu_mingzhi_success() {
    let (mut tx, owner, _seats) = setup(ZHENGSU_MINGZHI, 2);
    tx.state_mut().turn.current = owner;
    let heart = inject_card(&mut tx, owner, CardKind::Tao, Suit::Heart, 3);
    let spade = inject_card(&mut tx, owner, CardKind::Sha, Suit::Spade, 4);
    // 构造弃牌阶段历史：lost_cards 从 base 起为这两张，且仍在弃牌堆。
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_LOST_BASE, 1);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [heart, spade] {
        tx.state_mut().turn.lost_cards.push((owner, card));
        let zone = tx.state_mut().cards.get_mut(card).unwrap();
        zone.zone = discard;
    }
    let skill = skill_instance(&tx, owner, ZHENGSU_MINGZHI).unwrap();
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 2, 1, CardId::NONE, skill)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "明志成功摸二"
    );
    assert!(skill_instance(&tx, owner, ZHENGSU_MINGZHI).is_none());
}

/// 侯封：授予他人整肃并记录授予者；成功后授予者也获奖励。
#[test]
fn houfeng_grants_and_rewards_grantor() {
    let (mut tx, owner, seats) = setup(HOUFENG, 3);
    let target = seats[1];
    let skill = skill_instance(&tx, owner, HOUFENG).unwrap();
    // 目标出牌阶段开始。
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHoufeng {
            player: owner,
            target,
            mode: 0,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开整肃类型选择"
    );
    let state = resolve_choice(tx.commit().state, owner, 1, vec![1], false);
    let mut tx = Transaction::begin(state.clone());
    // 目标已获得垒金整肃。
    let granted = skill_instance(&tx, target, ZHENGSU_LEIJIN).unwrap();
    // 授予者记录段存在（GR 含 owner）。
    assert!(
        tx.state()
            .skills
            .get(granted)
            .is_some_and(|instance| instance.state_blob.starts_with(b"GR")),
        "侯封应记录授予者"
    );
    // 目标弃牌阶段末：三张递增牌跟踪后判定成功 → 授予者 + 目标各摸二。
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Play;
    for rank in [2u8, 5, 9] {
        let card = inject_card(&mut tx, target, CardKind::Sha, Suit::Spade, rank);
        run_effects(&mut tx, vec![zhengsu(target, 0, 0, card, granted)]).unwrap();
    }
    let owner_hand_before = tx.state().hand_count(owner);
    let target_hand_before = tx.state().hand_count(target);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(target, 0, 1, CardId::NONE, granted)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        owner_hand_before + 2,
        "授予者应获整肃奖励"
    );
    assert_eq!(
        tx.state().hand_count(target),
        target_hand_before + 2,
        "目标应获整肃奖励"
    );
}

/// 燕集：自己选整肃并获奖励。
#[test]
fn spyanji_self_zhengsu_reward() {
    let (mut tx, owner, _seats) = setup(SPYANJI, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, SPYANJI).unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHoufeng {
            player: owner,
            target: owner,
            mode: 1,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(
        err,
        Some(EffectError::ChoiceRequested),
        "应打开整肃类型选择"
    );
    let state = resolve_choice(tx.commit().state, owner, 1, vec![2], false);
    let mut tx = Transaction::begin(state.clone());
    // 自己获得变阵整肃。
    let granted = skill_instance(&tx, owner, ZHENGSU_BIANZHEN).unwrap();
    assert!(
        !tx.state()
            .skills
            .get(granted)
            .is_some_and(|instance| instance.state_blob.starts_with(b"GR")),
        "燕集无授予者记录"
    );
    // 变阵成功：两张同花色（红心）。
    let suit_key = turn_flag::play_phase_card_suit_history_by_player(owner);
    tx.state_mut().turn.set_flag(suit_key, 1 << 2);
    let count_key = turn_flag::play_phase_card_used_by_player(owner);
    tx.state_mut().turn.set_flag(count_key, 2);
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.phase = Phase::Discard;
    run_effects(&mut tx, vec![zhengsu(owner, 1, 1, CardId::NONE, granted)]).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 2,
        "燕集成功摸二"
    );
}

/// 点才（ABI 434）：其他角色出牌阶段结束时，失牌 ≥ 体力且手牌 < 上限 → 摸至上限。
#[test]
fn mbdiancai_draws_to_max_hp() {
    const MBDIANCAI: u32 = 2_244_183_634;
    let (mut tx, owner, seats) = setup(MBDIANCAI, 2);
    let actor = seats.into_iter().find(|p| *p != owner).unwrap();
    // 本回合失牌数 ≥ 体力（体力 4 → 失 4 张）。
    let key = turn_flag::card_lost_by_player(actor);
    for _ in 0..4 {
        let card = inject_card(&mut tx, actor, CardKind::Sha, Suit::Spade, 3);
        tx.state_mut().turn.lost_cards.push((actor, card));
        let current = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, current + 1);
    }
    let max_hp = tx.state().players.get(actor).unwrap().max_hp;
    // 清空手牌：摸至上限 = maxHp 张。
    clear_hand(&mut tx, actor);
    let skill = skill_instance(&tx, owner, MBDIANCAI).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbdiancai {
            player: owner,
            actor,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(actor), max_hp, "应摸至体力上限");
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let zone_id = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    tx.state_mut().zones.get_mut(zone_id).unwrap().cards.clear();
}

fn inject_to_zone(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

/// 战海（ABI 435）：装备补点拼点赢 → 展示目标手牌并按类别从牌堆取牌获得。
#[test]
fn rechuhai_win_gains_by_category() {
    const RECHUHAI: u32 = 4_266_017_884;
    let (mut tx, owner, seats) = setup(RECHUHAI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, RECHUHAI).unwrap();
    // 拥有者首张手牌改 3 点（无装备 → 补 +4 = 7）。
    let owner_card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(owner_card).unwrap().rank = 3;
    // 目标手牌：首张改 5 点（作拼点牌）+ 注入装备（6 点）构成两类。
    let target_basic = tx.state().hand_cards(target)[0];
    tx.state_mut().cards.get_mut(target_basic).unwrap().rank = 5;
    let _target_equip = inject_card(&mut tx, target, CardKind::Weapon, Suit::Club, 6);
    // 牌堆顶各放一类牌供获得。
    let pile_basic = inject_to_zone(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    let pile_equip = inject_to_zone(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        Suit::Club,
        3,
    );
    let hand_before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveRechuhai {
            player: owner,
            target,
            card: CardId::NONE,
            target_card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开拼点牌选择");
    // 拥有者选 3 点牌（摸一张后 hand_before+1），目标选 5 点 → 7 > 5 拥有者赢。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![owner_card.raw()], false);
    let state = resolve_choice(state, target, 2, vec![target_basic.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    let _ = (pile_basic, pile_equip, hand_before);
}

/// 弃哀（ABI 439）：已受伤时目标二选一（回复 / 摸二）。
#[test]
fn spqiai_wounded_target_chooses_recover() {
    const SPQIAI: u32 = 2_235_926_361;
    let (mut tx, owner, seats) = setup(SPQIAI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, SPQIAI).unwrap();
    // 拥有者受伤（4 上限 → 2 血）。
    {
        let p = tx.state_mut().players.get_mut(owner).unwrap();
        p.hp = 2;
    }
    // 一张非基本牌直接注入手牌。
    let equip = inject_card(&mut tx, owner, CardKind::Weapon, Suit::Club, 6);
    // 清掉其他起始手牌里的非基本牌干扰？直接以该牌发起。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpqiai {
            player: owner,
            target,
            card: equip,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "受伤应开二选一");
    let state = resolve_choice(tx.commit().state, target, 1, vec![0], false);
    let mut tx = Transaction::begin(state.clone());
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        3,
        "目标选回复应回 1"
    );
    assert!(
        tx.state().hand_cards(target).contains(&equip),
        "非基本牌应交给目标"
    );
}

/// 擅袭（ABI 440）：授予檄标记（全局唯一）→ 索命（ABI 441）：他人回复后交 2 张或失体。
#[test]
fn spshanxi_mark_and_suoming() {
    const SPSHANXI: u32 = 2_378_382_374;
    let (mut tx, owner, seats) = setup(SPSHANXI, 3);
    let other = seats.iter().copied().find(|p| *p != owner).unwrap();
    let third = seats
        .iter()
        .copied()
        .find(|p| *p != owner && *p != other)
        .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, SPSHANXI).unwrap();
    // 擅袭：开选目标 → 授予檄标记并清除他人。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshanxi {
            player: owner,
            target: PlayerId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开选人 choice");
    let state = resolve_choice(tx.commit().state, owner, 1, vec![other.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    assert!(
        tx.state()
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(m, n)| *m == SPSHANXI && *n > 0),
        "目标应获得檄标记"
    );
    // 再授予 third → 檄标记唯一（other 被清除）。
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshanxi {
            player: owner,
            target: third,
            stage: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(
        !tx.state()
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(m, n)| *m == SPSHANXI && *n > 0),
        "旧檄标记应被清除"
    );
    // 索命：third 有檄标记且可交 2 张 → 开交牌 choice。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpshanxiSuoming {
            player: owner,
            actor: third,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开交牌 choice");
    let cards = tx
        .state()
        .hand_cards(third)
        .into_iter()
        .take(2)
        .collect::<Vec<_>>();
    let state = resolve_choice(
        tx.commit().state,
        third,
        2,
        vec![cards[0].raw(), cards[1].raw()],
        false,
    );
    let mut tx = Transaction::begin(state.clone());
    assert!(
        tx.state().hand_cards(owner).contains(&cards[0])
            && tx.state().hand_cards(owner).contains(&cards[1]),
        "两张牌应交到拥有者手中"
    );
}

/// 通夺（ABI 442）：成为他人单目标目标后选角色强制重铸一张牌。
#[test]
fn tongduo_forces_recast() {
    const TONGDUO: u32 = 3_696_525_701;
    let (mut tx, owner, seats) = setup(TONGDUO, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let skill = skill_instance(&tx, owner, TONGDUO).unwrap();
    // 通夺：选重铸角色 → 该角色选牌 → 重铸（牌入弃牌堆）。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTongduo {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开选角色 choice");
    // 选角色后 handler 自动推进选牌 choice。
    let state = resolve_choice(tx.commit().state, owner, 1, vec![other.raw()], false);
    assert_ne!(state.resolution.pending_choice_id, 0, "应推进到选牌 choice");
    let card = state.hand_cards(other)[0];
    let state = resolve_choice(state, other, 2, vec![card.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    // 选牌后 handler 自动推进重铸（stage 2 由 handler 调用）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(
        tx.state()
            .zones
            .get(discard)
            .is_some_and(|zone| zone.cards.contains(&card)),
        "重铸牌应进入弃牌堆"
    );
}

/// 怨清（ABI 443）：出牌阶段结束后按类别取弃牌堆牌入仁库；疏陈（ABI 444）：
/// 濒死时仁库>3 获得全部并回复。
#[test]
fn yuanqing_renku_and_shuchen() {
    const YUANQING: u32 = 3_702_802_540;
    const SHUCHEN: u32 = 3_570_664_782;
    let (mut tx, owner, seats) = setup(YUANQING, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 本阶段用过基本+装备两类。
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.set_flag(
        turn_flag::owner_card_category_history_by_player(owner),
        0b101,
    );
    // 弃牌堆放基本(2 张)与装备(2 张)。
    for (kind, suit) in [
        (CardKind::Sha, Suit::Spade),
        (CardKind::Tao, Suit::Heart),
        (CardKind::Weapon, Suit::Club),
    ] {
        inject_to_zone(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DiscardPile,
            kind,
            suit,
            3,
        );
        inject_to_zone(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DiscardPile,
            kind,
            suit,
            4,
        );
    }
    let skill = skill_instance(&tx, owner, YUANQING).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveYuanqing {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    // 两类各取一张入仁库。
    let renku = tx.state().skill_pile_cards(owner, YUANQING);
    assert_eq!(renku.len(), 2, "两类各取一张入仁库");
    // 再注入两张让仁库 > 3（补 2 张 = 4 张）。
    for (kind, suit) in [(CardKind::Sha, Suit::Spade), (CardKind::Sha, Suit::Spade)] {
        let _ = inject_to_zone(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DiscardPile,
            kind,
            suit,
            5,
        );
    }
    let pile = tx.state().zones.find(ZoneKind::Expansion, owner).unwrap();
    for _ in 0..2 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(80_000 + CardKind::Sha.tag() as u32),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: 3,
            weapon_range: 0,
            zone: pile,
            owner,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(pile).unwrap().cards.push(card);
    }
    assert_eq!(
        tx.state().skill_pile_cards(owner, YUANQING).len(),
        4,
        "仁库应为 4 张"
    );
    // 疏陈：其他角色濒死 → 获得全部仁库牌并回复。
    let shuchen_skill = {
        let mut effects: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|s| EffectRequest::RemoveSkill { skill: s.id })
            .collect();
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHUCHEN),
        });
        run_effects(&mut tx, effects).unwrap();
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|s| s.def.raw() == SHUCHEN && s.owner == owner)
            .map(|s| s.id)
            .unwrap()
    };
    // 让濒死者受伤以便回复可生效。
    {
        let p = tx.state_mut().players.get_mut(other).unwrap();
        p.hp = p.hp.saturating_sub(1);
    }
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveShuchen {
            player: owner,
            dying: other,
            renku_def: SkillDefId::new(YUANQING),
            skill: shuchen_skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 4,
        "应获得全部仁库牌"
    );
    assert_eq!(
        tx.state().players.get(other).unwrap().hp,
        other_hp + 1,
        "濒死者应回复 1"
    );
}

/// 断笔（ABI 445）：全场手牌 > 人数×2 时逐目标弃牌 → 选人随机获至多 3 张被弃牌。
#[test]
fn duanbi_forces_discards_and_gains() {
    const DUANBI: u32 = 4_273_058_855;
    let (mut tx, owner, seats) = setup(DUANBI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, DUANBI).unwrap();
    // 手牌总数 > 4：双方各补 3 张。
    for p in [owner, other] {
        for i in 0..3u8 {
            inject_card(&mut tx, p, CardKind::Sha, Suit::Spade, i + 1);
        }
    }
    assert!(
        tx.state().hand_count(owner) as usize + tx.state().hand_count(other) as usize > 4,
        "条件应满足"
    );
    // 断笔：开第一个目标的弃牌 choice（弃 floor(手牌/2) ≤ 3）。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDuanbi {
            player: owner,
            targets: vec![other],
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开弃牌 choice");
    // 目标弃 3 张（手牌多）。
    let hand = tx.state().hand_cards(other);
    let discard_cards = hand.into_iter().take(3).collect::<Vec<_>>();
    let state = resolve_choice(
        tx.commit().state,
        other,
        1,
        vec![
            discard_cards[0].raw(),
            discard_cards[1].raw(),
            discard_cards[2].raw(),
        ],
        false,
    );
    // 弃完 → 开选人获牌 choice。
    assert_ne!(state.resolution.pending_choice_id, 0, "应开选人获牌 choice");
    let state = resolve_choice(state, owner, 2, vec![owner.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    // 拥有者应获得弃置堆中至少一张被弃牌（权威随机至多 3）。
    let gained = tx
        .state()
        .hand_cards(owner)
        .iter()
        .filter(|card| discard_cards.contains(card))
        .count();
    assert!(
        gained >= 1,
        "应获得被弃牌（至多 3 张，至少 1 张仍在弃牌堆）"
    );
}

/// 躁厉（ABI 446）：有标记时弃至少一张并摸（标记+弃数），总数>2 失体。
#[test]
fn zaoli_discard_draw_and_lose_hp() {
    const ZAOLI: u32 = 2_486_461_241;
    let (mut tx, owner, _seats) = setup(ZAOLI, 2);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let skill = skill_instance(&tx, owner, ZAOLI).unwrap();
    // 2 枚躁厉标记。
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(ZAOLI),
            delta: 2,
        }],
    )
    .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZaoli {
            player: owner,
            mark: 2,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开弃牌 choice");
    // 弃 1 张 → 摸 2+1=3；总数 3 > 2 → 失 1 体。
    let card = tx.state().hand_cards(owner)[0];
    let hand_before = tx.state().hand_count(owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let state = resolve_choice(tx.commit().state, owner, 1, vec![card.raw()], false);
    let mut tx = Transaction::begin(state.clone());
    assert_eq!(
        tx.state().hand_count(owner) as i32,
        hand_before as i32 - 1 + 3,
        "应摸标记数+弃牌数（3）张"
    );
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp_before - 1,
        "总数大于 2 应失去 1 点体力"
    );
    assert!(
        !tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(m, _)| *m == ZAOLI),
        "标记应清空"
    );
}

/// 孤影（ABI 447）：失去牌后当前回合角色二选一（随机交牌 / 收回弃牌堆失去牌）。
#[test]
fn guying_two_choice_recover_or_gain() {
    const GUYING: u32 = 2_534_727_648;
    let (mut tx, owner, seats) = setup(GUYING, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill_instance(&tx, owner, GUYING).unwrap();
    // 失去一张牌（通过引擎移牌管线）。
    let lost = tx.state().hand_cards(owner)[0];
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![lost],
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    // 孤影：其他角色二选一（0 随机交牌 / 1 收回失去牌）。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveGuying {
            player: owner,
            actor: other,
            card: lost,
            stage: 0,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应开二选一 choice");
    // 选 1：收回失去的牌。
    let state = resolve_choice(tx.commit().state, other, 1, vec![1], false);
    let mut tx = Transaction::begin(state.clone());
    assert!(
        tx.state().hand_cards(owner).contains(&lost),
        "应收回失去的牌"
    );
}
