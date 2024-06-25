# TODO

## support `skip_auth: false`
```Rust
let methods = stream.send_version_and_methods(methods).await?;
stream.which_method_accepted(methods).await?;
```