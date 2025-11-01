// pages/index/index.ts
import { getCurrentUser, unbindClass } from '../../utils/api';

const app = getApp<IAppOption>();

interface IndexData {
	classes: any[];
	loading: boolean;
	userInfo: any;
	hasUserInfo: boolean;
	canIUseGetUserProfile: boolean;
}

Page<IndexData, WechatMiniprogram.IAnyObject>({
	data: {
		classes: [],
		loading: true,
		userInfo: null,
		hasUserInfo: false,
		canIUseGetUserProfile: false,
	},

	onLoad() {
		if ((wx as any).getUserProfile) {
			this.setData({ canIUseGetUserProfile: true });
		}
	},

	onShow() {
		console.log('onShow');
		this.checkLoginStatus();
	},

	checkLoginStatus() {
		const token = wx.getStorageSync('token') as string | undefined;
		if (token) {
			console.log('token exists');
			this.fetchData();
		} else {
			app.userInfoReadyCallback = () => {
				this.fetchData();
			};
		}
	},

	onGoToProfile() {
		wx.navigateTo({ url: '/pages/profile/profile' });
	},

	async fetchData() {
		this.setData({ loading: true });
		try {
			const userInfo = await getCurrentUser();
			this.setData({
				userInfo: userInfo,
				hasUserInfo: true,
				classes: userInfo.class_infos,
			});
		} catch (error) {
			console.error('Failed to fetch data', error);
			wx.showToast({ title: '加载数据失败', icon: 'none' });
		} finally {
			this.setData({ loading: false });
		}
	},

	onClassClick(e: WechatMiniprogram.BaseEvent) {
		const { id } = (e.currentTarget as any).dataset as { id: number };
		const classInfo = this.data.classes.find((c: any) => c.id === id);
		if (classInfo) {
			wx.navigateTo({
				url: `/pages/class-detail/class-detail?class=` + encodeURIComponent(JSON.stringify(classInfo)),
			});
		}
	},

	onAddClass() {
		console.log('onAddClass');
		wx.navigateTo({ url: '/pages/bind-class/bind-class' });
	},

	onUnbind(e: WechatMiniprogram.BaseEvent) {
		const { id } = (e.currentTarget as any).dataset as { id: number };
		wx.showModal({
			title: '确认解绑',
			content: '您确定要解绑这个班级吗？',
			success: async (res) => {
				if (res.confirm) {
					try {
						await unbindClass(id);
						wx.showToast({ title: '解绑成功', icon: 'success' });
						this.fetchData();
					} catch (error) {
						console.error('Unbind failed', error);
						wx.showToast({ title: '解绑失败', icon: 'none' });
					}
				}
			},
		});
	},

	onPullDownRefresh() {
		this.fetchData().then(() => wx.stopPullDownRefresh());
	},

	slice(str: string, length: number) {
		console.log('slice', str, length);
		if (typeof str !== 'string' || str.length === 0) {
			return '';
		}
		return str.slice(0, length);
	},

});


