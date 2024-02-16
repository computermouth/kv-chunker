#!/bin/bash -ex

INFLNAME=${1}
FILENAME=$(basename ${INFLNAME})
PASSWORD=${2}
SIZE_TOTAL=$(stat --printf="%s" ${INFLNAME})
SIZE_REMAINING=${SIZE_TOTAL}
CHUNK_SIZE=26214400  # 25 MiB

COUNT=0

while [ ${SIZE_REMAINING} != 0 ]; do

    if [ ${CHUNK_SIZE} -gt ${SIZE_REMAINING} ]; then
        CHUNK_SIZE=${SIZE_REMAINING}
    fi

    SKIP=$((${SIZE_TOTAL}-${SIZE_REMAINING}))

    if [ ${CHUNK_SIZE} -eq 26214400 ]; then
        dd if=${INFLNAME} bs=${CHUNK_SIZE} count=1 skip=$((${SKIP}/${CHUNK_SIZE})) | \
        curl -i -XPUT -H "pw: ${PASSWORD}" -H "sz: ${CHUNK_SIZE}" -H "pc: ${COUNT}" -H "pcs: $((${COUNT}+1))" -H "name: ${FILENAME}" https://steadily-sound-boar.edgecompute.app --data-binary @-
    else
        dd if=${INFLNAME} bs=1 count=${CHUNK_SIZE} skip=${SKIP} | \
        curl -i -XPUT -H "pw: ${PASSWORD}" -H "sz: ${CHUNK_SIZE}" -H "pc: ${COUNT}" -H "pcs: $((${COUNT}+1))" -H "name: ${FILENAME}" https://steadily-sound-boar.edgecompute.app --data-binary @-
    fi

    SIZE_REMAINING=$((${SIZE_REMAINING}-${CHUNK_SIZE}))
    COUNT=$((${COUNT}+1))

done