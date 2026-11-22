//! L11 太虚幻境战役会话状态、商店、待决交互与序列化数据类型（`session_types.rs`）。

use serde::{Deserialize, Serialize};

use super::seasons::TaixuBattleTarget;
use super::universal::TaixuBtnResult;
use super::catalog::TaixuEffectKind;

/// 确定性 SplitMix64 伪随机流（完整入快照，保证断点恢复一致）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuSplitMix64 {
    pub state: u64,
}

impl TaixuSplitMix64 {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn gen_range(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            return 0;
        }
        (self.next_u64() % (bound as u64)) as usize
    }

    pub fn chance_thousandths(&mut self, thousandths: u32) -> bool {
        if thousandths >= 1000 {
            return true;
        }
        if thousandths == 0 {
            return false;
        }
        (self.next_u64() % 1000) < u64::from(thousandths)
    }
}

/// 可持久化按钮效果载荷。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuOwnedEffect {
    pub kind_key: String,
    pub number: i32,
    pub result: String,
}

impl TaixuOwnedEffect {
    pub fn kind(&self) -> TaixuEffectKind {
        match self.kind_key.as_str() {
            "coin" => TaixuEffectKind::Coin,
            "exp" => TaixuEffectKind::Exp,
            "hp" => TaixuEffectKind::Hp,
            "maxHp" => TaixuEffectKind::MaxHp,
            "minHs" => TaixuEffectKind::MinHs,
            "maxHs" => TaixuEffectKind::MaxHs,
            "adjust" => TaixuEffectKind::Adjust,
            "maxSkills" => TaixuEffectKind::MaxSkills,
            "card" => TaixuEffectKind::Card,
            "randomCard" => TaixuEffectKind::RandomCard,
            "basic" => TaixuEffectKind::Basic,
            "trick" => TaixuEffectKind::Trick,
            "removeCard" => TaixuEffectKind::RemoveCard,
            "equip" => TaixuEffectKind::Equip,
            "randomEquip" => TaixuEffectKind::RandomEquip,
            "equip1" => TaixuEffectKind::Equip1,
            "equip2" => TaixuEffectKind::Equip2,
            "equip5" => TaixuEffectKind::Equip5,
            "randomUnique" => TaixuEffectKind::RandomUnique,
            "removeEquip" => TaixuEffectKind::RemoveEquip,
            "buff" => TaixuEffectKind::Buff,
            "randomBuff" => TaixuEffectKind::RandomBuff,
            "removeBuff" => TaixuEffectKind::RemoveBuff,
            "skill" => TaixuEffectKind::Skill,
            "attack" => TaixuEffectKind::Attack,
            "defense" => TaixuEffectKind::Defense,
            "assist" => TaixuEffectKind::Assist,
            "burst" => TaixuEffectKind::Burst,
            "removeSkill" => TaixuEffectKind::RemoveSkill,
            _ => TaixuEffectKind::RandomSkill,
        }
    }

    pub fn kind_name(kind: TaixuEffectKind) -> &'static str {
        match kind {
            TaixuEffectKind::Coin => "coin",
            TaixuEffectKind::Exp => "exp",
            TaixuEffectKind::Hp => "hp",
            TaixuEffectKind::MaxHp => "maxHp",
            TaixuEffectKind::MinHs => "minHs",
            TaixuEffectKind::MaxHs => "maxHs",
            TaixuEffectKind::Adjust => "adjust",
            TaixuEffectKind::MaxSkills => "maxSkills",
            TaixuEffectKind::Card => "card",
            TaixuEffectKind::RandomCard => "randomCard",
            TaixuEffectKind::Basic => "basic",
            TaixuEffectKind::Trick => "trick",
            TaixuEffectKind::RemoveCard => "removeCard",
            TaixuEffectKind::Equip => "equip",
            TaixuEffectKind::RandomEquip => "randomEquip",
            TaixuEffectKind::Equip1 => "equip1",
            TaixuEffectKind::Equip2 => "equip2",
            TaixuEffectKind::Equip5 => "equip5",
            TaixuEffectKind::RandomUnique => "randomUnique",
            TaixuEffectKind::RemoveEquip => "removeEquip",
            TaixuEffectKind::Buff => "buff",
            TaixuEffectKind::RandomBuff => "randomBuff",
            TaixuEffectKind::RemoveBuff => "removeBuff",
            TaixuEffectKind::Skill => "skill",
            TaixuEffectKind::RandomSkill => "randomSkill",
            TaixuEffectKind::Attack => "attack",
            TaixuEffectKind::Defense => "defense",
            TaixuEffectKind::Assist => "assist",
            TaixuEffectKind::Burst => "burst",
            TaixuEffectKind::RemoveSkill => "removeSkill",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuOwnedSubButton {
    pub name: String,
    pub result_true: Option<bool>,
    pub effects: Vec<TaixuOwnedEffect>,
}

/// 地图槽位上的具体事件实例（对应 `optional1..3` 及 `procedure` 队列项）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuSlotNode {
    pub slot_seat: u16,
    pub season: String,
    pub chapter: String,
    pub id: String,
    pub kind: String,
    pub level: u8,
    pub change: bool,
    pub dlc_def_idx: Option<usize>,
    pub exam_tier: Option<String>,
    pub override_buttons: Option<Vec<TaixuOwnedSubButton>>,
}

/// 商店四大货架分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaixuShopCategory {
    Buff,
    Card,
    Equip,
    Skill,
}

