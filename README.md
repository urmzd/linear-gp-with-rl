
# Linear Genetic Programming

This repository contains a framework for solving tasks using linear genetic programming.

![build passing](https://github.com/urmzd/linear-genetic-programming/actions/workflows/build.yml/badge.svg)

## Prerequisites

To set up the environment and dependencies, follow the instructions below:

```bash
# Install required packages
sudo apt-get install docker
sudo apt-get install rust
sudo apt-get install docker-compose
sudo apt-get install python
...
```

## Usage

1. Setup the environment:


```bash
docker-compose up -d

python -m venv venv
pip install -r scripts/requirements.txt

cargo build --release
```

2. Execute the search script.
```bash
# Display help
./scripts/search.py -h # help

# Search for the best parameters for a specific environment
./scripts/search.py --env cart-pole-lgp --n-trials 40 --n-threads 4  

# Search for the best parameters for all environments
./scripts/search_all.sh
```

## Testing

To run all tests, execute the following command.

```bash
cargo nextest run --no-fail-fast --release --no-capture
```

## Contributions
Contributions are welcome. Please refer to the guidelines in [CONTRIBUTING.md](./CONTRIBUTING.md) for more information.
