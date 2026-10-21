//! Golden for the first complete historical one-skill character: 标曹真「司敌」.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const STD_CAOZHEN: u32 = 2_623_502_245;
const STD_CAOJINYU: u32 = 2_785_864_584;
const STD_QUYI: u32 = 2_888_834_865;
const STD_NIUJIN: u32 = 3_712_372_828;
const STD_DB_WENYANG: u32 = 2_396_957_062;
const STDSIDI: u32 = 4_128_943_540;
const STDSHIBEI: u32 = 2_559_343_711;
const STDDULIANG: u32 = 2_231_518_236;
const STDHUANGONG: u32 = 2_543_694_356;
const STDQIANSU: u32 = 2_964_345_042;
const STDWUYAN: u32 = 3_164_633_937;
const STDSHANSHEN: u32 = 2_423_225_569;
const STDZHONGYAN: u32 = 3_353_809_882;
const STDANRAN: u32 = 2_536_614_283;
const STDYUQI: u32 = 3_561_403_597;
const STDQINGMAN: u32 = 3_965_321_062;
const STDJIAOZI: u32 = 3_194_342_337;
const STDFUQI: u32 = 2_567_338_825;
const STD_MENGDA: u32 = 3_789_552_174;
const STDZHUAN: u32 = 4_034_502_794;
const STDCUORUI: u32 = 2_683_321_015;
const STDQUEDI: u32 = 3_311_025_145;
const STD_LVFAN: u32 = 3_319_229_677;
const MBDIAODU: u32 = 2_440_438_931;
const STDDIANFENG: u32 = 3_414_143_685;
const STD_ZHANGYI: u32 = 3_037_060_961;
const STDZHIYI: u32 = 3_959_114_365;
const STDTIAOHE: u32 = 2_595_769_846;
const STD_FEIYI: u32 = 2_878_317_274;
const STD_ZHONGHUI: u32 = 2_397_163_146;
const STDXINGFA: u32 = 2_490_051_933;
const STD_LIUBA: u32 = 2_185_226_076;
const STDDUANBI: u32 = 3_811_377_323;
const STD_JIAKUI: u32 = 3_406_075_592;
const STDZHONGZUO: u32 = 3_465_971_059;
const STDWANLAN: u32 = 3_664_427_038;
const STD_WANGYUN: u32 = 3_976_039_480;
const STDYUNJI: u32 = 2_318_160_114;
const STDZONGJI: u32 = 3_394_363_597;
const STDDUANBING: u32 = 2_170_694_873;
const STD_DONGYUN: u32 = 2_711_327_184;
const STDBINGZHENG: u32 = 3_798_362_737;
const STD_WUKE: u32 = 4_182_138_442;
const STDANDA: u32 = 3_646_320_211;
const STDZHUGUO: u32 = 3_147_233_152;
const STD_DINGFENG: u32 = 3_454_509_574;
const STDFENXUN: u32 = 2_579_637_455;
const STD_LIUZAN: u32 = 4_100_887_402;
const STD_LIUBIAO: u32 = 2_470_672_462;
const STD_LIJUE: u32 = 2_852_318_654;
const STD_ZHANGFEN: u32 = 3_289_092_799;
const STD_BAOXIN: u32 = 2_419_830_352;
const STD_GANFUREN: u32 = 2_343_280_510;
const STD_WENYUAN: u32 = 4_166_759_034;
const STD_CENHUN: u32 = 3_739_444_871;
const STD_HUANGFUSONG: u32 = 3_128_139_054;
const STD_SUNHAO: u32 = 4_237_199_211;
const STDFENYIN: u32 = 2_225_023_788;
const STDZISHOU: u32 = 2_683_293_404;
const ZONGSHI: u32 = 2_639_072_445;
const STDJUJIN: u32 = 3_408_182_272;
const STDXIONGSUAN: u32 = 3_084_450_928;
const STDWANGLU: u32 = 3_938_702_468;
const STDMUTAO: u32 = 3_513_095_743;
const STDYIMOU: u32 = 4_237_148_705;
const STDSHUSHENX: u32 = 3_633_500_345;
const STDZHIJIE: u32 = 2_635_655_868;
const STDKENGQIANG: u32 = 3_956_037_825;
const STDSHANGJUE: u32 = 3_835_885_608;
const STDJISHE: u32 = 2_148_577_926;
const STDWUDU: u32 = 4_151_205_179;
const STDTAOLUAN: u32 = 3_364_139_751;
const STDCANSHI: u32 = 2_290_683_272;
const STDCANSHI_EFFECT: u32 = 2_335_115_700;
const CHOUHAI: u32 = 2_265_775_915;
const GUIMING: u32 = 3_828_399_214;
const STDANRAN_USED_MARK: u32 = 221_181_016;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn start(seed: u64, player_count: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    inject_into(tx, owner, ZoneKind::Hand, kind, suit, rank)
}

