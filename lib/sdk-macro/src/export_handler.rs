#[allow(dead_code)]
pub mod land {
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
            #[derive(Clone, Copy, Eq, PartialEq)]
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
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod land {
        #[allow(dead_code)]
        pub mod asyncio {
            #[allow(dead_code, clippy::all)]
            pub mod context {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_is_pending_cabi<T: Guest>() -> i32 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::is_pending();
                    match result0 {
                        true => 1,
                        false => 0,
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_select_cabi<T: Guest>() -> i32 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::select();
                    match result0 {
                        true => 1,
                        false => 0,
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_cancel_cabi<T: Guest>() {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    T::cancel();
                }
                pub trait Guest {
                    /// is ctx pending
                    fn is_pending() -> bool;
                    /// select one task to run, if no task is ready, return false
                    fn select() -> bool;
                    /// cancel all ctx running
                    fn cancel();
                }
                #[doc(hidden)]
                #[macro_export]
                macro_rules! __export_land_asyncio_context_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "land:asyncio/context#is-pending"] unsafe extern "C" fn
                        export_is_pending() -> i32 { $($path_to_types)*::
                        _export_is_pending_cabi::<$ty > () } #[export_name =
                        "land:asyncio/context#select"] unsafe extern "C" fn
                        export_select() -> i32 { $($path_to_types)*::
                        _export_select_cabi::<$ty > () } #[export_name =
                        "land:asyncio/context#cancel"] unsafe extern "C" fn
                        export_cancel() { $($path_to_types)*:: _export_cancel_cabi::<$ty
                        > () } };
                    };
                }
                #[doc(hidden)]
                pub use __export_land_asyncio_context_cabi;
            }
        }
        #[allow(dead_code)]
        pub mod http {
            #[allow(dead_code, clippy::all)]
            pub mod incoming {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                pub type Request = super::super::super::super::land::http::types::Request;
                pub type Response = super::super::super::super::land::http::types::Response;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_handle_request_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                    arg4: *mut u8,
                    arg5: usize,
                    arg6: i32,
                    arg7: i32,
                ) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    let len1 = arg3;
                    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
                    let base8 = arg4;
                    let len8 = arg5;
                    let mut result8 = _rt::Vec::with_capacity(len8);
                    for i in 0..len8 {
                        let base = base8.add(i * 16);
                        let e8 = {
                            let l2 = *base.add(0).cast::<*mut u8>();
                            let l3 = *base.add(4).cast::<usize>();
                            let len4 = l3;
                            let bytes4 = _rt::Vec::from_raw_parts(l2.cast(), len4, len4);
                            let l5 = *base.add(8).cast::<*mut u8>();
                            let l6 = *base.add(12).cast::<usize>();
                            let len7 = l6;
                            let bytes7 = _rt::Vec::from_raw_parts(l5.cast(), len7, len7);
                            (_rt::string_lift(bytes4), _rt::string_lift(bytes7))
                        };
                        result8.push(e8);
                    }
                    _rt::cabi_dealloc(base8, len8 * 16, 4);
                    let result9 = T::handle_request(super::super::super::super::land::http::types::Request {
                        method: _rt::string_lift(bytes0),
                        uri: _rt::string_lift(bytes1),
                        headers: result8,
                        body: match arg6 {
                            0 => None,
                            1 => {
                                let e = arg7 as u32;
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        },
                    });
                    let ptr10 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let super::super::super::super::land::http::types::Response {
                        status: status11,
                        headers: headers11,
                        body: body11,
                    } = result9;
                    *ptr10.add(0).cast::<u16>() = (_rt::as_i32(status11)) as u16;
                    let vec15 = headers11;
                    let len15 = vec15.len();
                    let layout15 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec15.len() * 16,
                        4,
                    );
                    let result15 = if layout15.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout15).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout15);
                        }
                        ptr
                    } else {
                        { ::core::ptr::null_mut() }
                    };
                    for (i, e) in vec15.into_iter().enumerate() {
                        let base = result15.add(i * 16);
                        {
                            let (t12_0, t12_1) = e;
                            let vec13 = (t12_0.into_bytes()).into_boxed_slice();
                            let ptr13 = vec13.as_ptr().cast::<u8>();
                            let len13 = vec13.len();
                            ::core::mem::forget(vec13);
                            *base.add(4).cast::<usize>() = len13;
                            *base.add(0).cast::<*mut u8>() = ptr13.cast_mut();
                            let vec14 = (t12_1.into_bytes()).into_boxed_slice();
                            let ptr14 = vec14.as_ptr().cast::<u8>();
                            let len14 = vec14.len();
                            ::core::mem::forget(vec14);
                            *base.add(12).cast::<usize>() = len14;
                            *base.add(8).cast::<*mut u8>() = ptr14.cast_mut();
                        }
                    }
                    *ptr10.add(8).cast::<usize>() = len15;
                    *ptr10.add(4).cast::<*mut u8>() = result15;
                    match body11 {
                        Some(e) => {
                            *ptr10.add(12).cast::<u8>() = (1i32) as u8;
                            *ptr10.add(16).cast::<i32>() = _rt::as_i32(e);
                        }
                        None => {
                            *ptr10.add(12).cast::<u8>() = (0i32) as u8;
                        }
                    };
                    ptr10
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_handle_request<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(4).cast::<*mut u8>();
                    let l1 = *arg0.add(8).cast::<usize>();
                    let base6 = l0;
                    let len6 = l1;
                    for i in 0..len6 {
                        let base = base6.add(i * 16);
                        {
                            let l2 = *base.add(0).cast::<*mut u8>();
                            let l3 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l2, l3, 1);
                            let l4 = *base.add(8).cast::<*mut u8>();
                            let l5 = *base.add(12).cast::<usize>();
                            _rt::cabi_dealloc(l4, l5, 1);
                        }
                    }
                    _rt::cabi_dealloc(base6, len6 * 16, 4);
                }
                pub trait Guest {
                    /// handle request function
                    fn handle_request(req: Request) -> Response;
                }
                #[doc(hidden)]
                #[macro_export]
                macro_rules! __export_land_http_incoming_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name =
                        "land:http/incoming#handle-request"] unsafe extern "C" fn
                        export_handle_request(arg0 : * mut u8, arg1 : usize, arg2 : * mut
                        u8, arg3 : usize, arg4 : * mut u8, arg5 : usize, arg6 : i32, arg7
                        : i32,) -> * mut u8 { $($path_to_types)*::
                        _export_handle_request_cabi::<$ty > (arg0, arg1, arg2, arg3,
                        arg4, arg5, arg6, arg7) } #[export_name =
                        "cabi_post_land:http/incoming#handle-request"] unsafe extern "C"
                        fn _post_return_handle_request(arg0 : * mut u8,) {
                        $($path_to_types)*:: __post_return_handle_request::<$ty > (arg0)
                        } };
                    };
                }
                #[doc(hidden)]
                pub use __export_land_http_incoming_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 20]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 20],
                );
            }
        }
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen::rt::run_ctors_once();
    }
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
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
    pub use alloc_crate::alloc;
    extern crate alloc as alloc_crate;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
