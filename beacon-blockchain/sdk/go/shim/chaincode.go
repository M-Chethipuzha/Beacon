package shim

import (
	"context"
	"fmt"
	"log"
	"os"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "github.com/beacon-blockchain/sdk-go/proto"
)

// ChaincodeStub provides the interface for chaincodes to interact with the blockchain
type ChaincodeStub struct {
	client        pb.ChaincodeShimClient
	conn          *grpc.ClientConn
	transactionID string
	channelID     string
	creator       []byte
	timestamp     int64
}

// Chaincode interface that all Go chaincodes must implement
type Chaincode interface {
	// Init is called during chaincode instantiation to initialize chaincode data
	Init(stub ChaincodeStubInterface) Response

	// Invoke is called to update or query the ledger in a proposal transaction
	Invoke(stub ChaincodeStubInterface) Response
}

// ChaincodeStubInterface defines the interface for chaincode to access its state
type ChaincodeStubInterface interface {
	// State management operations
	GetState(key string) ([]byte, error)
	PutState(key string, value []byte) error
	DelState(key string) error

	// Range queries
	GetStateByRange(startKey, endKey string) (StateQueryIteratorInterface, error)
	GetStateByPartialCompositeKey(objectType string, keys []string) (StateQueryIteratorInterface, error)

	// Transaction context
	GetTxID() string
	GetChannelID() string
	GetCreator() ([]byte, error)
	GetTxTimestamp() (*time.Time, error)

	// Event operations
	SetEvent(name string, payload []byte) error

	// Logging
	LogMessage(level LogLevel, message string) error

	// Function and argument access
	GetFunctionAndParameters() (string, []string)
	GetStringArgs() []string
	GetArgs() [][]byte
}

// Response structure for chaincode functions
type Response struct {
	Status  int32
	Message string
	Payload []byte
}

// StateQueryIteratorInterface for iterating over query results
type StateQueryIteratorInterface interface {
	HasNext() bool
	Next() (*KeyValue, error)
	Close() error
}

// KeyValue represents a key-value pair
type KeyValue struct {
	Key   string
	Value []byte
}

// LogLevel enumeration for logging
type LogLevel int32

const (
	DEBUG LogLevel = 0
	INFO  LogLevel = 1
	WARN  LogLevel = 2
	ERROR LogLevel = 3
)

// Success creates a successful response
func Success(payload []byte) Response {
	return Response{
		Status:  200,
		Message: "OK",
		Payload: payload,
	}
}

// Error creates an error response
func Error(message string) Response {
	return Response{
		Status:  500,
		Message: message,
		Payload: nil,
	}
}

// NewChaincodeStub creates a new chaincode stub with gRPC connection
func NewChaincodeStub() (*ChaincodeStub, error) {
	// Get gRPC server address from environment
	grpcAddr := os.Getenv("BEACON_GRPC_ADDRESS")
	if grpcAddr == "" {
		grpcAddr = "127.0.0.1:9090" // Default address
	}

	// Establish gRPC connection
	conn, err := grpc.Dial(grpcAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, fmt.Errorf("failed to connect to gRPC server: %w", err)
	}

	client := pb.NewChaincodeShimClient(conn)

	stub := &ChaincodeStub{
		client:        client,
		conn:          conn,
		transactionID: os.Getenv("BEACON_TRANSACTION_ID"),
		channelID:     "beacon", // Default channel
	}

	// Initialize transaction context
	err = stub.initializeContext()
	if err != nil {
		conn.Close()
		return nil, fmt.Errorf("failed to initialize context: %w", err)
	}

	return stub, nil
}

// Close closes the gRPC connection
func (s *ChaincodeStub) Close() error {
	if s.conn != nil {
		return s.conn.Close()
	}
	return nil
}

// Initialize transaction context from the Rust node
func (s *ChaincodeStub) initializeContext() error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Get transaction ID
	if s.transactionID == "" {
		resp, err := s.client.GetTransactionID(ctx, &pb.Empty{})
		if err != nil {
			return fmt.Errorf("failed to get transaction ID: %w", err)
		}
		s.transactionID = resp.TransactionId
	}

	// Get channel ID
	resp, err := s.client.GetChannelID(ctx, &pb.Empty{})
	if err != nil {
		return fmt.Errorf("failed to get channel ID: %w", err)
	}
	s.channelID = resp.ChannelId

	// Get creator
	creatorResp, err := s.client.GetCreator(ctx, &pb.Empty{})
	if err != nil {
		return fmt.Errorf("failed to get creator: %w", err)
	}
	s.creator = creatorResp.Creator

	// Get timestamp
	tsResp, err := s.client.GetTransactionTimestamp(ctx, &pb.Empty{})
	if err != nil {
		return fmt.Errorf("failed to get timestamp: %w", err)
	}
	s.timestamp = tsResp.Timestamp

	return nil
}

// GetState retrieves the value for a given key from the ledger
func (s *ChaincodeStub) GetState(key string) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	resp, err := s.client.GetState(ctx, &pb.GetStateRequest{Key: key})
	if err != nil {
		return nil, fmt.Errorf("failed to get state for key %s: %w", key, err)
	}

	if !resp.Found {
		return nil, nil // Key not found
	}

	return resp.Value, nil
}

