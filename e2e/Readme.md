# End to End Tests

The e2e tests are built using the [interchaintest](https://github.com/strangelove-ventures/interchaintest) library by Strangelove. It runs multiple docker container validators, and lets you test IBC enabled smart contracts.

These end to end tests are designed to run in the ci, but you can also run them locally.

## Running the tests locally

The end to end tests are currently split into two parts:

### ICA Contract Tests

These tests are designed to test the ICA contract itself and its interaction with the relayer.

All contract tests are located in `interchaintest/contract_test.go` file. Currently, there is only one test in this file:

- `TestMintAndExecute`

<!-- (These three tests used to be one monolithic test, but they were split into three in order to run them in parallel in the CI.) -->

To run the tests locally, run the following commands from this directory:

```text
cd interchaintest/
go test -v . -run TestWithContractTestSuite -testify.m $TEST_NAME
```

where `$TEST_NAME` is one of the four tests listed above.

Before running the tests, you must have built the optimized contract in the `/artifacts` directory. To do this, run the following command from the root of the repository:

```text
cargo run-script optimize
```

## In the CI

The tests are run in the github CI after every push to the `main` branch. See the [github actions workflow](https://github.com/srdtrk/cw-ica-controller/blob/main/.github/workflows/e2e.yml) for more details.

## About the tests

The tests are currently run on wasmd `v0.45.0` and ibc-go `v7.3.0`'s simd.
