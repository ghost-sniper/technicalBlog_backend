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

## 文章搜索索引

数据库服务器需要安装与 PostgreSQL 版本匹配的扩展包。例如 Rocky Linux 的 PostgreSQL 13 使用对应版本的 `postgresql-contrib`。

逐条以自动提交方式执行 `migrations/20261104_article_search_indexes.sql`。该迁移使用并发创建索引，执行时须位于事务之外。若并发构建失败，先检查 `pg_index.indisvalid`，删除该迁移创建的无效索引后再重试。

搜索使用 GIN trigram 表达式索引，排序使用活动文章的更新时间和 ID 索引。关键词中的百分号、下划线和感叹号按普通文字处理。短关键词可能无法提取有效三元组，查询仍可能扫描大量数据。小表或匹配比例较高时，规划器也可能选择顺序扫描。


## 文章访问权限与评论

部署前，在应用数据库执行 `migrations/20261109_article_comments.sql`（完整事务执行一次）。该迁移新增统一阅读权限函数、评论表及索引，并将新文章的评论状态默认值设为 1；不改变已有文章的访问范围或评论开关。

先迁移数据库，再部署后端，最后部署前端。新后端依赖权限函数，不能跳过迁移。回退应用时保留新增表与函数；旧后端没有阅读权限保护，因此不能在已有受限文章的环境回退到旧后端。

访问范围：`public` 公开、`authenticated` 正常登录账号可见、`private` 仅作者及正常管理员可见；未知旧值同 private。`comment_status=1` 允许正常且有阅读权限的账号评论，其他值关闭新增评论，但保留阅读和删除能力。管理员沿用 `user_type=0`。

评论接口均位于 `/api/article/{id}/comments`：GET 使用 `page`（从 1 开始，每页 10 条一级评论），POST 接受 `content` 和可选 `reply_to_comment_id`。GET 返回 `comments/count/page/can_comment/can_delete/comments_enabled/logged_in`；每条一级评论附带 `replies/reply_count/next_cursor`。`/{root_id}/replies?after={next_cursor}` 每次取 10 条回复，返回 `replies/next_cursor/has_more`。DELETE `/{comment_id}` 删除单条回复或整条一级讨论；重复删除返回成功。

回复按 ID 升序，与同一文章锁内分配的创建顺序一致；一级评论按创建时间、ID 降序。提交与删除先锁账号再锁文章，与编辑保存保持一致，删除一级评论和回复插入不会交错。评论及正文响应禁止缓存；文章图片沿用禁止缓存策略。

验证：`cargo test` 运行不需要数据库的测试。配置独立测试数据库后，运行 `cargo test live_comment_permissions_threads_and_deletion -- --ignored --nocapture`，新评论表不存在时测试会在事务内执行迁移，所有测试数据及迁移最后回滚。生产连接不可用时不要把测试通过视为生产迁移已执行。


完整隔离验证：新建空的临时 PostgreSQL 数据库，依次执行 `tests/fixtures/comment-database.sql`、`migrations/20261102_article_tags.sql`、`migrations/20261103_article_images.sql`。将 `DATABASE_*` 环境变量指向该临时库，设置 `BLOG_ISOLATED_TEST_DATABASE=1`，运行 `cargo test live_comment_ -- --ignored --nocapture --test-threads=1`，再运行 `cargo test live_editor_permissions -- --ignored --nocapture`。第一条命令的接口及并发测试会提交迁移和测试数据，必须使用可丢弃的空数据库，不得指向实际应用数据库；每次完整重跑使用新的临时库。测试完成后停止临时数据库。
