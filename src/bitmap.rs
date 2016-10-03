
#[derive(Debug)]
pub struct Bitmap<T> {
    pub bitmap: Vec<T>,
    pub width: usize,
    pub height: usize,
}


#[derive(Debug, Copy, Clone)]
pub struct BitmapRef<'a, T: 'a> {
    pub bitmap: &'a [T],
    pub width: usize,
    pub height: usize,
}

impl<'a, T> BitmapRef<'a, T> {
    pub fn new(bitmap: &'a [T], width: usize, height: usize) -> BitmapRef<'a, T> {
        BitmapRef {
            bitmap: bitmap,
            width: width,
            height: height,
        }
    }
}

impl<T> Bitmap<T> {
    pub fn new(bitmap: Vec<T>, width: usize, height: usize) -> Bitmap<T> {
        Bitmap {
            bitmap: bitmap,
            width: width,
            height: height,
        }
    }

    pub fn new_ref(&self) -> BitmapRef<T> {
        BitmapRef::new(self.bitmap.as_ref(), self.width, self.height)
    }
}
