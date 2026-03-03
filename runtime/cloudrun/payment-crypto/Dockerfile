FROM golang:1.24-bookworm AS build

WORKDIR /src

COPY go.mod ./
RUN go mod download

COPY . ./
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -trimpath -ldflags="-s -w" -o /out/payment-crypto ./main.go

FROM gcr.io/distroless/static-debian12

WORKDIR /app
COPY --from=build /out/payment-crypto /app/payment-crypto

ENV PORT=8080
EXPOSE 8080

ENTRYPOINT ["/app/payment-crypto"]
