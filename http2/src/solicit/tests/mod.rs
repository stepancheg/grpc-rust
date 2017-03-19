#[cfg(test)]
pub mod common;

/// Tests for the structs defined in the root of the `solicit::http` module.
#[cfg(test)]
mod root_tests {
    use std::error::Error;

    use solicit::{Response, HttpError, HttpScheme, ErrorCode, ConnectionError, Header};

    /// Tests that the `Response` struct correctly parses a status code from
    /// its headers list.
    #[test]
    fn test_parse_status_code_response() {
        {
            // Only status => Ok
            let resp = Response::new(1, vec![(b":status".to_vec(), b"200".to_vec())], vec![]);
            assert_eq!(resp.status_code().ok().unwrap(), 200);
        }
        {
            // Extra headers => still works
            let resp = Response::new(1,
                                     vec![(b":status".to_vec(), b"200".to_vec()),
                                          (b"key".to_vec(), b"val".to_vec())],
                                     vec![]);
            assert_eq!(resp.status_code().ok().unwrap(), 200);
        }
        {
            // Status is not the first header => malformed
            let resp = Response::new(1,
                                     vec![(b"key".to_vec(), b"val".to_vec()),
                                          (b":status".to_vec(), b"200".to_vec())],
                                     vec![]);
            assert_eq!(resp.status_code().err().unwrap(),
                       HttpError::MalformedResponse);
        }
        {
            // No headers at all => Malformed
            let resp = Response::new(1, Vec::<Header>::new(), vec![]);
            assert_eq!(resp.status_code().err().unwrap(),
                       HttpError::MalformedResponse);
        }
    }

    #[test]
    fn test_connection_error_no_debug_data() {
        let err = ConnectionError::new(ErrorCode::ProtocolError);
        assert_eq!(err.error_code(), ErrorCode::ProtocolError);
        assert!(err.debug_data().is_none());
        assert!(err.debug_str().is_none());
        assert_eq!(err.description(), "ProtocolError");
    }

    #[test]
    fn test_connection_error_raw_debug_data() {
        let err = ConnectionError::with_debug_data(ErrorCode::ProtocolError, vec![0x80]);
        assert_eq!(err.error_code(), ErrorCode::ProtocolError);
        assert_eq!(err.debug_data().unwrap(), &[0x80][..]);
        assert!(err.debug_str().is_none());
        assert_eq!(err.description(), "ProtocolError");
    }

    #[test]
    fn test_connection_error_str_debug_data() {
        let err = ConnectionError::with_debug_data(ErrorCode::ProtocolError, b"Test".to_vec());
        assert_eq!(err.error_code(), ErrorCode::ProtocolError);
        assert_eq!(err.debug_data().unwrap(), b"Test");
        assert_eq!(err.debug_str().unwrap(), "Test");
        assert_eq!(err.description(), "Test");
    }

    /// Tests that the `HttpScheme` enum returns the correct scheme strings for
    /// the two variants.
    #[test]
    fn test_scheme_string() {
        assert_eq!(HttpScheme::Http.as_bytes(), b"http");
        assert_eq!(HttpScheme::Https.as_bytes(), b"https");
    }

    /// Make sure that the `HttpError` is both `Sync` and `Send`
    #[test]
    fn _assert_error_is_sync_send() {
        fn _is_sync_send<T: Sync + Send>() {}
        _is_sync_send::<HttpError>();
    }
}

#[cfg(test)]
mod test_header {
    use solicit::Header;

    fn _assert_is_static(_: Header) {}

    #[test]
    fn test_header_from_static_slices_lifetime() {
        let header = Header::new(b":method", b"GET");
        _assert_is_static(header);
    }

    #[test]
    fn test_partial_eq_of_headers() {
        let fully_static = Header::new(b":method", b"GET");
        let static_name = Header::new(b":method", b"GET".to_vec());
        let other = Header::new(b":path", b"/");

        assert!(fully_static == static_name);
        assert!(fully_static != other);
        assert!(static_name != other);
    }

    #[test]
    fn test_debug() {
        assert_eq!(
            "Header { name: b\":method\", value: b\"GET\" }",
            format!("{:?}", Header::new(b":method", b"GET")));
        assert_eq!(
            "Header { name: b\":method\", value: b\"\\xcd\" }",
            format!("{:?}", Header::new(b":method", b"\xcd")));
    }
}
