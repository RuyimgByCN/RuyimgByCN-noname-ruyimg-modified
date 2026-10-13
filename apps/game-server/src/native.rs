//! T6③ 产品路径：安装包 → 装载 → 原生建局（ADR 0777/0778）。
//!
//! 服务端原生对局生命周期：
//! 1. 安装包：content-runtime 包存储扫描 drop-in 根并安装（幂等）；
//! 2. 装载：读取安装后包的 `assets/cards.json`，经 content-native-loader
//!    桥接为 engine-native `ExtensionPack`（符号表上限门在装载器内）；
//! 3. 原生建局：`Engine::new_with_extensions` 合并目录建局；
//! 4. 有界 AI 驱动（T13 衔接）：`BoundedPlanner` 逐座规划/应用，
//!    直至终局或步数上限。
//!
//! 与旧线（engine-core drop-in `extensions/` 的 npack 安装）互不干扰：
//! 原生路径的包数据根与包存储各自独立配置。

use content_native_loader::BridgedPack;
use content_runtime::{import_extensions, InstallOptions, InstalledMeta, PackageStore};
use engine_native::ai::{ApplyPlanError, BoundedPlanner};
use engine_native::content::extension::ExtensionPack;
use engine_native::{CommandOutcome, Engine};
use serde::Deserialize;
use std::path::PathBuf;

/// 已安装原生包摘要（列表端点）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NativePackageInfo {
    pub package_id: String,
    pub version: String,
}

/// 原生内容服务：包存储 + drop-in 扫描根。
pub struct NativeContent {
    store: PackageStore,
    root: PathBuf,
}

impl NativeContent {
    /// 打开服务（存储目录不存在则初始化）。
    pub fn open(root: PathBuf, store_root: PathBuf) -> Result<Self, String> {
        let store = PackageStore::open(store_root).map_err(|e| e.to_string())?;
        Ok(Self { store, root })
    }

    fn options() -> InstallOptions {
        InstallOptions {
            activate: true,
            allow_unsigned: true,
            verify_key: None,
            host: content_runtime::tooling_host_versions(),
        }
    }

    fn meta_info(m: &InstalledMeta) -> NativePackageInfo {
        NativePackageInfo {
            package_id: m.package_id.clone(),
            version: m.version.clone(),
        }
    }

    /// 安装：扫描 drop-in 根下包目录并装入存储（重复安装同版本幂等）。
    pub fn import(&self) -> Result<Vec<NativePackageInfo>, String> {
        let report = import_extensions(&self.store, &self.root, &Self::options())
            .map_err(|e| e.to_string())?;
        Ok(report.installed.iter().map(Self::meta_info).collect())
    }

    /// 列出存储中已安装的包（不触发扫描）。
    pub fn installed(&self) -> Result<Vec<NativePackageInfo>, String> {
        let metas = self.store.list_installed().map_err(|e| e.to_string())?;
        Ok(metas.iter().map(Self::meta_info).collect())
    }

    /// 装载：按包 id 找安装记录 → 读安装后 `assets/cards.json` → 装载器桥接。
    pub fn load(&self, package_id: &str) -> Result<LoadedPack, String> {
        let metas = self.store.list_installed().map_err(|e| e.to_string())?;
        let meta = metas
            .iter()
            .find(|m| m.package_id == package_id)
            .ok_or_else(|| format!("包未安装: {package_id}"))?;
        let root = meta
            .extension_root
            .as_ref()
            .ok_or_else(|| format!("包缺少安装根: {package_id}"))?;
        let cards_path = PathBuf::from(root).join("assets/cards.json");
        let json = std::fs::read_to_string(&cards_path)
            .map_err(|e| format!("读取包数据失败 {cards_path:?}: {e}"))?;
        let bridged: BridgedPack =
            content_native_loader::bridge_pack(&json).map_err(|e| e.to_string())?;
        Ok(LoadedPack {
            package_id: package_id.to_string(),
            pack: bridged.pack,
            deferred: bridged.deferred,
        })
    }
}

/// 装载产物：原生扩展包 + 暂缓牌清单。
pub struct LoadedPack {
    pub package_id: String,
    pub pack: ExtensionPack,
    pub deferred: Vec<String>,
}

