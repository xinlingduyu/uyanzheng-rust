import axios from 'axios'
import { Message } from '@arco-design/web-vue'
import tool from '@/utils/tool'
import { get, isEmpty } from 'lodash'
import qs from 'qs'
import { h } from 'vue'
import { IconFaceFrownFill } from '@arco-design/web-vue/dist/arco-vue-icon'
import router from '@/router'

function createExternalService() {
  // 创建一个外部网络 axios 实例
  const service = axios.create()

  // HTTP request 拦截器
  service.interceptors.request.use(
    (config) => config,
    (error) => Promise.reject(error)
  )

  // HTTP response 拦截器
  service.interceptors.response.use(
    (response) => response,
    (error) => {
      Promise.reject(error.response ?? null)
    }
  )
  return service
}

function createService() {
  // 创建一个 axios 实例
  const service = axios.create()

  /**
   * 提取后端返回的错误信息（兼容多种字段）
   */
  function extractErrorMsg(data) {
    if (!data) return '未知错误'
    if (typeof data === 'string') return data
    // 优先尝试常见的错误字段
    return data.message || data.msg || data.error || data.detail || '服务器返回了未知的错误格式'
  }

  // HTTP request 拦截器
  service.interceptors.request.use(
    (config) => config,
    (error) => {
      console.error('[Request Error]', error)
      return Promise.reject(error)
    }
  )

  // HTTP response 拦截器
  service.interceptors.response.use(
    async (response) => {
      const { data, headers, status } = response

      // 1. 判断是否为文件下载 (非 JSON 或有 content-disposition)
      const isFileDownload = 
        (headers['content-disposition'] || 
        !/^application\/json/.test(headers['content-type'])) && 
        status === 200

      if (isFileDownload) {
        // 特殊情况：有些后端错误时返回 JSON 但头信息不对，导致 axios 认为是文件
        if (data instanceof Blob && data.type && data.type.includes('application/json')) {
          try {
            const text = await data.text()
            const json = JSON.parse(text)
            const msg = extractErrorMsg(json)
            Message.error({ content: msg, icon: () => h(IconFaceFrownFill) })
            return { code: 500, message: msg, success: false }
          } catch (e) {
            console.error('Blob Error Parse Failed', e)
          }
        }
        return response
      }

      // 2. 处理业务逻辑 (JSON)
      const code = data.code

      // 如果 code 不存在 (如直接返回对象/数组) 或 code 为 200，视为成功
      if (code === undefined || code === 200) {
        return data
      }

      // 3. 业务状态码错误处理
      console.error('[Response Business Error]', response.config.url, data)
      const errorMsg = extractErrorMsg(data)

      // 401 特殊处理
      if (code === 401) {
        throttle(() => {
          Message.error({
            content: errorMsg || '登录状态已过期，请重新登录',
            icon: () => h(IconFaceFrownFill)
          })
          tool.local.clear()
          router.push({ name: 'login' })
        })()
      } else {
        // 其他错误直接弹窗
        Message.error({
          content: errorMsg,
          icon: () => h(IconFaceFrownFill)
        })
      }

      return data
    },
    (error) => {
      // 4. HTTP 状态码错误处理
      console.error('[Response HTTP Error]', {
        url: error.config?.url,
        status: error.response?.status,
        data: error.response?.data,
        message: error.message
      })

      const response = error.response
      let errorMessage = '请求发生错误'

      if (response && response.data) {
        // 提取后端返回的错误信息
        errorMessage = extractErrorMsg(response.data)

        // 401 特殊处理
        if (response.status === 401) {
          throttle(() => {
            Message.error({
              content: errorMessage || '登录状态已过期，需要重新登录',
              icon: () => h(IconFaceFrownFill)
            })
            tool.local.clear()
            router.push({ name: 'login' })
          })()
        } else {
          // 其他错误 (404, 403, 500等)：统一展示后端返回的错误信息
          Message.error({
            content: errorMessage,
            icon: () => h(IconFaceFrownFill)
          })
        }
      } else if (error.code === 'ECONNABORTED') {
        errorMessage = '请求超时，服务器未响应'
        Message.error({ content: errorMessage, icon: () => h(IconFaceFrownFill) })
      } else if (!window.navigator.onLine) {
        errorMessage = '网络连接已断开'
        Message.error({ content: errorMessage, icon: () => h(IconFaceFrownFill) })
      } else {
        errorMessage = error.message || '未知错误'
        Message.error({ content: errorMessage, icon: () => h(IconFaceFrownFill) })
      }

      // 返回统一错误结构，防止前端代码报错
      return Promise.resolve({
        code: response?.status || 500,
        message: errorMessage,
        success: false
      })
    }
  )
  return service
}

//节流
function throttle(fn, wait = 1500) {
  return function () {
    let context = this
    if (!throttle.timer) {
      fn.apply(context, arguments)
      throttle.timer = setTimeout(function () {
        throttle.timer = null
      }, wait)
    }
  }
}

function stringify(data) {
  return qs.stringify(data, { allowDots: true, encode: false })
}

function formatToken(token) {
  return token ? `Bearer ${token}` : null
}

/**
 * @description 创建请求方法
 * @param service
 * @param externalService
 */
function createRequest(service, externalService) {
  return function (config) {
    const env = import.meta.env
    const token = tool.local.get(env.VITE_APP_TOKEN_PREFIX)
    const setting = tool.local.get('setting')
    const configDefault = {
      headers: Object.assign(
        {
          Authorization: formatToken(token),
          'Accept-Language': setting?.language || 'zh_CN',
          'Content-Type': get(
            config,
            'headers.Content-Type',
            'application/json;charset=UTF-8'
          )
        },
        config.headers
      ),

      timeout: 10000,
      data: {}
    }

    delete config.headers
    // return
    const option = Object.assign(configDefault, config)

    // json
    if (!isEmpty(option.params)) {
      option.url = option.url + '?' + stringify(option.params)
      option.params = {}
    }

    if (!/^(http|https)/g.test(option.url)) {
      option.baseURL =
        env.VITE_APP_OPEN_PROXY === 'true'
          ? env.VITE_APP_PROXY_PREFIX
          : env.VITE_APP_BASE_URL
      return service(option)
    } else {
      return externalService(option)
    }
  }
}

export const service = createService()
export const externalService = createExternalService()
export const request = createRequest(service, externalService)
