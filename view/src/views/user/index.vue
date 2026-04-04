<template>
  <div class="user-management">
    <!-- 统计卡片 -->
    <div class="stats-row">
      <div class="stat-card">
        <div class="stat-icon stat-icon-primary"><IconUserGroup /></div>
        <div class="stat-info">
          <span class="stat-value">{{ stats.total }}</span>
          <span class="stat-label">总用户</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-icon-success"><IconUser /></div>
        <div class="stat-info">
          <span class="stat-value">{{ stats.active }}</span>
          <span class="stat-label">活跃用户</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-icon-warning"><IconStar /></div>
        <div class="stat-info">
          <span class="stat-value">{{ stats.vip }}</span>
          <span class="stat-label">VIP会员</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon stat-icon-danger"><IconStop /></div>
        <div class="stat-info">
          <span class="stat-value">{{ stats.banned }}</span>
          <span class="stat-label">禁用用户</span>
        </div>
      </div>
    </div>

    <!-- 搜索栏 -->
    <div class="search-card">
      <div class="search-row">
        <div class="search-input-group">
          <a-select v-model="searchForm.keywordType" class="search-type">
            <a-option value="id">用户ID</a-option>
            <a-option value="acctno">账号</a-option>
            <a-option value="phone">手机号</a-option>
            <a-option value="email">邮箱</a-option>
            <a-option value="name">昵称</a-option>
            <a-option value="reg_ip">IP</a-option>
            <a-option value="reg_sn">机器码</a-option>
          </a-select>
          <a-input v-model="searchForm.keyword" :placeholder="keywordPlaceholder" allow-clear class="search-input" @press-enter="handleSearch">
            <template #prefix><IconSearch /></template>
          </a-input>
        </div>
        <div class="search-filters">
          <a-select v-model="searchForm.status" placeholder="状态" allow-clear class="filter-select" @change="handleSearch">
            <a-option value="">全部状态</a-option>
            <a-option value="n">已禁用</a-option>
          </a-select>
          <a-select v-model="searchForm.ug" placeholder="等级" allow-clear class="filter-select" @change="handleSearch">
            <a-option value="">全部等级</a-option>
            <a-option value="1">普通用户</a-option>
            <a-option value="2">VIP会员</a-option>
            <a-option value="3">永久会员</a-option>
          </a-select>
          <a-button type="primary" @click="handleSearch">
            <template #icon><IconSearch /></template>
            搜索
          </a-button>
        </div>
      </div>
    </div>

    <!-- 操作栏 -->
    <div class="action-bar">
      <div class="action-left">
        <a-button type="primary" @click="handleAdd">
          <template #icon><IconPlus /></template>
          添加用户
        </a-button>
        <a-button @click="handleAward">
          <template #icon><sa-icon icon="mdi:gift" :size="18" /></template>
          发送奖励
        </a-button>
      </div>
      <div class="action-right">
        <a-button v-if="selectedKeys.length > 0" status="danger" @click="handleBatchDelete">
          <template #icon><IconDelete /></template>
          删除选中 ({{ selectedKeys.length }})
        </a-button>
      </div>
    </div>

    <!-- 数据表格 -->
    <div class="table-card">
      <a-table
        :columns="columns"
        :data="tableData"
        :loading="loading"
        :pagination="false"
        :row-selection="rowSelection"
        v-model:selectedKeys="selectedKeys"
        row-key="id"
        :bordered="false"
        class="user-table"
      >
        <template #id="{ record }">
          <span class="user-id">#{{ record.id }}</span>
        </template>
        <template #account="{ record }">
          <div class="account-cell">
            <a-badge :count="record.online ? 1 : 0" dot :dot-style="{ background: '#52c41a' }">
              <a-avatar :size="40" class="user-avatar">
                <img v-if="record.avatars" :src="record.avatars" />
                <IconUser v-else />
              </a-avatar>
            </a-badge>
            <div class="account-info">
              <span class="account-primary">{{ record.phone || record.email || record.acctno || '-' }}</span>
              <span class="account-secondary">{{ record.nickname || '匿名用户' }}</span>
            </div>
          </div>
        </template>
        <template #vip="{ record }">
          <div class="vip-cell">
            <a-tag v-if="record.ug === 3 || record.vip >= 9999999999" color="gold" size="small">
              <template #icon><sa-icon icon="mdi:crown" :size="14" /></template>
              永久
            </a-tag>
            <a-tag v-else-if="record.vip > now && record.ug === 2" color="arcoblue" size="small">
              <template #icon><IconStar /></template>
              VIP
            </a-tag>
            <span v-else class="vip-none">未开通</span>
          </div>
        </template>
        <template #fen="{ record }">
          <span class="fen-value">{{ record.fen || 0 }}</span>
        </template>
        <template #last_login="{ record }">
          <div v-if="record.last_login_info" class="time-cell">
            <span class="time-main">{{ formatTime(record.last_login_info.time) }}</span>
            <span class="time-sub">{{ record.last_login_info.ip }}</span>
          </div>
          <span v-else class="time-empty">近期未登录</span>
        </template>
        <template #reg_time="{ record }">
          <div class="time-cell">
            <span class="time-main">{{ formatTime(record.reg_time) }}</span>
            <span class="time-sub">{{ record.reg_ip }}</span>
          </div>
        </template>
        <template #ban="{ record }">
          <a-tag :color="record.ban > now ? 'red' : 'green'" size="small" class="status-tag">
            {{ record.ban > now ? '禁用' : '正常' }}
          </a-tag>
        </template>
        <template #actions="{ record }">
          <a-space class="action-buttons">
            <a-button type="text" size="small" @click="handleEdit(record)">
              <template #icon><IconEdit /></template>
            </a-button>
            <a-popconfirm content="确认删除该用户？" @ok="handleDelete(record)">
              <a-button type="text" size="small" status="danger">
                <template #icon><IconDelete /></template>
              </a-button>
            </a-popconfirm>
          </a-space>
        </template>
      </a-table>

      <!-- 分页 -->
      <div class="pagination-bar">
        <span class="pagination-info">
          共 <strong>{{ pagination.total }}</strong> 条记录
        </span>
        <a-pagination
          v-model:current="pagination.current"
          v-model:page-size="pagination.pageSize"
          :total="pagination.total"
          show-total
          show-jumper
          show-page-size
          :page-size-options="[10, 20, 50, 100]"
          @change="handlePageChange"
          @page-size-change="handlePageSizeChange"
        />
      </div>
    </div>

    <!-- 添加用户弹窗 -->
    <a-modal v-model:visible="addModalVisible" :footer="false" :width="380" class="custom-modal">
      <template #title>
        <div class="modal-title">
          <sa-icon icon="mdi:account-plus" :size="20" class="modal-icon" />
          添加用户
        </div>
      </template>
      <a-form :model="addForm" layout="vertical" @submit="handleAddSubmit" class="modal-form">
        <a-form-item label="账号" class="form-item-clean">
          <a-input v-model="addForm.acctno" placeholder="请设置账号 5~12 位" allow-clear size="large">
            <template #prefix><IconUser /></template>
          </a-input>
        </a-form-item>
        <a-form-item label="密码" class="form-item-clean">
          <a-input-password v-model="addForm.password" placeholder="请输入密码 6~18 位" allow-clear size="large">
            <template #prefix><IconLock /></template>
            <template #suffix>
              <a-tooltip content="随机生成">
                <IconRefresh class="action-icon" @click="generatePassword" />
              </a-tooltip>
            </template>
          </a-input-password>
        </a-form-item>
        <a-button type="primary" :loading="addLoading" long html-type="submit" size="large">
          确认添加
        </a-button>
      </a-form>
    </a-modal>

    <!-- 用户奖励弹窗 -->
    <a-modal v-model:visible="awardModalVisible" :footer="false" :width="400" class="custom-modal">
      <template #title>
        <div class="modal-title">
          <sa-icon icon="mdi:gift" :size="20" class="modal-icon" />
          发送奖励
        </div>
      </template>
      <a-form :model="awardForm" layout="vertical" @submit="handleAwardSubmit" class="modal-form">
        <a-form-item label="奖励类型" class="form-item-clean">
          <a-radio-group v-model="awardForm.type" type="button">
            <a-radio value="vip">
              <sa-icon icon="mdi:crown" :size="16" class="mr-1" /> 会员时长
            </a-radio>
            <a-radio value="fen">
              <IconStar class="mr-1" /> 积分
            </a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item :label="awardForm.type === 'fen' ? '积分数量' : '会员时长'" class="form-item-clean">
          <a-input-number v-model="awardForm.val" placeholder="1" :min="1" style="width: 100%" size="large">
            <template v-if="awardForm.type === 'vip'" #append>
              <a-select v-model="awardForm.vipType" style="width: 80px">
                <a-option value="d">天</a-option>
                <a-option value="h">时</a-option>
                <a-option value="i">分</a-option>
                <a-option value="s">秒</a-option>
              </a-select>
            </template>
            <template v-else #append>
              <span>积分</span>
            </template>
          </a-input-number>
        </a-form-item>
        <a-form-item label="奖励对象" class="form-item-clean">
          <a-select v-model="awardForm.object" size="large">
            <a-option value="all">所有用户</a-option>
            <a-option value="vip">仅VIP会员</a-option>
          </a-select>
        </a-form-item>
        <a-button type="primary" :loading="awardLoading" long html-type="submit" size="large">
          确认发送
        </a-button>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Message } from '@arco-design/web-vue'
