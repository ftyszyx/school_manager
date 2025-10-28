import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type { ListAppsParams, AppModel, AddAppReq, UpdateAppReq } from '@/types/apps'

export const fetchApps = async (params: ListAppsParams = {}) => {
    const response = await request.get('/admin/apps/list', { params }) as ApiResponse<PagingResponse<AppModel>>
    return response.data
}

export const createApp = async (payload: AddAppReq) => {
    const response = await request.post('/admin/apps', payload) as ApiResponse<AppModel>
    return response.data
}

export const updateApp = async (id: number, payload: UpdateAppReq) => {
    const response = await request.put(`/admin/apps/${id}`, payload) as ApiResponse<AppModel>
    return response.data
}

export const deleteApp = async (id: number) => {
    const response = await request.delete(`/admin/apps/${id}`) as ApiResponse<void>
    return response.data
}


