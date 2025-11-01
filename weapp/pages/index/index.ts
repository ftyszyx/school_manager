// pages/index/index.ts
import { getCurrentUser, unbindClass, updateClassStatus } from '../../utils/api';

const app = getApp<IAppOption>();

interface IndexData {
	classes: any[];
	loading: boolean;
	userInfo: any;
	hasUserInfo: boolean;
	canIUseGetUserProfile: boolean;
	statusOptions: { value: string; label: string }[];
	statusMap: Record<number, { text: string; type: string }>;
	updatingStatusId: number | null;
}

Page<IndexData, WechatMiniprogram.IAnyObject>({
	data: {
		classes: [],
		loading: true,
		userInfo: null,
		hasUserInfo: false,
		canIUseGetUserProfile: false,
		statusOptions: [
			{ value: '0', label: '已放学' },
			{ value: '1', label: '上课中' },
			{ value: '2', label: '放学中' },
		],
		statusMap: {
			0: { text: '已放学', type: 'default' },
			1: { text: '上课中', type: 'success' },
			2: { text: '放学中', type: 'warning' },
		},
		updatingStatusId: null,
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
			const isTeacher = (userInfo?.role_infos || []).some((role: any) =>
				['teacher', 'admin'].includes((role.name || '').toLowerCase()),
			);
			const normalizedClasses = (userInfo.class_infos || []).map((c: any) => ({
				...c,
				id: c.id !== undefined && c.id !== null ? Number(c.id) : c.class_id !== undefined ? Number(c.class_id) : undefined,
				status: c.status !== undefined && c.status !== null ? Number(c.status) : 0,
			}));
			this.setData({
				userInfo: userInfo,
				hasUserInfo: true,
				classes: normalizedClasses,
				isTeacher,
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

	onStatusChange(e: WechatMiniprogram.CustomEvent) {
		const { classId, classIndex } = (e.currentTarget as any).dataset as { classId: string; classIndex: string };
		const id = Number(classId);
		const index = Number(classIndex);
		if (Number.isNaN(id) || Number.isNaN(index)) {
			console.warn('Invalid dataset for status change', e.currentTarget.dataset);
			return;
		}
		const detail = e.detail as any;
		const value = detail && typeof detail === 'object' && 'value' in detail ? detail.value : detail;
		const newStatus = Number(value);
		const classItem = this.data.classes[index];
		if (!classItem) {
			return;
		}
		const oldStatus = classItem.status;
		if (Number.isNaN(newStatus) || oldStatus === newStatus) {
			if (Number.isNaN(newStatus)) {
				console.warn('Invalid status value from event', e.detail);
			}
			return;
		}

		this.setData({
			updatingStatusId: id,
			[`classes[${index}].status`]: newStatus,
		});
		console.log('onStatusChange', id, newStatus);

		updateClassStatus(id, { status: newStatus })
			.then(() => {
				wx.showToast({ title: '状态更新成功', icon: 'success' });
				this.setData({ updatingStatusId: null });
			})
			.catch((error) => {
				console.error('Failed to update status', error);
				this.setData({ [`classes[${index}].status`]: oldStatus, updatingStatusId: null });
				wx.showToast({ title: '更新失败', icon: 'none' });
			});
	},

	slice(str: string, length: number) {
		console.log('slice', str, length);
		if (typeof str !== 'string' || str.length === 0) {
			return '';
		}
		return str.slice(0, length);
	},

});


