#[macro_export]
/// For naming a new enum, passing in what type it serializes to and deserializes
/// from, and providing a mapping from variants to expressions (such as libc consts) that
/// will ultimately be used in the serialization/deserialization step when sending the netlink
/// message over the wire.
///
/// # Usage
///  Create an `enum` named "MyNetlinkProtoAttrs" that can be serialized into `u16`s to use with Netlink.
///  Possibly represents the fields on a message you received from Netlink.
///  ```ignore
///  impl_var!(MyNetlinkProtoAttrs, u16,
///     Id => 16 as u16,
///     Name => 17 as u16,
///     Size => 18 as u16
///  );
/// ```
/// Or, with doc comments (if you're developing a library)
/// ```ignore
///  impl_var!(
///     /// These are the attributes returned
///     /// by a fake netlink protocol.
///     ( MyNetlinkProtoAttrs, u16,
///     Id => 16 as u16,
///     Name => 17 as u16,
///     Size => 18 as u16 )
///  );
/// ```
///
macro_rules! impl_var {
    (
        $( #[$outer:meta] )*
        $name:ident, $ty:ty, $( $( #[cfg($meta:meta)] )* $var:ident => $val:expr ),*
    ) => (
        $(#[$outer])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum $name {
            $(
                $(
                    #[cfg($meta)]
                )*
                #[allow(missing_docs)]
                $var,
            )*
            /// Variant that signifies an invalid value while deserializing
            UnrecognizedVariant($ty),
        }

        impl $name {
            /// Returns true if no variant corresponds to the value it was parsed from
            pub fn is_unrecognized(&self) -> bool {
                match *self {
                    $name::UnrecognizedVariant(_) => true,
                    _ => false,
                }
            }
        }

        impl From<$ty> for $name {
            fn from(v: $ty) -> Self {
                match v {
                    $(
                        $(
                            #[cfg($meta)]
                        )*
                        i if i == $val => $name::$var,
                    )*
                    i => $name::UnrecognizedVariant(i)
                }
            }
        }

        impl From<$name> for $ty {
            fn from(v: $name) -> Self {
                match v {
                    $(
                        $(
                            #[cfg($meta)]
                        )*
                        $name::$var => $val,
                    )*
                    $name::UnrecognizedVariant(i) => i,
                }
            }
        }

        impl<'a> From<&'a $name> for $ty {
            fn from(v: &'a $name) -> Self {
                match *v {
                    $(
                        $(
                            #[cfg($meta)]
                        )*
                        $name::$var => $val,
                    )*
                    $name::UnrecognizedVariant(i) => i,
                }
            }
        }

        impl $crate::Nl for $name {
            fn serialize(&self, mem: &mut $crate::StreamWriteBuffer) -> Result<(), $crate::err::SerError> {
                let v: $ty = self.clone().into();
                v.serialize(mem)
            }

            fn deserialize<T>(mem: &mut $crate::StreamReadBuffer<T>) -> Result<Self, $crate::err::DeError>
                    where T: AsRef<[u8]> {
                let v = <$ty>::deserialize(mem)?;
                Ok(v.into())
            }

            fn size(&self) -> usize {
                std::mem::size_of::<$ty>()
            }
        }
    );
}

#[macro_export]
/// For generating a marker trait that flags a new enum as usable in a field that accepts a generic
/// type.
/// This way, the type can be constrained when the impl is provided to only accept enums that
/// implement the marker trait that corresponds to the given marker trait. The current
/// convention is to use `impl_trait` to create the trait with the name of the field that
/// is the generic type and then use `impl_var_trait` to flag the new enum as usable in
/// this field. See the examples below for more details.
macro_rules! impl_trait {
    ( $(#[$outer:meta])* $trait_name:ident, $to_from_ty:ty, $(#[$wrapper_outer:meta])* $wrapper_type:ident, $( $const_enum:ident ),+ ) => {
        $(#[$outer])*
        pub trait $trait_name: $crate::Nl + PartialEq + From<$to_from_ty> + Into<$to_from_ty> {}

        impl $trait_name for $to_from_ty {}

        $(
            impl $trait_name for $const_enum {}
        )+

        #[derive(Debug,PartialEq)]
        $(
            #[$wrapper_outer]
        )*
        pub enum $wrapper_type {
            $(
                #[allow(missing_docs)]
                $const_enum($const_enum),
            )+
            /// Constant could not be parsed into a type
            UnrecognizedConst($to_from_ty),
        }

        impl $trait_name for $wrapper_type {}

        impl Into<$to_from_ty> for $wrapper_type {
            fn into(self) -> $to_from_ty {
                match self {
                    $(
                        $wrapper_type::$const_enum(inner) => inner.into(),
                    )+
                    $wrapper_type::UnrecognizedConst(v) => v,
                }
            }
        }

        impl From<$to_from_ty> for $wrapper_type {
            fn from(v: $to_from_ty) -> Self {
                $(
                    let var = $const_enum::from(v);
                    if !var.is_unrecognized() {
                        return $wrapper_type::$const_enum(var);
                    }
                )*
                $wrapper_type::UnrecognizedConst(v)
            }
        }

        impl $crate::Nl for $wrapper_type {
            fn serialize(&self, mem: &mut $crate::StreamWriteBuffer) -> Result<(), $crate::err::SerError> {
                match *self {
                    $(
                        $wrapper_type::$const_enum(ref inner) => inner.serialize(mem)?,
                    )+
                    $wrapper_type::UnrecognizedConst(v) => v.serialize(mem)?,
                };
                Ok(())
            }

            fn deserialize<B>(mem: &mut $crate::StreamReadBuffer<B>) -> Result<Self, $crate::err::DeError> where B: AsRef<[u8]> {
                let v = <$to_from_ty>::deserialize(mem)?;
                Ok($wrapper_type::from(v))
            }

            fn size(&self) -> usize {
                std::mem::size_of::<$to_from_ty>()
            }
        }
    };
}
