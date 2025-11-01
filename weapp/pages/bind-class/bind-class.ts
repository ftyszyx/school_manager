// pages/bind-class/bind-class.ts
import { bindClass, getClassesBySchool, getCurrentUser } from '../../utils/api';

Page({
	data: {
		password: '',
		classes: [] as any[],
		classColumns: [] as any[],
		selectedClass: null as any,
		showPicker: false,
	},

	onLoad: async function () {
		try {
			const user = await getCurrentUser();
			console.log('User:', user);
			if (!user || !user.school_id) {
				wx.showToast({ title: '请先绑定学校', icon: 'none' });
				wx.navigateBack();
				return;
			}
			const classes = await getClassesBySchool(user.school_id);
			console.log('Fetched Classes:', classes);
			const classColumns = classes.map((c) => `${c.grade}年级 ${c.class}班 (${c.name})`);
			console.log('Picker Columns:', classColumns);
			this.setData({
				classes,
				classColumns: classColumns,
			});
		} catch (error) {
			console.error('Failed to load data', error);
			wx.showToast({ title: '加载班级列表失败', icon: 'none' });
		}
	},

	onPasswordInput(e: WechatMiniprogram.CustomEvent) {
		this.setData({ password: e.detail as unknown as string });
	},

	openClassPicker() {
		console.log('Opening picker');
		this.setData({ showPicker: true });
	},

	closeClassPicker() {
		console.log('Closing picker');
		this.setData({ showPicker: false });
	},

	onClassConfirm(e: WechatMiniprogram.CustomEvent) {
		const { index } = e.detail as { value: string; index: number };
		this.setData({
			selectedClass: this.data.classes[index],
		});
		this.closeClassPicker();
	},

	async onSubmit() {
		if (!this.data.selectedClass) {
			wx.showToast({ title: '请选择班级', icon: 'none' });
			return;
		}
		if (!this.data.password) {
			wx.showToast({ title: '请输入班级口令', icon: 'none' });
			return;
		}

		try {
			await bindClass({
				class_id: this.data.selectedClass.id,
				password: this.data.password,
			});
			wx.showToast({ title: '绑定成功', icon: 'success' });
			wx.navigateBack();
		} catch (error) {
			console.error('Bind class failed', error);
			wx.showToast({ title: '绑定失败，请检查口令', icon: 'none' });
		}
	},
});


