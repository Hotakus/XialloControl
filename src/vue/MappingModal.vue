<template>
  <div class="modal-overlay" id="mapping-modal">
    <div class="modal">
      <div class="modal-header">
        <span id="modal-title">添加按键映射</span>
        <button class="modal-close" id="close-modal">&times;</button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label><i class="fas fa-gamepad"></i> 选择手柄按键</label>
          <select class="form-control" id="controller-button">
            <option value="">-- 请选择按键 --</option>
            <option value="A">A 按钮</option>
            <option value="B">B 按钮</option>
            <option value="X">X 按钮</option>
            <option value="Y">Y 按钮</option>
            <option value="LB">左肩键 (LB)</option>
            <option value="RB">右肩键 (RB)</option>
          </select>
        </div>

        <div class="form-group key-detector">
          <label><i class="fas fa-keyboard"></i> 映射到：</label>
          <div class="detector-area" id="key-detector-area">
            点击此处并按下键盘按键、鼠标按键或滚动滚轮
          </div>
          <div class="detector-hint">支持单键或组合键（如 Ctrl+C、Alt+F4）</div>
          <div class="key-display" id="key-display"></div>
        </div>
        <div id="modal-error" class="status-message error" style="display: none; margin-top: 15px;"></div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-outline" id="cancel-btn">取消</button>
        <button class="btn btn-primary" id="confirm-btn">确认</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 可以写组件逻辑
</script>

<style scoped>
/* 如果只作用于这个组件，可以写 scoped 样式 */
</style>
