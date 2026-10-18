//! L12 如真似幻（noname-liuli）的可移植决策。
//!
//! 上游是**纯渲染/大厅**扩展（596 KB 主文件 + 5.9 MB 资源脚本）：`lib.skill`
//! 0 个、`lib.card` 2 处（仅牌表引用）、`lib.character` 8 处（展示用），
//! 无 `extension_如真似幻_*` 配置键；59 个函数中绝大多数直接操作 DOM/WebGL。
//! 本模块只迁移**语义明确的纯函数**：
//! - `numtoroma`：1–6 → 罗马数字（大厅段位/关卡编号展示）；
//! - `getRandomSanguoText` / `getRandomTip`：从固定文案表随机取一条
//!   （数据表随模块迁移；随机性由调用方给定 `roll`，可复现）。
//!
//! 渲染/动画/大厅流程（WebGL、spine、抽屉、按钮）**不迁移**（登记差异）。
//! 本模块不读写文件、不请求网络、不 eval。

pub const SANGUO_TEXTS: [&str; 11] = [
    "三国杀是一款流行的桌面卡牌游戏，基于三国历史背景。",
    "在三国杀中，玩家需要策略地使用各种角色卡牌来击败对手。",
    "诸葛亮、曹操和刘备是三国杀中的著名角色。",
    "游戏中的卡牌包括杀、闪、桃等各种不同的功能。",
    "每位角色都有独特的技能和特点，增加了游戏的变化性。",
    "三国杀的策略性和战术性使其成为一款受欢迎的卡牌游戏。",
    "在三国杀中，胜利需要巧妙地使用卡牌和角色技能。",
    "游戏中的合作和背叛元素增加了战局的紧张感。",
    "使用你喜欢的武将，积累武将经验，可以获得炫酷的武将表现效果",
    "付费购买的武将及招募的武将均可以分解成一定数量的将魂",
    "开通会员后，可以加速等级的提升哦",
];

pub const TIPS: [&str; 4] = [
    "寻找对手中,不要着急哦~",
    "回合结束时,能保留的手牌数不能超过当前体力值",
    "[万箭齐发] : ( 锦囊 ) 所有人,展现你灵巧的身姿吧",
    "[无懈可击] : ( 锦囊 ) 休想得逞",
];

/// `numtoroma`：1–6 → 罗马数字；其余返回 `None`。
pub fn roman(num: u32) -> Option<&'static str> {
    match num {
        1 => Some("Ⅰ"),
        2 => Some("Ⅱ"),
        3 => Some("Ⅲ"),
        4 => Some("Ⅳ"),
        5 => Some("Ⅴ"),
        6 => Some("Ⅵ"),
        _ => None,
    }
}

/// `getRandomSanguoText`：按 `roll` 取文案（表内取模；空表返回 `None`）。
pub fn sanguo_text(roll: usize) -> Option<&'static str> {
    if SANGUO_TEXTS.is_empty() {
        return None;
    }
    SANGUO_TEXTS.get(roll % SANGUO_TEXTS.len()).copied()
}

/// `getRandomTip`：按 `roll` 取提示（表内取模；空表返回 `None`）。
pub fn tip(roll: usize) -> Option<&'static str> {
    if TIPS.is_empty() {
        return None;
    }
    TIPS.get(roll % TIPS.len()).copied()
}
