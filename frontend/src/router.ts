import { createWebHistory, createRouter } from "vue-router";
import { useAuth } from "@/composables/useAuth";

const routes = [
    {
        path: "/",
        redirect: () => {
            const { isAuthenticated } = useAuth();
            return isAuthenticated.value ? "/dashboard" : "/login";
        },
    },
    {
        path: "/login",
        name: "Login",
        component: () => import("@/views/Login.vue"),
        meta: { requiresGuest: true },
    },
    {
        path: "/dashboard",
        name: "Dashboard",
        component: () => import("@/views/DashBoard.vue"),
        meta: { requiresAuth: true },
    },
];

export const router = createRouter({
    history: createWebHistory(),
    routes,
});

// 导航守卫
router.beforeEach((to) => {
    const { isAuthenticated } = useAuth();

    // 需要登录但未登录 -> 跳转登录页
    if (to.meta.requiresAuth && !isAuthenticated.value) {
        return { name: "Login", query: { redirect: to.fullPath } };
    }

    // 已登录访问登录页 -> 跳转dashboard
    if (to.meta.requiresGuest && isAuthenticated.value) {
        return { name: "Dashboard" };
    }
});