/// 原生建局请求。
#[derive(Debug, Clone, Deserialize)]
pub struct NativeGameRequest {
    /// 参与建局的扩展包 id（空 = 纯标准目录）。
    #[serde(default)]
    pub packages: Vec<String>,
    pub seed: u64,
    pub seats: u8,
    /// AI 驱动步数上限（0 = 只建局不驱动）。
    #[serde(default = "default_max_steps")]
    pub max_steps: u64,
}

fn default_max_steps() -> u64 {
    2000
}

/// 原生建局与驱动结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NativeGameSummary {
    /// 实际装载成功的包 id（装载失败整体报错，故等于请求集）。
    pub packages: Vec<String>,
    /// 装载器登记的暂缓牌总数（诊断口径，不阻断装载）。
    pub deferred_cards: usize,
    pub seed: u64,
    pub seats: u8,
    /// 合并后目录卡牌数。
    pub catalog_cards: usize,
    /// 纯标准目录卡牌数（对照基线）。
    pub base_catalog_cards: usize,
    /// 实际驱动步数（每次规划/应用记一步）。
    pub steps: u64,
    /// 是否到达终局。
    pub finished: bool,
    /// 终局胜利方（Debug 语义字符串；未终局为 None）。
    pub winner: Option<String>,
    /// 驱动器停滞退出（连续拒绝/取消，未终局）。
    pub stalled: bool,
    /// 建局+驱动实测耗时（毫秒）。
    pub elapsed_ms: u128,
}

/// 安装包 → 装载 → 原生建局 → 有界 AI 驱动。
///
/// 驱动循环：开放询问的座位优先应答，否则当前回合座位行动；
/// 规划经 `BoundedPlanner`（节点预算+时限），应用走主通道。
/// 连续 64 步无进展记 `stalled` 退出（诚实口径，不伪装终局）。
pub fn create_native_game(
    content: &NativeContent,
    req: &NativeGameRequest,
) -> Result<NativeGameSummary, String> {
    let started = std::time::Instant::now();
    let mut loaded = Vec::new();
    for id in &req.packages {
        loaded.push(content.load(id)?);
    }
    let deferred_cards: usize = loaded.iter().map(|l| l.deferred.len()).sum();
    let loaded_ids: Vec<String> = loaded.iter().map(|l| l.package_id.clone()).collect();
    let pack_refs: Vec<&ExtensionPack> = loaded.iter().map(|l| &l.pack).collect();

    let base_catalog_cards = Engine::new(1, req.seats as usize).catalog().cards.len();
    let mut e = Engine::new_with_extensions(req.seed, req.seats as usize, &pack_refs)
        .map_err(|err| format!("原生建局失败: {err:?}"))?;
    let catalog_cards = e.catalog().cards.len();

    let mut steps = 0u64;
    let mut since_progress = 0u64;
    let mut stalled = false;
    if req.max_steps > 0 {
        let planner = BoundedPlanner::new(64).with_deadline_ms(50);
        while e.outcome().is_none() && steps < req.max_steps {
            let seat = match e.open_choice() {
                Some(open) => open.observer,
                None => e.turn().current,
            };
            let plan = e.plan_action(seat, &planner);
            match e.apply_plan(&plan) {
                Ok(CommandOutcome::Accepted { .. })
                | Ok(CommandOutcome::Pending { .. })
                | Ok(CommandOutcome::Suspended) => since_progress = 0,
                Ok(CommandOutcome::Rejected(_)) => since_progress += 1,
                Err(ApplyPlanError::Stale { .. }) => since_progress += 1,
                Err(ApplyPlanError::Cancelled) => {
                    stalled = true;
                    break;
                }
            }
            steps += 1;
            if since_progress >= 64 {
                stalled = true;
                break;
            }
        }
    }

    Ok(NativeGameSummary {
        packages: loaded_ids,
        deferred_cards,
        seed: req.seed,
        seats: req.seats,
        catalog_cards,
        base_catalog_cards,
        steps,
        finished: e.outcome().is_some(),
        winner: e.outcome().map(|o| format!("{:?}", o.winner)),
        stalled,
        elapsed_ms: started.elapsed().as_millis(),
    })
}
