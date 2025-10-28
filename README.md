# scholl_manager
scholl_manager

放学和上学的管理


## 功能需求
系统结构：
1. server: 后端服务
2. weapp: 小程序前端
3. admin: 管理后台

功能：
1. 管理员在后台配置好学校的所有班级
2. 老师可以在小程序上修改班级的状态：上课中，已放学，放学中（默认已放学）
3. 状态在每天零点为清空为 ：已放学状态
4. 后台有一个展示页面，可以显示全校各班级的状态。
5. 老师在小程序登录时需要申请权限，管理员在后台审核后，老师才能登录。


1.找到一个小程序端的代码@weapp,可以在此代码基础上修改



### 用户管理
1. 登录，注册，退出

## 技术
1. 数据库使用postgres
1. 使用salvo作为web框架
1. 使用sea-orm作为orm
1. 使用sqlx作为数据库操作

## 一些命令

### 数据库迁移
```bash
sqlx migrate run
```

清除
```
sqlx migrate revert 
sqlx migrate revert --target-version 0

```


### 生成entity
```
sea-orm-cli generate entity -u "postgres://test:123456@localhost:5432/test_chat" -o "crates/data_model/src" --with-serde both
```