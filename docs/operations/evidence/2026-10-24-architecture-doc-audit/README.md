# 架构与文档核验（2026-10-24）

核验基准：`7e28a93799a9c01794085e911a72a4faa5673883` 加 17:09 起的未提交工作树；规则 v322、NGSA 格式 v105。此页保存本次审查依据与建议，不是另一份当前状态表。当前优先级只在[进度主稿](../../../architecture/native-progress.md#architecture-audit-2026-10-24)维护；稳定范围沿用[NC01–NC12](../../../architecture/native-convergence-plan.md)，逐来源状态仍由[扩展台账](../../../migration/upstream-extension-reconstruction.md)维护。

## 范围与结论

本项目以 Rust 单写规则核心承载牌局，通过本地 WASM Worker 或服务端原生房间提供权威状态、选择和恢复。审查假定一个服务进程管理多个房间、玩家和旁观者，单房按命令串行执行；多进程共用存档目录不是当前已获保证的部署方式。没有服务器容量或延迟 SLA，因此不根据代码体积推断实测性能。

**保留核心架构，调整边界与验收顺序。** 不重开 Rust／rule-vm 后端决定，不引入第二套状态源、包仓库、分布式存储或新的 UI 框架。已接通的正式包开局、原生 WASM 构建、真 WS、原局包副本、提交日志及双重故障恢复应继续复用。优先处理以下信任边界和隔离问题，再推进已有 NC06／NC07 的实际内容状态迁移。

审查读取了根 README、目标架构、NC 方案、候选方案、扩展契约、主稿、Cargo／前端构建配置和 CI，深入检查本地开局／恢复、Worker 回包、网络重连、房间广播、保存决定与收据路径。不是逐技能规则等价审查，也没有逐文件审计所有旧引擎代码。

## 发现与最小调整

<a id="a1"></a>

### 1. P1：坏十六进制存档会使 WASM 句柄失效

- **具体情况**：`hex_decode` 用字符串下标按两个字节切片，未先验证偶数长度和 UTF-8 边界（`crates/engine-native-wasm/src/lib.rs:78`）。对现有原生 WASM 调用 `restore_hex("0")` 或 `restore_hex("中")`，实际得到 `RuntimeError: unreachable`，随后同句柄 `save_hex()` 报 Rust 别名借用错误。仅应拒绝坏文件的操作会使当前牌局无法继续读取。
- **最小修正**：在 WASM 边界按字节检查长度、ASCII hex 和资源上限，再进入解码；错误回传受控结果，恢复成功前保留原引擎。不能依赖 UI 已校验输入。
- **验收**：奇数、非 ASCII、非法字符、超限和截断均受控失败；错误后原局状态／选择／存档仍可用。已有归档解码器的未知牌种与尾载荷修复不能替代这个更外层的检查。
- **本轮证据**：[探针脚本](wasm-restore-boundary-probe.mjs)、[完整运行元数据](wasm-restore-boundary-probe.json)、[原始输出](wasm-restore-boundary-probe.log)。使用既有生成包，未重建；JS／WASM／Rust 文件摘要运行前后不变。

<a id="a2"></a>

### 2. P1：慢连接可阻塞整个原生房间

- **具体情况**：`apps/game-server/src/session.rs:24` 的注释称慢连接会丢弃，实际 `send().await` 等待队列；`apps/game-server/src/main.rs:390` 建立 64 格队列，socket writer 无发送超时。房间 actor 在 `apps/game-server/src/room_native_actor.rs:478` 和 `apps/game-server/src/room_native_actor.rs:654` 逐成员等待广播。一个停读旁观者填满队列后，该 actor 的后续命令、快照和断线处理也会等待。
- **最小修正**：房间向连接入列不能无限等待；满载时明确断开慢连接并要求重同步，socket writer 加发送超时／取消。不能无声丢失不可替代的权威事件；可合并快照与不可丢事件应区分。
- **验收**：人为填满一个 observer 队列，其他玩家命令和快照在期限内完成；再用真实 WS 停读客户端验证断开及重连同步。此项为源码控制流结论，本轮未做网络压力实测。

<a id="a3"></a>

### 3. P1：浏览器未确认命令的身份没有跨重连保留

- **具体情况**：`apps/web-client/src/engine/networkClient.ts:329` 每次提交生成新连接号衍生的 key；断线在 `apps/web-client/src/engine/networkClient.ts:512` 拒绝未完成请求，`apps/web-client/src/engine/networkClient.ts:554` 清空 pending。若服务端已提交而 ACK 丢失，重连后再次点击仍合法的同一操作会分配新 key，不能命中服务端旧收据；不是服务端同键去重失效。
- **最小修正**：把待确认命令内容和原 key 绑定房间／重连身份保存；入房后先用原 key 解析不确定结果，再允许该意图的新提交。跨刷新支持范围与取消语义需明确，不能把所有错误当成已撤销。
- **验收**：真实 NetworkEngineClient 与 WS 在持久提交后、ACK 前断线，重连用原 key 得到原 Accepted 与当前投影，动作只执行一次；同时覆盖 Prepared 允许重试一次的分支。

<a id="a4"></a>

### 4. P1（多实例部署前）：目录单写者尚未强制

- **具体情况**：`apps/game-server/src/lobby.rs:79` 为配置目录建立 store；`apps/game-server/src/native_room_store.rs:207` 的提交协议没有跨进程排他所有权。两个不同监听端口的服务使用相同 `NONAME_NATIVE_ROOM_STORE` 可以各自恢复、写入同一房间。独占 UUID 临时文件只防名称冲突，不保护整个 journal／正文交易。
- **最小修正**：服务启动时持有存档目录排他锁，第二写者明确拒绝启动，部署契约写明一目录一写者；不先引入分布式 lease。
- **验收**：同目录两进程互斥、持锁进程异常退出后可恢复、不同目录允许并行。当前单进程故障回归不能证明这一点；本轮未跑多写者破坏实验。

<a id="a5"></a>

### 5. P2：收敛产品契约和完整包载荷

- **具体情况**：`apps/web-client/src/worker/nativeBackend.ts:759` 仍返回空 `events`，`apps/web-client/src/worker/nativeBackend.ts:763` 返回空 `hash_hex`；WS 的权威状态 hash 已接通，两者不能写成同一验收结果。`apps/web-client/src/engine/bundledNativePackages.ts:89` 从构建期 bundled 表获取包并明确拒绝 VM modules；已装载的 manifest／卡牌／武将／牌堆切片并不等于第三方作者可以直接发布新规则。
- **最小修正**：按 NC03 建立同源或自动校验的命令结果、座位投影、可见事件、hash／修订契约；按 NC02／NC05 扩展已验证的包载荷与锁，把 VM／配置／资源接入现有装载器及 Worker，复用服务端内容锁原则，不并建另一个包系统。
- **验收**：同包在本地 Worker 与 WS 的同命令轨迹符合各自信息边界；第三方新技能无需重编译客户端即安装、选择、保存恢复、升级／卸载后按原锁恢复。演示卡牌包通过不替代该验收。

<a id="a6"></a>

### 6. P2：实际迁移内容状态，再统一编码与语义摘要

- **具体情况**：`pingzhang_count` 等内容字段仍进入 `PlayerState`（`crates/engine-native-kernel/src/state.rs:78`），同时手写在 `archive/state_encode.rs:162`、`archive/state_decode.rs:117` 与 `engine/hash.rs:209`。`vm_aux` 通用面已存在，但没有让这些真实字段自动退出核心。EffectRequest 拆成小文件解决行数门禁，不等于 NC06／NC07 完成。
- **最小修正**：选一个有生命周期、选择和恢复的真实扩展，把私有状态迁入已有命名空间状态能力；用最小共享遍历或声明统一字段／tag／编码检查，语义 hash 仍显式决定，不预设全过程宏。不要以关键词清零作为架构验收。
- **验收**：普通新技能无需同步扩张核心状态、封闭行为枚举和多份编码表；验证获得／失去／失效、回合结束、死亡、跨进程恢复、未知 tag、长度限制和版本拒绝。规则变化与格式迁移分别登记。

<a id="a7"></a>

### 7. P2：先量测长局持久化成本，再做有界化

- **具体情况**：`apps/game-server/src/room_native_actor/durability.rs:68` 在 Tokio actor 直接同步保存；`apps/game-server/src/room_native.rs:311` 导出完整原包副本和所有收据，journal 另含上一份完整正文（`apps/game-server/src/native_room_store/journal.rs:26`）。收据逐命令增加，记忆体 WAL／快照也持续保留，不能继续用“房间状态小”推定写放大可接受。此为成本结构，**没有实测慢或收益百分比结论**。
- **最小修正**：建立大资源包、长局、多房间的提交 p95/p99、写入量和内存基线；将阻塞 I/O 放入有界执行边界，保持逐房保存确认后发布。若基线超标，再按内容锁保存一次不可变包；收据裁剪必须先定义重试窗口／确认水位，不能直接删除旧键。
- **验收**：故障语义、同键去重和信息边界不退化，同时验证负载下的队列上限。`NativeRoomPersist` 的 WAL／历史快照仍在内存；最新局面重启恢复不等于签名录像或跨重启任意跳转，后者归 NC09／NC12 独立验收。

<a id="a8"></a>

### 8. P2：原生 CI 接通之后仍需真实跨目标比较

- **具体情况**：`.github/workflows/ci.yml:65` 已有 native-web 构建与 Worker 回归，不应再称“原生没有 CI”。但 `scripts/cross-target-hash.sh:13` 仍属于旧引擎；`crates/engine-native-wasm/tests/nc08_cross_target.rs:4` 明示 host 端直调包装层，不能替代真实 wasm32 轨迹一致性。现有正式包 Worker 测试与 Rust 测试分别通过，也不能自动证明两者逐步相等。
- **最小调整**：复用现有场景与包，原生执行程序和当次生成的真实 WASM 输入同一命令、选择与存档；比较权威状态、可见事件、选择、随机流与恢复轨迹，加入受控变异负例。GitHub Actions 是否实际运行、跨设备结果分别登记。
- **验收**：原生 CI 中有真实执行目标对照，再按 NC08／NC12 扩正式包组合与平台。不要因此重复搭建已存在的 WASM 构建流水。

## 本轮文件清理

只整理现有文档，不新增第二份架构执行方案。具体增量及入链核对见[清理清单](documentation-changes.json)：

- 架构索引改为实际 native Worker、原生 WS actor 和显式旧线回退关系，并补 NC 方案入口。
- 候选方案撤下已修存档阻断及过期 T/P 挂靠，关联已采纳 NC 目标；历史 S 表保留当时数值，ADR 0782 不重开决策；已有 CI／ADR 检查不重复立项。
- 进度头部只保留当前交付、边界和本轮优先级；10/24 多轮完整摘要移到既有历史页，原文保留。10/19、10/18 和更早区块改用历史语境，保留旧锚点及机器消费者要求的正文。
- 自检指南使用完整链接检查、能保留失败退出码的命令，移除重复基线数字、固定文件数与把新坏链列为例外的建议。

检查了入链与机器消费者：`refresh_recent_rounds.py` 从进度主稿全文生成旧轮次索引，`check_comment_counts.py` 读取主稿缺陷总账，`check_stale_numbers.py` 固定主稿清单。本轮保留这些正文及标记，未迁移 10/19 的轮次块；只移走未被这些规则读取的 10/24 交付摘要。正式 ADR、原始证据、独有历史阶段文档和目标性能架构都保留；没有仅因日期旧而删除整份文件。

## 验证与证据边界

| 本轮检查 | 结果与证据 |
|---|---|
| `python3 -u -B scripts/preflight.py` 完整模式 | 17:16:51–17:22:31 +08:00，20/20 通过、0 失败、0 跳过、exit 0；脚本单测 180/180，源码行数门禁 8576 文件不超 2000 行；[摘要](preflight-full-01.log)、[全部子命令输出](preflight-full-01-raw.log)、[命令与时间](preflight-full-01.json) |
| `python3 -B scripts/check-adr-index.py` | exit 0；既有冲突均有消歧登记，未登记冲突／标题错号／索引坏链为 0；[输出](adr-index.log)、[元数据](adr-index.json) |
| 补写后的全量链接、严格行号、旧值、轮次索引及 diff | 各命令退出码、完整输出见[最终门禁](final-doc-gates.json)及[日志](final-doc-gates.log)；不以摘要或过滤管线替代命令结果 |
| 文档入链、锚点与历史内容 | 5 份改动文件入链核对无新增失效锚点，改动文件出链锚点均可解析；S1–S13 历史表逐字一致，24,273 字节交付摘要完整迁入历史页；[清理清单](documentation-changes.json)、[原文摘要](history-move.json) |
| 既有工作树保护 | 47,064 个基准路径中仅 5 份文档增量变化，其余 47,059 路径保持内容／模式／链接属性；产品受验 1067 文件、3 个生成目录共 18 文件及原 dirty dist 46 文件均不变，HEAD/index 不变；[保护记录](preservation.json) |

完整 preflight 通过未修改的 `preflight.main` 执行，捕获层仅保存子命令原始 stdout/stderr；没有修改脚本、阈值或豁免。现有链接检查只验证路径并保留原有历史例外，严格行号检查不判断引用语义；本轮另核对改动文档的锚点及发现所指的实际源码。

此前最新完整产品回归见[17:00 双重故障证据](../2026-10-24-native-room-double-fault/README.md)，其受验源文件摘要已与本轮工作树核对一致，但不作为本轮重新跑过的测试。备份位于 `/tmp/noname-next-architecture-doc-audit-20261024/`；未暂存、未提交或推送。

WASM 探针主动复现失败而退出码为 0，仅代表探针收集完成，**不是产品回归全绿**。`gg` 对照受控拒绝且旧档不变；合法归档恢复后所有座位投影／revision 保持，存档字节因 `auth_generation + 1` 授权代次轮换而改变，不能以字节完全相等判恢复失败。NativeEngine 没有导出权威 hash，探针不冒称验证该值。

未跑 Rust 全量、前端整套或 Worker 整套回归，未重建 WASM／桌面包，未执行 GitHub Actions、跨平台／跨机／断电或长局性能验收。本轮未修改产品代码，没有将新发现记成已修复，也没有改变全 37 来源范围或整包完成口径。
