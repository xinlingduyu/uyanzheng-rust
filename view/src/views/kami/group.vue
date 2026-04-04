<template>
  <div class="p-5 bg-[--color-bg-1]">
    <div class="w-full mx-auto">
      <!-- 顶部操作栏 -->
      <div class="lg:flex items-center justify-between">
        <div class="w-full lg:w-auto">
          <a-button type="primary" class="mb-5" @click="handleAdd" :disabled="!auth('cdkGroup')">
            <template #icon><icon-plus /></template>
            添加卡密组
          </a-button>
        </div>
        <div class="w-full lg:w-auto">
          <a-form :model="searchForm" class="grow lg:!flex-row gap-x-5 justify-end" auto-label-width>
            <a-form-item field="keyword" hide-label class="w-auto">
              <a-input-search
                v-model="searchForm.keyword"
                placeholder="关键词搜索...."
                :loading="searchBtnLoading"
                @search="handleSearch"
                @press-enter="handleSearch"
                allow-clear
                @clear="handleSearch"
                class="lg:!w-[280px] w-full"
              />
            </a-form-item>
          </a-form>
        </div>
      </div>

      <!-- 数据表格 -->
      <a-table
        :columns="columns"
        :data="tableData"
        :loading="loading"
        :row-selection="rowSelection"
        v-model:selectedKeys="selectedKeys"
        row-key="id"
        :bordered="false"
        :pagination="false"
      >
        <template #val="{ record }">
          {{ formatVal(record) }}
        </template>
        <template #type="{ record }">
          <a-tag :color="getTypeColor(record.type)">
            {{ getTypeName(record.type) }}
          </a-tag>
        </template>
        <template #operate="{ record }">
          <div>
            <a-button type="text" size="small" @click="handleEdit(record)" :disabled="!auth('cdkGroup')">
              编辑
            </a-button>
            <a-popconfirm type="warning" position="tr" @before-ok="() => handleDelete(record.id)">
              <template #content>
                确认删除：{{ record.name }} ？
              </template>
              <a-button type="text" size="small" status="danger" :disabled="!auth('cdkGroup')">
                删除
              </a-button>
            </a-popconfirm>
          </div>
        </template>
      </a-table>

      <!-- 分页和批量操作 -->
      <div class="w-full md:flex items-center justify-between mt-4">
        <div class="mb-5 md:mb-0 text-center">
          <span v-if="selectedKeys.length > 0" class="text-gray-500 text-sm">
            <a-button type="text" size="small" status="danger" @click="handleBatchDelete" :loading="delSelectedLoading" :disabled="!auth('cdkGroup')">
              删除
            </a-button>
            当前选中的 {{ selectedKeys.length }} 条数据
          </span>
          <span v-else class="text-gray-500 text-sm">
            当前第 {{ pagination.current }} 页 共 {{ pagination.pageTotal }} 页 {{ pagination.total }} 条结果
          </span>
        </div>
        <div class="flex justify-center">
          <a-pagination
            :total="pagination.total"
            :current="pagination.current"
            :page-size="pagination.pageSize"
            @change="handlePageChange"
            @page-size-change="handlePageSizeChange"
            show-page-size
          />
        </div>
      </div>
    </div>

    <!-- 添加/编辑弹窗 -->
    <a-modal
      v-model:visible="modalVisible"
      :title="modalTitle"
      :width="400"
      :footer="false"
      title-align="start"
      :mask-closable="false"
    >
      <div class="md:w-80">
        <a-form :model="modalForm" auto-label-width @submit="handleSubmit">
          <a-form-item field="type" label="卡密组类型">
            <a-radio-group v-model="modalForm.type" :disabled="modalForm.typeDisabled">
              <a-radio value="vip">会员卡</a-radio>
              <a-radio value="fen">积分卡</a-radio>
              <a-radio value="addsn">设备增绑卡</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item field="name" label="卡密组名称">
            <a-input v-model="modalForm.name" placeholder="如：天卡" />
          </a-form-item>
          <a-form-item field="val" label="卡密组面值">
            <a-input-number
              v-model="modalForm.val"
              placeholder="1"
              :min="1"
              :max="9999999999"
              style="width: 100%"
            >
              <template #append>
                <!-- VIP类型显示单位选择 -->
                <template v-if="modalForm.type === 'vip'">
                  <a-select v-model="modalForm.vipType" style="width: 75px">
                    <a-option value="s">秒</a-option>
                    <a-option value="i">分</a-option>
                    <a-option value="h">时</a-option>
                    <a-option value="d">天</a-option>
                    <a-option value="yj" @click="setPermanent">永久</a-option>
                  </a-select>
                </template>
                <!-- 非VIP类型显示固定单位 -->
                <template v-else>
                  <span>{{ modalForm.type === 'fen' ? '积分' : '台' }}</span>
                </template>
              </template>
            </a-input-number>
          </a-form-item>
          <a-form-item field="price" label="卡密组定价">
            <a-input-number
              v-model="modalForm.price"
              placeholder="1.00"
              :min="0"
              :precision="2"
              style="width: 100%"
            >
              <template #append>元</template>
            </a-input-number>
          </a-form-item>
          <a-space direction="vertical" fill>
            <a-button type="primary" html-type="submit" :loading="modalBtnLoading" long>
              提交
            </a-button>
          </a-space>
        </a-form>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { Message } from '@arco-design/web-vue'
