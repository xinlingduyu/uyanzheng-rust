<template>
  <div class="pay-config">
    <a-spin :loading="loading" class="w-full">
      <a-row :gutter="[16, 16]">
        <!-- 支付宝配置 -->
        <a-col :xs="24" :sm="24" :md="12" :lg="12">
          <a-card :bordered="false" class="h-full">
            <template #title>
              <div class="flex items-center justify-between">
                <span>支付宝支付</span>
                <a-switch 
                  v-model="aliForm.state" 
                  checked-value="on" 
                  unchecked-value="off"
                  size="small"
                >
                  <template #checked>启用</template>
                  <template #unchecked>关闭</template>
                </a-switch>
              </div>
            </template>

            <!-- 启用状态时显示配置表单 -->
            <div v-if="aliForm.state === 'on'" class="config-form">
              <a-form :model="aliForm" layout="vertical">
                <!-- 支付引擎选择 -->
                <a-form-item label="支付引擎">
                  <a-select v-model="aliForm.type" placeholder="请选择支付引擎">
                    <a-option v-for="p in aliPlugins" :key="p.id" :value="p.id">
                      {{ p.name }}
                    </a-option>
                  </a-select>
                  <div v-if="currentAliPlugin?.extra" class="text-xs text-gray-400 mt-1">
                    {{ currentAliPlugin.extra }}
                  </div>
                </a-form-item>

                <!-- 动态渲染表单字段 -->
                <template v-if="currentAliFormFields.length > 0">
                  <template v-for="field in currentAliFormFields" :key="field.key">
                    <!-- 选择类型 -->
                    <a-form-item v-if="field.config.type === 'select'" :label="field.config.name">
                      <a-select 
                        v-model="aliForm.config[field.key]"
                        :placeholder="field.config.placeholder"
                        :multiple="field.config.multiple"
                        allow-clear
                      >
                        <a-option v-for="(label, value) in field.config.option" :key="value" :value="value">
                          {{ label }}
                        </a-option>
                      </a-select>
                      <div v-if="field.config.extra" class="text-xs text-gray-400 mt-1">
                        {{ field.config.extra }}
                      </div>
                    </a-form-item>
                    
                    <!-- 文本域 -->
                    <a-form-item v-else-if="field.config.type === 'textarea'" :label="field.config.name">
                      <a-textarea 
                        v-model="aliForm.config[field.key]" 
                        :placeholder="field.config.placeholder" 
                        :auto-size="{ minRows: 2, maxRows: 4 }" 
                      />
                    </a-form-item>
                    
                    <!-- 普通输入框 -->
                    <a-form-item v-else :label="field.config.name">
                      <a-input 
                        v-model="aliForm.config[field.key]" 
                        :placeholder="field.config.placeholder" 
                      />
                      <div v-if="field.config.extra" class="text-xs text-gray-400 mt-1">
                        {{ field.config.extra }}
                      </div>
                    </a-form-item>
                  </template>
                </template>

                <a-form-item>
                  <a-button type="primary" @click="saveAliPay" :loading="saving" long>
                    保存配置
                  </a-button>
                </a-form-item>
              </a-form>
            </div>

            <!-- 未启用状态 -->
            <div v-else class="disabled-tip">
              <a-empty description="支付功能未启用，请开启后配置">
                <template #image>
                  <icon-pay-circle :size="48" style="color: var(--color-text-3)" />
                </template>
              </a-empty>
            </div>
          </a-card>
        </a-col>

        <!-- 微信支付配置 -->
        <a-col :xs="24" :sm="24" :md="12" :lg="12">
          <a-card :bordered="false" class="h-full">
            <template #title>
              <div class="flex items-center justify-between">
                <span>微信支付</span>
                <a-switch 
                  v-model="wxForm.state" 
                  checked-value="on" 
                  unchecked-value="off"
                  size="small"
                >
                  <template #checked>启用</template>
                  <template #unchecked>关闭</template>
                </a-switch>
              </div>
            </template>

            <!-- 启用状态时显示配置表单 -->
            <div v-if="wxForm.state === 'on'" class="config-form">
              <a-form :model="wxForm" layout="vertical">
                <!-- 支付引擎选择 -->
                <a-form-item label="支付引擎">
                  <a-select v-model="wxForm.type" placeholder="请选择支付引擎">
                    <a-option v-for="p in wxPlugins" :key="p.id" :value="p.id">
                      {{ p.name }}
                    </a-option>
                  </a-select>
                  <div v-if="currentWxPlugin?.extra" class="text-xs text-gray-400 mt-1">
                    {{ currentWxPlugin.extra }}
                  </div>
                </a-form-item>

                <!-- 动态渲染表单字段 -->
                <template v-if="currentWxFormFields.length > 0">
                  <template v-for="field in currentWxFormFields" :key="field.key">
                    <!-- 选择类型 -->
                    <a-form-item v-if="field.config.type === 'select'" :label="field.config.name">
                      <a-select 
                        v-model="wxForm.config[field.key]"
                        :placeholder="field.config.placeholder"
                        :multiple="field.config.multiple"
                        allow-clear
                      >
                        <a-option v-for="(label, value) in field.config.option" :key="value" :value="value">
                          {{ label }}
                        </a-option>
                      </a-select>
                      <div v-if="field.config.extra" class="text-xs text-gray-400 mt-1">
                        {{ field.config.extra }}
                      </div>
                    </a-form-item>
                    
                    <!-- 文本域 -->
                    <a-form-item v-else-if="field.config.type === 'textarea'" :label="field.config.name">
                      <a-textarea 
                        v-model="wxForm.config[field.key]" 
                        :placeholder="field.config.placeholder" 
                        :auto-size="{ minRows: 2, maxRows: 4 }" 
                      />
                    </a-form-item>
                    
                    <!-- 普通输入框 -->
                    <a-form-item v-else :label="field.config.name">
                      <a-input 
                        v-model="wxForm.config[field.key]" 
                        :placeholder="field.config.placeholder" 
                      />
                      <div v-if="field.config.extra" class="text-xs text-gray-400 mt-1">
                        {{ field.config.extra }}
                      </div>
                    </a-form-item>
                  </template>
                </template>

                <a-form-item>
                  <a-button type="primary" @click="saveWxPay" :loading="saving" long>
                    保存配置
                  </a-button>
                </a-form-item>
              </a-form>
            </div>

            <!-- 未启用状态 -->
            <div v-else class="disabled-tip">
              <a-empty description="支付功能未启用，请开启后配置">
                <template #image>
                  <icon-pay-circle :size="48" style="color: var(--color-text-3)" />
                </template>
              </a-empty>
            </div>
          </a-card>
        </a-col>
      </a-row>

      <!-- 支付插件说明 -->
      <a-card title="支付插件说明" :bordered="false" class="mt-4 plugin-info-card">
        <div class="plugin-list">
          <div v-for="plugin in plugins" :key="plugin.id" class="plugin-item">
            <div class="plugin-name">{{ plugin.name }}</div>
            <div v-if="plugin.extra" class="plugin-extra">{{ plugin.extra }}</div>
            <div class="plugin-fields">
              <a-tag 
                v-for="field in getFormFields(plugin.form)" 
                :key="field.key" 
                color="arcoblue" 
                class="field-tag"
              >
                {{ field.config.name }}
              </a-tag>
            </div>
          </div>
        </div>
      </a-card>
    </a-spin>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { Message } from '@arco-design/web-vue'
