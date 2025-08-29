declare module 'esbuild-plugin-path-alias' {
  export default function pathAlias(options: { alias: Record<string, string> }): any;
}