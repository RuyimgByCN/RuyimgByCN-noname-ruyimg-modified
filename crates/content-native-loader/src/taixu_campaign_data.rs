//! L11（太虚幻境）Batch L11-6b 完整战役目录与 206 项 `L11:modes:*` 锚点桥接。
//!
//! 本模块保留与既有验收入口兼容的公有 API，并将全部节点、题库正误与奖励、
//! 通用奇遇、商店价格、侍灵等阶与章节流程对接到 `crate::taixu_mode::*` 真实上游定义。

use crate::taixu::{TaixuConfig, TaixuError, TaixuRunState};
use crate::taixu_mode::{
    find_exam_by_id, find_universal_event, taixu_majun_upgrade_name, TaixuBtnResult,
    TaixuEffectKind, L11_TAIXU_BUFF_STORE, L11_TAIXU_SEASONS, L11_TAIXU_SERVANT_SPECS,
};

/// `L11`（太虚幻境）上游模式节点大类。
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

/// 单个 `L11:modes:*` 上游节点条目规格（对应 `inventory_status.json` 全部 206 项）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaixuCampaignNodeSpec {
    pub scope: &'static str,
    pub section: &'static str,
    pub source_line: u32,
    pub id: &'static str,
    pub zh_name: &'static str,
    pub kind: TaixuNodeKind,
    pub correct_choice_idx: usize,
    pub option_count: usize,
    pub gold_delta: i32,
    pub max_hp_delta: i8,
    pub hp_delta: i8,
    pub step_count: u8,
}

/// 执行单个 `太虚幻境` 战役节点的确定性结果。
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

/// 全部 21 项真实 `L11` 侍灵档案：`(id, zh_name, grade, skills)`（等阶来自 `extension_servant.js:420`）。
pub const L11_TAIXU_SERVANTS: [(&str, &str, u8, &[&str]); 21] = [
    ("lulu", "鲁鲁", 2, &["txhj_luluSkill1", "txhj_luluSkill2"]),
    ("datong", "大桶", 0, &["txhj_datongSkill1"]),
    ("ale", "阿乐", 2, &["txhj_aleSkill1", "txhj_aleSkill2"]),
    ("rui", "瑞", 3, &["txhj_ruiSkill1", "txhj_ruiSkill2", "txhj_ruiSkill3"]),
    ("yueer", "月儿", 2, &["txhj_yueerSkill1", "txhj_yueerSkill2"]),
    ("yan", "焱", 3, &["txhj_yanSkill1", "txhj_yanSkill2", "txhj_yanSkill3"]),
    ("ahao", "阿豪", 3, &["txhj_aHaoSkill1", "txhj_aHaoSkill2", "txhj_aHaoSkill3"]),
    ("liuli", "琉璃", 2, &["txhj_liuliSkill1", "txhj_liuliSkill2"]),
    ("manman", "蛮蛮", 2, &["txhj_manmanSkill1", "txhj_manmanSkill2"]),
    ("xiaoxiao", "枭枭", 2, &["txhj_xiaoxiaoSkill1", "txhj_xiaoxiaoSkill2"]),
    ("xueren", "雪人", 0, &["txhj_xuerenSkill1"]),
    ("xuanwu", "玄武", 3, &["txhj_xuanwuSkill1", "txhj_xuanwuSkill2", "txhj_xuanwuSkill3"]),
    ("dundun", "盾盾", 2, &["txhj_dundunSkill1", "txhj_dundunSkill2"]),
    ("jiuwei", "九尾", 3, &["txhj_jiuweiSkill1", "txhj_jiuweiSkill2", "txhj_jiuweiSkill3"]),
    ("tengshe", "腾蛇", 3, &["txhj_tengsheSkill1", "txhj_tengsheSkill2", "txhj_tengsheSkill3"]),
    ("yaya", "鸭鸭", 2, &["txhj_yayaSkill1", "txhj_yayaSkill2"]),
    ("youyou", "悠悠", 2, &["txhj_youyouSkill1", "txhj_youyouSkill2"]),
    ("minmin", "敏敏", 2, &["txhj_minminSkill1", "txhj_minminSkill2"]),
    ("qiaoqiao", "乔乔", 2, &["txhj_qiaoqiaoSkill1", "txhj_qiaoqiaoSkill2"]),
    ("qilin", "麒麟", 3, &["txhj_qilinSkill1", "txhj_qilinSkill2", "txhj_qilinSkill3"]),
    ("diting", "谛听", 3, &["txhj_ditingSkill1", "txhj_ditingSkill2", "txhj_ditingSkill3"]),
];

