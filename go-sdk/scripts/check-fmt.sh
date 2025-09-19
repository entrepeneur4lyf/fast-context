#!/bin/bash

# Check if go fmt needs to be run
if [ $(gofmt -l . | wc -l) -gt 0 ]; then
    echo "The following files are not formatted:"
    gofmt -l .
    echo "Please run 'make fmt' to format the code."
    exit 1
else
    echo "All files are properly formatted."
fi