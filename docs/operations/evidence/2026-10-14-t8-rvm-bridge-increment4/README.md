# T8 增量 4：game-server /native 磁盘路径接装载器 VM 桥（2026-10-14）

## 任务来源

[T8 增量 3](../2026-10-14-t8-rvm-bridge-increment3/README.md) 登记的「game-server `/native` 磁盘路径接线」：drop-in 编译包（`package.npack.json`，含 rule-vm 模块的 npack）→ 安装 → 装载器 `bridge_vm_skill_decls` → `vm_skills` 入 `ExtensionPack` → 建局——服务端产品路径的完整数据流。

## 交付物（apps/game-server/src/native.rs）

- `NativeContent::load` 改为**编译包优先**：安装根存在 `package.npack.json` 时读取并解析 `ContentPackage`，经 `bridge_vm_skill_decls` 产出 `vm_skills` 挂到 `ExtensionPack`；此时 `assets/cards.json` **可选**（vm-only 包——元数据取自 npack manifest：package_id/namespace intern，version 取 semver 首数字段）。无编译包时维持 T6③ 口径（cards.json 必需）不变。
- `create_native_game` 零改动——vm_skills 随 `ExtensionPack` 走既有合并路径（增量 1 的 `standard_catalog_with_extensions`）。
- 依赖调整：`content-schema` 由 dev-dep 移入 [dependencies]；dev-dep 新增 `content-compiler`（测试 DSL 编译）。

## 实测记录

| 项 | 结果 |
|---|---|
| `t8_native_disk_path_vm_package_end_to_end` | drop-in 编译包（vm-only，无 cards.json）→ 安装 → 装载（vm_skills=1，声明名=`hnl_jiuwei_t8_phase_end`）→ `new_with_extensions` 建局 → 授技 → 阶段结束 hp 2→3 落账 |
| T6③ 回归 | `t63_import_load_build_and_drive_with_extension`（cards.json 路径不变绿）/`t63_standard_only_build_and_drive` 等 3 测不变绿 |
| game-server 全量 | **47/0/0**（46→47，新增 1 测）——[日志](game-server-tests.log) |
| engine-native 全量 | **662/0/0**（不受影响）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 坑位实录

- npack manifest `version` 为类 semver 字符串、无 description 字段——`ExtensionDef`（name/version:u32/description:&'static str）映射：version 取首数字段（失败 0，生命周期指纹另含内容锁）、description 用 namespace intern。
- `NativeContent::open` 形参为 `PathBuf`（非 `&Path`）——测试传参同步。

## 登记边界（后续增量）

- DSL filter 表达式映射（现恒 owner 域）；`HostContext.skill_id` 反查；剩余效果/查询变体与查询映射。
- 跨进程恢复、恶意字节码拒收矩阵（C1 验收剩余项）。
