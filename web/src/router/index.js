import Vue from 'vue'
import Router from 'vue-router'
import Dashboard from '@/pages/Dashboard'
import Devices from '@/pages/Devices'
import NotFound from '@/pages/NotFound'

Vue.use(Router)

export default new Router({
  routes: [
    {
      path: '/',
      name: 'Dashboard',
      component: Dashboard
    },
    {
      path: '/devices',
      name: 'Devices',
      component: Devices
    },
    {
      path: '*',
      name: 'NotFound',
      component: NotFound
    }
  ]
})
