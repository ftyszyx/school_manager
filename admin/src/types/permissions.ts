import type { ListParamsReq } from "./api";
export interface Permission {
  id: number
  name: string
  resource: string
  action: string
  description?: string
}

export type PermissionListRequest = {
  name?: string
  resource?: string
  action?: string
}&ListParamsReq;

export interface PermissionUpdateRequest {
  name?: string
  resource?: string
  action?: string
  description?: string
}

export interface PermissionCreateRequest {
  name: string
  resource: string
  action: string
  description?: string
}

export enum  ActionType {
  READ = "read",
  CREATE = "create",
  UPDATE = "update",
  DELETE = "delete",
}