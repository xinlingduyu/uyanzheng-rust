<template>
  <div class="cloud-function">
    <!-- 操作栏 -->
    <a-card class="mb-4 ma-content-block" :bordered="false">
      <a-space>
        <a-button type="primary" @click="handleAdd">
          <template #icon><icon-plus /></template>
          添加函数
        </a-button>
      </a-space>
    </a-card>

    <!-- 函数列表 -->
    <a-card class="ma-content-block" :bordered="false">
      <a-table :columns="columns" :data="tableData" :loading="loading" row-key="id">
        <template #type="{ record }">
          <a-tag color="blue">{{ getTypeName(record.type) }}</a-tag>
        </template>
        <template #status="{ record }">
          <a-switch v-model="record.status" :checked-value="1" :unchecked-value="0" @change="handleStatusChange(record)" />
        </template>
        <template #actions="{ record }">
          <a-space>
            <a-button type="text" size="small" @click="handleEdit(record)">编辑</a-button>
            <a-button type="text" size="small" @click="handleViewCode(record)">查看代码</a-button>
            <a-popconfirm content="确定删除该函数吗？" @ok="handleDelete(record)">
              <a-button type="text" size="small" status="danger">删除</a-button>
            </a-popconfirm>
          </a-space>
        </template>
      </a-table>
    </a-card>

    <!-- 编辑弹窗 -->
    <a-modal v-model:visible="modalVisible" :title="modalTitle" :width="900" :footer="false" unmount-on-close>
      <a-form ref="formRef" :model="form" :rules="rules" layout="vertical">
        <a-row :gutter="16">
          <a-col :span="8">
            <a-form-item field="name" label="函数名称" required>
              <a-input v-model="form.name" placeholder="请输入函数名称" />
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item field="type" label="函数类型" required>
              <a-select v-model="form.type" placeholder="请选择类型">
                <a-option value="login">登录回调</a-option>
                <a-option value="register">注册回调</a-option>
                <a-option value="payment">支付回调</a-option>
                <a-option value="custom">自定义</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="8">
            <a-form-item field="status" label="状态">
              <a-radio-group v-model="form.status">
                <a-radio :value="1">启用</a-radio>
                <a-radio :value="0">禁用</a-radio>
              </a-radio-group>
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item field="code" label="函数代码">
          <div class="code-editor-wrapper">
            <div class="editor-toolbar">
              <a-space>
                <a-select v-model="editorTheme" size="small" style="width: 120px">
                  <a-option value="vs-dark">深色主题</a-option>
                  <a-option value="vs-light">浅色主题</a-option>
                </a-select>
                <a-select v-model="editorLanguage" size="small" style="width: 100px">
                  <a-option value="javascript">JavaScript</a-option>
                  <a-option value="typescript">TypeScript</a-option>
                </a-select>
                <a-button size="small" @click="formatCode">
                  <template #icon><icon-code /></template>
                  格式化
                </a-button>
                <a-button size="small" @click="insertTemplate">
                  <template #icon><icon-file /></template>
                  插入模板
                </a-button>
              </a-space>
            </div>
            <div ref="editorContainer" class="monaco-editor-container"></div>
          </div>
        </a-form-item>
        <a-form-item field="description" label="描述">
          <a-textarea v-model="form.description" placeholder="请输入函数描述" :auto-size="{ minRows: 2 }" />
        </a-form-item>
        <div class="form-footer">
          <a-space>
            <a-button @click="handleCancel">取消</a-button>
            <a-button type="primary" :loading="submitLoading" @click="handleSubmit">保存</a-button>
          </a-space>
        </div>
      </a-form>
    </a-modal>

    <!-- 查看代码弹窗 -->
    <a-modal v-model:visible="codeModalVisible" title="函数代码" :width="800" :footer="false" unmount-on-close>
      <div class="code-viewer-wrapper">
        <div class="viewer-toolbar">
          <a-space>
            <a-select v-model="viewerTheme" size="small" style="width: 120px" @change="updateViewerTheme">
              <a-option value="vs-dark">深色主题</a-option>
              <a-option value="vs-light">浅色主题</a-option>
            </a-select>
            <a-button size="small" @click="copyCode">
              <template #icon><icon-copy /></template>
              复制代码
            </a-button>
          </a-space>
        </div>
        <div ref="viewerContainer" class="monaco-viewer-container"></div>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import { Message } from '@arco-design/web-vue'
