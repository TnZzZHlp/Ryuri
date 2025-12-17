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
        path: "/read/:libraryId/:contentId/:chapterId",
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

// Route guard
router.beforeEach((to) => {
    const { isAuthenticated } = useAuthStore();

    // Requires authentication but not authenticated -> redirect to login
    if (to.meta.requiresAuth && !isAuthenticated) {
        return { name: "Login", query: { redirect: to.fullPath } };
    }

    // Authenticated user accessing login page -> redirect to dashboard
    if (to.name === "Login" && isAuthenticated) {
        return { name: "Dashboard" };
    }

    // Change html title
    if (to.name) {
        document.title = `Ryuri - ${to.name
            .toString()
            .replace(/([A-Z])/g, " $1")
            .trim()}`;
    }
});
