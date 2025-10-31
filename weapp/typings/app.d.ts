// Global app option typing for WeChat Mini Program
interface IAppOption {
	globalData: {
		userInfo: WechatMiniprogram.UserInfo | null;
		token: string | null;
		apiBase: string;
	};
	userInfoReadyCallback?: (data: any) => void;
	// Methods implemented in app.ts
	autoLogin: () => void;
	login: (userInfo?: any) => void;
}


