package main

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"regexp"
	"strings"
	"time"

	kms "cloud.google.com/go/kms/apiv1"
	_ "github.com/jackc/pgx/v5/stdlib"
	kmspb "google.golang.org/genproto/googleapis/cloud/kms/v1"
)

const (
	maxBodyBytes          = 128 * 1024
	defaultAddr           = ":8080"
	defaultProvider       = "srt"
	defaultAadScope       = "gcp-kms:payment-method:ev-token-bundle:v1"
	defaultHTTPTimeoutSec = 10
)

const upsertPaymentMethodSQL = `
insert into payment_method_secrets (
  provider,
  owner_ref,
  payment_method_ref,
  method_kind,
  card_brand,
  card_last4,
  card_exp_month,
  card_exp_year,
  payment_payload_envelope_ciphertext,
  payment_payload_envelope_dek_ciphertext,
  payment_payload_envelope_kek_version,
  payment_payload_envelope_aad_scope,
  payment_payload_envelope_aad_subject,
  payment_payload_envelope_aad_hash,
  redacted_metadata,
  updated_at,
  revoked_at
)
values ($1, $2, $3, $4, $5, $6, null, null, $7, $8, $9, $10, $11, $12, $13::jsonb, $14, null)
on conflict (provider, owner_ref, payment_method_ref)
do update set
  method_kind = excluded.method_kind,
  card_brand = excluded.card_brand,
  card_last4 = excluded.card_last4,
  payment_payload_envelope_ciphertext = excluded.payment_payload_envelope_ciphertext,
  payment_payload_envelope_dek_ciphertext = excluded.payment_payload_envelope_dek_ciphertext,
  payment_payload_envelope_kek_version = excluded.payment_payload_envelope_kek_version,
  payment_payload_envelope_aad_scope = excluded.payment_payload_envelope_aad_scope,
  payment_payload_envelope_aad_subject = excluded.payment_payload_envelope_aad_subject,
  payment_payload_envelope_aad_hash = excluded.payment_payload_envelope_aad_hash,
  redacted_metadata = excluded.redacted_metadata,
  updated_at = excluded.updated_at,
  revoked_at = excluded.revoked_at
`

const selectActivePaymentMethodSQL = `
select
  payment_payload_envelope_ciphertext,
  payment_payload_envelope_aad_scope,
  redacted_metadata::text
from payment_method_secrets
where provider = $1
  and owner_ref = $2
  and payment_method_ref = $3
  and revoked_at is null
`

var keyVersionPattern = regexp.MustCompile(`/cryptoKeyVersions/([0-9]+)$`)

type config struct {
	AppEnv               string
	Addr                 string
	DatabaseURL          string
	KMSKeyName           string
	AadScope             string
	InternalServiceToken string
	HTTPOperationTimeout time.Duration
}

type evPayload struct {
	PanEV                 string `json:"pan_ev"`
	ExpiryMonthEV         string `json:"expiry_month_ev"`
	ExpiryYearEV          string `json:"expiry_year_ev"`
	BirthOrBusinessEV     string `json:"birth_or_business_ev"`
	CardPasswordTwoDigits string `json:"card_password_two_digits_ev"`
}

type storeRequest struct {
	Provider         string         `json:"provider"`
	OwnerRef         string         `json:"owner_ref"`
	PaymentMethodRef string         `json:"payment_method_ref"`
	EvPayload        evPayload      `json:"ev_payload"`
	Metadata         map[string]any `json:"metadata"`
}

type decryptRequest struct {
	Provider         string `json:"provider"`
	OwnerRef         string `json:"owner_ref"`
	PaymentMethodRef string `json:"payment_method_ref"`
}

type executeRequest struct {
	Provider         string `json:"provider"`
	OwnerRef         string `json:"owner_ref"`
	PaymentMethodRef string `json:"payment_method_ref"`
	DryRun           *bool  `json:"dry_run"`
}

type storeResponse struct {
	OK            bool   `json:"ok"`
	PaymentMethod string `json:"payment_method_ref,omitempty"`
	KMSKeyVersion string `json:"kms_key_version,omitempty"`
	StorageMode   string `json:"storage_mode,omitempty"`
	Detail        string `json:"detail,omitempty"`
}

