import request from "./request";

// Auth
export const login = (data: any) => request<{ token: string }>({ url: "/api/login/wechat", method: "POST", data });

// User
export const getCurrentUser = () => request<any>({ url: "/api/admin/me" });
export const updateMyInfo = (data: any) => request({ url: "/api/admin/me", method: "PUT", data });

// Classes
export const getClassDetail = (id: number) => request<any>({ url: `/api/admin/classes/${id}` });
export const updateClassStatus = (id: number, data: { status: number }) =>
	request({ url: `/api/admin/classes/${id}/status`, method: 'PUT', data });
export const bindClass = (data: { class_id: number; password: string }) => request({ url: '/api/admin/bind/class', method: 'POST', data });
export const unbindClass = (id: number) => request({ url: `/api/admin/unbind/class/${id}`, method: 'DELETE' });
export const getClassesBySchool = (schoolId: number) => request<any[]>({ url: `/api/classes/school/${schoolId}` });

// Schools
export const getAllSchools = () => request<any[]>({ url: "/api/schools/all" });
export const bindSchool = (data: { school_id: number; password: string }) => request({ url: "/api/admin/bind/school", method: "POST", data });
