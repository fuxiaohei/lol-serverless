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
            pub type AsyncHandle = u32;
        }
        #[allow(dead_code, clippy::all)]
        pub mod asyncio {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type AsyncHandle = super::super::super::land::asyncio::types::AsyncHandle;
            #[allow(unused_unsafe, clippy::all)]
            /// sleep for ms milliseconds, create a sleep task and return the handle
            pub fn sleep(ms: u32) -> Result<AsyncHandle, ()> {
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
            /// cancel a task, no-op if the task is already done or not found
            pub fn cancel(handle: AsyncHandle) {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:asyncio/asyncio")]
                    extern "C" {
                        #[link_name = "cancel"]
                        fn wit_import(_: i32);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32) {
                        unreachable!()
                    }
                    wit_import(_rt::as_i32(handle));
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// return true if there is a pending job
            pub fn is_job_pending() -> bool {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:asyncio/asyncio")]
                    extern "C" {
                        #[link_name = "is-job-pending"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    _rt::bool_lift(ret as u8)
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// execute a job, return true if there is a pending job
            pub fn execute_job() -> bool {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "land:asyncio/asyncio")]
                    extern "C" {
                        #[link_name = "execute-job"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    _rt::bool_lift(ret as u8)
                }
            }
        }
    }
}
mod _rt {
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
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
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
}
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:asyncio-service-with-all-of-its-exports-removed:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 438] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x90\x02\x01A\x02\x01\
A\x05\x01B\x02\x01y\x04\0\x0casync-handle\x03\0\0\x03\x01\x12land:asyncio/types\x05\
\0\x02\x03\0\0\x0casync-handle\x01B\x0a\x02\x03\x02\x01\x01\x04\0\x0casync-handl\
e\x03\0\0\x01j\x01\x01\0\x01@\x01\x02msy\0\x02\x04\0\x05sleep\x01\x03\x01@\x01\x06\
handle\x01\x01\0\x04\0\x06cancel\x01\x04\x01@\0\0\x7f\x04\0\x0eis-job-pending\x01\
\x05\x04\0\x0bexecute-job\x01\x05\x03\x01\x14land:asyncio/asyncio\x05\x02\x04\x01\
;land:worker/asyncio-service-with-all-of-its-exports-removed\x04\0\x0b5\x01\0/as\
yncio-service-with-all-of-its-exports-removed\x03\0\0\0G\x09producers\x01\x0cpro\
cessed-by\x02\x0dwit-component\x070.215.0\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen::rt::maybe_link_cabi_realloc();
}