import * as monaco from 'monaco-editor'
import functionApi from '@/api/system/function'

// 表格数据
const tableData = ref([])
const loading = ref(false)

// 弹窗
const modalVisible = ref(false)
const modalTitle = computed(() => form.id ? '编辑函数' : '添加函数')
const formRef = ref(null)
const submitLoading = ref(false)
const form = reactive({
  id: '',
  name: '',
  type: '',
  code: '',
  description: '',
  status: 1
})

const rules = {
  name: [{ required: true, message: '请输入函数名称' }],
  type: [{ required: true, message: '请选择函数类型' }]
}

// 代码编辑器
const editorContainer = ref(null)
const editorInstance = ref(null)
const editorTheme = ref('vs-dark')
const editorLanguage = ref('javascript')

// 代码查看器
const codeModalVisible = ref(false)
const viewerContainer = ref(null)
const viewerInstance = ref(null)
const viewerTheme = ref('vs-dark')
const currentCode = ref('')

// 类型名称映射
const typeNames = {
  login: '登录回调',
  register: '注册回调',
  payment: '支付回调',
  custom: '自定义'
}

// 函数模板
const functionTemplate = `/**
 * 云函数示例
 * @param {Object} data - 传入的数据
 * @param {Object} user - 用户信息
 * @returns {Object} 返回处理结果
 */
async function handler(data, user) {
  // 在这里编写你的业务逻辑
  
  return {
    success: true,
    message: '处理成功',
    data: {}
  }
}`

// 表格列
const columns = [
  { title: 'ID', dataIndex: 'id', width: 80 },
  { title: '函数名称', dataIndex: 'name' },
  { title: '函数类型', dataIndex: 'type', slotName: 'type' },
  { title: '描述', dataIndex: 'description', ellipsis: true },
  { title: '状态', dataIndex: 'status', slotName: 'status', width: 80 },
  { title: '更新时间', dataIndex: 'update_time', width: 160 },
  { title: '操作', slotName: 'actions', width: 180 }
]

const getTypeName = (type) => typeNames[type] || type

// 初始化Monaco编辑器
const initEditor = () => {
  if (!editorContainer.value) return
  
  // 销毁旧实例
  if (editorInstance.value) {
    editorInstance.value.dispose()
  }
  
  editorInstance.value = monaco.editor.create(editorContainer.value, {
    value: form.code || functionTemplate,
    language: editorLanguage.value,
    theme: editorTheme.value,
    minimap: { enabled: true },
    fontSize: 14,
    lineNumbers: 'on',
    roundedSelection: true,
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 2,
    wordWrap: 'on',
    folding: true,
    renderLineHighlight: 'all',
    scrollbar: {
      verticalScrollbarSize: 10,
      horizontalScrollbarSize: 10
    }
  })
  
  // 监听内容变化
  editorInstance.value.onDidChangeModelContent(() => {
    form.code = editorInstance.value.getValue()
  })
}

// 初始化代码查看器
const initViewer = () => {
  if (!viewerContainer.value) return
  
  if (viewerInstance.value) {
    viewerInstance.value.dispose()
  }
  
  viewerInstance.value = monaco.editor.create(viewerContainer.value, {
    value: currentCode.value,
    language: 'javascript',
    theme: viewerTheme.value,
    minimap: { enabled: false },
    fontSize: 14,
    lineNumbers: 'on',
    readOnly: true,
    scrollBeyondLastLine: false,
    automaticLayout: true,
    wordWrap: 'on'
  })
}

// 更新编辑器主题
watch(editorTheme, (theme) => {
  if (editorInstance.value) {
    monaco.editor.setTheme(theme)
  }
})

