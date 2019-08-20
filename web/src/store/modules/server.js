import api from '@/api'

const state = {
  info: null
}

const actions = {
  async load ({ commit }) {
    try {
      var response = await api.server.info()
      commit('loadSuccess', response.data)
    } catch (error) {
      console.log(error)
      commit('loadError', error)
    }
  }
}

const mutations = {
  loadSuccess (state, server) {
    state.info = server
  },
  loadError (state, error) {
    state.info = null
  }
}

export default {
  namespaced: true,
  state,
  actions,
  mutations
}
