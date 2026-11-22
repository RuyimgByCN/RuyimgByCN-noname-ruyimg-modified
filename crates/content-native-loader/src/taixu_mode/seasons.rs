//! L11 太虚幻境 8 赛季 × 52 章节 `seasonPack` 流程与事件绑定总表。

use super::catalog::TaixuEffectSpec;
use super::season_events_a::L11_TAIXU_DLC_EVENTS_A;
use super::season_events_b::L11_TAIXU_DLC_EVENTS_B;
use super::universal::{TaixuBtnResult, TaixuButtonSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaixuBattleTarget { All, Kill, Repair, Subsist }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuSpoilSpec { pub random_thousandths: u32, pub effects: &'static [TaixuEffectSpec] }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuNpcSpec {
    pub general_name: &'static str, pub level: u8, pub is_boss: bool,
    pub seat: i8, pub hp: i8, pub max_hp_bonus: i32, pub min_hs: u32,
    pub correct_scaling: bool, pub equips: &'static [&'static str], pub spoils: &'static [TaixuSpoilSpec],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuChooseBtnSpec {
    pub name: &'static str, pub result: TaixuBtnResult, pub effects: &'static [TaixuEffectSpec],
    pub enemies: &'static [TaixuNpcSpec], pub friends: &'static [TaixuNpcSpec],
    pub sub_buttons: &'static [TaixuButtonSpec], pub spoils: &'static [TaixuSpoilSpec],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuDlcEventSpec {
    pub id: &'static str, pub zh_name: &'static str, pub kind: &'static str,
    pub target: TaixuBattleTarget, pub unique: bool, pub max_rounds: u32, pub exp: u32,
    pub rank: u8, pub level: u8, pub change: bool, pub forced: bool,
    pub random_thousandths: u32, pub premise: &'static str,
    pub enemies: &'static [TaixuNpcSpec], pub friends: &'static [TaixuNpcSpec],
    pub buttons: &'static [TaixuChooseBtnSpec],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuChapterSpec {
    pub id: &'static str, pub zh_name: &'static str, pub info: &'static str,
    pub premise: &'static [&'static str], pub procedure: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuSeasonSpec {
    pub id: &'static str, pub zh_name: &'static str, pub info: &'static str,
    pub season_order: u8, pub chapters: &'static [TaixuChapterSpec],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuChapterEventRef {
    pub season: &'static str, pub chapter: &'static str, pub id: &'static str,
    pub kind: &'static str, pub def_idx: i32, pub exam_id: &'static str,
}

pub fn get_dlc_event_by_idx(idx: usize) -> Option<&'static TaixuDlcEventSpec> {
    let a = L11_TAIXU_DLC_EVENTS_A.len();
    if idx < a { Some(&L11_TAIXU_DLC_EVENTS_A[idx]) }
    else { L11_TAIXU_DLC_EVENTS_B.get(idx - a) }
}

