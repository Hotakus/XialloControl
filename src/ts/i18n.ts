import { createI18n } from 'vue-i18n';
import en_US from '../locales/en_US.json';
import zh_CN from '../locales/zh_CN.json';

const i18n = createI18n({
    legacy: false, // 使用 Composition API 模式
    locale: 'zh_CN', // 默认语言
    fallbackLocale: 'en_US', // 回退语言
    missingWarn: false, // 禁用找不到翻译的警告
    fallbackWarn: false, // 禁用回退的警告
    missing: (locale, key) => { // 找不到翻译时的处理函数
        return key;
    },
    messages: {
        'en_US': en_US,
        'zh_CN': zh_CN,
    },
});

export default i18n;

/**
 * 通用翻译函数
 * @param key 翻译的 key, e.g., "A" or "buttons.a_button"
 * @returns 翻译后的字符串
 */
export function translate(key: string): string {
    return i18n.global.t(key);
}