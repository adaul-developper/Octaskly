#!/bin/bash

# OCTASKLY - Quick Demo Script

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}[OCTASKLY - Demo & Showcase]${NC}"
echo -e "${BLUE}Offline Compute Task Coordinator${NC}"

echo ""
echo -e "${YELLOW}[1] Building project...${NC}"
cargo build --release --quiet
echo -e "${GREEN}[OK] Build complete${NC}"

echo ""
echo -e "${YELLOW}[2] Running unit tests...${NC}"
TEST_OUTPUT=$(cargo test --lib --quiet 2>&1)
PASSED=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= passed)' || echo "?")
echo -e "${GREEN}[OK] ${PASSED} tests passed${NC}"

echo ""
echo -e "${YELLOW}[3] Running integration tests...${NC}"
INTEGRATION=$(cargo test --test integration_tests --quiet 2>&1)
INT_PASSED=$(echo "$INTEGRATION" | grep -oP '\d+(?= passed)' || echo "?")
echo -e "${GREEN}[OK] ${INT_PASSED} integration tests passed${NC}"

echo ""
echo -e "${YELLOW}[4] Checking binary...${NC}"
BINARY="target/release/octaskly"
if [ -f "$BINARY" ]; then
    SIZE=$(ls -lh "$BINARY" | awk '{print $5}')
    echo -e "${GREEN}[OK] Binary ready: $BINARY ($SIZE)${NC}"
else
    echo -e "${RED}[ERROR] Binary not found${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}[5] Displaying help...${NC}"
echo ""
$BINARY --help | sed 's/^/  /'

echo ""
echo -e "${BLUE}═══════════════════════════════════════${NC}"
echo -e "${GREEN}[COMPLETE] OCTASKLY Implementation Finished${NC}"
echo -e "${BLUE}═══════════════════════════════════════${NC}"

echo ""
echo -e "${YELLOW}[Quick Start]${NC}"
echo ""
echo -e "  ${BLUE}Dispatcher Mode:${NC}"
echo "    $BINARY dispatcher --port 7878"
echo ""
echo -e "  ${BLUE}Worker Mode:${NC}"
echo "    $BINARY worker --name worker-01 --max-jobs 2"
echo ""

echo -e "${YELLOW}[Documentation]${NC}"
echo "  - README.md              (Overview & quick start)"
echo "  - IMPLEMENTATION.md      (Technical details)"
echo "  - FEATURES.md                   (Feature overview)"
echo "  - examples/basic_usage.rs (Usage examples)"
echo ""

echo -e "${YELLOW}[Testing]${NC}"
echo "  cargo test              (Run all tests)"
echo "  cargo test --lib        (Unit tests only)"
echo "  cargo test --test integration (Integration tests)"
echo ""

echo -e "${YELLOW}[Deployment]${NC}"
echo "  Binary location: $BINARY"
echo "  Size: $(ls -lh $BINARY | awk '{print $5}')"
echo "  Copy to any machine with Rust or use standalone"
echo ""

echo -e "${GREEN}[READY] Project is ready to use${NC}"
