# Yet Another CSV Validators Combinator

## Introduction

Much like nom, this library is a combinator for CSV validators. It is written in rust and is intended to be used in python.
The heavy lifting is done by rust and the python interface is a thin wrapper around it.

## TODO

- [ ] publish a list of validators with parameters
- [ ] pass in a filename or a buffer/iterator/stream
- [ ] return a list of errors
- [ ] have rust handle file reading
- [ ] add http endpoint (rust?) for requesting validation (synchronous if fast enough)
- [ ] validator datatype?
- [ ] add a way to construct a custom validator? -> polars expression?
- [ ] implement both mmap/seq async reader
- [ ] accept these readers: RCPReader, stdin, file, buffer, stream
- 