package main

import (
    "log"
    "os"
    "strconv"
    "fmt"

	"golang.org/x/net/context"
    "google.golang.org/grpc"

    pb "../long_tests_pb"
)

const (
	address     = "localhost:23432"
)

func run_echo(c pb.LongTestsClient, cmd_args []string) {
    count := 0
    if len(cmd_args) > 1 {
        log.Fatalf("too many echo params: %s", len(cmd_args))
    } else if len(cmd_args) == 1 {
        count_tmp, err := strconv.Atoi(cmd_args[0])
        if err != nil {
            log.Fatalf("failed to parse int: %v", err)
        }
        count = count_tmp
    } else {
        count = 1
    }

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
    default:
        log.Fatalf("unknown command: %s", cmd)
    }
}
