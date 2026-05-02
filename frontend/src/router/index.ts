import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '../views/DashboardView.vue'
import PortsView from '../views/PortsView.vue'
import MocksView from '../views/MocksView.vue'
import LogsView from '../views/LogsView.vue'
import FunctionsView from '../views/FunctionsView.vue'

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/',           component: DashboardView,  meta: { title: 'Dashboard' } },
    { path: '/ports',      component: PortsView,      meta: { title: 'Ports' } },
    { path: '/mocks',      component: MocksView,       meta: { title: 'Mocks' } },
    { path: '/logs',       component: LogsView,        meta: { title: 'Logs' } },
    { path: '/functions',  component: FunctionsView,   meta: { title: 'Functions' } },
  ],
})
