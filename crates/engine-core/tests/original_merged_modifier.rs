//! O2 全量合并第二批（2026-10-14）：家族 modifier 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_modifier_temp_skills {
//! Golden coverage for reusable legacy modifier-only and temporary skills.

use engine_api::{
    CardDefId, CardId, CardKindTag, ChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const CLAN: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");
const COLLAB: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");
const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");
const HUICUI: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const SXRM: &[u8] = include_bytes!("../../../content/original/ported/sxrm/sxrm.npack.json");
const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");
const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const XIANDING: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");
const YINGBIAN: &[u8] =
    include_bytes!("../../../content/original/ported/yingbian/yingbian.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SP_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sp/gongao.skill"),
    include_str!("../../../content/original/ported/sp/liewei.skill"),
    include_str!("../../../content/original/ported/sp/olcuorui.skill"),
    include_str!("../../../content/original/ported/sp/fengpo.skill"),
    include_str!("../../../content/original/ported/sp/chongzhen.skill"),
    include_str!("../../../content/original/ported/sp/hongde.skill"),
    include_str!("../../../content/original/ported/sp/bizheng.skill"),
    include_str!("../../../content/original/ported/sp/xiashu.skill"),
    include_str!("../../../content/original/ported/sp/xinfu_guanchao.skill"),
    include_str!("../../../content/original/ported/sp/xinfu_guanwei.skill"),
    include_str!("../../../content/original/ported/sp/lanjiang.skill"),
    include_str!("../../../content/original/ported/sp/zhuishe.skill"),
    include_str!("../../../content/original/ported/sp/weizhong.skill"),
    include_str!("../../../content/original/ported/sp/wangzun3.skill"),
    include_str!("../../../content/original/ported/sp/sanlve_skill.skill"),
    include_str!("../../../content/original/ported/sp/xuwangzhimian.skill"),
    include_str!("../../../content/original/ported/sp/qingyi3.skill"),
    include_str!("../../../content/original/ported/sp/zhanyi_trick.skill"),
    include_str!("../../../content/original/ported/sp/zhanyi_basic_sha.skill"),
    include_str!("../../../content/original/ported/sp/zhanyi_basic_tao.skill"),
    include_str!("../../../content/original/ported/sp/zhanyi_basic_jiu.skill"),
    include_str!("../../../content/original/ported/sp/olwuniang2.skill"),
    include_str!("../../../content/original/ported/sp/tuifeng3.skill"),
    include_str!("../../../content/original/ported/sp/zhidao2.skill"),
    include_str!("../../../content/original/ported/sp/rebiluan2.skill"),
    include_str!("../../../content/original/ported/sp/xiefang.skill"),
    include_str!("../../../content/original/ported/sp/zhuiji.skill"),
    include_str!("../../../content/original/ported/sp/yicong.skill"),
    include_str!("../../../content/original/ported/sp/xiandeng.skill"),
    include_str!("../../../content/original/ported/sp/guixiu.skill"),
    include_str!("../../../content/original/ported/sp/fenyue2.skill"),
    include_str!("../../../content/original/ported/sp/olhexu.skill"),
    include_str!("../../../content/original/ported/sp/guilongzhanyuedao.skill"),
    include_str!("../../../content/original/ported/sp/chouhai.skill"),
    include_str!("../../../content/original/ported/sp/xinfu_gongqing.skill"),
    include_str!("../../../content/original/ported/sp/jugu.skill"),
    include_str!("../../../content/original/ported/sp/zhuangshu_equip.skill"),
    include_str!("../../../content/original/ported/sp/ranshang.skill"),
    include_str!("../../../content/original/ported/sp/ranshang2.skill"),
    include_str!("../../../content/original/ported/sp/jingong2.skill"),
    include_str!("../../../content/original/ported/sp/shengxi.skill"),
    include_str!("../../../content/original/ported/sp/olzhennan2.skill"),
    include_str!("../../../content/original/ported/sp/jiaozi.skill"),
    include_str!("../../../content/original/ported/sp/olbiri.skill"),
    include_str!("../../../content/original/ported/sp/shoucheng.skill"),
    include_str!("../../../content/original/ported/sp/shenxian.skill"),
    include_str!("../../../content/original/ported/sp/rechouhai.skill"),
    include_str!("../../../content/original/ported/sp/shulv.skill"),
    include_str!("../../../content/original/ported/sp/xinfu_zhanji.skill"),
    include_str!("../../../content/original/ported/sp/xinfu_tushe.skill"),
    include_str!("../../../content/original/ported/sp/guofengyupao.skill"),
    include_str!("../../../content/original/ported/sp/gzjili.skill"),
    include_str!("../../../content/original/ported/sp/fuqi.skill"),
    include_str!("../../../content/original/ported/sp/zhenlue.skill"),
    include_str!("../../../content/original/ported/sp/qimenbagua.skill"),
    include_str!("../../../content/original/ported/sp/zniaoxiang.skill"),
    include_str!("../../../content/original/ported/sp/spweiwu.skill"),
    include_str!("../../../content/original/ported/sp/yanzheng.skill"),
    include_str!("../../../content/original/ported/sp/kuiwei.skill"),
    include_str!("../../../content/original/ported/sp/olyinji.skill"),
    include_str!("../../../content/original/ported/sp/shushen.skill"),
    include_str!("../../../content/original/ported/sp/shenzhi.skill"),
    include_str!("../../../content/original/ported/sp/zhenwei_three.skill"),
    include_str!("../../../content/original/ported/sp/qiongshou.skill"),
    include_str!("../../../content/original/ported/sp/sijian.skill"),
    include_str!("../../../content/original/ported/sp/wangong.skill"),
    include_str!("../../../content/original/ported/sp/juyi.skill"),
    include_str!("../../../content/original/ported/sp/xiongyi.skill"),
    include_str!("../../../content/original/ported/sp/hanyong.skill"),
    include_str!("../../../content/original/ported/sp/moukui.skill"),
    include_str!("../../../content/original/ported/sp/cunsi.skill"),
    include_str!("../../../content/original/ported/sp/yongjue.skill"),
    include_str!("../../../content/original/ported/sp/qiangwu.skill"),
    include_str!("../../../content/original/ported/sp/fenming.skill"),
    include_str!("../../../content/original/ported/sp/recanshi.skill"),
    include_str!("../../../content/original/ported/sp/juesi.skill"),
    include_str!("../../../content/original/ported/sp/lizhan.skill"),
    include_str!("../../../content/original/ported/sp/weikui.skill"),
    include_str!("../../../content/original/ported/sp/kaikang.skill")
);
const SP2_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sp2/juntun.skill"),
    include_str!("../../../content/original/ported/sp2/zhihu.skill"),
    include_str!("../../../content/original/ported/sp2/chenggong.skill"),
    include_str!("../../../content/original/ported/sp2/yanjiao2.skill"),
    include_str!("../../../content/original/ported/sp2/redaoji2.skill"),
    include_str!("../../../content/original/ported/sp2/dcbenshi.skill"),
    include_str!("../../../content/original/ported/sp2/xiying2.skill"),
    include_str!("../../../content/original/ported/sp2/dczhuiwang.skill"),
    include_str!("../../../content/original/ported/sp2/nhguizhu.skill"),
    include_str!("../../../content/original/ported/sp2/jiaojie.skill"),
    include_str!("../../../content/original/ported/sp2/huangkong.skill"),
    include_str!("../../../content/original/ported/sp2/quanjiu.skill"),
    include_str!("../../../content/original/ported/sp2/starzhonggu.skill"),
    include_str!("../../../content/original/ported/sp2/xianshuai.skill"),
    include_str!("../../../content/original/ported/sp2/reliewei.skill"),
    include_str!("../../../content/original/ported/sp2/starsujun.skill"),
    include_str!("../../../content/original/ported/sp2/tanbei_effect1.skill"),
    include_str!("../../../content/original/ported/sp2/tanbei_effect2.skill"),
    include_str!("../../../content/original/ported/sp2/kuimang.skill"),
    include_str!("../../../content/original/ported/sp2/gangzhi.skill"),
    include_str!("../../../content/original/ported/sp2/starjianyu.skill"),
    include_str!("../../../content/original/ported/sp2/xiongsuan.skill"),
    include_str!("../../../content/original/ported/sp2/nhxianshou.skill"),
    include_str!("../../../content/original/ported/sp2/nhhuoqi.skill"),
    include_str!("../../../content/original/ported/sp2/nhyinbing.skill"),
    include_str!("../../../content/original/ported/sp2/xinfu_xingzhao3.skill"),
    include_str!("../../../content/original/ported/sp2/xinfu_xingzhao.skill"),
    include_str!("../../../content/original/ported/sp2/xinfu_xingzhao2.skill"),
    include_str!("../../../content/original/ported/sp2/xz_xunxun.skill"),
    include_str!("../../../content/original/ported/sp2/refuqi.skill"),
    include_str!("../../../content/original/ported/sp2/nifu.skill"),
    include_str!("../../../content/original/ported/sp2/lxzhuixi.skill"),
    include_str!("../../../content/original/ported/sp2/langmie.skill")
);
const XIANDING_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/xianding/spzhuilie2.skill"),
    include_str!("../../../content/original/ported/xianding/dclima.skill"),
    include_str!("../../../content/original/ported/xianding/dcsbchensheng.skill"),
    include_str!("../../../content/original/ported/xianding/dcyuxin.skill"),
    include_str!("../../../content/original/ported/xianding/dcduanxie.skill"),
    include_str!("../../../content/original/ported/xianding/dckunli.skill"),
    include_str!("../../../content/original/ported/xianding/dcshangjue.skill"),
    include_str!("../../../content/original/ported/xianding/dcsbkangyong.skill")
);
const YINGBIAN_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/yingbian/yifa2.skill"),
    include_str!("../../../content/original/ported/yingbian/qiangshou.skill"),
    include_str!("../../../content/original/ported/yingbian/g_hidden_ai.skill"),
    include_str!("../../../content/original/ported/yingbian/xuanmu2.skill"),
    include_str!("../../../content/original/ported/yingbian/ciwei_ai.skill")
);
const SXRM_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/sxrm/old_wushen.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrm_meta.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrmbiyi.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrmchengbian.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrmfubei.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrmlanjiao.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrmlishui.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrmqianliu.skill"),
    include_str!("../../../content/original/ported/sxrm/sxrmqiewang.skill")
);
fn yijiang_source() -> String {
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

fn loaded(package: &[u8], seed: u64) -> (Transaction, PlayerId) {
    loaded_players(package, seed, 2)
}

fn loaded_players(package: &[u8], seed: u64, player_count: u8) -> (Transaction, PlayerId) {
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

fn strip_skills(tx: &mut Transaction, owner: PlayerId) {
    let skills = tx.state().players.get(owner).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, skill: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill),
        }],
    )
    .unwrap();
}

