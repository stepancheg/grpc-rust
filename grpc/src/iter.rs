use result;

type BoxIterator<T> = Box<Iterator<Item = T> + Send>;

pub type GrpcIterator<T> = BoxIterator<result::Result<T>>;