/// 按侍灵 ID 解析其全部真实技能列表（同步校验 `L11_TAIXU_SERVANT_SPECS`）。
pub fn resolve_taixu_servant_skills(servant_id: &str) -> Option<&'static [&'static str]> {
    let _ = L11_TAIXU_SERVANT_SPECS.iter().find(|s| s.id == servant_id)?;
    L11_TAIXU_SERVANTS
        .iter()
        .find(|&&(id, _, _, _)| id == servant_id)
        .map(|&(_, _, _, sk)| sk)
}

/// 真实商店可售 `buff_txhj_*` 祝福池（全部源自 `extension_buff.js` `store: true`）。
pub const L11_TAIXU_SHOP_BUFF_POOL: [&str; 16] = [
    "buff_txhj_juejingzhice",
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

/// 全部 206 项 `L11:modes:*` 模式/赛季/事件/题库/侍灵节点清单。
pub const L11_TAIXU_CAMPAIGN_NODES: [TaixuCampaignNodeSpec; 206] = [
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 4, id: "txhj_yinandejiaoyi", zh_name: "阴暗的交易", kind: TaixuNodeKind::UniversalTrade, correct_choice_idx: 0, option_count: 2, gold_delta: 400, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 30, id: "txhj_linghundejiaoyi", zh_name: "灵魂的交易", kind: TaixuNodeKind::UniversalTrade, correct_choice_idx: 0, option_count: 2, gold_delta: 500, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 56, id: "txhj_luorushenyuan", zh_name: "落入深渊", kind: TaixuNodeKind::UniversalTrade, correct_choice_idx: 0, option_count: 4, gold_delta: 0, max_hp_delta: -1, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 94, id: "txhj_tianmingjiashen", zh_name: "天命加身", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 116, id: "txhj_wannianxianguo", zh_name: "万年仙果", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 1, hp_delta: 1, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 144, id: "txhj_tianjiangshenbing", zh_name: "天匠神兵", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 166, id: "txhj_fujiahaoli", zh_name: "富贾豪礼", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 1000, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 188, id: "txhj_puyuanchuanren", zh_name: "蒲元传人", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 3, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 222, id: "txhj_fujiazhili", zh_name: "富贾之礼", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 600, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 244, id: "txhj_jiangxuntianming", zh_name: "将循天命", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 272, id: "txhj_guyuanxianguo", zh_name: "固元鲜果", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 300, id: "txhj_yiqiweiren", zh_name: "以气为刃", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 322, id: "txhj_hezhongshafa", zh_name: "何种杀法？", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 350, id: "txhj_gongfangzhize", zh_name: "攻防之择", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 378, id: "txhj_diwozhize", zh_name: "敌我之择", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 406, id: "txhj_qihaikuolan", zh_name: "气海扩澜", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 434, id: "txhj_cuitilianhun", zh_name: "淬体炼魂", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 1, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 462, id: "txhj_cuitijindan", zh_name: "淬体金丹", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 1, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 490, id: "txhj_tiandijinghua", zh_name: "天地精华", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 512, id: "txhj_huanjingzhibao", zh_name: "幻境之宝", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 534, id: "txhj_zuocizhizhuan", zh_name: "左慈之传", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "gainPack", source_line: 556, id: "txhj_linghunzhichi", zh_name: "灵魂之池", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "examPack", source_line: 586, id: "exam1", zh_name: "exam1", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "examPack", source_line: 2532, id: "exam2", zh_name: "exam2", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "examPack", source_line: 4484, id: "exam3", zh_name: "exam3", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "examPack", source_line: 6436, id: "exam4", zh_name: "exam4", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6437, id: "exam4001", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6486, id: "exam4002", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6535, id: "exam4003", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6581, id: "exam4004", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6627, id: "exam4005", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6676, id: "exam4006", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6725, id: "exam4007", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6774, id: "exam4008", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6823, id: "exam4009", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6872, id: "exam4010", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6921, id: "exam4011", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 6970, id: "exam4012", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 7019, id: "exam4013", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 7068, id: "exam4014", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 7117, id: "exam4015", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 7166, id: "exam4016", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 7215, id: "exam4017", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 9046, id: "exam4057", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 9092, id: "exam4058", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 9736, id: "exam4072", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "exam4", source_line: 10104, id: "exam4080", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_store", source_line: 10154, id: "txhj_sibingheishi", zh_name: "私兵黑市", kind: TaixuNodeKind::UniversalShop, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_store", source_line: 10176, id: "txhj_anxianghonglou", zh_name: "暗巷红楼", kind: TaixuNodeKind::UniversalShop, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_store", source_line: 10198, id: "txhj_xiaojishi", zh_name: "小集市", kind: TaixuNodeKind::UniversalShop, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_store", source_line: 10220, id: "txhj_dajishi", zh_name: "大集市", kind: TaixuNodeKind::UniversalShop, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 4 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_store", source_line: 10243, id: "txhj_huanjingjishi", zh_name: "幻境集市", kind: TaixuNodeKind::UniversalShop, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 5 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10265, id: "txhj_puyuanchuanren", zh_name: "蒲元传人", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 3, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10298, id: "txhj_fujiazhili", zh_name: "富贾之礼", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 600, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10319, id: "txhj_jiangxuntianming", zh_name: "将循天命", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10346, id: "txhj_guyuanxianguo", zh_name: "固元鲜果", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10373, id: "txhj_yiqiweiren", zh_name: "以气为刃", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10394, id: "txhj_hezhongshafa", zh_name: "何种杀法？", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10421, id: "txhj_gongfangzhize", zh_name: "攻防之择", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10448, id: "txhj_diwozhize", zh_name: "敌我之择", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10475, id: "txhj_qihaikuolan", zh_name: "气海扩澜", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10502, id: "txhj_cuitilianhun", zh_name: "淬体炼魂", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 1, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10529, id: "txhj_cuitijindan", zh_name: "淬体金丹", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 1, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10556, id: "txhj_tiandijinghua", zh_name: "天地精华", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10577, id: "txhj_huanjingzhibao", zh_name: "幻境之宝", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10598, id: "txhj_zuocizhizhuan", zh_name: "左慈之传", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10619, id: "txhj_linghunzhichi", zh_name: "灵魂之池", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 2, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "event_universal", section: "changePack_gain", source_line: 10646, id: "txhj_tianmingjiashen", zh_name: "天命加身", kind: TaixuNodeKind::UniversalGain, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "season_chap_0", source_line: 8, id: "0", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 10 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第二章①", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第二章②", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "season_chap_3", source_line: 23, id: "3", zh_name: "第三章①", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "season_chap_4", source_line: 28, id: "4", zh_name: "第三章②", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "season_chap_5", source_line: 33, id: "5", zh_name: "第三章③", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L414", source_line: 414, id: "exam4001", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L463", source_line: 463, id: "exam4002", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L512", source_line: 512, id: "exam4003", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L604", source_line: 604, id: "exam4005", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L653", source_line: 653, id: "exam4006", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L702", source_line: 702, id: "exam4007", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L751", source_line: 751, id: "exam4008", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L800", source_line: 800, id: "exam4009", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L849", source_line: 849, id: "exam4010", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L1512", source_line: 1512, id: "exam4011", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L1561", source_line: 1561, id: "exam4012", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L1610", source_line: 1610, id: "exam4013", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L1659", source_line: 1659, id: "exam4014", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L1708", source_line: 1708, id: "exam4015", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L1757", source_line: 1757, id: "exam4016", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L1806", source_line: 1806, id: "exam4017", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_ev_L4292", source_line: 4292, id: "txhj_fangongwancheng", zh_name: "反攻宛城", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 250, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L6346", source_line: 6346, id: "exam4011", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L6395", source_line: 6395, id: "exam4012", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L6444", source_line: 6444, id: "exam4013", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L6493", source_line: 6493, id: "exam4014", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L6542", source_line: 6542, id: "exam4015", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L6591", source_line: 6591, id: "exam4016", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L6640", source_line: 6640, id: "exam4017", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8552", source_line: 8552, id: "exam4001", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8601", source_line: 8601, id: "exam4002", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8650", source_line: 8650, id: "exam4003", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8742", source_line: 8742, id: "exam4005", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8791", source_line: 8791, id: "exam4006", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8840", source_line: 8840, id: "exam4007", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8889", source_line: 8889, id: "exam4008", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8938", source_line: 8938, id: "exam4009", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_exam_L8987", source_line: 8987, id: "exam4010", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_ev_L9824", source_line: 9824, id: "txhj_xiaojishi", zh_name: "小集市", kind: TaixuNodeKind::UniversalShop, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "GFHuangJinZhiLuan", section: "dlc_ev_L10106", source_line: 10106, id: "txhj_souchajianxi", zh_name: "搜查奸细", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 200, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_0", source_line: 8, id: "0", zh_name: "第一章①", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第一章②", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第一章终", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_5", source_line: 33, id: "5", zh_name: "第二章③", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_6", source_line: 38, id: "6", zh_name: "第二章终", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_7", source_line: 43, id: "7", zh_name: "第三章上", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_8", source_line: 48, id: "8", zh_name: "第三章下", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_9", source_line: 53, id: "9", zh_name: "第四章上", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 7 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "season_chap_10", source_line: 58, id: "10", zh_name: "第四章下", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L401", source_line: 401, id: "exam4018", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L447", source_line: 447, id: "exam4019", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L585", source_line: 585, id: "exam4022", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L1076", source_line: 1076, id: "exam4023", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L2365", source_line: 2365, id: "exam4030", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L6420", source_line: 6420, id: "exam4041", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L8866", source_line: 8866, id: "exam4051", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L9313", source_line: 9313, id: "txhj_manzuqixi", zh_name: "蛮族奇袭", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 290, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L9501", source_line: 9501, id: "exam4056", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L9547", source_line: 9547, id: "exam4057", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L9593", source_line: 9593, id: "exam4058", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L10375", source_line: 10375, id: "exam4062", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "event_chap_9", source_line: 10698, id: "9", zh_name: "GFQiQinMengHuo_events_9", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 11 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L10699", source_line: 10699, id: "txhj_huoshaopanshe", zh_name: "火烧盘蛇", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 320, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L10774", source_line: 10774, id: "txhj_wunaiyingzhan", zh_name: "无奈应战", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 270, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L10833", source_line: 10833, id: "txhj_hezhongtoudu", zh_name: "河中投毒", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 270, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L10892", source_line: 10892, id: "txhj_tuoweijiekun", zh_name: "脱围解困", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 320, max_hp_delta: 0, hp_delta: 0, step_count: 5 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L10999", source_line: 10999, id: "txhj_yongweixianfeng", zh_name: "勇为先锋", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 290, max_hp_delta: 0, hp_delta: 0, step_count: 5 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L11246", source_line: 11246, id: "exam4072", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "event_chap_10", source_line: 11360, id: "10", zh_name: "GFQiQinMengHuo_events_10", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 14 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L11361", source_line: 11361, id: "txhj_manwangzhongfu", zh_name: "蛮王终服", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 350, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L11420", source_line: 11420, id: "txhj_yumanjizhan", zh_name: "与蛮激战", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 280, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L11495", source_line: 11495, id: "txhj_xiangbingjianxi", zh_name: "降兵奸细", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 290, max_hp_delta: 0, hp_delta: 0, step_count: 4 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L11586", source_line: 11586, id: "txhj_zhiyuanzhuli", zh_name: "支援主力", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 290, max_hp_delta: 0, hp_delta: 0, step_count: 5 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_ev_L11693", source_line: 11693, id: "txhj_fanghuoshaoshan", zh_name: "放火烧山", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 290, max_hp_delta: 0, hp_delta: 0, step_count: 5 },
    TaixuCampaignNodeSpec { scope: "GFQiQinMengHuo", section: "dlc_exam_L12078", source_line: 12078, id: "exam4080", zh_name: "仙人考验", kind: TaixuNodeKind::ExamQuiz, correct_choice_idx: 3, option_count: 4, gold_delta: 220, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "season_chap_0", source_line: 8, id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 10 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 13 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "season_chap_3", source_line: 23, id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 13 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "season_chap_4", source_line: 28, id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 12 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L68", source_line: 68, id: "txhj_langqixunluo", zh_name: "狼骑突袭", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L113", source_line: 113, id: "txhj_huanjingnantong", zh_name: "幻境守护男童", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L159", source_line: 159, id: "txhj_baimayicong", zh_name: "白马义从", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 280, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L342", source_line: 342, id: "txhj_langqituxi", zh_name: "狼骑突袭", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L631", source_line: 631, id: "txhj_michenghuanjingsi", zh_name: "迷城幻境", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L703", source_line: 703, id: "txhj_michenghuanjingba", zh_name: "迷城幻境", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 280, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L904", source_line: 904, id: "txhj_baimaxiaoqi", zh_name: "白马骁骑", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 260, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L961", source_line: 961, id: "txhj_yingzhanzoudan", zh_name: "迎战邹丹", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 320, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L1033", source_line: 1033, id: "txhj_tonghu", zh_name: "守境铜虎", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L4662", source_line: 4662, id: "txhj_jizhanzhangliao", zh_name: "激战张辽", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 350, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L4734", source_line: 4734, id: "txhj_gongsuncongdi", zh_name: "公孙从弟", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 330, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L4881", source_line: 4881, id: "txhj_michenghuanjingshi", zh_name: "迷城幻境", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L4953", source_line: 4953, id: "txhj_michenghuanjingyi", zh_name: "迷城幻境", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L5010", source_line: 5010, id: "txhj_kuangshipima", zh_name: "狂仕批骂", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L5082", source_line: 5082, id: "txhj_yingzhansunguan", zh_name: "迎战孙观", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 310, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L6591", source_line: 6591, id: "txhj_longqianghuqi", zh_name: "龙枪虎骑", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 400, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L6858", source_line: 6858, id: "txhj_michenghuanjingshier", zh_name: "迷城幻境", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 280, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L6930", source_line: 6930, id: "txhj_xingyangdabai", zh_name: "荥阳大败", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 400, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L7020", source_line: 7020, id: "txhj_biantazhiyuan", zh_name: "斩杀曹豹", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 260, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L7095", source_line: 7095, id: "txhj_gongsunjiazu", zh_name: "公孙家族", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L7954", source_line: 7954, id: "txhj_anxianghonglou", zh_name: "暗巷红楼", kind: TaixuNodeKind::UniversalShop, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HaiWaiFengHe", section: "dlc_ev_L8011", source_line: 8011, id: "txhj_yinandejiaoyi", zh_name: "阴暗的交易", kind: TaixuNodeKind::UniversalTrade, correct_choice_idx: 0, option_count: 2, gold_delta: 400, max_hp_delta: 0, hp_delta: 0, step_count: 1 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "season_chap_0", source_line: 8, id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 10 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "season_chap_3", source_line: 23, id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 12 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "season_chap_4", source_line: 28, id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "dlc_ev_L277", source_line: 277, id: "txhj_michenghuanjingB", zh_name: "迷城幻境", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "dlc_ev_L349", source_line: 349, id: "txhj_michenghuanjingC", zh_name: "迷城幻境", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 3 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "dlc_ev_L3393", source_line: 3393, id: "txhj_jiangdongerzhang", zh_name: "江东二张", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 300, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "dlc_ev_L5067", source_line: 5067, id: "txhj_mingdaoanjian", zh_name: "明刀暗箭", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 290, max_hp_delta: 0, hp_delta: 0, step_count: 4 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "dlc_ev_L5961", source_line: 5961, id: "txhj_yingzhanhanjiang", zh_name: "迎战镖骑", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 350, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "dlc_ev_L6018", source_line: 6018, id: "txhj_zhuangbeidashi", zh_name: "装备大师", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 330, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "MGBaWangZhengCheng", section: "dlc_ev_L6134", source_line: 6134, id: "txhj_jiangdongbawang", zh_name: "江东霸王", kind: TaixuNodeKind::BattleNode, correct_choice_idx: 0, option_count: 1, gold_delta: 350, max_hp_delta: 0, hp_delta: 0, step_count: 5 },
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", section: "season_chap_0", source_line: 8, id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 12 },
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", section: "season_chap_3", source_line: 23, id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 12 },
    TaixuCampaignNodeSpec { scope: "HeJinZhuHuan", section: "season_chap_4", source_line: 28, id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", section: "season_chap_0", source_line: 8, id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 12 },
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", section: "season_chap_3", source_line: 23, id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 13 },
    TaixuCampaignNodeSpec { scope: "HuangTianZhiNu", section: "season_chap_4", source_line: 28, id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", section: "season_chap_0", source_line: 8, id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 4 },
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 10 },
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", section: "season_chap_3", source_line: 23, id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 10 },
    TaixuCampaignNodeSpec { scope: "LiGuoZhiLuan", section: "season_chap_4", source_line: 28, id: "4", zh_name: "第四章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 9 },
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", section: "season_chap_0", source_line: 8, id: "0", zh_name: "序章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 2 },
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", section: "season_chap_1", source_line: 13, id: "1", zh_name: "第一章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", section: "season_chap_2", source_line: 18, id: "2", zh_name: "第二章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 12 },
    TaixuCampaignNodeSpec { scope: "ChongYingChuLin", section: "season_chap_3", source_line: 23, id: "3", zh_name: "第三章", kind: TaixuNodeKind::SeasonChapter, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 13 },
    TaixuCampaignNodeSpec { scope: "extension_servant", section: "servant", source_line: 420, id: "servant", zh_name: "servant", kind: TaixuNodeKind::ServantRegistry, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 21 },
    TaixuCampaignNodeSpec { scope: "extension_servant", section: "action", source_line: 1329, id: "action", zh_name: "action", kind: TaixuNodeKind::ServantRegistry, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 21 },
    TaixuCampaignNodeSpec { scope: "extension_servant", section: "skillDesc", source_line: 1507, id: "skillDesc", zh_name: "skillDesc", kind: TaixuNodeKind::ServantRegistry, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 21 },
    TaixuCampaignNodeSpec { scope: "extension_framework", section: "mode", source_line: 9919, id: "taixuhuanjing", zh_name: "太虚幻境", kind: TaixuNodeKind::FrameworkMode, correct_choice_idx: 0, option_count: 1, gold_delta: 0, max_hp_delta: 0, hp_delta: 0, step_count: 8 },
];

