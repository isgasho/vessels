use crate::{Context, Protocol, Transport};
use futures::future::{ready, Ready};
use void::Void;

impl<'a, C: Context> Protocol<'a, C> for () {
    type Unravel = Void;
    type UnravelFuture = Ready<Result<(), Void>>;
    type Coalesce = Void;
    type CoalesceFuture = Ready<Result<(), Void>>;

    fn unravel(self, _: &'a mut C::Unravel) -> Self::UnravelFuture
    where
        C: Transport<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }

    fn coalesce(_: &'a mut C::Coalesce) -> Self::CoalesceFuture
    where
        C: Transport<Self::Unravel, Self::Coalesce>,
    {
        ready(Ok(()))
    }
}
