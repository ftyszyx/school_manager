import type { ListParamsReq } from "./api";

export interface School {
  id: number
  name: string
  password: string
}

export type VantTagType = 'default' | 'success' | 'warning' | 'danger' | 'primary'

export interface SchoolClassStatusConfigItem {
  status: number
  label: string
  type: VantTagType
  sort_order: number
}

export interface UpdateSchoolClassStatusConfigsRequest {
  configs: SchoolClassStatusConfigItem[]
}

export interface PublicSchool {
  id: number
  name: string
}

export type SchoolListRequest = {
  name?: string
}&ListParamsReq;

export interface SchoolUpdateRequest {
  name?: string
  password?: string
}

export interface SchoolCreateRequest {
  name: string
  password: string
}
