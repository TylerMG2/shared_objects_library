use serde::{de::DeserializeOwned, Serialize};

pub trait Networked {
    type Optional: Serialize + DeserializeOwned + Copy;

    fn update_from_optional(&mut self, optional: Option<Self::Optional>);

    // Returns an option to indicate if there are any differences between the two structs
    // If there are differences it should return Some with the differences, if there are no differences it should return None
    fn differences_with(&self, other: &Self) -> Option<Self::Optional>;

    // Convert this into an optional
    fn into_optional(&self) -> Option<Self::Optional>;

    // Convert an optional into this
    fn from_optional(optional: Self::Optional) -> Self;
}

impl<T, const N: usize> Networked for [T; N]
where
    T: Networked + Serialize + DeserializeOwned + Copy + Default,
    [Option<T::Optional>; N]: Serialize + DeserializeOwned
{
    type Optional = [Option<T::Optional>; N];

    fn update_from_optional(&mut self, optional: Option<Self::Optional>) {
        if let Some(optional) = optional {
            for (this, optional) in self.iter_mut().zip(optional.iter()) {
                this.update_from_optional(*optional);
            }
        }
    }

    fn differences_with(&self, other: &Self) -> Option<Self::Optional> {
        let mut optional: Option<Self::Optional> = None;
    
        for (index, (x, y)) in self.iter().zip(other.iter()).enumerate() {
            if let Some(diff) = x.differences_with(y) {
                optional.get_or_insert_with(|| [None; N])[index] = Some(diff);
            }
        }
    
        optional
    }

    fn into_optional(&self) -> Option<Self::Optional> {
        let mut optional = [None; N];
        for (index, this) in self.iter().enumerate() {
            optional[index] = this.into_optional();
        }
        Some(optional)
    }

    // This is the reason the array type must implement default
    fn from_optional(optional: Self::Optional) -> Self {
        let mut this = [T::default(); N];
        for (index, optional) in optional.iter().enumerate() {
            if let Some(optional) = optional {
                this[index] = T::from_optional(*optional);
            }
        }
        this
    }
}

impl<T: Networked + Copy> Networked for Option<T> {
    type Optional = Option<T::Optional>;

    fn from_optional(optional: Self::Optional) -> Self {
        optional.map(T::from_optional)
    }

    fn update_from_optional(&mut self, optional: Option<Self::Optional>) {
        if let Some(optional) = optional {
            match (self.as_mut(), optional) {
                (Some(this), Some(other)) => this.update_from_optional(Some(other)),
                (Some(_), None) => *self = None,
                (None, Some(other)) => *self = Some(T::from_optional(other)),
                (None, None) => {},
            }
        }
    }

    fn differences_with(&self, other: &Self) -> Option<Self::Optional> {
        match (self, other) {
            (Some(this), Some(other)) => {
                let diff = this.differences_with(other);

                // If there is a difference return it, otherwise return None
                if let Some(diff) = diff {
                    Some(Some(diff))
                } else {
                    None
                }
            },
            (None, Some(other)) => Some(other.into_optional()),
            (Some(_), None) => Some(None),
            (None, None) => None,
        }
    }

    fn into_optional(&self) -> Option<Self::Optional> {
        match self {
            Some(this) => Some(this.into_optional()),
            None => None,
        }
    }
}

macro_rules! impl_networked {
    ($($t:ty),*) => {
        $(
            impl Networked for $t {
                type Optional = $t;

                fn from_optional(optional: Self::Optional) -> Self {
                    optional
                }

                fn update_from_optional(&mut self, optional: Option<Self::Optional>) {
                    if let Some(optional) = optional {
                        *self = optional;
                    }
                }

                fn differences_with(&self, other: &Self) -> Option<Self::Optional> {
                    if self != other {
                        Some(*other)
                    } else {
                        None
                    }
                }

                fn into_optional(&self) -> Option<Self::Optional> {
                    Some(*self)
                }
            }
        )*
    };
}

impl_networked!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool);