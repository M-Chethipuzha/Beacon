package main

import (
	"encoding/json"
	"fmt"
	"log"
	"strconv"
	"time"

	"github.com/beacon-blockchain/sdk-go/shim"
)

// GatewayManagementChaincode implements the Chaincode interface for gateway management
type GatewayManagementChaincode struct{}

// Gateway represents a registered gateway in the network
type Gateway struct {
	ID              string            `json:"id"`
	PublicKey       string            `json:"publicKey"`
	OrganizationID  string            `json:"organizationID"`
	Status          string            `json:"status"`
	RegistrationTime int64            `json:"registrationTime"`
	LastHeartbeat   int64            `json:"lastHeartbeat"`
	Metadata        map[string]string `json:"metadata"`
}

// AccessPolicy represents an access control policy
type AccessPolicy struct {
	ID          string   `json:"id"`
	Name        string   `json:"name"`
	Description string   `json:"description"`
	Rules       []Rule   `json:"rules"`
	CreatedAt   int64    `json:"createdAt"`
	UpdatedAt   int64    `json:"updatedAt"`
	Version     int      `json:"version"`
}

// Rule represents a single access control rule
type Rule struct {
	Resource   string   `json:"resource"`
	Action     string   `json:"action"`
	Principals []string `json:"principals"`
	Conditions []string `json:"conditions"`
}

// AuditLog represents an audit log entry
type AuditLog struct {
	ID            string `json:"id"`
	Timestamp     int64  `json:"timestamp"`
	GatewayID     string `json:"gatewayID"`
	Action        string `json:"action"`
	Resource      string `json:"resource"`
	Success       bool   `json:"success"`
	ErrorMessage  string `json:"errorMessage,omitempty"`
	TransactionID string `json:"transactionID"`
}

// Init initializes the chaincode
func (cc *GatewayManagementChaincode) Init(stub shim.ChaincodeStubInterface) shim.Response {
	log.Println("Initializing Gateway Management Chaincode")
	
	// Initialize default configuration
	config := map[string]interface{}{
		"version":                "1.0.0",
		"maxGateways":           1000,
		"heartbeatTimeout":      300, // 5 minutes
		"policyVersioning":      true,
		"auditLoggingEnabled":   true,
	}
	
	err := shim.PutStateAsJSON(stub, "config:system", config)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to initialize configuration: %s", err.Error()))
	}
	
	// Create initial admin policy
	adminPolicy := AccessPolicy{
		ID:          "policy:admin",
		Name:        "Administrator Policy",
		Description: "Full administrative access to all resources",
		Rules: []Rule{
			{
				Resource:   "*",
				Action:     "*",
				Principals: []string{"admin"},
				Conditions: []string{},
			},
		},
		CreatedAt: time.Now().Unix(),
		UpdatedAt: time.Now().Unix(),
		Version:   1,
	}
	
	err = shim.PutStateAsJSON(stub, "policy:admin", adminPolicy)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to create admin policy: %s", err.Error()))
	}
	
	return shim.Success([]byte("Gateway Management Chaincode initialized successfully"))
}

// Invoke handles chaincode invocations
func (cc *GatewayManagementChaincode) Invoke(stub shim.ChaincodeStubInterface) shim.Response {
	function, args := stub.GetFunctionAndParameters()
	
	switch function {
	case "registerGateway":
		return cc.registerGateway(stub, args)
	case "updateGateway":
		return cc.updateGateway(stub, args)
	case "getGateway":
		return cc.getGateway(stub, args)
	case "listGateways":
		return cc.listGateways(stub, args)
	case "deactivateGateway":
		return cc.deactivateGateway(stub, args)
	case "heartbeat":
		return cc.heartbeat(stub, args)
	case "createPolicy":
		return cc.createPolicy(stub, args)
	case "updatePolicy":
		return cc.updatePolicy(stub, args)
	case "getPolicy":
		return cc.getPolicy(stub, args)
	case "listPolicies":
		return cc.listPolicies(stub, args)
	case "auditLog":
		return cc.auditLog(stub, args)
	case "queryAuditLogs":
		return cc.queryAuditLogs(stub, args)
	default:
		return shim.Error(fmt.Sprintf("Unknown function: %s", function))
	}
}

