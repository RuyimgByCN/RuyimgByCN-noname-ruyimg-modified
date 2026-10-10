//! Card store with stable instance IDs.

use engine_api::{CardDefId, CardId, PlayerId, Suit, ZoneId};
use serde::{Deserialize, Serialize};

/// Stable equipment-slot identity used by package costs and canonical player state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum EquipSlot {
    /// Weapon slot (`equip1`).
    Weapon = 1,
    /// Armor slot (`equip2`).
    Armor = 2,
    /// Defensive mount slot (`equip3`).
    DefensiveMount = 3,
    /// Offensive mount slot (`equip4`).
    OffensiveMount = 4,
    /// Treasure slot (`equip5`).
    Treasure = 5,
}

impl EquipSlot {
    /// Decode a stable equipment-slot tag.
    pub fn from_tag(tag: u8) -> Option<Self> {
        Some(match tag {
            1 => Self::Weapon,
            2 => Self::Armor,
            3 => Self::DefensiveMount,
            4 => Self::OffensiveMount,
            5 => Self::Treasure,
            _ => return None,
        })
    }

    /// Zero-based index in the player's disabled-slot counter array.
    pub const fn index(self) -> usize {
        self as usize - 1
    }
}

/// Card category for Phase 5 standard rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CardKind {
    /// 杀（普通）
    Sha = 1,
    /// 桃
    Tao = 2,
    /// 闪
    Shan = 3,
    /// 酒
    Jiu = 4,
    /// 火杀
    ShaFire = 5,
    /// 雷杀
    ShaThunder = 6,
    /// 武器
    Weapon = 10,
    /// 防具
    Armor = 11,
    /// 进攻马（-1 距离）
    HorseOff = 12,
    /// 防御马（被计算距离 +1）
    HorseDef = 13,
    /// 无中生有
    TrickWuzhong = 20,
    /// 过河拆桥
    TrickGuohe = 21,
    /// 顺手牵羊
    TrickShunshou = 22,
    /// 决斗
    TrickJuedou = 23,
    /// 南蛮入侵
    TrickNanman = 24,
    /// 万箭齐发
    TrickWanjian = 25,
    /// 桃园结义
    TrickTaoyuan = 26,
    /// 五谷丰登（简化：全场各摸 1）
    TrickWugu = 27,
    /// 乐不思蜀（延时）
    TrickLe = 30,
    /// 兵粮寸断（延时）
    TrickBingliang = 31,
    /// 闪电（延时）
    TrickShandian = 32,
    /// 借刀杀人
    TrickJiedao = 33,
    /// 铁索连环
    TrickTiesuo = 34,
    /// 火攻
    TrickHuogong = 35,
    /// 无懈可击（响应锦囊，不可主动打出）
    TrickWuxie = 36,
    /// 其他
    Other = 255,
    /// 宝物（追加在末尾以保持既有序列化变体索引稳定）
    Treasure = 14,
    /// 远交近攻.
    TrickYuanjiao = 37,
    /// 调剂盐梅.
    TrickTiaojiyanmei = 38,
    /// 瞒天过海.
    TrickMantianguohai = 39,
    /// 诱敌深入（仅在【杀】生效前作为响应使用）.
    TrickYoudishenru = 40,
    /// 调兵遣将（全体锦囊）.
    TrickDiaobingqianjiang = 41,
    /// 望梅止渴（单目标锦囊）.
    TrickWangmeizhike = 42,
    /// 以逸待劳（国战：对己方全体，各摸两张再弃两张）.
    TrickYiyi = 43,
    /// 水淹七军（国战：目标弃置全部装备牌或受 1 点雷电伤害）.
    TrickShuiyanqijunx = 44,
    /// 戮力同心（国战：对全部大势力或全部小势力角色；未横置→横置，已横置→摸一张）.
    TrickLulitongxin = 45,
    /// 克复中原（国战：任意目标，各目标视为使用杀或摸牌；蜀势力伤害基数+1/摸两张）.
    TrickKefuzhongyuan = 46,
    /// 固国安邦（国战：自用，摸八弃≥六；吴势力可将弃牌交给其他吴角色）.
    TrickGuguoanbang = 47,
    /// 火烧连营（国战：对下家及其队列所有角色造成 1 点火属性伤害）.
    TrickHuoshaolianying = 48,
    /// 联军盛宴（国战：对己方及一敌方势力全体；己方分配摸/回，敌方摸1并重置）.
    TrickLianjunshengyan = 49,
    /// 文和乱武（国战：全角色为目标；使用者选弃两张类型不同手牌或弃其一张手牌，
    /// 群势力空手牌则补至体力至多 5）.
    TrickWenheluanwu = 50,
    /// 号令天下（国战：对一名体力值非全场最少者使用；其他角色依次二选一
    /// ——弃牌视为对目标用杀 / 弃置或获得目标一张牌）.
    TrickHaolingtianxia = 51,
    /// 敕令（国战：对所有有暗置武将的角色使用；各目标选明置一张武将牌并摸一张、
    /// 失去1点体力或弃置一张装备牌）.
    TrickChiling = 52,
    /// 影（jsrg 虚拟基本牌：黑桃 1，不可直接使用，作为【影】系技能素材）.
    /// Appended at tail to keep existing serialized variant indices stable.
    Ying = 53,
    /// 出其不意（对一名有手牌的其他角色使用：展示其一张手牌，与此牌花色
    /// 不同则对其造成 1 点伤害）.
    /// Appended at tail to keep existing serialized variant indices stable.
    TrickChuqibuyi = 54,
}

