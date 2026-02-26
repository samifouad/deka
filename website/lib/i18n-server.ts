import { cookies } from 'next/headers'
import { languages } from '@/i18n'

const DEFAULT_LANG = languages[0]?.code ?? 'en'

export async function getRequestLang() {
  const cookieStore = await cookies()
  const cookieLang = cookieStore.get('deka-language')?.value
  const isValid = languages.some((lang) => lang.code === cookieLang)
  return isValid ? cookieLang! : DEFAULT_LANG
}
