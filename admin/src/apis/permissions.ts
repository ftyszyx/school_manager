import type { PagingResponse } from '@/types/api'
import type {
  Permission,
  PermissionCreateRequest,
  PermissionListRequest,
  PermissionUpdateRequest
} from '@/types/permissions'
import request from '@/utils/request'

export const getPermissions = async (
  params: PermissionListRequest
): Promise<PagingResponse<Permission>> => {
  return (await request.get('/api/admin/permissions', { params })).data
}

export const getPermission = async (id: number): Promise<Permission> => {
  return (await request.get(`/api/admin/permissions/${id}`)).data
}

export const createPermission = async (data: PermissionCreateRequest): Promise<Permission> => {
  return (await request.post('/api/admin/permissions', data)).data
}

export const updatePermission = async (id: number, data: PermissionUpdateRequest): Promise<Permission> => {
  return (await request.put(`/api/admin/permissions/${id}`, data)).data
}

export const deletePermission = async (id: number): Promise<void> => {
  return (await request.delete(`/api/admin/permissions/${id}`)).data
}
