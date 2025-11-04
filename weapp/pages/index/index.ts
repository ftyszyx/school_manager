// pages/index/index.ts
import { getCurrentUser, unbindClass, updateClassStatus } from '../../utils/api';
const app = getApp<IAppOption>();
const BASE_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 15000;
let socketTask: WechatMiniprogram.SocketTask | null = null;
let reconnectTimer: number | null = null;
let reconnectAttempts = 0;
let shouldReconnect = true;
let currentSchoolId: number | null = null;
let manualClose = false;

interface IndexData {
	classes: any[];
	loading: boolean;
	userInfo: any;
	hasUserInfo: boolean;
	canIUseGetUserProfile: boolean;
	statusOptions: { value: string; label: string }[];
	statusMap: Record<number, { text: string; type: string }>;
	updatingStatusId: number | null;
	schoolId: number | null;
	hasSchool: boolean;
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
		schoolId: null,
		hasSchool: false,
	},

	onLoad() {
		// if ((wx as any).getUserProfile) {
		// 	this.setData({ canIUseGetUserProfile: true });
		// }
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

	clearReconnectTimer() {
		if (reconnectTimer) {
			clearTimeout(reconnectTimer);
			reconnectTimer = null;
		}
	},

	cleanupSocket() {
		shouldReconnect = false;
		this.clearReconnectTimer();
		if (socketTask) {
			manualClose = true;
			socketTask.close({ reason: 'page-unload' });
			socketTask = null;
		}
		currentSchoolId = null;
	},

	scheduleReconnect(schoolId: number) {
		if (!shouldReconnect || reconnectTimer) {
			return;
		}
		const delay = Math.min(BASE_RECONNECT_DELAY * Math.pow(2, reconnectAttempts), MAX_RECONNECT_DELAY);
		reconnectTimer = setTimeout(() => {
			reconnectTimer = null;
			reconnectAttempts += 1;
			if (shouldReconnect) {
				this.openSocket(schoolId);
			}
		}, delay);
	},

	determineSchoolId(userInfo: any, classes: any[]): number | null {
		const direct = userInfo && (userInfo.school_id ?? userInfo.schoolId);
		if (direct !== undefined && direct !== null) {
			const parsed = Number(direct);
			if (!Number.isNaN(parsed)) {
				return parsed;
			}
		}
		const found = classes.find((c: any) => c && (c.school_id !== undefined || c.schoolId !== undefined));
		if (found) {
			const parsed = Number(found.school_id ?? found.schoolId);
			if (!Number.isNaN(parsed)) {
				return parsed;
			}
		}
		return null;
	},

	openSocket(schoolId: number) {
		const base = app && app.globalData ? app.globalData.apiBase : '';
		console.log('openSocket',base);
		if (!base) {
			console.warn('API base url is not configured');
			return;
		}
		const token = wx.getStorageSync('token') as string | undefined;
		const wsBase = base.replace(/^http/, 'ws');
		const url = `${wsBase}/ws/school/${schoolId}`;
		console.log('openSocket',url);
		if (socketTask) {
			manualClose = true;
			socketTask.close({ reason: 'reconnect' });
			socketTask = null;
		}
		this.clearReconnectTimer();
		shouldReconnect = true;
		socketTask = wx.connectSocket({ url, header: token ? { Authorization: `Bearer ${token}` } : {} });
		socketTask.onOpen(() => {
			reconnectAttempts = 0;
		});
		socketTask.onMessage((event) => {
			this.handleSocketMessage(event);
		});
		socketTask.onError((err) => {
			console.error('WebSocket error', err);
			socketTask = null;
			if (shouldReconnect) {
				this.scheduleReconnect(schoolId);
			}
		});
		socketTask.onClose((evt) => {
			socketTask = null;
			if (manualClose) {
				manualClose = false;
				shouldReconnect = true;
				return;
			}
			if (shouldReconnect) {
				this.scheduleReconnect(schoolId);
			}
		});
	},

	handleSocketMessage(event: WechatMiniprogram.SocketTask.OnMessageCallbackResult) {
		const message = typeof event.data === 'string' ? event.data : '';
		if (!message) {
			return;
		}
		try {
			const parsed = JSON.parse(message);
			const payload = parsed && typeof parsed === 'object' && 'data' in parsed ? (parsed as any).data : parsed;
			const rawClassId = (payload as any).class_id ?? (payload as any).classId ?? (payload as any).id;
			const rawStatus = (payload as any).new_status ?? (payload as any).newStatus ?? (payload as any).status;
			const classId = Number(rawClassId);
			const status = Number(rawStatus);
			if (Number.isNaN(classId) || Number.isNaN(status)) {
				return;
			}
			const index = this.data.classes.findIndex((item: any) => {
				const itemId = Number(item.id ?? item.class_id ?? item.classId);
				return !Number.isNaN(itemId) && itemId === classId;
			});
			if (index >= 0) {
				this.setData({ [`classes[${index}].status`]: status });
			}
		} catch (error) {
			console.warn('Invalid websocket payload', error);
		}
	},

	ensureRealtimeConnection(schoolId: number | null) {
		console.log('ensureRealtimeConnection', schoolId);
		const normalized = schoolId !== null && !Number.isNaN(schoolId) ? schoolId : null;
		this.setData({ schoolId: normalized, hasSchool: normalized !== null });
		if (normalized === null) {
			this.cleanupSocket();
			return;
		}
		shouldReconnect = true;
		if (currentSchoolId !== normalized || !socketTask) {
			currentSchoolId = normalized;
			reconnectAttempts = 0;
			this.openSocket(normalized);
		}
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
			const schoolId = this.determineSchoolId(userInfo, normalizedClasses);
			this.ensureRealtimeConnection(schoolId);
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

	onUnload() {
		this.cleanupSocket();
	},

	onHide() {
		shouldReconnect = false;
		this.clearReconnectTimer();
	},

	onShow() {
		console.log('onShow');
		shouldReconnect = true;
		this.clearReconnectTimer();
		if (socketTask === null && this.data.schoolId) {
			this.scheduleReconnect(this.data.schoolId);
		}
		this.checkLoginStatus();
	},

});
