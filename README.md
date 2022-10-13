# Display::fmt rounding discrepancies

This is a basic program that looks for discrepancies in floating-point values rounded by `format!("{f:.prec$}")`,
for f64 `f` values.

The rounded values are compared to a naive string-based rounding. This benchmark is not infaillible since it uses
the full Display representation of the floating-point values, though a visual inspection on a number of tests hasn't
revealed any issue. At worst there might be a few false positives or negatives, but far fewer than `Display::fmt` 
errors.

Usage:

Usage: `rounding [-v][-n] [depth]`

* `depth` : max number of digits in the fractional part in the test (default = 6)
* `-v` : verbose output
* `-n` : negative values (by default, the test is performed on positive values)

Observed results: 

=> 5555555 / 22222222 error(s) for depth 0-8, so 25.0 %

The ratio is 5/22 for all tested depths.

# LICENSE

Copyright (c) 2022 Redglyph, All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
   disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following
   disclaimer in the documentation and/or other materials provided with the distribution.
3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products
   derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.