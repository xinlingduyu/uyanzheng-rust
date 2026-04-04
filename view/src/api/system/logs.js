import { request } from '@/utils/request.js'

/**
 * 日志管理 API
 * 后端请求格式:
 * - 总列表: { pg, size, so: { type, keyword } }
 * - 用户日志: { page, size, so: { time, type, keyword, appid } }
 * - 管理员日志: { page, size, so: { time, type, keyword, appid } }
 * 响应格式: { code, msg, data: { list, currentPage, pageTotal, dataTotal } }
 */
export default {
  /**
   * 获取日志列表（总览）
   * @param {Object} params - 查询参数
   * @param {number} params.pg - 页码
   * @param {number} params.size - 每页数量
   * @param {Object} params.so - 搜索条件
   * @param {string} params.so.type - 日志类型
   * @param {string} params.so.keyword - 搜索关键词
   */
  getList(params = {}) {
    const backendParams = {
      pg: params.pg || params.page || 1,
      size: params.size || params.page_size || 20
    }
    
    const so = {}
    if (params.type) so.type = params.type
    if (params.keyword) so.keyword = params.keyword
    
    if (Object.keys(so).length > 0) {
      backendParams.so = so
    }
    
    return request({
      url: '/admin/logs/list',
      method: 'post',
      data: backendParams
    })
  },

  /**
   * 获取用户日志
   * @param {Object} params - 查询参数
   * @param {number} params.page - 页码
   * @param {number} params.size - 每页数量
   * @param {Object} params.so - 搜索条件
   * @param {Array} params.so.time - 时间范围 [start, end]
   * @param {string} params.so.type - 日志类型
   * @param {string} params.so.keyword - 搜索关键词
   * @param {number} params.so.appid - 应用ID
   */
  getUserLogs(params = {}) {
    const backendParams = {
      page: params.page || 1,
      size: params.size || params.page_size || 20
    }
    
    const so = {}
    if (params.time || params.dateRange) {
      so.time = params.time || params.dateRange
    }
    if (params.type) so.type = params.type
    if (params.keyword) so.keyword = params.keyword
    if (params.appid) so.appid = params.appid
    
    if (Object.keys(so).length > 0) {
      backendParams.so = so
    }
    
    return request({
      url: '/admin/logs/list/user',
      method: 'post',
      data: backendParams
    })
  },

  /**
   * 获取管理员日志
   * @param {Object} params - 查询参数 (同 getUserLogs)
   */
  getAdminLogs(params = {}) {
    const backendParams = {
      page: params.page || 1,
      size: params.size || params.page_size || 20
    }
    
    const so = {}
    if (params.time || params.dateRange) {
      so.time = params.time || params.dateRange
    }
    if (params.type) so.type = params.type
    if (params.keyword) so.keyword = params.keyword
    if (params.appid) so.appid = params.appid
    
    if (Object.keys(so).length > 0) {
      backendParams.so = so
    }
    
    return request({
      url: '/admin/logs/list/admin',
      method: 'post',
      data: backendParams
    })
  },

  /**
   * 获取用户日志类型
   * 返回: { label, value }[] 或映射对象
   */
  getUserLogTypes() {
    return request({
      url: '/admin/logs/type/user',
      method: 'get'
    })
  },

  /**
   * 获取管理员日志类型
   * 返回: { label, value }[] 或映射对象
   */
  getAdminLogTypes() {
    return request({
      url: '/admin/logs/type/admin',
      method: 'get'
    })
  },

  /**
   * 删除日志
   */
  del(id) {
    return request({
      url: '/admin/logs/del',
      method: 'post',
      data: { id }
    })
  },

  /**
   * 清理日志
   * @param {Object} params - 清理参数
   * @param {string} params.type - 日志类型
   * @param {Array} params.time - 时间范围
   */
  clean(params = {}) {
    return request({
      url: '/admin/logs/clean',
      method: 'post',
      data: params
    })
  }
}