import payApi from '@/api/system/pay'
import tool from '@/utils/tool'

const loading = ref(false)
const saving = ref(false)
const appid = computed(() => tool.local.get('currentAppId'))

// 所有插件列表
const plugins = ref([])

/**
 * 根据 type 字段筛选插件
 * type: "all" - 皆网支付，可用于支付宝和微信
 * type: "ali" - 原生支付宝
 * type: "wx" - 原生微信
 */
const aliPlugins = computed(() => {
  return plugins.value.filter(p => p.type === 'all' || p.type === 'ali')
})

const wxPlugins = computed(() => {
  return plugins.value.filter(p => p.type === 'all' || p.type === 'wx')
})

// 支付宝表单
const aliForm = reactive({
  state: 'off',
  type: '',
  config: {}
})

// 微信表单
const wxForm = reactive({
  state: 'off',
  type: '',
  config: {}
})

/**
 * 将 form 对象转换为数组，方便遍历
 * form 格式: { "fieldKey": { name, type, placeholder }, ... }
 */
const getFormFields = (form) => {
  if (!form || typeof form !== 'object') return []
  return Object.entries(form).map(([key, config]) => ({
    key,
    config
  }))
}

// 当前选中的支付宝插件
const currentAliPlugin = computed(() => {
  return aliPlugins.value.find(p => p.id === aliForm.type)
})