import dayjs from 'dayjs'
import userApi from '@/api/system/userMgmt'

const router = useRouter()
const now = dayjs().unix()

const keywordPlaceholderMap = {
  id: '输入用户ID',
  acctno: '输入用户账号',
  phone: '输入手机号',
  email: '输入邮箱',
  name: '输入昵称',
  reg_ip: '输入注册IP',
  reg_sn: '输入机器码'
}

const searchForm = reactive({
  keyword: '',
  keywordType: 'acctno',
  status: undefined,
  ug: undefined
})

const keywordPlaceholder = computed(() => keywordPlaceholderMap[searchForm.keywordType] || '')

const tableData = ref([])
const loading = ref(false)
const pagination = reactive({ current: 1, pageSize: 20, total: 0 })

const stats = computed(() => {
  const total = pagination.total
  const active = tableData.value.filter(u => u.online).length
  const vip = tableData.value.filter(u => u.ug >= 2 || u.vip > now).length
  const banned = tableData.value.filter(u => u.ban > now).length
  return { total, active, vip, banned }
})

const selectedKeys = ref([])
const rowSelection = reactive({ type: 'checkbox', showCheckedAll: true, onlyCurrent: true })

const addModalVisible = ref(false)
const addLoading = ref(false)
const addForm = reactive({ acctno: '', password: '' })