fn select_shop_buff_for_node(
    state: &TaixuRunState,
    seed_offset: usize,
) -> Option<(&'static str, u32)> {
    let n = L11_TAIXU_SHOP_BUFF_POOL.len();
    for step in 0..n {
        let candidate = L11_TAIXU_SHOP_BUFF_POOL[(seed_offset + step) % n];
        if !state.buffs.iter().any(|b| b == candidate) {
            if let Some(entry) = L11_TAIXU_BUFF_STORE
                .iter()
                .find(|b| b.id == candidate && b.store)
            {
                return Some((candidate, entry.value));
            }
        }
    }
    None
}

fn find_campaign_node(
    scope: &str,
    section: Option<&str>,
    node_id: &str,
) -> Option<&'static TaixuCampaignNodeSpec> {
    if let Some(sec) = section {
        if let Some(n) = L11_TAIXU_CAMPAIGN_NODES
            .iter()
            .find(|n| n.scope == scope && n.section == sec && n.id == node_id)
        {
            return Some(n);
        }
    }
    L11_TAIXU_CAMPAIGN_NODES
        .iter()
        .find(|n| n.scope == scope && n.id == node_id)
        .or_else(|| L11_TAIXU_CAMPAIGN_NODES.iter().find(|n| n.id == node_id))
}

