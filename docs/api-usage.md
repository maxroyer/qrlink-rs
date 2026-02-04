# API Usage (curl)

Set your base URL for examples:

```bash
BASE_URL="http://localhost:8080"
```

## Create a short link

```bash
curl -X POST "$BASE_URL/api/v1/links" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/very/long/path",
    "ttl": "1_month"
  }'
```

## Create a permanent link (never expires)

```bash
curl -X POST "$BASE_URL/api/v1/links" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com/permanent",
    "ttl": "never"
  }'
```

## Resolve a short link

```bash
curl -v "$BASE_URL/Ab3kP9x"
```

## Get QR code for a short link

```bash
curl -o qr.png "$BASE_URL/Ab3kP9x/qr"
```

## Generate QR directly (no shortening)

```bash
curl -X POST "$BASE_URL/api/v1/qr" \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com"}' \
  -o qr.png
```

## List all links

```bash
curl "$BASE_URL/api/v1/links"
```

If `ADMIN_SECRET` is set, provide the secret using the header:

```bash
curl "$BASE_URL/api/v1/links" \
  -H "X-Delete-Secret: your-secret"
```

## Delete a link

```bash
curl -X DELETE "$BASE_URL/api/v1/links/{id}"
```

If `ADMIN_SECRET` is set, provide the secret using the header:

```bash
curl -X DELETE "$BASE_URL/api/v1/links/{id}" \
  -H "X-Delete-Secret: your-secret"
```

## Rate limiting

- Default: 60 requests per minute per IP
- On limit: `429 Too Many Requests` with `Retry-After` header
