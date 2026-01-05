import { APP_OK, APP_USER_NOT_FOUND } from "../typings/const";
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

interface RequestOptions {
  url: string;
  method?: HttpMethod;
  data?: any;
  header?: Record<string, string>;
}
let reloginInFlight: Promise<string> | null = null;

const getGlobalApp = () => {
  const app = typeof getApp === "function" ? getApp<IAppOption>() : null;
  return app;
};


const getBaseUrl = () => {
  const app = getGlobalApp();
  if (!app) {
    throw new Error("Global app instance not found");
  }
  const apiBase = app && app.globalData ? app.globalData.apiBase : "";
  if (!apiBase) {
    throw new Error("API base url is not configured");
  }
  return apiBase;
}

const shouldRequireAuth = (url: string) => url.startsWith("/api/admin");

const requestOnce = <T = any>(options: RequestOptions, token?: string): Promise<T> => {
  return new Promise((resolve, reject) => {
    const baseurl=getBaseUrl();
    if (!baseurl) {
      reject(new Error("API base url is not configured"));
      return;
    }
    const url = `${baseurl}${options.url}`;
    console.log("request url", url);
    wx.request({
      url,
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
            reject({ statusCode: 401, data: res.data });
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
            reject({ statusCode: 401, data: res.data });
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

const request = async <T = any>(options: RequestOptions): Promise<T> => {
  const app = getGlobalApp();
  if (!app) {
    throw new Error("Global app instance not found");
  }
  let token = wx.getStorageSync("token") as string | undefined;
  if (!token && shouldRequireAuth(options.url)) {
    try {
      token = await app.appLogin();
    } catch (e) {
      console.error("Login failed", e);
      throw e;
    }
  }

  try {
    return await requestOnce<T>(options, token);
  } catch (err: any) {
    if (err && typeof err === "object" && err.statusCode === 401) {
      const newToken = await app.appLogin();
      return await requestOnce<T>(options, newToken);
    }
    throw err;
  }
};

export default request;
