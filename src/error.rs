use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("无法获取 HOME 环境变量")]
    HomeEnv(#[from] std::env::VarError),

    #[error("文件操作失败：{0}")]
    Io(#[from] std::io::Error),

    #[error("命令执行失败，退出码: {0}")]
    CommandFailed(i32),

    #[error("参数 -s 需要指定搜索词")]
    InvalidArgument,
}

pub type Result<T> = std::result::Result<T, Error>;