// 当前选中的微信插件
const currentWxPlugin = computed(() => {
  return wxPlugins.value.find(p => p.id === wxForm.type)
})

// 当前支付宝插件的表单字段
const currentAliFormFields = computed(() => {
  return getFormFields(currentAliPlugin.value?.form)
})

// 当前微信插件的表单字段
const currentWxFormFields = computed(() => {
  return getFormFields(currentWxPlugin.value?.form)
})

// 初始化配置对象
const initConfig = (form) => {
  const config = {}
  if (form && typeof form === 'object') {
    Object.keys(form).forEach(key => {
      config[key] = form[key].type === 'select' && form[key].multiple ? [] : ''
    })
  }
  return config
}

// 当支付宝引擎切换时
watch(() => aliForm.type, (newType) => {
  const plugin = aliPlugins.value.find(p => p.id === newType)
  if (plugin) {
    const newConfig = initConfig(plugin.form)
    aliForm.config = { ...newConfig, ...aliForm.config }
  }
})

// 当微信引擎切换时
watch(() => wxForm.type, (newType) => {
  const plugin = wxPlugins.value.find(p => p.id === newType)
  if (plugin) {
    const newConfig = initConfig(plugin.form)
    wxForm.config = { ...newConfig, ...wxForm.config }
  }
})

// 加载配置
const loadConfig = async () => {
  loading.value = true
  try {
    const res = await payApi.getInfo()
    if (res.code === 200 && res.data) {
      plugins.value = res.data.plug || []
      const info = res.data.info || {}
      
      aliForm.state = info.pay_ali_state || 'off'
      aliForm.type = info.pay_ali_type || ''
      aliForm.config = info.pay_ali_config || {}
      
      wxForm.state = info.pay_wx_state || 'off'
      wxForm.type = info.pay_wx_type || ''
      wxForm.config = info.pay_wx_config || {}
    }
  } catch (e) {
    console.error('加载配置失败', e)
  } finally {
    loading.value = false
  }
}

// 保存支付宝配置
const saveAliPay = async () => {
  if (!appid.value) {
    Message.warning('请先选择应用')
    return
  }
  
  saving.value = true
  try {
    const res = await payApi.edit({
      id: parseInt(appid.value),
      pay_ali_state: aliForm.state,
      pay_ali_type: aliForm.type,
      pay_ali_config: aliForm.config
    })
    if (res.code === 200) {
      Message.success('支付宝配置保存成功')
    } else {
      Message.error(res.msg || '保存失败')
    }
  } catch (e) {
    Message.error('保存失败')
  } finally {
    saving.value = false
  }
}

// 保存微信配置
const saveWxPay = async () => {
  if (!appid.value) {
    Message.warning('请先选择应用')
    return
  }
  
  saving.value = true
  try {
    const res = await payApi.edit({
      id: parseInt(appid.value),
      pay_wx_state: wxForm.state,
      pay_wx_type: wxForm.type,
      pay_wx_config: wxForm.config
    })
    if (res.code === 200) {
      Message.success('微信配置保存成功')
    } else {
      Message.error(res.msg || '保存失败')
    }
  } catch (e) {
    Message.error('保存失败')
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadConfig()
})
</script>

