import request from "./request";

// Auth
export const login = (data) => request({ url: "/api/login/wechat", method: "POST", data });

// User
export const getCurrentUser = () => request({ url: "/api/admin/me" });

// Classes
export const getClassDetail = (id) => request({ url: `/api/admin/classes/${id}` });
export const updateClassStatus = (id, data) => request({ url: `/api/admin/classes/${id}/status`, method: "PUT", data });
export const bindClass = (data) => request({ url: "/api/admin/bind/class", method: "POST", data });
export const unbindClass = (id) => request({ url: `/api/admin/unbind/class/${id}`, method: "DELETE" });
