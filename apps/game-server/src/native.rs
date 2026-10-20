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
    ///
    /// **空结果检查**（2026-10-19）：若 drop-in 根**非空**却**一个包都没装上**，
    /// 返回**错误**而不是静默成功——最常见的原因是**包目录是符号链接**
    /// （扫描不跟随 symlink），或目录里缺 `package.npack.json`。
    /// 此前这种情形返回 `Ok(空)`，部署者会误以为安装成功。
    pub fn import(&self) -> Result<Vec<NativePackageInfo>, String> {
        let report = import_extensions(&self.store, &self.root, &Self::options())
            .map_err(|e| e.to_string())?;
        let out: Vec<NativePackageInfo> = report.installed.iter().map(Self::meta_info).collect();
        if out.is_empty() {
            if let Ok(rd) = std::fs::read_dir(&self.root) {
                let entries = rd.flatten().count();
                if entries > 0 {
                    return Err(format!(
                        "drop-in 根非空（{entries} 个条目）却未安装任何包：\
                         包目录须为**真实目录**（不支持符号链接）且含 package.npack.json"
                    ));
                }
            }
        }
        Ok(out)
    }

    /// 列出存储中已安装的包（不触发扫描）。
    pub fn installed(&self) -> Result<Vec<NativePackageInfo>, String> {
        let metas = self.store.list_installed().map_err(|e| e.to_string())?;
        Ok(metas.iter().map(Self::meta_info).collect())
    }

    /// 装载：**以已验证的安装副本为权威**（NC02）——按内容锁装载并校验
    /// logic/content 指纹，卡牌资源经 store 解析安装目录读取；不再回读
    /// drop-in 原始来源目录（`extension_root` 仅保留来源信息）。
    ///
    /// 同 id 多命名空间时报错要求显式命名空间；见 [`Self::load_scoped`]。
    pub fn load(&self, package_id: &str) -> Result<LoadedPack, String> {
        self.load_scoped(None, package_id)
    }

    /// 装载（可指定命名空间）：`namespace = None` 时若同 id 出现在多个
    /// 命名空间则明确失败，不静默挑一个。
    pub fn load_scoped(
        &self,
        namespace: Option<&str>,
        package_id: &str,
    ) -> Result<LoadedPack, String> {
        use engine_native::content::extension::{ExtensionDef, ExtensionPack};

        // ① 内容锁 + 安装副本（load_locked_packages 校验 logic/content 指纹）
        let lock = self.store.current_lock().map_err(|e| e.to_string())?;
        let packages = self
            .store
            .load_locked_packages(&lock)
            .map_err(|e| format!("装载安装副本失败: {e}"))?;
        let matched: Vec<&content_schema::ContentPackage> = packages
            .iter()
            .filter(|p| {
                p.manifest.package_id == package_id
                    && namespace
                        .map(|ns| p.manifest.namespace == ns)
                        .unwrap_or(true)
            })
            .collect();
        let parsed: &content_schema::ContentPackage = match matched.len() {
            0 => return Err(format!("包未安装或未激活: {package_id}")),
            1 => matched[0],
            n => {
                return Err(format!(
                    "包 id 在 {n} 个命名空间下存在，请显式指定命名空间: {package_id}"
                ))
            }
        };
        let ns = parsed.manifest.namespace.clone();
        let pid = parsed.manifest.package_id.clone();

        // ② 卡牌资源：按 asset_paths 里的相对路径解析到**安装目录**（不读源目录）
        let cards_path = self
            .store_asset_path(parsed, "assets/cards.json")
            .map_err(|e| format!("解析安装资源失败: {e}"))?;

        let (pack, deferred) = {
            let outcome = content_native_loader::bridge_vm_skill_decls(parsed)
                .map_err(|e| e.to_string())?;
            let vm_skills = outcome.decls;
            if let Some(cards_path) = cards_path {
                let json = std::fs::read_to_string(&cards_path)
                    .map_err(|e| format!("读取安装包数据失败 {cards_path:?}: {e}"))?;
                let b: BridgedPack =
                    content_native_loader::bridge_pack(&json).map_err(|e| e.to_string())?;
                let mut pack = b.pack;
                // 混合包：VM 声明并入桥接产物（不得因存在 cards.json 丢技能）
                pack.vm_skills.extend(vm_skills);
                (pack, b.deferred)
            } else {
                // vm-only 包：无卡牌桥接面，元数据取自锁内 manifest。
                (
                    ExtensionPack {
                        def: ExtensionDef {
                            name: engine_native::content::names::intern(pid.clone()),
                            version: parsed
                                .manifest
                                .version
                                .split('.')
                                .next()
                                .and_then(|v| v.parse::<u32>().ok())
                                .unwrap_or(0),
                            description: engine_native::content::names::intern(ns.clone()),
                        },
                        generals: Vec::new(),
                        cards: Vec::new(),
                        host_skills: Vec::new(),
                        vm_skills,
                        init_hook: engine_native::content::extension::InitHook::None,
                        enabled: true,
                    },
                    Vec::new(),
                )
            }
        };
        Ok(LoadedPack {
            package_id: package_id.to_string(),
            pack,
            deferred,
        })
    }

    /// 包内资产相对路径 → 安装目录中的不可变路径（未登记该路径时返回 None）。
    fn store_asset_path(
        &self,
        package: &content_schema::ContentPackage,
        rel: &str,
    ) -> Result<Option<PathBuf>, String> {
        let Some((asset_id, _)) = package
            .asset_paths
            .iter()
            .find(|(_, path)| path.as_str() == rel)
        else {
            return Ok(None);
        };
        self.store
            .resolve_active_asset(
                &package.manifest.namespace,
                &package.manifest.package_id,
                asset_id,
            )
            .map(Some)
            .map_err(|e| e.to_string())
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
    /// NC05 正式选将：按座位顺序给出武将名（空/缺省 = 保持建局随机装配）。
    /// 走 `Engine::assign_general_named`（产品入口，非 `debug_grant`）。
    #[serde(default)]
    pub generals: Vec<String>,
    pub seed: u64,
    pub seats: u8,
    /// AI 驱动步数上限（0 = 只建局不驱动）。
    #[serde(default = "default_max_steps")]
    pub max_steps: u64,
    /// 规划节点预算（T13 策略质量评估口：0 = 无规划基线，动作退化为
    /// NextPhase 兜底；默认 64=现状）。
    #[serde(default = "default_planner_nodes")]
    pub planner_nodes: usize,
    /// 搜索深度（T13 更强候选：1 = 单层启发式默认；2 = 两层）。
    #[serde(default = "default_planner_lookahead")]
    pub planner_lookahead: u8,
    /// 策略变体（T13 更强候选：0 = 现状仅杀候选/体力差评分；1 = 全手牌
    /// 候选 + 牌值评分）。
    #[serde(default)]
    pub planner_variant: u8,
}

