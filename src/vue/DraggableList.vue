<template>
  <VueDraggable ref="el" v-model="state.mappings" :disabled="disabled" :force-fallback="true" :animation="200"
    ghostClass="ghost" class="button-map" @start="onStart" @update="onUpdate" @end="onEnd">
    <div v-for="mapping in state.mappings" :key="mapping.id" class="button-map-item">
      <div class="button-icon">{{ mapping.composed_button }}</div>
      <div class="key-text">映射到</div>
      <div class="key-value">{{ formatKeyDisplay(mapping.composed_shortcut_key) }}</div>
      <div class="item-actions">
        <button class="item-action-btn edit" @click="editButtonMap(mapping.id)">
          <svg t="1753769162786" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg"
            p-id="3801" width="200" height="200">
            <path
              d="M869.62198 290.936185c-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 501.688143c0 40.342824-32.8205 73.163323-73.163323 73.163323L252.963339 897.142777c-40.342824 0-73.163323-32.8205-73.163323-73.163323l0-606.206592c0-40.342824 32.8205-73.163323 73.163323-73.163323l407.621744 0c17.316387 0 31.355125-14.039761 31.355125-31.355125s-14.039761-31.355125-31.355125-31.355125L252.963339 81.899288c-74.92341 0-135.873574 60.950164-135.873574 135.873574l0 606.206592c0 74.92341 60.950164 135.873574 135.873574 135.873574l512.140193 0c74.92341 0 135.873574-60.950164 135.873574-135.873574L900.977106 322.292334C900.978129 304.975946 886.938368 290.936185 869.62198 290.936185z"
              fill="#707070" p-id="3802"></path>
            <path
              d="M535.946388 467.382826c6.01704 5.496178 13.59053 8.205892 21.143553 8.205892 8.502651 0 16.97358-3.434216 23.159466-10.201339L898.602012 116.986411c11.682064-12.779048 10.783601-32.615838-1.995447-44.297902-12.784164-11.676947-32.615838-10.783601-44.303019 2.000564L533.950941 423.084924C522.269901 435.863972 523.167341 455.700763 535.946388 467.382826z"
              fill="#707070" p-id="3803"></path>
            <path
              d="M355.315448 594.978876l0 30.589692c0 17.316387 14.039761 31.355125 31.355125 31.355125 17.316387 0 31.355125-14.039761-31.355125 31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125C369.355209 563.623751 355.315448 577.663512 355.315448 594.978876z"
              fill="#707070" p-id="3804"></path>
            <path
              d="M631.396297 656.924717c17.316387 0 31.355125-14.039761 31.355125-31.355125l0-30.589692c0-17.316387-14.039761-31.355125-31.355125-31.355125-17.316387 0-31.355125 14.039761-31.355125 31.355125l0 30.589692C600.041172 642.884956 614.07991 656.924717 631.396297 656.924717z"
              fill="#707070" p-id="3805"></path>
          </svg>
        </button>
        <button class="item-action-btn delete" @click="deleteButtonMap(mapping.id)">
          <svg t="1753765954234" class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg"
            p-id="2368" width="200" height="200">
            <path
              d="M840 288H688v-56c0-40-32-72-72-72h-208C368 160 336 192 336 232V288h-152c-12.8 0-24 11.2-24 24s11.2 24 24 24h656c12.8 0 24-11.2 24-24s-11.2-24-24-24zM384 288v-56c0-12.8 11.2-24 24-24h208c12.8 0 24 11.2 24 24V288H384zM758.4 384c-12.8 0-24 11.2-24 24v363.2c0 24-19.2 44.8-44.8 44.8H332.8c-24 0-44.8-19.2-44.8-44.8V408c0-12.8-11.2-24-24-24s-24 11.2-24 24v363.2c0 51.2 41.6 92.8 92.8 92.8h358.4c51.2 0 92.8-41.6 92.8-92.8V408c-1.6-12.8-12.8-24-25.6-24z"
              fill="#f57070" p-id="2369"></path>
            <path
              d="M444.8 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24zM627.2 744v-336c0-12.8-11.2-24-24-24s-24 11.2-24 24v336c0 12.8 11.2 24 24 24s24-11.2 24-24z"
              fill="#f57070" p-id="2370"></path>
          </svg>
        </button>
      </div>
    </div>
  </VueDraggable>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  type DraggableEvent,
  SortableEvent,
  type UseDraggableReturn,
  VueDraggable
} from 'vue-draggable-plus'
import {
  deleteButtonMap,
  editButtonMap,
  formatKeyDisplay,
  updateMappingsOrder
} from "@/ts/RightPanel.ts";
import { state } from '@/ts/global_states'

const el = ref<UseDraggableReturn>()
const disabled = ref(false)

const onStart = (e: SortableEvent) => {
  // console.log('start', e)
}

const onEnd = (e: SortableEvent) => {
  // console.log('onEnd', e)
  updateMappingsOrder(state.mappings)
}

const onUpdate = () => {
  console.log('update')
}
</script>

<style scoped>
.ghost {
  opacity: 0.3;
  background: #dbf4ffc4;
}

.button-svg-icon {
  padding: 3px;
  width: var(--button-icon-size);
  height: var(--button-icon-size);
}
</style>
