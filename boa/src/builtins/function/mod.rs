//! This module implements the global `Function` object as well as creates Native Functions.
//!
//! Objects wrap `Function`s and expose them via call/construct slots.
//!
//! `The `Function` object is used for matching text with a pattern.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-function-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function

use crate::{
    builtins::BuiltIn,
    object::{function::Function, ConstructorBuilder, FunctionBuilder, ObjectData, PROTOTYPE},
    property::Attribute,
    BoaProfiler, Context, JsResult, JsValue,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy)]
pub struct BuiltInFunctionObject;

impl BuiltInFunctionObject {
    pub const LENGTH: usize = 1;

    fn constructor(
        new_target: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.__get__(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.standard_objects().object_object().prototype());
        let this = JsValue::new_object(context);

        this.as_object()
            .expect("this should be an object")
            .set_prototype_instance(prototype.into());

        this.set_data(ObjectData::function(Function::Native {
            function: |_, _, _| Ok(JsValue::undefined()),
            constructable: true,
        }));
        Ok(this)
    }

    fn prototype(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    /// `Function.prototype.call`
    ///
    /// The call() method invokes self with the first argument as the `this` value.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.call
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call
    fn call(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg: JsValue = args.get(0).cloned().unwrap_or_default();
        // TODO?: 3. Perform PrepareForTailCall
        let start = if !args.is_empty() { 1 } else { 0 };
        context.call(this, &this_arg, &args[start..])
    }

    /// `Function.prototype.apply`
    ///
    /// The apply() method invokes self with the first argument as the `this` value
    /// and the rest of the arguments provided as an array (or an array-like object).
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.apply
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/apply
    fn apply(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg = args.get(0).cloned().unwrap_or_default();
        let arg_array = args.get(1).cloned().unwrap_or_default();
        if arg_array.is_null_or_undefined() {
            // TODO?: 3.a. PrepareForTailCall
            return context.call(this, &this_arg, &[]);
        }
        let arg_list = arg_array.create_list_from_array_like(&[], context)?;
        // TODO?: 5. PrepareForTailCall
        context.call(this, &this_arg, &arg_list)
    }
}

impl BuiltIn for BuiltInFunctionObject {
    const NAME: &'static str = "Function";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event("function", "init");

        let function_prototype = context.standard_objects().function_object().prototype();
        FunctionBuilder::native(context, Self::prototype)
            .name("")
            .length(0)
            .constructable(false)
            .build_function_prototype(&function_prototype);

        let function_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().function_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::call, "call", 1)
        .method(Self::apply, "apply", 1)
        .build();

        (Self::NAME, function_object.into(), Self::attribute())
    }
}
