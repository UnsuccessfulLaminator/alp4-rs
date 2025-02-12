use std::ops::{Range, Deref, DerefMut};



pub struct Bitplanes<D: AsRef<[u8]>> {
    width: usize,
    height: usize,
    planes: usize,
    plane_stride: usize,
    row_stride: usize,
    data: D
}

impl<D: AsRef<[u8]>> Bitplanes<D> {
    pub fn width(&self) -> usize {
        self.width
    }
    
    pub fn height(&self) -> usize {
        self.height
    }
    
    pub fn planes(&self) -> usize {
        self.planes
    }

    fn calc_indices(&self, plane: usize, x: usize, y: usize) -> (usize, usize) {
        let byte_idx = self.plane_stride*plane+self.row_stride*y+x/8;
        let bit_idx = 7-x%8;

        (byte_idx, bit_idx)
    }

    pub fn get(&self, plane: usize, x: usize, y: usize) -> bool {
        let (byte_idx, bit_idx) = self.calc_indices(plane, x, y);

        (self.data.as_ref()[byte_idx] & 1 << bit_idx) > 0
    }
}

impl<D: AsRef<[u8]> + AsMut<[u8]>> Bitplanes<D> {
    pub fn set(&mut self, plane: usize, x: usize, y: usize, val: bool) {
        let (byte_idx, bit_idx) = self.calc_indices(plane, x, y);
        let data = self.data.as_mut();

        if val { data[byte_idx] |= 1 << bit_idx; }
        else { data[byte_idx] &= !(1 << bit_idx); }
    }

    pub fn fill(&mut self, val: bool) {
        self.data.as_mut().fill(if val { 255 } else { 0 });
    }
    
    pub fn fill_from_fn<F>(&mut self, mut f: F)
    where F: FnMut(usize, usize, usize) -> bool {
        let data = self.data.as_mut();

        for (i, plane) in data.chunks_mut(self.plane_stride).enumerate() {
            for (y, row) in plane.chunks_mut(self.row_stride).enumerate() {
                for x in 0..self.width {
                    let val = f(i, x, y);
                    let (byte_idx, bit_idx) = (x/8, 7-x%8);

                    if val { row[byte_idx] |= 1 << bit_idx; }
                    else { row[byte_idx] &= !(1 << bit_idx); }
                }
            }
        }
    }

    pub fn plane(&mut self, n: usize) -> Bitplanes<&mut [u8]> {
        self.plane_range(n..n+1)
    }

    pub fn plane_range(&mut self, range: Range<usize>) -> Bitplanes<&mut [u8]> {
        let start = range.start*self.plane_stride;
        let end = range.end*self.plane_stride;

        Bitplanes {
            width: self.width,
            height: self.height,
            planes: 1,
            plane_stride: self.plane_stride,
            row_stride: self.row_stride,
            data: &mut self.data.as_mut()[start..end]
        }
    }
}

impl Bitplanes<Vec<u8>> {
    pub fn new(planes: usize, width: usize, height: usize) -> Self {
        let row_stride = (width+7)/8;
        let plane_stride = row_stride*height;

        Self {
            width,
            height,
            planes,
            plane_stride,
            row_stride,
            data: vec![0; plane_stride*planes]
        }
    }

    pub fn from_fn<F>(planes: usize, width: usize, height: usize, f: F) -> Self
    where F: FnMut(usize, usize, usize) -> bool {
        let mut this = Self::new(planes, width, height);

        this.fill_from_fn(f);
        this
    }
}

impl<D: AsRef<[u8]>> Deref for Bitplanes<D> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<D: AsRef<[u8]> + AsMut<[u8]>> DerefMut for Bitplanes<D> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }
}
