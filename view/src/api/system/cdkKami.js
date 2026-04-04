import { request } from '@/utils/request.js'

/**
 * 卡密管理 API
 * 后端请求格式:
 * - 列表: { page, size, so: { add_time, use_time, add_role, state, out_state, type, use_state, keyword, keywordType } }
 * - 添加: { gid, note, length, pre, num, out }
 * - 编辑: { id, note, password, vip, val, sn_max, ban, ban_msg }
 */
export default {
  /**
   * 获取卡密列表
   */
  getList(params = {}) {
    const backendParams = {
      page: params.page || 1,
      size: params.size || params.page_size || 20
    }
    
    // 构建搜索条件
    const so = {}
    if (params.code || params.keyword) {
      so.keyword = params.code || params.keyword
      so.keywordType = params.keywordType || 'cdk'
    }
    if (params.group_id) {
      so.gid = params.group_id
    }
    if (params.status !== undefined) {
      so.use_state = params.status === 1 ? 'y' : ''
      so.state = params.state || 'y'
    }
    if (params.add_time || params.addTimeRange) {
      so.add_time = params.add_time || params.addTimeRange
    }
    if (params.use_time || params.useTimeRange) {
      so.use_time = params.use_time || params.useTimeRange
    }
    if (params.type) {
      so.type = params.type
    }
    if (params.out_state) {
      so.out_state = params.out_state
    }
    
    if (Object.keys(so).length > 0) {
      backendParams.so = so
    }
    
    return request({
      url: '/admin/cdkKami/list',
      method: 'post',
      data: backendParams
    })
  },

  /**
   * 添加/生成卡密
   * @param {Object} data - 卡密数据
   * @param {number} data.gid - 卡密组ID (必填)
   * @param {string} data.note - 备注
   * @param {number} data.length - 卡密长度 (16-32)
   * @param {string} data.pre - 卡密前缀
   * @param {number} data.num - 生成数量 (1-10000)
   * @param {string} data.out - 导出状态
   */
  add(data = {}) {
    return request({
      url: '/admin/cdkKami/add',
      method: 'post',
      data: {
        gid: data.gid || data.group_id,
        note: data.note || data.remark,
        length: data.length || 20,
        pre: data.pre || data.prefix || '',
        num: data.num || data.count || 1,
        out: data.out || ''
      }
    })
  },

  /**
   * 编辑卡密
   * @param {Object} data - 卡密数据
   * @param {number} data.id - 卡密ID (必填)
   * @param {string} data.note - 备注
   * @param {string} data.password - 卡密密码
   * @param {number} data.vip - VIP类型
   * @param {number} data.val - 值
   * @param {number} data.sn_max - 最大设备数
   * @param {number} data.ban - 封禁状态
   * @param {string} data.ban_msg - 封禁原因
   */
  edit(data = {}) {
    const backendData = { id: data.id }
    if (data.note !== undefined) backendData.note = data.note
    if (data.password !== undefined) backendData.password = data.password
    if (data.vip !== undefined) backendData.vip = data.vip
    if (data.val !== undefined) backendData.val = data.val
    if (data.sn_max !== undefined) backendData.sn_max = data.sn_max
    if (data.ban !== undefined) backendData.ban = data.ban
    if (data.ban_msg !== undefined) backendData.ban_msg = data.ban_msg
    
    return request({
      url: '/admin/cdkKami/edit',
      method: 'post',
      data: backendData
    })
  },

  /**
   * 删除卡密
   */
  del(id) {
    return request({
      url: '/admin/cdkKami/del',
      method: 'post',
      data: { id }
    })
  },

  /**
   * 批量删除卡密
   */
  delAll(ids) {
    return request({
      url: '/admin/cdkKami/delall',
      method: 'post',
      data: { ids }
    })
  },

  /**
   * 导出卡密
   * @param {Object} params - 导出参数
   * @param {Array} params.ids - 卡密ID列表
   * @param {string} params.out - 导出状态
   */
  exportAll(params = {}) {
    return request({
      url: '/admin/cdkKami/outall',
      method: 'post',
      data: {
        ids: params.ids || [],
        out: params.out || 'y'
      },
      responseType: 'blob'
    })
  }
}
