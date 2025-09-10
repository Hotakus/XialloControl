/**
 * 通用翻译函数 (占位符)
 * TODO: 集成 i18n 库后在此处实现真正的翻译逻辑
 * @param key 翻译的 key, e.g., "A" or "buttons.a_button"
 * @returns 翻译后的字符串
 */
export function translate(key: string): string {
    // 当前只是一个占位符，直接返回 key
    // 未来会替换为 i18n.global.t(key) 之类的实现
    return key;
}