impl CardKind {
    /// Legacy suit→basic mapping (bootstrap fallback).
    pub fn from_suit(suit: Suit) -> Self {
        match suit {
            Suit::None => CardKind::Other,
            Suit::Spade => CardKind::Sha,
            Suit::Heart => CardKind::Tao,
            Suit::Club => CardKind::Shan,
            Suit::Diamond => CardKind::Jiu,
        }
    }

    /// Stable tag for hashing.
    pub fn tag(self) -> u8 {
        self as u8
    }

    /// Decode a stable rule-category tag.
    pub fn from_tag(tag: u8) -> Option<Self> {
        Some(match tag {
            1 => Self::Sha,
            2 => Self::Tao,
            3 => Self::Shan,
            4 => Self::Jiu,
            5 => Self::ShaFire,
            6 => Self::ShaThunder,
            10 => Self::Weapon,
            11 => Self::Armor,
            12 => Self::HorseOff,
            13 => Self::HorseDef,
            14 => Self::Treasure,
            20 => Self::TrickWuzhong,
            21 => Self::TrickGuohe,
            22 => Self::TrickShunshou,
            23 => Self::TrickJuedou,
            24 => Self::TrickNanman,
            25 => Self::TrickWanjian,
            26 => Self::TrickTaoyuan,
            27 => Self::TrickWugu,
            30 => Self::TrickLe,
            31 => Self::TrickBingliang,
            32 => Self::TrickShandian,
            33 => Self::TrickJiedao,
            34 => Self::TrickTiesuo,
            35 => Self::TrickHuogong,
            36 => Self::TrickWuxie,
            37 => Self::TrickYuanjiao,
            38 => Self::TrickTiaojiyanmei,
            39 => Self::TrickMantianguohai,
            40 => Self::TrickYoudishenru,
            41 => Self::TrickDiaobingqianjiang,
            42 => Self::TrickWangmeizhike,
            43 => Self::TrickYiyi,
            44 => Self::TrickShuiyanqijunx,
            45 => Self::TrickLulitongxin,
            46 => Self::TrickKefuzhongyuan,
            47 => Self::TrickGuguoanbang,
            48 => Self::TrickHuoshaolianying,
            49 => Self::TrickLianjunshengyan,
            50 => Self::TrickWenheluanwu,
            51 => Self::TrickHaolingtianxia,
            52 => Self::TrickChiling,
            53 => Self::Ying,
            54 => Self::TrickChuqibuyi,
            255 => Self::Other,
            _ => return None,
        })
    }

    /// Whether this is equipment.
    pub fn is_equip(self) -> bool {
        matches!(
            self,
            CardKind::Weapon
                | CardKind::Armor
                | CardKind::HorseOff
                | CardKind::HorseDef
                | CardKind::Treasure
        )
    }

    /// Equipment slot occupied by this card kind.
    pub fn equip_slot(self) -> Option<EquipSlot> {
        match self {
            Self::Weapon => Some(EquipSlot::Weapon),
            Self::Armor => Some(EquipSlot::Armor),
            Self::HorseDef => Some(EquipSlot::DefensiveMount),
            Self::HorseOff => Some(EquipSlot::OffensiveMount),
            Self::Treasure => Some(EquipSlot::Treasure),
            _ => None,
        }
    }

    /// Delayed judge-area trick.
    pub fn is_delayed_trick(self) -> bool {
        matches!(
            self,
            CardKind::TrickLe | CardKind::TrickBingliang | CardKind::TrickShandian
        )
    }

    /// Any 杀（普通/火/雷）.
    pub fn is_sha(self) -> bool {
        matches!(
            self,
            CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder
        )
    }