fn inject_into(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::new(9_200 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let id = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(id);
    id
}

#[test]
fn committed_package_matches_source_and_complete_character() {
    let committed = package();

    assert_eq!(
        committed.manifest.logic_hash,
        committed.compute_logic_hash()
    );
    assert_eq!(committed.skill_index.len(), 181);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSIDI && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHIBEI && skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDULIANG && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDHUANGONG && skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDQUEDI && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDTIAOHE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDDUANBI && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDMUTAO && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDYIMOU && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHUSHENX && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHIJIE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDKENGQIANG && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDSHANGJUE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDJISHE && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDWUDU && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDTAOLUAN && !skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDCANSHI && skill.locked));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDCANSHI_EFFECT && skill.locked));
    assert_eq!(committed.character_index.len(), 101);
    let find_char = |id: u32| committed.character_index.iter().find(|c| c.id == id);
    let caozhen = find_char(STD_CAOZHEN).unwrap();
    assert_eq!(caozhen.name_key, "std_caozhen");
    assert_eq!(caozhen.max_hp, 4);
    assert_eq!(caozhen.skills, vec![STDSIDI]);
    let caojinyu = find_char(STD_CAOJINYU).unwrap();
    assert_eq!(caojinyu.skills, vec![STDYUQI, STDSHANSHEN]);
    let mengda = find_char(STD_MENGDA).unwrap();
    assert_eq!(mengda.name_key, "std_mengda");
    assert_eq!(mengda.max_hp, 4);
    assert_eq!(mengda.skills, vec![STDZHUAN]);
    let quyi = find_char(STD_QUYI).unwrap();
    assert_eq!(quyi.name_key, "std_quyi");
    assert_eq!(quyi.max_hp, 4);
    assert_eq!(quyi.skills, vec![STDFUQI, STDJIAOZI]);
    let niujin = find_char(STD_NIUJIN).unwrap();
    assert_eq!(niujin.name_key, "std_niujin");
    assert_eq!(niujin.max_hp, 4);
    let niujin = find_char(STD_NIUJIN).unwrap();
    assert_eq!(niujin.skills, vec![STDCUORUI]);
    let db_wenyang = find_char(STD_DB_WENYANG).unwrap();
    assert_eq!(db_wenyang.name_key, "std_db_wenyang");
    assert_eq!(db_wenyang.max_hp, 4);
    assert_eq!(db_wenyang.skills, vec![STDQUEDI]);
    let lvfan = find_char(STD_LVFAN).unwrap();
    assert_eq!(lvfan.name_key, "std_lvfan");
    assert_eq!(lvfan.skills, vec![MBDIAODU, STDDIANFENG]);
    let zhangyi = find_char(STD_ZHANGYI).unwrap();
    assert_eq!(zhangyi.name_key, "std_zhangyì");
    assert_eq!(zhangyi.max_hp, 4);
    assert_eq!(zhangyi.skills, vec![STDZHIYI]);
    let feiyi = find_char(STD_FEIYI).unwrap();
    assert_eq!(feiyi.name_key, "std_feiyi");
    assert_eq!(feiyi.max_hp, 3);
    assert_eq!(feiyi.skills, vec![STDTIAOHE, STDQIANSU]);
    let zhonghui = find_char(STD_ZHONGHUI).unwrap();
    assert_eq!(zhonghui.name_key, "std_zhonghui");
    assert_eq!(zhonghui.max_hp, 4);
    let zhonghui_xingfa = find_char(STD_ZHONGHUI).unwrap();
    assert_eq!(zhonghui_xingfa.skills, vec![STDXINGFA]);
    let liuba = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBA)
        .unwrap();
    assert_eq!(liuba.name_key, "std_liuba");
    assert_eq!(liuba.max_hp, 3);
    assert_eq!(liuba.skills, vec![STDDUANBI]);
    let jiakui = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_JIAKUI)
        .unwrap();
    assert_eq!(jiakui.name_key, "std_jiakui");
    assert_eq!(jiakui.max_hp, 3);
    assert_eq!(jiakui.skills, vec![STDZHONGZUO, STDWANLAN]);
    let wangyun = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_WANGYUN)
        .unwrap();
    assert_eq!(wangyun.name_key, "std_wangyun");
    assert_eq!(wangyun.max_hp, 3);
    assert_eq!(wangyun.skills, vec![STDYUNJI, STDZONGJI]);
    let dongyun = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_DONGYUN)
        .unwrap();
    assert_eq!(dongyun.name_key, "std_dongyun");
    assert_eq!(dongyun.max_hp, 3);
    assert_eq!(dongyun.skills, vec![STDBINGZHENG, STDDULIANG]);
    let wuke = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_WUKE)
        .unwrap();
    assert_eq!(wuke.name_key, "std_wuke");
    assert_eq!(wuke.max_hp, 3);
    assert_eq!(wuke.skills, vec![STDANDA, STDZHUGUO]);
    let dingfeng = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_DINGFENG)
        .unwrap();
    assert_eq!(dingfeng.name_key, "std_dingfeng");
    assert_eq!(dingfeng.max_hp, 4);
    assert_eq!(dingfeng.skills, vec![STDDUANBING, STDFENXUN]);
    let liuzan = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUZAN)
        .unwrap();
    assert_eq!(liuzan.name_key, "std_liuzan");
    assert_eq!(liuzan.max_hp, 4);
    assert_eq!(liuzan.skills, vec![STDFENYIN]);
    let liubiao = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUBIAO)
        .unwrap();
    assert_eq!(liubiao.name_key, "std_liubiao");
    assert_eq!(liubiao.max_hp, 3);
    assert_eq!(liubiao.skills, vec![STDZISHOU, ZONGSHI, STDJUJIN]);
    let lijue = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_LIJUE)
        .unwrap();
    assert_eq!(lijue.name_key, "std_lijue");
    assert_eq!(lijue.max_hp, 5);
    assert_eq!(lijue.skills, vec![STDXIONGSUAN]);
    let zhangfen = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_ZHANGFEN)
        .unwrap();
    assert_eq!(zhangfen.name_key, "std_zhangfen");
    assert_eq!(zhangfen.max_hp, 3);
    assert_eq!(zhangfen.skills, vec![STDWANGLU]);
    let baoxin = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_BAOXIN)
        .unwrap();
    assert_eq!(baoxin.name_key, "std_baoxin");
    assert_eq!(baoxin.max_hp, 4);
    assert_eq!(baoxin.skills, vec![STDYIMOU, STDMUTAO]);
    let ganfuren = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_GANFUREN)
        .unwrap();
    assert_eq!(ganfuren.name_key, "std_ganfuren");
    assert_eq!(ganfuren.max_hp, 3);
    assert_eq!(ganfuren.skills, vec![STDZHIJIE, STDSHUSHENX]);
    let wenyuan = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_WENYUAN)
        .unwrap();
    assert_eq!(wenyuan.name_key, "std_wenyuan");
    assert_eq!(wenyuan.max_hp, 3);
    assert_eq!(wenyuan.skills, vec![STDKENGQIANG, STDSHANGJUE]);
    let cenhun = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_CENHUN)
        .unwrap();
    assert_eq!(cenhun.name_key, "std_cenhun");
    assert_eq!(cenhun.max_hp, 3);
    assert_eq!(cenhun.skills, vec![STDJISHE, STDWUDU]);
    let huangfusong = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_HUANGFUSONG)
        .unwrap();
    assert_eq!(huangfusong.name_key, "std_huangfusong");
    assert_eq!(huangfusong.max_hp, 4);
    assert_eq!(huangfusong.skills, vec![STDTAOLUAN]);
    let sunhao = committed
        .character_index
        .iter()
        .find(|character| character.id == STD_SUNHAO)
        .unwrap();
    assert_eq!(sunhao.name_key, "std_sunhao");
    assert_eq!(sunhao.max_hp, 5);
    assert_eq!(sunhao.skills, vec![STDCANSHI, CHOUHAI, GUIMING]);
}

