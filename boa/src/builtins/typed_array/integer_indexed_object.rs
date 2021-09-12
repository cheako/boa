use super::TypedArrayName;
use crate::{
    gc::{empty_trace, Finalize, Trace},
    object::{JsObject, ObjectData},
    Context, JsResult,
};

/// Type of the array content.
#[derive(Debug, Clone, Copy, Finalize)]
pub(super) enum ContentType {
    Number,
    BigInt,
}

unsafe impl Trace for ContentType {
    // safe beacuase `ContentType` is `Copy`
    empty_trace!();
}

/// <https://tc39.es/ecma262/#integer-indexed-exotic-object>
#[derive(Debug, Default, Clone, Trace, Finalize)]
pub struct IntegerIndexedObject {
    pub(super) viewed_array_buffer: Option<JsObject>,
    pub(super) typed_array_name: Option<TypedArrayName>,
    pub(super) content_type: Option<ContentType>,
    pub(super) byte_offset: usize,
    pub(super) byte_length: usize,
    pub(super) array_length: usize,
}

impl IntegerIndexedObject {
    /// `IntegerIndexedObjectCreate ( prototype )`
    ///
    /// <https://tc39.es/ecma262/#sec-integerindexedobjectcreate>
    pub(super) fn create(prototype: JsObject, context: &Context) -> JsObject {
        // 1. Let internalSlotsList be « [[Prototype]], [[Extensible]], [[ViewedArrayBuffer]], [[TypedArrayName]], [[ContentType]], [[ByteLength]], [[ByteOffset]], [[ArrayLength]] ».
        let internal_slots_list = Self::default();

        // 2. Let A be ! MakeBasicObject(internalSlotsList).
        let a = context.construct_object();

        // 3. Set A.[[GetOwnProperty]] as specified in 10.4.5.1.
        // 4. Set A.[[HasProperty]] as specified in 10.4.5.2.
        // 5. Set A.[[DefineOwnProperty]] as specified in 10.4.5.3.
        // 6. Set A.[[Get]] as specified in 10.4.5.4.
        // 7. Set A.[[Set]] as specified in 10.4.5.5.
        // 8. Set A.[[Delete]] as specified in 10.4.5.6.
        // 9. Set A.[[OwnPropertyKeys]] as specified in 10.4.5.7.
        a.borrow_mut().data = ObjectData::typed_array(internal_slots_list);

        // 10. Set A.[[Prototype]] to prototype.
        a.set_prototype_instance(prototype.into());

        // 11. Return A.
        a
    }

    // /// <https://tc39.es/ecma262/#sec-integerindexedobjectcreate>
    // pub(super) fn new(constructor_name: TypedArrayName, length: Option<usize>) -> JsResult<Self> {
    //     let content_type = match constructor_name {
    //         // 5. If constructorName is "BigInt64Array" or "BigUint64Array", set obj.[[ContentType]] to BigInt.
    //         TypedArrayName::BigInt64Array | TypedArrayName::BigUint64Array => ContentType::BigInt,
    //         // 6. Otherwise, set obj.[[ContentType]] to Number.
    //         _ => ContentType::Number,
    //     };

    //     if let Some(length) = length {
    //         Self::allocate_typed_array_buffer(constructor_name, content_type, length)
    //     } else {
    //         Ok(Self {
    //             viewed_array_buffer: Default::default(),
    //             typed_array_name: constructor_name,
    //             content_type,
    //             // a. Set obj.[[ByteLength]] to 0.
    //             byte_length: 0,
    //             // b. Set obj.[[ByteOffset]] to 0.
    //             byte_offset: 0,
    //             // c. Set obj.[[ArrayLength]] to 0.
    //             array_length: 0,
    //         })
    //     }
    // }
}

/// A Data Block
///
/// The Data Block specification type is used to describe a distinct and mutable sequence of
/// byte-sized (8 bit) numeric values. A byte value is an integer value in the range `0` through
/// `255`, inclusive. A Data Block value is created with a fixed number of bytes that each have
/// the initial value `0`.
///
/// For more information, check the [spec][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-data-blocks
#[derive(Debug, Clone, Default, Trace, Finalize)]
pub struct DataBlock {
    inner: Vec<u8>,
}

impl DataBlock {
    /// `CreateByteDataBlock ( size )` abstract operation.
    ///
    /// The abstract operation `CreateByteDataBlock` takes argument `size` (a non-negative
    /// integer). For more information, check the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createbytedatablock
    pub fn create_byte_data_block(size: usize) -> JsResult<Self> {
        // 1. Let db be a new Data Block value consisting of size bytes. If it is impossible to
        //    create such a Data Block, throw a RangeError exception.
        // 2. Set all of the bytes of db to 0.
        // 3. Return db.
        // TODO: waiting on <https://github.com/rust-lang/rust/issues/48043> for having fallible
        // allocation.
        Ok(Self {
            inner: vec![0u8; size],
        })
    }
}
