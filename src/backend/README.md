# CS 25 303 – Backend / API Documentation

This repository contains the backend and core API logic for the CS 25 303 project. The backend is designed to support secure, structured, and scalable interactions between client applications and the project’s data infrastructure.

> **Live API Docs**: Explore the OpenAPI specification and endpoint documentation here:  
> [cs-25-303.wyatt-herkamp.dev/scalar](https://cs-25-303.wyatt-herkamp.dev/scalar)

---

## 🧩 Project Structure

| Folder            | Description                                                            |
| ----------------- | ---------------------------------------------------------------------- |
| `backend/`        | Main source code for the web API implementation                        |
| `core/`           | Core logic modules including database design, RedCap integration, etc. |
| `test-data/`      | Test data generation utilities and related scripts                     |
| `macros/`         | Reusable procedural macros written to support project functionality    |
| `databaseReport/` | Documentation and diagrams related to the project’s database schema    |

Each folder is organized for modularity and maintainability, making it easier for contributors to understand and extend the system.

---

## ⚙️ Setting Up the Development Environment

### 1. Install Rust

Follow the official instructions:  
🔗 [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

#### Windows-Specific Note

To build dependencies like `aws-lc-rs`, you may need the C++ Build Tools.  
We recommend using [Visual Studio Community Edition](https://visualstudio.microsoft.com/vs/community/) to install them.

#### AWS-LC-RS Build Requirements

This project uses [`aws-lc-rs`](https://aws.github.io/aws-lc-rs/). Make sure to follow their [setup instructions](https://aws.github.io/aws-lc-rs/requirements/index.html) to install the necessary build tooling (e.g., Perl, NASM, CMake).

---

### 2. Set Up a PostgreSQL Database

We recommend using Docker to quickly spin up a local PostgreSQL instance:

🔗 [Official Docker Image](https://hub.docker.com/_/postgres)

Basic Docker command (example):

```bash
docker run --name my-postgres -e POSTGRES_PASSWORD=yourpassword -p 5432:5432 -d postgres
```