#[test]
fn stdyuqi_uses_event_player_to_owner_distance_and_owner_hp() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_716, 4));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDYUQI),
        }],
    )
    .unwrap();
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .max_by_key(|player| {
            match engine_core::query::query(
                tx.state(),
                &rule_api::QueryRequest::Distance {
                    from: *player,
                    to: owner,
                },
                &mut engine_core::query::QueryContext::new(),
            ) {
                rule_api::QueryResponse::Int(distance) => distance,
                _ => 0,
            }
        })
        .unwrap();
    let source = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner && *player != target)
        .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(target);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), hand + 1);

    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(target), hand + 1);
}

#[test]
fn stdqingman_draws_to_three_or_own_empty_equip_slots() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_717, 2));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDQINGMAN),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::SetTurnState {
            current: other,
            phase: Phase::End.to_u8(),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    assert_eq!(tx.state().hand_count(owner), 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        5,
    );
    inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        6,
    );
    inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Treasure,
        Suit::Club,
        7,
    );
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);
}

#[test]
fn stdwuyan_uses_wuxie_as_the_effective_name_for_physical_tricks() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_714, 2));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDWUYAN),
        }],
    )
    .unwrap();
    let card = inject(&mut tx, owner, CardKind::TrickJuedou, Suit::Spade, 1);
    let physical = tx.state().cards.get(card).unwrap();
    assert_eq!(physical.kind, CardKind::TrickJuedou);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::TrickWuxie
    );
    assert_eq!(
        tx.state().cards.get(card).unwrap().kind,
        CardKind::TrickJuedou
    );
}

