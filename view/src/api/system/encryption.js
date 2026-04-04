import { request } from '@/utils/request.js'

export default {
  /**
   * 获取加密方案列表
   */
  getList(params = {}) {
    return request({
      url: '/admin/encryption/list',
      method: 'post',
      data: params
    })
  },

  /**
   * 添加加密方案
   */
  add(data = {}) {
    return request({
      url: '/admin/encryption/add',
      method: 'post',
      data
    })
  },

  /**
   * 编辑加密方案
   */
  edit(data = {}) {
    return request({
      url: '/admin/encryption/edit',
      method: 'post',
      data
    })
  },

  /**
   * 删除加密方案
   */
  del(id) {
    return request({
      url: '/admin/encryption/del',
      method: 'post',
      data: { id }
    })
  },

  /**
   * 获取加密插件列表
   */
  getPlugins() {
    return request({
      url: '/admin/encryption/plug',
      method: 'get'
    })
  }
}
