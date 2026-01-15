// app.ts
import { wechatLogin } from './utils/api';

App<IAppOption>({
	onLaunch() {
		console.log('onLaunch');
		this.checkForUpdate();
		this.autoLogin();
	},

	checkForUpdate() {
		if (!wx.canIUse || !wx.canIUse('getUpdateManager')) {
			return;
		}

		const updateManager = wx.getUpdateManager();

		updateManager.onCheckForUpdate(function (res) {
			console.log('hasUpdate', res.hasUpdate);
		});

		updateManager.onUpdateReady(function () {
			wx.showModal({
				title: '更新提示',
				content: '新版本已经准备好，请重启应用以继续使用。',
				showCancel: false,
				success() {
					updateManager.applyUpdate();
				}
			});
		});

		updateManager.onUpdateFailed(function () {
			wx.showModal({
				title: '更新失败',
				content: '新版本下载失败，请稍后重试或重新打开小程序。',
				showCancel: false,
			});
		});
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
		// apiBase: 'https://frptest.bytefuse.cn',
	}
});


