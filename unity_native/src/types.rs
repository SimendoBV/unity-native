use mint::IntoMint;

macro_rules! mint_conversion {
    ($name:ty) => {
        impl $name {
            #[inline]
            pub fn from_mint(
                val: impl IntoMint<MintType = <$name as mint::IntoMint>::MintType>,
            ) -> Self {
                Self::from(val.into())
            }

            #[inline]
            pub fn to_mint(self) -> <$name as mint::IntoMint>::MintType {
                <$name as mint::IntoMint>::MintType::from(self)
            }
        }
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Column-major
/// First digit is the row index, second digit is the column index
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    pub m00: f32,
    pub m10: f32,
    pub m20: f32,
    pub m30: f32,

    pub m01: f32,
    pub m11: f32,
    pub m21: f32,
    pub m31: f32,

    pub m02: f32,
    pub m12: f32,
    pub m22: f32,
    pub m32: f32,

    pub m03: f32,
    pub m13: f32,
    pub m23: f32,
    pub m33: f32,
}

impl From<Matrix4x4> for mint::ColumnMatrix4<f32> {
    fn from(value: Matrix4x4) -> Self {
        Self {
            x: mint::Vector4 {
                x: value.m00,
                y: value.m10,
                z: value.m20,
                w: value.m30,
            },
            y: mint::Vector4 {
                x: value.m01,
                y: value.m11,
                z: value.m21,
                w: value.m31,
            },
            z: mint::Vector4 {
                x: value.m02,
                y: value.m12,
                z: value.m22,
                w: value.m32,
            },
            w: mint::Vector4 {
                x: value.m03,
                y: value.m13,
                z: value.m23,
                w: value.m33,
            },
        }
    }
}

impl IntoMint for Matrix4x4 {
    type MintType = mint::ColumnMatrix4<f32>;
}

impl From<<Matrix4x4 as IntoMint>::MintType> for Matrix4x4 {
    fn from(value: <Matrix4x4 as IntoMint>::MintType) -> Self {
        Self {
            m00: value.x.x,
            m10: value.x.y,
            m20: value.x.z,
            m30: value.x.w,

            m01: value.y.x,
            m11: value.y.y,
            m21: value.y.z,
            m31: value.y.w,

            m02: value.z.x,
            m12: value.z.y,
            m22: value.z.z,
            m32: value.z.w,

            m03: value.w.x,
            m13: value.w.y,
            m23: value.w.z,
            m33: value.w.w,
        }
    }
}

mint_conversion!(Matrix4x4);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<Quaternion> for mint::Quaternion<f32> {
    fn from(value: Quaternion) -> Self {
        Self {
            v: mint::Vector3 {
                x: value.x,
                y: value.y,
                z: value.z,
            },
            s: value.w,
        }
    }
}

impl IntoMint for Quaternion {
    type MintType = mint::Quaternion<f32>;
}

impl From<<Quaternion as IntoMint>::MintType> for Quaternion {
    fn from(value: <Quaternion as IntoMint>::MintType) -> Self {
        Self {
            x: value.v.x,
            y: value.v.y,
            z: value.v.z,
            w: value.s,
        }
    }
}

mint_conversion!(Quaternion);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl From<Vector2> for mint::Vector2<f32> {
    fn from(value: Vector2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl IntoMint for Vector2 {
    type MintType = mint::Vector2<f32>;
}

impl From<<Vector2 as IntoMint>::MintType> for Vector2 {
    fn from(value: <Vector2 as IntoMint>::MintType) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

mint_conversion!(Vector2);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector2Int {
    pub x: i32,
    pub y: i32,
}

impl From<Vector2Int> for mint::Vector2<i32> {
    fn from(value: Vector2Int) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl IntoMint for Vector2Int {
    type MintType = mint::Vector2<i32>;
}

impl From<<Vector2Int as IntoMint>::MintType> for Vector2Int {
    fn from(value: <Vector2Int as IntoMint>::MintType) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

mint_conversion!(Vector2Int);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vector3> for mint::Vector3<f32> {
    fn from(value: Vector3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl IntoMint for Vector3 {
    type MintType = mint::Vector3<f32>;
}

impl From<<Vector3 as IntoMint>::MintType> for Vector3 {
    fn from(value: <Vector3 as IntoMint>::MintType) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

mint_conversion!(Vector3);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector3Int {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<Vector3Int> for mint::Vector3<i32> {
    fn from(value: Vector3Int) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl IntoMint for Vector3Int {
    type MintType = mint::Vector3<i32>;
}

impl From<<Vector3Int as IntoMint>::MintType> for Vector3Int {
    fn from(value: <Vector3Int as IntoMint>::MintType) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

mint_conversion!(Vector3Int);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<Vector4> for mint::Vector4<f32> {
    fn from(value: Vector4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl IntoMint for Vector4 {
    type MintType = mint::Vector4<f32>;
}

impl From<<Vector4 as IntoMint>::MintType> for Vector4 {
    fn from(value: <Vector4 as IntoMint>::MintType) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

mint_conversion!(Vector4);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub normal: Vector3,
    pub distance: f32,
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn mint_Matrix4x4() {
        let mat = Matrix4x4 {
            m00: 0.0,
            m10: 1.0,
            m20: 2.0,
            m30: 3.0,
            m01: 4.0,
            m11: 5.0,
            m21: 6.0,
            m31: 7.0,
            m02: 8.0,
            m12: 9.0,
            m22: 10.0,
            m32: 11.0,
            m03: 12.0,
            m13: 13.0,
            m23: 14.0,
            m33: 15.0,
        };

        assert_eq!(
            mat,
            Matrix4x4::from(<Matrix4x4 as IntoMint>::MintType::from(mat))
        );
    }

    #[test]
    fn mint_Quaternion() {
        let quat = Quaternion {
            x: 0.0,
            y: 1.0,
            z: 2.0,
            w: 3.0,
        };

        assert_eq!(
            quat,
            Quaternion::from(<Quaternion as IntoMint>::MintType::from(quat))
        );
    }

    #[test]
    fn mint_Vector2() {
        let vec = Vector2 { x: 0.0, y: 1.0 };

        assert_eq!(
            vec,
            Vector2::from(<Vector2 as IntoMint>::MintType::from(vec))
        );
    }

    #[test]
    fn mint_Vector2Int() {
        let vec = Vector2Int { x: 0, y: 1 };

        assert_eq!(
            vec,
            Vector2Int::from(<Vector2Int as IntoMint>::MintType::from(vec))
        );
    }

    #[test]
    fn mint_Vector3() {
        let vec = Vector3 {
            x: 0.0,
            y: 1.0,
            z: 2.0,
        };

        assert_eq!(
            vec,
            Vector3::from(<Vector3 as IntoMint>::MintType::from(vec))
        );
    }

    #[test]
    fn mint_Vector3Int() {
        let vec = Vector3Int { x: 0, y: 1, z: 2 };

        assert_eq!(
            vec,
            Vector3Int::from(<Vector3Int as IntoMint>::MintType::from(vec))
        );
    }

    #[test]
    fn mint_Vector4() {
        let vec = Vector4 {
            x: 0.0,
            y: 1.0,
            z: 2.0,
            w: 3.0,
        };

        assert_eq!(
            vec,
            Vector4::from(<Vector4 as IntoMint>::MintType::from(vec))
        );
    }
}
