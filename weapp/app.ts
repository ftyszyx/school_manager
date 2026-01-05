// app.ts
import { wechatLogin } from './utils/api';

App<IAppOption>({
	onLaunch() {
		console.log('onLaunch');
		this.autoLogin();
	},

	autoLogin() {
		const token = wx.getStorageSync('token') as string | undefined;
		if (token) {
			wx.checkSession({
				success: () => {
					this.globalData.token = token;
					console.log('Token有效, 已登录');
				},
				fail: () => {
					this.appLogin();
				}
			});
		} else {
			this.appLogin();
		}
	},
	async wxlogin() {
			 return new Promise<string>((resolve, reject) => {
				wx.login({
					success: (res) => {
						if (res.code) {
							resolve(res.code);
						} else {
							reject(new Error('wx.login 失败：' + (res.errMsg || 'missing code')));
						}
					},
					fail: (err) => {
						reject(err);
					},
				});
			});

	},

	async appLogin() {
		try {
			const code = await this.wxlogin();
			const loginRes = await wechatLogin({ code });
			if (!loginRes || !loginRes.token) {
				throw new Error('登录返回缺少 token');
			}
			this.globalData.token = loginRes.token as string;
			wx.setStorageSync('token', loginRes.token as string);
			console.log('登录成功，token 已存储');
			if (this.userInfoReadyCallback) {
				this.userInfoReadyCallback(loginRes);
			}
			return loginRes.token as string;
		} catch (error) {
			console.error('登录失败', error);
			wx.showToast({ title: '登录失败', icon: 'none' });
			throw error;
		}
	},

	globalData: {
		userInfo: null,
		token: null,
		apiBase: 'https://school.bytefuse.cn',
	}
});


