//! <https://tc39.es/ecma262/#sec-typedarray-objects>

pub use self::integer_indexed_object::IntegerIndexedObject;
use crate::{
    builtins::{
        array_buffer::ArrayBuffer, iterable::iterable_to_list,
        typed_array::integer_indexed_object::ContentType, Array, BuiltIn, JsArgs,
    },
    context::{StandardConstructor, StandardObjects},
    gc::{empty_trace, Finalize, Trace},
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject,
    },
    property::Attribute,
    symbol::WellKnownSymbols,
    value::JsValue,
    BoaProfiler, Context, JsResult,
};

pub mod integer_indexed_object;

macro_rules! typed_array {
    ($ty:ident, $name:literal) => {
        #[doc = concat!("JavaScript `", $name, "` built-in implementation.")]
        #[derive(Debug, Clone, Copy)]
        pub struct $ty;

        impl BuiltIn for $ty {
            const NAME: &'static str = $name;

            fn attribute() -> Attribute {
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
            }

            fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
                let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

                let typed_array = ConstructorBuilder::with_standard_object(
                    context,
                    Self::constructor,
                    context.standard_objects().typed_array_object().clone(),
                )
                .name(Self::NAME)
                .length(Self::LENGTH)
                .property(
                    "BYTES_PER_ELEMENT",
                    TypedArrayName::$ty.element_size(),
                    Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
                )
                .build();

                (Self::NAME, typed_array.into(), Self::attribute())
            }
        }

        impl $ty {
            const LENGTH: usize = 3;

            /// <https://tc39.es/ecma262/#sec-typedarray>
            fn constructor(
                new_target: &JsValue,
                args: &[JsValue],
                context: &mut Context,
            ) -> JsResult<JsValue> {
                // 1. If NewTarget is undefined, throw a TypeError exception.
                if new_target.is_undefined() {
                    return context.throw_type_error(concat!(
                        "new target was undefined when constructing an ",
                        $name
                    ));
                }

                // 2. Let constructorName be the String value of the Constructor Name value specified in Table 72 for this TypedArray constructor.
                let constructor_name = TypedArrayName::$ty;

                // 3. Let proto be "%TypedArray.prototype%".
                let proto = StandardObjects::typed_array_object;

                // 4. Let numberOfArgs be the number of elements in args.
                let number_of_args = args.len();

                // 5. If numberOfArgs = 0, then
                if number_of_args == 0 {
                    // a. Return ? AllocateTypedArray(constructorName, NewTarget, proto, 0).
                    return TypedArray::allocate(
                        constructor_name,
                        new_target,
                        proto,
                        Some(0),
                        context,
                    );
                }
                // 6. Else,

                // a. Let firstArgument be args[0].
                let first_argument = &args[0];

                // b. If Type(firstArgument) is Object, then
                if let Some(first_argument) = first_argument.as_object() {
                    // i. Let O be ? AllocateTypedArray(constructorName, NewTarget, proto).
                    let o =
                        TypedArray::allocate(constructor_name, new_target, proto, None, context)?;

                    // ii. If firstArgument has a [[TypedArrayName]] internal slot, then
                    if first_argument.is_typed_array() {
                        // 1. Perform ? InitializeTypedArrayFromTypedArray(O, firstArgument).
                        TypedArray::initialize_from_typed_array(
                            o.clone(),
                            first_argument,
                            context,
                        )?;
                    } else if first_argument.is_array_buffer() {
                        // iii. Else if firstArgument has an [[ArrayBufferData]] internal slot, then

                        // 1. If numberOfArgs > 1, let byteOffset be args[1]; else let byteOffset be undefined.
                        let byte_offset = args.get_or_undefined(1);

                        // 2. If numberOfArgs > 2, let length be args[2]; else let length be undefined.
                        let length = args.get_or_undefined(2);

                        // 3. Perform ? InitializeTypedArrayFromArrayBuffer(O, firstArgument, byteOffset, length).
                        TypedArray::initialize_from_array_buffer(
                            o.clone(),
                            first_argument,
                            byte_offset,
                            length,
                            context,
                        )?;
                    } else {
                        // iv. Else,

                        // 1. Assert: Type(firstArgument) is Object and firstArgument does not have
                        // either a [[TypedArrayName]] or an [[ArrayBufferData]] internal slot.

                        // 2. Let usingIterator be ? GetMethod(firstArgument, @@iterator).
                        let first_argument = JsValue::from(first_argument);
                        let using_iterator =
                            first_argument.get_method(context, WellKnownSymbols::replace())?;

                        // 3. If usingIterator is not undefined, then
                        if !using_iterator.is_undefined() {
                            // a. Let values be ? IterableToList(firstArgument, usingIterator).
                            let values =
                                iterable_to_list(context, first_argument, Some(using_iterator))?;

                            // b. Perform ? InitializeTypedArrayFromList(O, values).
                            TypedArray::initialize_from_list(o.clone(), values, context)?;
                        } else {
                            // 4. Else,

                            // a. NOTE: firstArgument is not an Iterable so assume it is already an array-like object.
                            // b. Perform ? InitializeTypedArrayFromArrayLike(O, firstArgument).
                            TypedArray::initialize_from_array_like(
                                o.clone(),
                                first_argument,
                                context,
                            )?;
                        }
                    }

                    // v. Return O.
                    Ok(o)
                } else {
                    // c. Else,

                    // i. Assert: firstArgument is not an Object.
                    assert!(!first_argument.is_object(), "firstArgument was an object");

                    // ii. Let elementLength be ? ToIndex(firstArgument).
                    let element_length = first_argument.to_index(context)?;

                    // iii. Return ? AllocateTypedArray(constructorName, NewTarget, proto, elementLength).
                    TypedArray::allocate(
                        constructor_name,
                        new_target,
                        proto,
                        Some(element_length),
                        context,
                    )
                }
            }
        }
    };
}