#[macro_export]
macro_rules! __export_export_handler_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::land::http::incoming::__export_land_http_incoming_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::land::http::incoming);
        $($path_to_types_root)*::
        exports::land::asyncio::context::__export_land_asyncio_context_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::land::asyncio::context); const _
        : () = { #[cfg(target_arch = "wasm32")] #[link_section =
        "component-type:wit-bindgen:0.30.0:export-handler:imports and exports"]
        #[doc(hidden)] pub static __WIT_BINDGEN_COMPONENT_TYPE : [u8; 771] = *
        b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xfe\x04\x01A\x02\x01\
A\x08\x01B\x16\x01{\x04\0\x0bstatus-code\x03\0\0\x01s\x04\0\x06method\x03\0\x02\x01\
o\x02ss\x01p\x04\x04\0\x07headers\x03\0\x05\x01s\x04\0\x03uri\x03\0\x07\x01y\x04\
\0\x0bbody-handle\x03\0\x09\x01k\x0a\x01r\x04\x06method\x03\x03uri\x08\x07header\
s\x06\x04body\x0b\x04\0\x07request\x03\0\x0c\x01r\x03\x06status\x01\x07headers\x06\
\x04body\x0b\x04\0\x08response\x03\0\x0e\x01q\x06\x0dnetwork-error\x01s\0\x07tim\
eout\0\0\x0binvalid-url\0\0\x17destination-not-allowed\0\0\x11too-many-requests\0\
\0\x0finvalid-request\x01s\0\x04\0\x0drequest-error\x03\0\x10\x01m\x03\x06follow\
\x06manual\x05error\x04\0\x0fredirect-policy\x03\0\x12\x01r\x02\x07timeouty\x08r\
edirect\x13\x04\0\x0frequest-options\x03\0\x14\x03\x01\x0fland:http/types\x05\0\x02\
\x03\0\0\x07request\x02\x03\0\0\x08response\x01B\x06\x02\x03\x02\x01\x01\x04\0\x07\
request\x03\0\0\x02\x03\x02\x01\x02\x04\0\x08response\x03\0\x02\x01@\x01\x03req\x01\
\0\x03\x04\0\x0ehandle-request\x01\x04\x04\x01\x12land:http/incoming\x05\x03\x01\
B\x05\x01@\0\0\x7f\x04\0\x0ais-pending\x01\0\x04\0\x06select\x01\0\x01@\0\x01\0\x04\
\0\x06cancel\x01\x01\x04\x01\x14land:asyncio/context\x05\x04\x04\x01\x1aland:wor\
ker/export-handler\x04\0\x0b\x14\x01\0\x0eexport-handler\x03\0\0\0G\x09producers\
\x01\x0cprocessed-by\x02\x0dwit-component\x070.215.0\x10wit-bindgen-rust\x060.30\
.0";
        };
    };
}
#[doc(inline)]
pub use __export_export_handler_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:export-handler-with-all-of-its-exports-removed:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 643] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xde\x03\x01A\x02\x01\
A\x02\x01B\x16\x01{\x04\0\x0bstatus-code\x03\0\0\x01s\x04\0\x06method\x03\0\x02\x01\
o\x02ss\x01p\x04\x04\0\x07headers\x03\0\x05\x01s\x04\0\x03uri\x03\0\x07\x01y\x04\
\0\x0bbody-handle\x03\0\x09\x01k\x0a\x01r\x04\x06method\x03\x03uri\x08\x07header\
s\x06\x04body\x0b\x04\0\x07request\x03\0\x0c\x01r\x03\x06status\x01\x07headers\x06\
\x04body\x0b\x04\0\x08response\x03\0\x0e\x01q\x06\x0dnetwork-error\x01s\0\x07tim\
eout\0\0\x0binvalid-url\0\0\x17destination-not-allowed\0\0\x11too-many-requests\0\
\0\x0finvalid-request\x01s\0\x04\0\x0drequest-error\x03\0\x10\x01m\x03\x06follow\
\x06manual\x05error\x04\0\x0fredirect-policy\x03\0\x12\x01r\x02\x07timeouty\x08r\
edirect\x13\x04\0\x0frequest-options\x03\0\x14\x03\x01\x0fland:http/types\x05\0\x04\
\x01:land:worker/export-handler-with-all-of-its-exports-removed\x04\0\x0b4\x01\0\
.export-handler-with-all-of-its-exports-removed\x03\0\0\0G\x09producers\x01\x0cp\
rocessed-by\x02\x0dwit-component\x070.215.0\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen::rt::maybe_link_cabi_realloc();
}
