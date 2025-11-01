// app.ts
import { login } from './utils/api';

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
					this.login();
				}
			});
		} else {
			this.login();
		}
	},

	login(userInfo?: any) {
		wx.login({
			success: async (res) => {
				if (res.code) {
					try {
						const loginPayload: any = { code: res.code };
						if (userInfo) {
							loginPayload.userInfo = userInfo;
						}
						const loginRes = await login(loginPayload);
						if (loginRes && loginRes.token) {
							this.globalData.token = loginRes.token as string;
							wx.setStorageSync('token', loginRes.token as string);
							console.log('登录成功，token 已存储');
							if (this.userInfoReadyCallback) {
								this.userInfoReadyCallback(loginRes);
							}
						}
					} catch (error) {
						console.error('登录失败', error);
						wx.showToast({ title: '登录失败', icon: 'none' });
					}
				} else {
					console.error('wx.login 失败：' + res.errMsg);
				}
			},
			fail: (err) => {
				console.error('wx.login 失败', err);
				wx.showToast({ title: '微信登录调用失败', icon: 'none' });
			}
		});
	},

	globalData: {
		userInfo: null,
		token: null,
		apiBase: 'https://school.bytefuse.cn',
	}
});


