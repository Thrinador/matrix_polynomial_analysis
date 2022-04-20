# Nonnegative matrix polynomial analysis

Three modes of operation are supported.

- The first takes a polynomial and a given size of matrix, then evaluates if that polynomial probably preserves the nonnegativity of those matrices.
- The second is given a polynomial and a given size of matrix, then mutates the polynomial to try and map out how you can minimize the coefficients and maximize the size of the negative values.
- Finally, the third mode will take a size of polynomial (number of terms) and size of matrix, then try and fully map out the cone of polynomials that preserve nonnegative values.

## Startup file

This program is built from the starting json file. The following are an explanation of how the arguments in the startup file work.

### starting_polynomial

Vector of f64 floats

A polynomial as a list of its coefficients. The first term is the largest i.e. [1,2,3] -> 1x^2 + 2x + 3.

### matrix_size

Single usize (generally 32 unsigned bit integer) value.

Size of matrices to evaluate against.

### matrices_to_fuzz

Single usize (generally 32 unsigned bit integer) value.

The number of random matrices to test against for each type of test.

### mutated_polynomials_to_evaluate

Single usize (generally 32 unsigned bit integer) value.

The number of mutated polynomials to generate based off of the `starting_polynomial`. Note that this is not the number of polynomials returned, but instead the number to start working off of. The number returned varies wildly since only polynomials with negative coefficients are returned and each of the mutated polynomials gets every combination of coefficients reduced.

### coefficients_in_polynomial

Single usize (generally 32 unsigned bit integer) value.

The size of the polynomials that are being generated to map out a space. Only used in `mode` 3

### mode

Single usize (generally 32 unsigned bit integer) value.

Which mode should this run as? Options are mode 1, 2, 3:

- Mode 1: Tests the `starting_polynomial` against matrices of size `matrix_size`.
- Mode 2: Returns a set of mutated polynomials constructed from the `starting_polynomial` that are likely nonnegative for matrices of size `matrix_size`.
- Mode 3: Returns a snapshot of what the space of polynomials with `number_of_coefficients_in_polynomial` terms looks like against `matrix_size` matrices returns a snapshot of what that space
