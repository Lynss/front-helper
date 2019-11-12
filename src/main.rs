use dotenv::dotenv;
use exitfailure::ExitFailure;
use human_panic::setup_panic;
use log::info;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "front-helper", about = "一个方便前端开发的工具")]
pub struct Cli {
    #[structopt(
        short,
        long,
        name = "框架名",
        help = "项目使用的框架（当前只支持taro）",
        default_value = "taro"
    )]
    pub framework: String,
}

fn main() -> Result<(), ExitFailure> {
    setup_panic!();
    dotenv().ok();
    env_logger::init();
    info!("{}", front_helper::format_information("工具启动"));
    let args = Cli::from_args();
    let framework = &args.framework;
    Ok(front_helper::match_framework(framework.as_str())?)
}
