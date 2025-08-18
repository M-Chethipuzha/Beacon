package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"strconv"
	"time"

	"github.com/beacon-blockchain/sdk-go/shim"
)

// IdentityVerificationChaincode implements identity verification and credential management
type IdentityVerificationChaincode struct{}

// Identity represents a digital identity
type Identity struct {
	ID            string                 `json:"id"`
	PublicKey     string                 `json:"publicKey"`
	Type          string                 `json:"type"`
	Organization  string                 `json:"organization"`
	Status        string                 `json:"status"`
	CreatedAt     int64                  `json:"createdAt"`
	UpdatedAt     int64                  `json:"updatedAt"`
	ExpiresAt     int64                  `json:"expiresAt,omitempty"`
	Attributes    map[string]interface{} `json:"attributes"`
	Metadata      map[string]interface{} `json:"metadata"`
}

// Credential represents a verifiable credential
type Credential struct {
	ID          string                 `json:"id"`
	Type        string                 `json:"type"`
	Issuer      string                 `json:"issuer"`
	Subject     string                 `json:"subject"`
	IssuedAt    int64                  `json:"issuedAt"`
	ExpiresAt   int64                  `json:"expiresAt,omitempty"`
	Status      string                 `json:"status"`
	Claims      map[string]interface{} `json:"claims"`
	Proof       Proof                  `json:"proof"`
	Metadata    map[string]interface{} `json:"metadata"`
}

// Proof represents a cryptographic proof
type Proof struct {
	Type               string `json:"type"`
	Created            int64  `json:"created"`
	VerificationMethod string `json:"verificationMethod"`
	ProofPurpose       string `json:"proofPurpose"`
	ProofValue         string `json:"proofValue"`
}

// VerificationRequest represents a verification request
type VerificationRequest struct {
	ID           string                 `json:"id"`
	RequesterID  string                 `json:"requesterID"`
	SubjectID    string                 `json:"subjectID"`
	CredentialID string                 `json:"credentialID"`
	Purpose      string                 `json:"purpose"`
	Status       string                 `json:"status"`
	RequestedAt  int64                  `json:"requestedAt"`
	RespondedAt  int64                  `json:"respondedAt,omitempty"`
	Result       map[string]interface{} `json:"result,omitempty"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// RevocationRecord represents a credential revocation
type RevocationRecord struct {
	ID           string                 `json:"id"`
	CredentialID string                 `json:"credentialID"`
	Issuer       string                 `json:"issuer"`
	Reason       string                 `json:"reason"`
	RevokedAt    int64                  `json:"revokedAt"`
	Status       string                 `json:"status"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// Init initializes the chaincode
func (cc *IdentityVerificationChaincode) Init(stub shim.ChaincodeStubInterface) shim.Response {
	log.Println("Initializing Identity Verification Chaincode")
	
	// Initialize system configuration
	config := map[string]interface{}{
		"version":                    "1.0.0",
		"maxIdentities":              100000,
		"maxCredentials":             500000,
		"defaultCredentialValidity":  86400 * 365, // 1 year in seconds
		"supportedCredentialTypes":   []string{"academic", "professional", "certification", "authorization"},
		"supportedProofTypes":        []string{"Ed25519Signature2020", "RsaSignature2018", "EcdsaSecp256k1Signature2019"},
		"autoVerificationEnabled":    true,
		"revocationEnabled":          true,
	}
	
	err := shim.PutStateAsJSON(stub, "config:system", config)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to initialize configuration: %s", err.Error()))
	}
	
	// Create root authority identity
	rootIdentity := Identity{
		ID:           "root-authority",
		PublicKey:    "root-public-key-placeholder",
		Type:         "authority",
		Organization: "BEACON Network",
		Status:       "active",
		CreatedAt:    time.Now().Unix(),
		UpdatedAt:    time.Now().Unix(),
		Attributes:   make(map[string]interface{}),
		Metadata:     make(map[string]interface{}),
	}
	
	err = shim.PutStateAsJSON(stub, "identity:root-authority", rootIdentity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to create root authority: %s", err.Error()))
	}
	
	return shim.Success([]byte("Identity Verification Chaincode initialized successfully"))
}

