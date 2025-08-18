package main

import (
	"encoding/json"
	"fmt"
	"log"
	"strconv"
	"time"

	"github.com/beacon-blockchain/sdk-go/shim"
)

// SupplyChainChaincode implements tracking and provenance for supply chain management
type SupplyChainChaincode struct{}

// Product represents a product in the supply chain
type Product struct {
	ID           string                 `json:"id"`
	Name         string                 `json:"name"`
	Description  string                 `json:"description"`
	SKU          string                 `json:"sku"`
	Category     string                 `json:"category"`
	Manufacturer string                 `json:"manufacturer"`
	CreatedAt    int64                  `json:"createdAt"`
	Status       string                 `json:"status"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// Shipment represents a shipment in the supply chain
type Shipment struct {
	ID           string                 `json:"id"`
	ProductID    string                 `json:"productID"`
	FromLocation string                 `json:"fromLocation"`
	ToLocation   string                 `json:"toLocation"`
	Carrier      string                 `json:"carrier"`
	TrackingID   string                 `json:"trackingID"`
	Status       string                 `json:"status"`
	ShippedAt    int64                  `json:"shippedAt"`
	DeliveredAt  int64                  `json:"deliveredAt,omitempty"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// Transaction represents a transaction in the supply chain
type Transaction struct {
	ID          string                 `json:"id"`
	ProductID   string                 `json:"productID"`
	ShipmentID  string                 `json:"shipmentID,omitempty"`
	Type        string                 `json:"type"`
	From        string                 `json:"from"`
	To          string                 `json:"to"`
	Timestamp   int64                  `json:"timestamp"`
	Amount      float64                `json:"amount,omitempty"`
	Currency    string                 `json:"currency,omitempty"`
	Status      string                 `json:"status"`
	Metadata    map[string]interface{} `json:"metadata"`
	TxHash      string                 `json:"txHash"`
}

// ProvenanceRecord represents a provenance record
type ProvenanceRecord struct {
	ID        string                 `json:"id"`
	ProductID string                 `json:"productID"`
	Action    string                 `json:"action"`
	Actor     string                 `json:"actor"`
	Location  string                 `json:"location"`
	Timestamp int64                  `json:"timestamp"`
	Evidence  map[string]interface{} `json:"evidence"`
	Verified  bool                   `json:"verified"`
	TxHash    string                 `json:"txHash"`
}

// Init initializes the chaincode
func (cc *SupplyChainChaincode) Init(stub shim.ChaincodeStubInterface) shim.Response {
	log.Println("Initializing Supply Chain Chaincode")
	
	// Initialize system configuration
	config := map[string]interface{}{
		"version":              "1.0.0",
		"maxProducts":          10000,
		"maxShipments":         50000,
		"provenanceEnabled":    true,
		"automaticVerification": false,
		"supportedCurrencies":  []string{"USD", "EUR", "GBP", "JPY"},
	}
	
	err := shim.PutStateAsJSON(stub, "config:system", config)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to initialize configuration: %s", err.Error()))
	}
	
	return shim.Success([]byte("Supply Chain Chaincode initialized successfully"))
}

// Invoke handles chaincode invocations
func (cc *SupplyChainChaincode) Invoke(stub shim.ChaincodeStubInterface) shim.Response {
	function, args := stub.GetFunctionAndParameters()
	
	switch function {
	// Product operations
	case "createProduct":
		return cc.createProduct(stub, args)
	case "updateProduct":
		return cc.updateProduct(stub, args)
	case "getProduct":
		return cc.getProduct(stub, args)
	case "listProducts":
		return cc.listProducts(stub, args)
	
	// Shipment operations
	case "createShipment":
		return cc.createShipment(stub, args)
	case "updateShipmentStatus":
		return cc.updateShipmentStatus(stub, args)
	case "deliverShipment":
		return cc.deliverShipment(stub, args)
	case "getShipment":
		return cc.getShipment(stub, args)
	case "trackShipment":
		return cc.trackShipment(stub, args)
	
	// Transaction operations
	case "recordTransaction":
		return cc.recordTransaction(stub, args)
	case "getTransaction":
		return cc.getTransaction(stub, args)
	case "getProductTransactions":
		return cc.getProductTransactions(stub, args)
	
	// Provenance operations
	case "recordProvenance":
		return cc.recordProvenance(stub, args)
	case "verifyProvenance":
		return cc.verifyProvenance(stub, args)
	case "getProductProvenance":
		return cc.getProductProvenance(stub, args)
	case "traceProduct":
		return cc.traceProduct(stub, args)
	
	default:
		return shim.Error(fmt.Sprintf("Unknown function: %s", function))
	}
}

