/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** native（默认）| legacy | wasm | old */
  readonly VITE_ENGINE_BACKEND?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<object, object, unknown>;
  export default component;
}
