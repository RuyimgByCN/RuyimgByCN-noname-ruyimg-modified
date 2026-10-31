//! NC05 第二类案例入口：**带扩展的 Boss 关卡建局**（战役/跨局档案）。
//!
//! 既有构造入口 `new_boss_stage` 不带扩展目录，`new_with_extensions` 不带 Boss
//! 模式；战役扩展（如 `boss-play-upstream-liuli`）需要两者组合。本模块提供
//! 产品构造入口，语义与两者一致：目录由启用扩展合并构建（内容锁语义），模式
//! 为 Boss 关卡，开局按 `OpeningPlan::BossStage` 发牌。

use crate::content::extension::{ExtensionError, ExtensionPack};
use crate::engine::Engine;
use crate::engine::OpeningPlan;
use crate::ids::PlayerId;
use crate::state::{BossStageState, GameMode};

impl Engine {
    /// 带扩展的 Boss 关卡建局（NC05 第二类案例入口）。
    ///
    /// 参数与 `new_boss_stage` 一致，另加启用扩展集合；目录构建失败（如符号表
    /// 上限）返回 `ExtensionError`，不产出引擎。
    #[allow(clippy::too_many_arguments)]
    pub fn new_boss_stage_with_extensions(
        seed: u64,
        seats: usize,
        boss: PlayerId,
        chongzheng: u8,
        boss_cards: u32,
        player_cards: u32,
        first: PlayerId,
        packs: &[&ExtensionPack],
    ) -> Result<Self, ExtensionError> {
        let catalog = crate::content::standard::standard_catalog_with_extensions(packs)?;
        let catalog: crate::catalog::SharedCatalog = std::sync::Arc::new(catalog);
        let mut e = Self::new_with_catalog(
            seed,
            seats,
            GameMode::Boss {
                boss,
                chongzheng,
                fellows: Vec::new(),
                stage: BossStageState::default_none(),
            },
            catalog,
            OpeningPlan::BossStage {
                boss_cards,
                player_cards,
                first,
            },
        );
        // 扩展包开局初始化钩子（wuxing 五行 / swd 机巧）按与
        // `new_with_extensions` 相同口径处理：钩子效果由各自包的首回合触发
        // 承载，构造入口不额外写状态（登记差异：构造期不预置钩子效果）。
        let _ = &mut e;
        Ok(e)
    }

    /// 战役档案只读快照：`(关卡等级, 孟婆已出现, 孟婆已死亡)`（非 Boss 模式 None）。
    pub fn boss_stage_progress(&self) -> Option<(u8, bool, bool)> {
        self.state_core()
            .boss_stage()
            .map(|s| (s.level, s.mengpo_spawned, s.mengpo_died))
    }
}