// Invoke handles chaincode invocations
func (cc *IdentityVerificationChaincode) Invoke(stub shim.ChaincodeStubInterface) shim.Response {
	function, args := stub.GetFunctionAndParameters()
	
	switch function {
	// Identity operations
	case "createIdentity":
		return cc.createIdentity(stub, args)
	case "updateIdentity":
		return cc.updateIdentity(stub, args)
	case "getIdentity":
		return cc.getIdentity(stub, args)
	case "listIdentities":
		return cc.listIdentities(stub, args)
	case "revokeIdentity":
		return cc.revokeIdentity(stub, args)
	
	// Credential operations
	case "issueCredential":
		return cc.issueCredential(stub, args)
	case "verifyCredential":
		return cc.verifyCredential(stub, args)
	case "getCredential":
		return cc.getCredential(stub, args)
	case "listCredentials":
		return cc.listCredentials(stub, args)
	case "revokeCredential":
		return cc.revokeCredential(stub, args)
	
	// Verification operations
	case "requestVerification":
		return cc.requestVerification(stub, args)
	case "respondToVerification":
		return cc.respondToVerification(stub, args)
	case "getVerificationRequest":
		return cc.getVerificationRequest(stub, args)
	case "listVerificationRequests":
		return cc.listVerificationRequests(stub, args)
	
	// Revocation operations
	case "checkRevocationStatus":
		return cc.checkRevocationStatus(stub, args)
	case "listRevocations":
		return cc.listRevocations(stub, args)
	
	default:
		return shim.Error(fmt.Sprintf("Unknown function: %s", function))
	}
}

// createIdentity creates a new digital identity
func (cc *IdentityVerificationChaincode) createIdentity(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 4); err != nil {
		return shim.Error(err.Error())
	}
	
	identityID := args[0]
	publicKey := args[1]
	identityType := args[2]
	organization := args[3]
	
	// Check if identity already exists
	existing, err := stub.GetState("identity:" + identityID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to check existing identity: %s", err.Error()))
	}
	if existing != nil {
		return shim.Error(fmt.Sprintf("Identity already exists: %s", identityID))
	}
	
	// Create new identity
	identity := Identity{
		ID:           identityID,
		PublicKey:    publicKey,
		Type:         identityType,
		Organization: organization,
		Status:       "active",
		CreatedAt:    time.Now().Unix(),
		UpdatedAt:    time.Now().Unix(),
		Attributes:   make(map[string]interface{}),
		Metadata:     make(map[string]interface{}),
	}
	
	// Set expiration for non-authority identities
	if identityType != "authority" {
		identity.ExpiresAt = time.Now().Unix() + 86400*365 // 1 year
	}
	
	// Store identity
	err = shim.PutStateAsJSON(stub, "identity:"+identityID, identity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store identity: %s", err.Error()))
	}
	
	// Emit event
	eventPayload, _ := json.Marshal(map[string]interface{}{
		"action":       "identity_created",
		"identityID":   identityID,
		"type":         identityType,
		"organization": organization,
	})
	stub.SetEvent("IdentityCreated", eventPayload)
	
	return shim.Success([]byte(fmt.Sprintf("Identity %s created successfully", identityID)))
}

