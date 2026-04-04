<template>
  <div class="version-management">
    <!-- 搜索栏 -->
    <a-card class="mb-4" :bordered="false">
      <a-form :model="searchForm" layout="inline" auto-label-width>
        <a-form-item label="版本号">
          <a-input v-model="searchForm.version" placeholder="请输入版本号" allow-clear style="width: 150px" />
        </a-form-item>
        <a-form-item label="状态">
          <a-select v-model="searchForm.status" placeholder="请选择" allow-clear style="width: 120px">
            <a-option :value="1">正常</a-option>
            <a-option :value="0">关闭</a-option>
            <a-option :value="-1">弃用</a-option>
          </a-select>
        </a-form-item>
        <a-form-item>
          <a-space>
            <a-button type="primary" @click="handleSearch">搜索</a-button>
            <a-button @click="handleReset">重置</a-button>
          </a-space>
        </a-form-item>
      </a-form>
    </a-card>

    <!-- 操作栏 -->
    <a-card class="mb-4" :bordered="false">
      <a-space>
        <a-button type="primary" @click="handleAdd">
          <template #icon><icon-plus /></template>
          添加版本
        </a-button>
        <a-button status="danger" :disabled="!selectedKeys.length" @click="handleBatchDelete">
          批量删除
        </a-button>
      </a-space>
    </a-card>

    <!-- 数据表格 -->
    <a-card :bordered="false">
      <a-table
        :columns="columns"
        :data="tableData"
        :loading="loading"
        :pagination="pagination"
        :row-selection="rowSelection"
        row-key="id"
        @page-change="handlePageChange"
      >
        <template #version="{ record }">
          <span class="font-mono font-medium">v{{ record.version }}</span>
        </template>
        <template #force_update="{ record }">
          <a-tag v-if="record.force_update" color="red">强制更新</a-tag>
          <a-tag v-else color="green">可选更新</a-tag>
        </template>
        <template #status="{ record }">
          <a-tag v-if="record.status === 1" color="green">正常</a-tag>
          <a-tag v-else-if="record.status === 0" color="orange">关闭</a-tag>
          <a-tag v-else color="gray">弃用</a-tag>
        </template>
        <template #actions="{ record }">
          <a-space>
            <a-button type="text" size="small" @click="handleEdit(record)">编辑</a-button>
            <a-button type="text" size="small" @click="handleDiscard(record)">
              {{ record.status === -1 ? '恢复' : '弃用' }}
            </a-button>
            <a-popconfirm content="确定删除该版本吗？" @ok="handleDelete(record)">
              <a-button type="text" size="small" status="danger">删除</a-button>
            </a-popconfirm>
          </a-space>
        </template>
      </a-table>
    </a-card>

    <!-- 编辑弹窗 -->
    <a-modal v-model:visible="modalVisible" :title="modalTitle" :width="560" @ok="handleSubmit" @cancel="handleCancel">
      <a-form ref="formRef" :model="form" :rules="rules" layout="vertical">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item field="version" label="版本号" required>
              <a-input v-model="form.version" placeholder="如: 1.0.0" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item field="version_code" label="版本号代码" required>
              <a-input-number v-model="form.version_code" :min="1" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item field="update_content" label="更新内容">
          <a-textarea v-model="form.update_content" placeholder="请输入更新内容" :auto-size="{ minRows: 3 }" />
        </a-form-item>
        <a-form-item field="download_url" label="下载地址">
          <a-input v-model="form.download_url" placeholder="请输入下载地址" />
        </a-form-item>
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item field="force_update" label="强制更新">
              <a-switch v-model="form.force_update" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item field="status" label="状态">
              <a-select v-model="form.status">
                <a-option :value="1">正常</a-option>
                <a-option :value="0">关闭</a-option>
                <a-option :value="-1">弃用</a-option>
              </a-select>
            </a-form-item>
          </a-col>
        </a-row>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { Message } from '@arco-design/web-vue'
import verApi from '@/api/system/ver'

const searchForm = reactive({ version: '', status: undefined })
const tableData = ref([])
const loading = ref(false)
const pagination = reactive({ current: 1, pageSize: 20, total: 0, showTotal: true })

const selectedKeys = ref([])
const rowSelection = reactive({ type: 'checkbox', showCheckedAll: true })

const modalVisible = ref(false)
const modalTitle = computed(() => form.id ? '编辑版本' : '添加版本')
const formRef = ref(null)
const form = reactive({
  id: '', version: '', version_code: 100, update_content: '', download_url: '', force_update: false, status: 1
})

const rules = {
  version: [{ required: true, message: '请输入版本号' }],
  version_code: [{ required: true, message: '请输入版本号代码' }]
}

const columns = [
  { title: 'ID', dataIndex: 'id', width: 80 },
  { title: '版本号', dataIndex: 'version', slotName: 'version' },
  { title: '版本代码', dataIndex: 'version_code', width: 100 },
  { title: '更新内容', dataIndex: 'update_content' },
  { title: '更新类型', dataIndex: 'force_update', slotName: 'force_update' },
  { title: '状态', dataIndex: 'status', slotName: 'status' },
  { title: '创建时间', dataIndex: 'create_time', width: 160 },
  { title: '操作', slotName: 'actions', width: 180 }
]

const loadData = async () => {
  loading.value = true
  try {
    const res = await verApi.getList({ ...searchForm, page: pagination.current, size: pagination.pageSize })
    if (res.code === 200) {
      // 后端返回格式: { list, currentPage, pageTotal, dataTotal }
      tableData.value = res.data.list || []
      pagination.total = res.data.dataTotal || 0
    }
  } finally {
    loading.value = false
  }
}

const handleSearch = () => { pagination.current = 1; loadData() }
const handleReset = () => { Object.assign(searchForm, { version: '', status: undefined }); handleSearch() }
const handlePageChange = (page) => { pagination.current = page; loadData() }

const handleAdd = () => {
  Object.assign(form, { id: '', version: '', version_code: 100, update_content: '', download_url: '', force_update: false, status: 1 })
  modalVisible.value = true
}

const handleEdit = (record) => { Object.assign(form, record); modalVisible.value = true }

const handleSubmit = async () => {
  const valid = await formRef.value?.validate()
  if (valid) return
  try {
    const api = form.id ? verApi.edit : verApi.add
    const res = await api(form)
    if (res.code === 200) {
      Message.success(form.id ? '编辑成功' : '添加成功')
      modalVisible.value = false
      loadData()
    }
  } catch (e) { Message.error('操作失败') }
}

const handleCancel = () => { modalVisible.value = false }

const handleDelete = async (record) => {
  try {
    const res = await verApi.del(record.id)
    if (res.code === 200) { Message.success('删除成功'); loadData() }
  } catch (e) { Message.error('删除失败') }
}

const handleBatchDelete = async () => {
  try {
    const res = await verApi.delAll(selectedKeys.value)
    if (res.code === 200) { Message.success('删除成功'); selectedKeys.value = []; loadData() }
  } catch (e) { Message.error('删除失败') }
}

const handleDiscard = async (record) => {
  try {
    const res = await verApi.discard({ id: record.id, status: record.status === -1 ? 1 : -1 })
    if (res.code === 200) { Message.success('操作成功'); loadData() }
  } catch (e) { Message.error('操作失败') }
}

onMounted(() => { loadData() })
</script>

<script>
export default { name: 'VersionList' }
</script>

<style scoped>
.version-management { padding: 16px; }
</style>
