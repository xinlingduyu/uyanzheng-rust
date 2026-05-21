<template>
  <div class="block lg:hidden button-menu">
    <a-trigger
      :trigger="['click']"
      clickToClose
      position="top"
      v-model:popupVisible="popupVisible"
    >
      <div :class="`button-trigger ${popupVisible ? 'button-trigger-active' : ''}`">
        <icon-close v-if="popupVisible" />
        <icon-menu v-else />
      </div>
      <template #content>
        <a-menu mode="popButton" showCollapseButton :popup-max-height="isMobile ? '80vh' : 360">
          <children-menu v-model="userStore.routers" />
        </a-menu>
      </template>
    </a-trigger>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useAppStore, useUserStore } from '@/store'

import ChildrenMenu from './components/children-menu.vue'

const userStore = useUserStore()
const popupVisible = ref(false)
const isMobile = computed(() => window.innerWidth < 768)

</script>

<style scoped>
.button-menu :deep(.arco-menu-pop-button) {
  max-height: 80vh;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
}
</style>
