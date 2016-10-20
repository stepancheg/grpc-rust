package main

import (
	"log"

	"golang.org/x/net/context"
    "google.golang.org/grpc"

    pb "../long_tests_pb"
)

const (
	address     = "localhost:23432"
)

func main() {
    conn, err := grpc.Dial(address, grpc.WithInsecure())
   	if err != nil {
   		log.Fatalf("did not connect: %v", err)
    }

    defer conn.Close()

    c := pb.NewLongTestsClient(conn)

    r, err := c.Echo(context.Background(), &pb.EchoRequest{Payload: "there"})

   	if err != nil {
   		log.Fatalf("could not greet: %v", err)
    }

    if r.Payload != "there" {
        log.Fatalf("wrong payload: %v", r)
    }
}
