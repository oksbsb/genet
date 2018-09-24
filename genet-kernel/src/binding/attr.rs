use genet_abi::{self, attr::Attr, layer::Layer, token::Token, variant::Variant};
use genet_napi::napi::{
    CallbackInfo, Env, HandleScope, PropertyAttributes, PropertyDescriptor, Result, Status,
    TypedArrayType, Value, ValueRef,
};
use std::{ffi::CString, ptr, rc::Rc};

fn variant_to_js<'env>(
    env: &'env Env,
    value: &genet_abi::result::Result<Variant>,
) -> Result<&'env Value> {
    match value {
        Err(err) => env.create_error(
            env.create_string("")?,
            env.create_string(err.description())?,
        ),
        Ok(Variant::Bool(b)) => env.get_boolean(*b),
        Ok(Variant::Int64(v)) => env.create_int64(*v),
        Ok(Variant::UInt64(v)) => env.create_double(*v as f64),
        Ok(Variant::Float64(v)) => env.create_double(*v),
        Ok(Variant::String(v)) => env.create_string(&v),
        Ok(Variant::Buffer(v)) => env.create_typedarray(
            TypedArrayType::Uint8Array,
            v.len(),
            env.create_arraybuffer_copy(&v)?,
            0,
        ),
        Ok(Variant::Slice(v)) => env.create_typedarray(
            TypedArrayType::Uint8Array,
            v.len(),
            env.create_arraybuffer_from_slice(&v)?,
            0,
        ),
        _ => env.get_null(),
    }
}

pub struct AttrWrapper {
    attr: *const Attr,
    layer: *const Layer,
}

impl AttrWrapper {
    pub fn new(attr: &Attr, layer: &Layer) -> AttrWrapper {
        AttrWrapper { attr, layer }
    }

    fn attr(&self) -> &Attr {
        unsafe { &*self.attr }
    }

    fn layer(&self) -> &Layer {
        unsafe { &*self.layer }
    }
}

pub fn wrapper(env: &Env) -> Rc<ValueRef> {
    fn ctor<'env>(env: &'env Env, info: &'env CallbackInfo) -> Result<&'env Value> {
        env.get_null()
    }

    fn attr_id<'env>(env: &'env Env, info: &'env CallbackInfo) -> Result<&'env Value> {
        let attr = env.unwrap::<AttrWrapper>(info.this())?.attr();
        env.create_string(&attr.id().to_string())
    }

    fn attr_type<'env>(env: &'env Env, info: &'env CallbackInfo) -> Result<&'env Value> {
        let attr = env.unwrap::<AttrWrapper>(info.this())?.attr();
        env.create_string(&attr.typ().to_string())
    }

    fn attr_range<'env>(env: &'env Env, info: &'env CallbackInfo) -> Result<&'env Value> {
        let attr = env.unwrap::<AttrWrapper>(info.this())?.attr();
        let range = attr.range();
        let array = env.create_array(2)?;
        env.set_element(array, 0, env.create_uint32(range.start as u32)?)?;
        env.set_element(array, 1, env.create_uint32(range.end as u32)?)?;
        Ok(array)
    }

    fn attr_value<'env>(env: &'env Env, info: &'env CallbackInfo) -> Result<&'env Value> {
        let wrapper = env.unwrap::<AttrWrapper>(info.this())?;
        variant_to_js(env, &wrapper.attr().try_get(wrapper.layer()))
    }

    let class = env
        .define_class(
            "Attr",
            ctor,
            &[
                PropertyDescriptor::new_property(
                    env,
                    "id",
                    PropertyAttributes::Default,
                    attr_id,
                    false,
                ),
                PropertyDescriptor::new_property(
                    env,
                    "type",
                    PropertyAttributes::Default,
                    attr_type,
                    false,
                ),
                PropertyDescriptor::new_property(
                    env,
                    "range",
                    PropertyAttributes::Default,
                    attr_range,
                    false,
                ),
                PropertyDescriptor::new_property(
                    env,
                    "value",
                    PropertyAttributes::Default,
                    attr_value,
                    false,
                ),
            ],
        ).unwrap();

    env.create_ref(class)
}