// createProduct creates a new product
func (cc *SupplyChainChaincode) createProduct(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 5); err != nil {
		return shim.Error(err.Error())
	}
	
	productID := args[0]
	name := args[1]
	description := args[2]
	sku := args[3]
	manufacturer := args[4]
	
	// Check if product already exists
	existing, err := stub.GetState("product:" + productID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to check existing product: %s", err.Error()))
	}
	if existing != nil {
		return shim.Error(fmt.Sprintf("Product already exists: %s", productID))
	}
	
	// Create new product
	product := Product{
		ID:           productID,
		Name:         name,
		Description:  description,
		SKU:          sku,
		Manufacturer: manufacturer,
		CreatedAt:    time.Now().Unix(),
		Status:       "created",
		Metadata:     make(map[string]interface{}),
	}
	
	// Store product
	err = shim.PutStateAsJSON(stub, "product:"+productID, product)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store product: %s", err.Error()))
	}
	
	// Record provenance
	cc.recordProvenanceInternal(stub, productID, "CREATE", manufacturer, "factory", map[string]interface{}{
		"sku": sku,
		"name": name,
	})
	
	// Emit event
	eventPayload, _ := json.Marshal(map[string]interface{}{
		"action":    "product_created",
		"productID": productID,
		"name":      name,
		"manufacturer": manufacturer,
	})
	stub.SetEvent("ProductCreated", eventPayload)
	
	return shim.Success([]byte(fmt.Sprintf("Product %s created successfully", productID)))
}

// updateProduct updates product metadata
func (cc *SupplyChainChaincode) updateProduct(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgsRange(args, 3, 20); err != nil {
		return shim.Error(err.Error())
	}
	
	productID := args[0]
	status := args[1]
	
	// Get existing product
	var product Product
	err := shim.GetStateAsJSON(stub, "product:"+productID, &product)
	if err != nil {
		return shim.Error(fmt.Sprintf("Product not found: %s", productID))
	}
	
	// Update status
	product.Status = status
	
	// Update metadata from key-value pairs
	for i := 2; i < len(args); i += 2 {
		if i+1 < len(args) {
			key := args[i]
			value := args[i+1]
			product.Metadata[key] = value
		}
	}
	
	// Store updated product
	err = shim.PutStateAsJSON(stub, "product:"+productID, product)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update product: %s", err.Error()))
	}
	
	// Record provenance
	cc.recordProvenanceInternal(stub, productID, "UPDATE", "system", "network", map[string]interface{}{
		"status": status,
		"metadata": product.Metadata,
	})
	
	return shim.Success([]byte(fmt.Sprintf("Product %s updated successfully", productID)))
}

// getProduct retrieves a product by ID
func (cc *SupplyChainChaincode) getProduct(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	productID := args[0]
	
	productBytes, err := stub.GetState("product:" + productID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get product: %s", err.Error()))
	}
	if productBytes == nil {
		return shim.Error(fmt.Sprintf("Product not found: %s", productID))
	}
	
	return shim.Success(productBytes)
}

// listProducts returns all products with optional category filter
func (cc *SupplyChainChaincode) listProducts(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	categoryFilter := ""
	if len(args) > 0 {
		categoryFilter = args[0]
	}
	
	iterator, err := stub.GetStateByRange("product:", "product:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get products: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var filteredProducts []Product
	for _, result := range results {
		var product Product
		productBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(productBytes, &product)
		
		if categoryFilter == "" || product.Category == categoryFilter {
			filteredProducts = append(filteredProducts, product)
		}
	}
	
	responseBytes, err := json.Marshal(filteredProducts)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// createShipment creates a new shipment
func (cc *SupplyChainChaincode) createShipment(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 6); err != nil {
		return shim.Error(err.Error())
	}
	
	shipmentID := args[0]
	productID := args[1]
	fromLocation := args[2]
	toLocation := args[3]
	carrier := args[4]
	trackingID := args[5]
	
	// Verify product exists
	var product Product
	err := shim.GetStateAsJSON(stub, "product:"+productID, &product)
	if err != nil {
		return shim.Error(fmt.Sprintf("Product not found: %s", productID))
	}
	
	// Create new shipment
	shipment := Shipment{
		ID:           shipmentID,
		ProductID:    productID,
		FromLocation: fromLocation,
		ToLocation:   toLocation,
		Carrier:      carrier,
		TrackingID:   trackingID,
		Status:       "in_transit",
		ShippedAt:    time.Now().Unix(),
		Metadata:     make(map[string]interface{}),
	}
	
	// Store shipment
	err = shim.PutStateAsJSON(stub, "shipment:"+shipmentID, shipment)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store shipment: %s", err.Error()))
	}
	
	// Record provenance
	cc.recordProvenanceInternal(stub, productID, "SHIP", carrier, fromLocation, map[string]interface{}{
		"shipmentID": shipmentID,
		"trackingID": trackingID,
		"destination": toLocation,
	})
	
	// Emit event
	eventPayload, _ := json.Marshal(map[string]interface{}{
		"action":     "shipment_created",
		"shipmentID": shipmentID,
		"productID":  productID,
		"carrier":    carrier,
		"trackingID": trackingID,
	})
	stub.SetEvent("ShipmentCreated", eventPayload)
	
	return shim.Success([]byte(fmt.Sprintf("Shipment %s created successfully", shipmentID)))
}

