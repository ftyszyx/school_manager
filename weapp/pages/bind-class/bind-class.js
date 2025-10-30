// pages/bind-class/bind-class.js
import { bindClass } from '../../utils/api';

Page({
  data: {
    password: ''
  },

  onPasswordInput: function (e) {
    this.setData({
      password: e.detail
    });
  },

  onSubmit: async function () {
    if (!this.data.password) {
      wx.showToast({
        title: '请输入班级口令',
        icon: 'none'
      });
      return;
    }

    try {
      await bindClass({ password: this.data.password });
      wx.showToast({
        title: '绑定成功',
        icon: 'success'
      });
      // Navigate back to the previous page and refresh it
      wx.navigateBack({
        delta: 1
      });
    } catch (error) {
      console.error('Bind class failed', error);
      wx.showToast({
        title: '绑定失败，请检查口令',
        icon: 'none'
      });
    }
  }
})
