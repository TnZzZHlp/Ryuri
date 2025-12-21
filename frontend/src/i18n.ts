import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zh from './locales/zh.json'

type MessageSchema = typeof en

const savedLanguage = localStorage.getItem('language')
let initialLocale: 'en' | 'zh'

if (savedLanguage && savedLanguage !== 'auto') {
  initialLocale = savedLanguage as 'en' | 'zh'
} else {
  initialLocale = navigator.language.startsWith('zh') ? 'zh' : 'en'
}

const i18n = createI18n<[MessageSchema], 'en' | 'zh'>({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'en',
  messages: {
    en,
    zh
  }
})

export default i18n
