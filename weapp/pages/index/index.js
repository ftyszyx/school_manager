// pages/index/index.js
import { getCurrentUser, getMyClasses, unbindClass } from '../../utils/api';
const app = getApp();

Page({
  data: {
    classes: [],
    loading: true,
    userInfo: null,
    hasUserInfo: false,
    canIUseGetUserProfile: false,
  },

  onLoad: function () {
    if (wx.getUserProfile) {
      this.setData({
        canIUseGetUserProfile: true
      })
    }
  },

  onShow: function () {
    this.checkLoginStatus();
  },
  
  checkLoginStatus() {
    const token = wx.getStorageSync('token');
    if (token) {
      this.fetchData();
    } else {
      // 等待 app.js 中的 login 调用完成
      app.userInfoReadyCallback = res => {
        this.fetchData();
      };
    }
  },

  async fetchData() {
    this.setData({ loading: true });
    try {
      const userInfo = await getCurrentUser();
      const classes = await getMyClasses();
      this.setData({ 
        userInfo: userInfo,
        hasUserInfo: true,
        classes: classes || [], 
      });
    } catch (error) {
      console.error("Failed to fetch data", error);
      wx.showToast({
        title: '加载数据失败',
        icon: 'none'
      });
    } finally {
      this.setData({ loading: false });
    }
  },
  
  getUserProfile(e) {
    // 推荐使用wx.getUserProfile获取用户信息，开发者每次通过该接口获取用户个人信息均需用户确认
    // 开发者妥善保管用户快速填写的头像昵称，避免重复弹窗
    wx.getUserProfile({
      desc: '用于完善会员资料', 
      success: (res) => {
        const userInfo = {
          nickName: res.userInfo.nickName,
          avatarUrl: res.userInfo.avatarUrl,
        };
        this.setData({
          userInfo: res.userInfo,
          hasUserInfo: true
        });
        // 在此处调用 app.js 中的 login 方法，并传递 userInfo
        app.login(userInfo);
        this.fetchData();
      }
    })
  },


  onClassClick: function (e) {
    const { id } = e.currentTarget.dataset;
    const classInfo = this.data.classes.find(c => c.id === id);
    if (classInfo) {
      wx.navigateTo({
        url: `/pages/class-detail/class-detail?class=` + encodeURIComponent(JSON.stringify(classInfo)),
      });
    }
  },

  onAddClass: function () {
    wx.navigateTo({
      url: '/pages/bind-class/bind-class',
    });
  },

  onUnbind: function (e) {
    const { id } = e.currentTarget.dataset;
    wx.showModal({
      title: '确认解绑',
      content: '您确定要解绑这个班级吗？',
      success: async (res) => {
        if (res.confirm) {
          try {
            await unbindClass(id);
            wx.showToast({
              title: '解绑成功',
              icon: 'success'
            });
            this.fetchData(); // Refresh data after unbinding
          } catch (error) {
            console.error('Unbind failed', error);
            wx.showToast({
              title: '解绑失败',
              icon: 'none'
            });
          }
        }
      }
    });
  },

  onPullDownRefresh: function () {
    this.fetchData().then(() => {
      wx.stopPullDownRefresh();
    });
  }
})
