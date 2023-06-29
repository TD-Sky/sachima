[TOC]

<h1 align="center">Sachima</h1>

<p align="center">
  简单的文件服务器
</p>

---

<p align="center">
  <a href="../README.md">English</a>
</p>

**Sachima**（沙琪玛）是一个基于 [poem](https://crates.io/crates/poem) 开发的文件服务器。

**Sachima** 旨在以简洁的方式提供**文件分享**功能，并且**管理员**同样能通过HTTP管理**工作空间**的文件。



## 配置

Sachima 使用 TOML 进行启动配置。

| 属性 | 类型 | 说明 |
|:-|:-|:-|
| **port**           | `u16` | 服务器部署的端口 |
| **workspace**      | `String` | 文件分享的工作空间路径 |
| **poem-log-level** | `Option<String>` | poem框架的日志等级 |
| **max-upload**     | `String` | 最大上传限制，单位为数据量（B,KB,G.） |
| **database-url**   | `String` | PostgreSQL数据库连接的URL |
| **jwt-secret-key** | `String` | JWT签名的密钥 |
| **password-salt**  | `String` | 管理员密码的哈希盐 |

> 数据库用于存放管理员账号信息
>
> 原本不打算引入数据库的，但出于某些原因还是用了，**后面的版本将替换成其它更简单的方式**



## 部署

1. 首先编译出程序

```bash
$ cargo build -r
```

2. 配合 [sea-orm-cli](https://crates.io/crates/sea-orm-cli) 快速创建数据库及表

```bash
# 在项目目录下
$ sea migrate up -u <DATABASE_URL>
```

3. 指定配置文件启动Sachima

```bash
$ sachima -c <CONFIG>
```

4. 完成！你可以向Sachima发送HTTP请求了。



## 文件操作接口

| 路径 | 方法 | 权限 | 功能 |
|:-|:-:|:-:|:-|
| `/wk/r/file/*path`   | **GET** | 所有人 | 下载文件 |
| `/wk/r/dir/*path`    | **GET** | 所有人 | 列举目录的项 |
| `/wk/w/upload/*path` | **POST** | 管理员 | 上传文件，请求MIME类型为[multipart](https://en.wikipedia.org/wiki/MIME#Multipart_messages) |
| `/wk/w/rename/*path?name={name}` | **PUT** | 管理员 | 重命名文件/目录 |
| `/wk/w/remove/*path` | **DELETE** | 管理员 | 移除文件/目录 |
| `/wk/w/mkdir/*parent`  | **POST** | 管理员 | 于指定父目录下新建目录 |



## 开发

```
src
├── config            # 配置解析
├── entity            # 数据库实体
├── handlers          # 请求处理服务
│  ├── file_system    ## 文件系统接口
│  │  ├── mod.rs
│  │  └── tests.rs    ### 文件系统接口单元测试
│  └── permission.rs  ## 权限服务接口
├── middlewares       # 中间件
│  ├── jwt.rs         ## JWT验证
├── models            # 服务所用的结构体
├── utils             # 工具
│  ├── pswd.rs        ## 密码
│  ├── tests.rs       ## 测试
│  └── time.rs        ## 时间
├── db.rs             # 数据库连接handler
├── error.rs          # 错误类型
├── lib.rs
├── main.rs
├── reply.rs          # 响应的封装
└── router.rs         # 请求路由器
```

所有的文件系统接口皆已被单元测试覆盖。



## TODO

- [ ] 用户管理采用更简单的方式
- [ ] 实现一个可用的前端
- [ ] 绘制示意图介绍架构