#[test]
fn stdhuangong_draws_only_after_the_last_field_card_leaves() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_715, 2));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDHUANGONG),
        }],
    )
    .unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let weapon = inject_into(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        5,
    );
    let delayed = inject_into(
        &mut tx,
        owner,
        ZoneKind::Judge,
        CardKind::TrickLe,
        Suit::Heart,
        6,
    );
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![delayed],
            to: discard,
            from: Some(judge),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn stdshibei_recovers_only_after_first_damage() {
    let committed = package();
    let mut engine = Engine::new();
    engine.submit(start(20_260_713, 2));
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDSHIBEI),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert!(!tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.def.raw() == STDSHIBEI));

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn stdduliang_checks_hand_count_after_drawing() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_714, 2));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDDULIANG),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 2,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    assert_eq!(hand, 2);

    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
}

#[test]
fn stdsidi_only_offers_draw_for_sha_response() {
    let committed = package();
    let mut engine = Engine::new();
    engine.submit(start(20_260_713, 2));
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;

    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDSIDI),
        }],
    )
    .unwrap();
    let hand_before = tx.state().hand_count(owner);
    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: CardId::NONE,
            response_kind: CardKind::Sha.tag(),
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert!(tx.state().has_pending_choice());

    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(!state.has_pending_choice());
    let mut non_sha = Transaction::begin(state);
    let hand_before = non_sha.state().hand_count(owner);
    run_effects(
        &mut non_sha,
        vec![EffectRequest::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert_eq!(non_sha.state().hand_count(owner), hand_before);
    assert!(!non_sha.state().has_pending_choice());
}

#[test]
fn optional_stdsidi_draw_resumes_juedou_exchange() {
    let committed = package();
    let mut engine = Engine::new();
    engine.submit(start(73, 4));
    engine.load_package(&committed).unwrap();
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;

    // Isolate the response/continuation path from standard character triggers.
    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: actor,
        skill_def: SkillDefId::new(STDSIDI),
    });
    run_effects(&mut tx, effects).unwrap();

    // Remove random response/trick kinds while keeping the canonical zones intact.
    let hand_cards: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .flat_map(|player| tx.state().hand_cards(player.id))
        .collect();
    for card in hand_cards {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }

    let juedou = inject(&mut tx, actor, CardKind::TrickJuedou, Suit::Diamond, 1);
    let actor_sha = inject(&mut tx, actor, CardKind::Sha, Suit::Spade, 7);
    let target_sha = inject(&mut tx, target, CardKind::Sha, Suit::Heart, 8);

    let (state, played) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: juedou,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let response = played.choice.expect("target should be asked for 杀");
    assert_eq!(response.observer, target);

    let (state, responded) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: response.choice_id,
                selections: vec![target_sha.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(responded.result, CommandResult::Accepted { .. }));
    let confirm = responded.choice.expect("司敌 should ask its owner");
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(confirm.observer, actor);
    let hand_before = state.hand_count(actor);

    let (state, resumed) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resumed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(actor), hand_before + 1);
    let next = resumed
        .choice
        .expect("决斗 must resume and ask the other side for the next 杀");
    assert_eq!(next.observer, actor);
    assert!(next
        .options
        .iter()
        .any(|option| option.token == actor_sha.raw()));
}

