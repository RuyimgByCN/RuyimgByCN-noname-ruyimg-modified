# NC03：本地终局编号与观察者胜负展示（2026-10-30）

整合基准为 NC09 投影提交 `f27dd34f`；[干净起点与父线程审查](integration-start.json)。本批仅改 TS/Vue 与对应测试，Rust胜者协议继续1基，联机投影不变，规则328/存档105/ABI753/WS5不变。

## 根因与修复

本地座位0基，而WASM胜者输出沿用1基协议；适配层原样传递导致实际座位3显示P4。`outcomeText` 又把有效的0当成缺失，BattleView则重复映射结果、按阵营硬编码自己获胜/败北，并漏掉单挑/势力横幅。

修复在本地适配边界只规范化SingleWin玩家编号，保持输入原对象与网络投影。共享展示函数负责结果文字和三态本人胜负，优先使用权威view、缺失时才兼容旧MatchEnded事件；身份胜负依据观察者自己的Lord/Loyalist/Rebel/Renegade身份。势力或未知结果缺少足够本人归属信息时保持中性，不猜测未建模的Versus/Boss语义。状态栏、横幅和事件记录复用同一展示入口。

## 回归与整合

[实施报告](implementation-report.txt)与[隔离红绿证据](isolated/)保留原文。修改前30项定向用例22通过/8失败，覆盖编号、0值、阵营硬编码、遗漏横幅及重复映射。隔离修后42项定向、138项全前端、typecheck/build通过。

主树叠加NC09后[实际命令与结果](integrated-results.json)：前端 **139/0**、typecheck、生产build、行数脚本2/0及总行/有效行门禁通过；完整preflight **21/21** 通过，受验源码指纹不变。Rust/WASM源码未变，复用的是f27dd34f已从当前源码构建的实际WASM产物，没有冒称本批又重建Rust。

测试执行实际SFC中的computed与通过Vue SSR编译的真实横幅模板；其后父线程另用真实Chrome、Worker及WASM经正常界面跑局，未mock引擎、未直接改权威状态。

## 真实浏览器终局

[主驱动](browser-native-product.mjs)、[两模式结果](native-product-report.json)与[截图](native-product.png)：原型seed42/4人319步，胜者已正确为本地P3，观察者1显示败北；身份seed42/4人280步反贼胜，非反贼观察者显示败北。两局均从界面存档、推进、读档，恢复后hash/revision相等，再真实推进至终局，检查实际横幅文字、win/lose类和图标；全程私有手牌/牌堆负检、console异常0。

另从实际WASM做有界seed搜索，得到身份seed3、观察者1为Rebel的获胜正例；[实际正常开局驱动](browser-observer-win.mjs)经主页种子输入开局，168步得到反贼胜，真实横幅为🎉、win=true、lose=false。[命令](integrated-browser-observer-win-result.json)、[报告](positive-observer/native-product-report.json)、[截图](positive-observer/native-product.png)。这是功能验收，显式跳过动画，不是帧延迟或功耗测量。

搜索辅助脚本最初错误假设两个模式的前32个seed都包含观察者胜局，第二次又错误假设命中的是原型；两次辅助断言失败分别保留在[first failure](find-observer-win-first-failure.json)和[第二次日志](find-observer-win-second-assumption-failure.log)。最终保留所有实际[搜索记录](observer-win-seeds.json)，只使用找到的身份正例；没有把未找到原型正例伪造为浏览器通过，也未将测试夹具搜索假设失败算成产品回归。

## 边界与保护

本批不代表NC03所有UI/API、所有模式或平台完成，未建模胜者仍明确显示未建模。生产构建覆盖的已跟踪dist须先保存再恢复，保持本地修复提交不夹带发布产物。之前已完成九批的私有构建缓存因磁盘余量不足而回收，源码/提交/原始日志/冻结快照保留，[保护记录](completed-nine-private-cache-retirement.json)；未清理共享target。
