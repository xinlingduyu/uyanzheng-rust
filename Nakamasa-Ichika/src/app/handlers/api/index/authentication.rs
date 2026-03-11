//! 认证程序
use salvo::prelude::*;

use crate::app::utils::response::ApiResponse;

#[handler]
pub async fn authentication(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    // PHP源码中此函数为注释状态，功能已被禁用
    // 保留接口结构以兼容旧代码，返回提示信息
    res.render(Json(ApiResponse::<()>::error("认证功能已禁用，请直接使用U验证官网获取认证文件", 201)));
}