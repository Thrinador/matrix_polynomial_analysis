# Nonnegative matrix polynomial analysis

Three modes of operation will be supported.

- The first will take a polynomial and a given size of matrix, then evaluates if that polynomial probably preserves the nonnegativity of those matrices.
- The second will be given a polynomial and a given size of matrix, then mutate the polynomial to try and map out how you can minimize the coefficients and maximize the size of the negative values.
- Finally, the third mode will take a size of polynomial (number of terms) and size of matrix, then try and fully map out the cone of polynomials that preserve nonnegative values.

## Flags

### starting_polynomial

Vector of f64 floats

Default: [1.0,1.0,1.0,1.0,1.0]

A polynomial as a list of its coefficients. The first term is the largest i.e. [1,2,3] -> 1x^2 + 2x + 3.

### matrix_size

Single usize (generally 32 unsigned bit integer) value.

Default: 2

Size of matrices to evaluate against.

### number_of_matrices_to_fuzz

Single usize (generally 32 unsigned bit integer) value.

Default: 1000

The number of random matrices to test against for each type of test.

### number_of_mutated_polynomials_to_evaluate

Single usize (generally 32 unsigned bit integer) value.

Default: 10

The number of mutated polynomials to generate based off of the `starting_polynomial`. Note that this is not the number of polynomials returned, but instead the number to start working off of. The number returned varies wildly since only polynomials with negative coefficients are returned.

### number_of_coefficients_in_polynomial

Single usize (generally 32 unsigned bit integer) value.

Default: 5

The size of the polynomials that are being generated to map out a space. Only used in `mode` 3

### mode

Single usize (generally 32 unsigned bit integer) value.

Default: 1

Which mode should this run as? Options are mode 1, 2, 3:

- Mode 1: Tests the `starting_polynomial` against matrices of size `matrix_size`.
- Mode 2: Returns a set of mutated polynomials constructed from the `starting_polynomial` that are likely nonnegative for matrices of size `matrix_size`.
- Mode 3: Returns a snapshot of what the space of polynomials with `number_of_coefficients_in_polynomial` terms looks like against `matrix_size` matrices returns a snapshot of what that space

## Examples runs

Mode 1; `matrix_analysis --starting_polynomial=[0,1,2,3,4] --matrix_size=3 --mode=1`

Mode 2; `matrix_analysis --starting_polynomial=[0,1,2,3,4] --number_of_mutations=10 --mode=2`.
