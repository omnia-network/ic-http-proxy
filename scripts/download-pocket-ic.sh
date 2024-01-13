#!/bin/bash

cd bin/

POCKET_IC_BIN=$(pwd)/pocket-ic
if [ -f "$POCKET_IC_BIN" ]; then
    echo -e "$POCKET_IC_BIN exists. Path: $POCKET_IC_BIN\n"
else 
    echo "$POCKET_IC_BIN does not exist."

    echo "Downloading Pocket IC binary..."
    if [[ "$OSTYPE" == "linux"* ]]; then
        ARCH="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        ARCH="darwin"
    else
        echo "Unsupported OS: $OSTYPE"
        exit 1
    fi
    curl -sLO https://download.dfinity.systems/ic/69e1408347723dbaa7a6cd2faa9b65c42abbe861/openssl-static-binaries/x86_64-$ARCH/pocket-ic.gz

    echo "Extracting Pocket IC binary..."
    gzip -d $POCKET_IC_BIN.gz
    chmod +x $POCKET_IC_BIN

    echo -e "Pocket IC binary downloaded and extracted successfully! Path: $POCKET_IC_BIN\n"
fi
