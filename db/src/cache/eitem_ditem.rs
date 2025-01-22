use std::mem::transmute_copy;
use heed::{BytesDecode, BytesEncode};

pub fn as_eitem<'a, T: BytesEncode<'a>>(item: &'a T) -> &'a T::EItem {
    unsafe { transmute_copy(&item) }
}

pub fn as_ditem<'a, T: BytesDecode<'a>>(item: &'a T) -> T::DItem {
    unsafe { transmute_copy(&item) }
}

pub fn from_eitem<'a, T: BytesEncode<'a>>(item: &<T as BytesEncode<'a>>::EItem) -> &'a T {
    unsafe { transmute_copy(&item) }
}

pub fn from_ditem<'a, T: BytesDecode<'a>>(item: &<T as BytesDecode<'a>>::DItem) -> &'a T {
    unsafe { transmute_copy(&item) }
}