fn add_mark(tx: &mut Transaction, owner: PlayerId, mark: u32, count: i32) {
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

#[test]
fn jiaozi_adds_damage_only_while_the_owner_uniquely_has_the_most_hand_cards() {
    let (mut tx, owner) = loaded(SP, 20_260_821);
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    for player in [owner, other] {
        strip_skills(&mut tx, player);
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);
    let (player, count) = if owner_hand < other_hand {
        (owner, other_hand - owner_hand)
    } else {
        (other, owner_hand - other_hand)
    };
    run_effects(&mut tx, vec![EffectRequest::DrawCards { player, count }]).unwrap();
    assert_eq!(tx.state().hand_count(owner), tx.state().hand_count(other));
    add_skill(&mut tx, owner, 2_415_897_133);

    let before = tx.state().players.get(other).unwrap().hp;
    schedule_damage(&mut tx, owner, other, 1).unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, before - 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    let before = tx.state().players.get(other).unwrap().hp;
    schedule_damage(&mut tx, owner, other, 1).unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, before - 2);

    let before = tx.state().players.get(owner).unwrap().hp;
    schedule_damage(&mut tx, other, owner, 1).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, before - 2);
}

#[test]
fn xinfu_xingzhao3_adds_damage_at_zero_or_four_wounded_players() {
    let (mut tx, owner) = loaded_players(SP2, 20_260_715, 4);
    let players = tx.state().players.alive_by_seat();
    let target = players
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    for player in &players {
        strip_skills(&mut tx, *player);
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    add_skill(&mut tx, owner, 2_896_246_690);

    let before = tx.state().players.get(target).unwrap().hp;
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 2);

    for player in &players {
        tx.state_mut().players.get_mut(*player).unwrap().hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 9;
    let before = tx.state().players.get(target).unwrap().hp;
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 1);

    for player in &players {
        tx.state_mut().players.get_mut(*player).unwrap().hp = 9;
    }
    let before = tx.state().players.get(target).unwrap().hp;
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 2);
}