// updateIdentity updates identity attributes
func (cc *IdentityVerificationChaincode) updateIdentity(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgsRange(args, 1, 20); err != nil {
		return shim.Error(err.Error())
	}
	
	identityID := args[0]
	
	// Get existing identity
	var identity Identity
	err := shim.GetStateAsJSON(stub, "identity:"+identityID, &identity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Identity not found: %s", identityID))
	}
	
	// Update attributes from key-value pairs
	for i := 1; i < len(args); i += 2 {
		if i+1 < len(args) {
			key := args[i]
			value := args[i+1]
			identity.Attributes[key] = value
		}
	}
	
	identity.UpdatedAt = time.Now().Unix()
	
	// Store updated identity
	err = shim.PutStateAsJSON(stub, "identity:"+identityID, identity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update identity: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Identity %s updated successfully", identityID)))
}

// getIdentity retrieves an identity by ID
func (cc *IdentityVerificationChaincode) getIdentity(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	identityID := args[0]
	
	identityBytes, err := stub.GetState("identity:" + identityID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get identity: %s", err.Error()))
	}
	if identityBytes == nil {
		return shim.Error(fmt.Sprintf("Identity not found: %s", identityID))
	}
	
	return shim.Success(identityBytes)
}

// listIdentities returns all identities with optional type filter
func (cc *IdentityVerificationChaincode) listIdentities(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	typeFilter := ""
	if len(args) > 0 {
		typeFilter = args[0]
	}
	
	iterator, err := stub.GetStateByRange("identity:", "identity:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get identities: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var filteredIdentities []Identity
	for _, result := range results {
		var identity Identity
		identityBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(identityBytes, &identity)
		
		if typeFilter == "" || identity.Type == typeFilter {
			filteredIdentities = append(filteredIdentities, identity)
		}
	}
	
	responseBytes, err := json.Marshal(filteredIdentities)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// revokeIdentity revokes an identity
func (cc *IdentityVerificationChaincode) revokeIdentity(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 2); err != nil {
		return shim.Error(err.Error())
	}
	
	identityID := args[0]
	reason := args[1]
	
	// Get existing identity
	var identity Identity
	err := shim.GetStateAsJSON(stub, "identity:"+identityID, &identity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Identity not found: %s", identityID))
	}
	
	// Update status
	identity.Status = "revoked"
	identity.UpdatedAt = time.Now().Unix()
	if identity.Metadata == nil {
		identity.Metadata = make(map[string]interface{})
	}
	identity.Metadata["revocationReason"] = reason
	identity.Metadata["revokedAt"] = time.Now().Unix()
	
	// Store updated identity
	err = shim.PutStateAsJSON(stub, "identity:"+identityID, identity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to revoke identity: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Identity %s revoked successfully", identityID)))
}

// issueCredential issues a new verifiable credential
func (cc *IdentityVerificationChaincode) issueCredential(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgsRange(args, 6, 7); err != nil {
		return shim.Error(err.Error())
	}
	
	credentialID := args[0]
	credentialType := args[1]
	issuer := args[2]
	subject := args[3]
	claimsJSON := args[4]
	proofValue := args[5]
	
	var expirationDays int64 = 365 // Default 1 year
	if len(args) > 6 {
		var err error
		expirationDays, err = strconv.ParseInt(args[6], 10, 64)
		if err != nil {
			return shim.Error(fmt.Sprintf("Invalid expiration days: %s", args[6]))
		}
	}
	
	// Verify issuer exists and is active
	var issuerIdentity Identity
	err := shim.GetStateAsJSON(stub, "identity:"+issuer, &issuerIdentity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Issuer identity not found: %s", issuer))
	}
	if issuerIdentity.Status != "active" {
		return shim.Error(fmt.Sprintf("Issuer identity is not active: %s", issuer))
	}
	
	// Verify subject exists
	var subjectIdentity Identity
	err = shim.GetStateAsJSON(stub, "identity:"+subject, &subjectIdentity)
	if err != nil {
		return shim.Error(fmt.Sprintf("Subject identity not found: %s", subject))
	}
	
	// Parse claims
	var claims map[string]interface{}
	err = json.Unmarshal([]byte(claimsJSON), &claims)
	if err != nil {
		return shim.Error(fmt.Sprintf("Invalid claims JSON: %s", err.Error()))
	}
	
	// Create proof
	proof := Proof{
		Type:               "Ed25519Signature2020",
		Created:            time.Now().Unix(),
		VerificationMethod: issuerIdentity.PublicKey,
		ProofPurpose:       "assertionMethod",
		ProofValue:         proofValue,
	}
	
	// Create credential
	credential := Credential{
		ID:        credentialID,
		Type:      credentialType,
		Issuer:    issuer,
		Subject:   subject,
		IssuedAt:  time.Now().Unix(),
		ExpiresAt: time.Now().Unix() + (expirationDays * 86400),
		Status:    "active",
		Claims:    claims,
		Proof:     proof,
		Metadata:  make(map[string]interface{}),
	}
	
	// Store credential
	err = shim.PutStateAsJSON(stub, "credential:"+credentialID, credential)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store credential: %s", err.Error()))
	}
	
	// Emit event
	eventPayload, _ := json.Marshal(map[string]interface{}{
		"action":       "credential_issued",
		"credentialID": credentialID,
		"type":         credentialType,
		"issuer":       issuer,
		"subject":      subject,
	})
	stub.SetEvent("CredentialIssued", eventPayload)
	
	return shim.Success([]byte(fmt.Sprintf("Credential %s issued successfully", credentialID)))
}

// verifyCredential verifies a credential's authenticity and validity
func (cc *IdentityVerificationChaincode) verifyCredential(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	credentialID := args[0]
	
	// Get credential
	var credential Credential
	err := shim.GetStateAsJSON(stub, "credential:"+credentialID, &credential)
	if err != nil {
		return shim.Error(fmt.Sprintf("Credential not found: %s", credentialID))
	}
	
	// Check if credential is revoked
	revocationStatus := cc.checkRevocationStatusInternal(stub, credentialID)
	if revocationStatus {
		return cc.createVerificationResult(false, "Credential has been revoked", map[string]interface{}{
			"credentialID": credentialID,
			"status": "revoked",
		})
	}
	
	// Check expiration
	if credential.ExpiresAt > 0 && credential.ExpiresAt < time.Now().Unix() {
		return cc.createVerificationResult(false, "Credential has expired", map[string]interface{}{
			"credentialID": credentialID,
			"status": "expired",
			"expiresAt": credential.ExpiresAt,
		})
	}
	
	// Verify issuer is still active
	var issuer Identity
	err = shim.GetStateAsJSON(stub, "identity:"+credential.Issuer, &issuer)
	if err != nil || issuer.Status != "active" {
		return cc.createVerificationResult(false, "Issuer is not active", map[string]interface{}{
			"credentialID": credentialID,
			"issuer": credential.Issuer,
		})
	}
	
	// Verify signature (simplified - in real implementation, would verify cryptographic signature)
	expectedHash := cc.generateCredentialHash(credential)
	if credential.Proof.ProofValue == expectedHash {
		return cc.createVerificationResult(true, "Credential is valid", map[string]interface{}{
			"credentialID": credentialID,
			"type": credential.Type,
			"issuer": credential.Issuer,
			"subject": credential.Subject,
			"verifiedAt": time.Now().Unix(),
		})
	}
	
	return cc.createVerificationResult(false, "Invalid credential signature", map[string]interface{}{
		"credentialID": credentialID,
	})
}

// getCredential retrieves a credential by ID
func (cc *IdentityVerificationChaincode) getCredential(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	credentialID := args[0]
	
	credentialBytes, err := stub.GetState("credential:" + credentialID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get credential: %s", err.Error()))
	}
	if credentialBytes == nil {
		return shim.Error(fmt.Sprintf("Credential not found: %s", credentialID))
	}
	
	return shim.Success(credentialBytes)
}

// listCredentials returns credentials with optional filters
func (cc *IdentityVerificationChaincode) listCredentials(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	var subjectFilter, issuerFilter, typeFilter string
	
	if len(args) > 0 {
		subjectFilter = args[0]
	}
	if len(args) > 1 {
		issuerFilter = args[1]
	}
	if len(args) > 2 {
		typeFilter = args[2]
	}
	
	iterator, err := stub.GetStateByRange("credential:", "credential:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get credentials: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var filteredCredentials []Credential
	for _, result := range results {
		var credential Credential
		credentialBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(credentialBytes, &credential)
		
		// Apply filters
		if subjectFilter != "" && credential.Subject != subjectFilter {
			continue
		}
		if issuerFilter != "" && credential.Issuer != issuerFilter {
			continue
		}
		if typeFilter != "" && credential.Type != typeFilter {
			continue
		}
		
		filteredCredentials = append(filteredCredentials, credential)
	}
	
	responseBytes, err := json.Marshal(filteredCredentials)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// revokeCredential revokes a credential
func (cc *IdentityVerificationChaincode) revokeCredential(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 3); err != nil {
		return shim.Error(err.Error())
	}
	
	credentialID := args[0]
	issuer := args[1]
	reason := args[2]
	
	// Get existing credential
	var credential Credential
	err := shim.GetStateAsJSON(stub, "credential:"+credentialID, &credential)
	if err != nil {
		return shim.Error(fmt.Sprintf("Credential not found: %s", credentialID))
	}
	
	// Verify issuer authority
	if credential.Issuer != issuer {
		return shim.Error(fmt.Sprintf("Only the issuer can revoke this credential"))
	}
	
	// Create revocation record
	revocationID := fmt.Sprintf("revocation:%s:%d", credentialID, time.Now().UnixNano())
	revocation := RevocationRecord{
		ID:           revocationID,
		CredentialID: credentialID,
		Issuer:       issuer,
		Reason:       reason,
		RevokedAt:    time.Now().Unix(),
		Status:       "active",
		Metadata:     make(map[string]interface{}),
	}
	
	// Store revocation record
	err = shim.PutStateAsJSON(stub, revocationID, revocation)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store revocation record: %s", err.Error()))
	}
	
	// Update credential status
	credential.Status = "revoked"
	credential.Metadata["revokedAt"] = time.Now().Unix()
	credential.Metadata["revocationReason"] = reason
	
	err = shim.PutStateAsJSON(stub, "credential:"+credentialID, credential)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update credential: %s", err.Error()))
	}
	
	// Emit event
	eventPayload, _ := json.Marshal(map[string]interface{}{
		"action":       "credential_revoked",
		"credentialID": credentialID,
		"issuer":       issuer,
		"reason":       reason,
	})
	stub.SetEvent("CredentialRevoked", eventPayload)
	
	return shim.Success([]byte(fmt.Sprintf("Credential %s revoked successfully", credentialID)))
}

// requestVerification creates a verification request
func (cc *IdentityVerificationChaincode) requestVerification(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 5); err != nil {
		return shim.Error(err.Error())
	}
	
	requestID := args[0]
	requesterID := args[1]
	subjectID := args[2]
	credentialID := args[3]
	purpose := args[4]
	
	// Verify requester exists
	var requester Identity
	err := shim.GetStateAsJSON(stub, "identity:"+requesterID, &requester)
	if err != nil {
		return shim.Error(fmt.Sprintf("Requester identity not found: %s", requesterID))
	}
	
	// Create verification request
	request := VerificationRequest{
		ID:           requestID,
		RequesterID:  requesterID,
		SubjectID:    subjectID,
		CredentialID: credentialID,
		Purpose:      purpose,
		Status:       "pending",
		RequestedAt:  time.Now().Unix(),
		Metadata:     make(map[string]interface{}),
	}
	
	// Store verification request
	err = shim.PutStateAsJSON(stub, "verification:"+requestID, request)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store verification request: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Verification request %s created successfully", requestID)))
}