pub const L11_TAIXU_SEASONS: [TaixuSeasonSpec; 8] = [
    TaixuSeasonSpec { id: "ChongYingChuLin", zh_name: "冲应初临", info: "承师前志寻天秘，与君共行遨太虚", season_order: 1, chapters: &[TaixuChapterSpec { id: "0", zh_name: "序章", info: "此为何方", premise: &[], procedure: &["gain", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第一章", info: "初涉幻境", premise: &[], procedure: &["side", "random", "side", "random", "side", "store", "random", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第二章", info: "幻境历险", premise: &[], procedure: &["side", "random", "side", "random", "side", "store", "choose", "side", "random", "side", "random", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第三章", info: "真传之秘", premise: &[], procedure: &["gain", "side", "choose", "random", "side", "random", "store", "side", "random", "side", "random", "side", "Ultimate"] }] },
    TaixuSeasonSpec { id: "GFHuangJinZhiLuan", zh_name: "黄巾之乱", info: "前朝黄天须臾尽，今日太虚卷土归", season_order: 5, chapters: &[TaixuChapterSpec { id: "0", zh_name: "第一章", info: "投奔汉军", premise: &[], procedure: &["main", "main", "gain", "exam4", "store", "main", "main", "random", "main", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第二章①", info: "涿郡之战", premise: &[], procedure: &["main", "random", "store", "main", "exam4", "main", "random", "main", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第二章②", info: "长社之战", premise: &[], procedure: &["main", "random", "store", "main", "main", "random", "main", "main", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第三章①", info: "反攻宛城", premise: &[], procedure: &["random", "main", "store", "main", "gain", "random", "main", "main", "boss"] }, TaixuChapterSpec { id: "4", zh_name: "第三章②", info: "兵临广宗", premise: &[], procedure: &["main", "store", "random", "main", "main", "exam4", "main", "random", "boss"] }, TaixuChapterSpec { id: "5", zh_name: "第三章③", info: "围困广宗", premise: &[], procedure: &["main", "main", "random", "store", "main", "main", "exam3", "boss"] }, TaixuChapterSpec { id: "6", zh_name: "第三章④", info: "进军广宗", premise: &[], procedure: &["main", "random", "main", "store", "main", "exam4", "boss"] }, TaixuChapterSpec { id: "7", zh_name: "第四章①", info: "进军曲阳", premise: &[], procedure: &["main", "random", "main", "store", "main", "random", "boss"] }, TaixuChapterSpec { id: "8", zh_name: "第四章②", info: "围攻曲阳", premise: &[], procedure: &["main", "random", "store", "main", "random", "main", "main", "boss"] }, TaixuChapterSpec { id: "9", zh_name: "第四章③", info: "镇压黄巾", premise: &[], procedure: &["main", "random", "main", "store", "main", "random", "main", "Ultimate"] }] },
    TaixuSeasonSpec { id: "GFQiQinMengHuo", zh_name: "七擒孟获", info: "羽扇纶巾拥碧幢，七擒妙策制蛮王", season_order: 7, chapters: &[TaixuChapterSpec { id: "0", zh_name: "第一章①", info: "初至益州", premise: &[], procedure: &["main", "main", "exam4", "main", "store", "random", "main", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第一章②", info: "初至益州", premise: &[], procedure: &["main", "main", "exam4", "main", "store", "main", "random", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第一章终", info: "初至益州", premise: &[], procedure: &["main", "exam4", "main", "exam4", "main", "store", "main", "random", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第二章①", info: "四擒蛮王", premise: &[], procedure: &["main", "main", "random", "main", "store", "main", "exam4", "boss"] }, TaixuChapterSpec { id: "4", zh_name: "第二章②", info: "将计就计", premise: &[], procedure: &["main", "main", "exam4", "store", "main", "random", "main", "boss"] }, TaixuChapterSpec { id: "5", zh_name: "第二章③", info: "计破蛮军", premise: &[], procedure: &["main", "main", "exam4", "store", "main", "random", "main", "boss"] }, TaixuChapterSpec { id: "6", zh_name: "第二章终", info: "计破蛮军", premise: &[], procedure: &["main", "main", "exam4", "store", "main", "random", "main", "boss"] }, TaixuChapterSpec { id: "7", zh_name: "第三章上", info: "诱敌之策", premise: &[], procedure: &["main", "main", "exam4", "store", "main", "random", "main", "exam4", "boss"] }, TaixuChapterSpec { id: "8", zh_name: "第三章下", info: "假兽退敌", premise: &[], procedure: &["main", "main", "exam4", "store", "main", "random", "main", "exam4", "boss"] }, TaixuChapterSpec { id: "9", zh_name: "第四章上", info: "平定南蛮", premise: &[], procedure: &["main", "main", "exam4", "store", "main", "random", "boss"] }, TaixuChapterSpec { id: "10", zh_name: "第四章下", info: "平定南蛮", premise: &[], procedure: &["main", "main", "exam4", "store", "random", "main", "exam4", "Ultimate"] }] },
    TaixuSeasonSpec { id: "HaiWaiFenghe", zh_name: "海外缝合", info: "银枪铁骑震塞外，金羁白马霸群雄", season_order: 4, chapters: &[TaixuChapterSpec { id: "0", zh_name: "序章", info: "此为何方", premise: &[], procedure: &["random", "main", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第一章", info: "初涉幻境", premise: &[], procedure: &["main", "random", "side", "random", "main", "store", "micheng", "random", "side", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第二章", info: "幻境历险", premise: &[], procedure: &["main", "random", "side", "random", "main", "store", "main", "micheng", "random", "side", "main", "random", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第三章", info: "真传之秘", premise: &[], procedure: &["main", "side", "gain", "random", "micheng", "main", "random", "store", "main", "main", "side", "random", "boss"] }, TaixuChapterSpec { id: "4", zh_name: "第四章", info: "白马之锋", premise: &[], procedure: &["main", "side", "random", "store", "micheng", "random", "main", "gain", "main", "store", "trade", "Ultimate"] }] },
    TaixuSeasonSpec { id: "HeJinZhuHuan", zh_name: "何进诛宦", info: "权戚兴兵诛邪宦，未料惊变天下喑", season_order: 3, chapters: &[TaixuChapterSpec { id: "0", zh_name: "序章", info: "此为何方", premise: &[], procedure: &["random", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第一章", info: "初涉幻境", premise: &[], procedure: &["main", "random", "side", "random", "side", "store", "random", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第二章", info: "幻境历险", premise: &[], procedure: &["main", "random", "side", "random", "main", "store", "choose", "side", "random", "side", "random", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第三章", info: "真传之秘", premise: &[], procedure: &["main", "side", "gain", "random", "main", "random", "store", "choose", "main", "side", "random", "boss"] }, TaixuChapterSpec { id: "4", zh_name: "第四章", info: "何进诛宦", premise: &[], procedure: &["main", "side", "random", "store", "random", "main", "gain", "main", "Ultimate"] }] },
    TaixuSeasonSpec { id: "HuangTianZhiNu", zh_name: "黄天之怒", info: "前朝黄天须臾尽，今日太虚卷土归", season_order: 2, chapters: &[TaixuChapterSpec { id: "0", zh_name: "序章", info: "此为何方", premise: &[], procedure: &["random", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第一章", info: "初涉幻境", premise: &[], procedure: &["main", "random", "side", "random", "side", "store", "random", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第二章", info: "幻境历险", premise: &[], procedure: &["main", "random", "side", "random", "main", "store", "side", "random", "side", "random", "side", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第三章", info: "真传之秘", premise: &[], procedure: &["side", "main", "random", "side", "gain", "random", "store", "choose", "side", "random", "main", "random", "boss"] }, TaixuChapterSpec { id: "4", zh_name: "第四章", info: "黄天之怒", premise: &[], procedure: &["side", "choose", "random", "store", "side", "gain", "main", "random", "Ultimate"] }] },
    TaixuSeasonSpec { id: "LiGuoZhiLuan", zh_name: "李郭之乱", info: "强者四散蠃者相食，二三年间关中无人复", season_order: 6, chapters: &[TaixuChapterSpec { id: "0", zh_name: "序章", info: "此为何方", premise: &[], procedure: &["main", "random", "store", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第一章", info: "初涉幻境", premise: &[], procedure: &["main", "random", "side", "random", "main", "store", "random", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第二章", info: "幻境历险", premise: &[], procedure: &["main", "random", "main", "random", "main", "store", "side", "random", "main", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第三章", info: "真传之秘", premise: &[], procedure: &["main", "side", "random", "main", "random", "store", "main", "main", "random", "boss"] }, TaixuChapterSpec { id: "4", zh_name: "第四章", info: "李郭之乱", premise: &[], procedure: &["main", "side", "random", "store", "main", "random", "micheng", "main", "Ultimate"] }] },
    TaixuSeasonSpec { id: "MGBaWangZhengCheng", zh_name: "霸王征程", info: "举众争衡寿不永，却得孙吴百年身", season_order: 8, chapters: &[TaixuChapterSpec { id: "0", zh_name: "序章", info: "此为何方", premise: &[], procedure: &["random", "boss"] }, TaixuChapterSpec { id: "1", zh_name: "第一章", info: "初涉幻境", premise: &[], procedure: &["main", "random", "side", "random", "micheng", "side", "store", "random", "boss"] }, TaixuChapterSpec { id: "2", zh_name: "第二章", info: "幻境历险", premise: &[], procedure: &["main", "random", "side", "random", "main", "store", "side", "random", "side", "boss"] }, TaixuChapterSpec { id: "3", zh_name: "第三章", info: "真传之秘", premise: &[], procedure: &["main", "random", "side", "micheng", "random", "store", "choose", "side", "random", "main", "random", "boss"] }, TaixuChapterSpec { id: "4", zh_name: "第四章", info: "霸王征程", premise: &[], procedure: &["main", "random", "random", "store", "side", "gain", "main", "random", "Ultimate"] }] },
];

pub use super::chapter_refs_a::L11_TAIXU_CHAPTER_EVENT_REFS_A;
pub use super::chapter_refs_b::L11_TAIXU_CHAPTER_EVENT_REFS_B;