#[test]
fn hm_yingzhan_adds_only_sourced_fire_and_thunder_damage() {
    let (mut tx, owner) = loaded(OFFLINE, 20_260_822);
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    for player in [owner, other] {
        strip_skills(&mut tx, player);
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    add_skill(&mut tx, owner, 2_176_897_227);

    for (source, target, nature, expected) in [
        (owner, other, 1, 2),
        (other, owner, 2, 2),
        (owner, other, 0, 1),
        (PlayerId::NONE, other, 1, 1),
    ] {
        let before = tx.state().players.get(target).unwrap().hp;
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(
            tx.state().players.get(target).unwrap().hp,
            before - expected
        );
    }
}

fn hand_limit(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn attack_range(tx: &Transaction, owner: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn distance(tx: &Transaction, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn equip_defensive_horse(tx: &mut Transaction, owner: PlayerId) -> CardId {
    equip_mount(tx, owner, CardKind::HorseDef)
}

fn equip_mount(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> CardId {
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

fn place_rank_thirteen_field_card(
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

fn assign_four_groups(tx: &mut Transaction, owner: PlayerId) -> Vec<PlayerId> {
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

#[test]
fn alive_group_count_drives_three_legacy_modifiers() {
    let (mut juliao, owner) = loaded_players(MOBILE, 20_260_754, 4);
    let players = assign_four_groups(&mut juliao, owner);
    let from = players[1];
    let base = distance(&juliao, from, owner);
    add_skill(&mut juliao, owner, 4_062_961_693);
    assert_eq!(distance(&juliao, from, owner), base + 3);
    juliao
        .state_mut()
        .players
        .get_mut(players[3])
        .unwrap()
        .character_def = 1;
    assert_eq!(distance(&juliao, from, owner), base + 2);

    let (mut tianshu, owner) = loaded_players(SIXIANG, 20_260_755, 4);
    let players = assign_four_groups(&mut tianshu, owner);
    let base = hand_limit(&tianshu, owner);
    add_skill(&mut tianshu, owner, 2_266_533_034);
    assert_eq!(hand_limit(&tianshu, owner), base + 3);
    tianshu
        .state_mut()
        .players
        .get_mut(players[3])
        .unwrap()
        .character_def = 1;
    assert_eq!(hand_limit(&tianshu, owner), base + 2);

    let (mut zongshi, owner) = loaded_players(YIJIANG, 20_260_756, 4);
    assign_four_groups(&mut zongshi, owner);
    let base = hand_limit(&zongshi, owner);
    add_skill(&mut zongshi, owner, 2_639_072_445);
    assert_eq!(hand_limit(&zongshi, owner), base + 4);
}

#[test]
fn alive_female_count_drives_xiefang() {
    let (mut tx, owner) = loaded_players(SP, 20_260_759, 6);
    strip_skills(&mut tx, owner);
    let players = tx.state().players.alive_by_seat();
    assert_eq!(players.len(), 6);
    for player in &players {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .character_def = 1;
    }
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = 20;
    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = 25;

    let owner_index = players.iter().position(|player| *player == owner).unwrap();
    let far = players[(owner_index + 3) % players.len()];
    let base = distance(&tx, owner, far);
    assert_eq!(base, 3);
    add_skill(&mut tx, owner, 3_450_775_931);
    assert_eq!(distance(&tx, owner, far), 1);

    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = 1;
    assert_eq!(distance(&tx, owner, far), 2);
}

#[test]
fn alive_other_count_capped_four_drives_twyuanbu() {
    let (mut tx, owner) = loaded_players(TW, 20_260_760, 6);
    strip_skills(&mut tx, owner);
    let players = tx.state().players.alive_by_seat();
    let target = *players.iter().find(|player| **player != owner).unwrap();
    let base_hand = hand_limit(&tx, owner);
    let base_from = distance(&tx, owner, target);
    let base_to = distance(&tx, target, owner);
    add_skill(&mut tx, owner, 3_356_399_020);
    assert_eq!(hand_limit(&tx, owner), base_hand + 4);
    assert_eq!(distance(&tx, owner, target), base_from + 4);
    assert_eq!(distance(&tx, target, owner), base_to + 4);

    for player in players
        .iter()
        .copied()
        .filter(|player| *player != owner && *player != target)
        .take(2)
    {
        tx.state_mut().players.get_mut(player).unwrap().alive = false;
    }
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    let base_hand = hand_limit(&tx, owner);
    let base_from = distance(&tx, owner, target);
    let base_to = distance(&tx, target, owner);
    add_skill(&mut tx, owner, 3_356_399_020);
    assert_eq!(hand_limit(&tx, owner), base_hand + 3);
    assert_eq!(distance(&tx, owner, target), base_from + 3);
    assert_eq!(distance(&tx, target, owner), base_to + 3);
}

#[test]
fn owner_no_mount_drives_twshenxing() {
    let (mut tx, owner) = loaded_players(TW, 20_260_785, 4);
    strip_skills(&mut tx, owner);
    let players = tx.state().players.alive_by_seat();
    let owner_index = players.iter().position(|player| *player == owner).unwrap();
    let far = players[(owner_index + 2) % players.len()];
    strip_skills(&mut tx, far);
    let base_hand = hand_limit(&tx, owner);
    let base_distance = distance(&tx, owner, far);
    assert_eq!(base_distance, 2);

    add_skill(&mut tx, owner, 4_258_102_225);
    assert_eq!(hand_limit(&tx, owner), base_hand + 1);
    assert_eq!(distance(&tx, owner, far), 1);
    assert_eq!(distance(&tx, far, owner), base_distance);

    let horse = equip_defensive_horse(&mut tx, owner);
    assert_eq!(hand_limit(&tx, owner), base_hand);
    assert_eq!(distance(&tx, owner, far), base_distance);
    assert_eq!(distance(&tx, far, owner), base_distance + 1);

    tx.state_mut().cards.get_mut(horse).unwrap().kind = CardKind::HorseOff;
    assert_eq!(hand_limit(&tx, owner), base_hand);
    assert_eq!(distance(&tx, owner, far), 1);
    assert_eq!(distance(&tx, far, owner), base_distance);

    tx.state_mut().cards.get_mut(horse).unwrap().kind = CardKind::Weapon;
    assert_eq!(hand_limit(&tx, owner), base_hand + 1);
    assert_eq!(distance(&tx, owner, far), 1);
    assert_eq!(distance(&tx, far, owner), base_distance);
}

#[test]
fn owner_has_treasure_drives_qiangshou_through_the_equip_path() {
    let (mut tx, owner) = loaded_players(YINGBIAN, 20_260_786, 4);
    strip_skills(&mut tx, owner);
    let players = tx.state().players.alive_by_seat();
    let owner_index = players.iter().position(|player| *player == owner).unwrap();
    let far = players[(owner_index + 2) % players.len()];
    strip_skills(&mut tx, far);
    assert_eq!(distance(&tx, owner, far), 2);
    assert_eq!(distance(&tx, far, owner), 2);

    add_skill(&mut tx, owner, 3_061_185_460);
    assert_eq!(distance(&tx, owner, far), 2);

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let treasures = tx.state().hand_cards(owner)[..2].to_vec();
    for card in &treasures {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Treasure;
    }

    let before = tx.state().clone();
    let (after, first) = apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(20_260_786),
            kind: CommandKind::PlayCard {
                card: treasures[0],
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let tx = Transaction::begin(after);
    assert_eq!(
        tx.state()
            .zones
            .get(tx.state().cards.get(treasures[0]).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::Equip
    );
    assert_eq!(distance(&tx, owner, far), 1);
    assert_eq!(distance(&tx, far, owner), 2);
    let view = engine_core::projection::project_player(tx.state(), owner);
    assert_eq!(
        view.known_cards
            .iter()
            .find(|card| card.id == treasures[0])
            .unwrap()
            .kind,
        CardKindTag::Treasure
    );

    let before = tx.state().clone();
    let (after, second) = apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(20_260_787),
            kind: CommandKind::PlayCard {
                card: treasures[1],
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(after);
    assert_eq!(
        tx.state()
            .zones
            .get(tx.state().cards.get(treasures[0]).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    assert_eq!(
        tx.state()
            .zones
            .get(tx.state().cards.get(treasures[1]).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::Equip
    );
    assert_eq!(distance(&tx, owner, far), 1);

    tx.state_mut().cards.get_mut(treasures[1]).unwrap().kind = CardKind::Armor;
    assert_eq!(distance(&tx, owner, far), 2);
}

#[test]
fn mounted_player_count_drives_dclima_with_a_minimum_of_one() {
    let (mut tx, owner) = loaded_players(XIANDING, 20_260_788, 8);
    strip_skills(&mut tx, owner);
    let players = tx.state().players.alive_by_seat();
    let owner_index = players.iter().position(|player| *player == owner).unwrap();
    let far = players[(owner_index + 4) % players.len()];
    strip_skills(&mut tx, far);
    assert_eq!(distance(&tx, owner, far), 4);
    assert_eq!(distance(&tx, far, owner), 4);

    add_skill(&mut tx, owner, 2_775_722_447);
    assert_eq!(distance(&tx, owner, far), 3);
    assert_eq!(distance(&tx, far, owner), 4);

    let mounted = players[(owner_index + 1) % players.len()];
    equip_mount(&mut tx, mounted, CardKind::HorseOff);
    assert_eq!(distance(&tx, owner, far), 3);
    equip_mount(&mut tx, mounted, CardKind::HorseDef);
    assert_eq!(distance(&tx, owner, far), 3);

    let second_mounted = players[(owner_index + 2) % players.len()];
    equip_mount(&mut tx, second_mounted, CardKind::HorseOff);
    assert_eq!(distance(&tx, owner, far), 2);
    tx.state_mut()
        .players
        .get_mut(second_mounted)
        .unwrap()
        .alive = false;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    let base_after_death = distance(&tx, owner, far);
    add_skill(&mut tx, owner, 2_775_722_447);
    assert_eq!(distance(&tx, owner, far), base_after_death - 1);
}

#[test]
fn equip_count_parity_drives_stdqianchong() {
    let (mut tx, owner) = loaded(SIXIANG, 20_260_789);
    strip_skills(&mut tx, owner);
    let card = tx.state().hand_cards(owner)[0];
    add_skill(&mut tx, owner, 2_607_738_803);

    assert!(!engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));
    assert!(engine_core::query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));

    equip_mount(&mut tx, owner, CardKind::HorseOff);
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));
    assert!(!engine_core::query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));

    equip_mount(&mut tx, owner, CardKind::HorseDef);
    assert!(!engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));
    assert!(engine_core::query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));
}

#[test]
fn lost_hp_drives_oldjuejing_draw_count_and_hand_limit() {
    let (mut tx, owner) = loaded(SIXIANG, 20_260_790);
    strip_skills(&mut tx, owner);
    let base_hand = hand_limit(&tx, owner);
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);

    add_skill(&mut tx, owner, 2_511_966_880);
    assert_eq!(hand_limit(&tx, owner), base_hand + 2);
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);

    tx.state_mut().players.get_mut(owner).unwrap().hp -= 2;
    assert_eq!(hand_limit(&tx, owner), base_hand);
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 4);
}

#[test]
fn current_hp_drives_jxxiongzi_draw_count_and_hand_limit() {
    let (mut tx, owner) = loaded(OFFLINE, 20_260_818);
    strip_skills(&mut tx, owner);
    let hp = tx.state().players.get(owner).unwrap().hp;
    add_skill(&mut tx, owner, 2_266_264_091);
    assert_eq!(hand_limit(&tx, owner), hp * 2);
    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        (2 + hp) as u32
    );

    tx.state_mut().players.get_mut(owner).unwrap().hp -= 2;
    assert_eq!(hand_limit(&tx, owner), (hp - 2) * 2);
    assert_eq!(
        engine_core::query::draw_count_for(tx.state(), owner),
        hp as u32
    );
}

#[test]
fn rewrite_zhuge_removes_the_sha_use_limit() {
    let (mut tx, owner) = loaded(MOBILE, 20_260_819);
    strip_skills(&mut tx, owner);
    add_skill(&mut tx, owner, 4_056_524_183);
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );
}

#[test]
fn liulongcanjia_counts_players_with_field_k_cards_at_query_time() {
    let (mut tx, owner) = loaded_players(MOBILE, 20_260_820, 4);
    strip_skills(&mut tx, owner);
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let other = others[0];
    let far = others[1];
    let inbound_base = distance(&tx, far, owner);
    let outbound_base = distance(&tx, owner, far);
    add_skill(&mut tx, owner, 3_011_623_709);
    assert_eq!(distance(&tx, far, owner), inbound_base);

    place_rank_thirteen_field_card(&mut tx, other, ZoneKind::Equip);
    place_rank_thirteen_field_card(&mut tx, other, ZoneKind::Judge);
    assert_eq!(distance(&tx, far, owner), inbound_base + 1);
    assert_eq!(distance(&tx, owner, far), (outbound_base - 1).max(1));

    place_rank_thirteen_field_card(&mut tx, owner, ZoneKind::Equip);
    assert_eq!(distance(&tx, far, owner), inbound_base + 2);
    assert_eq!(distance(&tx, owner, far), (outbound_base - 2).max(1));
}

#[test]
fn owner_hp_values_drive_yongsi_and_xinyicong() {
    let (mut yongsi, owner) = loaded(SHENHUA, 20_260_757);
    strip_skills(&mut yongsi, owner);
    yongsi.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    add_skill(&mut yongsi, owner, 3_870_932_875);
    for (hp, contribution) in [(4, -4), (2, 0), (0, 4), (-1, 4)] {
        yongsi.state_mut().players.get_mut(owner).unwrap().hp = hp;
        let with_skill = hand_limit(&yongsi, owner);
        let skill = yongsi.state().players.get(owner).unwrap().skills[0];
        run_effects(&mut yongsi, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        let base = hand_limit(&yongsi, owner);
        assert_eq!(with_skill, base + contribution);
        add_skill(&mut yongsi, owner, 3_870_932_875);
    }

    let (mut yicong, owner) = loaded_players(REFRESH, 20_260_758, 4);
    strip_skills(&mut yicong, owner);
    {
        let player = yicong.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 4;
    }
    let seats = yicong.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let base_from = distance(&yicong, owner, far);
    let base_to = distance(&yicong, far, owner);
    add_skill(&mut yicong, owner, 3_328_413_812);
    for (hp, from_delta, to_delta) in [(4, -3, 0), (2, -1, 1), (0, 0, 3)] {
        yicong.state_mut().players.get_mut(owner).unwrap().hp = hp;
        assert_eq!(
            distance(&yicong, owner, far),
            (base_from + from_delta).max(1)
        );
        assert_eq!(distance(&yicong, far, owner), base_to + to_delta);
    }
}

#[test]
fn hp_thresholds_drive_three_yicong_variants() {
    for (index, (package, skill, damaged_from_delta)) in [
        (SP, 4_025_383_419, 0),
        (REFRESH, 3_426_755_583, -1),
        (REFRESH, 3_742_694_356, -1),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded_players(package, 20_260_791 + index as u64, 4);
        strip_skills(&mut tx, owner);
        let seats = tx.state().players.alive_by_seat();
        let owner_index = seats.iter().position(|player| *player == owner).unwrap();
        let far = seats[(owner_index + 2) % seats.len()];
        strip_skills(&mut tx, far);
        {
            let player = tx.state_mut().players.get_mut(owner).unwrap();
            player.max_hp = 4;
            player.hp = 4;
        }
        let base_from = distance(&tx, owner, far);
        let base_to = distance(&tx, far, owner);

        add_skill(&mut tx, owner, skill);
        assert_eq!(distance(&tx, owner, far), base_from - 1);
        assert_eq!(distance(&tx, far, owner), base_to);

        tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
        assert_eq!(distance(&tx, owner, far), base_from - 1);
        assert_eq!(distance(&tx, far, owner), base_to);

        tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
        assert_eq!(distance(&tx, owner, far), base_from + damaged_from_delta);
        assert_eq!(distance(&tx, far, owner), base_to + 1);
    }
}

#[test]
fn xiandeng_only_removes_distance_from_the_first_sha() {
    let (mut tx, owner) = loaded_players(SP, 20_260_795, 4);
    strip_skills(&mut tx, owner);
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 1);
    assert!(!engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        owner
    ));

    add_skill(&mut tx, owner, 2_439_633_691);
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
    assert!(engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        owner
    ));

    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
    assert!(!engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        owner
    ));
}

