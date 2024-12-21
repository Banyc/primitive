macro_rules! define_unsigned {
    ($ty: ident, $size: expr, $primitive: ident, [$($from: ident),*], [$($into: ident),*], $nonzero_ty: ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ty {
            value: $primitive,
        }
        impl $ty {
            const MAX_MASK: $primitive = (1 << $size) - 1;
            pub const MAX: Self = Self::new((1 << $size) - 1).unwrap();
            pub const MIN: Self = Self::new(0).unwrap();
            pub const fn new(value: $primitive) -> Option<Self> {
                if value <= Self::MAX_MASK {
                    Some(Self { value })
                } else {
                    None
                }
            }
            pub const fn checked_add(&self, other: Self) -> Option<Self> {
                match self.value.checked_add(other.value) {
                    Some(value) => Self::new(value),
                    None => None,
                }
            }
            pub const fn checked_sub(&self, other: Self) -> Option<Self> {
                match self.value.checked_sub(other.value) {
                    Some(value) => Self::new(value),
                    None => None,
                }
            }
            pub const fn saturating_add(&self, other: Self) -> Self {
                let value = self.value.saturating_add(other.value);
                if value <= Self::MAX_MASK {
                    Self::new(value).unwrap()
                } else {
                    Self::MAX
                }
            }
            pub const fn saturating_sub(&self, other: Self) -> Self {
                let value = self.value.saturating_sub(other.value);
                Self::new(value).unwrap()
            }
            pub const fn wrapping_add(&self, other: Self) -> Self {
                Self::new((self.value + other.value) & Self::MAX_MASK).unwrap()
            }
            pub const fn wrapping_sub(&self, other: Self) -> Self {
                let a = self.value | (1 << $size);
                Self::new((a - other.value) & Self::MAX_MASK).unwrap()
            }
        }
        impl From<$ty> for $primitive {
            fn from(value: $ty) -> Self {
                value.value
            }
        }
        $(
            impl From<$from> for $ty {
                fn from(value: $from) -> Self {
                    Self::new(value.into()).unwrap()
                }
            }
        )*
        $(
            impl From<$ty> for $into {
                fn from(value: $ty) -> Self {
                    value.value.into()
                }
            }
        )*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $nonzero_ty {
            value: $ty,
        }
        impl $nonzero_ty {
            pub const fn new(value: $ty) -> Option<Self> {
                if value.value == $ty::MIN.value {
                    None
                } else {
                    Some(Self { value })
                }
            }
            pub const fn get(&self) -> $ty {
                self.value
            }
        }
    };
}

