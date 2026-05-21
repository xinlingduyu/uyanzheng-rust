<template>
  <div class="block lg:hidden button-menu" ref="menuRef">
    <a-trigger
      :trigger="['click']"
      clickToClose
      position="top"
      v-model:popupVisible="popupVisible"
      :popup-container="getContainer"
    >
      <div :class="`button-trigger ${popupVisible ? 'button-trigger-active' : ''}`">
        <icon-close v-if="popupVisible" />
        <icon-menu v-else />
      </div>
      <template #content>
        <a-menu mode="popButton" showCollapseButton :popup-max-height="10000">
          <children-menu v-model="userStore.routers" />
        </a-menu>
      </template>
    </a-trigger>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useAppStore, useUserStore } from '@/store'

import ChildrenMenu from './components/children-menu.vue'

const userStore = useUserStore()
const popupVisible = ref(false)
const menuRef = ref(null)

const getContainer = () => {
  return menuRef.value
}
</script>

<style scoped>
.button-menu {
  position: relative;
}

/* 弹窗容器：可滚动 */
.button-menu :deep(.arco-trigger-popup) {
  overflow-y: auto !important;
  max-height: 70vh !important;
}
.button-menu :deep(.arco-trigger-popup-content) {
  overflow-y: auto !important;
  max-height: 70vh !important;
}
.button-menu :deep(.arco-menu-pop-button) {
  overflow-y: auto !important;
  max-height: 70vh !important;
}
</style>