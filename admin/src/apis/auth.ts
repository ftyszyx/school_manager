import request from "@/utils/request"
import type { ApiResponse, AuthPayload, AuthResponse, RegisterPayload, ChangePasswordPayload } from "@/types"


export const sentLogin = async (payload: AuthPayload)=> {
    const response = await request.post('/api/login', payload) as ApiResponse<AuthResponse>
    return response.data
}

export const sentRegister = async (payload: RegisterPayload) => {
    const response = await request.post('/api/register', payload) as ApiResponse<AuthResponse>
    return response.data
}

export const sentLogout = async () => {
    const response = await request.post('/api/admin/logout') as ApiResponse<AuthResponse>
    return response.data
}

export const sentGetUserInfo = async () => {
    const response = await request.get('/api/admin/me') as ApiResponse<AuthResponse>
    return response.data
}

export const changeMyPassword = async (payload: ChangePasswordPayload) => {
    const response = await request.post('/api/admin/me/password', payload) as ApiResponse<boolean>
    return response.data
}