const awardModalVisible = ref(false)
const awardLoading = ref(false)
const awardForm = reactive({ type: 'vip', object: 'all', val: 1, vipType: 'd' })

const columns = [
  { title: 'ID', slotName: 'id', width: 90, align: 'center' },
  { title: '账号信息', slotName: 'account', minWidth: 200 },
  { title: '会员', slotName: 'vip', width: 100, align: 'center' },
  { title: '积分', slotName: 'fen', width: 80, align: 'center' },
  { title: '最后登录', slotName: 'last_login', width: 150 },
  { title: '注册信息', slotName: 'reg_time', width: 150 },
  { title: '状态', slotName: 'ban', width: 80, align: 'center' },
  { title: '操作', slotName: 'actions', width: 100, align: 'center', fixed: 'right' }
]

const formatTime = (timestamp) => {
  if (!timestamp) return '-'
  return dayjs.unix(timestamp).format('MM-DD HH:mm')
}

const generatePassword = () => {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789'
  let password = ''
  for (let i = 0; i < 10; i++) {
    password += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  addForm.password = password
}

const loadData = async () => {
  loading.value = true
  try {
    const res = await userApi.getList({
      keyword: searchForm.keyword,
      keywordType: searchForm.keywordType,
      status: searchForm.status,
      ug: searchForm.ug,
      page: pagination.current,
      size: pagination.pageSize
    })
    if (res.code === 200) {
      tableData.value = res.data.list || []
      pagination.total = res.data.dataTotal || 0
    }
  } catch (e) {
    Message.error('加载数据失败')
  } finally {
    loading.value = false
  }
}

const handleSearch = () => { pagination.current = 1; loadData() }
const handlePageChange = (page) => { pagination.current = page; loadData() }
const handlePageSizeChange = (size) => { pagination.pageSize = size; pagination.current = 1; loadData() }

const handleAdd = () => {
  addForm.acctno = ''
  addForm.password = ''
  generatePassword()
  addModalVisible.value = true
}

const handleAddSubmit = async () => {
  if (!addForm.acctno || !addForm.password) {
    Message.warning('请填写账号和密码')
    return
  }
  addLoading.value = true
  try {
    const res = await userApi.add(addForm)
    addLoading.value = false
    if (res.code === 200) {
      Message.success('添加成功')
      addModalVisible.value = false
      loadData()
    } else {
      Message.error(res.msg || '添加失败')
    }
  } catch (e) {
    addLoading.value = false
    Message.error('操作失败')
  }
}

const handleEdit = (record) => {
  router.push({ name: 'UserEdit', params: { uid: record.id } })
}

const handleDelete = async (record) => {
  try {
    const res = await userApi.del(record.id)
    if (res.code === 200) {
      Message.success('删除成功')
      loadData()
    } else {
      Message.error(res.msg || '删除失败')
    }
  } catch (e) {
    Message.error('删除失败')
  }
}

const handleBatchDelete = async () => {
  try {
    const res = await userApi.delAll(selectedKeys.value)
    if (res.code === 200) {
      Message.success('删除成功')
      selectedKeys.value = []
      loadData()
    } else {
      Message.error(res.msg || '删除失败')
    }
  } catch (e) {
    Message.error('删除失败')
  }
}

const handleAward = () => {
  awardForm.type = 'vip'
  awardForm.object = 'all'
  awardForm.val = 1
  awardForm.vipType = 'd'
  awardModalVisible.value = true
}

const handleAwardSubmit = async () => {
  if (!awardForm.val) {
    Message.warning('请填写奖励值')
    return
  }
  awardLoading.value = true
  try {
    const res = await userApi.award(awardForm)
    awardLoading.value = false
    if (res.code === 200) {
      Message.success('奖励发送成功')
      awardModalVisible.value = false
      loadData()
    } else {
      Message.error(res.msg || '奖励失败')
    }
  } catch (e) {
    awardLoading.value = false
    Message.error('操作失败')
  }
}

onMounted(() => loadData())
</script>

<script>
export default { name: 'UserList' }
</script>

<style scoped>
.user-management {
  padding: 16px;
  max-width: 1400px;
  margin: 0 auto;
}

/* 统计卡片 */
.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: var(--color-bg-2);
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  transition: transform 0.2s, box-shadow 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  font-size: 24px;
}

