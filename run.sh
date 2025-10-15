#!/bin/bash
./target/release/get_urls > url_test.txt
cat url_test.txt | head -n 10 | ./target/release/get_product_data