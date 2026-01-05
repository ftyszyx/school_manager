import request from "./request";

export type VantTagType = 'default' | 'success' | 'warning' | 'danger' | 'primary';

export interface ClassTeacherInfo {
	user_id: number;
	user_name: string;
}

export interface ClassDetail {
	id: number;
	name: string;
	grade: number;
	class: number;
	school_id: number;
	school_name: string;
	status: number;
	password: string;
	teacher_infos: ClassTeacherInfo[];
}

export interface UserClassInfo {
	class_id: number;
	class_name: string;
	school_id: number;
	school_name: string;
	grade: number;
	class: number;
	status: number;
	// compatibility
	id?: number;
	name?: string;
}

export interface UserRoleInfo {
	role_id: number;
	role_name: string;
	// compatibility
	name?: string;
}

export interface SchoolClassStatusConfig {
	status: number;
	label: string;
	type: VantTagType;
	sort_order: number;
}

export interface CurrentUser {
	id: number;
	username: string;
	school_id: number | null;
	school_name: string | null;
	phone: string | null;
	wechat_openid: string | null;
	wechat_unionid: string | null;
	wechat_nickname: string | null;
	wechat_avatar_url: string | null;
	class_infos: UserClassInfo[];
	role_infos: UserRoleInfo[];
	class_status_configs: SchoolClassStatusConfig[];
	created_at: string;
}

// Auth
export const wechatLogin = (data: any) => request<{ token: string }>({ url: "/api/login/wechat", method: "POST", data });

// User
export const getCurrentUser = () => request<CurrentUser>({ url: "/api/admin/me" });
export const updateMyInfo = (data: any) => request({ url: "/api/admin/me", method: "PUT", data });

// Classes
export const getClassDetail = (id: number) => request<ClassDetail>({ url: `/api/admin/classes/${id}` });
export const updateClass = (id: number, data: any) => request<ClassDetail>({ url: `/api/admin/classes/${id}`, method: "PUT", data });
export const updateClassStatus = (id: number, data: { status: number }) =>
	request({ url: `/api/admin/classes/${id}/status`, method: 'PUT', data });
export const bindClass = (data: { class_id: number; password: string }) => request({ url: '/api/admin/bind/class', method: 'POST', data });
export const unbindClass = (id: number) => request({ url: `/api/admin/unbind/class/${id}`, method: 'DELETE' });
export const getClassesBySchool = (schoolId: number) => request<any[]>({ url: `/api/classes/school/${schoolId}` });

// Schools
export const getAllSchools = () => request<any[]>({ url: "/api/schools/all" });
export const bindSchool = (data: { school_id: number; password: string }) => request({ url: "/api/admin/bind/school", method: "POST", data });
