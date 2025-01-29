# CS 25 303 Backend / API

## Layout

### backend
Within the backend directory you will find the src for the web api for the project
### Core
Within the core directory you find core code for the project. Database Design, Red Cap Integration, etc.

### test-data
Within the test-data directory you will find the src for the test data generator for the project

### Macros
Within the macros directory you will find the src for the macros for the project

### databaseReport

Within the databaseReport directory you will find documentation and explanation of the database design for the project

## Setting Up The Development Environment

### Setup Rust

Follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install)

#### On Windows
You might need to also install the C++ Build Tools.
You can use [Visual Studio Community](https://visualstudio.microsoft.com/vs/community/) to handle that for you.

#### AWS-LC-RS build issues
Please read through the [instructions](https://aws.github.io/aws-lc-rs/requirements/index.html) for aws-lc-rs on getting the nessary tooling to build the project.


### Setup a PostgresSQL Database

I recommend using docker to setup a local PostgresSQL database.

[Docker Image](https://hub.docker.com/_/postgres)


