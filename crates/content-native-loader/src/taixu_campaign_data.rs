//! Authentic upstream `L11` (`archive/upstream/noname-liuli/noname/extension/太虚幻境/{dlc/*/extension_season.js,event/event_universal.js,extension_servant.js,extension_framework.js}`)
//! 206 campaign/season/quiz/event/servant/mode nodes and deterministic execution state machine.

use crate::taixu::{TaixuConfig, TaixuError, TaixuRunState};

/// Category of a `太虚幻境` campaign or mode node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaixuNodeKind {
    SeasonChapter,
    ExamQuiz,
    BattleNode,
    UniversalTrade,
    UniversalGain,
    UniversalShop,
    ServantRegistry,
    FrameworkMode,
}

/// Specification of one upstream `L11` campaign/season/event/servant/mode entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuCampaignNodeSpec {
    pub scope: &'static str,
    pub id: &'static str,
    pub zh_name: &'static str,
    pub kind: TaixuNodeKind,
    pub gold_delta: i32,
    pub max_hp_delta: i8,
    pub hp_delta: i8,
    pub step_count: u8,
}

/// Deterministic outcome of executing a `太虚幻境` campaign node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaixuNodeOutcome {
    pub scope: String,
    pub node_id: String,
    pub kind: TaixuNodeKind,
    pub gold_after: u32,
    pub max_hp_delta: i8,
    pub hp_delta: i8,
    pub quiz_passed: bool,
    pub granted_buff: Option<String>,
}

/// All 21 authentic `L11` (`extension_servant.js:350-1760`) servants: `(id, zh_name, grade, skills)`.
pub const L11_TAIXU_SERVANTS: [(&str, &str, u8, &[&str]); 21] = [
    ("dundun", "盾盾", 1, &["txhj_dundunSkill1", "txhj_dundunSkill2"]),
    ("jiuwei", "九尾", 3, &["txhj_jiuweiSkill1", "txhj_jiuweiSkill2", "txhj_jiuweiSkill3"]),
    ("tengshe", "腾蛇", 3, &["txhj_tengsheSkill1", "txhj_tengsheSkill2", "txhj_tengsheSkill3"]),
    ("qilin", "麒麟", 3, &["txhj_qilinSkill1", "txhj_qilinSkill2", "txhj_qilinSkill3"]),
    ("diting", "谛听", 3, &["txhj_ditingSkill1", "txhj_ditingSkill2", "txhj_ditingSkill3"]),
    ("yaya", "鸭鸭", 2, &["txhj_yayaSkill1", "txhj_yayaSkill2"]),
    ("youyou", "佑佑", 2, &["txhj_youyouSkill1", "txhj_youyouSkill2"]),
    ("minmin", "敏敏", 2, &["txhj_minminSkill1", "txhj_minminSkill2"]),
    ("qiaoqiao", "巧巧", 2, &["txhj_qiaoqiaoSkill1", "txhj_qiaoqiaoSkill2"]),
    ("lulu", "露露", 1, &["txhj_luluSkill1", "txhj_luluSkill2"]),
    ("datong", "大桶", 0, &["txhj_datongSkill1"]),
    ("ale", "阿乐", 2, &["txhj_aleSkill1", "txhj_aleSkill2"]),
    ("rui", "瑞", 3, &["txhj_ruiSkill1", "txhj_ruiSkill2", "txhj_ruiSkill3"]),
    ("yueer", "玥儿", 2, &["txhj_yueerSkill1", "txhj_yueerSkill2"]),
    ("yan", "焱", 3, &["txhj_yanSkill1", "txhj_yanSkill2", "txhj_yanSkill3"]),
    ("ahao", "阿豪", 3, &["txhj_aHaoSkill1", "txhj_aHaoSkill2", "txhj_aHaoSkill3"]),
    ("liuli", "琉璃", 2, &["txhj_liuliSkill1", "txhj_liuliSkill2"]),
    ("manman", "蛮蛮", 2, &["txhj_manmanSkill1", "txhj_manmanSkill2"]),
    ("xiaoxiao", "枭枭", 2, &["txhj_xiaoxiaoSkill1", "txhj_xiaoxiaoSkill2"]),
    ("xueren", "雪人", 1, &["txhj_xuerenSkill1"]),
    ("xuanwu", "玄武", 3, &["txhj_xuanwuSkill1", "txhj_xuanwuSkill2", "txhj_xuanwuSkill3"]),
];

