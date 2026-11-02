//! 选择请求与应答：内核唯一的选择通道，类型化选项（无 `Vec<u32>` 裸载荷）。

use crate::ids::{CardId, PlayerId, SkillDefId};

/// 选择种类（供宿主 UI 渲染分类用，不参与规则分支）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceKind {
    /// 确认/放弃。
    Confirm,
    /// 从选项中选牌。
    SelectCards,
    /// 从选项中选玩家。
    SelectPlayers,
    /// 出牌响应（含放弃）。
    RespondCard,
    /// 从定界标签集中选择（G1 五行：金/木/水/火/土）。
    PickTag,
}

/// 单个可选项。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChoiceOption {
    /// 确认/使用。
    Confirm,
    /// 放弃/跳过（等价 cancel 的显式选项）。
    Pass,
    Card(CardId),
    Player(PlayerId),
    /// 定界标签（G1 五行属性 0..=4：金/木/水/火/土）。
    Tag(u8),
    /// 目录牌种码（`catalog::kind_code`）——「按牌名选择」类询问
    /// （T11 Boss 诸葛亮族批 116 起风：从可用锦囊牌名中选一张并使用）。
    Kind(u16),
    /// Stable skill definition selected as the next trigger.
    Skill(SkillDefId),
}

impl ChoiceOption {
    pub fn describe(&self) -> String {
        match self {
            ChoiceOption::Confirm => "confirm".into(),
            ChoiceOption::Pass => "pass".into(),
            ChoiceOption::Card(c) => format!("card:{c}"),
            ChoiceOption::Player(p) => format!("player:{p}"),
            ChoiceOption::Tag(t) => format!("nature:{t}"),
            ChoiceOption::Kind(k) => format!("kind:{k}"),
            ChoiceOption::Skill(s) => format!("skill:{s}"),
        }
    }
}

/// 选择请求规格。内核负责校验应答是规格的子集且数量合规。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceSpec {
    pub kind: ChoiceKind,
    /// 内容侧提示文本（静态串，进目录不进状态——状态只存目录引用编号）。
    pub prompt: &'static str,
    pub options: Vec<ChoiceOption>,
    pub min: usize,
    pub max: usize,
    /// 允许空应答表示放弃。可选触发的「放弃」即空应答 + allow_cancel。
    pub allow_cancel: bool,
}

impl ChoiceSpec {
    pub fn confirm(prompt: &'static str) -> Self {
        Self {
            kind: ChoiceKind::Confirm,
            prompt,
            options: vec![ChoiceOption::Confirm],
            min: 1,
            max: 1,
            allow_cancel: true,
        }
    }

    /// 校验应答：全部选项都在规格内、数量在 [min, max]、无重复、或合法的空放弃。
    ///
    /// 重复守卫是信任边界的一部分：同一张牌/玩家选两次会让下游
    /// 「从手牌移除」类结算二次移除同一实例而 panic。
    pub fn accepts(&self, selection: &[ChoiceOption]) -> bool {
        if selection.is_empty() {
            return self.allow_cancel || self.min == 0;
        }
        if selection.len() < self.min || selection.len() > self.max {
            return false;
        }
        selection.iter().all(|s| self.options.contains(s))
            && (0..selection.len()).all(|i| !selection[i + 1..].contains(&selection[i]))
    }
}

/// 已开启、等待应答的选择（宿主可见视图 + 挂起的机器一并保存）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenChoice {
    pub id: u64,
    pub observer: PlayerId,
    pub spec: ChoiceSpec,
}

/// 应答结果（内核校验通过后写回，供机器读取一次）。
pub type Selection = Vec<ChoiceOption>;
