/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

import '@tiptap/core';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    track: {
      setTrack: () => ReturnType;
      toggleTrack: () => ReturnType;
      unsetTrack: () => ReturnType;
    }
  }
}
