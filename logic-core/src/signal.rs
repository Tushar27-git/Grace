use serde::{Deserialize, Serialize};
use std::ops::{BitAnd, BitOr, BitXor, Not};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Signal {
    #[default]
    Zero,
    One,
    X,
    Z,
}

impl Signal {
    pub fn is_high(self) -> bool {
        matches!(self, Signal::One)
    }

    pub fn nand(self, other: Self) -> Self {
        !(self & other)
    }

    pub fn nor(self, other: Self) -> Self {
        !(self | other)
    }

    pub fn xnor(self, other: Self) -> Self {
        !(self ^ other)
    }
}

impl Not for Signal {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Signal::Zero => Signal::One,
            Signal::One => Signal::Zero,
            Signal::X | Signal::Z => Signal::X,
        }
    }
}

impl BitAnd for Signal {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Signal::Zero, _) | (_, Signal::Zero) => Signal::Zero,
            (Signal::One, Signal::One) => Signal::One,
            _ => Signal::X,
        }
    }
}

impl BitOr for Signal {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Signal::One, _) | (_, Signal::One) => Signal::One,
            (Signal::Zero, Signal::Zero) => Signal::Zero,
            _ => Signal::X,
        }
    }
}

impl BitXor for Signal {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Signal::Zero, Signal::Zero) | (Signal::One, Signal::One) => Signal::Zero,
            (Signal::Zero, Signal::One) | (Signal::One, Signal::Zero) => Signal::One,
            _ => Signal::X,
        }
    }
}