// respondToVerification responds to a verification request
func (cc *IdentityVerificationChaincode) respondToVerification(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 3); err != nil {
		return shim.Error(err.Error())
	}
	
	requestID := args[0]
	response := args[1] // "approved" or "denied"
	resultJSON := args[2]
	
	// Get verification request
	var request VerificationRequest
	err := shim.GetStateAsJSON(stub, "verification:"+requestID, &request)
	if err != nil {
		return shim.Error(fmt.Sprintf("Verification request not found: %s", requestID))
	}
	
	// Parse result
	var result map[string]interface{}
	err = json.Unmarshal([]byte(resultJSON), &result)
	if err != nil {
		return shim.Error(fmt.Sprintf("Invalid result JSON: %s", err.Error()))
	}
	
	// Update request
	request.Status = response
	request.RespondedAt = time.Now().Unix()
	request.Result = result
	
	// Store updated request
	err = shim.PutStateAsJSON(stub, "verification:"+requestID, request)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update verification request: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Verification request %s responded with %s", requestID, response)))
}

// getVerificationRequest retrieves a verification request
func (cc *IdentityVerificationChaincode) getVerificationRequest(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	requestID := args[0]
	
	requestBytes, err := stub.GetState("verification:" + requestID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get verification request: %s", err.Error()))
	}
	if requestBytes == nil {
		return shim.Error(fmt.Sprintf("Verification request not found: %s", requestID))
	}
	
	return shim.Success(requestBytes)
}

