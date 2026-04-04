import { defineStore } from 'pinia'
import loginApi from '@/api/login'
import tool from '@/utils/tool'
import router from '@/router'
import webRouter from '@/router/webRouter'
import { isUndefined } from 'lodash'
import { homePage } from '@/router/homePageRoutes'
import { useAppStore, useTagStore, useDictStore } from '@/store'
import { menuSeed } from '@/mock/index.js'

// 构建菜单树
function buildMenuTree(items, parentId = '0') {
  return items
    .filter(item => item.parent_id === parentId)
    .sort((a, b) => a.sort - b.sort)
    .map(item => ({
      ...item,
      children: buildMenuTree(items, item.id).length > 0 ? buildMenuTree(items, item.id) : undefined
    }))
}

const useUserStore = defineStore('user', {
  state: () => ({
    codes: undefined,
    roles: undefined,
    routers: undefined,
    user: undefined,
    menus: undefined
  }),

  getters: {
    getState() {
      return { ...this.$state }
    }
  },

  actions: {
    setToken(token) {
      tool.local.set(import.meta.env.VITE_APP_TOKEN_PREFIX, token)
    },

    getToken() {
      return tool.local.get(import.meta.env.VITE_APP_TOKEN_PREFIX)
    },

    clearToken() {
      tool.local.remove(import.meta.env.VITE_APP_TOKEN_PREFIX)
    },

    setInfo(data) {
      this.$patch(data)
    },

    resetUserInfo() {
      this.$reset()
    },

    setMenu(data) {
      const routers = flatAsyncRoutes(filterAsyncRouter(data))
      routers.map((item) => {
        if (isUndefined(item.meta.layout)) {
          router.addRoute('layout', item)
        } else {
          if (item.meta.layout) {
            router.addRoute('layout', item)
          } else {
            router.addRoute(item)
          }
        }
      })
    },

    requestUserInfo() {
      return new Promise((resolve, reject) => {
        loginApi.getInfo().then(async (response) => {
          if (!response || !response.data) {
            this.clearToken()
            await router.push({ name: 'login' })
            reject(false)
          } else {
            // 适配后端响应格式：data.info 包含用户信息
            const userData = response.data.info || response.data
            this.setInfo({
              user: {
                id: userData.id,
                username: userData.user,
                nickname: userData.notes || userData.user,
                avatar: userData.avatars || '',
                email: '',
                phone: '',
                dept_id: 1,
                dashboard: 'statistics',
                backend_setting: '{"mode":"light"}'
              },
              roles: [{ id: 1, name: '超级管理员', code: 'super_admin' }],
              codes: ['*'],
              routers: buildMenuTree(menuSeed)
            })
            const dictStore = useDictStore()
            await dictStore.initData()
            homePage.children = webRouter[0].children
            this.setMenu(this.routers)
            this.routers = removeButtonMenu(this.routers)
            this.routers.unshift(homePage)
            await this.setApp()
            resolve(response.data)
          }
        })
      })
    },

    login(form) {
      console.log('[UserStore Debug] login called with form:', form)
      return loginApi
        .login(form)
        .then((r) => {
          console.log('[UserStore Debug] login API response:', r)
          if (r.code === 200) {
            console.log('[UserStore Debug] login success, token:', r.data.token)
            this.setToken(r.data.token)
            return true
          } else {
            console.log('[UserStore Debug] login failed, code:', r.code, 'message:', r.message)
            return false
          }
        })
        .catch((e) => {
          console.error('[UserStore Debug] login error:', e)
          return false
        })
    },

    async logout() {
      // await loginApi.logout()
      const tagStore = useTagStore()
      tool.local.remove('tags')
      tagStore.clearTags()
      this.clearToken()
      this.resetUserInfo()
    },

    async setApp() {
      const appStore = useAppStore()
      const setting =
        typeof this.user.backend_setting === 'string'
          ? JSON.parse(this.user.backend_setting)
          : this.user.backend_setting
      appStore.toggleMode(setting?.mode ?? appStore.mode)
      appStore.toggleMenu(setting?.menuCollapse ?? appStore.menuCollapse)
      appStore.toggleTag(setting?.tag ?? appStore.tag)
      appStore.toggleRound(setting?.roundOpen ?? appStore.roundOpen)
      appStore.toggleWs(setting?.ws ?? appStore.ws)
      appStore.changeMenuWidth(setting?.menuWidth ?? appStore.menuWidth)
      appStore.changeLayout(setting?.layout ?? appStore.layout)
      appStore.useSkin(setting?.skin ?? appStore.skin)
      appStore.changeColor(setting?.color ?? appStore.color)
      appStore.toggleWater(setting?.waterMark ?? appStore.waterMark)
      appStore.changeWaterContent(
        setting?.waterContent ?? appStore.waterContent
      )
    }
  }
})

//路由扁平化
const flatAsyncRoutes = (routes, breadcrumb = []) => {
  let res = []
  routes.forEach((route) => {
    const tmp = { ...route }
    if (tmp.children) {
      let childrenBreadcrumb = [...breadcrumb]
      childrenBreadcrumb.push(route)
      let tmpRoute = { ...route }
      tmpRoute.meta.breadcrumb = childrenBreadcrumb
      delete tmpRoute.children
      res.push(tmpRoute)
      let childrenRoutes = flatAsyncRoutes(tmp.children, childrenBreadcrumb)
      childrenRoutes.map((item) => {
        res.push(item)
      })
    } else {
      let tmpBreadcrumb = [...breadcrumb]
      tmpBreadcrumb.push(tmp)
      tmp.meta.breadcrumb = tmpBreadcrumb
      res.push(tmp)
    }
  })
  return res
}

const views = import.meta.glob('../../views/**/**.vue')
const empty = import.meta.glob('../../layout/empty.vue')

// 菜单转换路由
const filterAsyncRouter = (routerMap) => {
  const accessedRouters = []
  routerMap.forEach((item) => {
    if (item.meta.type !== 'B') {
      if (item.meta.type === 'I') {
        item.meta.url = item.path
        item.path = `/maIframe/${item.name}`
      }

      const route = {
        path: item.path,
        name: item.name,
        hidden: item.hidden === 1,
        meta: item.meta,
        children: item.children ? filterAsyncRouter(item.children) : null,
        component: views[`../../views/${item.component}.vue`]
      }
      accessedRouters.push(route)
    }
  })
  return accessedRouters
}

// 去除按钮菜单
const removeButtonMenu = (routers) => {
  let handlerAfterRouters = []
  routers.forEach((item) => {
    if (item.meta.type !== 'B' && !item.meta.hidden) {
      let route = item
      if (item.children && item.children.length > 0) {
        route.children = removeButtonMenu(item.children)
      }
      handlerAfterRouters.push(route)
    }
  })
  return handlerAfterRouters
}
export default useUserStore