#[test]
fn hp_comparison_forces_zhuiji_and_dczhuiwang_distance_to_one() {
    for (index, (package, skill)) in [(SP, 3_152_159_217), (SP2, 4_219_161_988)]
        .into_iter()
        .enumerate()
    {
        let (mut tx, owner) = loaded_players(package, 20_260_784 + index as u64, 4);
        strip_skills(&mut tx, owner);
        let players = tx.state().players.alive_by_seat();
        let owner_index = players.iter().position(|player| *player == owner).unwrap();
        let target = players[(owner_index + 2) % players.len()];
        strip_skills(&mut tx, target);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
        tx.state_mut().players.get_mut(target).unwrap().hp = 3;
        let ordinary = distance(&tx, owner, target);
        assert_eq!(ordinary, 2);
        equip_defensive_horse(&mut tx, target);
        assert_eq!(distance(&tx, owner, target), ordinary + 1);

        add_skill(&mut tx, owner, skill);
        assert_eq!(distance(&tx, owner, target), 1);
        assert_eq!(distance(&tx, target, owner), ordinary);

        tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
        assert_eq!(distance(&tx, owner, target), ordinary + 1);
    }
}

#[test]
fn sha_cancel_surface_covers_four_legacy_temp_skills() {
    for (index, (package, skill)) in [
        (OFFLINE, 4_252_815_092),
        (REFRESH, 4_037_147_006),
        (SHENHUA, 2_573_090_805),
        (YIJIANG, 2_499_130_347),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded(package, 20_260_720 + index as u64);
        assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 1);
        add_skill(&mut tx, owner, skill);
        assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 0);
    }
}

