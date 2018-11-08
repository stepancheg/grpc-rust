use Metadata;

pub struct ServerRequestSingle<Req> {
    pub metadata: Metadata,
    pub message: Req,
}
