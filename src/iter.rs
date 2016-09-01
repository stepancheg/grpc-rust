use result::GrpcResult;

pub type BoxIterator<T> = Box<Iterator<Item=T> + Send>;

pub type GrpcIterator<T> = BoxIterator<GrpcResult<T>>;