// registerGateway registers a new gateway in the network
func (cc *GatewayManagementChaincode) registerGateway(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 3); err != nil {
		return shim.Error(err.Error())
	}
	
	gatewayID := args[0]
	publicKey := args[1]
	organizationID := args[2]
	
	// Check if gateway already exists
	existing, err := stub.GetState("gateway:" + gatewayID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to check existing gateway: %s", err.Error()))
	}
	if existing != nil {
		return shim.Error(fmt.Sprintf("Gateway already exists: %s", gatewayID))
	}
	
	// Create new gateway
	gateway := Gateway{
		ID:              gatewayID,
		PublicKey:       publicKey,
		OrganizationID:  organizationID,
		Status:          "active",
		RegistrationTime: time.Now().Unix(),
		LastHeartbeat:   time.Now().Unix(),
		Metadata:        make(map[string]string),
	}
	
	// Store gateway
	err = shim.PutStateAsJSON(stub, "gateway:"+gatewayID, gateway)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store gateway: %s", err.Error()))
	}
	
	// Log audit event
	cc.logAudit(stub, gatewayID, "REGISTER_GATEWAY", "gateway:"+gatewayID, true, "")
	
	// Emit event
	eventPayload, _ := json.Marshal(map[string]interface{}{
		"action":    "gateway_registered",
		"gatewayID": gatewayID,
		"orgID":     organizationID,
	})
	stub.SetEvent("GatewayRegistered", eventPayload)
	
	return shim.Success([]byte(fmt.Sprintf("Gateway %s registered successfully", gatewayID)))
}

// updateGateway updates gateway metadata
func (cc *GatewayManagementChaincode) updateGateway(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgsRange(args, 2, 10); err != nil {
		return shim.Error(err.Error())
	}
	
	gatewayID := args[0]
	
	// Get existing gateway
	var gateway Gateway
	err := shim.GetStateAsJSON(stub, "gateway:"+gatewayID, &gateway)
	if err != nil {
		return shim.Error(fmt.Sprintf("Gateway not found: %s", gatewayID))
	}
	
	// Update metadata from key-value pairs in args
	for i := 1; i < len(args); i += 2 {
		if i+1 < len(args) {
			key := args[i]
			value := args[i+1]
			gateway.Metadata[key] = value
		}
	}
	
	// Update gateway
	err = shim.PutStateAsJSON(stub, "gateway:"+gatewayID, gateway)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update gateway: %s", err.Error()))
	}
	
	// Log audit event
	cc.logAudit(stub, gatewayID, "UPDATE_GATEWAY", "gateway:"+gatewayID, true, "")
	
	return shim.Success([]byte(fmt.Sprintf("Gateway %s updated successfully", gatewayID)))
}

// getGateway retrieves a gateway by ID
func (cc *GatewayManagementChaincode) getGateway(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	gatewayID := args[0]
	
	gatewayBytes, err := stub.GetState("gateway:" + gatewayID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get gateway: %s", err.Error()))
	}
	if gatewayBytes == nil {
		return shim.Error(fmt.Sprintf("Gateway not found: %s", gatewayID))
	}
	
	return shim.Success(gatewayBytes)
}

