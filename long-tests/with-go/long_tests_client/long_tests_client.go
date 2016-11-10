package main

import (
    "log"
    "os"
    "strconv"
    "fmt"

	"golang.org/x/net/context"
    "google.golang.org/grpc"

    pb "../long_tests_pb"
    "math/rand"
    "io"
)

const (
	address     = "localhost:23432"
)

func single_num_arg_or(cmd_args []string, or int) int {
    if len(cmd_args) > 1 {
        log.Fatalf("too many char_count params: %s", len(cmd_args));
        return 0
    } else if len(cmd_args) == 1 {
        count_tmp, err := strconv.Atoi((cmd_args[0]));
        if err != nil {
            log.Fatalf("failed to parse int: %v", err);
        }
        return count_tmp
    } else {
        return or
    }
}

func run_echo(c pb.LongTestsClient, cmd_args []string) {
    count := single_num_arg_or(cmd_args, 1)

    log.Printf("running %d iterations of echo", count)

    for i := 0; i < count; i += 1 {
        payload := fmt.Sprintf("payload %s", i)

        r, err := c.Echo(context.Background(), &pb.EchoRequest{Payload: payload})

        if err != nil {
            log.Fatalf("could not greet: %v", err)
        }

        if r.Payload != payload {
            log.Fatalf("wrong payload: %v", r)
        }
    }

    log.Printf("done")
}

func run_char_count(c pb.LongTestsClient, cmd_args []string) {
    count := single_num_arg_or(cmd_args, 10)

    log.Printf("sending %d messages to count", count)

    client, err := c.CharCount(context.Background())
    if err != nil {
        log.Fatalf("failed to start request: %v", err)
    }

    expected := uint64(0);
    for i := 0; i < count; i += 1 {
        s := "abcdefghijklmnopqrstuvwxyz0123456789"
        part := s[:rand.Intn(len(s))]
        client.Send(&pb.CharCountRequest{Part: part})
        expected += uint64(len(part))
    }

    resp, err := client.CloseAndRecv()
    if err != nil {
        log.Fatalf("failed to get response: %v", err);
    }

    if expected != resp.CharCount {
        log.Fatalf("expected: %s actual: %s", expected, resp.CharCount)
    }

    log.Printf("successfully got correct answer")
}

func run_random_strings(c pb.LongTestsClient, cmd_args []string) {
    count := single_num_arg_or(cmd_args, 10)

    log.Printf("requesting %d string from server", count)

    client, err := c.RandomStrings(context.Background(), &pb.RandomStringsRequest{ Count: uint64(count) })
    if err != nil {
        log.Fatalf("failed to execute request: %v", err)
    }

    for {
        _, err := client.Recv()
        if err == io.EOF {
            log.Printf("got eof")
            break
        }
        if err != nil {
            log.Fatalf("could not read response part: %v", err)
        }
    }
}


func main() {
    conn, err := grpc.Dial(address, grpc.WithInsecure())
   	if err != nil {
   		log.Fatalf("did not connect: %v", err)
    }

    defer conn.Close()

    c := pb.NewLongTestsClient(conn)

    if len(os.Args) < 2 {
        log.Fatalf("too few args")
    }

    cmd := os.Args[1]

    cmd_args := os.Args[2:]

    switch cmd {
    case "echo":
        run_echo(c, cmd_args)
        return
    case "char_count":
        run_char_count(c, cmd_args)
        return
    case "random_strings":
        run_random_strings(c, cmd_args)
        return
    default:
        log.Fatalf("unknown command: %s", cmd)
    }
}
