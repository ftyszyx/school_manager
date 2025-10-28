import type { PagingResponse } from '@/types/api'
import type { Role, RoleCreateRequest, RoleListRequest, RoleUpdateRequest } from '@/types/roles'
import request from '@/utils/request'

export const getRoles = async (params: RoleListRequest): Promise<PagingResponse<Role>> => {
  return (await request.get('/api/admin/roles', { params })).data
}

export const getRole = async (id: number): Promise<Role> => {
  return (await request.get(`/api/admin/roles/${id}`)).data
}

export const createRole = async (data: RoleCreateRequest): Promise<Role> => {
  return (await request.post('/api/admin/roles', data)).data
}

export const updateRole = async (id: number, data: RoleUpdateRequest): Promise<Role> => {
  return (await request.put(`/api/admin/roles/${id}`, data)).data
}

export const deleteRole = async (id: number): Promise<void> => {
  return (await request.delete(`/api/admin/roles/${id}`)).data
}
