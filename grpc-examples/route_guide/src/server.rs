use crate::route_guide::Feature;
use crate::route_guide::Point;
use crate::route_guide::Rectangle;
use crate::route_guide::RouteNote;
use crate::route_guide::RouteSummary;
use crate::route_guide_grpc::RouteGuide;

use futures::stream::StreamExt;

use grpc::Metadata;
use grpc::ServerHandlerContext;
use grpc::ServerRequest;
use grpc::ServerRequestSingle;
use grpc::ServerResponseSink;
use grpc::ServerResponseUnarySink;
use json::JsonValue;
use std::collections::HashMap;
use std::f64;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

// https://github.com/grpc/grpc-go/blob/master/examples/route_guide/server/server.go
#[derive(Default)]
pub struct RouteGuideImpl {
    saved_features: Arc<Vec<Feature>>,
    route_notes: Arc<Mutex<HashMap<String, Vec<RouteNote>>>>,
}

impl RouteGuideImpl {
    pub fn new_and_load_db() -> RouteGuideImpl {
        RouteGuideImpl {
            saved_features: Arc::new(load_features(Path::new(ROUTE_GUIDE_DB_PATH))),
            route_notes: Default::default(),
        }
    }
}

impl RouteGuide for RouteGuideImpl {
    fn get_feature(
        &self,
        _o: ServerHandlerContext,
        req: ServerRequestSingle<Point>,
        resp: ServerResponseUnarySink<Feature>,
    ) -> grpc::Result<()> {
        for feature in &*self.saved_features {
            if feature.get_location() == &req.message {
                return resp.finish(feature.clone());
            }
        }

        resp.finish(Feature {
            location: Some(req.message).into(),
            ..Default::default()
        })
    }

    fn list_features(
        &self,
        o: ServerHandlerContext,
        mut req: ServerRequestSingle<Rectangle>,
        mut resp: ServerResponseSink<Feature>,
    ) -> grpc::Result<()> {
        let req = req.take_message();
        let saved_features = self.saved_features.clone();
        o.spawn(async move {
            for feature in &saved_features[..] {
                if in_range(feature.get_location(), &req) {
                    resp.ready().await?;
                    resp.send_data(feature.clone())?;
                }
            }
            resp.send_trailers(Metadata::new())
        });
        Ok(())
    }

    fn record_route(
        &self,
        o: ServerHandlerContext,
        req: ServerRequest<Point>,
        resp: ServerResponseUnarySink<RouteSummary>,
    ) -> grpc::Result<()> {
        let start_time = Instant::now();

        let saved_features = self.saved_features.clone();

        let mut stream = req.into_stream();

        o.spawn(async move {
            struct State {
                point_count: u32,
                feature_count: u32,
                distance: u32,
                last_point: Option<Point>,
            }

            let mut state = State {
                point_count: 0,
                feature_count: 0,
                distance: 0,
                last_point: None,
            };

            while let Some(point) = stream.next().await {
                let point = point?;
                state.point_count += 1;
                for feature in &saved_features[..] {
                    if feature.get_location() == &point {
                        state.feature_count += 1;
                    }
                }
                if let Some(last_point) = &state.last_point {
                    state.distance += calc_distance(last_point, &point);
                }
                state.last_point = Some(point);
            }

            resp.finish(RouteSummary {
                point_count: state.point_count as i32,
                feature_count: state.feature_count as i32,
                distance: state.distance as i32,
                elapsed_time: start_time.elapsed().as_secs() as i32,
                ..Default::default()
            })
        });

        Ok(())
    }

    fn route_chat(
        &self,
        o: ServerHandlerContext,
        req: ServerRequest<RouteNote>,
        mut resp: ServerResponseSink<RouteNote>,
    ) -> grpc::Result<()> {
        let route_notes_map = self.route_notes.clone();

        let mut req = req.into_stream();

        o.spawn(async move {
            loop {
                // Wait until resp is writable
                resp.ready().await?;

                match req.next().await {
                    Some(note) => {
                        let note = note?;

                        let key = serialize(note.get_location());

                        let mut route_notes_map = route_notes_map.lock().unwrap();

                        let route_notes = route_notes_map.entry(key).or_insert(Vec::new());
                        route_notes.push(note);

                        for note in route_notes {
                            resp.send_data(note.clone())?;
                        }
                    }
                    None => {
                        return resp.send_trailers(Metadata::new());
                    }
                }
            }
        });

        Ok(())
    }
}

fn in_range(point: &Point, rect: &Rectangle) -> bool {
    let left = f64::min(
        rect.get_lo().longitude as f64,
        rect.get_hi().longitude as f64,
    );
    let right = f64::max(
        rect.get_lo().longitude as f64,
        rect.get_hi().longitude as f64,
    );
    let top = f64::max(rect.get_lo().latitude as f64, rect.get_hi().latitude as f64);
    let bottom = f64::min(rect.get_lo().latitude as f64, rect.get_hi().latitude as f64);

    point.longitude as f64 >= left
        && point.longitude as f64 <= right
        && point.latitude as f64 >= bottom
        && point.latitude as f64 <= top
}

fn to_radians(num: f64) -> f64 {
    num * f64::consts::PI / 180.
}

fn calc_distance(p1: &Point, p2: &Point) -> u32 {
    let cord_factor: f64 = 1e7;
    let r = 6371000.; // earth radius in metres
    let lat1 = to_radians(p1.latitude as f64 / cord_factor);
    let lat2 = to_radians(p2.latitude as f64 / cord_factor);
    let lng1 = to_radians(p1.longitude as f64 / cord_factor);
    let lng2 = to_radians(p2.longitude as f64 / cord_factor);
    let dlat = lat2 - lat1;
    let dlng = lng2 - lng1;

    let a = f64::sin(dlat / 2.) * f64::sin(dlat / 2.)
        + f64::cos(lat1) * f64::cos(lat2) * f64::sin(dlng / 2.) * f64::sin(dlng / 2.);
    let c = 2. * f64::atan2(f64::sqrt(a), f64::sqrt(1. - a));

    let distance = r * c;
    distance as u32
}

fn serialize(point: &Point) -> String {
    format!("{} {}", point.latitude, point.longitude)
}

const ROUTE_GUIDE_DB_PATH: &str = "testdata/route_guide_db.json";

fn load_features(path: &Path) -> Vec<Feature> {
    let mut file = fs::File::open(path).expect("open");
    let mut s = String::new();
    file.read_to_string(&mut s).expect("read");

    // TODO: use protobuf mapper when new version is released

    let json_value = json::parse(&s).expect("parse json");
    let array = match json_value {
        JsonValue::Array(array) => array,
        _ => panic!(),
    };

    array
        .into_iter()
        .map(|item| {
            let object = match item {
                JsonValue::Object(object) => object,
                _ => panic!(),
            };

            let location = match object.get("location").expect("location") {
                JsonValue::Object(object) => object,
                _ => panic!(),
            };

            Feature {
                name: object
                    .get("name")
                    .expect("name")
                    .as_str()
                    .expect("unwrap")
                    .to_owned(),
                location: Some(Point {
                    latitude: location
                        .get("latitude")
                        .expect("latitude")
                        .as_i32()
                        .unwrap(),
                    longitude: location
                        .get("longitude")
                        .expect("longitude")
                        .as_i32()
                        .unwrap(),
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_features() {
        let features = load_features(Path::new(ROUTE_GUIDE_DB_PATH));
        assert!(features.len() > 0);
    }
}
