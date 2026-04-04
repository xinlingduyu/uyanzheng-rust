<template>
  <div class="encryption-management">
    <!-- 操作栏 -->
    <a-card class="mb-4" :bordered="false">
      <a-space>
        <a-button type="primary" @click="handleAdd">
          <template #icon><icon-plus /></template>
          添加方案
        </a-button>
      </a-space>
    </a-card>

    <!-- 数据表格 -->
    <a-card :bordered="false">
      <a-table :columns="columns" :data="tableData" :loading="loading" row-key="id">
        <template #type="{ record }">
          <a-tag :color="getTypeColor(record.type)">{{ record.type.toUpperCase() }}</a-tag>
        </template>
        <template #status="{ record }">
          <a-switch v-model="record.status" :checked-value="1" :unchecked-value="0" @change="handleStatusChange(record)" />
        </template>
        <template #actions="{ record }">
          <a-space>
            <a-button type="text" size="small" @click="handleEdit(record)">编辑</a-button>
            <a-popconfirm content="确定删除该方案吗？" @ok="handleDelete(record)">
              <a-button type="text" size="small" status="danger">删除</a-button>
            </a-popconfirm>
          </a-space>
        </template>
      </a-table>
    </a-card>

    <!-- 编辑弹窗 -->
    <a-modal v-model:visible="modalVisible" :title="modalTitle" :width="520" @ok="handleSubmit" @cancel="handleCancel">
      <a-form ref="formRef" :model="form" :rules="rules" layout="vertical">
        <a-form-item field="name" label="方案名称" required>
          <a-input v-model="form.name" placeholder="请输入方案名称" />
        </a-form-item>
        <a-form-item field="type" label="加密类型" required>
          <a-select v-model="form.type" placeholder="请选择加密类型">
            <a-option v-for="p in plugins" :key="p.id" :value="p.id">{{ p.name }}</a-option>
          </a-select>
        </a-form-item>
        <a-form-item field="key" label="加密密钥">
          <a-input-password v-model="form.key" placeholder="请输入加密密钥" />
        </a-form-item>
        <a-form-item field="status" label="状态">
          <a-radio-group v-model="form.status">
            <a-radio :value="1">启用</a-radio>
            <a-radio :value="0">禁用</a-radio>
          </a-radio-group>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { Message } from '@arco-design/web-vue'
import encryptionApi from '@/api/system/encryption'

const tableData = ref([])
const loading = ref(false)
const plugins = ref([])

const modalVisible = ref(false)
const modalTitle = computed(() => form.id ? '编辑方案' : '添加方案')
const formRef = ref(null)
const form = reactive({ id: '', name: '', type: '', key: '', status: 1 })

const rules = {
  name: [{ required: true, message: '请输入方案名称' }],
  type: [{ required: true, message: '请选择加密类型' }]
}

const columns = [
  { title: 'ID', dataIndex: 'id', width: 80 },
  { title: '方案名称', dataIndex: 'name' },
  { title: '加密类型', dataIndex: 'type', slotName: 'type' },
  { title: '状态', dataIndex: 'status', slotName: 'status', width: 80 },
  { title: '创建时间', dataIndex: 'create_time', width: 160 },
  { title: '操作', slotName: 'actions', width: 120 }
]

const getTypeColor = (type) => {
  const colors = { rc4: 'blue', aes: 'green', des: 'orange', rsa: 'purple' }
  return colors[type] || 'gray'
}

const loadPlugins = async () => {
  try {
    const res = await encryptionApi.getPlugins()
    if (res.code === 200) plugins.value = res.data
  } catch (e) {}
}

const loadData = async () => {
  loading.value = true
  try {
    const res = await encryptionApi.getList({ page: 1, size: 100 })
    if (res.code === 200) {
      // 后端返回格式: { list, currentPage, pageTotal, dataTotal }
      tableData.value = res.data.list || []
    }
  } finally {
    loading.value = false
  }
}

const handleAdd = () => {
  Object.assign(form, { id: '', name: '', type: '', key: '', status: 1 })
  modalVisible.value = true
}

const handleEdit = (record) => { Object.assign(form, record); modalVisible.value = true }

const handleSubmit = async () => {
  const valid = await formRef.value?.validate()
  if (valid) return
  try {
    const api = form.id ? encryptionApi.edit : encryptionApi.add
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
    const res = await encryptionApi.del(record.id)
    if (res.code === 200) { Message.success('删除成功'); loadData() }
  } catch (e) { Message.error('删除失败') }
}

const handleStatusChange = async (record) => {
  try {
    await encryptionApi.edit({ id: record.id, status: record.status })
    Message.success('状态更新成功')
  } catch (e) {
    record.status = record.status === 1 ? 0 : 1
    Message.error('操作失败')
  }
}

onMounted(() => { loadPlugins(); loadData() })
</script>

<script>
export default { name: 'EncryptionList' }
</script>

<style scoped>
.encryption-management { padding: 16px; }
</style>
