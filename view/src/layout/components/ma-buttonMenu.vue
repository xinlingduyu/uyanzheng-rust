<template>
  <div class="block lg:hidden button-menu">
    <!-- 触发按钮 -->
    <div
      :class="['button-trigger', { 'button-trigger-active': showSheet }]"
      @click="toggleSheet"
    >
      <icon-close v-if="showSheet" />
      <icon-menu v-else />
    </div>

    <!-- 遮罩层 -->
    <div v-if="showSheet" class="sheet-overlay" @click="closeSheet" />

    <!-- 底部弹出面板 -->
    <div v-if="showSheet" class="sheet-panel">
      <div class="sheet-header">
        <span class="sheet-title">导航菜单</span>
        <icon-close class="sheet-close" @click="closeSheet" />
      </div>
      <div class="sheet-body">
        <div
          v-for="menu in flatMenus"
          :key="menu.name"
          :class="['sheet-item', { active: route.name === menu.name }]"
          @click="goTo(menu)"
        >
          <sa-icon v-if="menu.meta.icon" :icon="menu.meta.icon" :size="18" class="sheet-item-icon" />
          <span class="sheet-item-text">{{ menu.meta.title }}</span>
          <icon-right class="sheet-item-arrow" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAppStore, useUserStore, useTagStore } from '@/store'

const router = useRouter()
const route = useRoute()
const userStore = useUserStore()
const appStore = useAppStore()
const tagStore = useTagStore()

const showSheet = ref(false)

/** 将所有路由拍平成一级菜单列表（只保留可点击的叶子节点） */
const flatMenus = computed(() => {
  const result = []
  const walk = (items) => {
    if (!items) return
    for (const item of items) {
      if (item.meta && item.meta.hidden) continue
      if (!item.children || item.children.length === 0) {
        result.push(item)
      } else {
        walk(item.children)
      }
    }
  }
  walk(userStore.routers)
  return result
})

const toggleSheet = () => {
  showSheet.value = !showSheet.value
}

const closeSheet = () => {
  showSheet.value = false
}

const goTo = (menu) => {
  if (menu.meta && menu.meta.type === 'L') {
    window.open(menu.path, '_blank', 'noopener,noreferrer')
  } else {
    router.push(menu.path)
    tagStore.addTag({ name: menu.name, title: menu.meta.title, path: menu.path })
  }
  closeSheet()
}

// 路由变化时自动关闭
watch(() => route.path, () => { showSheet.value = false })
</script>

<style scoped>
.button-menu {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 999;
}

/* 触发按钮 */
.button-trigger {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgb(var(--primary-6));
  color: #fff;
  font-size: 22px;
  cursor: pointer;
  box-shadow: 0 4px 16px rgba(var(--primary-6), 0.4);
  transition: all 0.2s;
}
.button-trigger:active {
  transform: scale(0.92);
}
.button-trigger-active {
  background: rgb(var(--danger-6));
  box-shadow: 0 4px 16px rgba(var(--danger-6), 0.4);
}

/* 遮罩 */
.sheet-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 998;
}

/* 底部面板 */
.sheet-panel {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  max-height: 75vh;
  z-index: 999;
  background: var(--color-bg-1);
  border-radius: 20px 20px 0 0;
  display: flex;
  flex-direction: column;
  animation: sheetUp 0.25s ease-out;
}

@keyframes sheetUp {
  from { transform: translateY(100%); }
  to { transform: translateY(0); }
}

/* 面板头部 */
.sheet-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border-2);
  flex-shrink: 0;
}
.sheet-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-1);
}
.sheet-close {
  font-size: 18px;
  color: var(--color-text-3);
  cursor: pointer;
  padding: 4px;
}
.sheet-close:active {
  color: var(--color-text-1);
}

/* 可滚动列表 */
.sheet-body {
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  padding: 8px 12px 24px;
  flex: 1;
}

/* 菜单项 */
.sheet-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 12px;
  cursor: pointer;
  transition: background 0.15s;
  margin-bottom: 2px;
}
.sheet-item:active {
  background: var(--color-fill-2);
}
.sheet-item.active {
  background: rgba(var(--primary-6), 0.08);
  color: rgb(var(--primary-6));
}
.sheet-item.active .sheet-item-text {
  font-weight: 600;
}

.sheet-item-icon {
  flex-shrink: 0;
  width: 20px;
  text-align: center;
  color: var(--color-text-3);
}
.sheet-item.active .sheet-item-icon {
  color: rgb(var(--primary-6));
}

.sheet-item-text {
  flex: 1;
  font-size: 14px;
  color: var(--color-text-1);
}

.sheet-item-arrow {
  font-size: 12px;
  color: var(--color-text-4);
  opacity: 0.5;
}
</style>