<script>
export default { name: 'PayConfig' }
</script>

<style scoped>
.pay-config {
  padding: 20px;
  min-height: 100%;
}

/* 玻璃卡片 */
.pay-config :deep(.arco-card) {
  position: relative;
  background: transparent;
  border: none;
  border-radius: 20px;
  overflow: hidden;
}

/* 卡片边缘玻璃效果层 - 置于底层 */
.pay-config :deep(.arco-card::before) {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 20px;
  padding: 1px;
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.5) 0%,
    rgba(255, 255, 255, 0.1) 30%,
    rgba(255, 255, 255, 0.05) 70%,
    rgba(255, 255, 255, 0.3) 100%
  );
  -webkit-mask: 
    linear-gradient(#fff 0 0) content-box, 
    linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
  z-index: 0;
}

/* 卡片内部 */
.pay-config :deep(.arco-card) .arco-card-header {
  position: relative;
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px 20px 0 0;
  z-index: 1;
}

.pay-config :deep(.arco-card) .arco-card-body {
  position: relative;
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  z-index: 1;
}

/* 卡片阴影 */
.pay-config :deep(.arco-card::after) {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 20px;
  box-shadow: 
    0 4px 24px -1px rgba(0, 0, 0, 0.06),
    0 1px 2px rgba(0, 0, 0, 0.04);
  pointer-events: none;
  z-index: 0;
}

/* Hover效果 */
.pay-config :deep(.arco-card:hover::before) {
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.6) 0%,
    rgba(255, 255, 255, 0.15) 30%,
    rgba(255, 255, 255, 0.08) 70%,
    rgba(255, 255, 255, 0.4) 100%
  );
}

.pay-config :deep(.arco-card:hover) .arco-card-body {
  background: rgba(255, 255, 255, 0.12);
}

/* 暗色主题 */
body[arco-theme='dark'] .pay-config :deep(.arco-card::before) {
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.15) 0%,
    rgba(255, 255, 255, 0.03) 30%,
    rgba(255, 255, 255, 0.02) 70%,
    rgba(255, 255, 255, 0.1) 100%
  );
}

body[arco-theme='dark'] .pay-config :deep(.arco-card) .arco-card-header {
  background: rgba(255, 255, 255, 0.05);
  border-bottom-color: rgba(255, 255, 255, 0.05);
}

body[arco-theme='dark'] .pay-config :deep(.arco-card) .arco-card-body {
  background: rgba(255, 255, 255, 0.02);
}

body[arco-theme='dark'] .pay-config :deep(.arco-card::after) {
  box-shadow: 
    0 4px 24px -1px rgba(0, 0, 0, 0.2),
    0 1px 2px rgba(0, 0, 0, 0.15);
}

/* 卡片标题 */
.pay-config :deep(.arco-card-header-title) {
  font-weight: 600;
  font-size: 15px;
  letter-spacing: 0.02em;
}

/* 表单项 */
.pay-config :deep(.arco-form-item-label) {
  font-weight: 500;
}

/* 输入框 - 确保在上层 */
.pay-config :deep(.arco-form-item-wrapper) {
  position: relative;
  z-index: 2;
}

.pay-config :deep(.arco-input-wrapper),
.pay-config :deep(.arco-textarea-wrapper),
.pay-config :deep(.arco-select-view) {
  position: relative;
  z-index: 2;
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 10px;
  transition: all 0.2s ease;
}

.pay-config :deep(.arco-input-wrapper:hover),
.pay-config :deep(.arco-textarea-wrapper:hover),
.pay-config :deep(.arco-select-view:hover) {
  background: rgba(255, 255, 255, 0.65);
}

