use std::ops::{Range, Deref, DerefMut, Index};



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

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn to_owned(&self) -> Bitplanes<Vec<u8>> {
        Bitplanes {
            width: self.width,
            height: self.height,
            planes: self.planes,
            plane_stride: self.plane_stride,
            row_stride: self.row_stride,
            data: self.as_slice().to_vec()
        }
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

    pub fn copy_from<O: AsRef<[u8]>>(&mut self, other: &Bitplanes<O>) {
        if self.planes != other.planes { panic!("different number of planes"); }
        if self.width != other.width { panic!("different plane widths"); }
        if self.height != other.height { panic!("different plane heights"); }

        let dst = self.data.as_mut();
        let src = other.as_slice();

        if self.plane_stride == other.plane_stride
        && self.row_stride == other.row_stride {
            dst.copy_from_slice(src);
        }
        else if self.row_stride == other.row_stride {
            let len = self.row_stride*self.height;

            dst.chunks_mut(self.plane_stride)
                .zip(src.chunks(other.plane_stride))
                .for_each(|(dst_plane, src_plane)| {
                    dst_plane[..len].copy_from_slice(&src_plane[..len])
                });
        }
        else {
            let len = (self.width+7)/8;

            dst.chunks_mut(self.plane_stride)
                .zip(src.chunks(other.plane_stride))
                .for_each(|(dst_plane, src_plane)| {
                    dst_plane.chunks_mut(self.row_stride)
                        .zip(src_plane.chunks(other.row_stride))
                        .take(self.height)
                        .for_each(|(dst_row, src_row)| {
                            dst_row[..len].copy_from_slice(&src_row[..len]);
                        })
                });
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
            planes: range.end-range.start,
            plane_stride: self.plane_stride,
            row_stride: self.row_stride,
            data: &mut self.data.as_mut()[start..end]
        }
    }

    pub fn swap_planes(&mut self, p0: usize, p1: usize) {
        if p0 == p1 { panic!("cannot swap plane with itself"); }

        let (p0, p1) = (p0.min(p1), p0.max(p1));
        let split = p1*self.plane_stride;
        let (data1, data2) = self.data.as_mut().split_at_mut(split);
        let start = p0*self.plane_stride;
        let end = start+self.plane_stride;

        data1[start..end].swap_with_slice(&mut data2[..self.plane_stride]);
    }

    pub fn split_at_plane(&mut self, p: usize)
    -> (Bitplanes<&mut [u8]>, Bitplanes<&mut [u8]>) {
        let split = p*self.plane_stride;
        let (d0, d1) = self.data.as_mut().split_at_mut(split);

        let p0 = Bitplanes {
            width: self.width,
            height: self.height,
            planes: p,
            plane_stride: self.plane_stride,
            row_stride: self.row_stride,
            data: d0
        };
        
        let p1 = Bitplanes {
            width: self.width,
            height: self.height,
            planes: self.planes-p,
            plane_stride: self.plane_stride,
            row_stride: self.row_stride,
            data: d1
        };

        (p0, p1)
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.data.as_mut()
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

impl<D: AsRef<[u8]>> Index<[usize; 3]> for Bitplanes<D> {
    type Output = bool;

    fn index(&self, idx: [usize; 3]) -> &bool {
        if self.get(idx[0], idx[1], idx[2]) { &true }
        else { &false }
    }
}