    /// Basic card (杀/桃/闪/酒, including elemental 杀).
    pub fn is_basic(self) -> bool {
        self.is_sha() || matches!(self, CardKind::Tao | CardKind::Shan | CardKind::Jiu)
    }

    /// Damage nature when this card is used as 杀 (`rule_api::damage_nature`).
    pub fn sha_nature(self) -> u8 {
        match self {
            CardKind::ShaFire => 1,    // FIRE
            CardKind::ShaThunder => 2, // THUNDER
            _ => 0,                    // NORMAL
        }
    }

    /// 锦囊（含延时、无懈等）.
    pub fn is_trick(self) -> bool {
        matches!(
            self,
            CardKind::TrickWuzhong
                | CardKind::TrickGuohe
                | CardKind::TrickShunshou
                | CardKind::TrickJuedou
                | CardKind::TrickNanman
                | CardKind::TrickWanjian
                | CardKind::TrickTaoyuan
                | CardKind::TrickWugu
                | CardKind::TrickLe
                | CardKind::TrickBingliang
                | CardKind::TrickShandian
                | CardKind::TrickJiedao
                | CardKind::TrickTiesuo
                | CardKind::TrickHuogong
                | CardKind::TrickWuxie
                | CardKind::TrickYuanjiao
                | CardKind::TrickTiaojiyanmei
                | CardKind::TrickMantianguohai
                | CardKind::TrickYoudishenru
                | CardKind::TrickDiaobingqianjiang
                | CardKind::TrickWangmeizhike
                | CardKind::TrickYiyi
                | CardKind::TrickShuiyanqijunx
                | CardKind::TrickLulitongxin
                | CardKind::TrickKefuzhongyuan
                | CardKind::TrickGuguoanbang
                | CardKind::TrickHuoshaolianying
                | CardKind::TrickLianjunshengyan
                | CardKind::TrickWenheluanwu
                | CardKind::TrickHaolingtianxia
                | CardKind::TrickChiling
                | CardKind::TrickChuqibuyi
        )
    }

    /// 卡牌大类（上游 `get.type2` 简化）：0=基本牌, 1=锦囊牌, 2=装备牌。
    /// 用于「弃置两张类型不同的手牌」类约束（如国战文和乱武）.
    pub fn card_type_group(self) -> u8 {
        if self.is_basic() {
            0
        } else if self.is_trick() {
            1
        } else {
            2
        }
    }

    /// Damage-tagged trick cards used by target legality and legacy skill selectors.
    pub fn is_damage_trick(self) -> bool {
        matches!(
            self,
            CardKind::TrickJuedou
                | CardKind::TrickNanman
                | CardKind::TrickWanjian
                | CardKind::TrickShandian
                | CardKind::TrickHuogong
        )
    }
}

/// One card instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    /// Stable instance id.
    pub id: CardId,
    /// Definition id.
    pub def: CardDefId,
    /// Rule category.
    pub kind: CardKind,
    /// Suit.
    pub suit: Suit,
    /// Rank 1–13 (A=1 … K=13).
    pub rank: u8,
    /// Weapon range (0 if not a weapon).
    pub weapon_range: u8,
    /// Current zone.
    pub zone: ZoneId,
    /// Owner player if in a personal zone; NONE otherwise.
    pub owner: PlayerId,
    /// Face-up flag.
    pub face_up: bool,
}

/// Dense card store; ids never reused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CardStore {
    next_id: u32,
    cards: Vec<Option<Card>>,
}

impl CardStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a card.
    pub fn alloc(&mut self, mut card: Card) -> CardId {
        self.next_id = self.next_id.max(1);
        let id = CardId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        card.id = id;
        let idx = (id.raw() - 1) as usize;
        while self.cards.len() <= idx {
            self.cards.push(None);
        }
        self.cards[idx] = Some(card);
        id
    }

    /// Get card.
    pub fn get(&self, id: CardId) -> Option<&Card> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.cards.get(idx).and_then(|c| c.as_ref())
    }

    /// Get mutable card.
    pub fn get_mut(&mut self, id: CardId) -> Option<&mut Card> {
        if id.is_none() {
            return None;
        }
        let idx = (id.raw() - 1) as usize;
        self.cards.get_mut(idx).and_then(|c| c.as_mut())
    }

    /// Live cards sorted by id.
    pub fn sorted_live(&self) -> Vec<&Card> {
        let mut v: Vec<&Card> = self.cards.iter().filter_map(|c| c.as_ref()).collect();
        v.sort_by_key(|c| c.id.raw());
        v
    }

    /// Next id.
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// Total live cards.
    pub fn live_count(&self) -> usize {
        self.cards.iter().filter(|c| c.is_some()).count()
    }
}