/// 商店节点持久库存与分类涨价计数（`extension_framework.js:6954-6965`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuShopStock {
    pub level: u8,
    pub card_num: u32,
    pub buff_num: u32,
    pub skill_num: u32,
    pub equip_num: u32,
    pub buffs: Vec<String>,
    pub cards: Vec<String>,
    pub equips: Vec<String>,
    pub skills: Vec<String>,
}

/// 当前打开的按钮视图（支持答题洗牌、`博闻强识`错项标记、`思考一下`插入）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuButtonOptionView {
    pub orig_idx: Option<usize>,
    pub name: String,
    pub result: Option<bool>,
    pub bowen_marked_wrong: bool,
}

impl TaixuButtonOptionView {
    pub fn result_enum(&self) -> TaixuBtnResult {
        match self.result {
            Some(true) => TaixuBtnResult::True,
            Some(false) => TaixuBtnResult::False,
            None => TaixuBtnResult::ThinkLater,
        }
    }
}

/// `choose` 节点选项视图。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuChooseOptionView {
    pub idx: Option<usize>,
    pub name: String,
    pub launches_battle: bool,
    pub opens_sub_trade: bool,
}

/// 待触发战役战斗描述（来自 `lookBoss` 或 `lookChoose`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuPendingBattleSpec {
    pub slot_idx: usize,
    pub event_id: String,
    pub event_kind: String,
    pub season: String,
    pub chapter: String,
    pub dlc_def_idx: Option<usize>,
    pub choose_btn_idx: Option<usize>,
    pub target_str: String,
    pub unique: bool,
    pub max_rounds: u32,
    pub exp: u32,
}

impl TaixuPendingBattleSpec {
    pub fn target(&self) -> TaixuBattleTarget {
        match self.target_str.as_str() {
            "kill" => TaixuBattleTarget::Kill,
            "repair" => TaixuBattleTarget::Repair,
            "subsist" => TaixuBattleTarget::Subsist,
            _ => TaixuBattleTarget::All,
        }
    }
}

/// 升级 9 选 4 选项（`extension_framework.js:9337-9393`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuLevelUpChoice {
    pub kind_key: String,
    pub name: String,
    pub number: i32,
}

/// 会话当前打开或挂起的交互窗。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaixuPendingInteraction {
    Quiz {
        slot_idx: usize,
        tier: String,
        exam_id: String,
        buttons: Vec<TaixuButtonOptionView>,
    },
    GainOrTrade {
        slot_idx: usize,
        event_id: String,
        is_trade: bool,
        is_sub_trade: bool,
        buttons: Vec<TaixuButtonOptionView>,
    },
    Choose {
        slot_idx: usize,
        event_id: String,
        buttons: Vec<TaixuChooseOptionView>,
    },
    Shop {
        slot_idx: usize,
        shop_id: String,
    },
    Battle(TaixuPendingBattleSpec),
    LevelUp {
        choices: Vec<TaixuLevelUpChoice>,
        pending_chapter_clear: bool,
        pending_run_victory: bool,
    },
}

/// 战役结算得分汇总（`extension_framework.js:4420-4527`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuRunScore {
    pub won: bool,
    pub fights: u32,
    pub total_rounds: u32,
    pub max_coin: i32,
    pub value_score: u32,
    pub effect_bonus: u32,
    pub total_score: u32,
}
