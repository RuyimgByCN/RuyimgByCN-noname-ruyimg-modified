# 34e 正常发布与 CI41214274772：检查点，非最终结果

核验2026-10-31；accepted commit34e83f0550bb70ef4af6083697428de7be2f02f8，tree22e6eab72d104baf7a658d5ff2d832128ac32e4a、parent d07b35f9。源I02_30＋CI_1共31路径保持，3433变更文件／105988114B逐blob与mode核对。撤回的bdaf不在34e祖先链，原d07及已发布历史保留；bdaf仍由本地保护引用持有，495误跟踪profile／schema备份仅取消跟踪，552排除物全保留本地。

15:19:09.395399Z实际push exit0，15:19:09.396745Z明确释放只覆盖推送的Main只读租约。固定URL／native-convergence正常全历史提交，guard正负8控制（含拒绝bdaf新SHA）通过，原pre-push hook e349af1e…不变，无force／no-verify／持久hooksPath／remote设置变更。见[原push结果](original/push-stage/result.json)、[guard与自测](original/push-stage/scope.json)、[租约释放](original/push-stage/lease-release.json)及[复制清单](push-copy-manifest.json)。

父fresh核36推送文件／所有gzip、before-immediate-after raw index／逻辑项／flags／untracked字节相同，以及552排除项和真实祖先关系；15:43独立live远端读确认native-convergence=34e、main=47347c66，见[父核验](parent/noname-parent-34e-publication-push-audit.json)。只声明该推送窗口稳定，不归因更早的index变化。随后本地父文档／证据更新另有收据，不能算在原窗口或源34eCI输入中。

Git输出Writing2082objects、Resolving1476deltas、25.88MiB是打印舍入量，不是精确网络字节；preflight可达新对象1947／78883946未压缩字节是另一口径。push含队列／启动wall39.274s，不把差值称纯排队时间。

新[CI41214274772](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41214274772)绑定34e，15:33:03检查点为7success／3running／conclusion null，见[不可变检查点](checkpoint-envelope.json)。两个Windows checkout step2实际success：build124575425342于15:20:10结束，desktop124575425298于15:20:27结束；普通Windows build整job已success。desktop Windows/macOS与gates在该检查点仍运行，**不宣最终CI／桌面安装器全绿**。旧d07 run41202703042的8success／2checkout失败保持，不rerun或挪作本次结果。

本目录只保全当时已封存push／checkpoint API文件；live watch／未完成日志不混入该映射。随后新run实际完成10/10及父fresh核验／独立复审通过，十job完整原日志、警告／忽略／skip和三安装器API元数据另见[最终登记](../publication-ci/README.md)。这里的七绿三运行检查点不重写，元数据或job绿都不代表实际设备安装／整局／功耗通过。当前状态见[进度主稿](../../../../architecture/native-progress.md)。