#[test]
fn stdqiansu_only_draws_when_targeted_by_a_trick_with_empty_equip_zone() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_716, 2));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());

    let skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDQIANSU),
    });
    run_effects(&mut tx, effects).unwrap();

    let ignored = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Club, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: ignored,
            targets: vec![source],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    let targeted = inject(&mut tx, source, CardKind::TrickWuzhong, Suit::Diamond, 2);
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: source,
                card: targeted,
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
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let mut equipped = Transaction::begin(state);
    inject_into(
        &mut equipped,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        3,
    );
    let blocked = inject(
        &mut equipped,
        source,
        CardKind::TrickWuzhong,
        Suit::Heart,
        4,
    );
    let hand = equipped.state().hand_count(owner);
    run_effects(
        &mut equipped,
        vec![EffectRequest::UseCard {
            player: source,
            card: blocked,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(equipped.state().hand_count(owner), hand);
    assert!(!equipped.state().has_pending_choice());
}

fn death_recovery_case(
    skill_def: u32,
    victim_character: u32,
    owner_is_killer: bool,
) -> (
    Transaction,
    PlayerId,
    Result<Vec<engine_api::DomainEvent>, EffectError>,
) {
    let mut committed = package();
    committed.character_index.extend(
        [(9_216_001, "wei"), (9_216_002, "qun"), (9_216_003, "shu")]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                id,
                name_key: format!("death_group_{id}"),
                max_hp: 4,
                initial_hp: 4,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            }),
    );
    committed.manifest.logic_hash = committed.compute_logic_hash();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(skill_def as u64, 4));
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.identity == Identity::Lord)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let victim = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.character_def = 9_216_001;
        owner_state.hp = 2;
        owner_state.max_hp = 4;
    }
    tx.state_mut()
        .players
        .get_mut(victim)
        .unwrap()
        .character_def = victim_character;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: if owner_is_killer {
                owner
            } else {
                PlayerId::NONE
            },
        }],
    );
    (tx, owner, result)
}

fn accept_optional_recovery(tx: Transaction, owner: PlayerId) -> engine_core::state::GameState {
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "death recovery choice rejected: {:?}",
        result.result
    );
    state
}

#[test]
fn stdshanshen_recovers_only_when_someone_else_caused_the_death() {
    let (tx, owner, result) = death_recovery_case(STDSHANSHEN, 9_216_002, false);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let state = accept_optional_recovery(tx, owner);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);

    let (tx, owner, result) = death_recovery_case(STDSHANSHEN, 9_216_002, true);
    result.unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}

#[test]
fn stdzhongyan_is_lord_only_and_requires_a_qun_death() {
    let (tx, owner, result) = death_recovery_case(STDZHONGYAN, 9_216_002, false);
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let state = accept_optional_recovery(tx, owner);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);

    let (tx, owner, result) = death_recovery_case(STDZHONGYAN, 9_216_003, false);
    result.unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}

#[test]
fn stdanran_draws_one_to_four_and_then_stays_capped() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_824, 2));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 10;
        player.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDANRAN),
        }],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let mut expected_draws = 0;
    let mut state = tx.state().clone();

    for activation in 1..=5 {
        let mut damage = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut damage,
                vec![EffectRequest::DealDamage {
                    source,
                    target: owner,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let (next, result) = apply_command(
            damage.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(activation),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(damage.state().resolution.pending_choice_id),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        expected_draws += activation.min(4) as u32;
        assert_eq!(next.hand_count(owner), initial_hand + expected_draws);
        assert_eq!(
            next.players
                .get(owner)
                .unwrap()
                .marks
                .iter()
                .find(|(mark, _)| *mark == STDANRAN_USED_MARK)
                .map_or(0, |(_, count)| *count),
            activation.min(3) as i32
        );
        state = next;
    }
}

#[test]
fn stdjiaozi_only_boosts_the_first_damage_when_tied_for_max_hand() {
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(start(20_260_825, 3));
    let owner = engine.state().turn.current;
    let mut others = engine
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id);
    let target = others.next().unwrap();
    let leader = others.next().unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    for player in [owner, target, leader] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    inject(&mut tx, owner, CardKind::Other, Suit::Spade, 1);
    inject(&mut tx, leader, CardKind::Other, Suit::Heart, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDJIAOZI),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = target;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 20;
    target_state.max_hp = 20;

    let damage = |tx: &mut Transaction| {
        run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
    };
    damage(&mut tx);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 18);
    damage(&mut tx);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 17);

    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDJIAOZI)
        .unwrap()
        .id;
    tx.state_mut()
        .skills
        .get_mut(skill_id)
        .unwrap()
        .turn_fire_count = 0;
    inject(&mut tx, leader, CardKind::Other, Suit::Club, 3);
    damage(&mut tx);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 16);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: leader,
            count: u32::MAX,
        }],
    )
    .unwrap();
    damage(&mut tx);
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        15,
        "首段伤害未满足手牌最多也应消耗本回合首次伤害窗口"
    );
}

