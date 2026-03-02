package main

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"os"
	"strings"
	"time"

	evervault "github.com/evervault/evervault-go"
)

type relayRequest struct {
	AppUUID        string            `json:"app_uuid"`
	APIKey         string            `json:"api_key"`
	DestinationURL string            `json:"destination_url"`
	Method         string            `json:"method"`
	Headers        map[string]string `json:"headers"`
	Body           interface{}       `json:"body"`
	TimeoutMS      int               `json:"timeout_ms"`
}

type relayResponse struct {
	OK              bool                `json:"ok"`
	StatusCode      int                 `json:"status_code"`
	ResponseHeaders map[string][]string `json:"response_headers"`
	ResponseBody    string              `json:"response_body"`
	Error           string              `json:"error,omitempty"`
}

func writeJSON(payload relayResponse) {
	encoder := json.NewEncoder(os.Stdout)
	_ = encoder.Encode(payload)
}

func normalizeMethod(method string) string {
	if strings.TrimSpace(method) == "" {
		return http.MethodPost
	}
	return strings.ToUpper(strings.TrimSpace(method))
}

func main() {
	var input relayRequest
	if err := json.NewDecoder(os.Stdin).Decode(&input); err != nil {
		writeJSON(relayResponse{OK: false, Error: "invalid input JSON: " + err.Error()})
		os.Exit(1)
	}

	if strings.TrimSpace(input.AppUUID) == "" || strings.TrimSpace(input.APIKey) == "" {
		writeJSON(relayResponse{OK: false, Error: "app_uuid and api_key are required"})
		os.Exit(1)
	}

	if strings.TrimSpace(input.DestinationURL) == "" {
		writeJSON(relayResponse{OK: false, Error: "destination_url is required"})
		os.Exit(1)
	}

	bodyBytes, err := json.Marshal(input.Body)
	if err != nil {
		writeJSON(relayResponse{OK: false, Error: "failed to encode body: " + err.Error()})
		os.Exit(1)
	}

	timeout := time.Duration(input.TimeoutMS) * time.Millisecond
	if timeout < time.Second {
		timeout = 20 * time.Second
	}

	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, normalizeMethod(input.Method), input.DestinationURL, bytes.NewReader(bodyBytes))
	if err != nil {
		writeJSON(relayResponse{OK: false, Error: "failed to build request: " + err.Error()})
		os.Exit(1)
	}

	for key, value := range input.Headers {
		req.Header.Set(key, value)
	}
	if req.Header.Get("Content-Type") == "" {
		req.Header.Set("Content-Type", "application/json")
	}

	client, err := evervault.MakeClient(input.AppUUID, input.APIKey)
	if err != nil {
		writeJSON(relayResponse{OK: false, Error: "failed to initialize evervault client: " + err.Error()})
		os.Exit(1)
	}

	outboundRelayClient, err := client.OutboundRelayClient()
	if err != nil {
		writeJSON(relayResponse{OK: false, Error: "failed to initialize outbound relay client: " + err.Error()})
		os.Exit(1)
	}

	resp, err := outboundRelayClient.Do(req)
	if err != nil {
		writeJSON(relayResponse{OK: false, Error: "outbound relay request failed: " + err.Error()})
		os.Exit(1)
	}
	defer resp.Body.Close()

	responseBody, _ := io.ReadAll(resp.Body)
	writeJSON(relayResponse{
		OK:              resp.StatusCode < 400,
		StatusCode:      resp.StatusCode,
		ResponseHeaders: resp.Header,
		ResponseBody:    string(responseBody),
	})
}
