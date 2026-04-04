<template>
  <div class="fen-management">
    <!-- 标签页 -->
    <a-tabs v-model:active-key="activeTab">
      <a-tab-pane key="event" title="积分事件">
        <a-card class="mb-4" :bordered="false">
          <a-button type="primary" @click="handleAddEvent">
            <template #icon><icon-plus /></template>
            添加事件
          </a-button>
        </a-card>
        <a-card :bordered="false">
          <a-table :columns="eventColumns" :data="eventData" :loading="eventLoading" row-key="id">
            <template #type="{ record }">
              <a-tag :color="record.type === 1 ? 'green' : 'red'">
                {{ record.type === 1 ? '增加' : '消耗' }}
              </a-tag>
            </template>
            <template #status="{ record }">
              <a-switch v-model="record.status" :checked-value="1" :unchecked-value="0" @change="handleEventStatusChange(record)" />
            </template>
            <template #actions="{ record }">
              <a-space>
                <a-button type="text" size="small" @click="handleEditEvent(record)">编辑</a-button>
                <a-popconfirm content="确定删除吗？" @ok="handleDeleteEvent(record)">
                  <a-button type="text" size="small" status="danger">删除</a-button>
                </a-popconfirm>
              </a-space>
            </template>
          </a-table>
        </a-card>
      </a-tab-pane>

      <a-tab-pane key="order" title="积分订单">
        <a-card :bordered="false">
          <a-table :columns="orderColumns" :data="orderData" :loading="orderLoading" :pagination="orderPagination" row-key="id" @page-change="handleOrderPageChange">
            <template #type="{ record }">
              <a-tag :color="record.type === 1 ? 'green' : 'red'">
                {{ record.type === 1 ? '收入' : '支出' }}
              </a-tag>
            </template>
            <template #amount="{ record }">
              <span :class="record.type === 1 ? 'text-green-500' : 'text-red-500'">
                {{ record.type === 1 ? '+' : '-' }}{{ record.amount }}
              </span>
            </template>
          </a-table>
        </a-card>
      </a-tab-pane>
    </a-tabs>

    <!-- 事件编辑弹窗 -->
    <a-modal v-model:visible="eventModalVisible" :title="eventModalTitle" :width="520" @ok="handleEventSubmit">
      <a-form ref="eventFormRef" :model="eventForm" :rules="eventRules" layout="vertical">
        <a-form-item field="name" label="事件名称" required>
          <a-input v-model="eventForm.name" placeholder="请输入事件名称" />
        </a-form-item>
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item field="type" label="类型">
              <a-select v-model="eventForm.type">
                <a-option :value="1">增加积分</a-option>
                <a-option :value="2">消耗积分</a-option>
              </a-select>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item field="amount" label="积分数量">
              <a-input-number v-model="eventForm.amount" :min="0" style="width: 100%" />
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item field="description" label="描述">
          <a-textarea v-model="eventForm.description" placeholder="请输入描述" />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { Message } from '@arco-design/web-vue'
import fenApi from '@/api/system/fen'

const activeTab = ref('event')

// 积分事件
const eventData = ref([])
const eventLoading = ref(false)
const eventModalVisible = ref(false)
const eventModalTitle = computed(() => eventForm.id ? '编辑事件' : '添加事件')
const eventFormRef = ref(null)
const eventForm = reactive({ id: '', name: '', type: 1, amount: 0, description: '' })
const eventRules = { name: [{ required: true, message: '请输入事件名称' }] }

const eventColumns = [
  { title: 'ID', dataIndex: 'id', width: 80 },
  { title: '事件名称', dataIndex: 'name' },
  { title: '类型', dataIndex: 'type', slotName: 'type' },
  { title: '积分数量', dataIndex: 'amount' },
  { title: '描述', dataIndex: 'description' },
  { title: '状态', dataIndex: 'status', slotName: 'status', width: 80 },
  { title: '操作', slotName: 'actions', width: 120 }
]

// 积分订单
const orderData = ref([])
const orderLoading = ref(false)
const orderPagination = reactive({ current: 1, pageSize: 20, total: 0, showTotal: true })

const orderColumns = [
  { title: 'ID', dataIndex: 'id', width: 80 },
  { title: '用户', dataIndex: 'username' },
  { title: '类型', dataIndex: 'type', slotName: 'type' },
  { title: '积分', dataIndex: 'amount', slotName: 'amount' },
  { title: '事件', dataIndex: 'event_name' },
  { title: '余额', dataIndex: 'balance' },
  { title: '时间', dataIndex: 'create_time', width: 160 }
]

const loadEventData = async () => {
  eventLoading.value = true
  try {
    const res = await fenApi.getEventList({ page: 1, size: 100 })
    if (res.code === 200) {
      // 后端返回格式: { list, currentPage, pageTotal, dataTotal }
      eventData.value = res.data.list || []
    }
  } finally {
    eventLoading.value = false
  }
}

const loadOrderData = async () => {
  orderLoading.value = true
  try {
    const res = await fenApi.getOrderList({ page: orderPagination.current, size: orderPagination.pageSize })
    if (res.code === 200) {
      // 后端返回格式: { list, currentPage, pageTotal, dataTotal }
      orderData.value = res.data.list || []
      orderPagination.total = res.data.dataTotal || 0
    }
  } finally {
    orderLoading.value = false
  }
}

const handleAddEvent = () => {
  Object.assign(eventForm, { id: '', name: '', type: 1, amount: 0, description: '' })
  eventModalVisible.value = true
}

const handleEditEvent = (record) => { Object.assign(eventForm, record); eventModalVisible.value = true }

const handleEventSubmit = async () => {
  const valid = await eventFormRef.value?.validate()
  if (valid) return
  try {
    const api = eventForm.id ? fenApi.editEvent : fenApi.addEvent
    const res = await api(eventForm)
    if (res.code === 200) { Message.success('操作成功'); eventModalVisible.value = false; loadEventData() }
  } catch (e) { Message.error('操作失败') }
}

const handleEventStatusChange = async (record) => {
  try {
    await fenApi.editEventState({ id: record.id, status: record.status })
    Message.success('状态更新成功')
  } catch (e) { record.status = record.status === 1 ? 0 : 1; Message.error('操作失败') }
}

const handleDeleteEvent = async (record) => {
  try {
    const res = await fenApi.delEvent(record.id)
    if (res.code === 200) { Message.success('删除成功'); loadEventData() }
  } catch (e) { Message.error('删除失败') }
}

const handleOrderPageChange = (page) => { orderPagination.current = page; loadOrderData() }

onMounted(() => { loadEventData(); loadOrderData() })
</script>

<script>
export default { name: 'FenManagement' }
</script>

<style scoped>
.fen-management { padding: 16px; }
</style>
