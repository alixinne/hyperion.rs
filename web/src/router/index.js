import Vue from 'vue'
import Router from 'vue-router'
import Dashboard from '@/views/Dashboard'
import Devices from '@/views/Devices'
import NotFound from '@/views/NotFound'

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
