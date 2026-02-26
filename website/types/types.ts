export type Lang = string

export type LangEav = {
  data: Lang | null
  action: ((lang: Lang) => void) | null
  error: { code: number; message: string } | null
}
