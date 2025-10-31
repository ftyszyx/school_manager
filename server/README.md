# school_manager_server


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

### 运行测试
```
// test all
cargo test 
//test one module
cargo test --test auth_test 
```