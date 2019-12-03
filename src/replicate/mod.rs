use crate::{
    channel::{Channel, ForkHandle},
    kind,
    kind::{ConstructResult, DeconstructResult, Future, Stream, WrappedError},
    reflect::{
        CallError, Cast, CastError, Erased, MethodIndex, MethodTypes, NameError, OutOfRangeError,
        Reflected, Trait,
    },
    Kind,
};

use futures::{channel::mpsc::unbounded, stream::empty, SinkExt, StreamExt};
use std::{
    any::{Any, TypeId},
    mem::swap,
    sync::{Arc, Mutex},
};

pub mod collections;

pub trait Share {
    fn share(&self) -> Self;
}

impl<T: Clone> Share for T {
    fn share(&self) -> Self {
        self.clone()
    }
}

pub struct Shared<T: Trait<T> + Reflected + ?Sized>(Arc<Mutex<Box<T>>>);

impl<T: Trait<T> + Reflected + ?Sized> Shared<T> {
    pub fn new(item: Box<T>) -> Self {
        Shared(Arc::new(Mutex::new(item)))
    }
}

impl<T: Trait<T> + Reflected + ?Sized> Share for Shared<T> {
    fn share(&self) -> Self {
        Shared(self.0.share())
    }
}

impl<T: Trait<T> + Reflected + ?Sized> Trait<T> for Shared<T> {
    fn call(
        &self,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send + Sync>>,
    ) -> Result<Box<dyn Any + Send + Sync>, CallError> {
        self.0.lock().unwrap().call(index, args)
    }
    fn call_mut(
        &mut self,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send + Sync>>,
    ) -> Result<Box<dyn Any + Send + Sync>, CallError> {
        self.0.lock().unwrap().call_mut(index, args)
    }
    fn call_move(
        self: Box<Self>,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send + Sync>>,
    ) -> Result<Box<dyn Any + Send + Sync>, CallError> {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!())
            .into_inner()
            .unwrap()
            .call_move(index, args)
    }
    fn by_name(&self, name: &'_ str) -> Result<MethodIndex, NameError> {
        self.0.lock().unwrap().by_name(name)
    }
    fn count(&self) -> MethodIndex {
        self.0.lock().unwrap().count()
    }
    fn name_of(&self, index: MethodIndex) -> Result<String, OutOfRangeError> {
        self.0.lock().unwrap().name_of(index)
    }
    fn this(&self) -> TypeId {
        self.0.lock().unwrap().this()
    }
    fn name(&self) -> String {
        self.0.lock().unwrap().name()
    }
    fn types(&self, index: MethodIndex) -> Result<MethodTypes, OutOfRangeError> {
        self.0.lock().unwrap().types(index)
    }
    fn supertraits(&self) -> Vec<TypeId> {
        self.0.lock().unwrap().supertraits()
    }
    fn upcast_erased(self: Box<Self>, ty: TypeId) -> Result<Box<dyn Erased>, CastError> {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!())
            .into_inner()
            .unwrap()
            .upcast_erased(ty)
    }
    fn erase(self: Box<Self>) -> Box<dyn Erased> {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!())
            .into_inner()
            .unwrap()
            .erase()
    }
}

#[kind]
impl<T: Reflected + Trait<T> + ?Sized> Kind for Shared<T>
where
    Box<T>: Kind,
{
    type ConstructItem = ForkHandle;
    type ConstructError = WrappedError<<Box<T> as Kind>::ConstructError>;
    type ConstructFuture = Future<ConstructResult<Self>>;
    type DeconstructItem = ();
    type DeconstructError = WrappedError<<Box<T> as Kind>::DeconstructError>;
    type DeconstructFuture = Future<DeconstructResult<Self>>;
    fn deconstruct<C: Channel<Self::DeconstructItem, Self::ConstructItem>>(
        self,
        mut channel: C,
    ) -> Self::DeconstructFuture {
        Box::pin(async move {
            Ok(channel
                .send(
                    channel
                        .fork::<Box<T>>(Box::new(self).erase().downcast().unwrap())
                        .await?,
                )
                .await?)
        })
    }
    fn construct<C: Channel<Self::ConstructItem, Self::DeconstructItem>>(
        mut channel: C,
    ) -> Self::ConstructFuture {
        Box::pin(async move {
            let handle = channel.next().await.ok_or(WrappedError::Insufficient {
                got: 0,
                expected: 1,
            })?;
            Ok(Shared::new(channel.get_fork(handle).await?))
        })
    }
}

pub struct React<T: Reactive + Kind>(T, Mutex<Stream<T::Mutation>>);

