import { APP_OK, APP_USER_NOT_FOUND } from "../typings/const";
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

interface RequestOptions {
  url: string;
  method?: HttpMethod;
  data?: any;
  header?: Record<string, string>;
}
const handlerRelogin = () => {
  console.log("handlerRelogin");
  wx.removeStorageSync("token");
  const app = getApp<IAppOption>();
  if (app && app.login) {
    console.log("app.login");
    app.login();
  }
};

const request = <T = any>(options: RequestOptions): Promise<T> => {
  return new Promise((resolve, reject) => {
    const token = wx.getStorageSync("token") as string | undefined;
    const app = typeof getApp === "function" ? getApp<IAppOption>() : null;
    const apiBase = app && app.globalData ? app.globalData.apiBase : "";
    if (!apiBase) {
      console.warn("API base url is not configured");
    }

    wx.request({
      url: `${apiBase}${options.url}`,
      method: (options.method || "GET") as any,
      data: options.data || {},
      header: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(options.header || {}),
      },
      success: (res: WechatMiniprogram.RequestSuccessCallbackResult<any>) => {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          // @ts-ignore: common backend shape { data }
          if (res.data.code === APP_USER_NOT_FOUND) {
            handlerRelogin();
            reject(res.data);
            return;
          }
          console.log("res.data", res.data);
          if (res.data.code === APP_OK) {
            resolve((res.data as any).data ?? (res.data as any));
          } else {
            wx.showToast({ title: (res.data as any).message || "操作失败", icon: "none" });
            reject(res.data);
          }
        } else {
          if (res.statusCode === 401) {
            handlerRelogin();
            reject(res.data);
            return;
          }
          wx.showToast({ title: (res.data as any).message || "请求失败", icon: "none" });
          reject(res.data);
        }
      },
      fail: (err) => {
        wx.showToast({ title: "网络错误", icon: "none" });
        reject(err);
      },
    });
  });
};

export default request;
