# Security Considerations

This system deals with **ePHI (Electronic Protected Health Information)** and must follow best practices in alignment with HIPAA principles, even in a prototype or academic setting.

---

## 🔐 Access Control

- Role-based access control (RBAC) is in place to ensure users only see data relevant to their permissions.
- Administrative or sensitive routes are restricted.

---

## 🔑 Authentication

- All routes assume future integration with a secure authentication mechanism.
- Token-based access (e.g., JWT or session keys) is planned for production-readiness.

---

## 📦 Data at Rest

- PostgreSQL storage is assumed to reside on encrypted volumes in compliant hosting environments.
- Future production versions should enforce:
  - Full disk encryption
  - Encrypted backups
  - Role-restricted DB access

---

## 🔒 Data in Transit

- All API calls are served over HTTPS (in production environments).
- TLS 1.2+ is required for secure communication.

---

## 📜 Logging & Audit Trail

- Login attempts and API data access are logged.
- Write operations involving ePHI are tracked in a persistent audit log.

---

## 🔧 Development Environment Practices

- Secrets (e.g., REDCap API tokens, DB passwords) are managed via environment variables or secrets managers.
- No hard-coded credentials are permitted in version control.
- Developers should avoid working with real patient data in dev environments unless de-identified.
