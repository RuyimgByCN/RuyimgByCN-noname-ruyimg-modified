//! Golden for xianding 三首 sanshou (ABI 751, tag 767):
//! 即将受到伤害时可亮牌堆顶三张：存在本局未使用类型的牌则防止伤害；
//! 亮出的牌移入弃牌堆。真管道（DamageBefore 事件派发）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::damage_nature;
use rule_api::EffectRequest;

const SANSHOU: u32 = 3_583_301_051;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
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
}

fn inject_draw_top(state: &mut GameState, kind: CardKind, tag: u32) -> CardId {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(104_000 + tag),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_sanshou_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SANSHOU)
        .expect("sanshou trigger");
    assert_eq!(entry.event.as_str(), "damage_before");
    assert_eq!(
        entry.filter.as_str(),
        "owner_is_target_sanshou_type_tracked"
    );
    assert!(!entry.optional);
}

#[test]
fn sanshou_cancels_damage_when_new_type_revealed() {
    let (mut state, players) = started(2_026_107_001);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, SANSHOU);
    // 本局无人用过牌（mask=0）→ 牌堆顶注入三张全新类型（基本/锦囊/装备）。
    let _a = inject_draw_top(&mut state, CardKind::Tao, 1);
    let _b = inject_draw_top(&mut state, CardKind::TrickWuxie, 2);
    let _c = inject_draw_top(&mut state, CardKind::Weapon, 3);
    let hp_before = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    // DealDamage 管道内构造 DamageBefore ctx → optional 确认窗。
    let dmg = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    );
    match dmg {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    // 确认发动 → 机器亮顶判型 → 全新类型 → PreventDamage。
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before,
        "全新类型亮出应防止伤害"
    );
    // 亮出的三张移入弃牌堆。
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.len() >= 3);
}

#[test]
fn sanshou_passes_when_all_types_known() {
    let (mut state, players) = started(2_026_107_002);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, SANSHOU);
    // 位图预置三种类型全部已知 → 无全新类型 → 不防伤。
    state.lifetime_used_type_groups = 0b0000_0111;
    let _a = inject_draw_top(&mut state, CardKind::Tao, 4);
    let _b = inject_draw_top(&mut state, CardKind::TrickWuxie, 5);
    let _c = inject_draw_top(&mut state, CardKind::Weapon, 6);
    let hp_before = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let dmg = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    );
    match dmg {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    // forced：窗口内机器同步执行 → 无全新类型 → 伤害照常。
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before - 2,
        "无全新类型应照常受伤"
    );
}
