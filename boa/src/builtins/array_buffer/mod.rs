use crate::{
    context::{StandardConstructor, StandardObjects},
    object::JsObject,
    Context, JsResult,
};

#[derive(Debug, Clone, Copy)]
pub struct ArrayBuffer;

impl ArrayBuffer {
    /// <https://tc39.es/ecma262/#sec-allocatearraybuffer>
    pub(crate) fn allocate<F>(
        constructor: F,
        byte_length: usize,
        context: &Context,
    ) -> JsResult<JsObject>
    where
        F: FnOnce(&StandardObjects) -> &StandardConstructor,
    {
        // 1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", « [[ArrayBufferData]], [[ArrayBufferByteLength]], [[ArrayBufferDetachKey]] »).
        // 2. Let block be ? CreateByteDataBlock(byteLength).
        // 3. Set obj.[[ArrayBufferData]] to block.
        // 4. Set obj.[[ArrayBufferByteLength]] to byteLength.
        // 5. Return obj.
        todo!("AllocateArrayBuffer")
    }
}