type decryptResponse struct {
	OK            bool      `json:"ok"`
	Provider      string    `json:"provider,omitempty"`
	OwnerRef      string    `json:"owner_ref,omitempty"`
	PaymentMethod string    `json:"payment_method_ref,omitempty"`
	EvPayload     evPayload `json:"ev_payload,omitempty"`
	Detail        string    `json:"detail,omitempty"`
}

type executeResponse struct {
	OK       bool            `json:"ok"`
	DryRun   bool            `json:"dry_run"`
	NextStep string          `json:"next_step,omitempty"`
	Payload  decryptResponse `json:"payload"`
	Detail   string          `json:"detail,omitempty"`
}

type service struct {
	cfg       config
	db        *sql.DB
	kmsClient *kms.KeyManagementClient
}

func main() {
	cfg, err := loadConfig()
	if err != nil {
		log.Fatalf("config error: %v", err)
	}

	db, err := sql.Open("pgx", cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("database open error: %v", redactError(err))
	}
	db.SetMaxOpenConns(8)
	db.SetMaxIdleConns(8)
	db.SetConnMaxIdleTime(5 * time.Minute)
	db.SetConnMaxLifetime(30 * time.Minute)

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	if err := db.PingContext(ctx); err != nil {
		log.Fatalf("database ping error: %v", redactError(err))
	}

	kmsClient, err := kms.NewKeyManagementClient(context.Background())
	if err != nil {
		log.Fatalf("kms client error: %v", redactError(err))
	}
	defer kmsClient.Close()

	svc := &service{
		cfg:       cfg,
		db:        db,
		kmsClient: kmsClient,
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", svc.healthzHandler)
	mux.HandleFunc("/v1/payment-methods/store", svc.storeHandler)
	mux.HandleFunc("/v1/payment-methods/decrypt", svc.decryptHandler)
	mux.HandleFunc("/v1/payment-methods/execute-srt-payment", svc.executeHandler)

	handler := svc.withAuth(mux)
	log.Printf("payment-crypto listening addr=%s env=%s", cfg.Addr, cfg.AppEnv)
	if err := http.ListenAndServe(cfg.Addr, handler); err != nil {
		log.Fatalf("server error: %v", redactError(err))
	}
}

func loadConfig() (config, error) {
	port := clean(os.Getenv("PORT"))
	addr := defaultAddr
	if port != "" {
		addr = ":" + port
	}

	databaseURL := clean(os.Getenv("DATABASE_URL"))
	if databaseURL == "" {
		return config{}, errors.New("missing DATABASE_URL")
	}

	kmsKeyName := clean(os.Getenv("KMS_KEY_NAME"))
	if kmsKeyName == "" {
		return config{}, errors.New("missing KMS_KEY_NAME")
	}

	timeout := time.Duration(defaultHTTPTimeoutSec) * time.Second
	if raw := clean(os.Getenv("PAYMENT_CRYPTO_TIMEOUT_SECONDS")); raw != "" {
		parsed, err := time.ParseDuration(raw + "s")
		if err == nil && parsed > 0 {
			timeout = parsed
		}
	}

	return config{
		AppEnv:               fallback(clean(os.Getenv("APP_ENV")), "dev"),
		Addr:                 addr,
		DatabaseURL:          databaseURL,
		KMSKeyName:           kmsKeyName,
		AadScope:             fallback(clean(os.Getenv("PAYMENT_AAD_SCOPE")), defaultAadScope),
		InternalServiceToken: clean(os.Getenv("INTERNAL_SERVICE_TOKEN")),
		HTTPOperationTimeout: timeout,
	}, nil
}

func (s *service) withAuth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if s.cfg.InternalServiceToken == "" {
			next.ServeHTTP(w, r)
			return
		}

		token := extractAuthToken(r)
		if token != s.cfg.InternalServiceToken {
			writeJSON(w, http.StatusUnauthorized, map[string]any{
				"ok":     false,
				"detail": "unauthorized",
			})
			return
		}

		next.ServeHTTP(w, r)
	})
}

