type BundledExtensionCtor = new (
  url: string,
  name: string,
  productName?: string,
  active?: boolean,
  description?: string,
  version?: string
) => import('@aliv/core').BaseExtension

declare module '@aliv/assistant-extension' {
  const ext: BundledExtensionCtor
  export default ext
}
declare module '@aliv/download-extension' {
  const ext: BundledExtensionCtor
  export default ext
}
declare module '@aliv/llamacpp-extension' {
  const ext: BundledExtensionCtor
  export default ext
}
declare module '@aliv/mlx-extension' {
  const ext: BundledExtensionCtor
  export default ext
}
declare module '@aliv/rag-extension' {
  const ext: BundledExtensionCtor
  export default ext
}
declare module '@aliv/vector-db-extension' {
  const ext: BundledExtensionCtor
  export default ext
}
declare module '@aliv/conversational-extension' {
  const ext: BundledExtensionCtor
  export default ext
}
