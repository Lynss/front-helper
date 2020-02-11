import { ReactNode } from "react";
import Taro, { PureComponent } from "@tarojs/taro";
import { View } from "@tarojs/components";

import "./index.scss";

interface Props {}

interface State {}

export default class TestAbcde extends PureComponent<Props, State> {
  static options = {
    addGlobalClass: true
  };

  render(): ReactNode {
    return <View className="test-abcde"></View>;
  }
}
