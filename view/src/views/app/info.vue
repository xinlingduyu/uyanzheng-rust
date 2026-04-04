<template>
  <div class="app-info-page">
    <!-- 应用基本信息 -->
    <a-card class="mb-4" :bordered="false">
      <p class="font-semibold text-lg mb-4">APPID：{{ appInfo.id }}</p>
      <a-divider />
      <a-form :model="appInfo" layout="vertical" @submit="handleSubmit">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="应用名称">
              <a-input v-model="appInfo.app_name" placeholder="请设置应用名称" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="APPKEY">
              <a-input-group>
                <a-input v-model="appInfo.app_key" readonly placeholder="请设置APPKEY" />
                <a-button @click="generateKey">更换</a-button>
                <a-button type="primary" @click="copyKey">复制</a-button>
              </a-input-group>
            </a-form-item>
          </a-col>
        </a-row>
        
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="运营模式">
              <a-radio-group v-model="appInfo.app_mode">
                <a-radio value="y">收费</a-radio>
                <a-radio value="n">免费</a-radio>
              </a-radio-group>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="应用状态">
              <a-radio-group v-model="appInfo.app_state">
                <a-radio value="on">正常</a-radio>
                <a-radio value="off">关闭</a-radio>
              </a-radio-group>
            </a-form-item>
          </a-col>
        </a-row>

        <a-form-item v-if="appInfo.app_state === 'off'" label="关闭消息">
          <a-textarea v-model="appInfo.app_off_msg" placeholder="如：系统维护中" :auto-size="{ minRows: 2, maxRows: 4 }" />
        </a-form-item>

        <a-form-item label="应用图标">
          <a-upload
            :action="uploadUrl"
            :headers="uploadHeaders"
            list-type="picture-card"
            :file-list="fileList"
            :limit="1"
            @success="handleUploadSuccess"
            @error="handleUploadError"
          >
            <template #upload-button>
              <div class="upload-trigger">
                <icon-plus />
                <div class="text-sm mt-1">上传图标</div>
              </div>
            </template>
          </a-upload>
        </a-form-item>

        <a-form-item>
          <a-button type="primary" html-type="submit" :loading="loading">
            提交
          </a-button>
        </a-form-item>
      </a-form>
    </a-card>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { Message } from '@arco-design/web-vue'
import appApi from '@/api/system/app'
import tool from '@/utils/tool'

const loading = ref(false)
const fileList = ref([])

const appInfo = reactive({
  id: '',
  app_name: '',
  app_key: '',
  app_logo: '',
  app_mode: 'y',
  app_state: 'on',
  app_off_msg: ''
})

const uploadUrl = import.meta.env.VITE_APP_BASE_URL + '/api/admin/upload/img'
const uploadHeaders = {
  Token: tool.local.get(import.meta.env.VITE_APP_TOKEN_PREFIX),
  appid: tool.local.get('currentAppId') || ''
}

// 加载应用信息
const loadAppInfo = async () => {
  try {
    const res = await appApi.getInfo(['id', 'app_name', 'app_key', 'app_logo', 'app_mode', 'app_state', 'app_off_msg'])
    if (res.code === 200) {
      Object.assign(appInfo, res.data)
      if (appInfo.app_logo) {
        fileList.value = [{ url: tool.attachUrl(appInfo.app_logo), name: 'logo' }]
      }
    } else {
      Message.error(res.msg)
    }
  } catch (e) {
    Message.error('加载应用信息失败：' + e)
  }
}

// 生成随机KEY
const generateKey = () => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789'
  let key = ''
  for (let i = 0; i < 32; i++) {
    key += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  appInfo.app_key = key
  Message.success('已生成新的APPKEY')
}

// 复制KEY
const copyKey = async () => {
  try {
    await navigator.clipboard.writeText(appInfo.app_key)
    Message.success('复制成功')
  } catch (e) {
    Message.error('复制失败，请手动复制')
  }
}

// 上传成功
const handleUploadSuccess = (file) => {
  if (file.response?.code === 200) {
    appInfo.app_logo = file.response.data.url
    Message.success('上传成功')
  } else {
    Message.error(file.response?.msg || '上传失败')
  }
}

// 上传失败
const handleUploadError = () => {
  Message.error('上传失败')
}

// 提交
const handleSubmit = async () => {
  loading.value = true
  try {
    const res = await appApi.edit({ id: appInfo.id, ...appInfo })
    if (res.code === 200) {
      Message.success(res.msg || '保存成功')
    } else {
      Message.error(res.msg)
    }
  } catch (e) {
    Message.error('保存失败：' + e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadAppInfo()
})
</script>

<script>
export default { name: 'AppInfo' }
</script>

<style scoped>
.app-info-page {
  padding: 16px;
}

.upload-trigger {
  width: 100px;
  height: 100px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-3);
}
</style>