/// 按精确 `(scope, section, node_id)` 执行节点，消除同文件同名键冲突。
pub fn execute_taixu_campaign_node_at(
    cfg: &TaixuConfig,
    state: &mut TaixuRunState,
    scope: &str,
    section: &str,
    node_id: &str,
    choice_idx: usize,
) -> Result<TaixuNodeOutcome, TaixuError> {
    execute_campaign_spec(
        cfg,
        state,
        find_campaign_node(scope, Some(section), node_id).ok_or_else(|| {
            TaixuError::UnknownCommand(format!("unknown taixu mode node {scope}:{section}:{node_id}"))
        })?,
        choice_idx,
    )
}

/// 执行任意一项 `L11` 战役模式节点（使用真实上游 `examPack` / `gainPack` / `changePack` / `seasonPack` 规则）。
pub fn execute_taixu_campaign_node(
    cfg: &TaixuConfig,
    state: &mut TaixuRunState,
    scope: &str,
    node_id: &str,
    choice_idx: usize,
) -> Result<TaixuNodeOutcome, TaixuError> {
    let spec = find_campaign_node(scope, None, node_id).ok_or_else(|| {
        TaixuError::UnknownCommand(format!("unknown taixu mode node {scope}:{node_id}"))
    })?;
    execute_campaign_spec(cfg, state, spec, choice_idx)
}

