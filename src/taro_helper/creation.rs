use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, prelude::*, Read, SeekFrom, Write};

use failure::{Error, ResultExt};
use inflector::Inflector;
use log::{info, warn};
use regex::Regex;
use string_template::Template;

///执行新增操作
pub fn execute_action() -> Result<(), Error> {
    //todo:增加可配置内容
    let mut config = String::new();
    info!("请根据提示进行输入: <file-name> <creation_type>");
    io::stdin()
        .read_line(&mut config)
        .with_context(|_| "读取create操作配置失败")?;
    let config_regex = Regex::new(r"^(?P<file_name>[\w-]+)\s+(?P<creation_type>\w+)")?;
    config_regex
        .captures(&config)
        .ok_or(failure::err_msg("请输入操作配置"))
        .and_then(|capture| {
            let file_name: &str = capture
                .name("file_name")
                .ok_or(failure::err_msg("请输入创建的名称"))?
                .into();
            let creation_type: &str = capture
                .name("creation_type")
                .ok_or(failure::err_msg("请输入创建类型"))?
                .into();
            let file_name = file_name.trim();
            let creation_type = creation_type.trim();

            let mut args = HashMap::new();
            if !file_name.is_kebab_case() {
                warn!("文件名称只支持kebab case");
                return execute_action();
            }
            args.insert("file_name", file_name);
            match creation_type {
                "p" | "page" => {
                    execute_create_page_action(&args)?;
                    super::safe_exit(execute_action)
                }
                "c" | "component" => {
                    execute_create_component_action(&args)?;
                    super::safe_exit(execute_action)
                }
                other => {
                    warn!("创建类型{}暂不支持", &other);
                    execute_action()
                }
            }
        })
}

const CREATION_STYLE_TEMPLATE: &'static str = r"
.{{class_name}}{
}
";

const CREATION_PAGE_TEMPLATE: &'static str = r"
import Taro, { Config } from '@tarojs/taro';
import { Text, View } from '@tarojs/components';
import { observer } from '@tarojs/mobx';
import { formatMsg } from '@/services';

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
    let style_content = Template::new(CREATION_STYLE_TEMPLATE).render(&args);
    let mut file = File::create(&full_style_path)?;
    file.write_all(style_content.as_bytes())?;
    //第三步，在pages文件夹下创建对应的页面文件（todo:需要判断是否多语言）
    let full_path = format!("src/pages/{}/index.tsx", file_name);
    let content = Template::new(CREATION_PAGE_TEMPLATE).render(&args);
    let mut file = File::create(&full_path)?;
    file.write_all(content.as_bytes())?;
    //第四步，为page页面增加路由
    let mut app_file = OpenOptions::new()
        .write(true)
        .read(true)
        .open("src/app.tsx")?;
    let mut app_contents = String::new();
    app_file.read_to_string(&mut app_contents)?;
    let pages_regex = Regex::new(r"pages: (?P<pages>\[[^\]]*,)\s*\]")?;
    let replace_pages = format!("pages: ${} \n'pages/{}/index',]", "pages", file_name);
    let new_content = pages_regex.replace_all(app_contents.as_str(), replace_pages.as_str());
    //将游标移动到文件开头
    app_file.seek(SeekFrom::Start(0))?;
    app_file
        .write_all(new_content.as_bytes())
        .with_context(|_| "写入配置信息失败")?;
    //第五步，为当前page增加多语言内容
    fs::read_dir("src/languages")?
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            let language_mod = OsStr::new("index.ts");
            !path.is_dir()
                && match path.file_name() {
                    Some(file_name) if file_name == language_mod => false,
                    _ => true,
                }
        })
        .for_each(|path| {
            let mut language_file = OpenOptions::new()
                .write(true)
                .read(true)
                .open(path)
                .unwrap();
            let mut file_content = String::new();
            language_file.read_to_string(&mut file_content).unwrap();
            let languages_regex =
                Regex::new(r"export default (?P<languages>\{[\s\S]*,)\s*\}\s*;\s*$").unwrap();
            let replace_page_language = format!(
                "export default ${} \n{}:{{title:''}},\n}};",
                "languages", name
            );
            let new_content =
                languages_regex.replace_all(file_content.as_str(), replace_page_language.as_str());
            language_file.seek(SeekFrom::Start(0)).unwrap();
            language_file.write_all(new_content.as_bytes()).unwrap();
        });
    crate::prettier_after_action()?;
    Ok(())
}

const CREATION_COMPONENT_TEMPLATE: &'static str = r"
import { ReactNode } from 'react';
import Taro, { PureComponent } from '@tarojs/taro';
import { View } from '@tarojs/components';

import './index.scss';

interface Props {}

interface State {}

export default class {{container_name}} extends PureComponent<Props,State> {
    static options = {
        addGlobalClass: true,
    };

    render():ReactNode {
        return (
            <View className='{{class_name}}'>
            </View>
        );
    }
}
";

pub fn execute_create_component_action(configs: &HashMap<&str, &str>) -> Result<(), Error> {
    let file_name = configs.get("file_name").unwrap();
    let mut args = HashMap::new();
    let container_name = file_name.to_class_case();
    let name = file_name.to_camel_case();
    let class_name = file_name.to_owned();
    args.insert("container_name", container_name.as_str());
    args.insert("name", name.as_str());
    args.insert("class_name", class_name);
    //第一步创建目录(如果不存在)
    let full_dir = format!("src/components/{}", file_name);
    fs::create_dir_all(&full_dir)?;
    //第二步，创建样式文件
    let full_style_path = format!("src/components/{}/index.scss", file_name);
    let style_content = Template::new(CREATION_STYLE_TEMPLATE).render(&args);
    let mut file = File::create(&full_style_path)?;
    file.write_all(style_content.as_bytes())?;
    //第三步，components文件夹下创建对应的页面文件
    let full_path = format!("src/components/{}/index.tsx", file_name);
    let content = Template::new(CREATION_COMPONENT_TEMPLATE).render(&args);
    let mut file = File::create(&full_path)?;
    file.write_all(content.as_bytes())?;
    //第四步，为components index 中导出相应组件
    let mut component_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("src/components/index.ts")?;
    let component_export = format!(
        "export {{default as {}}} from './{}';",
        container_name, file_name
    );
    component_file.write_all(component_export.as_bytes())?;
    crate::prettier_after_action()?;
    Ok(())
}
