import { request } from '@/utils/request.js'

export default {
  /**
   * 获取版本列表
   */
  getList(params = {}) {
    return request({
      url: '/admin/ver/list',
      method: 'post',
      data: params
    })
  },

  /**
   * 获取版本分组
   */
  getGroup() {
    return request({
      url: '/admin/ver/group',
      method: 'get'
    })
  },

  /**
   * 添加版本
   */
  add(data = {}) {
    return request({
      url: '/admin/ver/add',
      method: 'post',
      data
    })
  },

  /**
   * 编辑版本
   */
  edit(data = {}) {
    return request({
      url: '/admin/ver/edit',
      method: 'post',
      data
    })
  },

  /**
   * 删除版本
   */
  del(id) {
    return request({
      url: '/admin/ver/del',
      method: 'post',
      data: { id }
    })
  },

  /**
   * 批量删除版本
   */
  delAll(ids) {
    return request({
      url: '/admin/ver/delall',
      method: 'post',
      data: { ids }
    })
  },

  /**
   * 设置弃用状态
   */
  discard(data = {}) {
    return request({
      url: '/admin/ver/discard',
      method: 'post',
      data
    })
  }
}