// updateShipmentStatus updates shipment status
func (cc *SupplyChainChaincode) updateShipmentStatus(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 3); err != nil {
		return shim.Error(err.Error())
	}
	
	shipmentID := args[0]
	status := args[1]
	location := args[2]
	
	// Get existing shipment
	var shipment Shipment
	err := shim.GetStateAsJSON(stub, "shipment:"+shipmentID, &shipment)
	if err != nil {
		return shim.Error(fmt.Sprintf("Shipment not found: %s", shipmentID))
	}
	
	// Update status
	shipment.Status = status
	if shipment.Metadata == nil {
		shipment.Metadata = make(map[string]interface{})
	}
	shipment.Metadata["lastLocation"] = location
	shipment.Metadata["lastUpdate"] = time.Now().Unix()
	
	// Store updated shipment
	err = shim.PutStateAsJSON(stub, "shipment:"+shipmentID, shipment)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to update shipment: %s", err.Error()))
	}
	
	// Record provenance
	cc.recordProvenanceInternal(stub, shipment.ProductID, "TRACK", shipment.Carrier, location, map[string]interface{}{
		"shipmentID": shipmentID,
		"status": status,
	})
	
	return shim.Success([]byte(fmt.Sprintf("Shipment %s status updated to %s", shipmentID, status)))
}

// deliverShipment marks a shipment as delivered
func (cc *SupplyChainChaincode) deliverShipment(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 2); err != nil {
		return shim.Error(err.Error())
	}
	
	shipmentID := args[0]
	recipient := args[1]
	
	// Get existing shipment
	var shipment Shipment
	err := shim.GetStateAsJSON(stub, "shipment:"+shipmentID, &shipment)
	if err != nil {
		return shim.Error(fmt.Sprintf("Shipment not found: %s", shipmentID))
	}
	
	// Update delivery status
	shipment.Status = "delivered"
	shipment.DeliveredAt = time.Now().Unix()
	if shipment.Metadata == nil {
		shipment.Metadata = make(map[string]interface{})
	}
	shipment.Metadata["recipient"] = recipient
	
	// Store updated shipment
	err = shim.PutStateAsJSON(stub, "shipment:"+shipmentID, shipment)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to deliver shipment: %s", err.Error()))
	}
	
	// Record provenance
	cc.recordProvenanceInternal(stub, shipment.ProductID, "DELIVER", recipient, shipment.ToLocation, map[string]interface{}{
		"shipmentID": shipmentID,
		"deliveredAt": shipment.DeliveredAt,
	})
	
	// Emit event
	eventPayload, _ := json.Marshal(map[string]interface{}{
		"action":     "shipment_delivered",
		"shipmentID": shipmentID,
		"productID":  shipment.ProductID,
		"recipient":  recipient,
		"deliveredAt": shipment.DeliveredAt,
	})
	stub.SetEvent("ShipmentDelivered", eventPayload)
	
	return shim.Success([]byte(fmt.Sprintf("Shipment %s delivered successfully", shipmentID)))
}

// getShipment retrieves a shipment by ID
func (cc *SupplyChainChaincode) getShipment(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	shipmentID := args[0]
	
	shipmentBytes, err := stub.GetState("shipment:" + shipmentID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get shipment: %s", err.Error()))
	}
	if shipmentBytes == nil {
		return shim.Error(fmt.Sprintf("Shipment not found: %s", shipmentID))
	}
	
	return shim.Success(shipmentBytes)
}