#[test]
fn stdfuqi_locks_the_whole_sha_use_in_both_directions() {
    const MULTI_SHA: u32 = 4_290_000_001;
    let fixture = content_compiler::compile_source_named(
        "sixiang-test-fixture",
        "skill multi_sha(4290000001) { modifier card_distance_unlimited(sha) override 1; modifier sha_target_limit add 1; }",
    )
    .unwrap();
    let committed = package();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.load_package(&fixture).unwrap();
    engine.submit(start(20_260_826, 4));
    let source = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != source)
        .collect();
    let distance_to_source =
        |state: &engine_core::state::GameState, player| match engine_core::query::query(
            state,
            &QueryRequest::Distance {
                from: player,
                to: source,
            },
            &mut engine_core::query::QueryContext::new(),
        ) {
            QueryResponse::Int(distance) => distance,
            other => panic!("unexpected distance response: {other:?}"),
        };
    let near = *others
        .iter()
        .min_by_key(|player| distance_to_source(tx.state(), **player))
        .unwrap();
    let far = *others
        .iter()
        .max_by_key(|player| distance_to_source(tx.state(), **player))
        .unwrap();
    assert_eq!(distance_to_source(tx.state(), near), 1);
    assert!(distance_to_source(tx.state(), far) > 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(STDFUQI),
        }],
    )
    .unwrap();
    assert!(engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[near]
    ));
    assert!(!engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[far]
    ));

    let source_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDFUQI)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: source_skill,
            },
            EffectRequest::AddSkill {
                player: near,
                skill_def: SkillDefId::new(STDFUQI),
            },
        ],
    )
    .unwrap();
    assert!(engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[near]
    ));
    assert!(!engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[far]
    ));

    let target_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == STDFUQI)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: target_skill,
            },
            EffectRequest::AddSkill {
                player: source,
                skill_def: SkillDefId::new(STDFUQI),
            },
            EffectRequest::AddSkill {
                player: source,
                skill_def: SkillDefId::new(MULTI_SHA),
            },
        ],
    )
    .unwrap();
    for player in [source, near, far] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 7);
    inject(&mut tx, near, CardKind::Shan, Suit::Heart, 8);
    inject(&mut tx, far, CardKind::Shan, Suit::Diamond, 9);
    for target in [near, far] {
        let player = tx.state_mut().players.get_mut(target).unwrap();
        player.hp = 10;
        player.max_hp = 10;
    }
    assert!(engine_core::query::sha_use_unrespondable_within_distance(
        tx.state(),
        source,
        &[near, far]
    ));
    tx.state_mut().turn.phase = Phase::Play;
    let revision = tx.state().revision;
    let (next, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![near, far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!next.has_pending_choice());
    assert_eq!(next.players.get(near).unwrap().hp, 9);
    assert_eq!(next.players.get(far).unwrap().hp, 9);
}

#[test]
fn historical_character_joins_standard_roster_and_can_be_dealt() {
    let committed = package();
    let mut baseline = Engine::new();
    assert!(matches!(
        baseline.submit(start(1, 4)),
        CommandResult::Accepted { .. }
    ));
    let expected_character_count =
        baseline.content().characters().count() + committed.character_index.len();
    let mut selected_seed = None;
    for seed in 1..=128 {
        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        let result = engine.submit(start(seed, 4));
        assert!(matches!(result, CommandResult::Accepted { .. }));
        assert_eq!(
            engine.content().characters().count(),
            expected_character_count
        );
        if engine
            .state()
            .players
            .iter_live()
            .any(|player| player.character_def == STD_CAOZHEN)
        {
            selected_seed = Some(seed);
            break;
        }
    }
    assert!(
        selected_seed.is_some(),
        "std_caozhen must be reachable from the combined deterministic roster"
    );
}
