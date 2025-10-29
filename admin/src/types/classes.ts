import type { ListParamsReq } from "./api"

export interface ClassUserInfo {
  user_id: number
  user_name: string
}

export interface ClassInfo {
  id: number
  name: string
  grade: number
  class: number
  school_id: number
  status: number
  password?: string
  teacher_infos: ClassUserInfo[]
}

export type ClassListRequest = {
  name?: string
  school_id?: number
  grade?: number
  class?: number
  status?: number
}&ListParamsReq;

export interface ClassUpdateRequest {
  name?: string
  grade?: number
  class?: number
  school_id?: number
  status?: number
  password?: string
}

export interface ClassCreateRequest {
  name: string
  grade: number
  class: number
  school_id: number
  status?: number
  password?: string
}

export interface ClassBulkCreatePayload {
  classes: ClassCreateRequest[]
}
