import type { ListParamsReq } from "./api";
export interface Role {
  id: number
  name: string
  description?: string
}

export type RoleListRequest = {
  name?: string
  id?: number
}&ListParamsReq;

export interface RoleUpdateRequest {
  name?: string
  description?: string
}

export interface RoleCreateRequest {
  name: string
  description?: string
}
