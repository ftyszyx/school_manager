// pages/class-detail/class-detail.js
import { updateClassStatus, getCurrentUser } from '../../utils/api';

const app = getApp();

Page({
  data: {
    classInfo: null,
    isTeacher: false, // We'll determine this based on user roles
    loading: true,
    statusOptions: [
      { value: '0', label: '已放学', type: 'default' },
      { value: '1', label: '上课中', type: 'success' },
      { value: '2', label: '放学中', type: 'warning' },
    ],
    statusMap: {
      0: { text: '已放学', type: 'default' },
      1: { text: '上课中', type: 'success' },
      2: { text: '放学中', type: 'warning' }
    }
  },

  onLoad: function (options) {
    if (options.class) {
      const classInfo = JSON.parse(decodeURIComponent(options.class));
      this.setData({
        classInfo: classInfo,
        loading: false
      });
      this.checkUserRole();
      this.connectWebSocket();
    } else {
      wx.showToast({
        title: '加载班级信息失败',
        icon: 'none',
        complete: () => {
          wx.navigateBack();
        }
      })
    }
  },
  
  onUnload: function () {
    // Close WebSocket connection when leaving the page
    if (this.socketTask) {
      this.socketTask.close({
        code: 1000,
        reason: 'Page closed'
      });
    }
  },

  checkUserRole: async function() {
    try {
      const user = await getCurrentUser();
      // Assuming teacher role name is 'teacher' or 'admin', this should be configured
      const isTeacher = user.role_infos.some(role => ['teacher', 'admin'].includes(role.name.toLowerCase()));
      this.setData({ isTeacher });
    } catch (error) {
      console.error('Failed to get user info', error);
    }
  },

  onStatusChange: function (e) {
    const newStatus = parseInt(e.detail.value);
    const oldStatus = this.data.classInfo.status;
    
    // Optimistic update
    this.setData({
      'classInfo.status': newStatus
    });

    updateClassStatus(this.data.classInfo.id, { status: newStatus })
      .then(() => {
        wx.showToast({
          title: '状态更新成功',
          icon: 'success'
        });
      })
      .catch((error) => {
        console.error('Failed to update status', error);
        // Rollback on failure
        this.setData({
          'classInfo.status': oldStatus
        });
        wx.showToast({
          title: '更新失败',
          icon: 'none'
        });
      });
  },

  connectWebSocket: function () {
    if (!this.data.classInfo || !this.data.classInfo.id) return;
    const wsBaseUrl = app.globalData.apiBase.replace(/^http/, 'ws');
    const wsUrl = `${wsBaseUrl}/ws/class/${this.data.classInfo.id}`;

    this.socketTask = wx.connectSocket({
      url: wsUrl,
      header:{
        'content-type': 'application/json'
      },
      success: () => {
        console.log('WebSocket connecting...');
      },
      fail: (err) => {
        console.error('WebSocket connection failed', err);
      }
    });

    this.socketTask.onOpen(() => {
      console.log('WebSocket connected');
    });

    this.socketTask.onMessage((res) => {
      console.log('Received WebSocket message:', res.data);
      try {
        const data = JSON.parse(res.data);
        if (data.event_type === 'CLASS_STATUS_UPDATE' && data.data && data.data.id === this.data.classInfo.id) {
          this.setData({
            'classInfo.status': data.data.status
          });
          wx.showToast({
            title: '班级状态已更新',
            icon: 'none'
          });
        }
      } catch (e) {
        console.error('Error parsing WebSocket message:', e);
      }
    });

    this.socketTask.onClose((res) => {
      console.log('WebSocket closed', res);
    });

    this.socketTask.onError((err) => {
      console.error('WebSocket error', err);
    });
  },
  
  onCopy: function(e) {
    const text = e.currentTarget.dataset.text;
    wx.setClipboardData({
      data: text,
      success: () => {
        wx.showToast({
          title: '已复制',
          icon: 'success'
        });
      }
    });
  }
})
