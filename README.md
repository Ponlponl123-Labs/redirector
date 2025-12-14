# Redirector

This repository documents the lightweight redirect behavior used for short links. The implementation has moved to [Ponlponl123-Labs/api](https://github.com/Ponlponl123-Labs/api), but this README explains how the redirect endpoint works and how to use it.

**Overview**

- Source: `GET ponl.link/:id`
- Behavior: Issues an HTTP redirect to the API endpoint
- Target: `GET labs.ponlponl123.com/api/v1/redirect/:id`

**Endpoint Schema**

- **Domain:** `labs.ponlponl123.com`
- **Base Path:** `/api/`
- **Version:** `v1/`
- **Function:** `redirect/`
- **Parameter:** `:id` — the short link identifier

**How It Works**

- A request to `ponl.link/:id` looks up the short link by `id`.
- If found, the service responds with a redirect to the resolved destination via `labs.ponlponl123.com/api/v1/redirect/:id`.
- If not found, the service responds with a suitable error (e.g., 404).

**Example**

- Request: `GET https://ponl.link/abc123`
- Redirects to: `https://labs.ponlponl123.com/api/v1/redirect/abc123`

**Status Codes**

- **302 Found:** Temporary redirect to the API endpoint (typical for standard redirects).
- **301 Moved Permanently:** May be used when the short link is permanently mapped.
- **404 Not Found:** Unknown `id`.

**Notes**

- The actual redirect logic and storage of mappings live in the API repository: [Ponlponl123-Labs/api](https://github.com/Ponlponl123-Labs/api).
- Use HTTPS for all requests.
