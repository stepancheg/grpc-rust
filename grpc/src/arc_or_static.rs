use std::sync::Arc;
use std::ops;

#[derive(Clone)]
pub enum ArcOrStatic<A: 'static> {
    Arc(Arc<A>),
    Static(&'static A),
}

impl<A: 'static> ops::Deref for ArcOrStatic<A> {
    type Target = A;

    fn deref(&self) -> &A {
        match self {
            ArcOrStatic::Arc(a) => &*a,
            ArcOrStatic::Static(a) => a,
        }
    }
}
