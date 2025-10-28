/// <reference types="vite/client" />

declare module "*.md" {
  // "unknown" would be more detailed depends on how you structure frontmatter
  const attributes: Record<string, unknown>;

  import React from "react";
  const ReactComponent: React.VFC;

  export { attributes, ReactComponent };
}
