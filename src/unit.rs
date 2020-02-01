use crate::{Channels, Protocol};
use core::{
    convert::Infallible,
    marker::{PhantomData, PhantomPinned},
};
use futures::future::{ready, Ready};

pub trait Unit {
    const VALUE: Self;
}

impl<T: Unit, C> Protocol<C> for T {
    type Unravel = Infallible;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Infallible;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<T, Infallible>>;

    fn unravel(self, _: C::Unravel) -> Self::UnravelFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }

    fn coalesce(_: C::Coalesce) -> Self::CoalesceFuture
    where
        C: Channels<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(T::VALUE))
    }
}

impl Unit for () {
    const VALUE: Self = ();
}

impl<T> Unit for [T; 0] {
    const VALUE: Self = [];
}

impl Unit for PhantomPinned {
    const VALUE: Self = PhantomPinned;
}

impl<T: ?Sized> Unit for PhantomData<T> {
    const VALUE: Self = PhantomData;
}
