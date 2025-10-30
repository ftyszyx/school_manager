// app.js
import { login } from './utils/api';

App({
  onLaunch: function () {
    // 自动登录
    this.autoLogin();
  },

  autoLogin: function () {
    const token = wx.getStorageSync('token');
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

  login: function(userInfo) {
    wx.login({
      success: async (res) => {
        if (res.code) {
          try {
            const loginPayload = { code: res.code };
            if (userInfo) {
              loginPayload.userInfo = userInfo;
            }
            const loginRes = await login(loginPayload);
            if (loginRes && loginRes.token) {
              this.globalData.token = loginRes.token;
              wx.setStorageSync('token', loginRes.token);
              console.log('登录成功，token 已存储');
              // 可以在这里触发一个全局事件，通知页面登录成功
              if (this.userInfoReadyCallback) {
                this.userInfoReadyCallback(loginRes);
              }
            }
          } catch (error) {
            console.error("登录失败", error);
            wx.showToast({
              title: '登录失败',
              icon: 'none'
            });
          }
        } else {
          console.error('wx.login 失败：' + res.errMsg);
        }
      },
      fail: (err) => {
        console.error('wx.login 失败', err);
        wx.showToast({
          title: '微信登录调用失败',
          icon: 'none'
        });
      }
    });
  },

  globalData: {
    userInfo: null,
    token: null,
    apiBase: 'https://school.bytefuse.cn', // Should be configured based on environment
  }
})
