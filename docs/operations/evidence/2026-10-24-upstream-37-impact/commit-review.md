# 37 提交核对快照（2026-10-24）

最终待办以[规则差异主账](../../../architecture/native-rules-baseline.md#upstream-2026-10-24)为准。以下每行覆盖一个真实提交；多次修改同一技能已合并，不把“本体已导入”“ported”“本轮已实现”混用。模式大改已完成范围和接口影响分类，动态语义等价仍未验证。

| # | 提交 | 分类 / 差异编号 | 核对结论 |
|---|---|---|---|
| 1 | [`1b10052d`](https://github.com/libnoname/noname/commit/1b10052dbef4bca7acbb30af6a770172d4d58a8a) feat: 新武将TW魔张梁 (#4501) | 新内容；UP37-N03 | 魔张梁：新增角色和人方/绝灭；既有技能依赖另验 |
| 2 | [`aab18ef2`](https://github.com/libnoname/noname/commit/aab18ef2632fbbb2ddef81fc70c2dbb8b67e8b4e) fix: 修复星董允可能报错的部分 (#4502) | 规则/取消；UP37-R04 | 弼士取消给牌后 cards 可为空 |
| 3 | [`c2c9ae0b`](https://github.com/libnoname/noname/commit/c2c9ae0be72e4284416f122a220f9c6081ddd768) fix: 修复冯方女报错 (#4503) | 规则/显示；UP37-R05, UP37-R10 | 涕泣空历史求和；TW扶蜀显示实际伤害层数 |
| 4 | [`143b746f`](https://github.com/libnoname/noname/commit/143b746f108e8ccea8b9c8a1afcf5f07e76aef48) fix: 谋卫瓘报错问题修复 (#4505) | 选择续行；UP37-R08 | 清势保留事件返回值；造疾取消结果防护 |
| 5 | [`217edc4b`](https://github.com/libnoname/noname/commit/217edc4bd4df4b64d7693966c88555976bc17233) feat: 新武将OL许劭 | 新内容；UP37-N07 | OL许劭初版；最终行为按102ef4fa并入，不单独复刻 |
| 6 | [`2dba987d`](https://github.com/libnoname/noname/commit/2dba987ddd8e7bfba687b7a3d9cfd4a61310d91d) feat: 旧武将OL赤兔 | 新内容/共享接口；UP37-N05, UP37-N07 | OL赤兔启用；强制手牌排序参数；许劭托管切换 |
| 7 | [`a5ed6a70`](https://github.com/libnoname/noname/commit/a5ed6a70f0c5af3204b154434380979d4976bc4a) fix: 删除自动填充的棍母引用 | 工程清理；UP37-N05 | 删除误引入的 Vue TriggerOpTypes，不新增规则 |
| 8 | [`83539838`](https://github.com/libnoname/noname/commit/83539838c20e4cc0cdd94b196e94e6a2c12f3454) feat: 旧武将乐孟获 | 新内容；UP37-N06 | OL乐孟获启用，五菇丰登/菌子与幻视 |
| 9 | [`f1f17061`](https://github.com/libnoname/noname/commit/f1f1706100498893ab3c8ddb6c135cb1eb8bcb99) fix: 孩子们我也吃菌子了 | 规则修正；UP37-N06, UP37-N07 | 菌子候选技能非空判断修正；许劭技能筛选/授予/使用/撤销改写 |
| 10 | [`b31df527`](https://github.com/libnoname/noname/commit/b31df5276694ec864d7d3dcf16a97a0b271bde4e) feat: 两个周瑜及星董允素材补充 (#4510) | 资源；UP37-I05 | 周瑜/星董允画像语音；TW两技补audio |
| 11 | [`19d86519`](https://github.com/libnoname/noname/commit/19d865193056ec3285dfbac63cdc1695acbfa71c) fix: 补充新杀诸葛均names与clans、威关银屏结算调整、tw包所有preconetnt异步化 (#4506) | 规则/异步/元数据；UP37-R01, UP37-R05, UP37-R08, UP37-R10, UP37-I03 | 涤仇、宝梳、神赋、诸葛均宗族；TW precontent及勇进显示顺序 |
| 12 | [`4f195a50`](https://github.com/libnoname/noname/commit/4f195a50776e7d3242eb2813402f0633247b9e82) fix: 修复神孙策【冯河】&乐诸葛亮零元购的问题 (#4517) | 权威规则；UP37-R02, UP37-R03 | 讲武主机重新扣款与离线兜底；冯河检查被封英霸仍拥有 |
| 13 | [`2492902c`](https://github.com/libnoname/noname/commit/2492902c41a1b10935cb8c019e8c3a68cf662791) fix: 衰曹节王甫纵害修复、史阿鸡哥修复 (#4513) | 规则/AI/资源；UP37-R06, UP37-R07, UP37-R08, UP37-R10, UP37-I05 | 纵害、明节、击戈、举战、自固、荡尘、争功；缮甲确认和元数据 |
| 14 | [`0d1ab937`](https://github.com/libnoname/noname/commit/0d1ab937b88bc3d87db46743670cbdd764cb06f7) fix: 修复OL谋周瑜的报错bug (#4515) | 边界；UP37-R08 | 顾曲无currentPhase不崩溃 |
| 15 | [`673bc921`](https://github.com/libnoname/noname/commit/673bc921291d622a36c563f0850748ae89e2e10f) fix: 修复一些描述bug (#4519) | 文案；UP37-R10 | 执盟半角X；柔克失效期限文字修正 |
| 16 | [`98d90ce9`](https://github.com/libnoname/noname/commit/98d90ce99fb580870f49a8d834e3342d3b89da99) docs(changelog): 汇总 7fcf23ed..673bc921 的变更 | 历史摘要；UP37-I04 | changelog首段范围早于本次起点，不能把其全部项目重复算入37提交 |
| 17 | [`db773dad`](https://github.com/libnoname/noname/commit/db773dad585bdb09134a701c2d1d0eeb5662312e) perf(mode): 重写乱斗模式下的所有Step Content (#4500) | 模式迁移；UP37-I02 | 乱斗Step到Async，构建individual入口；未做全模式轨迹等价 |
| 18 | [`c8d29532`](https://github.com/libnoname/noname/commit/c8d2953235d940de1941219b53543470c93fc3ef) feat(character): OL界朱然 and OL界朱桓 (#4363) | 新内容；UP37-N01 | OL界朱然和OL界朱桓，共3顶层技能 |
| 19 | [`7c3cc590`](https://github.com/libnoname/noname/commit/7c3cc590ea6ab17bdbdaa3f11bda4cf41d80da73) feat(character): add OL孙寒华 (#4378) | 新内容/资源；UP37-N04, UP37-I05 | OL孙寒华、彩莲和三形态 |
| 20 | [`08b0f659`](https://github.com/libnoname/noname/commit/08b0f659a05ecbd3b74f31e9e3df4e5b199377e4) fix: 修复星董允bug (#4522) | 规则；UP37-R04 | 弼士拥有者绑定、第一张牌检查实际trigger |
| 21 | [`4195e67b`](https://github.com/libnoname/noname/commit/4195e67ba408ff38e94f66eccd5a7ee00e54fb83) perf(mode): 重写挑战模式下的所有Step Content (#4527) | 模式迁移；UP37-I02 | 挑战模式Step到Async；影响Boss来源依赖，未做全模式轨迹等价 |
| 22 | [`c30c312c`](https://github.com/libnoname/noname/commit/c30c312c8514763bd6e48675bc8b55911b0dda31) docs: 让GPT优化给GPT读的迁移代码 | 文档；UP37-I04 | 上游事件对象/Step迁移指南，不作为Rust实现规范 |
| 23 | [`bd563595`](https://github.com/libnoname/noname/commit/bd563595e53057f7ee641710699eb45376ba7b5c) docs: 补充 Step 转 Async 的迁移目标选择说明 | 文档；UP37-I04 | 补充Step迁移目标，不改变游戏规则 |
| 24 | [`480980d2`](https://github.com/libnoname/noname/commit/480980d23894082c5ae55e75e1f0cc926663d8f1) perf(mode): 重写联机模式的Step Content | 模式迁移；UP37-I02 | 联机模式Step到Async；不是noname-next WS协议改动 |
| 25 | [`60bca279`](https://github.com/libnoname/noname/commit/60bca279f119bf5a2a1e745e9aa67139753e4743) perf(mode): 将神秘的realtime模式改成新格式 | 导出格式；UP37-I02 | realtime从函数返回对象改为对象，空start变async |
| 26 | [`3e72c4b7`](https://github.com/libnoname/noname/commit/3e72c4b7980050c3b7d58adc53d232b64bb00791) perf(mode): 重写炉石模式下的所有Step Content (#4530) | 模式/类型声明；UP37-I02 | 炉石Step到Async；Card/Skill事件类型接受ContentFuncsByAll |
| 27 | [`3a24b5a5`](https://github.com/libnoname/noname/commit/3a24b5a58eccb7412816988069e755c57dafc3e6) perf(mode): 重写对决模式下的所有Step Content (#4533) | 模式/共享接口；UP37-I02 | 对决Step到Async；chooseCharacterDouble补return next；新增构建入口 |
| 28 | [`b17a6198`](https://github.com/libnoname/noname/commit/b17a619841f9162b60bc50ffa88a8962c1618b63) fix: 将璀璨星河包中仍未转换成 Async Content 的技能重构为 Async Content (#4534) | 技能迁移；UP37-I03 | SP显式事件参数/对象形式调用；逐技能等价仍待核验 |
| 29 | [`83931d24`](https://github.com/libnoname/noname/commit/83931d24edd8bed70799feab890a0947ce10e4b3) fix: 将卡牌包残留的 Step Content 转换成 Async Content | 卡牌迁移；UP37-I03 | 国战/仙侠setContent回调改async |
| 30 | [`076d1c77`](https://github.com/libnoname/noname/commit/076d1c7736b4dfe98c06fbf3de88f3974e743460) fix: 去除本体残留的 Step Content | 剩余本体迁移；UP37-I03 | 角色/单挑/国战/炉石及进入游戏/投降；兴衰伤害循环、疠火event.card |
| 31 | [`ad1065b0`](https://github.com/libnoname/noname/commit/ad1065b0873ce6526e36c1776cce96d76bb7253c) style: 格式化 noname 全部文件并添加 CI 检查 (#4536) | 格式/CI；UP37-I04 | 35份JS/TS归一比较；模板/空语句人工复核；上游lint不直接移植 |
| 32 | [`a15a1e4f`](https://github.com/libnoname/noname/commit/a15a1e4f0433cc4d46421530bfe93b7873f97dbd) style: 移除残留的死注释代码 | 死注释；UP37-I04 | 5份JS归一AST相同，含standard.js；文件字节变化不能说无改动 |
| 33 | [`3ffbe10e`](https://github.com/libnoname/noname/commit/3ffbe10e228bf5010ab90b7c6f5b36440e97f3f9) feat(character): 添加唐棠 (#4398) | 新内容；UP37-N02, UP37-I05 | 唐棠，择行/知意与画像 |
| 34 | [`90b58bb9`](https://github.com/libnoname/noname/commit/90b58bb9934edb1712fd187a80c8323b7aeabaf3) fix: 优化部分技能ai+修复部分技能异常问题 (#4542) | 规则/AI；UP37-R07, UP37-R09, UP37-R10 | 摧锋虚拟牌、荡尘收牌者/空给牌、共患AI、华裳AI类型初始化 |
| 35 | [`102ef4fa`](https://github.com/libnoname/noname/commit/102ef4fa382b0a36cc4f01b949ba0363689efee9) feat(character): 重写OL许劭评鉴连线玩法 (#4555) | 新内容最终形态；UP37-N07, UP37-I05 | OL许劭评鉴UI/联网玩法重写，15项界面资源和CSS |
| 36 | [`a44934c5`](https://github.com/libnoname/noname/commit/a44934c57ad05879d06e54e87e25da7a4ae5c67c) fix: 优化部分技能ai+修复部分技能效果异常 (#4552) | 规则/AI；UP37-R05, UP37-R09, UP37-R10 | 避祸下回合状态；相谶无实体牌；讨州授予自矜；其余AI和文案 |
| 37 | [`1a548c95`](https://github.com/libnoname/noname/commit/1a548c95acdf8ac1ef44d74473403eda2ac86cf0) docs(changelog): 汇总 db773dad..a44934c5 的变更 | 历史摘要；UP37-I04 | 本批后半changelog；不另计新规则 |
