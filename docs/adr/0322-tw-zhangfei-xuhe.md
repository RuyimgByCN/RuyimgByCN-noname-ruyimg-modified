# ADR 0322：TW 张飞（虚吓）

本批为 TW 包张飞（`tw_zhangfei`）的「虚吓」（`twxuhe`，3586311917）完整移植，
复用既有 `sha_dodged` 时机、`deal_damage` 原语、`add_skill` 临时技能与
`damage_taken_amount` 修饰符，不新增 Rule VM ABI。TW 张飞的另一技能
「咆哮」（`new_repaoxiao`，2733157850）属 REFRESH 包且已移植，跨包引用。

## 技能移植

「虚吓」（`twxuhe`，3586311917）：
- 触发：`sha_dodged owner_source_sha_dodged`（自己使用的【杀】被闪避后）。
- 分支选择 `choose(0, 1)`：
  - 0：`deal_damage(self, event_player, 1)` —— 使用者对闪避目标造成 1 点伤害。
  - 1：`add_skill(event_player, 3796539642)` —— 目标获得临时技能「虚吓·威吓」。
- 「虚吓·威吓」（`twxuhe_damage`，3796539642，新派生技能定义）：
  - `modifier damage_taken_amount add 2` —— 持有者（闪避目标）受伤 +2。
  - `trigger turn_ended owner_is_target { remove_skill(self_skill) }` —— 其回合结束时移除。

## 文档化近似

- 旧 JS 由闪避目标 `trigger.target.chooseControl` 选择分支；DSL 的 `choose` 选择者
  固定为技能拥有者，选择权近似由拥有者行使。
- 旧 JS 仅对使用者下一张牌对目标造成的伤害 +2，并以该牌使用/结算为消解条件；
  DSL 以 `damage_taken_amount` 修饰符（任意来源的受伤 +2）与目标回合结束移除近似。
  两者均记录于 `.skill` 文件注释。

## 打包与验证

- `twxuhe.skill` 与 `twxuhe_damage` 加入 `content/original/ported/tw/`；
  `tw_zhangfei`（3775361373，男/蜀/4 体力，技能 [2733157850, 3586311917]）加入
  tw `characters.json` 选将池；catalog 中 `twxuhe` 标记为 `ported`，
  新增派生技能 `twxuhe_damage`（3796539642）。
- tw npack 重建为 78 模块（44 技能文件，含 twxuhe），README 构建命令可逐字节复现
  （`cmp` 通过），已部署至 `extensions/tw/`（`package.npack.json` 与 `tw.npack.json`）。
- 26 个扩展包全部经 `import-extensions --allow-unsigned`（全新 store）导入成功。
- 旧 JavaScript 不进入 npack 或运行时。