// PutState saves the key-value pair to the ledger
func (s *ChaincodeStub) PutState(key string, value []byte) error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	resp, err := s.client.PutState(ctx, &pb.PutStateRequest{
		Key:   key,
		Value: value,
	})
	if err != nil {
		return fmt.Errorf("failed to put state for key %s: %w", key, err)
	}

	if !resp.Success {
		return fmt.Errorf("put state failed: %s", resp.Error)
	}

	return nil
}

// DelState removes the key from the ledger
func (s *ChaincodeStub) DelState(key string) error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	resp, err := s.client.DeleteState(ctx, &pb.DeleteStateRequest{Key: key})
	if err != nil {
		return fmt.Errorf("failed to delete state for key %s: %w", key, err)
	}

	if !resp.Success {
		return fmt.Errorf("delete state failed: %s", resp.Error)
	}

	return nil
}

// GetStateByRange returns a range query iterator for keys between startKey and endKey
func (s *ChaincodeStub) GetStateByRange(startKey, endKey string) (StateQueryIteratorInterface, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := s.client.GetStateByRange(ctx, &pb.GetStateByRangeRequest{
		StartKey: startKey,
		EndKey:   endKey,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get state by range: %w", err)
	}

	return &StateQueryIterator{
		results: resp.Results,
		index:   0,
	}, nil
}

// GetStateByPartialCompositeKey returns an iterator for keys matching the partial composite key
func (s *ChaincodeStub) GetStateByPartialCompositeKey(objectType string, keys []string) (StateQueryIteratorInterface, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	resp, err := s.client.GetStateByPartialCompositeKey(ctx, &pb.GetStateByPartialCompositeKeyRequest{
		ObjectType: objectType,
		Keys:       keys,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get state by partial composite key: %w", err)
	}

	return &StateQueryIterator{
		results: resp.Results,
		index:   0,
	}, nil
}

// GetTxID returns the transaction ID
func (s *ChaincodeStub) GetTxID() string {
	return s.transactionID
}

// GetChannelID returns the channel ID
func (s *ChaincodeStub) GetChannelID() string {
	return s.channelID
}

// GetCreator returns the creator of the transaction
func (s *ChaincodeStub) GetCreator() ([]byte, error) {
	return s.creator, nil
}

// GetTxTimestamp returns the transaction timestamp
func (s *ChaincodeStub) GetTxTimestamp() (*time.Time, error) {
	t := time.Unix(s.timestamp, 0)
	return &t, nil
}

// SetEvent sets an event for the transaction
func (s *ChaincodeStub) SetEvent(name string, payload []byte) error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	resp, err := s.client.SetEvent(ctx, &pb.SetEventRequest{
		Name:    name,
		Payload: payload,
	})
	if err != nil {
		return fmt.Errorf("failed to set event: %w", err)
	}

	if !resp.Success {
		return fmt.Errorf("set event failed: %s", resp.Error)
	}

	return nil
}

// LogMessage sends a log message to the Rust node
func (s *ChaincodeStub) LogMessage(level LogLevel, message string) error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	pbLevel := pb.LogLevel(level)
	resp, err := s.client.LogMessage(ctx, &pb.LogMessageRequest{
		Level:   pbLevel,
		Message: message,
	})
	if err != nil {
		return fmt.Errorf("failed to log message: %w", err)
	}

	if !resp.Success {
		return fmt.Errorf("log message failed")
	}

	return nil
}

// GetFunctionAndParameters returns the function name and parameters from command line
func (s *ChaincodeStub) GetFunctionAndParameters() (string, []string) {
	function := os.Getenv("BEACON_FUNCTION")
	if function == "" {
		function = "invoke" // Default function
	}

	args := os.Args[1:] // Skip program name
	return function, args
}

// GetStringArgs returns command line arguments as strings
func (s *ChaincodeStub) GetStringArgs() []string {
	return os.Args[1:]
}

// GetArgs returns command line arguments as byte arrays
func (s *ChaincodeStub) GetArgs() [][]byte {
	args := os.Args[1:]
	result := make([][]byte, len(args))
	for i, arg := range args {
		result[i] = []byte(arg)
	}
	return result
}

// StateQueryIterator implements StateQueryIteratorInterface
type StateQueryIterator struct {
	results []*pb.KeyValue
	index   int
}

// HasNext returns true if there are more results
func (iter *StateQueryIterator) HasNext() bool {
	return iter.index < len(iter.results)
}

// Next returns the next key-value pair
func (iter *StateQueryIterator) Next() (*KeyValue, error) {
	if !iter.HasNext() {
		return nil, fmt.Errorf("no more results")
	}

	result := iter.results[iter.index]
	iter.index++

	return &KeyValue{
		Key:   result.Key,
		Value: result.Value,
	}, nil
}

// Close closes the iterator (no-op for in-memory results)
func (iter *StateQueryIterator) Close() error {
	return nil
}

// Start is the main entry point for chaincodes
func Start(cc Chaincode) error {
	stub, err := NewChaincodeStub()
	if err != nil {
		log.Fatalf("Failed to create chaincode stub: %v", err)
	}
	defer stub.Close()

	// Determine if this is Init or Invoke based on environment
	function := os.Getenv("BEACON_FUNCTION")
	
	var response Response
	if function == "init" {
		response = cc.Init(stub)
	} else {
		response = cc.Invoke(stub)
	}

	// Log the response
	stub.LogMessage(INFO, fmt.Sprintf("Chaincode completed with status: %d, message: %s", 
		response.Status, response.Message))

	// Exit with appropriate code
	if response.Status == 200 {
		os.Exit(0)
	} else {
		os.Exit(1)
	}

	return nil
}