#[test]
fn card_use_cancel_surface_covers_drlt_wanglie2_without_blocking_responses() {
    let (mut tx, owner) = loaded(SHENHUA, 20_260_725);
    add_skill(&mut tx, owner, 2_154_023_320);

    let intent = engine_core::intent::resolve_card_use_intent(
        tx.state(),
        engine_core::intent::CardUseIntent {
            player: owner,
            card: CardId::new(1),
            targets: Vec::new(),
            cancelled: false,
        },
    );
    assert!(intent.cancelled);
}

#[test]
fn xiying2_blocks_use_response_and_save_channels() {
    let (mut tx, owner) = loaded(SP2, 20_260_761);
    strip_skills(&mut tx, owner);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = engine_core::state::CardKind::Jiu;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));
    assert!(!engine_core::query::card_use_cancelled_for(
        tx.state(),
        owner
    ));
    assert!(!engine_core::query::card_response_cancelled_for(
        tx.state(),
        owner
    ));
    assert!(!engine_core::query::card_save_cancelled_for(
        tx.state(),
        owner
    ));

    add_skill(&mut tx, owner, 3_555_033_084);
    assert!(engine_core::query::card_use_cancelled_for(
        tx.state(),
        owner
    ));
    assert!(engine_core::query::card_response_cancelled_for(
        tx.state(),
        owner
    ));
    assert!(engine_core::query::card_save_cancelled_for(
        tx.state(),
        owner
    ));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(false)
    ));
}

