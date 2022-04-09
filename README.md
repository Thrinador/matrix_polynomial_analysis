# Nonnegative matrix polynomial analysis

Three modes of operation will be supported.

- The first will take a polynomial and a given size of matrix, then evaluates if that polynomial probably preserves the nonnegativity of those matrices.
- The second will be given a polynomial and a given size of matrix, then mutate the polynomial to try and map out how you can minimize the coefficients and maximize the size of the negative values.
- Finally, the third mode will take a size of polynomial (number of terms) and size of matrix, then try and fully map out the cone of polynomials that preserve nonnegative values.