.stat-icon-primary { background: rgba(var(--primary-6), 0.1); color: rgb(var(--primary-6)); }
.stat-icon-success { background: rgba(var(--success-6), 0.1); color: rgb(var(--success-6)); }
.stat-icon-warning { background: rgba(var(--warning-6), 0.1); color: rgb(var(--warning-6)); }
.stat-icon-danger { background: rgba(var(--danger-6), 0.1); color: rgb(var(--danger-6)); }

.stat-info {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text-1);
}

.stat-label {
  font-size: 13px;
  color: var(--color-text-3);
}

/* 搜索卡片 */
.search-card {
  background: var(--color-bg-2);
  border-radius: 12px;
  padding: 16px 20px;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.search-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.search-input-group {
  display: flex;
  gap: 0;
  flex: 1;
  max-width: 450px;
}

.search-type {
  width: 100px;
}

.search-type :deep(.arco-select-view) {
  border-radius: 8px 0 0 8px;
  border-right: none;
}

.search-input {
  flex: 1;
}

.search-input :deep(.arco-input-wrapper) {
  border-radius: 0 8px 8px 0;
}

.search-filters {
  display: flex;
  gap: 12px;
}

.filter-select {
  width: 120px;
}

/* 操作栏 */
.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.action-left, .action-right {
  display: flex;
  gap: 12px;
}

/* 表格卡片 */
.table-card {
  background: var(--color-bg-2);
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  overflow: hidden;
}

/* 表格透明化 - 全面覆盖 */
.table-card :deep(.arco-table),
.table-card :deep(.arco-table-container),
.table-card :deep(.arco-table-th),
.table-card :deep(.arco-table-td),
.table-card :deep(.arco-table-tr),
.table-card :deep(.arco-table-header),
.table-card :deep(.arco-table-body),
.table-card :deep(.arco-table-content),
.table-card :deep(.arco-table-element),
.table-card :deep(.arco-table-tr-empty),
.table-card :deep(.arco-table-td-empty) {
  background: transparent !important;
}

/* 表格行hover状态 */
.table-card :deep(.arco-table-tr:hover .arco-table-td),
.table-card :deep(.arco-table-tr:not(.arco-table-tr-empty):hover .arco-table-td) {
  background: rgba(var(--primary-6), 0.05) !important;
}

/* 表格行选中状态 */
.table-card :deep(.arco-table-tr-checked .arco-table-td),
.table-card :deep(.arco-table-tr.arco-table-tr-checked .arco-table-td),
.table-card :deep(.arco-table-tr-selected .arco-table-td),
.table-card :deep(.arco-table-tr.arco-table-tr-selected .arco-table-td) {
  background: rgba(var(--primary-6), 0.08) !important;
}

/* 表头 */
.table-card :deep(.arco-table-th) {
  background: rgba(var(--primary-6), 0.03) !important;
}

/* 表头容器 */
.table-card :deep(.arco-table-header),
.table-card :deep(.arco-table-thead),
.table-card :deep(.arco-table-header-wrap) {
  background: transparent !important;
}

/* 分页栏 */
.table-card :deep(.pagination-bar),
.pagination-bar {
  background: transparent !important;
}

/* 复选框透明 */
.table-card :deep(.arco-checkbox),
.table-card :deep(.arco-checkbox-wrapper),
.table-card :deep(.arco-checkbox-mask) {
  background: transparent !important;
}

/* 操作按钮区域 */
.table-card :deep(.arco-space),
.table-card :deep(.arco-space-item) {
  background: transparent !important;
}

.user-table {
  padding: 0 4px;
}

.user-id {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 13px;
  color: var(--color-text-2);
  background: var(--color-fill-1);
  padding: 4px 8px;
  border-radius: 4px;
}

.account-cell {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-avatar {
  background: var(--color-fill-2);
  color: var(--color-text-3);
  flex-shrink: 0;
}

.account-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.account-primary {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.account-secondary {
  font-size: 12px;
  color: var(--color-text-3);
}

.vip-cell {
  display: flex;
  justify-content: center;
}

.vip-none {
  font-size: 12px;
  color: var(--color-text-4);
}

.fen-value {
  font-weight: 500;
  color: rgb(var(--warning-6));
}

.time-cell {
  display: flex;
  flex-direction: column;
  line-height: 1.4;
}

.time-main {
  font-size: 13px;
  color: var(--color-text-1);
}

.time-sub {
  font-size: 11px;
  color: var(--color-text-4);
}

.time-empty {
  font-size: 12px;
  color: var(--color-text-4);
}

.status-tag {
  min-width: 48px;
  justify-content: center;
}

.action-buttons {
  opacity: 0.6;
  transition: opacity 0.2s;
}

.action-buttons:hover {
  opacity: 1;
}

/* 分页 */
.pagination-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border-1);
}

.pagination-info {
  font-size: 13px;
  color: var(--color-text-3);
}

.pagination-info strong {
  color: var(--color-text-1);
}

/* 弹窗 */
.custom-modal :deep(.arco-modal-header) {
  border-bottom: none;
  padding-bottom: 0;
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
}

.modal-icon {
  color: rgb(var(--primary-6));
}

.modal-form {
  padding-top: 8px;
}

.form-item-clean {
  margin-bottom: 20px;
}

.action-icon {
  cursor: pointer;
  color: var(--color-text-3);
  transition: color 0.2s;
}

.action-icon:hover {
  color: rgb(var(--primary-6));
}

/* 响应式 */
@media (max-width: 1200px) {
  .stats-row {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .stats-row {
    grid-template-columns: 1fr;
  }

  .search-row {
    flex-direction: column;
    align-items: stretch;
  }

  .search-input-group {
    max-width: none;
  }

  .search-filters {
    flex-wrap: wrap;
  }

  .action-bar {
    flex-direction: column;
    gap: 12px;
  }

  .action-left, .action-right {
    width: 100%;
  }
}
</style>