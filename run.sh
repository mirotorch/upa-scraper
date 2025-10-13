#!/bin/bash
./target/release/get_urls
cat urls.txt | head -n 10 | ./target/release/get_product_data