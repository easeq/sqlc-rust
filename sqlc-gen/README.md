# sqlc-gen-rust

Check [sqlc docs](https://docs.sqlc.dev/en/latest/guides/plugins.html) for more info.

### sqlc.yaml

```yaml 
version: '2'
plugins:
- name: rust-gen 
  wasm:
    url: file://sqlc-gen-rust/target/wasm32-wasi/release/sqlc-gen-rust.wasm 
    sha256: 614c9de4c8f5e35e2da94bff54ed71d041a3a4a4f353abef9b01eca09693ee91

sql:
- schema: postgresql/schema.sql
  queries: postgresql/query.sql
  engine: postgresql
  codegen:
  - out: gen
    plugin: rust-gen 
    options:
      lang: en-us
```

### Run

The following command generates rust code for all examples

```sh 
make generate
```
