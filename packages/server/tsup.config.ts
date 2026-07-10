import { defineConfig } from 'tsup';
import { readFileSync, writeFileSync } from 'node:fs';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['esm', 'cjs'],
  // 关闭 DTS：服务端运行不需要 .d.ts，且 tsc 不支持项目里 .ts 扩展名的 import
  dts: false,
  splitting: false,
  sourcemap: true,
  clean: true,
  // ws/bcryptjs 作为外部依赖（从 node_modules 加载，不 inline）
  external: ['ws', 'bcryptjs'],
  // esbuild 会把 node:sqlite（实验性内置）错误改写成 sqlite，build 后修正回来
  async onSuccess() {
    for (const f of ['dist/index.js', 'dist/index.cjs']) {
      const src = readFileSync(f, 'utf-8');
      const fixed = src.replace(/from "sqlite"/g, 'from "node:sqlite"').replace(/require\("sqlite"\)/g, 'require("node:sqlite")');
      writeFileSync(f, fixed);
    }
    console.log('✓ node:sqlite 前缀修正完成');
  },
});