#[test]
fn legacy_hand_lock_skills_block_hand_use_response_and_save_without_cancelling_all_responses() {
    for (index, (package, skill)) in [
        (COLLAB, 3_268_113_720),
        (SP, 2_246_618_728),
        (REFRESH, 2_542_175_146),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded(package, 20_260_807 + index as u64);
        let attacker = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .find(|player| *player != owner)
            .unwrap();
        strip_skills(&mut tx, owner);
        strip_skills(&mut tx, attacker);
        let owner_card = tx.state().hand_cards(owner)[0];
        tx.state_mut().cards.get_mut(owner_card).unwrap().kind = CardKind::Jiu;
        add_skill(&mut tx, owner, skill);

        assert!(engine_core::query::hand_card_use_response_cancelled_for(
            tx.state(),
            owner
        ));
        assert!(!engine_core::query::card_response_cancelled_for(
            tx.state(),
            owner
        ));
        assert!(engine_core::query::card_save_cancelled_for(
            tx.state(),
            owner
        ));
        assert!(matches!(
            query(
                tx.state(),
                &QueryRequest::CardUsable {
                    player: owner,
                    card: owner_card,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(false)
        ));
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = engine_core::state::Phase::Play;
        let (_, rejected) = apply_command(
            tx.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(30 + index as u64),
                kind: CommandKind::PlayCard {
                    card: owner_card,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

        tx.state_mut().cards.get_mut(owner_card).unwrap().kind = CardKind::Shan;
        let attacker_card = tx.state().hand_cards(attacker)[0];
        tx.state_mut().cards.get_mut(attacker_card).unwrap().kind = CardKind::Sha;
        tx.state_mut().turn.current = attacker;
        let hp = tx.state().players.get(owner).unwrap().hp;
        let (after, accepted) = apply_command(
            tx.state().clone(),
            Command {
                actor: attacker,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(40 + index as u64),
                kind: CommandKind::PlayCard {
                    card: attacker_card,
                    targets: vec![owner],
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
        assert!(accepted.choice.is_none());
        assert_eq!(after.players.get(owner).unwrap().hp, hp - 1);
    }
}

#[test]
fn nsxiuxin_allows_targeting_only_when_the_target_holds_the_incoming_suit() {
    let (mut tx, attacker) = loaded(DIY, 20_260_810);
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != attacker)
        .unwrap();
    strip_skills(&mut tx, attacker);
    strip_skills(&mut tx, target);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let sha = tx.state().hand_cards(attacker)[0];
    let target_hand = tx.state().hand_cards(target);
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(sha).unwrap().suit = Suit::Spade;
    for card in &target_hand {
        let card = tx.state_mut().cards.get_mut(*card).unwrap();
        card.kind = CardKind::Other;
        card.suit = Suit::Heart;
    }
    add_skill(&mut tx, target, 3_579_127_646);

    let target_legal = |tx: &Transaction| {
        matches!(
            query(
                tx.state(),
                &QueryRequest::TargetLegal {
                    user: attacker,
                    card: sha,
                    target,
                },
                &mut QueryContext::new(),
            ),
            QueryResponse::Bool(true)
        )
    };
    assert!(!target_legal(&tx));
    let command = Command {
        actor: attacker,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(50),
        kind: CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
    };
    let (_, rejected) = apply_command(tx.state().clone(), command.clone());
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    tx.state_mut().cards.get_mut(target_hand[0]).unwrap().suit = Suit::Spade;
    assert!(target_legal(&tx));
    let hp = tx.state().players.get(target).unwrap().hp;
    let (after, accepted) = apply_command(tx.state().clone(), command);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(after.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn olhexu_counts_distinct_non_damage_hand_categories_at_query_time() {
    let (mut tx, owner) = loaded(SP, 20_260_812);
    strip_skills(&mut tx, owner);
    let cards = tx.state().hand_cards(owner);
    assert!(cards.len() >= 4);
    for card in &cards {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    let base = hand_limit(&tx, owner);
    add_skill(&mut tx, owner, 3_135_057_536);
    assert_eq!(hand_limit(&tx, owner), base);

    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::Tao;
    assert_eq!(hand_limit(&tx, owner), base + 1);
    tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickWuzhong;
    assert_eq!(hand_limit(&tx, owner), base + 2);
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Weapon;
    assert_eq!(hand_limit(&tx, owner), base + 3);
    tx.state_mut().cards.get_mut(cards[3]).unwrap().kind = CardKind::Shan;
    assert_eq!(hand_limit(&tx, owner), base + 3);

    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::TrickJuedou;
    tx.state_mut().cards.get_mut(cards[3]).unwrap().kind = CardKind::ShaFire;
    assert_eq!(hand_limit(&tx, owner), base + 2);
}

#[test]
fn jigong2_uses_current_turn_damage_as_the_hand_limit_base() {
    let (mut tx, owner) = loaded(YIJIANG, 20_260_813);
    for player in tx.state().players.alive_by_seat() {
        strip_skills(&mut tx, player);
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 8;
        state.max_hp = 8;
    }
    add_skill(&mut tx, owner, 3_671_794_947);
    assert_eq!(hand_limit(&tx, owner), 0);

    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    schedule_damage(&mut tx, owner, other, 2).unwrap();
    assert_eq!(hand_limit(&tx, owner), 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(hand_limit(&tx, owner), 2);

    schedule_damage(&mut tx, other, owner, 1).unwrap();
    assert_eq!(hand_limit(&tx, owner), 2);

    tx.state_mut().turn.phase = engine_core::state::Phase::End;
    let before = tx.state().clone();
    let (after, result) = apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(20_260_813),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        after.turn.flag(engine_core::state::turn_flag::DAMAGE_DEALT),
        0
    );
    assert!(!after
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == 3_671_794_947));
    let expected = after.players.get(owner).unwrap().hp.max(0);
    assert_eq!(hand_limit(&Transaction::begin(after), owner), expected);
}

#[test]
fn jigong_adds_two_cards_and_expires_its_hand_limit_override_at_turn_end() {
    const JIGONG: u32 = 2_776_333_113;
    const JIGONG2: u32 = 3_671_794_947;

    let (mut tx, owner) = loaded(YIJIANG, 20_260_818);
    for player in tx.state().players.alive_by_seat() {
        strip_skills(&mut tx, player);
    }
    add_skill(&mut tx, owner, JIGONG);
    let hand = tx.state().hand_count(owner);

    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Play.to_u8(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_818),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == JIGONG2));
    assert_eq!(hand_limit(&Transaction::begin(state.clone()), owner), 0);

    let mut state = state;
    state.turn.phase = engine_core::state::Phase::End;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(20_260_819),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == JIGONG2));
}

#[test]
fn effective_suit_modifiers_cover_cards_judgments_and_heart_equipment() {
    let (mut inari, owner) = loaded(KEY, 20_260_814);
    strip_skills(&mut inari, owner);
    let club = inari.state().hand_cards(owner)[0];
    inari.state_mut().cards.get_mut(club).unwrap().suit = Suit::Club;
    let base = hand_limit(&inari, owner);
    add_skill(&mut inari, owner, 2_733_421_794);
    assert_eq!(hand_limit(&inari, owner), base + 1);
    assert_eq!(inari.state().cards.get(club).unwrap().suit, Suit::Club);
    assert_eq!(
        engine_core::query::effective_card_suit(
            inari.state(),
            owner,
            inari.state().cards.get(club).unwrap()
        ),
        Suit::Diamond
    );

    let (mut olhongyan, owner) = loaded(REFRESH, 20_260_815);
    strip_skills(&mut olhongyan, owner);
    let max_hp = olhongyan.state().players.get(owner).unwrap().max_hp;
    olhongyan.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 1;
    add_skill(&mut olhongyan, owner, 3_860_377_728);
    assert_eq!(hand_limit(&olhongyan, owner), max_hp as i32 - 1);
    let spade_equip = equip_mount(&mut olhongyan, owner, CardKind::HorseOff);
    assert_eq!(
        olhongyan.state().cards.get(spade_equip).unwrap().suit,
        Suit::Spade
    );
    assert_eq!(hand_limit(&olhongyan, owner), max_hp as i32);

    let (mut hongyan, owner) = loaded(SHENHUA, 20_260_816);
    strip_skills(&mut hongyan, owner);
    add_skill(&mut hongyan, owner, 3_138_402_391);
    let delayed = hongyan.state().hand_cards(owner)[0];
    hongyan.state_mut().cards.get_mut(delayed).unwrap().kind = CardKind::TrickLe;
    let hand = hongyan.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let judge = hongyan.state().zones.find(ZoneKind::Judge, owner).unwrap();
    run_effects(
        &mut hongyan,
        vec![EffectRequest::MoveCards {
            cards: vec![delayed],
            to: judge,
            from: Some(hand),
        }],
    )
    .unwrap();
    let draw = hongyan
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = *hongyan
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .last()
        .unwrap();
    hongyan.state_mut().cards.get_mut(top).unwrap().suit = Suit::Spade;
    hongyan.state_mut().turn.phase = engine_core::state::Phase::Judge;
    let before = hongyan.state().clone();
    let (after, result) = apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(20_260_816),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!after.turn.skip_play());
    assert_eq!(after.cards.get(top).unwrap().suit, Suit::Spade);
}

#[test]
fn xiying2_blocked_tao_is_skipped_without_a_dying_choice() {
    let (mut tx, owner) = loaded(SP2, 20_260_762);
    strip_skills(&mut tx, owner);
    add_skill(&mut tx, owner, 3_555_033_084);
    let tao = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = engine_core::state::CardKind::Tao;
    assert!(engine_core::query::card_save_cancelled_for(
        tx.state(),
        owner
    ));
    let source = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let events = schedule_damage(&mut tx, source, owner, 99).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert!(!tx.state().players.get(owner).unwrap().alive);
    assert!(events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::PlayerKilled { player, .. } if *player == owner
    )));
}

#[test]
fn qiaoshui2_blocks_tricks_but_not_basic_cards() {
    let (mut tx, owner) = loaded(YIJIANG, 20_260_763);
    strip_skills(&mut tx, owner);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let cards = tx.state().hand_cards(owner);
    assert!(cards.len() >= 2);
    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind =
        engine_core::state::CardKind::TrickWuzhong;
    tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = engine_core::state::CardKind::Jiu;
    add_skill(&mut tx, owner, 2_280_240_537);

    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        cards[0]
    ));
    assert!(!engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        cards[1]
    ));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: cards[0]
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(false)
    ));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: cards[1]
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));

    let before = tx.state().clone();
    let (after, pipe) = apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(20_260_764),
            kind: CommandKind::PlayCard {
                card: cards[0],
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(after, before);
}