/// All 206 authentic `L11` campaign chapters, exams, battles, universal events, servant registries, and mode entry.
pub const L11_TAIXU_CAMPAIGN_NODES: [TaixuCampaignNodeSpec; 206] = [
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // ChongYingChuLin/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // ChongYingChuLin/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // ChongYingChuLin/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // ChongYingChuLin/extension_season.js:23
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "0", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFHuangJinZhiLuan/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "1", zh_name: "第二章①", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFHuangJinZhiLuan/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "2", zh_name: "第二章②", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFHuangJinZhiLuan/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "3", zh_name: "第三章①", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFHuangJinZhiLuan/extension_season.js:23
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "4", zh_name: "第三章②", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFHuangJinZhiLuan/extension_season.js:28
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "5", zh_name: "全部五星", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFHuangJinZhiLuan/extension_season.js:33
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4001", zh_name: "exam4001", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:414
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4002", zh_name: "exam4002", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:463
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4003", zh_name: "exam4003", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:512
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4005", zh_name: "exam4005", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:604
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4006", zh_name: "exam4006", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:653
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4007", zh_name: "exam4007", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:702
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4008", zh_name: "exam4008", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:751
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4009", zh_name: "exam4009", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:800
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4010", zh_name: "exam4010", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:849
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4011", zh_name: "exam4011", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:1512
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4012", zh_name: "exam4012", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:1561
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4013", zh_name: "exam4013", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:1610
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4014", zh_name: "exam4014", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:1659
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4015", zh_name: "exam4015", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:1708
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4016", zh_name: "exam4016", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:1757
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4017", zh_name: "exam4017", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:1806
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "txhj_fangongwancheng", zh_name: "txhj_fangongwancheng", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:4292
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4011", zh_name: "exam4011", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:6346
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4012", zh_name: "exam4012", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:6395
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4013", zh_name: "exam4013", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:6444
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4014", zh_name: "exam4014", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:6493
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4015", zh_name: "exam4015", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:6542
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4016", zh_name: "exam4016", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:6591
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4017", zh_name: "exam4017", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:6640
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4001", zh_name: "exam4001", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8552
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4002", zh_name: "exam4002", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8601
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4003", zh_name: "exam4003", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8650
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4005", zh_name: "exam4005", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8742
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4006", zh_name: "exam4006", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8791
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4007", zh_name: "exam4007", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8840
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4008", zh_name: "exam4008", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8889
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4009", zh_name: "exam4009", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8938
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "exam4010", zh_name: "exam4010", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:8987
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "txhj_xiaojishi", zh_name: "小集市", kind: TaixuNodeKind::UniversalShop, gold_delta: -100, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:9824
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", id: "txhj_souchajianxi", zh_name: "txhj_souchajianxi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFHuangJinZhiLuan/extension_season.js:10106
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "0", zh_name: "第一章①", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "1", zh_name: "第一章②", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "2", zh_name: "第一章终", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "5", zh_name: "第二章③", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:33
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "6", zh_name: "第二章终", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:38
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "7", zh_name: "第三章上", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:43
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "8", zh_name: "第三章下", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:48
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "9", zh_name: "第四章上", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:53
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "10", zh_name: "第四章下", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:58
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4018", zh_name: "exam4018", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:401
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4019", zh_name: "exam4019", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:447
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4022", zh_name: "exam4022", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:585
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4023", zh_name: "exam4023", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:1076
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4030", zh_name: "exam4030", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:2365
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4041", zh_name: "exam4041", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:6420
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4051", zh_name: "exam4051", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:8866
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_manzuqixi", zh_name: "蛮族奇袭", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:9313
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4056", zh_name: "exam4056", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:9501
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4057", zh_name: "exam4057", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:9547
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4058", zh_name: "exam4058", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:9593
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4062", zh_name: "exam4062", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:10375
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "9", zh_name: "火烧盘蛇", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:10698
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_huoshaopanshe", zh_name: "火烧盘蛇", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:10699
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_wunaiyingzhan", zh_name: "无奈应战", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:10774
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_hezhongtoudu", zh_name: "河中投毒", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:10833
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_tuoweijiekun", zh_name: "脱围解困", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:10892
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_yongweixianfeng", zh_name: "勇为先锋", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:10999
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4072", zh_name: "exam4072", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:11246
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "10", zh_name: "蛮王终服", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // GFQiQinMengHuo/extension_season.js:11360
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_manwangzhongfu", zh_name: "蛮王终服", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:11361
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_yumanjizhan", zh_name: "与蛮激战", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:11420
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_xiangbingjianxi", zh_name: "降兵奸细", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:11495
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_zhiyuanzhuli", zh_name: "支援主力", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:11586
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "txhj_fanghuoshaoshan", zh_name: "放火烧山", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:11693
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", id: "exam4080", zh_name: "exam4080", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // GFQiQinMengHuo/extension_season.js:12078
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HaiWaiFengHe/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HaiWaiFengHe/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HaiWaiFengHe/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HaiWaiFengHe/extension_season.js:23
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HaiWaiFengHe/extension_season.js:28
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_langqixunluo", zh_name: "txhj_langqixunluo", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:68
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_huanjingnantong", zh_name: "幻境男童", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:113
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_baimayicong", zh_name: "白马义从", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:159
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_langqituxi", zh_name: "txhj_langqituxi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:342
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_michenghuanjingsi", zh_name: "txhj_michenghuanjingsi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:631
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_michenghuanjingba", zh_name: "txhj_michenghuanjingba", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:703
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_baimaxiaoqi", zh_name: "白马骁骑", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:904
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_yingzhanzoudan", zh_name: "txhj_yingzhanzoudan", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:961
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_tonghu", zh_name: "铜虎", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:1033
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_jizhanzhangliao", zh_name: "txhj_jizhanzhangliao", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:4662
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_gongsuncongdi", zh_name: "txhj_gongsuncongdi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:4734
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_michenghuanjingshi", zh_name: "txhj_michenghuanjingshi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:4881
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_michenghuanjingyi", zh_name: "txhj_michenghuanjingyi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:4953
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_kuangshipima", zh_name: "txhj_kuangshipima", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:5010
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_yingzhansunguan", zh_name: "txhj_yingzhansunguan", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:5082
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_longqianghuqi", zh_name: "txhj_longqianghuqi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:6591
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_michenghuanjingshier", zh_name: "txhj_michenghuanjingshier", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:6858
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_xingyangdabai", zh_name: "txhj_xingyangdabai", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:6930
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_biantazhiyuan", zh_name: "txhj_biantazhiyuan", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:7020
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_gongsunjiazu", zh_name: "txhj_gongsunjiazu", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:7095
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_anxianghonglou", zh_name: "暗巷红楼", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:7954
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", id: "txhj_yinandejiaoyi", zh_name: "阴暗的交易", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // HaiWaiFengHe/extension_season.js:8011
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HeJinZhuHuan/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HeJinZhuHuan/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HeJinZhuHuan/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HeJinZhuHuan/extension_season.js:23
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HeJinZhuHuan/extension_season.js:28
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HuangTianZhiNu/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HuangTianZhiNu/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HuangTianZhiNu/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HuangTianZhiNu/extension_season.js:23
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // HuangTianZhiNu/extension_season.js:28
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // LiGuoZhiLuan/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // LiGuoZhiLuan/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // LiGuoZhiLuan/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // LiGuoZhiLuan/extension_season.js:23
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // LiGuoZhiLuan/extension_season.js:28
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // MGBaWangZhengCheng/extension_season.js:8
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // MGBaWangZhengCheng/extension_season.js:13
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // MGBaWangZhengCheng/extension_season.js:18
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // MGBaWangZhengCheng/extension_season.js:23
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 8 }, // MGBaWangZhengCheng/extension_season.js:28
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "txhj_michenghuanjingB", zh_name: "txhj_michenghuanjingB", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // MGBaWangZhengCheng/extension_season.js:277
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "txhj_michenghuanjingC", zh_name: "txhj_michenghuanjingC", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // MGBaWangZhengCheng/extension_season.js:349
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "txhj_jiangdongerzhang", zh_name: "txhj_jiangdongerzhang", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // MGBaWangZhengCheng/extension_season.js:3393
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "txhj_mingdaoanjian", zh_name: "明刀暗箭", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // MGBaWangZhengCheng/extension_season.js:5067
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "txhj_yingzhanhanjiang", zh_name: "txhj_yingzhanhanjiang", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // MGBaWangZhengCheng/extension_season.js:5961
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "txhj_zhuangbeidashi", zh_name: "txhj_zhuangbeidashi", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // MGBaWangZhengCheng/extension_season.js:6018
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", id: "txhj_jiangdongbawang", zh_name: "江东霸王", kind: TaixuNodeKind::BattleNode, gold_delta: 180, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // MGBaWangZhengCheng/extension_season.js:6134
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_yinandejiaoyi", zh_name: "阴暗的交易", kind: TaixuNodeKind::UniversalTrade, gold_delta: 400, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:4
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_linghundejiaoyi", zh_name: "灵魂的交易", kind: TaixuNodeKind::UniversalTrade, gold_delta: 500, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:30
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_luorushenyuan", zh_name: "落入深渊", kind: TaixuNodeKind::UniversalTrade, gold_delta: 300, max_hp_delta: -1, hp_delta: 0, step_count: 1 }, // event/event_universal.js:56
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_tianmingjiashen", zh_name: "天命加身", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:94
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_wannianxianguo", zh_name: "万年仙果", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 1, hp_delta: 1, step_count: 1 }, // event/event_universal.js:116
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_tianjiangshenbing", zh_name: "天匠神兵", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:144
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_fujiahaoli", zh_name: "富贾豪礼", kind: TaixuNodeKind::UniversalGain, gold_delta: 1000, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:166
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_puyuanchuanren", zh_name: "蒲元传人", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:188
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_fujiazhili", zh_name: "富贾之礼", kind: TaixuNodeKind::UniversalGain, gold_delta: 600, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:222
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_jiangxuntianming", zh_name: "将循天命", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:244
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_guyuanxianguo", zh_name: "固元鲜果", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 1, hp_delta: 0, step_count: 1 }, // event/event_universal.js:272
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_yiqiweiren", zh_name: "以气为刃", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:300
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_hezhongshafa", zh_name: "何种杀法？", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:322
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_gongfangzhize", zh_name: "攻防之择", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:350
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_diwozhize", zh_name: "敌我之择", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:378
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_qihaikuolan", zh_name: "气海扩澜", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:406
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_cuitilianhun", zh_name: "淬体炼魂", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 1, hp_delta: 0, step_count: 1 }, // event/event_universal.js:434
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_cuitijindan", zh_name: "淬体金丹", kind: TaixuNodeKind::UniversalGain, gold_delta: 300, max_hp_delta: 1, hp_delta: 0, step_count: 1 }, // event/event_universal.js:462
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_tiandijinghua", zh_name: "天地精华", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:490
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_huanjingzhibao", zh_name: "幻境之宝", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:512
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_zuocizhizhuan", zh_name: "左慈之传", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:534
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_linghunzhichi", zh_name: "灵魂之池", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:556
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam1", zh_name: "宿命的诘问", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:586
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam2", zh_name: "异商的问询", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:2532
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam3", zh_name: "军师的考验", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:4484
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4", zh_name: "exam4", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6436
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4001", zh_name: "exam4001", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6437
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4002", zh_name: "exam4002", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6486
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4003", zh_name: "exam4003", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6535
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4004", zh_name: "exam4004", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6581
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4005", zh_name: "exam4005", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6627
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4006", zh_name: "exam4006", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6676
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4007", zh_name: "exam4007", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6725
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4008", zh_name: "exam4008", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6774
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4009", zh_name: "exam4009", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6823
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4010", zh_name: "exam4010", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6872
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4011", zh_name: "exam4011", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6921
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4012", zh_name: "exam4012", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:6970
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4013", zh_name: "exam4013", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:7019
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4014", zh_name: "exam4014", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:7068
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4015", zh_name: "exam4015", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:7117
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4016", zh_name: "exam4016", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:7166
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4017", zh_name: "exam4017", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:7215
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4057", zh_name: "exam4057", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:9046
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4058", zh_name: "exam4058", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:9092
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4072", zh_name: "exam4072", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:9736
    TaixuCampaignNodeSpec { scope: "event_universal", id: "exam4080", zh_name: "exam4080", kind: TaixuNodeKind::ExamQuiz, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10104
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_sibingheishi", zh_name: "私兵黑市", kind: TaixuNodeKind::UniversalShop, gold_delta: -100, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10154
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_anxianghonglou", zh_name: "暗巷红楼", kind: TaixuNodeKind::UniversalShop, gold_delta: -100, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10176
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_xiaojishi", zh_name: "小集市", kind: TaixuNodeKind::UniversalShop, gold_delta: -100, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10198
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_dajishi", zh_name: "大集市", kind: TaixuNodeKind::UniversalShop, gold_delta: -100, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10220
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_huanjingjishi", zh_name: "幻境集市", kind: TaixuNodeKind::UniversalShop, gold_delta: -100, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10243
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_puyuanchuanren", zh_name: "蒲元传人", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10265
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_fujiazhili", zh_name: "富贾之礼", kind: TaixuNodeKind::UniversalGain, gold_delta: 600, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10298
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_jiangxuntianming", zh_name: "将循天命", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10319
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_guyuanxianguo", zh_name: "固元鲜果", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 1, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10346
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_yiqiweiren", zh_name: "以气为刃", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10373
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_hezhongshafa", zh_name: "何种杀法？", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10394
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_gongfangzhize", zh_name: "攻防之择", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10421
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_diwozhize", zh_name: "敌我之择", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10448
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_qihaikuolan", zh_name: "气海扩澜", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10475
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_cuitilianhun", zh_name: "淬体炼魂", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 1, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10502
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_cuitijindan", zh_name: "淬体金丹", kind: TaixuNodeKind::UniversalGain, gold_delta: 300, max_hp_delta: 1, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10529
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_tiandijinghua", zh_name: "天地精华", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10556
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_huanjingzhibao", zh_name: "幻境之宝", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10577
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_zuocizhizhuan", zh_name: "左慈之传", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10598
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_linghunzhichi", zh_name: "灵魂之池", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10619
    TaixuCampaignNodeSpec { scope: "event_universal", id: "txhj_tianmingjiashen", zh_name: "天命加身", kind: TaixuNodeKind::UniversalGain, gold_delta: 150, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // event/event_universal.js:10646
    TaixuCampaignNodeSpec { scope: "extension_framework", id: "taixuhuanjing", zh_name: "taixuhuanjing", kind: TaixuNodeKind::FrameworkMode, gold_delta: 100, max_hp_delta: 0, hp_delta: 0, step_count: 5 }, // 太虚幻境/extension_framework.js:9919
    TaixuCampaignNodeSpec { scope: "extension_servant", id: "servant", zh_name: "servant", kind: TaixuNodeKind::ServantRegistry, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // 太虚幻境/extension_servant.js:420
    TaixuCampaignNodeSpec { scope: "extension_servant", id: "action", zh_name: "action", kind: TaixuNodeKind::ServantRegistry, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // 太虚幻境/extension_servant.js:1329
    TaixuCampaignNodeSpec { scope: "extension_servant", id: "skillDesc", zh_name: "勇往直前", kind: TaixuNodeKind::ServantRegistry, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 }, // 太虚幻境/extension_servant.js:1507
];

/// Look up the bound upstream skill IDs for a `太虚幻境` servant (`extension_servant.js:1507-1760`).
pub fn resolve_taixu_servant_skills(servant_id: &str) -> Option<&'static [&'static str]> {
    L11_TAIXU_SERVANTS
        .iter()
        .find(|&&(id, _, _, _)| id == servant_id)
        .map(|&(_, _, _, sks)| sks)
}

/// Authentic upstream `buff_txhj_*` shop & event blessing pool (`太虚幻境/extension_buff.js:7-790`).
pub const L11_TAIXU_SHOP_BUFF_POOL: [&str; 16] = [
    "buff_txhj_aozhan",
    "buff_txhj_yanhuo",
    "buff_txhj_jinglei",
    "buff_txhj_xihuo",
    "buff_txhj_dinglei",
    "buff_txhj_yueyingdezhihui",
    "buff_txhj_changbingguangren",
    "buff_txhj_xiongziyingfa",
    "buff_txhj_pozhenzhifeng",
    "buff_txhj_yanrenzhinu",
    "buff_txhj_ziyideyongzhan",
    "buff_txhj_taipingzhoufa",
    "buff_txhj_xumingshenyao",
    "buff_txhj_tianxiawushuang",
    "buff_txhj_mowangdejianiang",
    "buff_txhj_luoshenjuezhang",
];

fn pick_unowned_upstream_buff(owned: &[String], seed_offset: usize) -> Option<String> {
    let n = L11_TAIXU_SHOP_BUFF_POOL.len();
    for step in 0..n {
        let candidate = L11_TAIXU_SHOP_BUFF_POOL[(seed_offset + step) % n];
        if !owned.iter().any(|b| b == candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Deterministically execute any of the 206 `太虚幻境` campaign/season/quiz/event/shop/servant/mode nodes
/// against [`TaixuRunState`] (`extension_framework.js:9919`, `event_universal.js:3-10730`, `dlc/*/extension_season.js`).
pub fn execute_taixu_campaign_node(
    cfg: &TaixuConfig,
    state: &mut TaixuRunState,
    scope: &str,
    node_id: &str,
    choice_idx: usize,
) -> Result<TaixuNodeOutcome, TaixuError> {
    if !cfg.enable_campaign {
        return Err(TaixuError::CampaignDisabled);
    }
    if state.completed {
        return Err(TaixuError::AlreadyCompleted);
    }
    let spec = L11_TAIXU_CAMPAIGN_NODES
        .iter()
        .find(|n| n.scope == scope && n.id == node_id)
        .or_else(|| L11_TAIXU_CAMPAIGN_NODES.iter().find(|n| n.id == node_id))
        .ok_or_else(|| TaixuError::UnknownCommand(format!("{scope}:{node_id}")))?;

    let mut quiz_passed = false;
    let mut granted_buff = None;
    match spec.kind {
        TaixuNodeKind::ExamQuiz => {
            // Upstream exam4*/exam3* nodes award full coin bonus on correct answer (option 0).
            quiz_passed = choice_idx == 0;
            let reward = if quiz_passed { spec.gold_delta.max(0) as u32 } else { 30 };
            state.gold = state.gold.saturating_add(reward);
            state.node_index += 1;
        }
        TaixuNodeKind::UniversalShop => {
            // Upstream `buff_txhj_juguzhidao` (`extension_buff.js:189`): shop cost halved.
            let has_jugu = state.buffs.iter().any(|b| b == "buff_txhj_juguzhidao");
            let base_cost = spec.gold_delta.unsigned_abs();
            let cost = if has_jugu { base_cost / 2 } else { base_cost };
            if state.gold >= cost && state.buffs.len() < cfg.buff_limit {
                if let Some(buff_id) =
                    pick_unowned_upstream_buff(&state.buffs, state.node_index + choice_idx)
                {
                    state.gold -= cost;
                    state.buffs.push(buff_id.clone());
                    granted_buff = Some(buff_id);
                }
            }
            state.node_index += 1;
        }
        TaixuNodeKind::ServantRegistry => {
            if !cfg.spirit_enabled {
                return Err(TaixuError::SpiritDisabled);
            }
            let (srv_id, _, _, _) = L11_TAIXU_SERVANTS[choice_idx % L11_TAIXU_SERVANTS.len()];
            if !state.spirits.iter().any(|s| s == srv_id) {
                state.spirits.push(srv_id.to_string());
            }
        }
        TaixuNodeKind::FrameworkMode => {
            state.season = "taixuhuanjing".to_string();
            state.gold = state.gold.saturating_add(spec.gold_delta.max(0) as u32);
        }
        TaixuNodeKind::SeasonChapter
        | TaixuNodeKind::BattleNode
        | TaixuNodeKind::UniversalTrade
        | TaixuNodeKind::UniversalGain => {
            if spec.gold_delta >= 0 {
                state.gold = state.gold.saturating_add(spec.gold_delta as u32);
            } else {
                let need = spec.gold_delta.unsigned_abs();
                state.gold = state.gold.saturating_sub(need);
            }
            if matches!(spec.kind, TaixuNodeKind::UniversalGain)
                && state.buffs.len() < cfg.buff_limit
            {
                let hash_seed = spec.id.bytes().fold(choice_idx, |acc, b| acc.wrapping_add(b as usize));
                if let Some(buff_id) = pick_unowned_upstream_buff(&state.buffs, hash_seed) {
                    state.buffs.push(buff_id.clone());
                    granted_buff = Some(buff_id);
                }
            }
            state.node_index += 1;
            if matches!(spec.kind, TaixuNodeKind::SeasonChapter) && spec.id == "9" {
                state.completed = true;
            }
        }
    }

    Ok(TaixuNodeOutcome {
        scope: spec.scope.to_string(),
        node_id: spec.id.to_string(),
        kind: spec.kind,
        gold_after: state.gold,
        max_hp_delta: spec.max_hp_delta,
        hp_delta: spec.hp_delta,
        quiz_passed,
        granted_buff,
    })
}

/// Season-scoped multi-node `太虚幻境` campaign graph progression (`dlc/*/extension_season.js`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaixuCampaignRun {
    pub scope: String,
    pub cursor: usize,
    pub visited_nodes: Vec<String>,
    pub state: TaixuRunState,
}

impl TaixuCampaignRun {
    /// Start a new season campaign run on `scope` (e.g. `"ChongYingChuLin"`, `"GFHuangJinZhiLuan"`, `"GFQiQinMengHuo"`, `"HaiWaiFengHe"`).
    pub fn start_season_run(cfg: &TaixuConfig, scope: &str) -> Result<Self, TaixuError> {
        if !cfg.enable_campaign {
            return Err(TaixuError::CampaignDisabled);
        }
        if !L11_TAIXU_CAMPAIGN_NODES.iter().any(|n| n.scope == scope) {
            return Err(TaixuError::UnknownCommand(format!("unknown_season:{scope}")));
        }
        let mut state = TaixuRunState::from_config(cfg);
        state.season = scope.to_string();
        Ok(Self {
            scope: scope.to_string(),
            cursor: 0,
            visited_nodes: Vec::new(),
            state,
        })
    }

    /// Return the next reachable nodes (up to 3 branching choices) in the active season graph.
    pub fn available_next_nodes(&self) -> Vec<&'static TaixuCampaignNodeSpec> {
        if self.state.completed {
            return Vec::new();
        }
        L11_TAIXU_CAMPAIGN_NODES
            .iter()
            .filter(|n| n.scope == self.scope || n.scope == "event_universal")
            .skip(self.cursor)
            .take(3)
            .collect()
    }

    /// Advance along branch `branch_idx` (`0..available_next_nodes().len()`) with option `choice_idx`.
    pub fn advance_node(
        &mut self,
        cfg: &TaixuConfig,
        branch_idx: usize,
        choice_idx: usize,
    ) -> Result<TaixuNodeOutcome, TaixuError> {
        let nexts = self.available_next_nodes();
        let chosen = nexts
            .get(branch_idx)
            .copied()
            .ok_or_else(|| TaixuError::UnknownCommand(format!("branch:{branch_idx}")))?;
        let out = execute_taixu_campaign_node(cfg, &mut self.state, chosen.scope, chosen.id, choice_idx)?;
        self.visited_nodes.push(format!("{}:{}", chosen.scope, chosen.id));
        self.cursor += branch_idx + 1;
        if self.available_next_nodes().is_empty() {
            self.state.completed = true;
        }
        Ok(out)
    }

    /// Deterministic snapshot serialization for campaign save/restore.
    pub fn encode_snapshot(&self) -> String {
        format!(
            "scope={}|cursor={}|visited={}|{}",
            self.scope,
            self.cursor,
            self.visited_nodes.join(","),
            crate::taixu::snapshot_taixu(&self.state),
        )
    }

    /// Restore a `TaixuCampaignRun` from [`Self::encode_snapshot`].
    pub fn decode_snapshot(raw: &str) -> Result<Self, TaixuError> {
        let mut parts = raw.splitn(4, '|');
        let scope = parts
            .next()
            .and_then(|s| s.strip_prefix("scope="))
            .filter(|s| !s.is_empty())
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing scope".into()))?
            .to_string();
        let cursor = parts
            .next()
            .and_then(|s| s.strip_prefix("cursor="))
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing cursor".into()))?
            .parse::<usize>()
            .map_err(|e| TaixuError::SnapshotMalformed(e.to_string()))?;
        let visited_raw = parts
            .next()
            .and_then(|s| s.strip_prefix("visited="))
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing visited".into()))?;
        let visited_nodes = if visited_raw.is_empty() {
            Vec::new()
        } else {
            visited_raw.split(',').map(str::to_string).collect()
        };
        let state_raw = parts
            .next()
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing state".into()))?;
        let state = crate::taixu::restore_taixu(state_raw)?;
        Ok(Self {
            scope,
            cursor,
            visited_nodes,
            state,
        })
    }
}
