import { createRouter, createWebHistory } from 'vue-router'
import AdminLayout from '@/layouts/AdminLayout.vue'
import LoginView from '@/views/auth/LoginView.vue'
import RegisterView from '@/views/auth/RegisterView.vue'
import { useAuthStore } from '@/stores/auth'
import { RouteName, RoutePath } from '@/types'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            component: () => import('@/layouts/HomeLayout.vue'),
            children: [
                {
                    path: '',
                    name: RouteName.Home,
                    component: () => import('@/views/HomeView.vue')
                },
            ]
        },
        {
            path: RoutePath.Login,
            name: RouteName.Login,
            component: LoginView
        },
        {
            path: RoutePath.Register,
            name: RouteName.Register,
            component: RegisterView
        },
        {
            path: RoutePath.Admin,
            component: AdminLayout,
            meta: { requiresAuth: true },
            redirect: RoutePath.AdminDashboard,
            children: [
                {
                    path: RoutePath.AdminDashboard,
                    name: RouteName.AdminDashboard,
                    component: () => import('@/views/admin/DashboardView.vue')
                },
                {
                    path: RoutePath.AdminUsers,
                    name: RouteName.AdminUsers,
                    component: () => import('@/views/admin/UserAdminView.vue')
                },
                {
                    path: RoutePath.AdminRoles,
                    name: RouteName.AdminRoles,
                    component: () => import('@/views/admin/RoleAdminView.vue')
                },
                {
                    path: RoutePath.AdminPermissions,
                    name: RouteName.AdminPermissions,
                    component: () => import('@/views/admin/PermissionAdminView.vue')
                },
                {
                    path: 'schools',
                    name: 'AdminSchools',
                    component: () => import('@/views/admin/SchoolAdminView.vue'),
                    meta: {
                        title: 'Schools Management'
                    }
                },
                {
                    path: 'classes',
                    name: 'AdminClasses',
                    component: () => import('@/views/admin/ClassAdminView.vue'),
                    meta: {
                        title: 'Classes Management'
                    }
                }
            ]
        }
    ]
})

router.beforeEach((to, _, next) => {
    const authStore = useAuthStore()
    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next({ name: 'login' })
    } else {
        next()
    }
})

export default router 