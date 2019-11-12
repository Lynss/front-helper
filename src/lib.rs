use failure::{Error, ResultExt};
use log::{info, warn};
use std::io;
use std::process::Command;

mod structs;
mod taro_helper;

const INFO_LENGTH: usize = 80;

///使提示信息能够对齐
pub fn format_information(info: &str) -> String {
    let len = info.len();
    //让奇偶能整除
    let trimmer = if len % 2 == 0 { "=" } else { "" };
    let formatter = "=".repeat((INFO_LENGTH - len) / 2);
    format!("{}{}{}{}", &formatter, info, &formatter, trimmer)
}

///执行文件操作前，先用git add命令来确保对项目的修改能随时回退
pub fn stage_before_action() -> Result<(), Error> {
    info!("进行后续操作前，将会执行git add .，请确定是否继续执行（输入N放弃后续操作）");
    let mut continuable = String::new();
    io::stdin()
        .read_line(&mut continuable)
        .with_context(|_| "读取用户输入失败")?;
    let continuable = continuable.to_uppercase().trim() != "N";
    if !continuable {
        return Err(failure::err_msg("用户放弃后续操作"));
    };
    Command::new("git")
        .arg("add")
        .arg(".")
        .output()
        .with_context(|_| "git add . 执行失败")?;
    Ok(())
}

//匹配框架名
pub fn match_framework<'a>(framework: &'a str) -> Result<(), Error> {
    let supported_frameworks: Vec<&str> = vec!["taro"];
    match framework.to_lowercase().trim() {
        "taro" => {
            info!("{}", format_information("taro-helper启动"));
            taro_helper::match_taro_action()
        }
        other => {
            warn!(
                "暂不支持框架<{}>，请联系ly1169134156@163.com添加相关支持",
                other
            );
            info!(
                "请输入你当前使用的框架，（目前支持：{:?}）",
                supported_frameworks
            );
            let mut framework = String::new();
            io::stdin()
                .read_line(&mut framework)
                .with_context(|_| "读取框架名称失败")?;
            match_framework(framework.as_str())
        }
    }
}
