use super::super::callback::Callback;
use super::{Component, NodeRef, Scope};
use crate::virtual_dom::AttrValue;
use std::rc::Rc;

/// Marker trait for types that the [`html!`](macro@crate::html) macro may clone implicitly.
pub trait ImplicitClone: Clone {}

impl<T: ImplicitClone> ImplicitClone for Option<T> {}
impl<T> ImplicitClone for Rc<T> {}

impl ImplicitClone for NodeRef {}
impl<Comp: Component> ImplicitClone for Scope<Comp> {}
// TODO there are still a few missing

macro_rules! impl_implicit_clone {
    ($($ty:ty),+ $(,)?) => {
        $(impl ImplicitClone for $ty {})*
    };
}

#[rustfmt::skip]
impl_implicit_clone!(
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64,
    &'static str,
);

/// A trait similar to `Into<T>` which allows conversion to a value of a `Properties` struct.
pub trait IntoPropValue<T> {
    /// Convert `self` to a value of a `Properties` struct.
    fn into_prop_value(self) -> T;
}

impl<T> IntoPropValue<T> for T {
    #[inline]
    fn into_prop_value(self) -> T {
        self
    }
}
impl<T> IntoPropValue<T> for &T
where
    T: ImplicitClone,
{
    #[inline]
    fn into_prop_value(self) -> T {
        self.clone()
    }
}

impl<T> IntoPropValue<Option<T>> for T {
    #[inline]
    fn into_prop_value(self) -> Option<T> {
        Some(self)
    }
}
impl<T> IntoPropValue<Option<T>> for &T
where
    T: ImplicitClone,
{
    #[inline]
    fn into_prop_value(self) -> Option<T> {
        Some(self.clone())
    }
}

impl<I, O, F> IntoPropValue<Callback<I, O>> for F
where
    F: 'static + Fn(I) -> O,
{
    fn into_prop_value(self) -> Callback<I, O> {
        Callback::from(self)
    }
}

impl<I, O, F> IntoPropValue<Option<Callback<I, O>>> for F
where
    F: 'static + Fn(I) -> O,
{
    fn into_prop_value(self) -> Option<Callback<I, O>> {
        Some(Callback::from(self))
    }
}

impl<I, O, F> IntoPropValue<Option<Callback<I, O>>> for Option<F>
where
    F: 'static + Fn(I) -> O,
{
    fn into_prop_value(self) -> Option<Callback<I, O>> {
        self.map(|f| Callback::from(f))
    }
}

macro_rules! impl_into_prop {
    (|$value:ident: $from_ty:ty| -> $to_ty:ty { $conversion:expr }) => {
        // implement V -> T
        impl IntoPropValue<$to_ty> for $from_ty {
            #[inline]
            fn into_prop_value(self) -> $to_ty {
                let $value = self;
                $conversion
            }
        }
        // implement V -> Option<T>
        impl IntoPropValue<Option<$to_ty>> for $from_ty {
            #[inline]
            fn into_prop_value(self) -> Option<$to_ty> {
                let $value = self;
                Some({ $conversion })
            }
        }
        // implement Option<V> -> Option<T>
        impl IntoPropValue<Option<$to_ty>> for Option<$from_ty> {
            #[inline]
            fn into_prop_value(self) -> Option<$to_ty> {
                self.map(IntoPropValue::into_prop_value)
            }
        }
    };
}

// implemented with literals in mind
impl_into_prop!(|value: &'static str| -> String { value.to_owned() });

impl_into_prop!(|value: &'static str| -> AttrValue { AttrValue::Static(value) });
impl_into_prop!(|value: String| -> AttrValue { AttrValue::Rc(Rc::from(value)) });
impl_into_prop!(|value: Rc<str>| -> AttrValue { AttrValue::Rc(value) });

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_str() {
        let _: String = "foo".into_prop_value();
        let _: Option<String> = "foo".into_prop_value();
        let _: AttrValue = "foo".into_prop_value();
        let _: Option<AttrValue> = "foo".into_prop_value();
        let _: Option<AttrValue> = Rc::<str>::from("foo").into_prop_value();
    }

    #[test]
    fn test_callback() {
        let _: Callback<String> = (|_: String| ()).into_prop_value();
        let _: Option<Callback<String>> = (|_: String| ()).into_prop_value();
        let _: Option<Callback<String>> = Some(|_: String| ()).into_prop_value();
        let _: Callback<String, String> = (|s: String| s).into_prop_value();
        let _: Option<Callback<String, String>> = (|s: String| s).into_prop_value();
        let _: Option<Callback<String, String>> = Some(|s: String| s).into_prop_value();
    }
}
