use std::ops;
use std::sync::Arc;

pub enum ArcOrStatic<A: ?Sized + 'static> {
    Arc(Arc<A>),
    Static(&'static A),
}

impl<A: ?Sized> Clone for ArcOrStatic<A> {
    fn clone(&self) -> Self {
        match self {
            ArcOrStatic::Arc(a) => ArcOrStatic::Arc(a.clone()),
            ArcOrStatic::Static(a) => ArcOrStatic::Static(a),
        }
    }
}

impl<A: ?Sized> ops::Deref for ArcOrStatic<A> {
    type Target = A;

    fn deref(&self) -> &A {
        match self {
            ArcOrStatic::Arc(a) => &*a,
            ArcOrStatic::Static(a) => a,
        }
    }
}
