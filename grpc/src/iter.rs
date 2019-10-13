use result;

type BoxIterator<T> = Box<dyn Iterator<Item = T> + Send>;

pub type GrpcIterator<T> = BoxIterator<result::Result<T>>;
