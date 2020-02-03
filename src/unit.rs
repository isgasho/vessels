use crate::{Channels, Protocol};
use core::{
    convert::Infallible,
    marker::{PhantomData, PhantomPinned},
};
use futures::future::{ready, Ready};

impl<C> Protocol<C> for () {
    type Unravel = Infallible;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Infallible;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<(), Infallible>>;

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
        ready(Ok(()))
    }
}

impl<T, C> Protocol<C> for [T; 0] {
    type Unravel = Infallible;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Infallible;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<[T; 0], Infallible>>;

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
        ready(Ok([]))
    }
}

impl<T: ?Sized, C> Protocol<C> for PhantomData<T> {
    type Unravel = Infallible;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Infallible;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<PhantomData<T>, Infallible>>;

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
        ready(Ok(PhantomData))
    }
}

impl<C> Protocol<C> for PhantomPinned {
    type Unravel = Infallible;
    type UnravelError = Infallible;
    type UnravelFuture = Ready<Result<(), Infallible>>;
    type Coalesce = Infallible;
    type CoalesceError = Infallible;
    type CoalesceFuture = Ready<Result<PhantomPinned, Infallible>>;

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
        ready(Ok(PhantomPinned))
    }
}
