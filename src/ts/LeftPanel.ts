import {state} from "@/ts/global_states.ts";

// 更新状态消息
export function updateStatusMessage(message: string, isError = false) {
    state.statusMessage = message;
    // uiElements.statusMessage.className = 'status-message';
    // if (isError) uiElements.statusMessage.classList.add('error');
    state.statusMessageIsError = isError;

    if (message.includes('成功') || message.includes('连接')) {
        state.statusMessageIsSuccess = true;
    }
}
