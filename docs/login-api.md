# 登录失败响应

`POST /api/login` 请求字段继续使用 `username`（邮箱）、`password`、
`public_key`、`aes_key`、`iv`。成功响应保持为用户 JSON，密码不会序列化。

登录失败使用非 2xx HTTP 状态，响应类型为 `application/json`：

```json
{
  "status": "error",
  "code": 40102,
  "message": "用户密码错误"
}
```

| HTTP 状态 | 错误码 | 含义 |
| --- | --- | --- |
| 400 | 40001 | 缺少或空白参数、JSON 格式或字段类型错误 |
| 400 | 40002 | 密码解密失败或加密参数无效 |
| 400 | 40003 | 公钥对应的登录密钥不存在，需刷新页面 |
| 401 | 40101 | 用户不存在或已删除 |
| 401 | 40102 | 用户密码错误 |
| 403 | 40301 | 用户账号被冻结 |
| 500 | 50001 | 数据库、连接池或 Session 服务异常 |

`user_status=1` 表示正常，`user_status=0` 表示冻结；其他未知状态也拒绝登录。
冻结检查在密码验证和 Session 创建之前执行。数据库查询失败不会返回“用户不存在”，
底层异常仅记录在服务日志中。

前端登录页使用后端 `message` 显示错误弹窗。登录请求关闭全局重复提示，
网络故障使用本地兜底文案；加密失败时停止发送登录请求。

## 验证

后端：`cargo test --locked --all-targets`

前端：`pnpm test --watchAll=false --runInBand --runTestsByPath src/pages/User.test.js src/utils/common.test.js`

测试覆盖错误码/HTTP 状态/JSON 文案、缺失和畸形请求、正常/冻结/已删除账号、
密码和解密错误，以及前端弹窗和重复提示抑制。使用合成用户和测试密钥，不修改真实用户或数据库。
