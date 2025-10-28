import type { ListParamsReq } from "./api";

export interface School {
  id: number
  name: string
}

export type SchoolListRequest = {
  name?: string
}&ListParamsReq;

export interface SchoolUpdateRequest {
  name?: string
}

export interface SchoolCreateRequest {
  name: string
}
