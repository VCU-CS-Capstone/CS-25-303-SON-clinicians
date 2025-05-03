# System Architecture Overview

This document outlines the architectural structure of the CS 25 303 Backend/API project. The backend is designed to support a clinician-facing tablet application that surfaces patient survey data securely and efficiently.

---

## 🧱 Technologies Used

- **Rust**: Core backend language
- **PostgreSQL**: Relational database for storing structured patient-related data
- **REDCap**: External system for survey delivery and data collection
- **OpenAPI/Scalar**: API documentation and endpoint explorer
- **Docker** _(optional)_: Local development and containerization support

---

## 🗂 Major Components

### Backend (`src/backend/`)

- Hosts the Actix-based web API
- Handles routing, data validation, and service orchestration

### Core Logic (`src/backend/core/`)

- Database schema definitions and ORM logic
- Interfaces and logic for REDCap integration
- Core data transformation and utility functions

### Test Data Generator (`src/data-tools/`)

- CLI tool or module for generating mock data
- Used during development and for testing API behavior

### Macros (`macros/`)

- Rust procedural macros to streamline or simplify internal logic
- Example use cases: automatic logging, schema handling, etc.

---

## 🔄 Data Flow Diagram

```mermaid
[RedCAP] --> [RedCAP Integration Layer] --> [Database] --> [API Endpoint] --> [Tablet Client]
```

- Survey responses are pulled or received from REDCap.
- Data is transformed and normalized into the internal schema.
- Clinician queries via tablet interface hit the backend API.
- API returns relevant data (e.g., patient history summaries).

---

## 🔒 Compliance Considerations

- All ePHI access and modification is logged and secured.
- Access controls are in place to restrict sensitive endpoints.
- Integration layer limits exposure of sensitive REDCap tokens or endpoints.
