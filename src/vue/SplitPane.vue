<template>
  <div class="split-pane" ref="splitPaneRef">
    <div class="pane left-pane" :style="{ flex: `${leftRatio}`, minWidth: '200px' }">
      <slot name="left"></slot>
    </div>
    <div class="resizer" ref="resizerRef" @mousedown="startResize"></div>
    <div class="pane right-pane" :style="{ flex: `${rightRatio}`, minWidth: '300px' }">
      <slot name="right"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'

const splitPaneRef = ref<HTMLElement | null>(null)
const resizerRef = ref<HTMLElement | null>(null)

// 左右面板的比例，默认为6:4
const leftRatio = ref(6)
const rightRatio = ref(4)

let isResizing = false
let startX = 0
let startLeftRatio = 0
let startRightRatio = 0

const startResize = (e: MouseEvent) => {
  isResizing = true
  startX = e.clientX
  
  // 记录开始时的比例
startLeftRatio = leftRatio.value
startRightRatio = rightRatio.value
  
  // 添加全局事件监听器
  document.addEventListener('mousemove', resize)
  document.addEventListener('mouseup', stopResize)
  
  // 防止文本选中
  e.preventDefault()
}

const resize = (e: MouseEvent) => {
  if (!isResizing || !splitPaneRef.value) return
  
  const containerWidth = splitPaneRef.value.clientWidth
  const dx = e.clientX - startX
  
  // 计算新的比例
  const totalRatio = startLeftRatio + startRightRatio
  const pixelPerRatio = containerWidth / totalRatio
  
  const deltaRatio = dx / pixelPerRatio
  
  // 更新比例，确保不会小于最小值
  const newLeftRatio = Math.max(2, startLeftRatio + deltaRatio)
  const newRightRatio = Math.max(3, startRightRatio - deltaRatio)
  
  // 重新计算实际比例以保持总和不变
  const newTotal = newLeftRatio + newRightRatio
  leftRatio.value = (newLeftRatio / newTotal) * totalRatio
  rightRatio.value = (newRightRatio / newTotal) * totalRatio
}

const stopResize = () => {
  isResizing = false
  
  // 移除全局事件监听器
  document.removeEventListener('mousemove', resize)
  document.removeEventListener('mouseup', stopResize)
}

// 清理事件监听器
onMounted(() => {
  // 组件挂载时的初始化逻辑（如果需要）
})

onBeforeUnmount(() => {
  // 确保在组件销毁前移除所有事件监听器
  document.removeEventListener('mousemove', resize)
  document.removeEventListener('mouseup', stopResize)
})
</script>

<style scoped>
.split-pane {
  display: flex;
  flex-direction: row;
  height: 100%;
  width: 100%;
}

.pane {
  height: 100%;
  overflow: hidden;
}

.left-pane {
  background: var(--secondary-bg);
}

.right-pane {
  background: var(--card-bg);
}

.resizer {
  width: 6px;
  background: #e0e4eb;
  cursor: col-resize;
  position: relative;
  transition: background-color 0.2s ease;
}

.resizer:hover {
  background: #4c8bf5;
}

.resizer::before {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 2px;
  height: 30px;
  background: #a0a8b8;
  border-radius: 1px;
}
</style>