fn default_planner_nodes() -> usize {
    64
}

fn default_planner_lookahead() -> u8 {
    1
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
    /// 合并后目录武将数（T13 武将包产品面）。
    pub catalog_generals: usize,
    /// 纯标准目录武将数（对照基线）。
    pub base_catalog_generals: usize,
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
    let base_catalog_generals = Engine::new(1, req.seats as usize)
        .catalog()
        .general_count();
    let mut e = Engine::new_with_extensions(req.seed, req.seats as usize, &pack_refs)
        .map_err(|err| format!("原生建局失败: {err:?}"))?;
    let catalog_cards = e.catalog().cards.len();
    let catalog_generals = e.catalog().general_count();

    // NC05 正式选将（产品入口）：按座位顺序装配；失败整体报错。
    for (seat, name) in req.generals.iter().enumerate() {
        e.assign_general_named(seat as u8, name)
            .map_err(|reason| format!("选将失败（座位 {seat} → {name}）: {reason:?}"))?;
    }

    let mut steps = 0u64;
    let mut since_progress = 0u64;
    let mut stalled = false;
    if req.max_steps > 0 {
        let planner = BoundedPlanner::new(req.planner_nodes)
            .with_deadline_ms(50)
            .with_lookahead(req.planner_lookahead)
            .with_policy_variant(req.planner_variant);
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
        catalog_generals,
        base_catalog_generals,
        steps,
        finished: e.outcome().is_some(),
        winner: e.outcome().map(|o| format!("{:?}", o.winner)),
        stalled,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn load_refs<'a>(
    content: &'a NativeContent,
    packages: &[String],
) -> Result<Vec<LoadedPack>, String> {
    let mut loaded = Vec::new();
    for id in packages {
        loaded.push(content.load(id)?);
    }
    Ok(loaded)
}

/// 新对局存档（HTTP `/native/matches`）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NativeMatchArchive {
    pub packages: Vec<String>,
    pub catalog_cards: usize,
    pub archive_hex: String,
}

