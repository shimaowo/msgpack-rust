//! Change MessagePack behavior with configuration wrappers.
use rmp::encode;
use serde::{Serialize, Serializer};

use crate::encode::{Error, UnderlyingWrite};

/// Represents configuration that dicatates what the serializer does.
///
/// Implemented as an empty trait depending on a hidden trait in order to allow changing the
/// methods of this trait without breaking backwards compatibility.
pub trait SerializerConfig: sealed::SerializerConfig {}

impl<T: sealed::SerializerConfig> SerializerConfig for T {}

mod sealed {
    use serde::{Serialize, Serializer};

    use crate::encode::{Error, UnderlyingWrite};

    /// This is the inner trait - the real SerializerConfig.
    ///
    /// This hack disallows external implementations and usage of SerializerConfig and thus
    /// allows us to change SerializerConfig methods freely without breaking backwards compatibility.
    pub trait SerializerConfig: Copy {
        fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
        where
            S: UnderlyingWrite,
            for<'a> &'a mut S: Serializer<Ok = (), Error = Error>;

        fn write_struct_field<S, T>(ser: &mut S, key: &'static str, value: &T) -> Result<(), Error>
        where
            S: UnderlyingWrite,
            for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
            T: ?Sized + Serialize;

        /// Encodes an enum variant ident (id or name) according to underlying writer.
        ///
        /// Used in `Serializer::serialize_*_variant` methods.
        fn write_variant_ident<S>(
            ser: &mut S,
            variant_index: u32,
            variant: &'static str,
        ) -> Result<(), Error>
        where
            S: UnderlyingWrite,
            for<'a> &'a mut S: Serializer<Ok = (), Error = Error>;

        /// Determines the value of `Serializer::is_human_readable` and
        /// `Deserializer::is_human_readable`.
        fn is_human_readable() -> bool;
    }
}

/// The default serializer/deserializer configuration.
///
/// This configuration:
/// - Writes structs as a tuple, without field names
/// - Writes enum variants as integers
/// - Writes and reads types as binary, not human-readable
//
/// This is the most compact representation.
#[derive(Copy, Clone, Debug)]
pub
struct DefaultConfig;

impl sealed::SerializerConfig for DefaultConfig {
    fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        encode::write_array_len(ser.get_mut(), len as u32)?;

        Ok(())
    }

    #[inline]
    fn write_struct_field<S, T>(ser: &mut S, _key: &'static str, value: &T) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
        T: ?Sized + Serialize,
    {
        value.serialize(ser)
    }

    #[inline]
    fn write_variant_ident<S>(
        ser: &mut S,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        // smw note - this is a bug, and/or different from the docs in multiple places
        // but I'm leaving it untouched and instead introducing new configs and methods
        // on Serializer to customize the behavior
        ser.serialize_str(variant)
    }

    #[inline(always)]
    fn is_human_readable() -> bool {
        false
    }
}

/// Config wrapper, that overrides struct serialization by packing as a map with field names.
///
/// MessagePack specification does not tell how to serialize structs. This trait allows you to
/// extend serialization to match your app's requirements.
///
/// Default `Serializer` implementation writes structs as a tuple, i.e. only its length is encoded,
/// because it is the most compact representation.
#[derive(Copy, Clone, Debug)]
pub struct StructMapConfig<C>(C);

impl<C> StructMapConfig<C> {
    /// Creates a `StructMapConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        StructMapConfig(inner)
    }
}

impl<C> sealed::SerializerConfig for StructMapConfig<C>
where
    C: sealed::SerializerConfig,
{
    fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        encode::write_map_len(ser.get_mut(), len as u32)?;

        Ok(())
    }

    fn write_struct_field<S, T>(ser: &mut S, key: &'static str, value: &T) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
        T: ?Sized + Serialize,
    {
        encode::write_str(ser.get_mut(), key)?;
        value.serialize(ser)
    }

    #[inline]
    fn write_variant_ident<S>(
        ser: &mut S,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        C::write_variant_ident(ser, variant_index, variant)
    }

    #[inline(always)]
    fn is_human_readable() -> bool {
        C::is_human_readable()
    }
}

/// Config wrapper that overrides struct serlization by packing as a tuple without field
/// names.
#[derive(Copy, Clone, Debug)]
pub struct StructTupleConfig<C>(C);

impl<C> StructTupleConfig<C> {
    /// Creates a `StructTupleConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        StructTupleConfig(inner)
    }
}

