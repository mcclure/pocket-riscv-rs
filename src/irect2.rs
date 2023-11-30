// 2D rectangle class/helper methods based on glam ivec2
// Notes:
//    0,0 is top left
//    For all rectangle classes, br is "non-inclusive":
//    The rectangle [[10,10],[12,12]] includes [10,10], [11,11] but not [12,12]

#![allow(dead_code)]

use glam::IVec2;

// Vector helpers
pub fn ivec2_within(size:IVec2, at:IVec2) -> bool {
    IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
}
pub fn ivec2_le(left:IVec2, right:IVec2) -> bool {
    left.cmple(right).all()
}
pub fn ivec2_lt(left:IVec2, right:IVec2) -> bool { // Unused
    left.cmplt(right).all()
}
pub fn ivec2_ge(left:IVec2, right:IVec2) -> bool {
    left.cmpge(right).all()
}
pub fn ivec2_gt(left:IVec2, right:IVec2) -> bool {
    left.cmpgt(right).all()
}

// Rectangle class
#[derive(Debug, Clone, Copy)]
pub struct IRect2 { // br is non-inclusive
    pub ul: IVec2,  // Upper left
    pub br: IVec2   // Bottom right
}
impl IRect2 {
    pub fn new(ul:IVec2, br:IVec2) -> Self { Self {ul, br} }
    pub fn new_centered(center:IVec2, size:IVec2) -> Self {
        let br = center + size/2; // Bias placement toward upper-left
        let ul = br - size;
        Self {ul, br}
    }
    pub fn within(&self, test:IVec2) -> bool {
        ivec2_le(self.ul, test) && ivec2_gt(self.br, test)
    }
    pub fn intersect(&self, test:IRect2) -> bool { // Will misbehave on 0-size rects
        self.within(test.ul) || {
            let in_br = test.br+IVec2::NEG_ONE; // For testing within the point just inside must be in
            self.within(in_br) || // All 4 corners
            self.within(IVec2::new(test.ul.x, in_br.y)) ||
            self.within(IVec2::new(in_br.x, test.ul.y))
        }
    }
    pub fn enclose(&self, test:IRect2) -> bool {
        ivec2_le(self.ul, test.ul) && ivec2_ge(self.br, test.br) // For testing enclose the rects only need to coincide
    }
    
    pub fn size(&self) -> IVec2 {
        self.br - self.ul
    }
    pub fn center(&self) -> IVec2 {
        (self.br + self.ul)/2
    }
    pub fn offset(&self, by:IVec2) -> IRect2 {
        return IRect2::new(self.ul + by, self.br + by);
    }
    pub fn force_enclose_x(&self, test:IRect2) -> IRect2 { // ASSUMES SELF SMALLER THAN TEST
        let excess = test.ul.x - self.ul.x;
        if excess > 0 { return self.offset(IVec2::new(excess, 0)) }
        let excess = test.br.x - self.br.x;
        if excess < 0 { return self.offset(IVec2::new(excess, 0)) }
        self.clone()
    }
}

// Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_intersection() {
        let rect = IRect2::new(IVec2::new(5, 5), IVec2::new(15,15));
        for y in 0..3 {
            for x in 0..3 {
                let v = IVec2::new(x*10,y*10);
                assert_eq!(rect.within(v), (x==1 && y==1), "Incorrect within! rect: {:?} v: {:?}", rect, v);
                let r2 = IRect2::new_centered(v, IVec2::ONE*2);
                assert_eq!(rect.enclose(r2), (x==1 && y==1), "Incorrect enclose! rect: {:?} v: {:?}", rect, r2);
            }
        }
        for y in 0..5 {
            for x in 0..5 {
                let v = IVec2::new(x*5,y*5);
                let r2 = IRect2::new_centered(v, IVec2::ONE*2);
                assert_eq!(rect.intersect(r2), !(x==0 || x==4 || y==0 || y==4), "Incorrect intersect! rect: {:?} v: {:?}", rect, r2);
            }
        }
    }
}
