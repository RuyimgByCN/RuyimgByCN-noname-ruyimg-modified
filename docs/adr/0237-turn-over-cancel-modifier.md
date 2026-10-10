# ADR 0237：双向翻面取消修正器

## 决策

ABI 164 增加 `turn_over_cancel` modifier。`override 1` 时，拥有者从正面翻至背面或从背面翻至
正面都会取消。既有 `turn_face_down_cancel` 保持只阻止正面翻至背面，避免改变已发布技能。

该能力直接接入权威 `turn_over` 入口，不增加效果请求、HOST_EFFECT、continuation 或包字段。
Engine semantics 升至 170；发布前必须通过 Native/WASM 跨目标哈希门禁。
