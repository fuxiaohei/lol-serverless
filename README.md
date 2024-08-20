<p align="center"><img src="docs/logo-v2.png" width="120" height="120"/></p>
<h1 align="center">Runtime.land</h1>

`Runtime.land` is a function-as-a-service platform that allows you to run your code in the cloud. It runs in sandboxed environments based on WebAssembly, which means that you can run code in any language that compiles to WebAssembly. It is designed to be fast, secure, and easy to use.

## Current Status (Alpha)

`Runtime.land` is in **alpha** stage and **NOT READY** for production use. We are working on the platform and adding new features. 

If you want to try it out, you can sign up from developer dashboard [https://dash.runtime.land](https://dash.runtime.land).

## Features

- **Fast**: use WebAssembly to approach native performance.
- **Secure**: runs your code in a sandboxed environment.
- **Cloud**: runs your code in the cloud.

## Language Support

`Runtime.land` is working on adding support for multiple languages. Currently, we support the following languages:

| Language | SDK | Status | Features |
| -------- | ------ | ------ | ------ |
| Rust     | [land-sdk](https://crates.io/crates/land-sdk) | ✅ | HTTP Trigger, HTTP Router, Fetch HTTP request |
| JavaScript | [runtime-land-js](https://github.com/fuxiaohei/runtime-land-js) | ✅ | Fetch API with Request/Response <br/> Base64 Encoding <br/> TextEncoding <br/> Web Streams |
| Golang | planning | ❌ | |
| Python | planning | ❌ | |

## License

`Runtime.land` is licensed under the Apache 2.0 License. See [LICENSE](LICENSE) for the full license text.

## Acknowledgments

- Based on [Wasmtime](https://wasmtime.dev/) to run WebAssembly. 
- Use [zeabur.com](https://zeabur.com/) to deploy the dashbord.
- Use [Cloudflare](https://cloudflare.com/) to make custom domain avaiable and routing edge machines.