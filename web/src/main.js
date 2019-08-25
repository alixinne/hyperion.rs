import Vue from 'vue'
import Vuelidate from 'vuelidate'
import App from './App'
import router from './router'

import { library } from '@fortawesome/fontawesome-svg-core'
// internal icons
import { faCheck, faCheckCircle, faInfoCircle, faExclamationTriangle, faExclamationCircle,
  faArrowUp, faAngleRight, faAngleLeft, faAngleDown,
  faEye, faEyeSlash, faCaretDown, faCaretUp, faUpload } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import Buefy, { ToastProgrammatic as Toast } from 'buefy'

import store from './store'
import axios from 'axios'

library.add(faCheck, faCheckCircle, faInfoCircle, faExclamationTriangle, faExclamationCircle,
  faArrowUp, faAngleRight, faAngleLeft, faAngleDown,
  faEye, faEyeSlash, faCaretDown, faCaretUp, faUpload)
Vue.component('vue-fontawesome', FontAwesomeIcon)

Vue.use(Buefy, {
  defaultIconComponent: 'vue-fontawesome',
  defaultIconPack: 'fas'
})

Vue.use(Vuelidate)

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
  console.log(error.message)
  Toast.open({
    message: error.message,
    type: 'is-danger'
  })
  return Promise.reject(error)
})

/* eslint-disable no-new */
new Vue({
  el: '#app',
  router,
  store,
  render: h => h(App)
})
