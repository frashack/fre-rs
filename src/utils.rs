use super::*;


#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn transmute_unchecked <T: Copy, U: Copy> (any: T) -> U {
    *(std::ptr::addr_of!(any) as *const U)
}


#[derive(Debug, Clone)]
pub struct ArrayIter<'a> {
    arr: Array<'a>,
    len: u32,
    idx: u32,
}
impl<'a> ArrayIter<'a> {
    pub(crate) fn new (arr: Array<'a>) -> Self {
        Self { arr, len: arr.get_length(), idx: 0 }
    }
}
impl<'a> Iterator for ArrayIter<'a> {
    type Item = Object<'a>;
    fn next(&mut self) -> Option<Object<'a>> {
        if self.idx != self.len {
            let elem = self.arr.get_element(self.idx);
            self.idx = unsafe {self.idx.unchecked_add(1)};
            Some(elem)
        } else {None}
    }
}
#[derive(Debug, Clone)]
pub struct VectorIter<'a> {
    arr: Vector<'a>,
    len: u32,
    idx: u32,
}
impl<'a> VectorIter<'a> {
    pub(crate) fn new (arr: Vector<'a>) -> Self {
        Self { arr, len: arr.get_length(), idx: 0 }
    }
}
impl<'a> Iterator for VectorIter<'a> {
    type Item = Object<'a>;
    fn next(&mut self) -> Option<Object<'a>> {
        if self.idx != self.len {
            let elem = self.arr.get_element(self.idx).unwrap();
            self.idx = unsafe {self.idx.unchecked_add(1)};
            Some(elem)
        } else {None}
    }
}


#[derive(Debug)]
pub struct BitmapDataAdapter<'a> {
    object: FREObject,
    width: usize,
    height: usize,
    height_sub_one: usize,
    has_alpha: bool,
    is_premultiplied: bool,
    stride: usize,
    is_inverted_y: bool,
    data: &'a mut [u32],
}
impl<'a> BitmapDataAdapter<'a> {
    pub fn width(self) -> usize {self.width}
    pub fn height(self) -> usize {self.height}
    pub fn has_alpha(self) -> bool {self.has_alpha}
    pub fn is_premultiplied(self) -> bool {self.is_premultiplied}
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn new (object: FREObject, descriptor: FREBitmapData2) -> Self {
        debug_assert!(descriptor.lineStride32 >= descriptor.width);
        assert!(!descriptor.bits32.is_null());
        let (width, height) = (descriptor.width as usize, descriptor.height as usize);
        let line_stride = descriptor.lineStride32 as usize;
        Self {
            object,
            width,
            height,
            height_sub_one: unsafe {height.unchecked_sub(1)},
            has_alpha: descriptor.hasAlpha != 0,
            is_premultiplied: descriptor.isPremultiplied != 0,
            stride: line_stride,
            is_inverted_y: descriptor.isInvertedY != 0,
            data: std::slice::from_raw_parts_mut(descriptor.bits32, line_stride*height),
        }
    }
    pub fn pixel (&self, x: usize, y: usize) -> u32 {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        let offset = unsafe {
            if self.is_inverted_y {self.height_sub_one-y} else {y}
                .unchecked_mul(self.stride)
                .unchecked_add(x)
        };
        unsafe {*(self.data.as_ptr().add(offset))}
    }
    pub fn pixel_mut (&mut self, x: usize, y: usize) -> &'a mut u32 {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        let offset = unsafe {
            if self.is_inverted_y {self.height_sub_one-y} else {y}
                .unchecked_mul(self.stride)
                .unchecked_add(x)
        };
        unsafe {&mut *(self.data.as_mut_ptr().add(offset))}
    }
    pub fn row (&self, y: usize) -> &'a [u32] {
        debug_assert!(y < self.height);
        let offset = unsafe {
            if self.is_inverted_y {self.height_sub_one-y} else {y}
                .unchecked_mul(self.stride)
        };
        unsafe {std::slice::from_raw_parts(self.data.as_ptr().add(offset), self.width)}
    }
    pub fn row_mut (&mut self, y: usize) -> &'a mut [u32] {
        debug_assert!(y < self.height);
        let offset = unsafe {
            if self.is_inverted_y {self.height_sub_one-y} else {y}
                .unchecked_mul(self.stride)
        };
        unsafe {std::slice::from_raw_parts_mut(self.data.as_mut_ptr().add(offset), self.width)}
    }
    pub fn iter (&'a self) -> BitmapDataIter<'a> {BitmapDataIter::new(self)}
    pub fn iter_mut (&'a mut self) -> BitmapDataIterMut<'a> {BitmapDataIterMut::new(self)}
    pub fn invalidate_rect (&self, x: u32, y: u32, width: u32, height: u32) {
        let r = unsafe {FREInvalidateBitmapDataRect(self.object, x, y, width, height)};
        debug_assert!(r.is_ok());
    }
}
#[derive(Debug, Clone)]
pub struct BitmapDataIter<'a> {
    adapter: &'a BitmapDataAdapter<'a>,
    y: usize,
}
impl<'a> BitmapDataIter<'a> {
    fn new (adapter: &'a BitmapDataAdapter<'a>) -> Self {
        Self { adapter, y: 0 }
    }
}
impl<'a> Iterator for BitmapDataIter<'a> {
    type Item = &'a [u32];
    fn next(&mut self) -> Option<Self::Item> {
        if self.y != self.adapter.height {
            let row = self.adapter.row(self.y);
            self.y = unsafe {self.y.unchecked_add(1)};
            Some(row)
        } else {None}
    }
}
#[derive(Debug)]
pub struct BitmapDataIterMut<'a> {
    adapter: &'a mut BitmapDataAdapter<'a>,
    y: usize,
}
impl<'a> BitmapDataIterMut<'a> {
    fn new (adapter: &'a mut BitmapDataAdapter<'a>) -> Self {
        Self { adapter, y: 0 }
    }
}
impl<'a> Iterator for BitmapDataIterMut<'a> {
    type Item = &'a mut [u32];
    fn next(&mut self) -> Option<Self::Item> {
        if self.y != self.adapter.height {
            let row = self.adapter.row_mut(self.y);
            self.y = unsafe {self.y.unchecked_add(1)};
            Some(row)
        } else {None}
    }
}