func (s *service) healthzHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]any{"ok": false, "detail": "method not allowed"})
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), 3*time.Second)
	defer cancel()
	if err := s.db.PingContext(ctx); err != nil {
		writeJSON(w, http.StatusServiceUnavailable, map[string]any{
			"ok":     false,
			"detail": "database unavailable",
		})
		return
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"ok":      true,
		"service": "bominal-payment-crypto-service",
		"app_env": s.cfg.AppEnv,
	})
}

func (s *service) storeHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]any{"ok": false, "detail": "method not allowed"})
		return
	}

	var req storeRequest
	if err := decodeJSON(r, &req); err != nil {
		writeJSON(w, http.StatusBadRequest, storeResponse{OK: false, Detail: "invalid json payload"})
		return
	}

	req.Provider = strings.ToLower(fallback(clean(req.Provider), defaultProvider))
	req.OwnerRef = clean(req.OwnerRef)
	req.PaymentMethodRef = clean(req.PaymentMethodRef)
	if req.OwnerRef == "" || req.PaymentMethodRef == "" {
		writeJSON(w, http.StatusBadRequest, storeResponse{
			OK:     false,
			Detail: "provider, owner_ref, and payment_method_ref are required",
		})
		return
	}

	if err := validateEVPayload(req.EvPayload); err != nil {
		writeJSON(w, http.StatusBadRequest, storeResponse{OK: false, Detail: err.Error()})
		return
	}

	ciphertext, keyVersionName, kekVersion, aadHash, err := s.encryptTokenBundle(r.Context(), req)
	if err != nil {
		writeJSON(w, http.StatusBadGateway, storeResponse{OK: false, Detail: "kms encrypt failed"})
		return
	}

	cardBrand := stringOrNil(anyToString(req.Metadata["brand"]))
	cardLast4 := nullableLast4(anyToString(req.Metadata["last4"]))

	aadContext := s.buildAadContext(req.Provider, req.OwnerRef, req.PaymentMethodRef)
	redactedMetadata := map[string]any{
		"storage_mode":     "kms_envelope_over_evervault_tokens",
		"aad_context":      aadContext,
		"key_resource":     s.cfg.KMSKeyName,
		"key_version_name": keyVersionName,
		"contract":         "ciphertext-only-v1",
	}
	redactedMetadataJSON, _ := json.Marshal(redactedMetadata)

	ctx, cancel := context.WithTimeout(r.Context(), s.cfg.HTTPOperationTimeout)
	defer cancel()
	_, execErr := s.db.ExecContext(
		ctx,
		upsertPaymentMethodSQL,
		req.Provider,
		req.OwnerRef,
		req.PaymentMethodRef,
		"card",
		cardBrand,
		cardLast4,
		ciphertext,
		[]byte{},
		kekVersion,
		s.cfg.AadScope,
		req.OwnerRef,
		aadHash,
		string(redactedMetadataJSON),
		time.Now().UTC(),
	)
	if execErr != nil {
		writeJSON(w, http.StatusBadGateway, storeResponse{OK: false, Detail: "database upsert failed"})
		return
	}

	writeJSON(w, http.StatusAccepted, storeResponse{
		OK:            true,
		PaymentMethod: req.PaymentMethodRef,
		KMSKeyVersion: keyVersionName,
		StorageMode:   "kms_envelope_over_evervault_tokens",
	})
}

func (s *service) decryptHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]any{"ok": false, "detail": "method not allowed"})
		return
	}

	var req decryptRequest
	if err := decodeJSON(r, &req); err != nil {
		writeJSON(w, http.StatusBadRequest, decryptResponse{OK: false, Detail: "invalid json payload"})
		return
	}

	resp, status := s.decryptByRef(r.Context(), req.Provider, req.OwnerRef, req.PaymentMethodRef)
	writeJSON(w, status, resp)
}

