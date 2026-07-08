#!/bin/bash
echo "Step 1" && \
false && \
echo "Step 2" && \
if [ 1 -eq 1 ]; then exit 1; fi && \
echo "Step 3" || true
