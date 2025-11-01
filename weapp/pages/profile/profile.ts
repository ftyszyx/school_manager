import { getCurrentUser, getAllSchools, bindSchool, updateMyInfo } from '../../utils/api';

Page({
	data: {
		userInfo: {} as any,
		schools: [] as any[],
		schoolColumns: [] as any[],
		selectedSchool: null as any,
		schoolPassword: '',
		showPicker: false,
		showPasswordModal: false,
	},

	onLoad() {
		this.fetchUserInfo();
		this.fetchSchools();
	},

	async fetchUserInfo() {
		try {
			const userInfo = await getCurrentUser();
			// Ensure fields are not null for data binding
			userInfo.wechat_nickname = userInfo.wechat_nickname || '';
			userInfo.phone = userInfo.phone || '';
			this.setData({ userInfo });
		} catch (error) {
			wx.showToast({ title: '加载用户信息失败', icon: 'none' });
		}
	},

	async fetchSchools() {
		try {
			const schools = await getAllSchools();
			this.setData({
				schools,
				schoolColumns: schools.map((s) => s.name),
			});
		} catch (error) {
			wx.showToast({ title: '加载学校列表失败', icon: 'none' });
		}
	},

	onChooseAvatar(e: WechatMiniprogram.CustomEvent) {
		const { avatarUrl } = e.detail;
		// Note: This is a temporary path. For a real app, you'd upload this file
		// to your server/OSS and get a permanent URL back.
		this.setData({
			'userInfo.wechat_avatar_url': avatarUrl,
		});
	},

	onFieldInput(e: WechatMiniprogram.CustomEvent) {
		const { field } = e.currentTarget.dataset;
		const value = e.detail;
		this.setData({
			[`userInfo.${field}`]: value || '',
		});
	},

	async showSchoolPicker() {
		await this.fetchSchools();
		this.setData({ showPicker: !this.data.showPicker });
	},

	showPasswordModal() {
		this.setData({ showPasswordModal: !this.data.showPasswordModal });
	},

	onSchoolConfirm(e: WechatMiniprogram.CustomEvent) {
		const { value, index } = e.detail;
		this.setData({
			selectedSchool: this.data.schools[index],
			showPicker: false,
			showPasswordModal: true,
			schoolPassword: '',
		});
	},

	async onBindSchool() {
		if (!this.data.selectedSchool || !this.data.schoolPassword) {
			wx.showToast({ title: '请选择学校并输入密码', icon: 'none' });
			return;
		}

		try {
			await bindSchool({
				school_id: this.data.selectedSchool.id,
				password: this.data.schoolPassword,
			});
			wx.showToast({ title: '绑定成功', icon: 'success' });
			this.setData({ showPasswordModal: false });
			this.fetchUserInfo(); // Refresh user info
		} catch (error) {
			wx.showToast({ title: '绑定失败，请检查密码', icon: 'none' });
		}
	},

	async onSave() {
		try {
			console.log("onsave phone",this.data.userInfo.phone)
			console.log('onSave', this.data.userInfo);
			const payload: any = {
				wechat_nickname: this.data.userInfo.wechat_nickname,
				phone: this.data.userInfo.phone,
				wechat_avatar_url: this.data.userInfo.wechat_avatar_url,
			};
			await updateMyInfo(payload);
			wx.showToast({ title: '保存成功', icon: 'success' });
			wx.navigateBack();
		} catch (error) {
			wx.showToast({ title: '保存失败', icon: 'none' });
		}
	},
});
