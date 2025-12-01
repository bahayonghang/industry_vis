# 配置

Industry Vis 使用 TOML 格式的配置文件。

## 配置文件位置

配置文件 `config.toml` 位于应用程序可执行文件所在目录。开发时位于项目根目录。

## 配置项说明

### 数据库配置 `[database]`

```toml
[database]
# SQL Server 服务器地址
server = "localhost"

# SQL Server 端口（默认 1433）
port = 1433

# 数据库名称
database = "控制器数据库"

# 用户名（SQL Server 认证）
username = "sa"

# 密码
password = "your_password"
```

### 查询配置 `[query]`

```toml
[query]
# 默认查询表名
default_table = "历史表"
```

## 默认值

如果配置文件不存在，系统将使用以下默认值：

| 配置项 | 默认值 |
|--------|--------|
| server | localhost |
| port | 1433 |
| database | 控制器数据库 |
| username | sa |
| password | (空) |
| default_table | 历史表 |

## 在应用中配置

你也可以在应用的"设置"页面中配置数据库连接：

1. 点击顶部导航栏的"设置"
2. 填写数据库连接信息
3. 点击"测试连接"验证配置
4. 点击"保存配置"

## 安全建议

- 不要将包含密码的 `config.toml` 提交到版本控制
- 在生产环境中使用强密码
- 限制数据库用户的权限，只授予必要的读取权限