// 更新编辑器语言
watch(editorLanguage, (lang) => {
  if (editorInstance.value) {
    const model = editorInstance.value.getModel()
    if (model) {
      monaco.editor.setModelLanguage(model, lang)
    }
  }
})

// 更新查看器主题
const updateViewerTheme = (theme) => {
  if (viewerInstance.value) {
    monaco.editor.setTheme(theme)
  }
}

// 格式化代码
const formatCode = () => {
  if (editorInstance.value) {
    editorInstance.value.getAction('editor.action.formatDocument').run()
  }
}

// 插入模板
const insertTemplate = () => {
  if (editorInstance.value) {
    editorInstance.value.setValue(functionTemplate)
    Message.success('已插入模板')
  }
}

// 复制代码
const copyCode = async () => {
  try {
    await navigator.clipboard.writeText(currentCode.value)
    Message.success('代码已复制到剪贴板')
  } catch (e) {
    Message.error('复制失败')
  }
}

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const res = await functionApi.getList({ page: 1, size: 100 })
    if (res.code === 200) {
      tableData.value = res.data.list || []
    }
  } catch (e) {
    Message.error('加载数据失败')
  } finally {
    loading.value = false
  }
}

// 添加
const handleAdd = () => {
  Object.assign(form, { id: '', name: '', type: '', code: '', description: '', status: 1 })
  modalVisible.value = true
  nextTick(() => {
    initEditor()
  })
}

// 编辑
const handleEdit = (record) => {
  Object.assign(form, record)
  modalVisible.value = true
  nextTick(() => {
    initEditor()
  })
}

// 查看代码
const handleViewCode = async (record) => {
  try {
    const res = await functionApi.getCode(record.id)
    if (res.code === 200) {
      currentCode.value = res.data.code || ''
      codeModalVisible.value = true
      nextTick(() => {
        initViewer()
      })
    }
  } catch (e) {
    Message.error('获取代码失败')
  }
}

// 提交
const handleSubmit = async () => {
  const valid = await formRef.value?.validate()
  if (valid) return

  submitLoading.value = true
  try {
    const api = form.id ? functionApi.edit : functionApi.add
    const res = await api(form)
    if (res.code === 200) {
      Message.success(form.id ? '编辑成功' : '添加成功')
      modalVisible.value = false
      loadData()
    }
  } catch (e) {
    Message.error('操作失败')
  } finally {
    submitLoading.value = false
  }
}

// 取消
const handleCancel = () => {
  modalVisible.value = false
  if (editorInstance.value) {
    editorInstance.value.dispose()
    editorInstance.value = null
  }
}

// 删除
const handleDelete = async (record) => {
  try {
    const res = await functionApi.del(record.id)
    if (res.code === 200) {
      Message.success('删除成功')
      loadData()
    }
  } catch (e) {
    Message.error('删除失败')
  }
}

// 状态切换
const handleStatusChange = async (record) => {
  try {
    await functionApi.edit({ id: record.id, status: record.status })
    Message.success('状态更新成功')
  } catch (e) {
    record.status = record.status === 1 ? 0 : 1
    Message.error('操作失败')
  }
}

onMounted(() => {
  loadData()
})

onBeforeUnmount(() => {
  if (editorInstance.value) {
    editorInstance.value.dispose()
  }
  if (viewerInstance.value) {
    viewerInstance.value.dispose()
  }
})
</script>

<script>
export default { name: 'CloudFunction' }
</script>

<style scoped>
.cloud-function {
  padding: 16px;
}

.code-editor-wrapper {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.editor-toolbar {
  padding: 8px 12px;
  background: transparent;
  border-bottom: 1px solid var(--color-border);
}

.monaco-editor-container {
  height: 400px;
}

.form-footer {
  padding-top: 16px;
  text-align: right;
  border-top: 1px solid var(--color-border);
  margin-top: 16px;
}

.code-viewer-wrapper {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.viewer-toolbar {
  padding: 8px 12px;
  background: transparent;
  border-bottom: 1px solid var(--color-border);
}

.monaco-viewer-container {
  height: 450px;
}
</style>