// listGateways returns all gateways with optional status filter
func (cc *GatewayManagementChaincode) listGateways(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	statusFilter := ""
	if len(args) > 0 {
		statusFilter = args[0]
	}
	
	iterator, err := stub.GetStateByRange("gateway:", "gateway:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get gateways: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var filteredGateways []Gateway
	for _, result := range results {
		var gateway Gateway
		gatewayBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(gatewayBytes, &gateway)
		
		if statusFilter == "" || gateway.Status == statusFilter {
			filteredGateways = append(filteredGateways, gateway)
		}
	}
	
	responseBytes, err := json.Marshal(filteredGateways)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// deactivateGateway deactivates a gateway
func (cc *GatewayManagementChaincode) deactivateGateway(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	gatewayID := args[0]
	
	// Get existing gateway
	var gateway Gateway
	err := shim.GetStateAsJSON(stub, "gateway:"+gatewayID, &gateway)
	if err != nil {
		return shim.Error(fmt.Sprintf("Gateway not found: %s", gatewayID))
	}
	
	// Update status
	gateway.Status = "inactive"
	
	// Store updated gateway
	err = shim.PutStateAsJSON(stub, "gateway:"+gatewayID, gateway)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to deactivate gateway: %s", err.Error()))
	}
	
	// Log audit event
	cc.logAudit(stub, gatewayID, "DEACTIVATE_GATEWAY", "gateway:"+gatewayID, true, "")
	
	return shim.Success([]byte(fmt.Sprintf("Gateway %s deactivated successfully", gatewayID)))
}

// heartbeat updates the last heartbeat timestamp for a gateway
func (cc *GatewayManagementChaincode) heartbeat(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	gatewayID := args[0]
	
	// Get existing gateway
	var gateway Gateway
	err := shim.GetStateAsJSON(stub, "gateway:"+gatewayID, &gateway)
	if err != nil {
		return shim.Error(fmt.Sprintf("Gateway not found: %s", gatewayID))
	}
	
	// Update heartbeat
	gateway.LastHeartbeat = time.Now().Unix()
	
	// Store updated gateway
	err = shim.PutStateAsJSON(stub, "gateway:"+gatewayID, gateway)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update heartbeat: %s", err.Error()))
	}
	
	return shim.Success([]byte("Heartbeat updated"))
}

// createPolicy creates a new access policy
func (cc *GatewayManagementChaincode) createPolicy(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 4); err != nil {
		return shim.Error(err.Error())
	}
	
	policyID := args[0]
	name := args[1]
	description := args[2]
	rulesJSON := args[3]
	
	// Check if policy already exists
	existing, err := stub.GetState("policy:" + policyID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to check existing policy: %s", err.Error()))
	}
	if existing != nil {
		return shim.Error(fmt.Sprintf("Policy already exists: %s", policyID))
	}
	
	// Parse rules
	var rules []Rule
	err = json.Unmarshal([]byte(rulesJSON), &rules)
	if err != nil {
		return shim.Error(fmt.Sprintf("Invalid rules JSON: %s", err.Error()))
	}
	
	// Create new policy
	policy := AccessPolicy{
		ID:          policyID,
		Name:        name,
		Description: description,
		Rules:       rules,
		CreatedAt:   time.Now().Unix(),
		UpdatedAt:   time.Now().Unix(),
		Version:     1,
	}
	
	// Store policy
	err = shim.PutStateAsJSON(stub, "policy:"+policyID, policy)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store policy: %s", err.Error()))
	}
	
	// Log audit event
	cc.logAudit(stub, "", "CREATE_POLICY", "policy:"+policyID, true, "")
	
	return shim.Success([]byte(fmt.Sprintf("Policy %s created successfully", policyID)))
}

// updatePolicy updates an existing access policy
func (cc *GatewayManagementChaincode) updatePolicy(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 2); err != nil {
		return shim.Error(err.Error())
	}
	
	policyID := args[0]
	rulesJSON := args[1]
	
	// Get existing policy
	var policy AccessPolicy
	err := shim.GetStateAsJSON(stub, "policy:"+policyID, &policy)
	if err != nil {
		return shim.Error(fmt.Sprintf("Policy not found: %s", policyID))
	}
	
	// Parse new rules
	var rules []Rule
	err = json.Unmarshal([]byte(rulesJSON), &rules)
	if err != nil {
		return shim.Error(fmt.Sprintf("Invalid rules JSON: %s", err.Error()))
	}
	
	// Update policy
	policy.Rules = rules
	policy.UpdatedAt = time.Now().Unix()
	policy.Version++
	
	// Store updated policy
	err = shim.PutStateAsJSON(stub, "policy:"+policyID, policy)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update policy: %s", err.Error()))
	}
	
	// Log audit event
	cc.logAudit(stub, "", "UPDATE_POLICY", "policy:"+policyID, true, "")
	
	return shim.Success([]byte(fmt.Sprintf("Policy %s updated successfully", policyID)))
}

