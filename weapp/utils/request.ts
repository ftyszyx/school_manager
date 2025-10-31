type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

interface RequestOptions {
	url: string;
	method?: HttpMethod;
	data?: any;
	header?: Record<string, string>;
}

const request = <T = any>(options: RequestOptions): Promise<T> => {
	return new Promise((resolve, reject) => {
		const token = wx.getStorageSync('token') as string | undefined;
		const app = typeof getApp === 'function' ? getApp<IAppOption>() : null;
		const apiBase = app && app.globalData ? app.globalData.apiBase : '';
		if (!apiBase) {
			console.warn('API base url is not configured');
		}

		wx.request<T>({
			url: `${apiBase}${options.url}`,
			method: (options.method || 'GET') as any,
			data: options.data || {},
			header: {
				'Content-Type': 'application/json',
				...(token ? { Authorization: `Bearer ${token}` } : {}),
				...(options.header || {}),
			},
			success: (res) => {
				if (res.statusCode >= 200 && res.statusCode < 300) {
					// @ts-ignore: common backend shape { data }
					resolve((res.data as any).data ?? (res.data as any));
				} else {
					wx.showToast({ title: (res.data as any).message || '请求失败', icon: 'none' });
					reject(res.data);
				}
			},
			fail: (err) => {
				wx.showToast({ title: '网络错误', icon: 'none' });
				reject(err);
			},
		});
	});
};

export default request;


