use crate::{setting, xeno_utils};



pub fn setup() {
    xeno_utils::create_config_dir();
}



// 1. 软件启动
// 2. 软件读取设置文件，读取上一次软件使用的预设名
// 3. 通过上一次预设名，读取预设文件，设置当前预设
// 4. 前端初始化，调用后端命令，设置当前预设到前端