// getPolicy retrieves a policy by ID
func (cc *GatewayManagementChaincode) getPolicy(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	policyID := args[0]
	
	policyBytes, err := stub.GetState("policy:" + policyID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get policy: %s", err.Error()))
	}
	if policyBytes == nil {
		return shim.Error(fmt.Sprintf("Policy not found: %s", policyID))
	}
	
	return shim.Success(policyBytes)
}

// listPolicies returns all policies
func (cc *GatewayManagementChaincode) listPolicies(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	iterator, err := stub.GetStateByRange("policy:", "policy:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get policies: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var policies []AccessPolicy
	for _, result := range results {
		var policy AccessPolicy
		policyBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(policyBytes, &policy)
		policies = append(policies, policy)
	}
	
	responseBytes, err := json.Marshal(policies)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// auditLog creates an audit log entry
func (cc *GatewayManagementChaincode) auditLog(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgsRange(args, 4, 5); err != nil {
		return shim.Error(err.Error())
	}
	
	gatewayID := args[0]
	action := args[1]
	resource := args[2]
	successStr := args[3]
	errorMessage := ""
	if len(args) > 4 {
		errorMessage = args[4]
	}
	
	success, err := strconv.ParseBool(successStr)
	if err != nil {
		return shim.Error(fmt.Sprintf("Invalid success value: %s", successStr))
	}
	
	return cc.logAuditResponse(stub, gatewayID, action, resource, success, errorMessage)
}

// queryAuditLogs queries audit logs with optional filters
func (cc *GatewayManagementChaincode) queryAuditLogs(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	gatewayFilter := ""
	actionFilter := ""
	
	if len(args) > 0 {
		gatewayFilter = args[0]
	}
	if len(args) > 1 {
		actionFilter = args[1]
	}
	
	iterator, err := stub.GetStateByRange("audit:", "audit:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get audit logs: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var auditLogs []AuditLog
	for _, result := range results {
		var auditLog AuditLog
		logBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(logBytes, &auditLog)
		
		// Apply filters
		if gatewayFilter != "" && auditLog.GatewayID != gatewayFilter {
			continue
		}
		if actionFilter != "" && auditLog.Action != actionFilter {
			continue
		}
		
		auditLogs = append(auditLogs, auditLog)
	}
	
	responseBytes, err := json.Marshal(auditLogs)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// Helper function to log audit events
func (cc *GatewayManagementChaincode) logAudit(stub shim.ChaincodeStubInterface, gatewayID, action, resource string, success bool, errorMessage string) {
	auditID := fmt.Sprintf("audit:%d:%s", time.Now().UnixNano(), stub.GetTxID())
	
	auditLog := AuditLog{
		ID:            auditID,
		Timestamp:     time.Now().Unix(),
		GatewayID:     gatewayID,
		Action:        action,
		Resource:      resource,
		Success:       success,
		ErrorMessage:  errorMessage,
		TransactionID: stub.GetTxID(),
	}
	
	shim.PutStateAsJSON(stub, auditID, auditLog)
}

// Helper function to log audit events and return response
func (cc *GatewayManagementChaincode) logAuditResponse(stub shim.ChaincodeStubInterface, gatewayID, action, resource string, success bool, errorMessage string) shim.Response {
	auditID := fmt.Sprintf("audit:%d:%s", time.Now().UnixNano(), stub.GetTxID())
	
	auditLog := AuditLog{
		ID:            auditID,
		Timestamp:     time.Now().Unix(),
		GatewayID:     gatewayID,
		Action:        action,
		Resource:      resource,
		Success:       success,
		ErrorMessage:  errorMessage,
		TransactionID: stub.GetTxID(),
	}
	
	err := shim.PutStateAsJSON(stub, auditID, auditLog)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to log audit entry: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Audit log created: %s", auditID)))
}

// main function - entry point for the chaincode
func main() {
	err := shim.Start(new(GatewayManagementChaincode))
	if err != nil {
		log.Fatalf("Error starting Gateway Management Chaincode: %v", err)
	}
}