import { auth } from '@/utils/common'
import cdkGroupApi from '@/api/system/cdkGroup'
import { parseVipTime, toSeconds, formatVipTime } from '@/utils/sun.js'

// 表格配置
const columns = [
  { title: 'ID', dataIndex: 'id', width: 60, align: 'center' },
  { title: '组名称', dataIndex: 'name' },
  { title: '定价', dataIndex: 'price', width: 100, align: 'center' },
  { title: '面值', slotName: 'val', width: 100, align: 'center' },
  { title: '类型', slotName: 'type', width: 100, align: 'center' },
  { title: '操作', slotName: 'operate', width: 100, align: 'center' }
]

// 表格数据
const tableData = ref([])
const loading = ref(false)
const selectedKeys = ref([])
const delSelectedLoading = ref(false)

const rowSelection = {
  type: 'checkbox',
  showCheckedAll: true,
  onlyCurrent: false
}

// 分页
const pagination = reactive({
  current: 1,
  pageSize: 10,
  total: 0,
  pageTotal: 0
})

// 搜索
const searchForm = reactive({
  keyword: ''
})
const searchBtnLoading = ref(false)

// 弹窗
const modalVisible = ref(false)
const modalBtnLoading = ref(false)
const modalTitle = computed(() => modalForm.id ? '编辑卡密组' : '添加卡密组')
const modalForm = reactive({
  id: '',
  name: '',
  type: 'vip',
  val: undefined,
  price: undefined,
  vipType: 'd',
  typeDisabled: false
})

// 获取类型名称
const getTypeName = (type) => {
  const map = {
    'vip': '会员卡',
    'fen': '积分卡',
    'addsn': '设备增绑卡'
  }
  return map[type] || type
}

// 获取类型颜色
const getTypeColor = (type) => {
  const map = {
    'vip': 'red',
    'fen': 'orange',
    'addsn': ''
  }
  return map[type] || ''
}

// 格式化面值显示
const formatVal = (record) => {
  if (record.type === 'vip') {
    return formatVipTime(Number(record.val))
  }
  return record.val + (record.type === 'fen' ? '积分' : '台')
}

// 加载数据
const loadData = async () => {
  loading.value = true
  try {
    const res = await cdkGroupApi.get(pagination.current, pagination.pageSize, searchForm)
    if (res.code !== 200) {
      Message.error(res.msg)
      return
    }
    tableData.value = res.data.list
    pagination.current = res.data.currentPage
    pagination.total = res.data.dataTotal
    pagination.pageTotal = res.data.pageTotal
  } catch (e) {
    Message.error('出错了：' + e)
  } finally {
    loading.value = false
    searchBtnLoading.value = false
  }
}

// 搜索
const handleSearch = () => {
  searchBtnLoading.value = true
  pagination.current = 1
  loadData()
}

// 分页
const handlePageChange = (page) => {
  pagination.current = page
  loading.value = true
  loadData()
}

const handlePageSizeChange = (size) => {
  pagination.pageSize = size
  loading.value = true
  loadData()
}

// 清空表单
const getEmptyForm = () => ({
  id: '',
  name: '',
  type: 'vip',
  val: undefined,
  price: undefined,
  vipType: 'd',
  typeDisabled: false
})

// 添加
const handleAdd = () => {
  modalTitle.value = '添加卡密组'
  Object.assign(modalForm, getEmptyForm())
  modalForm.typeDisabled = false
  modalVisible.value = true
}

// 编辑
const handleEdit = (record) => {
  modalForm.id = record.id
  modalForm.name = record.name
  modalForm.type = record.type
  modalForm.typeDisabled = true // 编辑时类型不可修改
  modalForm.price = parseFloat(record.price)
  
  if (record.type === 'vip') {
    // 解析秒数为值和单位
    modalForm.val = parseVipTime(Number(record.val), 1)
    modalForm.vipType = parseVipTime(Number(record.val), 2)
  } else {
    modalForm.val = Number(record.val)
    modalForm.vipType = 'd'
  }
  
  modalVisible.value = true
}

// 设置永久
const setPermanent = () => {
  modalForm.val = 9999999999
  modalForm.vipType = 's'
}

// 提交
const handleSubmit = async () => {
  modalBtnLoading.value = true
  try {
    const res = await cdkGroupApi.submit(modalForm)
    if (res.code !== 200) {
      Message.error(res.msg)
      return
    }
    modalVisible.value = false
    Message.success(res.msg)
    loadData()
  } catch (e) {
    Message.error('出错了：' + e)
  } finally {
    modalBtnLoading.value = false
  }
}

// 删除
const handleDelete = async (id) => {
  try {
    const res = await cdkGroupApi.del(id)
    if (res.code !== 200) {
      Message.error(res.msg)
      return false
    }
    Message.success(res.msg)
    loadData()
    return true
  } catch (e) {
    Message.error('出错了：' + e)
    return false
  }
}

// 批量删除
const handleBatchDelete = async () => {
  delSelectedLoading.value = true
  try {
    const res = await cdkGroupApi.delAll(selectedKeys.value)
    if (res.code !== 200) {
      Message.error(res.msg)
      return
    }
    selectedKeys.value = []
    Message.success(res.msg)
    loadData()
  } catch (e) {
    Message.error('出错了：' + e)
  } finally {
    delSelectedLoading.value = false
  }
}

onMounted(() => {
  loadData()
})
</script>

<script>
export default { name: 'KamiGroup' }
</script>