// trackShipment tracks a shipment by tracking ID
func (cc *SupplyChainChaincode) trackShipment(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	trackingID := args[0]
	
	// Search for shipment by tracking ID
	iterator, err := stub.GetStateByRange("shipment:", "shipment:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to search shipments: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	for _, result := range results {
		var shipment Shipment
		shipmentBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(shipmentBytes, &shipment)
		
		if shipment.TrackingID == trackingID {
			responseBytes, _ := json.Marshal(shipment)
			return shim.Success(responseBytes)
		}
	}
	
	return shim.Error(fmt.Sprintf("Shipment with tracking ID %s not found", trackingID))
}

// recordTransaction records a transaction
func (cc *SupplyChainChaincode) recordTransaction(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgsRange(args, 6, 8); err != nil {
		return shim.Error(err.Error())
	}
	
	transactionID := args[0]
	productID := args[1]
	txType := args[2]
	from := args[3]
	to := args[4]
	status := args[5]
	
	var amount float64
	var currency string
	
	if len(args) > 6 {
		var err error
		amount, err = strconv.ParseFloat(args[6], 64)
		if err != nil {
			return shim.Error(fmt.Sprintf("Invalid amount: %s", args[6]))
		}
	}
	if len(args) > 7 {
		currency = args[7]
	}
	
	// Create transaction
	transaction := Transaction{
		ID:        transactionID,
		ProductID: productID,
		Type:      txType,
		From:      from,
		To:        to,
		Timestamp: time.Now().Unix(),
		Amount:    amount,
		Currency:  currency,
		Status:    status,
		Metadata:  make(map[string]interface{}),
		TxHash:    stub.GetTxID(),
	}
	
	// Store transaction
	err := shim.PutStateAsJSON(stub, "transaction:"+transactionID, transaction)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to store transaction: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Transaction %s recorded successfully", transactionID)))
}

// getTransaction retrieves a transaction by ID
func (cc *SupplyChainChaincode) getTransaction(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	transactionID := args[0]
	
	transactionBytes, err := stub.GetState("transaction:" + transactionID)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get transaction: %s", err.Error()))
	}
	if transactionBytes == nil {
		return shim.Error(fmt.Sprintf("Transaction not found: %s", transactionID))
	}
	
	return shim.Success(transactionBytes)
}

// getProductTransactions retrieves all transactions for a product
func (cc *SupplyChainChaincode) getProductTransactions(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	productID := args[0]
	
	iterator, err := stub.GetStateByRange("transaction:", "transaction:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get transactions: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var productTransactions []Transaction
	for _, result := range results {
		var transaction Transaction
		transactionBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(transactionBytes, &transaction)
		
		if transaction.ProductID == productID {
			productTransactions = append(productTransactions, transaction)
		}
	}
	
	responseBytes, err := json.Marshal(productTransactions)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// recordProvenance records a provenance entry
func (cc *SupplyChainChaincode) recordProvenance(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 5); err != nil {
		return shim.Error(err.Error())
	}
	
	productID := args[0]
	action := args[1]
	actor := args[2]
	location := args[3]
	evidenceJSON := args[4]
	
	// Parse evidence
	var evidence map[string]interface{}
	err := json.Unmarshal([]byte(evidenceJSON), &evidence)
	if err != nil {
		return shim.Error(fmt.Sprintf("Invalid evidence JSON: %s", err.Error()))
	}
	
	return cc.recordProvenanceResponse(stub, productID, action, actor, location, evidence)
}

// verifyProvenance verifies a provenance record
func (cc *SupplyChainChaincode) verifyProvenance(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 2); err != nil {
		return shim.Error(err.Error())
	}
	
	provenanceID := args[0]
	verifier := args[1]
	
	// Get provenance record
	var provenance ProvenanceRecord
	err := shim.GetStateAsJSON(stub, provenanceID, &provenance)
	if err != nil {
		return shim.Error(fmt.Sprintf("Provenance record not found: %s", provenanceID))
	}
	
	// Update verification status
	provenance.Verified = true
	if provenance.Evidence == nil {
		provenance.Evidence = make(map[string]interface{})
	}
	provenance.Evidence["verifiedBy"] = verifier
	provenance.Evidence["verifiedAt"] = time.Now().Unix()
	
	// Store updated provenance
	err = shim.PutStateAsJSON(stub, provenanceID, provenance)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to verify provenance: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Provenance %s verified successfully", provenanceID)))
}

