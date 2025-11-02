import type { PagingResponse } from '@/types/api'
import type {
  School,
  SchoolCreateRequest,
  SchoolListRequest,
  SchoolUpdateRequest
} from '@/types/schools'
import request from '@/utils/request'

export const getSchools = async (params: SchoolListRequest): Promise<PagingResponse<School>> => {
  return (await request.get('/api/admin/schools', { params })).data
}

export const getSchool = async (id: number): Promise<School> => {
  return (await request.get(`/api/admin/schools/${id}`)).data
}

export const createSchool = async (data: SchoolCreateRequest): Promise<School> => {
  return (await request.post('/api/admin/schools', data)).data
}

export const updateSchool = async (id: number, data: SchoolUpdateRequest): Promise<School> => {
  return (await request.put(`/api/admin/schools/${id}`, data)).data
}

export const deleteSchool = async (id: number): Promise<void> => {
  return (await request.delete(`/api/admin/schools/${id}`)).data
}

export const getSimpleSchool = async (id: number): Promise<School> => {
  return (await request.get(`/api/schools/${id}/simple`)).data
}
