# REDCap Integration Guide

This document explains how the backend integrates with REDCap to retrieve and process patient survey data.

---

## 📄 What is REDCap?

[REDCap](https://projectredcap.org/) (Research Electronic Data Capture) is a secure web application used to collect data for research studies and clinical trials. It is widely adopted in academic and healthcare institutions.

---

## 🔌 Integration Strategy

- REDCap exposes a **RESTful API** that allows secure export of survey data.
- The backend includes logic in `core/redcap.rs` (or equivalent) to:
  - Authenticate with the REDCap API using a token
  - Fetch survey results for a given patient ID or time range
  - Normalize the data to fit the internal PostgreSQL schema

---

## 🔁 Syncing Strategy

- The current implementation uses **manual or event-based syncing** from REDCap.
- Future improvements may include automated polling or webhook-style updates if REDCap instance permits.

---

## 📦 Data Mapping

- REDCap field names are mapped to internal field names using a translation layer.
- Transformation includes:
  - Date formatting
  - Score normalization
  - Null handling

---

## 🛡️ Access Security

- REDCap tokens are stored securely (never hardcoded).
- All access to REDCap APIs is abstracted behind a controlled interface.

---

## ⚠️ Limitations

- Rate limits or availability may depend on institution-specific REDCap setup.
- Currently scoped for _read-only_ operations; no write-back supported.