/// The JavaScript `%TypedArray%` object.
///
/// <https://tc39.es/ecma262/#sec-%typedarray%-intrinsic-object>
#[derive(Debug, Clone, Copy)]
struct TypedArray;

impl TypedArray {
    const NAME: &'static str = "TypedArray";
    const LENGTH: usize = 0;

    fn init(context: &mut Context) -> JsObject {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructable(false)
            .build();

        ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().function_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
        .property(
            "length",
            0,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        )
        // 23.2.3.29 %TypedArray%.prototype.toString ( )
        // The initial value of the %TypedArray%.prototype.toString data property is the same
        // built-in function object as the Array.prototype.toString method defined in 23.1.3.30.
        .method(Array::to_string, "toString", 0)
        .build()
    }

    /// <https://tc39.es/ecma262/#sec-%typedarray%>
    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Throw a TypeError exception.
        context.throw_type_error("the TypedArray constructor should never be called directly")
    }

    /// `get %TypedArray% [ @@species ]`
    ///
    /// The `%TypedArray% [ @@species ]` accessor property returns the constructor of a typed array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray/@@species
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// <https://tc39.es/ecma262/#sec-allocatetypedarraybuffer>
    fn allocate_buffer(o: JsValue, length: usize, context: &Context) -> JsResult<JsValue> {
        {
            let o = o.as_object().expect("expected an object");
            let mut o = o.borrow_mut();
            let o_inner = o.as_mut_typed_array().expect("expected a TypedArray");

            // 1. Assert: O.[[ViewedArrayBuffer]] is undefined.
            assert!(o_inner.viewed_array_buffer.is_none());

            // 2. Let constructorName be the String value of O.[[TypedArrayName]].
            // 3. Let elementSize be the Element Size value specified in Table 73 for constructorName.
            let element_size = o_inner
                .typed_array_name
                .expect("typed array name not found")
                .element_size();

            // 4. Let byteLength be elementSize × length.
            let byte_length = element_size * length;

            // 5. Let data be ? AllocateArrayBuffer(%ArrayBuffer%, byteLength).
            let data =
                ArrayBuffer::allocate(StandardObjects::array_buffer_object, byte_length, context)?;

            // 6. Set O.[[ViewedArrayBuffer]] to data.
            o_inner.viewed_array_buffer = Some(data);

            // 7. Set O.[[ByteLength]] to byteLength.
            o_inner.byte_length = byte_length;
            // 8. Set O.[[ByteOffset]] to 0.
            o_inner.byte_offset = 0;
            // 9. Set O.[[ArrayLength]] to length.
            o_inner.array_length = length;
        }

        // 10. Return O.
        Ok(o)
    }

    /// <https://tc39.es/ecma262/#sec-initializetypedarrayfromlist>
    fn initialize_from_list(
        o: JsValue,
        values: Vec<JsValue>,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let len be the number of elements in values.
        let len = values.len();

        // 2. Perform ? AllocateTypedArrayBuffer(O, len).
        TypedArray::allocate_buffer(o, len, context)?;

        // 3. Let k be 0.
        // 4. Repeat, while k < len,
        for (k, k_value) in values.into_iter().enumerate() {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be the first element of values and remove that element from values.
            // c. Perform ? Set(O, Pk, kValue, true).
            todo!("Set");
            // d. Set k to k + 1.
        }

        // 5. Assert: values is now an empty List.
        // It no longer exists.
        Ok(())
    }

    /// `AllocateTypedArray ( constructorName, newTarget, defaultProto [ , length ] )`
    ///
    /// It is used to validate and create an instance of a `TypedArray` constructor. If the `length`
    /// argument is passed, an `ArrayBuffer` of that length is also allocated and associated with the
    /// new `TypedArray` instance. `AllocateTypedArray` provides common semantics that is used by
    /// `TypedArray`.
    ///
    /// For more information, check the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-allocatetypedarray
    fn allocate<P>(
        constructor_name: TypedArrayName,
        new_target: &JsValue,
        default_proto: P,
        length: Option<usize>,
        context: &mut Context,
    ) -> JsResult<JsValue>
    where
        P: FnOnce(&StandardObjects) -> &StandardConstructor,
    {
        // 1. Let proto be ? GetPrototypeFromConstructor(newTarget, defaultProto).
        let proto = get_prototype_from_constructor(new_target, default_proto, context)?;

        // 2. Let obj be ! IntegerIndexedObjectCreate(proto).
        let obj = IntegerIndexedObject::create(proto, context);

        {
            let mut obj = obj.borrow_mut();
            let obj_inner = obj.as_mut_typed_array().expect("obj was not a typed array");

            // 3. Assert: obj.[[ViewedArrayBuffer]] is undefined.
            assert!(obj_inner.viewed_array_buffer.is_none());

            // 4. Set obj.[[TypedArrayName]] to constructorName.
            obj_inner.typed_array_name = Some(constructor_name);

            obj_inner.content_type = Some(match constructor_name {
                // 5. If constructorName is "BigInt64Array" or "BigUint64Array", set obj.[[ContentType]] to BigInt.
                TypedArrayName::BigInt64Array | TypedArrayName::BigUint64Array => {
                    ContentType::BigInt
                }
                // 6. Otherwise, set obj.[[ContentType]] to Number.
                _ => ContentType::Number,
            });

            if length.is_none() {
                // 7. If length is not present, then

                // a. Set obj.[[ByteLength]] to 0.
                obj_inner.byte_length = 0;
                // b. Set obj.[[ByteOffset]] to 0.
                obj_inner.byte_offset = 0;
                // c. Set obj.[[ArrayLength]] to 0.
                obj_inner.array_length = 0;
            }
        }

        if let Some(length) = length {
            // 8. Else,
            // a. Perform ? AllocateTypedArrayBuffer(obj, length).
            TypedArray::allocate_buffer(obj.clone().into(), length, context)?;
        }

        // 9. Return obj.
        Ok(obj.into())
    }

    /// InitializeTypedArrayFromTypedArray
    ///
    /// <https://tc39.es/ecma262/#sec-initializetypedarrayfromtypedarray>
    fn initialize_from_typed_array(
        o: JsValue,
        src_array: JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        todo!("InitializeTypedArrayFromTypedArray");
    }

    /// InitializeTypedArrayFromArrayBuffer
    ///
    /// <https://tc39.es/ecma262/#sec-initializetypedarrayfromarraybuffery>
    fn initialize_from_array_buffer(
        o: JsValue,
        buffer: JsObject,
        byte_offset: &JsValue,
        length: &JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        todo!("InitializeTypedArrayFromArrayBuffer");
    }

    /// InitializeTypedArrayFromArrayLike
    ///
    /// <https://tc39.es/ecma262/#sec-initializetypedarrayfromarraylike>
    fn initialize_from_array_like(
        o: JsValue,
        array_like: JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        todo!("InitializeTypedArrayFromArrayLike");
    }
}

