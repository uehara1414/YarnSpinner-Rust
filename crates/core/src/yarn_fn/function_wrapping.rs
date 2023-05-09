use crate::prelude::*;
use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use yarn_slinger_macros::all_tuples;

/// A function that can be registered into and called from Yarn.
/// It must have the following properties:
/// - It is allowed to have zero or more parameters
/// - Each parameter must be one of the following types:
///   - [`bool`]
///   - [`String`]
///   - A numeric type, i.e. one of [`f32`], [`f64`], [`i8`], [`i16`], [`i32`], [`i64`], [`i128`], [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`usize`], [`isize`]
///   - [`YarnValue`], which means that this parameter may be any of any of the above types
/// - Its parameters must be passed by value
/// - It must have a return type
/// - Its return type must be one of the following types:
///     - [`bool`]
///     - [`String`]
///     - A numeric type, i.e. one of [`f32`], [`f64`], [`i8`], [`i16`], [`i32`], [`i64`], [`i128`], [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`usize`], [`isize`]
pub trait YarnFn<Marker>: Clone + Send + Sync {
    type Out: IntoYarnValueFromNonYarnValue + 'static;
    fn call(&self, input: Vec<YarnValue>) -> Self::Out;
    fn parameter_types(&self) -> Vec<TypeId>;
    fn return_type(&self) -> TypeId {
        TypeId::of::<Self::Out>()
    }
}

/// A [`YarnFn`] with the `Marker` type parameter erased.
/// See its documentation for more information about what kind of functions are allowed.
pub trait UntypedYarnFn: Debug + Send + Sync {
    fn call(&self, input: Vec<YarnValue>) -> YarnValue;
    fn clone_box(&self) -> Box<dyn UntypedYarnFn + Send + Sync>;
    fn parameter_types(&self) -> Vec<TypeId>;
    fn return_type(&self) -> TypeId;
}

impl Clone for Box<dyn UntypedYarnFn + Send + Sync> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<Marker, F> UntypedYarnFn for YarnFnWrapper<Marker, F>
where
    Marker: 'static + Clone,
    F: YarnFn<Marker> + 'static + Clone + Send + Sync,
    F::Out: IntoYarnValueFromNonYarnValue + 'static + Clone,
{
    fn call(&self, input: Vec<YarnValue>) -> YarnValue {
        let output = self.function.call(input);
        output.into_untyped_value()
    }

    fn clone_box(&self) -> Box<dyn UntypedYarnFn + Send + Sync> {
        Box::new(self.clone())
    }

    fn parameter_types(&self) -> Vec<TypeId> {
        self.function.parameter_types()
    }

    fn return_type(&self) -> TypeId {
        self.function.return_type()
    }
}

#[derive(Clone)]
pub(crate) struct YarnFnWrapper<Marker, F>
where
    F: YarnFn<Marker>,
{
    function: F,

    // NOTE: PhantomData<fn()-> T> gives this safe Send/Sync impls
    _marker: PhantomData<fn() -> Marker>,
}

impl<Marker, F> From<F> for YarnFnWrapper<Marker, F>
where
    F: YarnFn<Marker>,
{
    fn from(function: F) -> Self {
        Self {
            function,
            _marker: PhantomData,
        }
    }
}

impl<Marker, F> Debug for YarnFnWrapper<Marker, F>
where
    F: YarnFn<Marker>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let signature = std::any::type_name::<Marker>();
        let function_path = std::any::type_name::<F>();
        let debug_message = format!("{signature} {{{function_path}}}");
        f.debug_struct(&debug_message).finish()
    }
}

impl PartialEq for Box<dyn UntypedYarnFn + Send + Sync> {
    fn eq(&self, other: &Self) -> bool {
        // Not guaranteed to be unique, but that's good enough for our purposes.
        let debug = format!("{:?}", self);
        let other_debug = format!("{:?}", other);
        debug == other_debug
    }
}

impl Eq for Box<dyn UntypedYarnFn + Send + Sync> {}

#[derive(Debug)]
pub struct YarnValueWrapper {
    raw: Option<YarnValue>,
    converted: Option<Box<dyn Any>>,
}

impl From<YarnValue> for YarnValueWrapper {
    fn from(value: YarnValue) -> Self {
        Self {
            raw: Some(value),
            converted: None,
        }
    }
}

impl YarnValueWrapper {
    fn convert<T>(&mut self)
    where
        T: TryFrom<YarnValue> + 'static,
        <T as TryFrom<YarnValue>>::Error: Debug,
    {
        let raw = std::mem::take(&mut self.raw).unwrap();
        let converted: T = raw.try_into().unwrap();
        self.converted.replace(Box::new(converted));
    }
}

