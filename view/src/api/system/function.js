import { request } from '@/utils/request.js'

export default {
  /**
   * 获取云函数列表
   */
  getList(params = {}) {
    return request({
      url: '/admin/functions/list',
      method: 'post',
      data: params
    })
  },

  /**
   * 添加云函数
   */
  add(data = {}) {
    return request({
      url: '/admin/functions/add',
      method: 'post',
      data
    })
  },

  /**
   * 编辑云函数
   */
  edit(data = {}) {
    return request({
      url: '/admin/functions/edit',
      method: 'post',
      data
    })
  },

  /**
   * 删除云函数
   */
  del(id) {
    return request({
      url: '/admin/functions/del',
      method: 'post',
      data: { id }
    })
  },

  /**
   * 获取函数代码
   */
  getCode(id) {
    return request({
      url: '/admin/functions/getCode',
      method: 'post',
      data: { id }
    })
  }
}