/// Names of all the typed arrays.
#[derive(Debug, Clone, Copy, Finalize)]
enum TypedArrayName {
    Int8Array,
    Uint8Array,
    Uint8ClampedArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    BigInt64Array,
    BigUint64Array,
    Float32Array,
    Float64Array,
}

unsafe impl Trace for TypedArrayName {
    // Safe because `TypedArrayName` is `Copy`
    empty_trace!();
}

impl TypedArrayName {
    /// Gets the element size of the given typed array name, as per the [spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#table-the-typedarray-constructors
    #[inline]
    const fn element_size(self) -> usize {
        match self {
            Self::Int8Array | Self::Uint8Array | Self::Uint8ClampedArray => 1,
            Self::Int16Array | Self::Uint16Array => 2,
            Self::Int32Array | Self::Uint32Array | Self::Float32Array => 4,
            Self::BigInt64Array | Self::BigUint64Array | Self::Float64Array => 8,
        }
    }
}

typed_array!(Int8Array, "Int8Array");
typed_array!(Uint8Array, "UInt8Array");
typed_array!(Uint8ClampedArray, "Uint8ClampedArray");
typed_array!(Int16Array, "Int16Array");
typed_array!(Uint16Array, "Uint16Array");
typed_array!(Int32Array, "Int32Array");
typed_array!(Uint32Array, "Uint32Array");
typed_array!(BigInt64Array, "BigInt64Array");
typed_array!(BigUint64Array, "BigUint64Array");
typed_array!(Float32Array, "Float32Array");
typed_array!(Float64Array, "Float64Array");