fn execute_campaign_spec(
    cfg: &TaixuConfig,
    state: &mut TaixuRunState,
    spec: &TaixuCampaignNodeSpec,
    choice_idx: usize,
) -> Result<TaixuNodeOutcome, TaixuError> {
    if !cfg.enable_campaign {
        return Err(TaixuError::CampaignDisabled);
    }
    if state.completed {
        return Err(TaixuError::AlreadyCompleted);
    }

    let mut quiz_passed = false;
    let mut granted_buff = None;
    let mut max_hp_delta = spec.max_hp_delta;
    let mut hp_delta = spec.hp_delta;

    match spec.kind {
        TaixuNodeKind::ExamQuiz => {
            if let Some(q) = find_exam_by_id(spec.id) {
                let btn = &q.buttons[choice_idx % q.buttons.len()];
                quiz_passed = btn.result == TaixuBtnResult::True;
                for ef in btn.effects {
                    match ef.kind {
                        TaixuEffectKind::Coin => {
                            let next_gold = i64::from(state.gold) + i64::from(ef.number);
                            state.gold = next_gold.max(0) as u32;
                        }
                        TaixuEffectKind::MaxHp => {
                            max_hp_delta = ef.number.clamp(-127, 127) as i8;
                        }
                        _ => {}
                    }
                }
            } else {
                // `exam1..exam4` 包入口映射到对应题库首题
                quiz_passed = choice_idx == spec.correct_choice_idx;
                if quiz_passed && spec.gold_delta > 0 {
                    state.gold = state.gold.saturating_add(spec.gold_delta as u32);
                }
            }
        }
        TaixuNodeKind::UniversalShop => {
            let seed = spec.source_line as usize + state.node_index + choice_idx;
            if let Some((buff, base_cost)) = select_shop_buff_for_node(state, seed) {
                if state.buffs.len() >= cfg.buff_limit as usize {
                    return Err(TaixuError::BuffLimitReached {
                        limit: cfg.buff_limit,
                    });
                }
                if state.gold < base_cost {
                    return Err(TaixuError::InsufficientGold {
                        have: state.gold,
                        need: base_cost,
                    });
                }
                state.gold -= base_cost;
                state.buffs.push(buff.to_string());
                granted_buff = Some(buff.to_string());
            }
        }
        TaixuNodeKind::UniversalTrade | TaixuNodeKind::UniversalGain => {
            if let Some(u) = find_universal_event(spec.id) {
                if !u.buttons.is_empty() {
                    let btn = &u.buttons[choice_idx % u.buttons.len()];
                    max_hp_delta = 0;
                    hp_delta = 0;
                    for ef in btn.effects {
                        match ef.kind {
                            TaixuEffectKind::Coin => {
                                let next = i64::from(state.gold) + i64::from(ef.number);
                                state.gold = next.max(0) as u32;
                            }
                            TaixuEffectKind::MaxHp => {
                                max_hp_delta = ef.number.clamp(-127, 127) as i8;
                            }
                            TaixuEffectKind::Hp => {
                                hp_delta = ef.number.clamp(-127, 127) as i8;
                            }
                            TaixuEffectKind::RandomBuff => {
                                if state.buffs.len() < cfg.buff_limit as usize {
                                    if let Some((b, _)) = select_shop_buff_for_node(state, choice_idx) {
                                        state.buffs.push(b.to_string());
                                        granted_buff = Some(b.to_string());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        TaixuNodeKind::ServantRegistry => {
            if cfg.spirit_enabled {
                let (srv_id, _, _, _) =
                    L11_TAIXU_SERVANTS[choice_idx % L11_TAIXU_SERVANTS.len()];
                if !state.spirits.iter().any(|s| s == srv_id) {
                    state.spirits.push(srv_id.to_string());
                }
            }
        }
        TaixuNodeKind::SeasonChapter | TaixuNodeKind::BattleNode | TaixuNodeKind::FrameworkMode => {
            let next_gold = i64::from(state.gold) + i64::from(spec.gold_delta);
            state.gold = next_gold.max(0) as u32;
        }
    }

    state.node_index += 1;
    Ok(TaixuNodeOutcome {
        scope: spec.scope.to_string(),
        node_id: spec.id.to_string(),
        kind: spec.kind,
        gold_after: state.gold,
        max_hp_delta,
        hp_delta,
        quiz_passed,
        granted_buff,
    })
}

/// 可持久化 `L11` 赛季图推进器（支持跨赛季选择、分章与往返快照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaixuCampaignRun {
    pub scope: String,
    pub cursor: usize,
    pub visited_nodes: Vec<String>,
    pub state: TaixuRunState,
}

impl TaixuCampaignRun {
    pub fn start_season_run(cfg: &TaixuConfig, scope: &str) -> Result<Self, TaixuError> {
        if !cfg.enable_campaign {
            return Err(TaixuError::CampaignDisabled);
        }
        let canon = if scope == "HaiWaiFenghe" { "HaiWaiFengHe" } else { scope };
        let has_nodes = L11_TAIXU_CAMPAIGN_NODES.iter().any(|n| n.scope == canon)
            || L11_TAIXU_SEASONS.iter().any(|s| s.id == scope);
        if !has_nodes {
            return Err(TaixuError::UnknownCommand(format!("unknown_season:{scope}")));
        }
        let mut state = TaixuRunState::from_config(cfg);
        state.season = canon.to_string();
        Ok(Self {
            scope: canon.to_string(),
            cursor: 0,
            visited_nodes: Vec::new(),
            state,
        })
    }

    pub fn available_next_nodes(&self) -> Vec<&'static TaixuCampaignNodeSpec> {
        L11_TAIXU_CAMPAIGN_NODES
            .iter()
            .filter(|n| n.scope == self.scope)
            .skip(self.cursor)
            .take(3)
            .collect()
    }

    pub fn advance_node(
        &mut self,
        cfg: &TaixuConfig,
        branch_idx: usize,
        choice_idx: usize,
    ) -> Result<TaixuNodeOutcome, TaixuError> {
        let choices = self.available_next_nodes();
        if choices.is_empty() {
            self.state.completed = true;
            return Err(TaixuError::AlreadyCompleted);
        }
        let chosen = choices.get(branch_idx).copied().ok_or_else(|| {
            TaixuError::UnknownCommand(format!("branch {branch_idx} out of range"))
        })?;
        let out = execute_campaign_spec(cfg, &mut self.state, chosen, choice_idx)?;
        self.visited_nodes.push(format!("{}@{}", chosen.id, chosen.section));
        self.cursor += branch_idx + 1;
        if self.available_next_nodes().is_empty() {
            self.state.completed = true;
        }
        Ok(out)
    }

    pub fn encode_snapshot(&self) -> String {
        let state_line = crate::taixu::snapshot_taixu(&self.state);
        format!("run|{}|{}|{}|{state_line}", self.scope, self.cursor, self.visited_nodes.join(","))
    }

    pub fn decode_snapshot(raw: &str) -> Result<Self, TaixuError> {
        let mut parts = raw.splitn(5, '|');
        let tag = parts.next().unwrap_or("");
        if tag != "run" {
            return Err(TaixuError::SnapshotMalformed("missing run prefix".to_string()));
        }
        let scope = parts
            .next()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing run scope".to_string()))?
            .to_string();
        let cursor = parts
            .next()
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing cursor".to_string()))?
            .parse::<usize>()
            .map_err(|_| TaixuError::SnapshotMalformed("bad cursor".to_string()))?;
        let visited_raw = parts
            .next()
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing visited".to_string()))?;
        let visited_nodes = if visited_raw.is_empty() {
            Vec::new()
        } else {
            visited_raw.split(',').map(|s| s.to_string()).collect()
        };
        let state_raw = parts
            .next()
            .ok_or_else(|| TaixuError::SnapshotMalformed("missing state".to_string()))?;
        let state = crate::taixu::restore_taixu(state_raw)?;
        Ok(Self {
            scope,
            cursor,
            visited_nodes,
            state,
        })
    }
}

/// `buff_txhj_bowenqiangzhi`（博闻强识）：在真实答题中排除一个 `result == false` 的错项。
pub fn exam_excluded_wrong_option_with_correct(
    state: &TaixuRunState,
    option_count: usize,
    correct_idx: usize,
    seed: u64,
) -> Option<usize> {
    if option_count <= 1 || !state.buffs.iter().any(|b| b == "buff_txhj_bowenqiangzhi") {
        return None;
    }
    let wrong: Vec<usize> = (0..option_count).filter(|&i| i != correct_idx).collect();
    if wrong.is_empty() {
        None
    } else {
        Some(wrong[(seed as usize) % wrong.len()])
    }
}

/// 兼容既有 3 参签名（默认正确答案在 0 号位时排除 `1..option_count`）。
pub fn exam_excluded_wrong_option(
    state: &TaixuRunState,
    option_count: usize,
    seed: u64,
) -> Option<usize> {
    exam_excluded_wrong_option_with_correct(state, option_count, 0, seed)
}

/// `buff_txhj_majundegaizao`（马钧的改造）：基础装备槽升阶为 `rewrite_*`。
pub fn upgrade_taixu_equip_majun<'a>(state: &TaixuRunState, equip_name: &'a str) -> &'a str {
    if !state.buffs.iter().any(|b| b == "buff_txhj_majundegaizao") {
        return equip_name;
    }
    taixu_majun_upgrade_name(equip_name)
}

/// `buff_txhj_xianhujiqu`（仙葫汲取）：非败北结束时保留战斗中获得的体力上限提升。
pub fn carryover_max_hp_after_battle(
    state: &TaixuRunState,
    battle_max_hp: u32,
    campaign_max_hp: u32,
    battle_survived: bool,
) -> u32 {
    if battle_survived
        && battle_max_hp > campaign_max_hp
        && state.buffs.iter().any(|b| b == "buff_txhj_xianhujiqu")
    {
        campaign_max_hp + (battle_max_hp - campaign_max_hp)
    } else {
        campaign_max_hp
    }
}
