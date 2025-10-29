import type { PagingResponse } from '@/types/api'
import type {
  ClassInfo,
  ClassCreateRequest,
  ClassListRequest,
  ClassUpdateRequest,
  ClassBulkCreatePayload
} from '@/types/classes'
import request from '@/utils/request'

export const getClasses = async (params: ClassListRequest): Promise<PagingResponse<ClassInfo>> => {
  return (await request.get('/api/admin/classes', { params })).data
}

export const getClass = async (id: number): Promise<ClassInfo> => {
  return (await request.get(`/api/admin/classes/${id}`)).data
}

export const createClass = async (data: ClassCreateRequest): Promise<ClassInfo> => {
  return (await request.post('/api/admin/classes', data)).data
}

export const updateClass = async (id: number, data: ClassUpdateRequest): Promise<ClassInfo> => {
  return (await request.put(`/api/admin/classes/${id}`, data)).data
}

export const deleteClass = async (id: number): Promise<void> => {
  return (await request.delete(`/api/admin/classes/${id}`)).data
}

export const createClassesBulk = async (data: ClassBulkCreatePayload): Promise<void> => {
  return (await request.post('/api/admin/classes/bulk', data)).data
}