#[test]
fn reanjian3_blocked_tao_is_skipped_without_a_dying_choice() {
    let (mut tx, owner) = loaded(YIJIANG, 20_260_765);
    strip_skills(&mut tx, owner);
    add_skill(&mut tx, owner, 4_203_756_175);
    let tao = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = engine_core::state::CardKind::Tao;
    assert!(engine_core::query::card_save_cancelled_for(
        tx.state(),
        owner
    ));

    let source = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let events = schedule_damage(&mut tx, source, owner, 99).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert!(!tx.state().players.get(owner).unwrap().alive);
    assert!(events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::PlayerKilled { player, .. } if *player == owner
    )));
}

#[test]
fn ttt_prevents_tao_targeting_in_queries_commands_and_saves() {
    let (mut tx, owner) = loaded(YIJIANG, 20_260_766);
    strip_skills(&mut tx, owner);
    add_skill(&mut tx, owner, 3_286_554_904);
    let tao = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = engine_core::state::CardKind::Tao;
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;

    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: tao
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(false)
    ));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::TargetLegal {
                user: owner,
                card: tao,
                target: owner
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(false)
    ));
    assert!(engine_core::query::card_save_cancelled_for(
        tx.state(),
        owner
    ));

    let before = tx.state().clone();
    let (after, pipe) = apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(20_260_766),
            kind: CommandKind::PlayCard {
                card: tao,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(after, before);
}

#[test]
fn two_legacy_skills_block_both_proactive_and_response_sha_channels() {
    for (index, (package, skill)) in [(YIJIANG, 3_940_214_578), (SP2, 2_389_818_943)]
        .into_iter()
        .enumerate()
    {
        let (mut tx, owner) = loaded(package, 20_260_726 + index as u64);
        add_skill(&mut tx, owner, skill);

        assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 0);
        assert!(engine_core::query::sha_response_cancelled_for(
            tx.state(),
            owner
        ));
    }
}

#[test]
fn three_xianzhen_temp_skills_share_sha_hand_limit_filtering() {
    for (index, (package, skill_def, blocks_use)) in [
        (REFRESH, 2_539_047_010, true),
        (REFRESH, 3_222_374_511, false),
        (YIJIANG, 3_077_071_581, true),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded(package, 20_260_727 + index as u64);
        add_skill(&mut tx, owner, skill_def);
        assert!(engine_core::query::sha_ignored_for_hand_limit(
            tx.state(),
            owner
        ));
        assert_eq!(
            engine_core::query::sha_use_limit_for(tx.state(), owner) == 0,
            blocks_use
        );
    }
}

#[test]
fn three_legacy_temp_skills_reduce_hand_limit_by_one() {
    for (index, (package, skill)) in [
        (EXTRA, 2_484_371_675),
        (SP, 4_107_407_259),
        (SP2, 2_653_426_854),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded(package, 20_260_730 + index as u64);
        let base = hand_limit(&tx, owner);
        add_skill(&mut tx, owner, skill);
        assert_eq!(hand_limit(&tx, owner), base - 1);
    }
}

#[test]
fn sp_equipment_modifiers_share_public_queries() {
    let (mut sanlve, owner) = loaded(SP, 20_260_740);
    let base_hand = hand_limit(&sanlve, owner);
    let base_range = attack_range(&sanlve, owner);
    let base_sha = engine_core::query::sha_use_limit_for(sanlve.state(), owner);
    add_skill(&mut sanlve, owner, 3_136_590_309);
    assert_eq!(hand_limit(&sanlve, owner), base_hand + 1);
    assert_eq!(attack_range(&sanlve, owner), base_range + 1);
    assert_eq!(
        engine_core::query::sha_use_limit_for(sanlve.state(), owner),
        base_sha + 1
    );

    let (mut crown, owner) = loaded(SP, 20_260_741);
    let base_hand = hand_limit(&crown, owner);
    let base_draw = engine_core::query::draw_count_for(crown.state(), owner);
    add_skill(&mut crown, owner, 3_228_154_738);
    assert_eq!(hand_limit(&crown, owner), base_hand - 1);
    assert_eq!(
        engine_core::query::draw_count_for(crown.state(), owner),
        base_draw + 2
    );
}

#[test]
fn yijiang_package_is_reproducible_and_exposes_range_modifiers() {
    let committed = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut compiled =
        content_compiler::compile_source_named("yijiang", &yijiang_source()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);

    let (mut jiangchi, owner) = loaded(YIJIANG, 20_260_750);
    strip_skills(&mut jiangchi, owner);
    add_skill(&mut jiangchi, owner, 4_168_410_091);
    assert_eq!(attack_range(&jiangchi, owner), i32::MAX);
    assert_eq!(
        engine_core::query::sha_use_limit_for(jiangchi.state(), owner),
        2
    );

    let (mut gongji, owner) = loaded(YIJIANG, 20_260_751);
    strip_skills(&mut gongji, owner);
    add_skill(&mut gongji, owner, 2_776_126_573);
    assert_eq!(attack_range(&gongji, owner), i32::MAX);
}

#[test]
fn jinjiu_uses_the_effective_card_name_without_mutating_the_card() {
    let (mut tx, owner) = loaded(YIJIANG, 20_260_750_1);
    strip_skills(&mut tx, owner);
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Jiu;
    add_skill(&mut tx, owner, 2_885_443_165);

    let physical = tx.state().cards.get(card).unwrap();
    assert_eq!(physical.kind, CardKind::Jiu);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert_eq!(tx.state().cards.get(card).unwrap().kind, CardKind::Jiu);
}

#[test]
fn sxrm_old_wushen_is_reproducible_and_ignores_distance_for_heart_sha() {
    let committed = content_schema::ContentPackage::from_json_bytes(SXRM).unwrap();
    let mut compiled = content_compiler::compile_source_named("sxrm", SXRM_SOURCE).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);

    let (mut tx, owner) = loaded_players(SXRM, 20_260_750_2, 4);
    strip_skills(&mut tx, owner);
    let seats = tx.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let card = tx.state().hand_cards(owner)[0];
    {
        let physical = tx.state_mut().cards.get_mut(card).unwrap();
        physical.kind = CardKind::Tao;
        physical.suit = Suit::Heart;
    }
    add_skill(&mut tx, owner, 3_444_544_012);

    let physical = tx.state().cards.get(card).unwrap();
    assert_eq!(physical.kind, CardKind::Tao);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert!(engine_core::query::card_target_within_distance(
        tx.state(),
        owner,
        card,
        far,
        1
    ));
    assert_eq!(tx.state().cards.get(card).unwrap().kind, CardKind::Tao);
}

