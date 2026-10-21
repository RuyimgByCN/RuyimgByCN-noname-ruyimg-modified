use super::*;

pub(super) fn cards() -> Vec<CardDef> {
    let mut cards = Vec::new();
    // Sha x30
    for (st, rk, cnt) in [
        (Suit::Spade, 7u8, 1),
        (Suit::Spade, 8u8, 2),
        (Suit::Spade, 9u8, 2),
        (Suit::Spade, 10u8, 2),
        (Suit::Club, 2u8, 1),
        (Suit::Club, 3u8, 1),
        (Suit::Club, 4u8, 1),
        (Suit::Club, 5u8, 1),
        (Suit::Club, 6u8, 1),
        (Suit::Club, 7u8, 1),
        (Suit::Club, 8u8, 2),
        (Suit::Club, 9u8, 2),
        (Suit::Club, 10u8, 2),
        (Suit::Club, 11u8, 2),
        (Suit::Heart, 10u8, 2),
        (Suit::Heart, 11u8, 1),
        (Suit::Diamond, 6u8, 1),
        (Suit::Diamond, 7u8, 1),
        (Suit::Diamond, 8u8, 1),
        (Suit::Diamond, 9u8, 1),
        (Suit::Diamond, 10u8, 1),
        (Suit::Diamond, 13u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("sha", CardKind::Sha).with_suit_rank(st, rk));
        }
    }
    // Shan x15
    for (st, rk, cnt) in [
        (Suit::Heart, 2u8, 2),
        (Suit::Heart, 13u8, 1),
        (Suit::Diamond, 2u8, 2),
        (Suit::Diamond, 3u8, 1),
        (Suit::Diamond, 4u8, 1),
        (Suit::Diamond, 5u8, 1),
        (Suit::Diamond, 6u8, 1),
        (Suit::Diamond, 7u8, 1),
        (Suit::Diamond, 8u8, 1),
        (Suit::Diamond, 9u8, 1),
        (Suit::Diamond, 10u8, 1),
        (Suit::Diamond, 11u8, 2),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("shan", CardKind::Shan).with_suit_rank(st, rk));
        }
    }
    // Tao x8
    for (st, rk, cnt) in [
        (Suit::Heart, 3u8, 1),
        (Suit::Heart, 4u8, 1),
        (Suit::Heart, 6u8, 1),
        (Suit::Heart, 7u8, 1),
        (Suit::Heart, 8u8, 1),
        (Suit::Heart, 9u8, 1),
        (Suit::Heart, 12u8, 1),
        (Suit::Diamond, 12u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("tao", CardKind::Tao).with_suit_rank(st, rk));
        }
    }
    // SavageAttack x3
    for (st, rk, cnt) in [
        (Suit::Spade, 7u8, 1),
        (Suit::Spade, 13u8, 1),
        (Suit::Club, 7u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("savage", CardKind::SavageAttack).with_suit_rank(st, rk));
        }
    }
    // Duel x3
    for (st, rk, cnt) in [
        (Suit::Spade, 1u8, 1),
        (Suit::Club, 1u8, 1),
        (Suit::Diamond, 1u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("duel", CardKind::Duel).with_suit_rank(st, rk));
        }
    }
    // Snatch x5
    for (st, rk, cnt) in [
        (Suit::Spade, 3u8, 1),
        (Suit::Spade, 4u8, 1),
        (Suit::Spade, 11u8, 1),
        (Suit::Diamond, 3u8, 1),
        (Suit::Diamond, 4u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("snatch", CardKind::Snatch).with_suit_rank(st, rk));
        }
    }
    // Dismantle x6
    for (st, rk, cnt) in [
        (Suit::Spade, 3u8, 1),
        (Suit::Spade, 4u8, 1),
        (Suit::Spade, 12u8, 1),
        (Suit::Club, 3u8, 1),
        (Suit::Club, 4u8, 1),
        (Suit::Heart, 12u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("dismantle", CardKind::Dismantle).with_suit_rank(st, rk));
        }
    }
    // Nullify x4
    for (st, rk, cnt) in [
        (Suit::Spade, 11u8, 1),
        (Suit::Club, 12u8, 1),
        (Suit::Club, 13u8, 1),
        (Suit::Diamond, 12u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("nullify", CardKind::Nullify).with_suit_rank(st, rk));
        }
    }
    // Le x3
    for (st, rk, cnt) in [
        (Suit::Spade, 6u8, 1),
        (Suit::Club, 6u8, 1),
        (Suit::Heart, 6u8, 1),
    ] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("le", CardKind::Le).with_suit_rank(st, rk));
        }
    }
    // ShanDian x2（批③-c：上游 ♠1/♥12 逐张落位；♥12 命名 shandian_safe）
    for (st, rk, name) in [
        (Suit::Spade, 1u8, "shandian"),
        (Suit::Heart, 12u8, "shandian_safe"),
    ] {
        cards.push(CardDef::basic(name, CardKind::ShanDian).with_suit_rank(st, rk));
    }
    // —— 原生扩充（G1 前置，上游无对应；花色点数为原生取值）——
    for _ in 0..4 {
        cards.push(CardDef::basic("horse_plus", CardKind::HorsePlus));
    }
    for _ in 0..2 {
        cards.push(CardDef::basic("horse_minus", CardKind::HorseMinus));
    }
    cards.push(CardDef::basic("jiu", CardKind::Jiu).with_suit_rank(Suit::Spade, 3));
    cards.push(CardDef::basic("jiu", CardKind::Jiu).with_suit_rank(Suit::Club, 9));
    cards.push(CardDef::basic("leisha", CardKind::LeiSha).with_suit_rank(Suit::Spade, 4));
    cards.push(CardDef::basic("leisha", CardKind::LeiSha).with_suit_rank(Suit::Club, 5));
    let mut weapon2 = CardDef::basic("weapon2", CardKind::Weapon);
    weapon2.range = 2;
    cards.push(weapon2);
    let mut weapon2b = CardDef::basic("weapon2b", CardKind::Weapon);
    weapon2b.range = 2;
    cards.push(weapon2b);
    let mut weapon3 = CardDef::basic("weapon3", CardKind::Weapon);
    weapon3.range = 3;
    cards.push(weapon3);
    for _ in 0..2 {
        cards.push(CardDef::basic("armor", CardKind::Armor));
    }
    for _ in 0..4 {
        cards.push(CardDef::basic("fire_sha", CardKind::FireSha));
    }
    for _ in 0..4 {
        cards.push(CardDef::basic("chain", CardKind::Chain));
    }
    cards.push(CardDef::basic("archery", CardKind::ArcheryAttack).with_suit_rank(Suit::Heart, 1));
    cards.push(CardDef::basic("bing", CardKind::Bing).with_suit_rank(Suit::Club, 3));
    // P2/N08 缺牌种批①（2026-10-07）：无中生有 ♥7/8/9/11、桃园结义 ♥1
    //（上游 standard list 花色点数原样落位）。
    for (st, rk) in [
        (Suit::Heart, 7),
        (Suit::Heart, 8),
        (Suit::Heart, 9),
        (Suit::Heart, 11),
    ] {
        cards.push(CardDef::basic("wuzhong", CardKind::Wuzhong).with_suit_rank(st, rk));
    }
    cards.push(CardDef::basic("taoyuan", CardKind::Taoyuan).with_suit_rank(Suit::Heart, 1));
    // 批①-2（规则 v63）：五谷丰登 ♥3/♥4、借刀杀人 ♣12/♣13（上游落位）。
    for rk in [3u8, 4] {
        cards.push(CardDef::basic("wugu", CardKind::Wugu).with_suit_rank(Suit::Heart, rk));
    }
    for rk in [12u8, 13] {
        cards.push(CardDef::basic("jiedao", CardKind::Jiedao).with_suit_rank(Suit::Club, rk));
    }
    // 批③-a（规则 v64）：仁王盾 ♣2（防具；锁定黑色杀无效）。
    cards.push(CardDef::basic("renwang", CardKind::Renwang).with_suit_rank(Suit::Club, 2));
    // 批③-b 第一把（规则 v65）：青釭剑 ♠6（武器射程 2；锁定无视目标防具）
    //——命名武器牌种（特效按牌种接线，非泛型 Weapon）。
    let mut qinggang =
        CardDef::basic("qinggang", CardKind::Qinggang).with_suit_rank(Suit::Spade, 6);
    qinggang.range = 2;
    cards.push(qinggang);
    // 批③-b 第二把（规则 v66）：麒麟弓 ♥5（武器射程 5；杀伤害时可弃目标
    // 一匹坐骑）。
    let mut qilin = CardDef::basic("qilin", CardKind::Qilin).with_suit_rank(Suit::Heart, 5);
    qilin.range = 5;
    cards.push(qilin);
    // 批③-b 第三把（规则 v67）：寒冰剑 ♠2（武器射程 2；杀伤害可防止改为
    // 弃目标两张牌）。
    let mut hanbing = CardDef::basic("hanbing", CardKind::Hanbing).with_suit_rank(Suit::Spade, 2);
    hanbing.range = 2;
    cards.push(hanbing);
    // 批③-b 第四把（规则 v68）：诸葛连弩 ♣1/♦1（武器射程 1；出杀不限次）。
    for (st, rk) in [(Suit::Club, 1u8), (Suit::Diamond, 1)] {
        let mut zhuge = CardDef::basic("zhuge", CardKind::Zhuge).with_suit_rank(st, rk);
        zhuge.range = 1;
        cards.push(zhuge);
    }
    // 批③-b 第五把（规则 v69）：贯石斧 ♦5（武器射程 3；杀被闪后可弃
    // 两张牌强制伤害）。
    let mut guanshi = CardDef::basic("guanshi", CardKind::Guanshi).with_suit_rank(Suit::Diamond, 5);
    guanshi.range = 3;
    cards.push(guanshi);
    // 批③-b 第六把（规则 v70）：青龙偃月刀 ♠5（武器射程 3；杀被闪后可
    // 立即对相同目标再使用一张杀）。
    let mut qinglong =
        CardDef::basic("qinglong", CardKind::Qinglong).with_suit_rank(Suit::Spade, 5);
    qinglong.range = 3;
    cards.push(qinglong);
    // 批③-b 第七把（规则 v71）：丈八蛇矛 ♠12（武器射程 3；两张手牌当杀
    // 的配套主动技随装备授予）。
    let mut zhangba = CardDef::basic("zhangba", CardKind::Zhangba).with_suit_rank(Suit::Spade, 12);
    zhangba.range = 3;
    zhangba.equip_skills.push(ZHANGBA);
    cards.push(zhangba);
    // 批③-b 第八把（规则 v72）：雌雄双股剑 ♠2（武器射程 2；对异性目标
    // 出杀：目标弃一张手牌或令使用者摸一张牌）。
    let mut cixiong = CardDef::basic("cixiong", CardKind::Cixiong).with_suit_rank(Suit::Spade, 2);
    cixiong.range = 2;
    cards.push(cixiong);
    // 批③-b 第九把（规则 v73）：方天画戟 ♦5（武器射程 4；最后手牌杀
    // 可指定至多三个目标）。
    let mut fangtian =
        CardDef::basic("fangtian", CardKind::Fangtian).with_suit_rank(Suit::Diamond, 5);
    fangtian.range = 4;
    cards.push(fangtian);
    cards
}
