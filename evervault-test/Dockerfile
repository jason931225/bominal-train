FROM golang:1.24-alpine AS go_builder

WORKDIR /go-src
COPY go-relay-sender/go.mod go-relay-sender/go.sum ./go-relay-sender/
WORKDIR /go-src/go-relay-sender
RUN go mod download
COPY go-relay-sender/. ./
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /out/relay-sender ./cmd/relay_sender

FROM node:20-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --omit=dev

COPY . .
COPY --from=go_builder /out/relay-sender /app/bin/relay-sender

EXPOSE 8787

CMD ["npm", "start"]
