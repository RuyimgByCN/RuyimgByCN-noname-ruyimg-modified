# d07 发布与 CI41202703042：失败如实登记

核验2026-10-31，accepted commit d07b35f94a42b282222dda4ab7bbc5c39ab8fddb、tree7cfa8e085e94e760999e14b90861857f8c9962bb。裁决25限定URL／native-convergence的正常全历史push真实exit0；正负guard八控制、原hook未变、远端main47347c66未变。见[发布原报告](publication.raw.txt)、[完整包映射](packet-manifest.json)及[独立登记复审](registration-review.raw.txt)。

[原run](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41202703042) attempt1最终8success／2failure，126steps=110success／2failure／14skip／0null。两个Windows job124541504027／124541504124都在actions/checkout step2报Git128、Filename too long；各四文件与一目录过长。Windows编译／测试／安装器未执行，不算测试绿，也不是断言红。一次watch因真实失败exit1；33 API读取exit0，无rerun／dispatch／第二watch。旧5fc ten-green只属于旧提交。

其余实际结果含workspace6922/0/77、native-web1830/0/75、macOS／Ubuntu各lib853/0/0及前端179／Worker82／resources17／conformance19；计数重叠。十日志原字节4245871、3166 warning行含重复；安装器元数据不等同安装／真机整局。父fresh101 SHA／CRC、十gzip和五次独立live API核对通过；原解析push_attempts类型错误及修正保留。

最小修正只给两个含Windows的checkout action设置局部GIT_CONFIG_COUNT（Windows1，其余0）、GIT_CONFIG_KEY_0=core.longpaths、GIT_CONFIG_VALUE_0=true。原workflow354f1231…→b46fd58a…；本地Git优先级正负控制／Ruby YAML与独立源码复审通过，无持久Git配置变化。14:07:43在I02明确解除冻结后由父精确应用，见I02证据的parent receipt。它不属于之前的冻结preflight；标准JS action子进程继承环境为所依赖行为，真正Windows checkout／构建结果仍须下一正常提交CI验证。

当前结果和后续待办以[进度主稿](../../../../architecture/native-progress.md)为准。