#[derive(Debug)]
pub struct MediaBufferDataAdapter<'a> {
    width: usize,
    height: usize,
    stride: usize,
    format: u32,
    data: &'a mut [u8],
}
impl<'a> MediaBufferDataAdapter<'a> {
    pub fn width(self) -> usize {self.width}
    pub fn height(self) -> usize {self.height}
    /// For future usage: currently images are ARGB format.
    pub fn format(self) -> u32 {self.format}
    #[allow(unsafe_op_in_unsafe_fn)]
    pub(crate) unsafe fn new (bytes: *mut u8, width: u32, height: u32, stride: u32, format: u32) -> Self {
        debug_assert!(stride >= width);
        debug_assert!(!bytes.is_null());
        let (width, height) = (width as usize, height as usize);
        let stride = stride as usize;
        Self {
            width,
            height,
            stride,
            format,
            data: std::slice::from_raw_parts_mut(bytes, stride * height * 4),
        }
    }
    pub fn pixel (&self, x: usize, y: usize) -> u32 {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        unsafe {
            let offset = y.unchecked_mul(self.stride).unchecked_add(x);
            let ptr = self.data.as_ptr() as *const u32;
            *(ptr.add(offset))
        }
    }
    pub fn pixel_mut (&mut self, x: usize, y: usize) -> &'a mut u32 {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        unsafe {
            let offset = y.unchecked_mul(self.stride).unchecked_add(x);
            let ptr = self.data.as_mut_ptr() as *mut u32;
            &mut *(ptr.add(offset))
        }
    }
    pub fn row (&self, y: usize) -> &'a [u32] {
        debug_assert!(y < self.height);
        unsafe {
            let offset = y.unchecked_mul(self.stride);
            let ptr = self.data.as_ptr() as *const u32;
            std::slice::from_raw_parts(ptr.add(offset), self.width)
        }
    }
    pub fn row_mut (&mut self, y: usize) -> &'a mut [u32] {
        debug_assert!(y < self.height);
        unsafe {
            let offset = y.unchecked_mul(self.stride);
            let ptr = self.data.as_mut_ptr() as *mut u32;
            std::slice::from_raw_parts_mut(ptr.add(offset), self.width)
        }
    }
    pub fn iter (&'a self) -> MediaBufferDataIter<'a> {MediaBufferDataIter::new(self)}
    pub fn iter_mut (&'a mut self) -> MediaBufferDataIterMut<'a> {MediaBufferDataIterMut::new(self)}
}
#[derive(Debug, Clone)]
pub struct MediaBufferDataIter<'a> {
    adapter: &'a MediaBufferDataAdapter<'a>,
    y: usize,
}
impl<'a> MediaBufferDataIter<'a> {
    fn new (adapter: &'a MediaBufferDataAdapter<'a>) -> Self {
        Self { adapter, y: 0 }
    }
}
impl<'a> Iterator for MediaBufferDataIter<'a> {
    type Item = &'a [u32];
    fn next(&mut self) -> Option<Self::Item> {
        if self.y != self.adapter.height {
            let row = self.adapter.row(self.y);
            self.y = unsafe {self.y.unchecked_add(1)};
            Some(row)
        } else {None}
    }
}
#[derive(Debug)]
pub struct MediaBufferDataIterMut<'a> {
    adapter: &'a mut MediaBufferDataAdapter<'a>,
    y: usize,
}
impl<'a> MediaBufferDataIterMut<'a> {
    fn new (adapter: &'a mut MediaBufferDataAdapter<'a>) -> Self {
        Self { adapter, y: 0 }
    }
}
impl<'a> Iterator for MediaBufferDataIterMut<'a> {
    type Item = &'a mut [u32];
    fn next(&mut self) -> Option<Self::Item> {
        if self.y != self.adapter.height {
            let row = self.adapter.row_mut(self.y);
            self.y = unsafe {self.y.unchecked_add(1)};
            Some(row)
        } else {None}
    }
}