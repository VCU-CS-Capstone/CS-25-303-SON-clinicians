# CS25-303 Test Data

This is a test data generator for the CS25-30X projects.



## Adding test data.

All data inside the `random/sets/` directory contains test data that is selected randomly to create the test data.


## Generating test data.

### Step 1: Setup a postgres database.
I recommend you use a docker container to be the database.

[Postgres Docker](https://hub.docker.com/_/postgres)

### Step 2: Compiling the generator.

Read the steps in the parent directory to compile this project and once you have the dev environment setup, you can compile the generator by running the following command.

```bash
cargo build
```

You will now have the binary in the `target/debug/` directory.
File will be named `cs25-303-data-tools` or `cs25-303-data-tools.exe` depending on your OS.

### Step 3: Running the generator.

You can either create a config file or use cli arguments to run the generator.

```toml
# config.toml
[database]
user = "postgres"
password = "password"
database = "cs25_303"
host = "localhost:5432"
```

```bash
./target/debug/cs25-303-data-tools
```