// getProductProvenance retrieves all provenance records for a product
func (cc *SupplyChainChaincode) getProductProvenance(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	productID := args[0]
	
	iterator, err := stub.GetStateByRange("provenance:", "provenance:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get provenance records: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process results: %s", err.Error()))
	}
	
	var provenanceRecords []ProvenanceRecord
	for _, result := range results {
		var provenance ProvenanceRecord
		provenanceBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(provenanceBytes, &provenance)
		
		if provenance.ProductID == productID {
			provenanceRecords = append(provenanceRecords, provenance)
		}
	}
	
	responseBytes, err := json.Marshal(provenanceRecords)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// traceProduct provides complete traceability for a product
func (cc *SupplyChainChaincode) traceProduct(stub shim.ChaincodeStubInterface, args []string) shim.Response {
	if err := shim.ValidateArgs(args, 1); err != nil {
		return shim.Error(err.Error())
	}
	
	productID := args[0]
	
	// Get product
	var product Product
	err := shim.GetStateAsJSON(stub, "product:"+productID, &product)
	if err != nil {
		return shim.Error(fmt.Sprintf("Product not found: %s", productID))
	}
	
	// Get provenance records
	provenanceResponse := cc.getProductProvenance(stub, []string{productID})
	if provenanceResponse.Status != 200 {
		return provenanceResponse
	}
	
	var provenanceRecords []ProvenanceRecord
	json.Unmarshal(provenanceResponse.Payload, &provenanceRecords)
	
	// Get transactions
	transactionsResponse := cc.getProductTransactions(stub, []string{productID})
	var transactions []Transaction
	if transactionsResponse.Status == 200 {
		json.Unmarshal(transactionsResponse.Payload, &transactions)
	}
	
	// Get shipments
	iterator, err := stub.GetStateByRange("shipment:", "shipment:~")
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to get shipments: %s", err.Error()))
	}
	
	results, err := shim.IteratorToArray(iterator)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to process shipment results: %s", err.Error()))
	}
	
	var shipments []Shipment
	for _, result := range results {
		var shipment Shipment
		shipmentBytes, _ := json.Marshal(result.Value)
		json.Unmarshal(shipmentBytes, &shipment)
		
		if shipment.ProductID == productID {
			shipments = append(shipments, shipment)
		}
	}
	
	// Create trace response
	trace := map[string]interface{}{
		"product":     product,
		"provenance":  provenanceRecords,
		"transactions": transactions,
		"shipments":   shipments,
		"generatedAt": time.Now().Unix(),
	}
	
	responseBytes, err := json.Marshal(trace)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to marshal trace response: %s", err.Error()))
	}
	
	return shim.Success(responseBytes)
}

// Helper function to record provenance internally
func (cc *SupplyChainChaincode) recordProvenanceInternal(stub shim.ChaincodeStubInterface, productID, action, actor, location string, evidence map[string]interface{}) {
	provenanceID := fmt.Sprintf("provenance:%s:%d:%s", productID, time.Now().UnixNano(), stub.GetTxID())
	
	provenance := ProvenanceRecord{
		ID:        provenanceID,
		ProductID: productID,
		Action:    action,
		Actor:     actor,
		Location:  location,
		Timestamp: time.Now().Unix(),
		Evidence:  evidence,
		Verified:  false,
		TxHash:    stub.GetTxID(),
	}
	
	shim.PutStateAsJSON(stub, provenanceID, provenance)
}

// Helper function to record provenance and return response
func (cc *SupplyChainChaincode) recordProvenanceResponse(stub shim.ChaincodeStubInterface, productID, action, actor, location string, evidence map[string]interface{}) shim.Response {
	provenanceID := fmt.Sprintf("provenance:%s:%d:%s", productID, time.Now().UnixNano(), stub.GetTxID())
	
	provenance := ProvenanceRecord{
		ID:        provenanceID,
		ProductID: productID,
		Action:    action,
		Actor:     actor,
		Location:  location,
		Timestamp: time.Now().Unix(),
		Evidence:  evidence,
		Verified:  false,
		TxHash:    stub.GetTxID(),
	}
	
	err := shim.PutStateAsJSON(stub, provenanceID, provenance)
	if err != nil {
		return shim.Error(fmt.Sprintf("Failed to record provenance: %s", err.Error()))
	}
	
	return shim.Success([]byte(fmt.Sprintf("Provenance recorded: %s", provenanceID)))
}

// main function - entry point for the chaincode
func main() {
	err := shim.Start(new(SupplyChainChaincode))
	if err != nil {
		log.Fatalf("Error starting Supply Chain Chaincode: %v", err)
	}
}
