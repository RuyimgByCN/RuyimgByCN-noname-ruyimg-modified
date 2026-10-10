# ADR 0542：diy characters.json 技能引用修复

- 日期：2026-08-03
- 领取者：E 段 Hermes
- 涉及包：diy（ADR 0512，omp 交付）

## 背景

全包一致性终验（npack skill_index vs character_index 交叉核对）发现 diy 包
characters.json 中 **61 名角色引用 84 个不存在的技能 def_id**（全库检索均无）。
与 xianding（ADR 0540）同性质：内容编译器 pack 时不校验 character→skill 引用，
缺陷静默进入已发布 npack。

此前的"元数据登记设计"判断仅适用于空技能数组角色；经复核，diy 中 6 名
README 声称"完整武将（真实运行时）"（ns_zhangwei/ns_sunyi/ns_zuoci/ns_yanliang/
ns_sunchensunjun/ns_huangwudie）也引用失效 id——确认是缺陷而非设计。

## 修复

1. `content/original/ported/diy/characters.json`：61 角色 skills 按上游
   character.js 技能名顺序重建：
   - 本包 .skill 定义（含 diy_meta.skill 占位 id 3934300000+）
   - 跨包技能（lvli/choujue/shibei/gongji 等）引用目标包既有定义 id
   - `dualside`（双面武将标记，非技能）跳过；空技能名跳过
2. `content/original/ported/diy/metadata.json`：deps 扩充至 8 包
   （standard/refresh/sp2 → +onlyOL/shenhua/sp/xianding/yijiang）
3. 重打包（180 技能 / 66 武将不变，ABI 478 不变）
4. `extensions/diy/{diy,package}.npack.json` 同步（md5 三份一致）

## 验收

- npack character_index 全部技能引用可解析（未解析 = 0，原 84）
- skill_index 180、character_index 66 与修复前一致
- 完整武将引用正确：ns_yanliang = [nsduijue, nsshuangxiong]（dualside 剔除）
- conformance 8/8 通过
