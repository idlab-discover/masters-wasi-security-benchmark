top dir is for microbenchmarks of host function call of policy 2 in my thesis
- this tests the time of host function calls with different policies.
- select what runtime needs to be tested via cargo features.

e2e dir is for end to end benchmarks for both policy 1 and 2 of my thesis.
- guests contain the tests to run
- policies contains the policies used during benchmarking
- test_runtimes are the runtimes that are being benchmarked