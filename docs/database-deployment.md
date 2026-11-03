# PostgreSQL 部署配置

真机、Docker 和 Podman 使用相同的环境变量。后端进程启动时会校验配置和证书文件。

| 环境变量 | 用途 | 默认值 |
| --- | --- | --- |
| `DATABASE_HOST` | PostgreSQL 主机名或 IP | 必填 |
| `DATABASE_PORT` | PostgreSQL 端口 | `5432` |
| `DATABASE_NAME` | 数据库名 | 必填 |
| `DATABASE_USER` | 应用数据库账号 | 必填 |
| `DATABASE_USER_PASSWORD_FILE` | 可读的密码文件路径 | 与下项二选一 |
| `DATABASE_USER_PASSWORD` | 由部署平台注入的密码 | 与上项二选一 |
| `DATABASE_TLS_MODE` | `verify-full` 或 `disable` | 回环地址为 `disable`，其他地址为 `verify-full` |
| `DATABASE_TLS_CA_FILE` | 用于验证服务端的 PEM CA 证书文件 | `verify-full` 时必填 |
| `DATABASE_POOL_MAX_SIZE` | 每个进程的连接池上限 | `16` |
| `DATABASE_POOL_WAIT_TIMEOUT_SECS` | 等待空闲连接的秒数 | `5` |
| `DATABASE_POOL_CREATE_TIMEOUT_SECS` | 创建数据库连接的秒数 | `5` |
| `DATABASE_POOL_RECYCLE_TIMEOUT_SECS` | 检查复用连接的秒数 | `3` |

`verify-full` 要求加密连接并校验服务端证书和主机名。将 `DATABASE_HOST` 设置为服务端证书包含的名称。若使用 IP，证书须包含对应的 IP SAN。部署方提供数据库服务商的 CA，或签发服务端证书的内部 CA。只有回环地址允许使用 `disable`，可用于本地 SSH 隧道和同网络命名空间的数据库。

## 真机部署

通过服务管理器注入数据库地址、账号和池参数。可以把密码放在只有服务账号可读的文件中，并将文件路径传给 `DATABASE_USER_PASSWORD_FILE`。CA 文件也由服务账号读取。使用 systemd 的 `LoadCredential=` 时，可将 `DATABASE_USER_PASSWORD_FILE` 指向相应凭据文件。

## Docker 和 Podman

使用 `-f DockerFile` 构建。运行时将密码和 CA 文件挂载到容器中，建议使用只读挂载或容器平台的 secrets 功能。把 `DATABASE_USER_PASSWORD_FILE` 和 `DATABASE_TLS_CA_FILE` 设置为容器内的路径。数据库地址使用容器可访问且与证书匹配的主机名。容器内的 `127.0.0.1` 指向容器自己的网络命名空间。

连接池上限按进程计算。配置时用每实例上限乘以实例数，并为迁移、管理和其他应用保留数据库连接。三个超时分别作用于等待、创建和回收阶段。
