#!/usr/bin/env node

import {execSync} from 'child_process';
import {readFileSync} from 'fs';

// 获取提交消息
function getCommitMessage() {
    try {
        // 尝试从 COMMIT_EDITMSG 文件获取提交消息（这是更可靠的方法）
        const msgFilePath = process.argv[2] || '.git/COMMIT_EDITMSG';
        return readFileSync(msgFilePath, 'utf8').trim();
    } catch (e) {
        try {
            // 备选方案：使用 git log 获取最后一次提交信息
            return execSync('git log -1 --pretty=%B').toString().trim();
        } catch (err) {
            console.error('❌ 无法获取提交消息:', err.message);
            process.exit(1);
        }
    }
}

// 解析提交消息
function parseCommitMessage(message) {
    // 移除注释行
    const lines = message.split('\n').filter(line => !line.startsWith('#'));
    const firstLine = lines[0] || '';

    // 尝试匹配 Conventional Commits 格式
    const match = firstLine.match(/^(\w+)(?:\(([^)]+)\))?: (.+)$/);

    if (!match) {
        return { valid: false, type: null, scope: null, subject: null, raw: message };
    }

    return {
        valid: true,
        type: match[1],
        scope: match[2] || null,
        subject: match[3],
        raw: message
    };
}

// 验证提交消息
function validateCommitMessage(parsed) {
    const errors = [];
    const warnings = [];

    if (!parsed.valid) {
        errors.push('提交消息不符合 Conventional Commits 规范格式');
        return { errors, warnings };
    }

    // 检查类型是否有效
    const validTypes = [
        'feat', 'fix', 'docs', 'style', 'refactor', 'perf', 'test', 'chore', 'revert',
        'build', 'ci', 'Merge', 'merge'
    ];
    if (!validTypes.includes(parsed.type)) {
        errors.push(`类型 "${parsed.type}" 无效。允许的类型: ${validTypes.join(', ')}`);
    }

    // 检查主题是否为空
    if (!parsed.subject || parsed.subject.trim().length === 0) {
        errors.push('提交主题不能为空');
    }

    // 检查主题长度
    if (parsed.subject && parsed.subject.length > 100) {
        warnings.push('提交主题长度建议不超过 100 个字符');
    }

    // 检查是否有正文但未空行分隔
    const lines = parsed.raw.split('\n');
    if (lines.length > 1 && lines[1].trim().length > 0) {
        warnings.push('提交消息正文前应有一个空行');
    }

    return { errors, warnings };
}

// 主函数
function main() {
    const message = getCommitMessage();
    const parsed = parseCommitMessage(message);
    const { errors, warnings } = validateCommitMessage(parsed);

    if (errors.length === 0 && warnings.length === 0) {
        // 提交消息有效
        process.exit(0);
    }

    // 输出错误和警告
    console.error('\n❌ 提交消息验证失败:\n');

    if (errors.length > 0) {
        console.error('错误:');
        errors.forEach(error => console.error(`  • ${error}`));
        console.error('');
    }

    if (warnings.length > 0) {
        console.error('警告:');
        warnings.forEach(warning => console.error(`  • ${warning}`));
        console.error('');
    }

    // 显示详细帮助信息
    console.log(`
📋 Conventional Commits 规范要求:

  提交消息格式: <类型>[可选 范围]: <描述>

  例如:
    feat(认证): 添加微信登录支持
    fix: 解决订单页面崩溃问题

✅ 允许的提交类型:
  • feat:     新功能
  • fix:      修复错误
  • docs:     文档更新
  • style:    代码格式调整（不影响功能）
  • refactor: 代码重构
  • perf:     性能优化
  • test:     测试相关
  • ci:       CI/CD
  • chore:    杂项
  • build:    构建过程或辅助工具变动
  • revert:   回退提交

💡 提示:
  • 提交消息首字母不需要大写
  • 不要在类型后使用冒号以外的标点
  • 描述应该简洁明了，说明更改的目的而非细节
  • 正文（如果需要）应该更详细地解释更改内容

🌰 更多示例:
  • "feat: 添加用户管理功能"
  • "fix(API): 修复分页参数错误"
  • "docs: 更新安装指南"
  • "style: 格式化代码"
  `);

    // 显示当前提交消息
    console.log('📝 您的提交消息:');
    console.log(`   ${message.split('\n').join('\n   ')}\n`);

    process.exit(1);
}

// 执行主函数
main();
