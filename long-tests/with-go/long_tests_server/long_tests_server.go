package main

import (
    "net"
    "log"
    "google.golang.org/grpc"

    //"os"
    //"strconv"
    //"fmt"
    //
	"golang.org/x/net/context"
    //"google.golang.org/grpc"

    pb "../long_tests_pb"
    //"math/rand"
    //"io"
    "io"
)

const (
	port     = ":23432"
)

type server struct{}

func (s *server) Echo(context context.Context, req *pb.EchoRequest) (*pb.EchoResponse, error) {
    return &pb.EchoResponse{Payload: req.Payload}, nil
}

func (s *server) CharCount(req pb.LongTests_CharCountServer) error {
    count := 0
    for {
        m, err := req.Recv()
        if err == io.EOF {
            req.SendAndClose(&pb.CharCountResponse{CharCount: uint64(count)})
            return nil
        }
        if err != nil {
            return err
        }
        count += len(m.Part)
    }
}

func (s *server) RandomStrings(req *pb.RandomStringsRequest, resp pb.LongTests_RandomStringsServer) error {
    for i := 0; i < int(req.Count); i += 1 {
        err := resp.Send(&pb.RandomStringsResponse{S: "aabb"})
        if err != nil {
            return err
        }
    }
    return nil
}


func main() {
    lis, err := net.Listen("tcp", port)
    if err != nil {
        log.Fatalf("failed to listen: %v", err)
    }

    s := grpc.NewServer()
    pb.RegisterLongTestsServer(s, &server{})

    if err := s.Serve(lis); err != nil {
   		log.Fatalf("failed to serve: %v", err)
   }
}
