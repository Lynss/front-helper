use std::io;
use std::process;

use failure::{Error, ResultExt};
use log::{info, warn};

mod creation;

pub fn match_taro_action() -> Result<(), Error> {
    let supported_action = vec!["c for create"];
    info!(
        "请参照提示输入一个能被taro-helper支持的操作（目前支持{:?}）",
        supported_action
    );
    let mut action = String::new();
    io::stdin()
        .read_line(&mut action)
        .with_context(|_| "读取taro-helper操作类型失败")?;
    match action.to_lowercase().trim() {
        action if action == "c" || action == "create" => {
            crate::stage_before_action().and_then(|_| creation::execute_action())
        }
        // action if action == "d" || action == "delete" => execute_taro_delete_action(),
        other => {
            warn!(
                "taro-helper暂不支持操作<{}>，请联系ly1169134156@163.com添加相关支持",
                other
            );
            match_taro_action()
        }
    }
}

pub fn safe_exit(continue_action: impl Fn() -> Result<(), Error>) -> Result<(), Error> {
    let mut then_action = String::new();
    info!("请进行下一步操作，输入q退出，c继续新增，其他返回上一步");
    io::stdin()
        .read_line(&mut then_action)
        .with_context(|_| "读取后续操作失败")?;
    match then_action.to_lowercase().trim() {
        "q" => process::exit(exitcode::OK),
        "c" => continue_action(),
        _ => match_taro_action(),
    }
}
