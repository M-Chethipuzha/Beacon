package shim

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
)

// CompositeKey creates a composite key from object type and attributes
func CreateCompositeKey(objectType string, attributes []string) (string, error) {
	if objectType == "" {
		return "", fmt.Errorf("object type cannot be empty")
	}
	
	// Join with a delimiter that's unlikely to appear in normal keys
	key := objectType
	for _, attr := range attributes {
		if strings.Contains(attr, "\x00") {
			return "", fmt.Errorf("attribute cannot contain null character")
		}
		key += "\x00" + attr
	}
	
	return key, nil
}

// SplitCompositeKey splits a composite key into object type and attributes
func SplitCompositeKey(compositeKey string) (string, []string, error) {
	parts := strings.Split(compositeKey, "\x00")
	if len(parts) < 1 {
		return "", nil, fmt.Errorf("invalid composite key format")
	}
	
	objectType := parts[0]
	attributes := parts[1:]
	
	return objectType, attributes, nil
}

// Marshal converts a Go value to JSON bytes
func Marshal(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}

// Unmarshal converts JSON bytes to a Go value
func Unmarshal(data []byte, v interface{}) error {
	return json.Unmarshal(data, v)
}

// GetQueryResult represents a single result from a query
type GetQueryResult struct {
	Key   string      `json:"key"`
	Value interface{} `json:"value"`
}

// IteratorToArray converts a StateQueryIterator to an array of results
func IteratorToArray(iterator StateQueryIteratorInterface) ([]*GetQueryResult, error) {
	var results []*GetQueryResult
	
	defer iterator.Close()
	
	for iterator.HasNext() {
		kv, err := iterator.Next()
		if err != nil {
			return nil, fmt.Errorf("error iterating results: %w", err)
		}
		
		var value interface{}
		if len(kv.Value) > 0 {
			err = json.Unmarshal(kv.Value, &value)
			if err != nil {
				// If it's not JSON, treat as raw bytes
				value = kv.Value
			}
		}
		
		results = append(results, &GetQueryResult{
			Key:   kv.Key,
			Value: value,
		})
	}
	
	return results, nil
}

// GetStateAsString retrieves state and converts to string
func GetStateAsString(stub ChaincodeStubInterface, key string) (string, error) {
	value, err := stub.GetState(key)
	if err != nil {
		return "", err
	}
	if value == nil {
		return "", nil
	}
	return string(value), nil
}

// GetStateAsInt retrieves state and converts to integer
func GetStateAsInt(stub ChaincodeStubInterface, key string) (int, error) {
	value, err := stub.GetState(key)
	if err != nil {
		return 0, err
	}
	if value == nil {
		return 0, nil
	}
	
	return strconv.Atoi(string(value))
}

// PutStateAsString saves a string value to state
func PutStateAsString(stub ChaincodeStubInterface, key, value string) error {
	return stub.PutState(key, []byte(value))
}

// PutStateAsInt saves an integer value to state
func PutStateAsInt(stub ChaincodeStubInterface, key string, value int) error {
	return stub.PutState(key, []byte(strconv.Itoa(value)))
}

// PutStateAsJSON saves a Go value as JSON to state
func PutStateAsJSON(stub ChaincodeStubInterface, key string, value interface{}) error {
	jsonBytes, err := json.Marshal(value)
	if err != nil {
		return fmt.Errorf("failed to marshal to JSON: %w", err)
	}
	return stub.PutState(key, jsonBytes)
}

// GetStateAsJSON retrieves state and unmarshals from JSON
func GetStateAsJSON(stub ChaincodeStubInterface, key string, target interface{}) error {
	value, err := stub.GetState(key)
	if err != nil {
		return err
	}
	if value == nil {
		return fmt.Errorf("key not found: %s", key)
	}
	
	return json.Unmarshal(value, target)
}

// ValidateArgs checks if the required number of arguments are provided
func ValidateArgs(args []string, expected int) error {
	if len(args) != expected {
		return fmt.Errorf("expected %d arguments, got %d", expected, len(args))
	}
	return nil
}

// ValidateArgsRange checks if the number of arguments is within a range
func ValidateArgsRange(args []string, min, max int) error {
	if len(args) < min || len(args) > max {
		return fmt.Errorf("expected %d-%d arguments, got %d", min, max, len(args))
	}
	return nil
}

// ParseBool safely parses a string to boolean
func ParseBool(s string) (bool, error) {
	return strconv.ParseBool(s)
}

// ParseInt safely parses a string to integer
func ParseInt(s string) (int, error) {
	return strconv.Atoi(s)
}

// ParseFloat safely parses a string to float64
func ParseFloat(s string) (float64, error) {
	return strconv.ParseFloat(s, 64)
}

// StringInSlice checks if a string exists in a slice
func StringInSlice(str string, slice []string) bool {
	for _, s := range slice {
		if s == str {
			return true
		}
	}
	return false
}

// UniqueStrings removes duplicate strings from a slice
func UniqueStrings(slice []string) []string {
	keys := make(map[string]bool)
	var result []string
	
	for _, str := range slice {
		if !keys[str] {
			keys[str] = true
			result = append(result, str)
		}
	}
	
	return result
}