impl<T: Reactive + Kind> React<T> {
    pub fn split(self) -> (T, Stream<T::Mutation>) {
        (self.0, self.1.into_inner().unwrap())
    }
    pub fn react(&self) -> Stream<T::Mutation>{
        let (mut sender, receiver) = unbounded();
        let mut stream = self.1.lock().unwrap();
        let mut n_stream = Box::pin(empty()) as Stream<T::Mutation>;
        swap(&mut *stream, &mut n_stream);
        *stream = Box::pin(n_stream.inspect(move |item| {
            sender.start_send(item.share()).unwrap();
        }));
        Box::pin(receiver)
    }
    pub fn new(item: T) -> Self {
        let (item, stream) = item.react();
        React(item, Mutex::new(stream))
    }
}

impl<T: Share + Reactive + Kind> Share for React<T> {
    fn share(&self) -> Self {
        let (mut sender, receiver) = unbounded();
        let mut stream = self.1.lock().unwrap();
        let mut n_stream = Box::pin(empty()) as Stream<T::Mutation>;
        swap(&mut *stream, &mut n_stream);
        *stream = Box::pin(n_stream.inspect(move |item| {
            sender.start_send(item.share()).unwrap();
        }));
        React(self.0.share(), Mutex::new(Box::pin(receiver)))
    }
}

#[kind]
impl<T: Reactive + Kind> Kind for React<T> {
    type ConstructItem = ForkHandle;
    type ConstructError = WrappedError<<(T, Stream<T::Mutation>) as Kind>::ConstructError>;
    type ConstructFuture = Future<ConstructResult<Self>>;
    type DeconstructItem = ();
    type DeconstructError = WrappedError<<(T, Stream<T::Mutation>) as Kind>::DeconstructError>;
    type DeconstructFuture = Future<DeconstructResult<Self>>;
    fn deconstruct<C: Channel<Self::DeconstructItem, Self::ConstructItem>>(
        self,
        mut channel: C,
    ) -> Self::DeconstructFuture {
        Box::pin(async move {
            Ok(channel
                .send(
                    channel
                        .fork::<(T, Stream<T::Mutation>)>((self.0, self.1.into_inner().unwrap()))
                        .await?,
                )
                .await?)
        })
    }
    fn construct<C: Channel<Self::ConstructItem, Self::DeconstructItem>>(
        mut channel: C,
    ) -> Self::ConstructFuture {
        Box::pin(async move {
            let handle = channel.next().await.ok_or(WrappedError::<
                <(T, Stream<T::Mutation>) as Kind>::ConstructError,
            >::Insufficient {
                got: 0,
                expected: 1,
            })?;
            let data = channel.get_fork::<(T, Stream<T::Mutation>)>(handle).await?;
            Ok(React(data.0, Mutex::new(data.1)))
        })
    }
}

impl<T: Trait<T> + Reflected> Trait<T> for React<Box<T>>
where
    Box<T>: Reactive + Kind,
{
    fn call(
        &self,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send + Sync>>,
    ) -> Result<Box<dyn Any + Send + Sync>, CallError> {
        self.0.call(index, args)
    }
    fn call_mut(
        &mut self,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send + Sync>>,
    ) -> Result<Box<dyn Any + Send + Sync>, CallError> {
        self.0.call_mut(index, args)
    }
    fn call_move(
        self: Box<Self>,
        index: MethodIndex,
        args: Vec<Box<dyn Any + Send + Sync>>,
    ) -> Result<Box<dyn Any + Send + Sync>, CallError> {
        self.0.call_move(index, args)
    }
    fn by_name(&self, name: &'_ str) -> Result<MethodIndex, NameError> {
        self.0.by_name(name)
    }
    fn count(&self) -> MethodIndex {
        self.0.count()
    }
    fn name_of(&self, index: MethodIndex) -> Result<String, OutOfRangeError> {
        self.0.name_of(index)
    }
    fn this(&self) -> TypeId {
        self.0.this()
    }
    fn name(&self) -> String {
        self.0.name()
    }
    fn types(&self, index: MethodIndex) -> Result<MethodTypes, OutOfRangeError> {
        self.0.types(index)
    }
    fn supertraits(&self) -> Vec<TypeId> {
        self.0.supertraits()
    }
    fn upcast_erased(self: Box<Self>, ty: TypeId) -> Result<Box<dyn Erased>, CastError> {
        self.0.upcast_erased(ty)
    }
    fn erase(self: Box<Self>) -> Box<dyn Erased> {
        self.0.erase()
    }
}

pub trait Reactive: Sized {
    type Mutation: Kind + Share;

    fn react(self) -> (Self, Stream<Self::Mutation>);
}