define_unsigned!(U2, 2, u8, [], [u16, u32, u64, u128], NonZeroU2);
define_unsigned!(U3, 3, u8, [], [u16, u32, u64, u128], NonZeroU3);
define_unsigned!(U4, 4, u8, [], [u16, u32, u64, u128], NonZeroU4);
define_unsigned!(U5, 5, u8, [], [u16, u32, u64, u128], NonZeroU5);
define_unsigned!(U6, 6, u8, [], [u16, u32, u64, u128], NonZeroU6);
define_unsigned!(U7, 7, u8, [], [u16, u32, u64, u128], NonZeroU7);
define_unsigned!(U9, 9, u16, [u8], [u32, u64, u128], NonZeroU9);
define_unsigned!(U10, 10, u16, [u8], [u32, u64, u128], NonZeroU10);
define_unsigned!(U11, 11, u16, [u8], [u32, u64, u128], NonZeroU11);
define_unsigned!(U12, 12, u16, [u8], [u32, u64, u128], NonZeroU12);
define_unsigned!(U13, 13, u16, [u8], [u32, u64, u128], NonZeroU13);
define_unsigned!(U14, 14, u16, [u8], [u32, u64, u128], NonZeroU14);
define_unsigned!(U15, 15, u16, [u8], [u32, u64, u128], NonZeroU15);
define_unsigned!(U17, 17, u32, [u8, u16], [u64, u128], NonZeroU17);
define_unsigned!(U18, 18, u32, [u8, u16], [u64, u128], NonZeroU18);
define_unsigned!(U19, 19, u32, [u8, u16], [u64, u128], NonZeroU19);
define_unsigned!(U20, 20, u32, [u8, u16], [u64, u128], NonZeroU20);
define_unsigned!(U21, 21, u32, [u8, u16], [u64, u128], NonZeroU21);
define_unsigned!(U22, 22, u32, [u8, u16], [u64, u128], NonZeroU22);
define_unsigned!(U23, 23, u32, [u8, u16], [u64, u128], NonZeroU23);
define_unsigned!(U24, 24, u32, [u8, u16], [u64, u128], NonZeroU24);
define_unsigned!(U25, 25, u32, [u8, u16], [u64, u128], NonZeroU25);
define_unsigned!(U26, 26, u32, [u8, u16], [u64, u128], NonZeroU26);
define_unsigned!(U27, 27, u32, [u8, u16], [u64, u128], NonZeroU27);
define_unsigned!(U28, 28, u32, [u8, u16], [u64, u128], NonZeroU28);
define_unsigned!(U29, 29, u32, [u8, u16], [u64, u128], NonZeroU29);
define_unsigned!(U30, 30, u32, [u8, u16], [u64, u128], NonZeroU30);
define_unsigned!(U31, 31, u32, [u8, u16], [u64, u128], NonZeroU31);
define_unsigned!(U33, 33, u64, [u8, u16, u32], [u128], NonZeroU33);
define_unsigned!(U34, 34, u64, [u8, u16, u32], [u128], NonZeroU34);
define_unsigned!(U35, 35, u64, [u8, u16, u32], [u128], NonZeroU35);
define_unsigned!(U36, 36, u64, [u8, u16, u32], [u128], NonZeroU36);
define_unsigned!(U37, 37, u64, [u8, u16, u32], [u128], NonZeroU37);
define_unsigned!(U38, 38, u64, [u8, u16, u32], [u128], NonZeroU38);
define_unsigned!(U39, 39, u64, [u8, u16, u32], [u128], NonZeroU39);
define_unsigned!(U40, 40, u64, [u8, u16, u32], [u128], NonZeroU40);
define_unsigned!(U41, 41, u64, [u8, u16, u32], [u128], NonZeroU41);
define_unsigned!(U42, 42, u64, [u8, u16, u32], [u128], NonZeroU42);
define_unsigned!(U43, 43, u64, [u8, u16, u32], [u128], NonZeroU43);
define_unsigned!(U44, 44, u64, [u8, u16, u32], [u128], NonZeroU44);
define_unsigned!(U45, 45, u64, [u8, u16, u32], [u128], NonZeroU45);
define_unsigned!(U46, 46, u64, [u8, u16, u32], [u128], NonZeroU46);
define_unsigned!(U47, 47, u64, [u8, u16, u32], [u128], NonZeroU47);
define_unsigned!(U48, 48, u64, [u8, u16, u32], [u128], NonZeroU48);
define_unsigned!(U49, 49, u64, [u8, u16, u32], [u128], NonZeroU49);
define_unsigned!(U50, 50, u64, [u8, u16, u32], [u128], NonZeroU50);
define_unsigned!(U51, 51, u64, [u8, u16, u32], [u128], NonZeroU51);
define_unsigned!(U52, 52, u64, [u8, u16, u32], [u128], NonZeroU52);
define_unsigned!(U53, 53, u64, [u8, u16, u32], [u128], NonZeroU53);
define_unsigned!(U54, 54, u64, [u8, u16, u32], [u128], NonZeroU54);
define_unsigned!(U55, 55, u64, [u8, u16, u32], [u128], NonZeroU55);
define_unsigned!(U56, 56, u64, [u8, u16, u32], [u128], NonZeroU56);
define_unsigned!(U57, 57, u64, [u8, u16, u32], [u128], NonZeroU57);
define_unsigned!(U58, 58, u64, [u8, u16, u32], [u128], NonZeroU58);
define_unsigned!(U59, 59, u64, [u8, u16, u32], [u128], NonZeroU59);
define_unsigned!(U60, 60, u64, [u8, u16, u32], [u128], NonZeroU60);
define_unsigned!(U61, 61, u64, [u8, u16, u32], [u128], NonZeroU61);
define_unsigned!(U62, 62, u64, [u8, u16, u32], [u128], NonZeroU62);
define_unsigned!(U63, 63, u64, [u8, u16, u32], [u128], NonZeroU63);
define_unsigned!(U65, 65, u128, [u8, u16, u32, u64], [], NonZeroU65);
define_unsigned!(U66, 66, u128, [u8, u16, u32, u64], [], NonZeroU66);
define_unsigned!(U67, 67, u128, [u8, u16, u32, u64], [], NonZeroU67);
define_unsigned!(U68, 68, u128, [u8, u16, u32, u64], [], NonZeroU68);
define_unsigned!(U69, 69, u128, [u8, u16, u32, u64], [], NonZeroU69);
define_unsigned!(U70, 70, u128, [u8, u16, u32, u64], [], NonZeroU70);
define_unsigned!(U71, 71, u128, [u8, u16, u32, u64], [], NonZeroU71);
define_unsigned!(U72, 72, u128, [u8, u16, u32, u64], [], NonZeroU72);
define_unsigned!(U73, 73, u128, [u8, u16, u32, u64], [], NonZeroU73);
define_unsigned!(U74, 74, u128, [u8, u16, u32, u64], [], NonZeroU74);
define_unsigned!(U75, 75, u128, [u8, u16, u32, u64], [], NonZeroU75);
define_unsigned!(U76, 76, u128, [u8, u16, u32, u64], [], NonZeroU76);
define_unsigned!(U77, 77, u128, [u8, u16, u32, u64], [], NonZeroU77);
define_unsigned!(U78, 78, u128, [u8, u16, u32, u64], [], NonZeroU78);
define_unsigned!(U79, 79, u128, [u8, u16, u32, u64], [], NonZeroU79);
define_unsigned!(U80, 80, u128, [u8, u16, u32, u64], [], NonZeroU80);
define_unsigned!(U81, 81, u128, [u8, u16, u32, u64], [], NonZeroU81);
define_unsigned!(U82, 82, u128, [u8, u16, u32, u64], [], NonZeroU82);
define_unsigned!(U83, 83, u128, [u8, u16, u32, u64], [], NonZeroU83);
define_unsigned!(U84, 84, u128, [u8, u16, u32, u64], [], NonZeroU84);
define_unsigned!(U85, 85, u128, [u8, u16, u32, u64], [], NonZeroU85);
define_unsigned!(U86, 86, u128, [u8, u16, u32, u64], [], NonZeroU86);
define_unsigned!(U87, 87, u128, [u8, u16, u32, u64], [], NonZeroU87);
define_unsigned!(U88, 88, u128, [u8, u16, u32, u64], [], NonZeroU88);
define_unsigned!(U89, 89, u128, [u8, u16, u32, u64], [], NonZeroU89);
define_unsigned!(U90, 90, u128, [u8, u16, u32, u64], [], NonZeroU90);
define_unsigned!(U91, 91, u128, [u8, u16, u32, u64], [], NonZeroU91);
define_unsigned!(U92, 92, u128, [u8, u16, u32, u64], [], NonZeroU92);
define_unsigned!(U93, 93, u128, [u8, u16, u32, u64], [], NonZeroU93);
define_unsigned!(U94, 94, u128, [u8, u16, u32, u64], [], NonZeroU94);
define_unsigned!(U95, 95, u128, [u8, u16, u32, u64], [], NonZeroU95);
define_unsigned!(U96, 96, u128, [u8, u16, u32, u64], [], NonZeroU96);
define_unsigned!(U97, 97, u128, [u8, u16, u32, u64], [], NonZeroU97);
define_unsigned!(U98, 98, u128, [u8, u16, u32, u64], [], NonZeroU98);
define_unsigned!(U99, 99, u128, [u8, u16, u32, u64], [], NonZeroU99);
define_unsigned!(U100, 100, u128, [u8, u16, u32, u64], [], NonZeroU100);
define_unsigned!(U101, 101, u128, [u8, u16, u32, u64], [], NonZeroU101);
define_unsigned!(U102, 102, u128, [u8, u16, u32, u64], [], NonZeroU102);
define_unsigned!(U103, 103, u128, [u8, u16, u32, u64], [], NonZeroU103);
define_unsigned!(U104, 104, u128, [u8, u16, u32, u64], [], NonZeroU104);
define_unsigned!(U105, 105, u128, [u8, u16, u32, u64], [], NonZeroU105);
define_unsigned!(U106, 106, u128, [u8, u16, u32, u64], [], NonZeroU106);
define_unsigned!(U107, 107, u128, [u8, u16, u32, u64], [], NonZeroU107);
define_unsigned!(U108, 108, u128, [u8, u16, u32, u64], [], NonZeroU108);
define_unsigned!(U109, 109, u128, [u8, u16, u32, u64], [], NonZeroU109);
define_unsigned!(U110, 110, u128, [u8, u16, u32, u64], [], NonZeroU110);
define_unsigned!(U111, 111, u128, [u8, u16, u32, u64], [], NonZeroU111);
define_unsigned!(U112, 112, u128, [u8, u16, u32, u64], [], NonZeroU112);
define_unsigned!(U113, 113, u128, [u8, u16, u32, u64], [], NonZeroU113);
define_unsigned!(U114, 114, u128, [u8, u16, u32, u64], [], NonZeroU114);
define_unsigned!(U115, 115, u128, [u8, u16, u32, u64], [], NonZeroU115);
define_unsigned!(U116, 116, u128, [u8, u16, u32, u64], [], NonZeroU116);
define_unsigned!(U117, 117, u128, [u8, u16, u32, u64], [], NonZeroU117);
define_unsigned!(U118, 118, u128, [u8, u16, u32, u64], [], NonZeroU118);
define_unsigned!(U119, 119, u128, [u8, u16, u32, u64], [], NonZeroU119);
define_unsigned!(U120, 120, u128, [u8, u16, u32, u64], [], NonZeroU120);
define_unsigned!(U121, 121, u128, [u8, u16, u32, u64], [], NonZeroU121);
define_unsigned!(U122, 122, u128, [u8, u16, u32, u64], [], NonZeroU122);
define_unsigned!(U123, 123, u128, [u8, u16, u32, u64], [], NonZeroU123);
define_unsigned!(U124, 124, u128, [u8, u16, u32, u64], [], NonZeroU124);
define_unsigned!(U125, 125, u128, [u8, u16, u32, u64], [], NonZeroU125);
define_unsigned!(U126, 126, u128, [u8, u16, u32, u64], [], NonZeroU126);
define_unsigned!(U127, 127, u128, [u8, u16, u32, u64], [], NonZeroU127);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u2() {
        let a = U2::new(1).unwrap();
        let b = U2::new(1).unwrap();
        assert_eq!(a.checked_add(b), Some(U2::new(0b10).unwrap()));
        assert_eq!(a.checked_sub(b), Some(U2::new(0).unwrap()));
        assert_eq!(a.saturating_add(b), U2::new(0b10).unwrap());
        assert_eq!(a.saturating_sub(b), U2::new(0).unwrap());
        assert_eq!(a.wrapping_add(b), U2::new(0b10).unwrap());
        assert_eq!(a.wrapping_sub(b), U2::new(0).unwrap());

        let a = U2::new(0b10).unwrap();
        let b = U2::new(1).unwrap();
        assert_eq!(a.checked_add(b), Some(U2::new(0b11).unwrap()));
        assert_eq!(a.checked_sub(b), Some(U2::new(0b01).unwrap()));
        assert_eq!(a.saturating_add(b), U2::new(0b11).unwrap());
        assert_eq!(a.saturating_sub(b), U2::new(0b01).unwrap());
        assert_eq!(a.wrapping_add(b), U2::new(0b11).unwrap());
        assert_eq!(a.wrapping_sub(b), U2::new(0b01).unwrap());

        let a = U2::new(0b11).unwrap();
        let b = U2::new(1).unwrap();
        assert_eq!(a.checked_add(b), None);
        assert_eq!(a.checked_sub(b), Some(U2::new(0b10).unwrap()));
        assert_eq!(a.saturating_add(b), U2::new(0b11).unwrap());
        assert_eq!(a.saturating_sub(b), U2::new(0b10).unwrap());
        assert_eq!(a.wrapping_add(b), U2::new(0).unwrap());
        assert_eq!(a.wrapping_sub(b), U2::new(0b10).unwrap());

        let a = U2::new(1).unwrap();
        let b = U2::new(0b11).unwrap();
        assert_eq!(a.checked_add(b), None);
        assert_eq!(a.checked_sub(b), None);
        assert_eq!(a.saturating_add(b), U2::new(0b11).unwrap());
        assert_eq!(a.saturating_sub(b), U2::new(0).unwrap());
        assert_eq!(a.wrapping_add(b), U2::new(0).unwrap());
        assert_eq!(a.wrapping_sub(b), U2::new(2).unwrap());
    }
}
