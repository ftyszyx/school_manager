

import request from '@/utils/request'
import type {  PagingResponse } from '@/types'
import type { UserListRequest, User, UserCreateRequest, UserUpdateRequest } from '@/types/user'

export const getUsers = async (params: UserListRequest): Promise<PagingResponse<User>> => {
  return (await request.get('/api/admin/users', { params })).data
}

export async function createUser(payload: UserCreateRequest): Promise<User> {
  return (await request.post('/api/admin/users', payload)).data
}

export async function updateUser(id: number, payload: UserUpdateRequest): Promise<User> {
  return (await request.put(`/api/admin/users/${id}`, payload)).data
}

export async function deleteUser(id: number): Promise<void> {
  return (await request.delete(`/api/admin/users/${id}`)).data
}