/// 恢复请求（HTTP `/native/matches/restore`）。
#[derive(Debug, Clone, Deserialize)]
pub struct NativeRestoreRequest {
    #[serde(default)]
    pub packages: Vec<String>,
    pub archive_hex: String,
}

/// 恢复结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NativeRestoreSummary {
    pub catalog_cards: usize,
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("存档十六进制长度须为偶数".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("非法十六进制: {e}")))
        .collect()
}

/// 安装包进入新对局，并交出 NGSA 字节。调用方用同一包 id 列表恢复。
pub fn save_fresh_match(
    content: &NativeContent,
    packages: &[String],
    seed: u64,
    seats: u8,
) -> Result<(Vec<u8>, usize), String> {
    let loaded = load_refs(content, packages)?;
    let refs: Vec<&ExtensionPack> = loaded.iter().map(|l| &l.pack).collect();
    let engine = Engine::new_with_extensions(seed, seats as usize, &refs)
        .map_err(|err| format!("原生建局失败: {err:?}"))?;
    let cards = engine.catalog().cards.len();
    let bytes = engine_native::archive::encode_archive(&engine.save());
    Ok((bytes, cards))
}

/// 用已安装包恢复对局。缺包在装载阶段失败；错版或另一包集合在内容锁校验失败。
/// 成功时返回恢复后的目录牌数。
pub fn restore_match(
    content: &NativeContent,
    packages: &[String],
    bytes: &[u8],
) -> Result<usize, String> {
    let loaded = load_refs(content, packages)?;
    let refs: Vec<&ExtensionPack> = loaded.iter().map(|l| &l.pack).collect();
    let snapshot = engine_native::archive::decode_archive(bytes)
        .map_err(|err| format!("存档解码失败: {err:?}"))?;
    let engine = if refs.is_empty() {
        Engine::restore(&snapshot).map_err(|err| format!("恢复失败: {err:?}"))?
    } else {
        Engine::restore_with_extensions(&snapshot, &refs)
            .map_err(|err| format!("恢复失败: {err:?}"))?
    };
    Ok(engine.catalog().cards.len())
}

/// HTTP 建局：安装包进入新对局并返回存档十六进制。
pub fn open_native_match(
    content: &NativeContent,
    packages: &[String],
    seed: u64,
    seats: u8,
) -> Result<NativeMatchArchive, String> {
    let (bytes, catalog_cards) = save_fresh_match(content, packages, seed, seats)?;
    Ok(NativeMatchArchive {
        packages: packages.to_vec(),
        catalog_cards,
        archive_hex: hex_encode(&bytes),
    })
}

/// HTTP 恢复：同一安装集恢复；缺包或错版返回错误。
pub fn restore_native_match(
    content: &NativeContent,
    req: &NativeRestoreRequest,
) -> Result<NativeRestoreSummary, String> {
    let bytes = hex_decode(&req.archive_hex)?;
    let catalog_cards = restore_match(content, &req.packages, &bytes)?;
    Ok(NativeRestoreSummary { catalog_cards })
}
