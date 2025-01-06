#!/bin/bash

# 创建测试目录
mkdir -p test/source/subdir
mkdir -p test/destination

# 创建测试文件
dd if=/dev/urandom of=test/source/largefile.bin bs=1M count=100
echo "Hello, World!" > test/source/test.txt
echo "Test content" > test/source/subdir/subfile.txt

# 编译项目
cargo build

echo "Testing single file copy..."
# 测试单文件复制
./target/debug/cpui test/source/test.txt test/destination/

echo "Testing recursive directory copy..."
# 测试目录递归复制
./target/debug/cpui -r test/source test/destination/source_copy

# 验证文件是否正确复制
echo "Verifying files..."
if cmp -s test/source/test.txt test/destination/test.txt; then
    echo "Single file copy: SUCCESS"
else
    echo "Single file copy: FAILED"
fi

if [ -f test/destination/source_copy/largefile.bin ] && \
   [ -f test/destination/source_copy/test.txt ] && \
   [ -f test/destination/source_copy/subdir/subfile.txt ]; then
    echo "Directory copy: SUCCESS"
else
    echo "Directory copy: FAILED"
fi

# 清理测试文件
echo "Cleaning up..."
rm -rf test