import type { ListParamsReq } from "./api";
import type { Permission} from "./permissions";
export interface Role {
  id: number
  name: string
  description?: string
  permission_infos?: Permission[]
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
