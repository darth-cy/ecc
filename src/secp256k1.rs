use std::str::FromStr;
use crate::ru256::RU256;
use crate::bytes;

#[derive(Debug, Clone)]
pub struct Point {
    pub x: RU256,
    pub y: RU256
}

impl Point {
    pub fn from_hex_coordinates(x: &str, y: &str) -> Self {
        return Point {
            x: RU256::from_str(x).unwrap(),
            y: RU256::from_str(y).unwrap()
        };
    }
    pub fn to_hex_string(&self) -> String {
        return format!("04{}{}", self.x.to_string(), self.y.to_string());
    }
    pub fn is_zero_point(&self) -> bool {
        return self.x == RU256::from_str("0x0").unwrap() && self.y == RU256::from_str("0x0").unwrap();
    }
}

pub struct SECP256K1;

impl SECP256K1 {

    // ******************************************************************
    // SECP256K1 Curve Parameters
    // Reference: https://www.secg.org/sec2-v2.pdf
    // ******************************************************************

    pub fn p() -> RU256 {
        return RU256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F").unwrap();
    }
    pub fn g() -> Point {
        return Point {
            x: RU256::from_str("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798").unwrap(),
            y: RU256::from_str("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8").unwrap()
        };
    }
    pub fn n() -> RU256 {
        return RU256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141").unwrap();
    }

    // ******************************************************************
    // Identity Element
    // **NOTE: Imaginary. Implemented by setting both coordinates as 0
    //         need to check during operations
    // ******************************************************************

    pub fn zero_point() -> Point {
        return Point {
            x: RU256::from_str("0x0").unwrap(),
            y: RU256::from_str("0x0").unwrap()
        };
    }

    // ******************************************************************
    // Point addition
    // **NOTE: All arithmetics are mod p
    // 
    // use formula:
    // 
    // calculate slope (addition): lambda = (y1 - y2) / (x1 - x2)
    // 
    // Result point R = (x3, y3)
    // x3 = lambda^2 - x1 - x2
    // y3 = lambda * (x1 - x3) - y1
    // ******************************************************************

    pub fn add_points(pt1: &Point, pt2: &Point) -> Point {
        println!("adding");
        assert!(pt1.y != pt2.y);

        if pt1.is_zero_point() { return pt2.clone(); }
        if pt2.is_zero_point() { return pt1.clone(); }

        let p = &Self::p();

        // slope calculation
        let y_diff = &pt1.y.sub_mod(&pt2.y, p);
        let x_diff = &pt1.x.sub_mod(&pt2.x, p);
        let lambda = &y_diff.div_mod(x_diff, p);

        // calculate new x3
        let x3 = &lambda.mul_mod(lambda, p).sub_mod(&pt1.x, p).sub_mod(&pt2.x, p);

        // calculate new y3
        let y3 = &pt1.x.sub_mod(x3, p).mul_mod(lambda, p).sub_mod(&pt1.y, p);

        return Point { x: x3.clone(), y: y3.clone() };
    }

    // ******************************************************************
    // Point doubling
    // **NOTE: All arithmetics are mod p
    // 
    // use formula:
    // 
    // calculate slope (doubling): lambda = (3 * x1^2 + a) / (2 * y)
    // 
    // Result point R = (x3, y3)
    // x3 = lambda^2 - x1 - x2
    // y3 = lambda * (x1 - x3) - y1
    // ******************************************************************

    pub fn double_point(pt: &Point) -> Point {
        println!("doubling");
        if pt.is_zero_point() { return Self::zero_point().clone(); }
        if pt.y == RU256::from_str("0x0").unwrap() { return Self::zero_point().clone(); }

        let p = &Self::p();
        let const_2 = &RU256::from_str("0x2").unwrap();
        let const_3 = &RU256::from_str("0x3").unwrap();

        // calculate slope
        let two_y = &pt.y.mul_mod(const_2, p);
        let x1_2_3 = &pt.x.mul_mod(&pt.x, p).mul_mod(const_3, p);
        let lambda = &x1_2_3.div_mod(two_y, p);

        // calculate new x3
        let x3 = &lambda.mul_mod(lambda, p).sub_mod(&pt.x, p).sub_mod(&pt.x, p);

        // calculate new y3
        let y3 = &pt.x.sub_mod(x3, p).mul_mod(lambda, p).sub_mod(&pt.y, p);

        return Point { x: x3.clone(), y: y3.clone() };
    }

    pub fn pr_to_pub(pr: &RU256) -> Point {
        let mut bytes: [u8; 32] = [0; 32];
        pr.to_bytes(&mut bytes);

        let mut binaries: Vec<u8> = vec![];
        bytes::bytes_to_binary(&bytes, &mut binaries);

        let mut base = Self::zero_point().clone();
        let adder = Self::g().clone();

        let mut on = false;
        let mut step = 0;
        for d in binaries.into_iter() {
            println!("step: {}, bit: {}", step, d);
            if on {
                base = Self::double_point(&base);
            }
            if d > 0 { 
                on = true;
                base = Self::add_points(&base, &adder);
            }
            step += 1;
        }

        return base;
    }

}



mod tests {
    use crate::secp256k1::*;

    #[test]
    fn secp256k1_add_poins() {
        let pt1 = Point::from_hex_coordinates(
            "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"
        );
        let pt2 = Point::from_hex_coordinates(
            "C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5",
            "1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A"
        );
        let pt3 = SECP256K1::add_points(&pt1, &pt2);

        assert_eq!(pt3.to_hex_string(), "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9 388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672");
    }

    #[test]
    fn secp256k1_double_point() {
        let pt1 = Point::from_hex_coordinates(
            "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"
        );

        let pt2 = SECP256K1::double_point(&pt1);
        let pt3 = SECP256K1::double_point(&pt2);

        assert_eq!(pt3.to_hex_string(), "e493dbf1c10d80f3581e4904930b1404cc6c13900ee0758474fa94abe8c4cd13 51ed993ea0d455b75642e2098ea51448d967ae33bfbdfe40cfe97bdc47739922");
    }
}