#[test]
fn owner_lost_hp_values_follow_current_hp_for_three_legacy_skills() {
    let (mut qianju, owner) = loaded_players(YIJIANG, 20_260_752, 4);
    strip_skills(&mut qianju, owner);
    let seats = qianju.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    assert_eq!(distance(&qianju, owner, far), 2);
    add_skill(&mut qianju, owner, 2_828_017_083);
    assert_eq!(distance(&qianju, owner, far), 2);
    let player = qianju.state_mut().players.get_mut(owner).unwrap();
    player.hp = player.max_hp as i32 - 2;
    assert_eq!(distance(&qianju, owner, far), 1);

    let (mut reqianju, owner) = loaded_players(YIJIANG, 20_260_753, 4);
    strip_skills(&mut reqianju, owner);
    let seats = reqianju.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    assert_eq!(distance(&reqianju, owner, far), 2);
    add_skill(&mut reqianju, owner, 3_293_272_720);
    assert_eq!(distance(&reqianju, owner, far), 1);

    let (mut dcniyun, owner) = loaded(COLLAB, 20_260_754);
    strip_skills(&mut dcniyun, owner);
    let base_range = attack_range(&dcniyun, owner);
    let base_sha = engine_core::query::sha_use_limit_for(dcniyun.state(), owner);
    add_skill(&mut dcniyun, owner, 3_024_539_645);
    assert_eq!(attack_range(&dcniyun, owner), base_range);
    assert_eq!(
        engine_core::query::sha_use_limit_for(dcniyun.state(), owner),
        base_sha
    );
    let player = dcniyun.state_mut().players.get_mut(owner).unwrap();
    player.hp = player.max_hp as i32 - 2;
    assert_eq!(attack_range(&dcniyun, owner), base_range + 2);
    assert_eq!(
        engine_core::query::sha_use_limit_for(dcniyun.state(), owner),
        base_sha + 2
    );
}

#[test]
fn guixiu_only_draws_when_targeted_by_sha_below_hp() {
    const GUIXIU: u32 = 2_533_816_770;

    let (mut tx, owner) = loaded_players(SP, 20_260_786, 2);
    for player in tx.state().players.alive_by_seat() {
        strip_skills(&mut tx, player);
    }
    add_skill(&mut tx, owner, GUIXIU);
    let source = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp.max(1) as u32;
    let discard_count = tx
        .state()
        .hand_count(owner)
        .saturating_sub(hp.saturating_sub(1));
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: discard_count,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_count(owner) < tx.state().players.get(owner).unwrap().hp as u32);

    let cards = tx.state().hand_cards(source);
    assert!(cards.len() >= 3);
    for card in cards.iter().take(3) {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[0],
            targets: vec![source],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    let hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: source,
                card: cards[1],
                targets: vec![owner],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_786),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(
        state.hand_count(owner),
        state.players.get(owner).unwrap().hp as u32
    );

    let mut full_hand = Transaction::begin(state);
    run_effects(
        &mut full_hand,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[2],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert!(!full_hand.state().has_pending_choice());
}

#[test]
fn owner_skill_mark_values_cover_twenty_three_legacy_modifier_skills() {
    for (index, (package, skill, direction)) in [
        (HUICUI, 3_314_624_992, -1),
        (KEY, 3_115_638_476, 1),
        (KEY, 2_654_574_327, 1),
        (KEY, 3_086_026_740, 1),
        (MOBILE, 3_536_791_091, -1),
        (REFRESH, 3_547_066_515, 1),
        (STANDARD, 2_737_456_606, -1),
        (YIJIANG, 3_510_775_701, -1),
        (YIJIANG, 3_163_507_862, 1),
        (YINGBIAN, 3_192_554_646, -1),
        (CLAN, 3_882_274_053, 1),
        (DIY, 3_040_816_625, 1),
        (HUICUI, 3_326_879_274, 1),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded(package, 20_260_760 + index as u64);
        strip_skills(&mut tx, owner);
        let base = hand_limit(&tx, owner);
        add_skill(&mut tx, owner, skill);
        add_mark(&mut tx, owner, skill, 2);
        assert_eq!(hand_limit(&tx, owner), base + direction * 2);
    }

    for (index, (package, skill, increases)) in [
        (KEY, 3_115_638_476, true),
        (REFRESH, 3_848_225_310, true),
        (SHENHUA, 3_951_873_955, true),
        (SP, 4_020_090_823, true),
        (SP, 3_847_747_290, true),
        (XIANDING, 3_944_235_806, true),
        (YIJIANG, 3_364_392_152, false),
        (SHENHUA, 2_727_616_630, true),
        (REFRESH, 2_749_452_350, true),
        (YIJIANG, 3_163_507_862, true),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded(package, 20_260_770 + index as u64);
        strip_skills(&mut tx, owner);
        let base = engine_core::query::sha_use_limit_for(tx.state(), owner);
        add_skill(&mut tx, owner, skill);
        add_mark(&mut tx, owner, skill, 2);
        let expected = if increases {
            base + 2
        } else {
            base.saturating_sub(2)
        };
        assert_eq!(
            engine_core::query::sha_use_limit_for(tx.state(), owner),
            expected
        );
    }

    for (index, (package, skill)) in [
        (OFFLINE, 4_257_277_562),
        (SHENHUA, 2_727_616_630),
        (REFRESH, 2_749_452_350),
    ]
    .into_iter()
    .enumerate()
    {
        let (mut tx, owner) = loaded_players(package, 20_260_780 + index as u64, 4);
        strip_skills(&mut tx, owner);
        let seats = tx.state().players.alive_by_seat();
        let owner_index = seats.iter().position(|player| *player == owner).unwrap();
        let far = seats[(owner_index + 2) % seats.len()];
        let base = distance(&tx, owner, far);
        add_skill(&mut tx, owner, skill);
        add_mark(&mut tx, owner, skill, 1);
        assert_eq!(distance(&tx, owner, far), (base - 1).max(1));
    }

    let (mut rebiluan, owner) = loaded_players(SP, 20_260_783, 4);
    strip_skills(&mut rebiluan, owner);
    let from = rebiluan
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let base = distance(&rebiluan, from, owner);
    add_skill(&mut rebiluan, owner, 3_075_740_841);
    add_mark(&mut rebiluan, owner, 3_075_740_841, 2);
    assert_eq!(distance(&rebiluan, from, owner), base + 2);
    add_mark(&mut rebiluan, owner, 3_075_740_841, -3);
    assert_eq!(distance(&rebiluan, from, owner), (base - 1).max(1));
}

#[test]
fn ported_modifier_packages_are_reproducible() {
    std::thread::Builder::new().stack_size(16 << 20).spawn(|| {
        for (name, source, package) in [
            ("sp", SP_SOURCE, SP),
            ("sp2", SP2_SOURCE, SP2),
            ("xianding", XIANDING_SOURCE, XIANDING),
            ("yingbian", YINGBIAN_SOURCE, YINGBIAN),
        ] {
            let committed = content_schema::ContentPackage::from_json_bytes(package).unwrap();
            let mut compiled = content_compiler::compile_source_named(name, source).unwrap();
            compiled.character_index = committed.character_index.clone();
            compiled.manifest.dependencies = committed.manifest.dependencies.clone();
            compiled.manifest.logic_hash = compiled.compute_logic_hash();
            assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
        }
    });
}
}
