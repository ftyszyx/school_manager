import type { ListParamsReq } from "./api"

export interface User {
  id: number
  username: string
  role_ids: number[]
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