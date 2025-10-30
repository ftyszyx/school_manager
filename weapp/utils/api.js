import request from './request';

// Auth
export const login = (data) => request({ url: '/auth/login/wechat', method: 'POST', data });

// User
export const getCurrentUser = () => request({ url: '/users/current' });

// Classes
export const getMyClasses = () => request({ url: '/users/my-classes' });
export const getClassDetail = (id) => request({ url: `/classes/${id}` });
export const updateClassStatus = (id, data) => request({ url: `/classes/${id}/status`, method: 'PUT', data });
export const bindClass = (data) => request({ url: '/users/bind-class', method: 'POST', data });
export const unbindClass = (id) => request({ url: `/users/unbind-class/${id}`, method: 'DELETE' });
