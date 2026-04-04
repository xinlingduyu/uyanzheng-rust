import { request } from '@/utils/request.js'

export default {
  /**
   * 获取支付插件列表和配置信息
   * 后端返回格式:
   * {
   *   info: { pay_ali_state, pay_ali_type, pay_ali_config, pay_wx_state, pay_wx_type, pay_wx_config },
   *   plug: [{ id, type, name, extra, form }]
   * }
   */
  getInfo() {
    return request({
      url: '/admin/app/pay',
      method: 'get'
    })
  },

  /**
   * 编辑支付配置
   * @param {Object} data - 配置数据
   * @param {number} data.id - 应用ID
   * @param {string} [data.pay_ali_state] - 支付宝状态 "on"|"off"
   * @param {string} [data.pay_ali_type] - 支付宝引擎 "jie"|"ali"
   * @param {object} [data.pay_ali_config] - 支付宝配置
   * @param {string} [data.pay_wx_state] - 微信状态 "on"|"off"
   * @param {string} [data.pay_wx_type] - 微信引擎 "jie"|"wx"
   * @param {object} [data.pay_wx_config] - 微信配置
   */
  edit(data = {}) {
    return request({
      url: '/admin/app/pay/edit',
      method: 'post',
      data
    })
  }
}
