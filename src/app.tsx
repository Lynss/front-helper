import './app.scss';
import '@/pollify';
import '@tarojs/async-await';

import { setGlobalData } from '@/global-data';
import Index from '@/pages';
import {
    api,
    CmRouter,
    formatTabBar,
    getCurrentRouteName,
    Log,
} from '@/services';
import { store } from '@/stores';
import { onError, Provider } from '@tarojs/mobx';
import Taro, { Component, Config } from '@tarojs/taro';
import { autorun } from 'mobx';

onError(error => {
    Log.warn("Mobx global error:", error);
});

const { auth, common } = store;

const DESIGN_WIDTH = 750;

// 如果需要在 h5 环境中开启 React Devtools
// 取消以下注释：
// if (process.env.NODE_ENV !== 'production' && process.env.TARO_ENV === 'h5')  {
//   require('nerv-devtools')
// }
class App extends Component {
    config: Config = {
        pages: [
            "pages/qr-demo/index",
            "pages/user/index",
            "pages/index/index",
            "pages/other/index",
            "pages/auth/index",
        ],
        window: {
            backgroundTextStyle: "dark",
            navigationBarBackgroundColor: "#fff",
            navigationBarTextStyle: "black",
        },
        tabBar: {
            list: [
                {
                    pagePath: "pages/index/index",
                    text: "首页",
                    iconPath: "resources/imgs/tabs/wait.png",
                    selectedIconPath: "resources/imgs/tabs/wait-active.png",
                },
                {
                    pagePath: "pages/qr-demo/index",
                    text: "二维码demo",
                    iconPath: "resources/imgs/tabs/wait.png",
                    selectedIconPath: "resources/imgs/tabs/wait-active.png",
                },
                {
                    pagePath: "pages/user/index",
                    text: "我的",
                    iconPath: "resources/imgs/tabs/user.png",
                    selectedIconPath: "resources/imgs/tabs/user-active.png",
                },
            ],
            color: "#333",
            selectedColor: "#c0311a",
            backgroundColor: "#F7F7F7",
            borderStyle: "white",
        },
    };

    componentDidMount() {
        this.initGlobalData();
        this.initObserver();
    }

    componentWillUnmount() {
        this.authDisposer && this.authDisposer();
        this.localizeDisposer && this.localizeDisposer();
    }

    componentDidShow() {}

    componentDidHide() {}

    authDisposer;
    localizeDisposer;

    initGlobalData = () => {
        const settings = Taro.getSystemInfoSync();
        const model = settings.model;
        if (model) {
            const special = /iphone.*[x|xr|11]/.test(
                model.toLowerCase().trim(),
            );
            //是否异形屏
            setGlobalData("special", special);
        }
        //像素比例
        setGlobalData("ratio", DESIGN_WIDTH / settings.windowWidth);
    };

    initObserver = () => {
        this.authDisposer = autorun(() => {
            //如果未登录并且当前不在登录页，跳转登录
            if (!auth.isLogin && getCurrentRouteName(this) !== "auth") {
                //h5里需要做一个延迟
                setTimeout(() => {
                    CmRouter.navigate("auth");
                }, 200);
            } else if (auth.token) {
                //登录时需要设置请求头
                api.setHeader("Authorization", `Bearer ${auth.token}`);
            }
        });
        this.localizeDisposer = autorun(() => {
            if (common.currentLanguage) {
                //语言发生变化时
                api.setHeader("Accept-Language", common.currentLanguage);
                //修改tab页语言
                formatTabBar();
            }
        });
    };

    // 在 App 类中的 render() 函数没有实际作用
    // 请勿修改此函数
    render() {
        return (
            <Provider store={store}>
                <Index />
            </Provider>
        );
    }
}

Taro.render(<App />, document.getElementById("app"));
