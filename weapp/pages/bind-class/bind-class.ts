// pages/bind-class/bind-class.ts
import { bindClass } from '../../utils/api';

interface BindClassData {
	password: string;
}

Page<BindClassData, WechatMiniprogram.IAnyObject>({
	data: {
		password: '',
	},

	onPasswordInput(e: WechatMiniprogram.CustomEvent) {
		this.setData({ password: (e.detail as any) as string });
	},

	onSubmit: async function () {
		if (!this.data.password) {
			wx.showToast({ title: '请输入班级口令', icon: 'none' });
			return;
		}

		try {
			await bindClass({ password: this.data.password });
			wx.showToast({ title: '绑定成功', icon: 'success' });
			wx.navigateBack({ delta: 1 });
		} catch (error) {
			console.error('Bind class failed', error);
			wx.showToast({ title: '绑定失败，请检查口令', icon: 'none' });
		}
	},
});


