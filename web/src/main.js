import Vue from 'vue'
import App from './App'
import router from './router'
import Buefy from 'buefy'
import store from './store'
import axios from 'axios'

Vue.use(Buefy)

Vue.config.productionTip = false

axios.interceptors.request.use((config) => {
  store.commit('loadStart')
  return config
}, (error) => {
  return Promise.reject(error)
})

axios.interceptors.response.use((response) => {
  store.commit('loadSuccess')
  return response
}, (error) => {
  store.commit('loadError', error)
  return Promise.reject(error)
})

/* eslint-disable no-new */
new Vue({
  el: '#app',
  router,
  store,
  render: h => h(App)
})
