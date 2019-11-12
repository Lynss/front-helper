use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process;

use failure::{Error, ResultExt};
use inflector::Inflector;
use log::{info, warn};
use regex::Regex;
use string_template::Template;

///执行新增操作
pub fn execute_action() -> Result<(), Error> {
    //todo:增加可配置内容
    let mut config = String::new();
    info!("请根据提示进行输入: <file-name>|<creation_type>");
    io::stdin()
        .read_line(&mut config)
        .with_context(|_| "读取create操作配置失败")?;
    let config_regex = Regex::new(r"^(?P<file_name>[\w-]+)\|(?P<creation_type>\w+)").unwrap();
    config_regex
        .captures(&config)
        .ok_or(failure::err_msg("请输入操作配置"))
        .and_then(|capture| {
            let file_name: &str = capture
                .name("file_name")
                .ok_or(failure::err_msg("请输入创建的名称"))?
                .into();
            let file_name = file_name.trim();
            let creation_type: &str = capture
                .name("creation_type")
                .ok_or(failure::err_msg("请输入创建类型"))?
                .into();
            let creation_type = creation_type.trim();

            let mut args = HashMap::new();
            if !file_name.is_kebab_case() {
                warn!("文件名称只支持kebab case");
                return execute_action();
            }
            args.insert("file_name", file_name);
            match creation_type {
                "page" => {
                    execute_create_page_action(&args)?;
                    let mut then_action = String::new();
                    info!("请进行下一步操作，输入q退出，c继续新增，其他返回上一步");
                    io::stdin()
                        .read_line(&mut then_action)
                        .with_context(|_| "读取后续操作失败")?;
                    match then_action.to_lowercase().trim() {
                        "q" => process::exit(exitcode::OK),
                        "c" => execute_action(),
                        _ => super::match_taro_action(),
                    }
                }
                other => {
                    warn!("创建类型{}暂不支持", &other);
                    execute_action()
                }
            }
        })
}

const CREATION_PAGE_STYLE_TEMPLATE: &'static str = r"
.{{class_name}}{
}
";

const CREATION_PAGE_TEMPLATE: &'static str = r"
import Taro, { Config } from '@tarojs/taro';
import { Text, View } from '@tarojs/components';
import { observer } from '@tarojs/mobx';
import { formatMsg } from '@/services';
import { CmHeader } from '@/components';

import LocalComponent from '../local-component';

import './index.scss';

interface Props {}

interface State {}

@observer
export default class {{container_name}} extends LocalComponent<Props,State> {
    config: Config = {
        navigationBarTitleText: '',
    };

    render() {
        return (
            <View className='{{class_name}}'>
                <CmHeader title={formatMsg('{{name}}.title')} />
                <Text>请手动修改navigationBarTitleText</Text>
            </View>
        );
    }
}
";

pub fn execute_create_page_action(configs: &HashMap<&str, &str>) -> Result<(), Error> {
    let file_name = configs.get("file_name").unwrap();
    let mut args = HashMap::new();
    let container_name = file_name.to_class_case();
    let name = file_name.to_camel_case();
    let class_name = file_name.to_owned();
    args.insert("container_name", container_name.as_str());
    args.insert("name", name.as_str());
    args.insert("class_name", class_name);
    //第一步创建目录(如果不存在)
    let full_dir = format!("src/pages/{}", file_name);
    fs::create_dir_all(&full_dir)?;
    //第二步，创建样式文件
    let full_style_path = format!("src/pages/{}/index.scss", file_name);
    let style_content = Template::new(CREATION_PAGE_STYLE_TEMPLATE).render(&args);
    let mut file = File::create(&full_style_path)?;
    file.write_all(style_content.as_bytes())?;
    //第三步，在pages文件夹下创建对应的页面文件
    let full_path = format!("src/pages/{}/index.tsx", file_name);
    let content = Template::new(CREATION_PAGE_TEMPLATE).render(&args);
    let mut file = File::create(&full_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
