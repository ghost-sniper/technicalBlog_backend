# RSA 登录解密实现迁移

后端使用 `aws-lc-rs` / AWS-LC 进行 RSA 私钥解密，移除受
[RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html)
影响的 RustCrypto `rsa` 依赖。不能通过升级到 `rsa 0.9.10` 修复该公告。

## 协议兼容

- 数据库中的未加密 PKCS#8 PEM 私钥（`BEGIN PRIVATE KEY`）继续使用。
- RSA-OAEP 使用 SHA-256、MGF1-SHA-256 和空 label。
- RSA 明文仍然是 AES 密钥的 Base64 文本。
- 密码仍使用 AES-256-CBC 和 PKCS#7 填充，IV 为 16 字节。
- 已验证 2048 位和前端默认的 4096 位 RSA 密钥。
- 无效密钥、IV、填充和 UTF-8 返回统一的“解密失败”，避免畸形参数导致 panic。

前端请求字段和数据库结构不需要迁移。此次修改没有发布服务或更换数据库密钥；
需要重新构建并部署后端才能生效。替换实现无法消除过去可能已发生的私钥泄露；
如存在历史泄露迹象，部署后还需轮换相关密钥。

## 验证

```text
cargo test --locked --all-targets
cargo build --locked --release
```

`tests/fixtures/e2ee.json` 中的密钥均为测试专用，不得用于生产。
正常密文由项目实际使用的 `@chatereum/react-e2ee` 加密库生成，
通过 Node.js WebCrypto 运行；测试本身不需要 Node.js、前端依赖或数据库。
需要重新生成测试数据时，先安装相邻前端项目的依赖，再运行：

```text
node tests/fixtures/generate-e2ee.cjs
```

测试覆盖正常中文/Unicode 密码、空明文、错误私钥、OAEP 哈希不匹配、
无效或截断的 RSA 密文、无效 PEM/Base64、错误 AES 密钥和 IV 长度、
无效 CBC 填充以及非 UTF-8 明文。

AWS-LC 包含本地代码，构建环境需要 C/C++ 编译器。
Windows x86-64 已启用官方 `prebuilt-nasm` 功能，在没有 NASM 时使用库自带的汇编对象。
参考 [AWS-LC 构建要求](https://aws.github.io/aws-lc-rs/requirements/index.html)。

本次变更仅处理 RSA 风险；`h2` 的 RUSTSEC-2026-0258 需要另行处理。