.pay-config :deep(.arco-input-wrapper:focus-within),
.pay-config :deep(.arco-textarea-wrapper:focus-within),
.pay-config :deep(.arco-select-view-focused) {
  background: rgba(255, 255, 255, 0.85);
  border-color: rgb(var(--primary-6));
  box-shadow: 0 0 0 2px rgba(var(--primary-6), 0.1);
}

body[arco-theme='dark'] .pay-config :deep(.arco-input-wrapper),
body[arco-theme='dark'] .pay-config :deep(.arco-textarea-wrapper),
body[arco-theme='dark'] .pay-config :deep(.arco-select-view) {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.08);
}

body[arco-theme='dark'] .pay-config :deep(.arco-input-wrapper:hover),
body[arco-theme='dark'] .pay-config :deep(.arco-textarea-wrapper:hover),
body[arco-theme='dark'] .pay-config :deep(.arco-select-view:hover) {
  background: rgba(255, 255, 255, 0.08);
}

body[arco-theme='dark'] .pay-config :deep(.arco-input-wrapper:focus-within),
body[arco-theme='dark'] .pay-config :deep(.arco-textarea-wrapper:focus-within),
body[arco-theme='dark'] .pay-config :deep(.arco-select-view-focused) {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgb(var(--primary-6));
}

/* 下拉菜单 */
.pay-config :deep(.arco-select-popup) {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(12px);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
}

/* 按钮 */
.pay-config :deep(.arco-btn-primary) {
  background: rgb(var(--primary-6));
  border: none;
  border-radius: 10px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.pay-config :deep(.arco-btn-primary:hover) {
  background: rgb(var(--primary-5));
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(var(--primary-6), 0.3);
}

/* 开关 */
.pay-config :deep(.arco-switch-checked) {
  background: rgb(var(--primary-6));
}

/* 标签 */
.pay-config :deep(.arco-tag) {
  border-radius: 6px;
}

/* 空状态 */
.disabled-tip {
  padding: 40px 20px;
  text-align: center;
}

.disabled-tip :deep(.arco-empty-icon) {
  opacity: 0.6;
}

/* 插件说明卡片 */
.plugin-info-card :deep(.arco-card-body) {
  padding: 16px 20px;
}

.plugin-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

.plugin-item {
  padding: 12px 16px;
  background: rgba(var(--primary-6), 0.04);
  border-radius: 10px;
  border: 1px solid rgba(var(--primary-6), 0.08);
}

.plugin-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--color-text-1);
  margin-bottom: 6px;
}

.plugin-extra {
  font-size: 12px;
  color: var(--color-text-3);
  margin-bottom: 8px;
  word-break: break-all;
}

.plugin-fields {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.field-tag {
  margin: 0;
}

/* 工具类 */
.h-full { height: 100%; }
.mt-4 { margin-top: 16px; }
.mt-1 { margin-top: 4px; }
.text-xs { font-size: 11px; }
.text-gray-400 { color: var(--color-text-3); }
.w-full { width: 100%; }
.config-form { padding-top: 16px; }
.flex { display: flex; }
.items-center { align-items: center; }
.justify-between { justify-content: space-between; }

/* 移动端适配 */
@media screen and (max-width: 768px) {
  .pay-config {
    padding: 12px;
  }
  
  .pay-config :deep(.arco-card) {
    border-radius: 12px;
  }
  
  .pay-config :deep(.arco-card-header) {
    padding: 12px 16px;
    border-radius: 12px 12px 0 0;
  }
  
  .pay-config :deep(.arco-card-body) {
    padding: 16px;
  }
  
  .pay-config :deep(.arco-card-header-title) {
    font-size: 14px;
  }
  
  .pay-config :deep(.arco-form-item) {
    margin-bottom: 16px;
  }
  
  .pay-config :deep(.arco-form-item-label) {
    font-size: 13px;
  }
  
  .disabled-tip {
    padding: 24px 16px;
  }
  
  .plugin-list {
    grid-template-columns: 1fr;
    gap: 12px;
  }
  
  .plugin-info-card :deep(.arco-card-body) {
    padding: 12px 16px;
  }
}
</style>
