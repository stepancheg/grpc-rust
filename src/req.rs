use metadata::GrpcMetadata;

#[derive(Debug, Default)]
pub struct GrpcRequestOptions {
    pub metadata: GrpcMetadata,
}

impl GrpcRequestOptions {
    pub fn new() -> GrpcRequestOptions {
        Default::default()
    }
}
