# PA09显式桌面原型整合（2026-10-30）

源码整合基准 `100489f5` 加本批工作树；规则328/存档105/原生ABI753/WS5保持不变。本批交付显式启用的Tauri原生引擎原型，**WASM仍为默认**，PA09六项切换条件尚未全部满足。当前主树功能验收已完成，结果及失败尝试分开登记；不将隔离树结果当作主树重跑结果。

## 实现与接线

共享纯Rust [NativeSession](../../../../crates/engine-native-wasm/src/session.rs)持有同一个Engine及规划预算，WASM薄壳保持原25个公开绑定签名与根helper导出。Tauri每个EngineClient使用独立ResourceId和单写执行线程，Rust入口队列/资源数各限制8；前端Promise积压没有另设数量上限，不把它称为整个管线有界。关闭允许已开始命令完成、拒绝未开始命令，丢失回复明确报告结果可能未知，不自动重试或切换引擎。

原型独立审查发现整页刷新遗留校验会话，修复后由原生文档替换、renderer终止及窗口销毁路径只退休NativeEngineHost，保留无关Tauri资源。真实旧二进制第9次分配失败的红例、13文档/12次刷新且每文档可分配剩余7槽的绿例、无关Image保留、120个未决JS操作beacon，以及确定性的执行中/队列中Rust回归均保留。见[首次独立审查](independent-review.raw.txt)与[修复复审](review-fix-independent-review.raw.txt)。这些是原隔离树证据，主树当前运行结果另列。

主树接线保留NC09的characterDef，将既有NC03内联胜者转换换成同一[normalizeNativeOutcome](../../../../apps/web-client/src/engine/nativeOutcome.ts)，Worker及IPC分别仅在本地接收边界转换一次，网络/Rust协议仍1基。保留主树较完整的网络终局测试，没有把它替换成重复的旧分支测试。Referee新增ChoiceView字段的模式匹配适配放在实际session内。共享session读档原子性、包验证和原内容锁继续共用原实现。

父整合另以真实8座seed45000护驾窗复现自动应答崩溃：options为空、min0、不可取消时读取options[0]。保留[红例](parent-empty-hujia-red.log)，修复只按实际min/max/选项数量取值，空合法应答不再越界；[全桌面/WASM回归](parent-desktop-wasm-empty-hujia-green.log)通过。该测试同时比较NativeSession与保留WASM绑定的权威存档。

## 验证与证据

- 主树前端 **146/0**、TypeScript通过，见[parent-command-results.json](parent-command-results.json)。
- 主树六包（原生三包、WASM、server、desktop含Tauri）完整 **2143通过/0失败/74忽略，exit0**，见[原始日志](validation-integrated-six-packages.log)和[命令结果](validation-command-results.json)。其中手动规划测量仍忽略，不能算性能通过。
- 首次父工具60秒等待中断的六包输出保留为[parent-integrated-six-packages.interrupted-60s.log](parent-integrated-six-packages.interrupted-60s.log)，没有将未完成输出算为通过；完整109.47秒受锁命令另跑。
- 首次workspace遇ENOSPC，原始失败保留[validation-workspace.failed-enospc-retained.log](validation-workspace.failed-enospc-retained.log)。恢复后完整workspace **6782/0/75，exit0**，原生三包严格警告门禁、分层0与当前源码WASM重建均通过，见[恢复后命令结果](recovery-command-results.json)及[workspace原始日志](recovery-workspace.log)。实际Worker **82/0**、前端 **146/0**/typecheck/build、conformance 19向量、native/WASM及旧线cross-target均通过。五窗实际WASM32 **75状态比较/270座位检查/35命令/40字节重启**，普通同档 **40比较/32隐私检查**，失败0。真实WKWebView原生IPC与WASM Worker身份局280步逐步一致、跨恢复和VM包v1/v2安装至卸载生命周期通过；12次导航/13文档验证旧资源拒绝及无关Image保留。空护驾窗再经实际WASM32执行，完整reply/archive/16次前后座位choice/hash/revision/events与原生空应答一致。debug与production release桌面二进制已编译并保留SHA。最终完整preflight **21/21**。详见[验证交接原文](main-acceptance-handoff.raw.txt)、[命令索引](recovery-command-results.json)、[产品报告](validation-pa09-desktop-webview-recovery-report.json)及[护驾WASM报告](recovery-empty-hujia-wasm-report.json)。一次误用未注册nc09测试target的失败保留，改用已注册misc后18测全通过；未回填旧失败为绿。

