import { createWebHistory, createRouter } from "vue-router";
import { useAuthStore } from "@/stores/useAuthStore";

const routes = [
    {
        path: "/",
        redirect: () => {
            const { isAuthenticated } = useAuthStore();
            return isAuthenticated ? "/dashboard" : "/login";
        },
    },
    {
        path: "/login",
        name: "Login",
        component: () => import("@/views/Login.vue"),
        meta: { requiresAuth: false },
    },
    {
        path: "/read/:contentId/:chapterId",
        name: "Reader",
        component: () => import("@/views/Reader.vue"),
        meta: { requiresAuth: true },
    },
    {
        path: "/",
        component: () => import("@/layouts/MainLayout.vue"),
        meta: { requiresAuth: true },
        children: [
            {
                path: "dashboard",
                name: "Dashboard",
                component: () => import("@/views/Dashboard.vue"),
            },
            {
                path: "library/:libraryId",
                name: "Library",
                component: () => import("@/views/Library.vue"),
            },
            {
                path: "library/:libraryId/content/:contentId",
                name: "Content",
                component: () => import("@/views/Content.vue"),
            },
            {
                path: "scan-tasks",
                name: "ScanTasks",
                component: () => import("@/views/ScanQueue.vue"),
            },
        ],
    },
];

export const router = createRouter({
    history: createWebHistory(),
    routes,
});

// 导航守卫
router.beforeEach((to) => {
    const { isAuthenticated } = useAuthStore();

    // 需要登录但未登录 -> 跳转登录页
    if (to.meta.requiresAuth && !isAuthenticated) {
        return { name: "Login", query: { redirect: to.fullPath } };
    }

    // 已登录访问登录页 -> 跳转dashboard
    if (to.name === "Login" && isAuthenticated) {
        return { name: "Dashboard" };
    }
});
