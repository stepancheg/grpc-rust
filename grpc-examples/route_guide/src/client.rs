//! [client.go](https://github.com/grpc/grpc-go/blob/master/examples/route_guide/client/client.go)

use crate::route_guide::Point;
use crate::route_guide::Rectangle;
use crate::route_guide::RouteNote;
use crate::route_guide_grpc::RouteGuideClient;
use futures::executor;
use futures::StreamExt;
use protobuf::Message;
use protobuf::SingularPtrField;
use rand::thread_rng;
use rand::Rng;
use std::thread;

// print_feature gets the feature for the given point.
async fn print_feature(client: &RouteGuideClient, point: Point) {
    println!(
        "Getting feature for point ({}, {})",
        point.latitude, point.longitude
    );
    let feature = client
        .get_feature(grpc::RequestOptions::new(), point)
        // Drop response metadata
        .drop_metadata()
        .await
        .expect("get_feature");
    println!("feature: {:?}", feature);
}

// print_features lists all the features within the given bounding Rectangle.
async fn print_features(client: &RouteGuideClient, rect: Rectangle) {
    println!("Looking for features within {:?}", rect);
    let resp = client.list_features(grpc::RequestOptions::new(), rect);
    // Stream of features without response metadata
    let mut stream = resp.drop_metadata();
    while let Some(feature) = stream.next().await {
        let feature = feature.expect("feature");
        println!("{:?}", feature);
    }
}

fn random_point() -> Point {
    let mut rng = thread_rng();
    let mut point = Point::new();
    point.latitude = rng.gen_range(-90, 90) * 10_000_000;
    point.longitude = rng.gen_range(-180, 180) * 10_000_000;
    point
}

// run_record_route sends a sequence of points to server and expects to get a RouteSummary from server.
async fn run_record_route(client: &RouteGuideClient) {
    // Create a random number of random points
    let mut rng = thread_rng();
    let point_count = rng.gen_range(2, 102); // Traverse at least two points

    println!("Traversing {} points.", point_count);

    let (mut req, resp) = client
        .record_route(grpc::RequestOptions::new())
        .await
        .expect("request");

    for _ in 0..point_count {
        let point = random_point();
        // Waiting for buffer space to send data.
        req.wait().await.expect("block_wait");
        req.send_data(point).expect("send_data");
    }

    req.finish().unwrap();

    let reply = resp.drop_metadata().await.expect("resp");
    println!("Route summary: {:?}", reply);
}

// run_route_chat receives a sequence of route notes, while sending notes for various locations.
async fn run_route_chat(client: &RouteGuideClient) {
    fn new_note(latitude: i32, longitude: i32, message: &str) -> RouteNote {
        RouteNote {
            location: SingularPtrField::some(Point {
                latitude,
                longitude,
                ..Default::default()
            }),
            message: message.to_owned(),
            ..Default::default()
        }
    }
    let notes = vec![
        new_note(0, 1, "First message"),
        new_note(0, 2, "Second message"),
        new_note(0, 3, "Third message"),
        new_note(0, 1, "Fourth message"),
        new_note(0, 2, "Fifth message"),
        new_note(0, 3, "Sixth message"),
    ];

    let (mut req, resp) = client
        .route_chat(grpc::RequestOptions::new())
        .await
        .unwrap();

    let sender_thread = thread::spawn(move || {
        executor::block_on(async {
            for note in notes {
                req.wait().await.unwrap();
                req.send_data(note).expect("send");
            }
            req.finish().expect("finish");
        });
    });

    let mut responses = resp.drop_metadata();
    while let Some(message) = responses.next().await {
        let message = message.expect("message");
        let location = message
            .location
            .as_ref()
            .unwrap_or(Point::default_instance());
        println!(
            "Got message {} at point({}, {})",
            message.message, location.latitude, location.longitude
        );
    }

    sender_thread.join().expect("sender_thread");
}

pub async fn run_client(client: &RouteGuideClient) {
    // Looking for a valid feature
    let mut point = Point::new();
    point.latitude = 409146138;
    point.longitude = -746188906;
    print_feature(&client, point).await;

    // Feature missing.
    print_feature(&client, Point::new()).await;

    // Looking for features between 40, -75 and 42, -73.
    let mut rect = Rectangle::new();
    rect.hi = SingularPtrField::some({
        let mut point = Point::new();
        point.latitude = 400000000;
        point.longitude = -750000000;
        point
    })
    .into();
    rect.lo = SingularPtrField::some({
        let mut point = Point::new();
        point.latitude = 420000000;
        point.longitude = -730000000;
        point
    })
    .into();

    print_features(&client, rect).await;

    // RecordRoute
    run_record_route(&client).await;

    // RouteChat
    run_route_chat(&client).await;
}
