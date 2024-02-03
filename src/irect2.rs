// 2D rectangle class/helper methods based on glam ivec2
// Notes:
//    0,0 is top left
//    For all rectangle classes, br is "non-inclusive":
//    The rectangle [[10,10],[12,12]] includes [10,10], [11,11] but not [12,12]

#![allow(dead_code)]

use glam::IVec2;

// Vector helpers

// Is "at" within vect "size" rooted at 0,0?
pub fn ivec2_within(size:IVec2, at:IVec2) -> bool {
    IVec2::ZERO.cmple(at).all() && size.cmpgt(at).all()
}

// Is vector left (less than or equal, less than, less than, greater than or
// equal, greater than) vector right on all axes?
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
pub struct IRect2 {
    pub ul: IVec2,  // Upper Left
    pub br: IVec2   // Bottom Right (non-inclusive)
}

impl IRect2 {
    pub fn new(ul:IVec2, br:IVec2) -> Self { Self {ul, br} }

    pub fn new_centered(center:IVec2, size:IVec2) -> Self {
        let br = center + size/2; // Bias placement toward upper-left
        let ul = br - size;
        Self {ul, br}
    }

    // Arg vector is contained in rectangle
    pub fn within(&self, test:IVec2) -> bool {
        ivec2_le(self.ul, test) && ivec2_gt(self.br, test)
    }

    // Arg rectangle overlaps this one by at least one pixel
    pub fn intersect(&self, test:IRect2) -> bool { // Will misbehave on 0-size rects
        self.within(test.ul) || {
            let in_br = test.br+IVec2::NEG_ONE; // For testing within the point just inside must be in
            self.within(in_br) || // All 4 corners
            self.within(IVec2::new(test.ul.x, in_br.y)) ||
            self.within(IVec2::new(in_br.x, test.ul.y))
        }
    }

    // Arg rectangle is entirely contained within this one
    pub fn enclose(&self, test:IRect2) -> bool {
        ivec2_le(self.ul, test.ul) && ivec2_ge(self.br, test.br) // For testing enclose the rects only need to coincide
    }

    // Size of this rectangle
    pub fn size(&self) -> IVec2 {
        self.br - self.ul
    }

    // Integer midpoint of this rectangle
    pub fn center(&self) -> IVec2 {
        (self.br + self.ul)/2
    }

    // Copy of this rectangle offset by arg vector
    pub fn offset(&self, by:IVec2) -> IRect2 {
        return IRect2::new(self.ul + by, self.br + by);
    }

    // Copy of this rectangle, X-offset by whatever places it inside arg rectangle
    pub fn force_enclose_x(&self, test:IRect2) -> IRect2 { // ASSUMES SELF SMALLER THAN TEST
        let excess = test.ul.x - self.ul.x;
        if excess > 0 { return self.offset(IVec2::new(excess, 0)) }
        let excess = test.br.x - self.br.x;
        if excess < 0 { return self.offset(IVec2::new(excess, 0)) }
        self.clone()
    }

    pub fn overlap(&self, test:IRect2) -> Option<IRect2> { // FIXME: Naming in this class is vague about what is a query and what is a verb
        if !self.intersect(test) {
            None
        } else {
            Some(IRect2::new(
                self.ul.max(test.ul),
                self.br.min(test.br)
            ))
        }
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

    fn overlap() { // Not yet run
        let centered = IRect2::new(IVec2::new(1,1), IVec2::new(9,9));
        let away = IRect2::new(IVec2::ZERO, IVec2::new(1,1));
        let top    = IRect2::new(IVec2::new(4,0), IVec2::new(6,2));
        let left   = IRect2::new(IVec2::new(0,4), IVec2::new(2,6));
        let right  = IRect2::new(IVec2::new(8,4), IVec2::new(10,6));
        let bottom = IRect2::new(IVec2::new(4,8), IVec2::new(6,10));
        fn test_both(target:Option<IRect2>, a:IRect2, b:IRect2, b_name:&str) {
            assert_eq!(target, a.overlap(b), "Failed center [{:?}] .overlap {} [{:?}]", a, b_name, b);
            assert_eq!(target, b.overlap(a), "Failed {} [{:?}] .overlap center [{:?}]", b_name, b, a);
        }
        test_both(None, centered, away, "away");
        test_both(Some(IRect2::new(IVec2::new(4, 1), IVec2::new(6, 2))), centered, top, "top");
        test_both(Some(IRect2::new(IVec2::new(1, 4), IVec2::new(2, 6))), centered, left, "left");
        test_both(Some(IRect2::new(IVec2::new(8, 4), IVec2::new(4, 10))), centered, right, "right");
        test_both(Some(IRect2::new(IVec2::new(4, 9), IVec2::new(6, 10))), centered, bottom, "bottom");
    }
}
