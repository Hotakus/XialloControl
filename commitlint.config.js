// commitlint.config.js
export default {
    // 继承常规配置
    extends: ['@commitlint/config-conventional'],
    // 自定义规则
    rules: {
        // 类型必须小写
        'type-case': [2, 'always', 'lower-case'],
        // 类型不能为空
        'type-empty': [2, 'never'],
        // 类型必须是以下之一
        'type-enum': [
            2,
            'always',
            [
                'feat',     // 新功能
                'fix',      // 修复
                'docs',     // 文档
                'style',    // 格式
                'refactor', // 重构
                'perf',     // 性能优化
                'test',     // 测试
                'chore',    // 杂项
                'ci',       // CI/CD
                'build',    // 构建/辅助工具
                'revert'    // 回退
            ]
        ],
        // 主题不能为空
        'subject-empty': [2, 'never'],
        // 主题必须以中文或英文开头
        'subject-case': [0],
        // 主题最大长度100字符
        'subject-max-length': [2, 'always', 100],
        // 正文必须以空行开头
        'body-leading-blank': [1, 'always'],
        // 脚注必须以空行开头
        'footer-leading-blank': [1, 'always']
    }
};