func (s *service) executeHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, map[string]any{"ok": false, "detail": "method not allowed"})
		return
	}

	var req executeRequest
	if err := decodeJSON(r, &req); err != nil {
		writeJSON(w, http.StatusBadRequest, executeResponse{
			OK:     false,
			DryRun: true,
			Detail: "invalid json payload",
		})
		return
	}

	dryRun := true
	if req.DryRun != nil {
		dryRun = *req.DryRun
	}

	if !dryRun {
		writeJSON(w, http.StatusNotImplemented, executeResponse{
			OK:     false,
			DryRun: false,
			Detail: "live provider relay from this service is not enabled; use dry_run=true",
		})
		return
	}

	payload, status := s.decryptByRef(r.Context(), req.Provider, req.OwnerRef, req.PaymentMethodRef)
	writeJSON(w, http.StatusAccepted, executeResponse{
		OK:       payload.OK && status == http.StatusOK,
		DryRun:   true,
		NextStep: "submit ev_payload to provider via Evervault relay in payment worker",
		Payload:  payload,
	})
}

func (s *service) decryptByRef(ctx context.Context, provider, ownerRef, paymentMethodRef string) (decryptResponse, int) {
	provider = strings.ToLower(fallback(clean(provider), defaultProvider))
	ownerRef = clean(ownerRef)
	paymentMethodRef = clean(paymentMethodRef)
	if ownerRef == "" || paymentMethodRef == "" {
		return decryptResponse{
			OK:     false,
			Detail: "provider, owner_ref, and payment_method_ref are required",
		}, http.StatusBadRequest
	}

	queryCtx, cancel := context.WithTimeout(ctx, s.cfg.HTTPOperationTimeout)
	defer cancel()

	var ciphertext []byte
	var aadScope string
	var redactedMetadataRaw string
	row := s.db.QueryRowContext(queryCtx, selectActivePaymentMethodSQL, provider, ownerRef, paymentMethodRef)
	if err := row.Scan(&ciphertext, &aadScope, &redactedMetadataRaw); err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return decryptResponse{OK: false, Detail: "payment method not found"}, http.StatusNotFound
		}
		return decryptResponse{OK: false, Detail: "database query failed"}, http.StatusBadGateway
	}

	aadContext := s.buildAadContext(provider, ownerRef, paymentMethodRef)
	var metadata map[string]any
	if err := json.Unmarshal([]byte(redactedMetadataRaw), &metadata); err == nil {
		if rawAad, ok := metadata["aad_context"]; ok {
			if aadMap, ok := rawAad.(map[string]any); ok {
				aadContext = aadMap
			}
		}
	}
	if cleaned := clean(aadScope); cleaned != "" {
		aadContext["scope"] = cleaned
	}

	aadBytes, _ := json.Marshal(aadContext)
	decryptCtx, decryptCancel := context.WithTimeout(ctx, s.cfg.HTTPOperationTimeout)
	defer decryptCancel()

	kmsResp, err := s.kmsClient.Decrypt(decryptCtx, &kmspb.DecryptRequest{
		Name:                        s.cfg.KMSKeyName,
		Ciphertext:                  ciphertext,
		AdditionalAuthenticatedData: aadBytes,
	})
	if err != nil {
		return decryptResponse{OK: false, Detail: "kms decrypt failed"}, http.StatusBadGateway
	}

	var stored struct {
		EvPayload evPayload `json:"ev_payload"`
	}
	if err := json.Unmarshal(kmsResp.Plaintext, &stored); err != nil {
		return decryptResponse{OK: false, Detail: "invalid decrypted payload"}, http.StatusBadGateway
	}

	return decryptResponse{
		OK:            true,
		Provider:      provider,
		OwnerRef:      ownerRef,
		PaymentMethod: paymentMethodRef,
		EvPayload:     stored.EvPayload,
	}, http.StatusOK
}