隔离原型及修复的日志/patch/manifest分别保留在isolated/、review-fix/；源基准及无关修改检查见[parent-start.json](parent-start.json)、[源hash](parent-baseline-source-hashes.json)、[差异检查](parent-unexpected-source-check.json)。原型27个源文件hash在父接入前逐一验证。原测试和构建中的WASM宿主unused helper等警告如实保留，不宣称整个workspace零警告。

磁盘故障时父核对所有相关编译进程已终止，只回收明确任务私有的debug/incremental缓存，保留源码、日志、补丁、依赖对象、二进制、WASM及所有共享target；记录见[回收1](enospc-private-incremental-retirement.json)、[回收2](enospc-j2-private-incremental-retirement.json)。后续功能验证显式CARGO_INCREMENTAL=0，不用于性能前后比较。所有重命令仍经全局序列化锁。

共享规则源码在验证期间保持冻结；最终[维护源码hash](recovery-maintained-source-hashes-final.json)、[冻结差异](recovery-source-difference.json)及[解冻边界](recovery-thaw-boundary.json)保留。37条受跟踪dist文件已在保留当前产物后逐字恢复HEAD，不把生成目录变化混入源码提交。编译后清理和后续仍需保留产物的理由见[逐次记录](recovery-post-compile-cleanup.json)；任务已结束，私有target可由父协调回收。构建产物SHA见[产物索引](recovery-artifact-hashes.json)，二进制/压缩产物作为本地验收产物保留，文本日志、清单及复现代码入库。

## 尚未通过的切换门槛

受控同负载真实HID输入到绘制与IPC/Worker传输对照未完成；正向线程池收益未证明；独立服务器进程完整轨迹、整合树产物与平台验证仍按具体结果验收。隔离原型的280步IPC/Worker完整信封和跨恢复、VM包v1/v2全生命周期是对应样本的真实证据，不替代受控性能或所有包验收。Windows/Linux未实测，真机未连接，GitHub Actions未执行；只读检查发现候选远端只有旧线工作流且不存在noname-next/Cargo.toml，不能对该仓执行旧工作流后冒称本次原生CI通过，见[远端检查](github-candidate-readonly-check.json)。

已准备复用原采样层的[系统输入驱动生成器](prepare-hid-driver.py)、[生成驱动](hid-web-frame-external.mjs)及[Swift输入控制器](hid-tool.swift)。坐标来自真实DOM rect与AX窗口位置/大小，四项纯坐标自检和Node/Swift语法检查通过，见[准备核验](hid-v2-preparation-checks.json)；不截图、不请求权限，焦点/窗口几何变化即停止，必须证明实际trusted点击与每个请求对应才能接受样本。**尚未编译/运行输入控制器，未产生HID性能结果。** [独立release测试配置](hid-release-config.json)仅把测试窗口定位至本机采样HTTP源并限定同源core权限，生产配置未改；未来使用时须记录该fixture构建差异，不将其冒称未经修改的生产二进制。需要待并行构建结束后的受控静默窗口，再完成实际profile/输入/传输的同负载验证。

本批不关闭PA09、NC03/NC09、NC10或37来源整包。裁判授权投影及五窗口恢复见[独立整合批](../2026-10-30-nc09-referee/README.md)，已批准元数据/超时330/选序331的授权见[裁决20–22](../2026-10-28-user-decisions/README.md)，没有提前占用J2的329。
