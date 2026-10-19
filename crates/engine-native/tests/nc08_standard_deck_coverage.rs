//! NC08 附加：**标准牌组构成核对**——「`CardKind` 已实现」vs「牌组含实例」。
//!
//! 背景（第 54 项附带发现）：`SavageAttack`/`ArcheryAttack` 的 **kind 与行为已实现**，
//! 但**标准牌组 124 张不含实例** ⇒ 夹具不可达。本用例**量化**这一现象：
//! 逐个 `CardKind` 检查标准建局（无扩展）的目录中**是否有实例**。
//!
//! 判据：**不预设「应当全有」**——只输出「**kind 有实现但牌组无实例**」的清单，作为**覆盖面台账**。

use std::collections::BTreeSet;

/// 输出：有实现、但标准牌组无实例的 kind 清单。
#[test]
fn standard_deck_kind_instance_coverage() {
    let e = engine_native::Engine::new_with_extensions(29, 4, &[]).expect("标准建局");
    let cards = &e.catalog().cards;
    let mut present: BTreeSet<String> = BTreeSet::new();
    for (i, _) in cards.iter().enumerate() {
        let k = e.catalog().card_kind(i as engine_native::CardId);
        present.insert(format!("{k:?}"));
    }
    // 枚举全部 CardKind（用 Debug 字符串，避免依赖内部顺序）
    // 通过「全部卡种名」——从 catalog 的 name 映射反推不可行，改为：以 present 之外的常用 kind 名清单核对。
    let known = [
        "Sha", "Shan", "Tao", "Jiu", "Guohe", "Shunshou", "Wuzhong", "Juedou", "Jiedao",
        "Nanman", "Wanjian", "Taoyuan", "Wugu", "Wuxie", "LeBu", "Shandian", "Bingliang",
        "SavageAttack", "ArcheryAttack", "Hufu", "Tianxianjiu", "Caoyao", "Liuxinghuoyu",
        "Chilongya", "Xixueguizhihuan", "Guiyoujie", "Shentoumianju", "Xingjunyan",
        "Qinglonglingzhu", "Zhufangshenshi", "Fengxueren", "Yufulu", "Yangpijuan",
        "Daihuofenglun", "Xiayuncailing", "Langeguaiyi", "Jiguan",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for k in known {
        if !present.contains(k) {
            missing.push(k);
        }
    }
    eprintln!(
        "DBG 标准牌组：目录 {} 张实例、覆盖 {} 个不同 kind\nDBG 有实现但**牌组无实例**的 kind（{} 个）：{:?}",
        cards.len(),
        present.len(),
        missing.len(),
        missing
    );
    // 不设硬断言：本用例的作用是**输出清单**（台账化），避免把「牌组构成」误当缺陷
    eprintln!(
        "NOTE 口径：本清单是「**kind 实现 vs 牌组实例**」的差集；\
         差集非缺陷——部分 kind 只由扩展包提供实例（如 Hufu/Tianxianjiu 属 swd 包）"
    );
}