func (s *service) encryptTokenBundle(ctx context.Context, req storeRequest) ([]byte, string, int, []byte, error) {
	aadContext := s.buildAadContext(req.Provider, req.OwnerRef, req.PaymentMethodRef)
	aadBytes, _ := json.Marshal(aadContext)

	plaintextStruct := map[string]any{
		"provider":           req.Provider,
		"owner_ref":          req.OwnerRef,
		"payment_method_ref": req.PaymentMethodRef,
		"ev_payload":         req.EvPayload,
	}
	plaintext, _ := json.Marshal(plaintextStruct)

	encryptCtx, cancel := context.WithTimeout(ctx, s.cfg.HTTPOperationTimeout)
	defer cancel()
	resp, err := s.kmsClient.Encrypt(encryptCtx, &kmspb.EncryptRequest{
		Name:                        s.cfg.KMSKeyName,
		Plaintext:                   plaintext,
		AdditionalAuthenticatedData: aadBytes,
	})
	if err != nil {
		return nil, "", 0, nil, err
	}

	keyVersionName := clean(resp.GetName())
	kekVersion := parseKekVersion(keyVersionName)
	sum := sha256.Sum256(aadBytes)
	return resp.GetCiphertext(), keyVersionName, kekVersion, sum[:], nil
}

func (s *service) buildAadContext(provider, ownerRef, paymentMethodRef string) map[string]any {
	return map[string]any{
		"contract_version":   "v1",
		"scope":              s.cfg.AadScope,
		"provider":           provider,
		"owner_ref":          ownerRef,
		"payment_method_ref": paymentMethodRef,
	}
}

func decodeJSON(r *http.Request, target any) error {
	if r.Body == nil {
		return errors.New("missing body")
	}
	limited := io.LimitReader(r.Body, maxBodyBytes)
	decoder := json.NewDecoder(limited)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(target); err != nil {
		return err
	}
	if decoder.More() {
		return errors.New("multiple json values are not allowed")
	}
	return nil
}

func validateEVPayload(payload evPayload) error {
	if !looksLikeEVToken(payload.PanEV) {
		return errors.New("ev_payload.pan_ev must be an Evervault encrypted token")
	}
	if !looksLikeEVToken(payload.ExpiryMonthEV) {
		return errors.New("ev_payload.expiry_month_ev must be an Evervault encrypted token")
	}
	if !looksLikeEVToken(payload.ExpiryYearEV) {
		return errors.New("ev_payload.expiry_year_ev must be an Evervault encrypted token")
	}
	if !looksLikeEVToken(payload.BirthOrBusinessEV) {
		return errors.New("ev_payload.birth_or_business_ev must be an Evervault encrypted token")
	}
	if !looksLikeEVToken(payload.CardPasswordTwoDigits) {
		return errors.New("ev_payload.card_password_two_digits_ev must be an Evervault encrypted token")
	}
	return nil
}

func looksLikeEVToken(value string) bool {
	return strings.HasPrefix(clean(value), "ev:")
}

func extractAuthToken(r *http.Request) string {
	if token := clean(r.Header.Get("x-internal-service-token")); token != "" {
		return token
	}

	authHeader := clean(r.Header.Get("authorization"))
	if strings.HasPrefix(strings.ToLower(authHeader), "bearer ") {
		return clean(authHeader[len("bearer "):])
	}
	return ""
}

func parseKekVersion(keyVersionName string) int {
	matches := keyVersionPattern.FindStringSubmatch(clean(keyVersionName))
	if len(matches) != 2 {
		return 1
	}
	var parsed int
	if _, err := fmt.Sscanf(matches[1], "%d", &parsed); err != nil || parsed <= 0 {
		return 1
	}
	return parsed
}

func nullableLast4(raw string) any {
	value := clean(raw)
	if matched := regexp.MustCompile(`^[0-9]{4}$`).MatchString(value); matched {
		return value
	}
	return nil
}

func anyToString(value any) string {
	switch v := value.(type) {
	case string:
		return v
	case fmt.Stringer:
		return v.String()
	default:
		return ""
	}
}

func stringOrNil(value string) any {
	cleaned := clean(value)
	if cleaned == "" {
		return nil
	}
	return cleaned
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("content-type", "application/json")
	w.WriteHeader(status)
	encoder := json.NewEncoder(w)
	_ = encoder.Encode(payload)
}

func clean(value string) string {
	return strings.TrimSpace(value)
}

func fallback(value string, fallbackValue string) string {
	if clean(value) == "" {
		return fallbackValue
	}
	return value
}

func redactError(err error) string {
	if err == nil {
		return "internal error"
	}
	message := clean(err.Error())
	if message == "" {
		return "internal error"
	}
	if len(message) > 180 {
		return message[:180]
	}
	return message
}
