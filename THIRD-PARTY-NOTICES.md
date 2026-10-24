# Third-party notices

The `zam` compiler contains the source code of the libraries below. It compiles a library into your program only when your program uses the standard library module that needs it.

| Library | Version | Used by | License |
| --- | --- | --- | --- |
| [Mbed TLS](https://github.com/Mbed-TLS/mbedtls) | 3.6.7 | `std/tls`, and `https://` in `std/http` | Apache-2.0 (Mbed TLS is offered under Apache-2.0 OR GPL-2.0-or-later; Zamak uses it under Apache-2.0) |
| [libui-ng](https://github.com/libui-ng/libui-ng) | commit `43ba1ef` | `std/ui` | MIT |
| [SQLite](https://sqlite.org) | 3.54.1 | `std/sqlite` | Public domain |

If you distribute a program that uses `std/tls`, `https://`, or `std/ui`, include the matching notice below with it.

## Mbed TLS

Copyright The Mbed TLS Contributors.

Licensed under the Apache License, Version 2.0. The full text of the license is in [LICENSE-APACHE](LICENSE-APACHE), next to this file. You may not use these files except in compliance with the License. Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

## libui-ng

Copyright (c) 2022 libui-ng authors

Copyright (c) 2014 Pietro Gagliardi

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## SQLite

The author disclaims copyright to this source code. In place of a legal notice, here is a blessing:

- May you do good and not evil.
- May you find forgiveness for yourself and forgive others.
- May you share freely, never taking more than you give.