impl<C> sealed::SerializerConfig for StructTupleConfig<C>
where
    C: sealed::SerializerConfig,
{
    fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        encode::write_array_len(ser.get_mut(), len as u32)?;

        Ok(())
    }

    #[inline]
    fn write_struct_field<S, T>(ser: &mut S, _key: &'static str, value: &T) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
        T: ?Sized + Serialize,
    {
        value.serialize(ser)
    }

    #[inline]
    fn write_variant_ident<S>(
        ser: &mut S,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        C::write_variant_ident(ser, variant_index, variant)
    }

    #[inline(always)]
    fn is_human_readable() -> bool {
        C::is_human_readable()
    }
}

/// Config wrapper that overrides `Serializer::is_human_readable` and
/// `Deserializer::is_human_readable` to return `true`.
#[derive(Copy, Clone, Debug)]
pub struct HumanReadableConfig<C>(C);

impl<C> HumanReadableConfig<C> {
    /// Creates a `HumanReadableConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        Self(inner)
    }
}

impl<C> sealed::SerializerConfig for HumanReadableConfig<C>
where
    C: sealed::SerializerConfig,
{
    #[inline]
    fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        C::write_struct_len(ser, len)
    }

    #[inline]
    fn write_struct_field<S, T>(ser: &mut S, key: &'static str, value: &T) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
        T: ?Sized + Serialize,
    {
        C::write_struct_field(ser, key, value)
    }

    #[inline]
    fn write_variant_ident<S>(
        ser: &mut S,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        C::write_variant_ident(ser, variant_index, variant)
    }

    #[inline(always)]
    fn is_human_readable() -> bool {
        true
    }
}

/// Config wrapper that overrides `Serializer::is_human_readable` and
/// `Deserializer::is_human_readable` to return `false`.
#[derive(Copy, Clone, Debug)]
pub struct BinaryConfig<C>(C);

impl<C> BinaryConfig<C> {
    /// Creates a `BinaryConfig` inheriting unchanged configuration options from the given configuration.
    #[inline(always)]
    pub fn new(inner: C) -> Self {
        Self(inner)
    }
}

impl<C> sealed::SerializerConfig for BinaryConfig<C>
where
    C: sealed::SerializerConfig,
{
    #[inline]
    fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        C::write_struct_len(ser, len)
    }

    #[inline]
    fn write_struct_field<S, T>(ser: &mut S, key: &'static str, value: &T) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
        T: ?Sized + Serialize,
    {
        C::write_struct_field(ser, key, value)
    }

    #[inline]
    fn write_variant_ident<S>(
        ser: &mut S,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
    {
        C::write_variant_ident(ser, variant_index, variant)
    }

    #[inline(always)]
    fn is_human_readable() -> bool {
        false
    }
}

// smw edits

/// Config wrapper that overrides enum serialization by writing the enum variant index instead of the name.
#[derive(Copy, Clone, Debug)]
pub struct IntegerEnumConfig<C>(C);

impl<C> IntegerEnumConfig<C> {
    /// Creates a `IntegerEnumConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        Self(inner)
    }
}

impl<C> sealed::SerializerConfig for IntegerEnumConfig<C>
where
    C: sealed::SerializerConfig,
{
    fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error> {
            C::write_struct_len(ser, len)
    }

    fn write_struct_field<S, T>(ser: &mut S, key: &'static str, value: &T) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
        T: ?Sized + Serialize {
            C::write_struct_field(ser, key, value)
    }

    fn write_variant_ident<S>(
        ser: &mut S,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error> {
            // TODO: unsure if we should use encode::write* here to get a marker
            // or if the bare number is sufficient
            ser.serialize_u32(variant_index)
    }

    fn is_human_readable() -> bool {
        C::is_human_readable()
    }
}

/// Config wrapper that overrides enum serialization by writing the enum variant name instead of the index.
#[derive(Copy, Clone, Debug)]
pub struct StringEnumConfig<C>(C);

impl<C> StringEnumConfig<C> {
    /// Creates a `StringEnumConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        Self(inner)
    }
}

impl<C> sealed::SerializerConfig for StringEnumConfig<C>
where
    C: sealed::SerializerConfig,
{
    fn write_struct_len<S>(ser: &mut S, len: usize) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error> {
            C::write_struct_len(ser, len)
    }

    fn write_struct_field<S, T>(ser: &mut S, key: &'static str, value: &T) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error>,
        T: ?Sized + Serialize {
            C::write_struct_field(ser, key, value)
    }

    fn write_variant_ident<S>(
        ser: &mut S,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error>
    where
        S: UnderlyingWrite,
        for<'a> &'a mut S: Serializer<Ok = (), Error = Error> {
            ser.serialize_str(variant)
    }

    fn is_human_readable() -> bool {
        C::is_human_readable()
    }
}

// smw end edits
