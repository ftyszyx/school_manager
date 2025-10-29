import type { ListParamsReq } from "./api"

export interface UserRoleInfo {
  role_id: number
  role_name: string
}

export interface UserClassInfo {
  class_id: number
  class_name: string
  school_id: number
  school_name: string
  grade: number
  class: number
}

export interface User {
  id: number
  username: string
  role_infos: UserRoleInfo[]
  class_infos: UserClassInfo[]
  created_at: string
}

export type UserListRequest = {
  username?: string
  id?: number
}&ListParamsReq;

export interface UserUpdateRequest {
  username?: string
  password?: string
  role_ids?: number[]
  class_ids?: number[]
}

export interface UserCreateRequest {
  username: string
  password: string
  role_ids: number[]
  class_ids: number[]
}

export interface AuthPayload {
    username: string;
    password: string;
}

export interface AuthResponse {
    token: string;
}

export interface RegisterPayload {
    username: string;
    password: string;
}

export interface ChangePasswordPayload {
    old_password: string;
    new_password: string;
}