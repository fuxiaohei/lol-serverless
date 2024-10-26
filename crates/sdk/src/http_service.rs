#[allow(dead_code)]
pub mod land {
    #[allow(dead_code)]
    pub mod asyncio {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            /// async io task handle
            pub type Handle = u32;
        }
        #[allow(dead_code, clippy::all)]
        pub mod asyncio {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Handle = super::super::super::land::asyncio::types::Handle;
            #[allow(unused_unsafe, clippy::all)]
            /// new async task handle
            pub fn new() -> Result<Handle, ()> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:asyncio/asyncio")]
                    extern "C" {
                        #[link_name = "new"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => {
                            let e = {
                                let l2 = *ptr0.add(4).cast::<i32>();
                                l2 as u32
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = ();
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// new sleep time task handle and sleep
            pub fn sleep(ms: u32) -> Result<Handle, ()> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:asyncio/asyncio")]
                    extern "C" {
                        #[link_name = "sleep"]
                        fn wit_import(_: i32, _: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32, _: *mut u8) {
                        unreachable!()
                    }
                    wit_import(_rt::as_i32(&ms), ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => {
                            let e = {
                                let l2 = *ptr0.add(4).cast::<i32>();
                                l2 as u32
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = ();
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// select async task, return task handle or need waiting
            pub fn select() -> (Option<Handle>, bool) {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:asyncio/asyncio")]
                    extern "C" {
                        #[link_name = "select"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    let l3 = i32::from(*ptr0.add(8).cast::<u8>());
                    (
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    l2 as u32
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        },
                        _rt::bool_lift(l3 as u8),
                    )
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// ready wating async task ready
            pub fn ready() {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:asyncio/asyncio")]
                    extern "C" {
                        #[link_name = "ready"]
                        fn wit_import();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() {
                        unreachable!()
                    }
                    wit_import();
                }
            }
        }
    }
    #[allow(dead_code)]
    pub mod http {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// HTTP Status Codes
            pub type StatusCode = u16;
            /// HTTP Request Methods, use uppercase
            pub type Method = _rt::String;
            /// HTTP Request Headers
            pub type Headers = _rt::Vec<(_rt::String, _rt::String)>;
            /// HTTP URI
            pub type Uri = _rt::String;
            /// HTTP Request Body
            pub type BodyHandle = u32;
            /// HTTP Request
            #[derive(Clone)]
            pub struct Request {
                pub method: Method,
                pub uri: Uri,
                pub headers: Headers,
                pub body: Option<BodyHandle>,
            }
            impl ::core::fmt::Debug for Request {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Request")
                        .field("method", &self.method)
                        .field("uri", &self.uri)
                        .field("headers", &self.headers)
                        .field("body", &self.body)
                        .finish()
                }
            }
            /// HTTP Response
            #[derive(Clone)]
            pub struct Response {
                pub status: StatusCode,
                pub headers: Headers,
                pub body: Option<BodyHandle>,
            }
            impl ::core::fmt::Debug for Response {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Response")
                        .field("status", &self.status)
                        .field("headers", &self.headers)
                        .field("body", &self.body)
                        .finish()
                }
            }
            /// HTTP errors returned by the runtime.
            #[derive(Clone)]
            pub enum RequestError {
                /// The request failed due to a network error.
                NetworkError(_rt::String),
                /// The request failed due to a timeout.
                Timeout,
                /// The request failed due to a invalid url.
                InvalidUrl,
                /// The request failed due to a forbidden destination.
                DestinationNotAllowed,
                /// The request failed due to over requests limit.
                TooManyRequests,
                /// The request failed due to invalid request
                InvalidRequest(_rt::String),
            }
            impl ::core::fmt::Debug for RequestError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        RequestError::NetworkError(e) => {
                            f.debug_tuple("RequestError::NetworkError").field(e).finish()
                        }
                        RequestError::Timeout => {
                            f.debug_tuple("RequestError::Timeout").finish()
                        }
                        RequestError::InvalidUrl => {
                            f.debug_tuple("RequestError::InvalidUrl").finish()
                        }
                        RequestError::DestinationNotAllowed => {
                            f.debug_tuple("RequestError::DestinationNotAllowed").finish()
                        }
                        RequestError::TooManyRequests => {
                            f.debug_tuple("RequestError::TooManyRequests").finish()
                        }
                        RequestError::InvalidRequest(e) => {
                            f.debug_tuple("RequestError::InvalidRequest")
                                .field(e)
                                .finish()
                        }
                    }
                }
            }
            impl ::core::fmt::Display for RequestError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl std::error::Error for RequestError {}
            /// Fetch redirect policy
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
            pub enum RedirectPolicy {
                /// Follow redirects.
                Follow,
                /// Do not follow redirects. User handles the 3xx response.
                Manual,
                /// Throw an error when a 3xx response is received.
                Error,
            }
            impl ::core::fmt::Debug for RedirectPolicy {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        RedirectPolicy::Follow => {
                            f.debug_tuple("RedirectPolicy::Follow").finish()
                        }
                        RedirectPolicy::Manual => {
                            f.debug_tuple("RedirectPolicy::Manual").finish()
                        }
                        RedirectPolicy::Error => {
                            f.debug_tuple("RedirectPolicy::Error").finish()
                        }
                    }
                }
            }
            impl RedirectPolicy {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> RedirectPolicy {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => RedirectPolicy::Follow,
                        1 => RedirectPolicy::Manual,
                        2 => RedirectPolicy::Error,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
            /// HTTP request option
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct RequestOptions {
                /// The request timeout in milliseconds.
                pub timeout: u32,
                /// Follow redirects.
                pub redirect: RedirectPolicy,
            }
            impl ::core::fmt::Debug for RequestOptions {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("RequestOptions")
                        .field("timeout", &self.timeout)
                        .field("redirect", &self.redirect)
                        .finish()
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod body {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type BodyHandle = super::super::super::land::http::types::BodyHandle;
            /// an error type returned from a body operation.
            #[derive(Clone)]
            pub enum BodyError {
                /// The body is invalid
                InvalidHandle,
                /// The body is only readable
                ReadOnly,
                /// The body is reading closed
                ReadClosed,
                /// The body read failed
                ReadFailed(_rt::String),
                /// The body write failed
                WriteFailed(_rt::String),
                /// The body is writing closed
                WriteClosed,
            }
            impl ::core::fmt::Debug for BodyError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        BodyError::InvalidHandle => {
                            f.debug_tuple("BodyError::InvalidHandle").finish()
                        }
                        BodyError::ReadOnly => {
                            f.debug_tuple("BodyError::ReadOnly").finish()
                        }
                        BodyError::ReadClosed => {
                            f.debug_tuple("BodyError::ReadClosed").finish()
                        }
                        BodyError::ReadFailed(e) => {
                            f.debug_tuple("BodyError::ReadFailed").field(e).finish()
                        }
                        BodyError::WriteFailed(e) => {
                            f.debug_tuple("BodyError::WriteFailed").field(e).finish()
                        }
                        BodyError::WriteClosed => {
                            f.debug_tuple("BodyError::WriteClosed").finish()
                        }
                    }
                }
            }
            impl ::core::fmt::Display for BodyError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl std::error::Error for BodyError {}
            #[allow(unused_unsafe, clippy::all)]
            /// Read http body bytes with size and eof flag
            pub fn read(
                handle: BodyHandle,
                size: u32,
            ) -> Result<(_rt::Vec<u8>, bool), BodyError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:http/body")]
                    extern "C" {
                        #[link_name = "read"]
                        fn wit_import(_: i32, _: i32, _: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32, _: i32, _: *mut u8) {
                        unreachable!()
                    }
                    wit_import(_rt::as_i32(handle), _rt::as_i32(&size), ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => {
                            let e = {
                                let l2 = *ptr0.add(4).cast::<*mut u8>();
                                let l3 = *ptr0.add(8).cast::<usize>();
                                let len4 = l3;
                                let l5 = i32::from(*ptr0.add(12).cast::<u8>());
                                (
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4),
                                    _rt::bool_lift(l5 as u8),
                                )
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l6 = i32::from(*ptr0.add(4).cast::<u8>());
                                let v13 = match l6 {
                                    0 => BodyError::InvalidHandle,
                                    1 => BodyError::ReadOnly,
                                    2 => BodyError::ReadClosed,
                                    3 => {
                                        let e13 = {
                                            let l7 = *ptr0.add(8).cast::<*mut u8>();
                                            let l8 = *ptr0.add(12).cast::<usize>();
                                            let len9 = l8;
                                            let bytes9 = _rt::Vec::from_raw_parts(
                                                l7.cast(),
                                                len9,
                                                len9,
                                            );
                                            _rt::string_lift(bytes9)
                                        };
                                        BodyError::ReadFailed(e13)
                                    }
                                    4 => {
                                        let e13 = {
                                            let l10 = *ptr0.add(8).cast::<*mut u8>();
                                            let l11 = *ptr0.add(12).cast::<usize>();
                                            let len12 = l11;
                                            let bytes12 = _rt::Vec::from_raw_parts(
                                                l10.cast(),
                                                len12,
                                                len12,
                                            );
                                            _rt::string_lift(bytes12)
                                        };
                                        BodyError::WriteFailed(e13)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 5, "invalid enum discriminant");
                                        BodyError::WriteClosed
                                    }
                                };
                                v13
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Readall http body bytes with size and eof flag
            pub fn read_all(handle: BodyHandle) -> Result<_rt::Vec<u8>, BodyError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:http/body")]
                    extern "C" {
                        #[link_name = "read-all"]
                        fn wit_import(_: i32, _: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32, _: *mut u8) {
                        unreachable!()
                    }
                    wit_import(_rt::as_i32(handle), ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => {
                            let e = {
                                let l2 = *ptr0.add(4).cast::<*mut u8>();
                                let l3 = *ptr0.add(8).cast::<usize>();
                                let len4 = l3;
                                _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l5 = i32::from(*ptr0.add(4).cast::<u8>());
                                let v12 = match l5 {
                                    0 => BodyError::InvalidHandle,
                                    1 => BodyError::ReadOnly,
                                    2 => BodyError::ReadClosed,
                                    3 => {
                                        let e12 = {
                                            let l6 = *ptr0.add(8).cast::<*mut u8>();
                                            let l7 = *ptr0.add(12).cast::<usize>();
                                            let len8 = l7;
                                            let bytes8 = _rt::Vec::from_raw_parts(
                                                l6.cast(),
                                                len8,
                                                len8,
                                            );
                                            _rt::string_lift(bytes8)
                                        };
                                        BodyError::ReadFailed(e12)
                                    }
                                    4 => {
                                        let e12 = {
                                            let l9 = *ptr0.add(8).cast::<*mut u8>();
                                            let l10 = *ptr0.add(12).cast::<usize>();
                                            let len11 = l10;
                                            let bytes11 = _rt::Vec::from_raw_parts(
                                                l9.cast(),
                                                len11,
                                                len11,
                                            );
                                            _rt::string_lift(bytes11)
                                        };
                                        BodyError::WriteFailed(e12)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 5, "invalid enum discriminant");
                                        BodyError::WriteClosed
                                    }
                                };
                                v12
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Write http body bytes, return written bytes length
            pub fn write(handle: BodyHandle, data: &[u8]) -> Result<u64, BodyError> {
                unsafe {
                    #[repr(align(8))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 24]);
                    let vec0 = data;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:http/body")]
                    extern "C" {
                        #[link_name = "write"]
                        fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                        unreachable!()
                    }
                    wit_import(_rt::as_i32(handle), ptr0.cast_mut(), len0, ptr1);
                    let l2 = i32::from(*ptr1.add(0).cast::<u8>());
                    match l2 {
                        0 => {
                            let e = {
                                let l3 = *ptr1.add(8).cast::<i64>();
                                l3 as u64
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l4 = i32::from(*ptr1.add(8).cast::<u8>());
                                let v11 = match l4 {
                                    0 => BodyError::InvalidHandle,
                                    1 => BodyError::ReadOnly,
                                    2 => BodyError::ReadClosed,
                                    3 => {
                                        let e11 = {
                                            let l5 = *ptr1.add(12).cast::<*mut u8>();
                                            let l6 = *ptr1.add(16).cast::<usize>();
                                            let len7 = l6;
                                            let bytes7 = _rt::Vec::from_raw_parts(
                                                l5.cast(),
                                                len7,
                                                len7,
                                            );
                                            _rt::string_lift(bytes7)
                                        };
                                        BodyError::ReadFailed(e11)
                                    }
                                    4 => {
                                        let e11 = {
                                            let l8 = *ptr1.add(12).cast::<*mut u8>();
                                            let l9 = *ptr1.add(16).cast::<usize>();
                                            let len10 = l9;
                                            let bytes10 = _rt::Vec::from_raw_parts(
                                                l8.cast(),
                                                len10,
                                                len10,
                                            );
                                            _rt::string_lift(bytes10)
                                        };
                                        BodyError::WriteFailed(e11)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 5, "invalid enum discriminant");
                                        BodyError::WriteClosed
                                    }
                                };
                                v11
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Accquire http body and get http body handle
            pub fn new() -> Result<BodyHandle, BodyError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:http/body")]
                    extern "C" {
                        #[link_name = "new"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => {
                            let e = {
                                let l2 = *ptr0.add(4).cast::<i32>();
                                l2 as u32
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l3 = i32::from(*ptr0.add(4).cast::<u8>());
                                let v10 = match l3 {
                                    0 => BodyError::InvalidHandle,
                                    1 => BodyError::ReadOnly,
                                    2 => BodyError::ReadClosed,
                                    3 => {
                                        let e10 = {
                                            let l4 = *ptr0.add(8).cast::<*mut u8>();
                                            let l5 = *ptr0.add(12).cast::<usize>();
                                            let len6 = l5;
                                            let bytes6 = _rt::Vec::from_raw_parts(
                                                l4.cast(),
                                                len6,
                                                len6,
                                            );
                                            _rt::string_lift(bytes6)
                                        };
                                        BodyError::ReadFailed(e10)
                                    }
                                    4 => {
                                        let e10 = {
                                            let l7 = *ptr0.add(8).cast::<*mut u8>();
                                            let l8 = *ptr0.add(12).cast::<usize>();
                                            let len9 = l8;
                                            let bytes9 = _rt::Vec::from_raw_parts(
                                                l7.cast(),
                                                len9,
                                                len9,
                                            );
                                            _rt::string_lift(bytes9)
                                        };
                                        BodyError::WriteFailed(e10)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 5, "invalid enum discriminant");
                                        BodyError::WriteClosed
                                    }
                                };
                                v10
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Accquire http body and get http body handle
            pub fn new_stream() -> Result<BodyHandle, BodyError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:http/body")]
                    extern "C" {
                        #[link_name = "new-stream"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => {
                            let e = {
                                let l2 = *ptr0.add(4).cast::<i32>();
                                l2 as u32
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l3 = i32::from(*ptr0.add(4).cast::<u8>());
                                let v10 = match l3 {
                                    0 => BodyError::InvalidHandle,
                                    1 => BodyError::ReadOnly,
                                    2 => BodyError::ReadClosed,
                                    3 => {
                                        let e10 = {
                                            let l4 = *ptr0.add(8).cast::<*mut u8>();
                                            let l5 = *ptr0.add(12).cast::<usize>();
                                            let len6 = l5;
                                            let bytes6 = _rt::Vec::from_raw_parts(
                                                l4.cast(),
                                                len6,
                                                len6,
                                            );
                                            _rt::string_lift(bytes6)
                                        };
                                        BodyError::ReadFailed(e10)
                                    }
                                    4 => {
                                        let e10 = {
                                            let l7 = *ptr0.add(8).cast::<*mut u8>();
                                            let l8 = *ptr0.add(12).cast::<usize>();
                                            let len9 = l8;
                                            let bytes9 = _rt::Vec::from_raw_parts(
                                                l7.cast(),
                                                len9,
                                                len9,
                                            );
                                            _rt::string_lift(bytes9)
                                        };
                                        BodyError::WriteFailed(e10)
                                    }
                                    n => {
                                        debug_assert_eq!(n, 5, "invalid enum discriminant");
                                        BodyError::WriteClosed
                                    }
                                };
                                v10
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
        }
        #[allow(dead_code, clippy::all)]
        pub mod fetching {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Request = super::super::super::land::http::types::Request;
            pub type Response = super::super::super::land::http::types::Response;
            pub type RequestError = super::super::super::land::http::types::RequestError;
            pub type RequestOptions = super::super::super::land::http::types::RequestOptions;
            #[allow(unused_unsafe, clippy::all)]
            /// send request function
            pub fn send_request(
                req: &Request,
                options: RequestOptions,
            ) -> Result<Response, RequestError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 24]);
                    let super::super::super::land::http::types::Request {
                        method: method0,
                        uri: uri0,
                        headers: headers0,
                        body: body0,
                    } = req;
                    let vec1 = method0;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let vec2 = uri0;
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    let vec6 = headers0;
                    let len6 = vec6.len();
                    let layout6 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec6.len() * 16,
                        4,
                    );
                    let result6 = if layout6.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout6).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout6);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec6.into_iter().enumerate() {
                        let base = result6.add(i * 16);
                        {
                            let (t3_0, t3_1) = e;
                            let vec4 = t3_0;
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            *base.add(4).cast::<usize>() = len4;
                            *base.add(0).cast::<*mut u8>() = ptr4.cast_mut();
                            let vec5 = t3_1;
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            *base.add(12).cast::<usize>() = len5;
                            *base.add(8).cast::<*mut u8>() = ptr5.cast_mut();
                        }
                    }
                    let (result7_0, result7_1) = match body0 {
                        Some(e) => (1i32, _rt::as_i32(e)),
                        None => (0i32, 0i32),
                    };
                    let super::super::super::land::http::types::RequestOptions {
                        timeout: timeout8,
                        redirect: redirect8,
                    } = options;
                    let ptr9 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:http/fetching")]
                    extern "C" {
                        #[link_name = "send-request"]
                        fn wit_import(
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: i32,
                            _: i32,
                            _: i32,
                            _: i32,
                            _: *mut u8,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: i32,
                        _: i32,
                        _: i32,
                        _: i32,
                        _: *mut u8,
                    ) {
                        unreachable!()
                    }
                    wit_import(
                        ptr1.cast_mut(),
                        len1,
                        ptr2.cast_mut(),
                        len2,
                        result6,
                        len6,
                        result7_0,
                        result7_1,
                        _rt::as_i32(timeout8),
                        redirect8.clone() as i32,
                        ptr9,
                    );
                    let l10 = i32::from(*ptr9.add(0).cast::<u8>());
                    if layout6.size() != 0 {
                        _rt::alloc::dealloc(result6.cast(), layout6);
                    }
                    match l10 {
                        0 => {
                            let e = {
                                let l11 = i32::from(*ptr9.add(4).cast::<u16>());
                                let l12 = *ptr9.add(8).cast::<*mut u8>();
                                let l13 = *ptr9.add(12).cast::<usize>();
                                let base20 = l12;
                                let len20 = l13;
                                let mut result20 = _rt::Vec::with_capacity(len20);
                                for i in 0..len20 {
                                    let base = base20.add(i * 16);
                                    let e20 = {
                                        let l14 = *base.add(0).cast::<*mut u8>();
                                        let l15 = *base.add(4).cast::<usize>();
                                        let len16 = l15;
                                        let bytes16 = _rt::Vec::from_raw_parts(
                                            l14.cast(),
                                            len16,
                                            len16,
                                        );
                                        let l17 = *base.add(8).cast::<*mut u8>();
                                        let l18 = *base.add(12).cast::<usize>();
                                        let len19 = l18;
                                        let bytes19 = _rt::Vec::from_raw_parts(
                                            l17.cast(),
                                            len19,
                                            len19,
                                        );
                                        (_rt::string_lift(bytes16), _rt::string_lift(bytes19))
                                    };
                                    result20.push(e20);
                                }
                                _rt::cabi_dealloc(base20, len20 * 16, 4);
                                let l21 = i32::from(*ptr9.add(16).cast::<u8>());
                                super::super::super::land::http::types::Response {
                                    status: l11 as u16,
                                    headers: result20,
                                    body: match l21 {
                                        0 => None,
                                        1 => {
                                            let e = {
                                                let l22 = *ptr9.add(20).cast::<i32>();
                                                l22 as u32
                                            };
                                            Some(e)
                                        }
                                        _ => _rt::invalid_enum_discriminant(),
                                    },
                                }
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l23 = i32::from(*ptr9.add(4).cast::<u8>());
                                use super::super::super::land::http::types::RequestError as V30;
                                let v30 = match l23 {
                                    0 => {
                                        let e30 = {
                                            let l24 = *ptr9.add(8).cast::<*mut u8>();
                                            let l25 = *ptr9.add(12).cast::<usize>();
                                            let len26 = l25;
                                            let bytes26 = _rt::Vec::from_raw_parts(
                                                l24.cast(),
                                                len26,
                                                len26,
                                            );
                                            _rt::string_lift(bytes26)
                                        };
                                        V30::NetworkError(e30)
                                    }
                                    1 => V30::Timeout,
                                    2 => V30::InvalidUrl,
                                    3 => V30::DestinationNotAllowed,
                                    4 => V30::TooManyRequests,
                                    n => {
                                        debug_assert_eq!(n, 5, "invalid enum discriminant");
                                        let e30 = {
                                            let l27 = *ptr9.add(8).cast::<*mut u8>();
                                            let l28 = *ptr9.add(12).cast::<usize>();
                                            let len29 = l28;
                                            let bytes29 = _rt::Vec::from_raw_parts(
                                                l27.cast(),
                                                len29,
                                                len29,
                                            );
                                            _rt::string_lift(bytes29)
                                        };
                                        V30::InvalidRequest(e30)
                                    }
                                };
                                v30
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
        }
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    pub fn as_i32<T: AsI32>(t: T) -> i32 {
        t.as_i32()
    }
    pub trait AsI32 {
        fn as_i32(self) -> i32;
    }
    impl<'a, T: Copy + AsI32> AsI32 for &'a T {
        fn as_i32(self) -> i32 {
            (*self).as_i32()
        }
    }
    impl AsI32 for i32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for char {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for usize {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    pub unsafe fn bool_lift(val: u8) -> bool {
        if cfg!(debug_assertions) {
            match val {
                0 => false,
                1 => true,
                _ => panic!("invalid bool discriminant"),
            }
        } else {
            val != 0
        }
    }
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
    pub use alloc_crate::alloc;
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    extern crate alloc as alloc_crate;
}
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.34.0:land:worker:http-service-with-all-of-its-exports-removed:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 1340] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x99\x09\x01A\x02\x01\
A\x10\x01B\x16\x01{\x04\0\x0bstatus-code\x03\0\0\x01s\x04\0\x06method\x03\0\x02\x01\
o\x02ss\x01p\x04\x04\0\x07headers\x03\0\x05\x01s\x04\0\x03uri\x03\0\x07\x01y\x04\
\0\x0bbody-handle\x03\0\x09\x01k\x0a\x01r\x04\x06method\x03\x03uri\x08\x07header\
s\x06\x04body\x0b\x04\0\x07request\x03\0\x0c\x01r\x03\x06status\x01\x07headers\x06\
\x04body\x0b\x04\0\x08response\x03\0\x0e\x01q\x06\x0dnetwork-error\x01s\0\x07tim\
eout\0\0\x0binvalid-url\0\0\x17destination-not-allowed\0\0\x11too-many-requests\0\
\0\x0finvalid-request\x01s\0\x04\0\x0drequest-error\x03\0\x10\x01m\x03\x06follow\
\x06manual\x05error\x04\0\x0fredirect-policy\x03\0\x12\x01r\x02\x07timeouty\x08r\
edirect\x13\x04\0\x0frequest-options\x03\0\x14\x03\0\x0fland:http/types\x05\0\x02\
\x03\0\0\x0bbody-handle\x01B\x13\x02\x03\x02\x01\x01\x04\0\x0bbody-handle\x03\0\0\
\x01q\x06\x0einvalid-handle\0\0\x09read-only\0\0\x0bread-closed\0\0\x0bread-fail\
ed\x01s\0\x0cwrite-failed\x01s\0\x0cwrite-closed\0\0\x04\0\x0abody-error\x03\0\x02\
\x01p}\x01o\x02\x04\x7f\x01j\x01\x05\x01\x03\x01@\x02\x06handle\x01\x04sizey\0\x06\
\x04\0\x04read\x01\x07\x01j\x01\x04\x01\x03\x01@\x01\x06handle\x01\0\x08\x04\0\x08\
read-all\x01\x09\x01j\x01w\x01\x03\x01@\x02\x06handle\x01\x04data\x04\0\x0a\x04\0\
\x05write\x01\x0b\x01j\x01\x01\x01\x03\x01@\0\0\x0c\x04\0\x03new\x01\x0d\x04\0\x0a\
new-stream\x01\x0d\x03\0\x0eland:http/body\x05\x02\x02\x03\0\0\x07request\x02\x03\
\0\0\x08response\x02\x03\0\0\x0drequest-error\x02\x03\0\0\x0frequest-options\x01\
B\x0b\x02\x03\x02\x01\x03\x04\0\x07request\x03\0\0\x02\x03\x02\x01\x04\x04\0\x08\
response\x03\0\x02\x02\x03\x02\x01\x05\x04\0\x0drequest-error\x03\0\x04\x02\x03\x02\
\x01\x06\x04\0\x0frequest-options\x03\0\x06\x01j\x01\x03\x01\x05\x01@\x02\x03req\
\x01\x07options\x07\0\x08\x04\0\x0csend-request\x01\x09\x03\0\x12land:http/fetch\
ing\x05\x07\x01B\x02\x01y\x04\0\x06handle\x03\0\0\x03\0\x12land:asyncio/types\x05\
\x08\x02\x03\0\x03\x06handle\x01B\x0d\x02\x03\x02\x01\x09\x04\0\x06handle\x03\0\0\
\x01j\x01\x01\0\x01@\0\0\x02\x04\0\x03new\x01\x03\x01@\x01\x02msy\0\x02\x04\0\x05\
sleep\x01\x04\x01k\x01\x01o\x02\x05\x7f\x01@\0\0\x06\x04\0\x06select\x01\x07\x01\
@\0\x01\0\x04\0\x05ready\x01\x08\x03\0\x14land:asyncio/asyncio\x05\x0a\x04\08lan\
d:worker/http-service-with-all-of-its-exports-removed\x04\0\x0b2\x01\0,http-serv\
ice-with-all-of-its-exports-removed\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\
\x0dwit-component\x070.219.1\x10wit-bindgen-rust\x060.34.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen::rt::maybe_link_cabi_realloc();
}
