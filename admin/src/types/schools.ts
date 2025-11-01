import type { ListParamsReq } from "./api";

export interface School {
  id: number
  name: string
  password: string
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
