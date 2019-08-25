import Vue from 'vue'
import api from '@/api'

const state = {
  info: null
}

const actions = {
  async load ({ commit }) {
    try {
      let response = await api.devices.index()
      commit('load', { info: response.data })
    } catch (error) {
      commit('load', { error })
    }
  },
  async patch ({ state, commit }, { id, patch }) {
    try {
      let response = await api.devices.patch(id, patch)
      commit('patch', { id, patch, response: response.data })
    } catch (error) {
      commit('patch', { id, error })
    }
  }
}

const mutations = {
  load (state, { info, error }) {
    Vue.set(state, 'info', typeof error === 'undefined' ? info : null)
  },
  patch (state, { id, patch, error, response }) {
    if (typeof error === 'undefined') {
      // Only apply fields from the response changed in the patch
      Object.keys(patch).map((key, index) => {
        patch[key] = response[key]
      })

      Vue.set(state.info, id, { ...state.info[id], ...patch })
    }
  }
}

export default {
  namespaced: true,
  state,
  actions,
  mutations
}