pub trait YarnFnParam {
    type Item<'new>;
    fn retrieve<'r>(value: &'r mut YarnValueWrapper) -> Self::Item<'r>;
}

struct ResRef<'a, T, U = T>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Debug,
    T: AsRef<U>,
    U: ?Sized,
{
    value: &'a U,
    phantom_data: PhantomData<T>,
}

impl<'res, T, U> YarnFnParam for ResRef<'res, T, U>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Debug,
    T: AsRef<U>,
    U: ?Sized,
{
    type Item<'new> = ResRef<'res, T, U>;
    fn retrieve<'r>(value: &'r mut YarnValueWrapper) -> Self::Item<'r> {
        value.convert::<T>();
        let converted = value.converted.as_ref().unwrap();
        let value = converted.downcast_ref::<T>().unwrap();
        ResRef {
            value: value.as_ref(),
            phantom_data: PhantomData,
        }
    }
}

struct ResOwned<T>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Debug,
{
    value: T,
}

impl<'res, T> YarnFnParam for ResOwned<T>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Debug,
{
    type Item<'new> = ResOwned<T>;
    fn retrieve<'r>(value: &'r mut YarnValueWrapper) -> Self::Item<'r> {
        value.convert::<T>();
        let converted = value.converted.take().unwrap();
        let value = *converted.downcast::<T>().unwrap();
        ResOwned { value }
    }
}

macro_rules! impl_ref_param {
    ([$(&$param:ty $(=> $owned:ty)?),*]: YarnFnParam) => {
        $(
            impl YarnFnParam for &$param {
                type Item<'new> = &'new $param;

                fn retrieve<'r>(value: &'r mut YarnValueWrapper) -> Self::Item<'r> {
                    ResRef::<'r,$ ($owned,)? $param>::retrieve(value).value
                }
            }
        )*
    };
}

macro_rules! impl_owned_param {
    ([$($param: ty),*]: YarnFnParam) => {
        $(
            impl YarnFnParam for $param {
                type Item<'new> = $param;

                fn retrieve<'r>(value: &'r mut YarnValueWrapper) -> Self::Item<'r> {
                    ResOwned::<$param>::retrieve(value).value
                }
            }
        )*
    };
}

impl_ref_param! {
    [&str => String]: YarnFnParam
}
impl_owned_param! {
    [String, usize]: YarnFnParam
}

/// Adapted from <https://github.com/bevyengine/bevy/blob/fe852fd0adbce6856f5886d66d20d62cfc936287/crates/bevy_ecs/src/system/system_param.rs#L1370>
macro_rules! impl_yarn_fn_tuple {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<F, O, $($param,)*> YarnFn<fn($($param,)*) -> O> for F
            where
            for <'a>F:
                Send + Sync + Clone +
                Fn($($param,)*) -> O +
                Fn($(<$param as YarnFnParam>::Item<'a>,)*) -> O,
            O: IntoYarnValueFromNonYarnValue + 'static,
            $($param: YarnFnParam + 'static,)*
            {
                type Out = O;
                #[allow(non_snake_case)]
                fn call(&self, input: Vec<YarnValue>) -> Self::Out {
                    let [$($param,)*] = input[..] else {
                        panic!("Wrong number of arguments")
                    };

                    let ($($param,)*) = (
                        $(YarnValueWrapper::from($param),)*
                    );

                    let input = (
                        $($param::retrieve(&mut $param),)*
                    );
                    let ($($param,)*) = input;
                    self($($param,)*)
                }

                fn parameter_types(&self) -> Vec<TypeId> {
                    vec![$(TypeId::of::<$param>()),*]
                }
            }
    };
}

all_tuples!(impl_yarn_fn_tuple, 0, 1, P);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_no_params() {
        fn f() -> bool {
            true
        }
        accept_yarn_fn(f);
    }

    #[test]
    fn accepts_string() {
        fn f(_: String) -> bool {
            true
        }
        accept_yarn_fn(f);
    }

    #[test]
    fn accepts_string_slice() {
        fn f(_: &str) -> bool {
            true
        }
        accept_yarn_fn(f);
    }

    #[test]
    fn accepts_usize() {
        fn f(_: usize) -> bool {
            true
        }
        accept_yarn_fn(f);
    }

    fn accept_yarn_fn<Marker>(_: impl YarnFn<Marker>) {}
}
