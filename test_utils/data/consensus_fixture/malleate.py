"""Rewrites a signed EIP-155 legacy transaction into its high-s twin (s' = n - s, parity flipped).

Both encodings recover the same sender; the engine must treat them exactly as it always has.
Usage: python3 -I malleate.py <raw tx hex>
"""

import sys

SECP256K1_N = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141


def rlp_decode(data, i=0):
    prefix = data[i]
    if prefix < 0x80:
        return data[i : i + 1], i + 1
    if prefix < 0xB8:
        end = i + 1 + prefix - 0x80
        return data[i + 1 : end], end
    if prefix < 0xC0:
        n = prefix - 0xB7
        length = int.from_bytes(data[i + 1 : i + 1 + n], "big")
        start = i + 1 + n
        return data[start : start + length], start + length
    if prefix < 0xF8:
        length, start = prefix - 0xC0, i + 1
    else:
        n = prefix - 0xF7
        length = int.from_bytes(data[i + 1 : i + 1 + n], "big")
        start = i + 1 + n
    items, j = [], start
    while j < start + length:
        item, j = rlp_decode(data, j)
        items.append(item)
    return items, j


def rlp_length(length, offset):
    if length < 56:
        return bytes([offset + length])
    encoded = length.to_bytes((length.bit_length() + 7) // 8, "big")
    return bytes([offset + 55 + len(encoded)]) + encoded


def rlp_encode(item):
    if isinstance(item, list):
        body = b"".join(rlp_encode(x) for x in item)
        return rlp_length(len(body), 0xC0) + body
    if len(item) == 1 and item[0] < 0x80:
        return item
    return rlp_length(len(item), 0x80) + item


def int_bytes(value):
    return value.to_bytes((value.bit_length() + 7) // 8, "big")


raw = bytes.fromhex(sys.argv[1].removeprefix("0x"))
fields, _ = rlp_decode(raw)
v = int.from_bytes(fields[6], "big")
s = int.from_bytes(fields[8], "big")
fields[6] = int_bytes(v + 1 if (v - 35) % 2 == 0 else v - 1)
fields[8] = int_bytes(SECP256K1_N - s)
print("0x" + rlp_encode(fields).hex())
