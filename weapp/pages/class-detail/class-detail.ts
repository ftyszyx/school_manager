// pages/class-detail/class-detail.ts
import { updateClassStatus, getCurrentUser } from '../../utils/api';

const app = getApp<IAppOption>();

interface ClassDetailData {
	classInfo: any | null;
	isTeacher: boolean;
	loading: boolean;
	statusOptions: { value: string; label: string; type: string }[];
	statusMap: Record<number, { text: string; type: string }>;
}

Page<ClassDetailData, WechatMiniprogram.IAnyObject>({
	data: {
		classInfo: null,
		isTeacher: false,
		loading: true,
		statusOptions: [
			{ value: '0', label: '已放学', type: 'default' },
			{ value: '1', label: '上课中', type: 'success' },
			{ value: '2', label: '放学中', type: 'warning' },
		],
		statusMap: {
			0: { text: '已放学', type: 'default' },
			1: { text: '上课中', type: 'success' },
			2: { text: '放学中', type: 'warning' },
		},
	},

	onLoad(options: Record<string, string>) {
		if (options.class) {
			const classInfo = JSON.parse(decodeURIComponent(options.class));
			this.setData({ classInfo, loading: false });
			this.checkUserRole();
			this.connectWebSocket();
		} else {
			wx.showToast({ title: '加载班级信息失败', icon: 'none', complete: () => wx.navigateBack() });
		}
	},

	onUnload() {
		if ((this as any).socketTask) {
			(this as any).socketTask.close({ code: 1000, reason: 'Page closed' });
		}
	},

	checkUserRole: async function () {
		try {
			const user = await getCurrentUser();
			const isTeacher = user.role_infos.some((role: any) => ['teacher', 'admin'].includes((role.name as string).toLowerCase()));
			this.setData({ isTeacher });
		} catch (error) {
			console.error('Failed to get user info', error);
		}
	},

	onStatusChange(e: WechatMiniprogram.CustomEvent) {
		const newStatus = parseInt((e.detail as any).value);
		const oldStatus = this.data.classInfo?.status;

		this.setData({ 'classInfo.status': newStatus } as any);

		updateClassStatus((this.data.classInfo as any).id, { status: newStatus })
			.then(() => wx.showToast({ title: '状态更新成功', icon: 'success' }))
			.catch((error) => {
				console.error('Failed to update status', error);
				this.setData({ 'classInfo.status': oldStatus } as any);
				wx.showToast({ title: '更新失败', icon: 'none' });
			});
	},

	connectWebSocket() {
		if (!this.data.classInfo || !(this.data.classInfo as any).id) return;
		const wsBaseUrl = app.globalData.apiBase.replace(/^http/, 'ws');
		const wsUrl = `${wsBaseUrl}/ws/class/${(this.data.classInfo as any).id}`;

		(this as any).socketTask = wx.connectSocket({
			url: wsUrl,
			header: { 'content-type': 'application/json' },
			success: () => console.log('WebSocket connecting...'),
			fail: (err) => console.error('WebSocket connection failed', err),
		});

		(this as any).socketTask.onOpen(() => console.log('WebSocket connected'));
		(this as any).socketTask.onMessage((res: any) => {
			console.log('Received WebSocket message:', res.data as any);
			try {
				const data = JSON.parse(res.data as string);
				if (data.event_type === 'CLASS_STATUS_UPDATE' && data.data && data.data.id === (this.data.classInfo as any).id) {
					this.setData({ 'classInfo.status': data.data.status } as any);
					wx.showToast({ title: '班级状态已更新', icon: 'none' });
				}
			} catch (e) {
				console.error('Error parsing WebSocket message:', e);
			}
		});

		(this as any).socketTask.onClose((res: any) => console.log('WebSocket closed', res));
		(this as any).socketTask.onError((err: any) => console.error('WebSocket error', err));
	},

	onCopy(e: WechatMiniprogram.BaseEvent) {
		const text = (e.currentTarget as any).dataset.text as string;
		wx.setClipboardData({ data: text, success: () => wx.showToast({ title: '已复制', icon: 'success' }) });
	},
});