// listVerificationRequests lists verification requests with optional filters
func (cc *IdentityVerificationChaincode) listVerificationRequests(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	var requesterFilter, subjectFilter, statusFilter string
	
	if len(args) > 0 {
		requesterFilter = args[0]
	}
	if len(args) > 1 {
		subjectFilter = args[1]
	}
	if len(args) > 2 {
		statusFilter = args[2]
	}
	
	iterator, err := stub.GetStateByRange("verification:", "verification:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get verification requests: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var filteredRequests []VerificationRequest
	for _, result := range results {
		var request VerificationRequest
		requestBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(requestBytes, &request)
		
		// Apply filters
		if requesterFilter != "" && request.RequesterID != requesterFilter {
			continue
		}
		if subjectFilter != "" && request.SubjectID != subjectFilter {
			continue
		}
		if statusFilter != "" && request.Status != statusFilter {
			continue
		}
		
		filteredRequests = append(filteredRequests, request)
	}
	
	responseBytes, err := json.Marshal(filteredRequests)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// checkRevocationStatus checks if a credential is revoked
func (cc *IdentityVerificationChaincode) checkRevocationStatus(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	credentialID := args[0]
	
	isRevoked := cc.checkRevocationStatusInternal(stub, credentialID)
	
	result := map[string]interface{}{
		"credentialID": credentialID,
		"revoked":      isRevoked,
		"checkedAt":    time.Now().Unix(),
	}
	
	responseBytes, _ := json.Marshal(result)
	return shim.Success(responseBytes)
}

// listRevocations lists revocation records
func (cc *IdentityVerificationChaincode) listRevocations(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	iterator, err := stub.GetStateByRange("revocation:", "revocation:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get revocation records: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var revocations []RevocationRecord
	for _, result := range results {
		var revocation RevocationRecord
		revocationBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(revocationBytes, &revocation)
		revocations = append(revocations, revocation)
	}
	
	responseBytes, err := json.Marshal(revocations)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// Helper function to check revocation status internally
func (cc *IdentityVerificationChaincode) checkRevocationStatusInternal(stub shim.ChaincodeStubInterface, credentialID string) bool {
	iterator, err := stub.GetStateByRange("revocation:", "revocation:~")
	if err != nil {
		return false
	}
	
	results, _ := shim.IteratorToArray(iterator)
	
	for _, result := range results {
		var revocation RevocationRecord
		revocationBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(revocationBytes, &revocation)
		
		if revocation.CredentialID == credentialID && revocation.Status == "active" {
			return true
		}
	}
	
	return false
}

// Helper function to create verification result
func (cc *IdentityVerificationChaincode) createVerificationResult(valid bool, message string, details map[string]interface{}) shim.Response {
	result := map[string]interface{}{
		"valid":   valid,
		"message": message,
		"details": details,
		"verifiedAt": time.Now().Unix(),
	}
	
	responseBytes, _ := json.Marshal(result)
	return shim.Success(responseBytes)
}

// Helper function to generate credential hash (simplified)
func (cc *IdentityVerificationChaincode) generateCredentialHash(credential Credential) string {
	data := fmt.Sprintf("%s:%s:%s:%s:%d", credential.ID, credential.Type, credential.Issuer, credential.Subject, credential.IssuedAt)
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}

// main function - entry point for the chaincode
func main() {
	err := shim.Start(new(IdentityVerificationChaincode))
	if err != nil {
		log.Fatalf("Error starting Identity Verification Chaincode: %v", err)
	}
}
