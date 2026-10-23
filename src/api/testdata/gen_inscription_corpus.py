"""Generates inscription_corpus.txt: base64 inscription payloads covering the base64, nada and
zstd decoding edge cases of decode_bytes_from_inscription_data. Requires the zstd CLI.

Usage: python3 -I gen_inscription_corpus.py > inscription_corpus.txt
"""

import base64
import subprocess
import tempfile


def zstd(data, *args, stdin=False):
    if stdin:
        # Compressing from a pipe leaves the frame content size out of the header.
        return subprocess.run(["zstd", "-q", "-c", *args], input=data, capture_output=True, check=True).stdout
    with tempfile.NamedTemporaryFile() as f:
        f.write(data)
        f.flush()
        return subprocess.run(["zstd", "-q", "-c", *args, f.name], capture_output=True, check=True).stdout


def b64(data, pad=False):
    encoded = base64.b64encode(data).decode()
    return encoded if pad else encoded.rstrip("=")


payload = b"brc20 inscription payload " * 8
cases = []
add = lambda name, value: cases.append((name, value))

# base64 layer
add("raw_short", b64(b"\x00hello"))
add("raw_flag_only", b64(b"\x00"))
add("raw_padded", b64(b"\x00hey!", pad=True))
add("raw_padding_then_garbage", b64(b"\x00hey!", pad=True) + "garbage")
add("raw_noncanonical_trailing_bits", b64(b"\x00hey!")[:-1] + "F")
add("raw_whitespace", "AGhl eSE")
add("raw_url_safe_alphabet", b64(b"\x00\xfb\xff\xbf")[:-2] + "-_")
add("raw_invalid_length", "AGhle")
add("raw_only_padding", "=")
add("empty", "")
add("unknown_method", b64(b"\x03hello"))

# zstd layer
add("zstd_with_size", b64(b"\x02" + zstd(payload, "-19", "--no-check")))
add("zstd_no_size", b64(b"\x02" + zstd(payload, "--no-check", stdin=True)))
add("zstd_checksum", b64(b"\x02" + zstd(payload, "-C")))
bad = bytearray(zstd(payload, "-C"))
bad[-1] ^= 0xFF
add("zstd_checksum_mismatch", b64(b"\x02" + bytes(bad)))
add("zstd_two_frames", b64(b"\x02" + zstd(payload, "--no-check") + zstd(b"second frame", "--no-check")))
skippable = b"\x50\x2a\x4d\x18" + (4).to_bytes(4, "little") + b"skip"
add("zstd_skippable_then_frame", b64(b"\x02" + skippable + zstd(payload, "--no-check")))
add("zstd_only_skippable", b64(b"\x02" + skippable))
add("zstd_truncated", b64(b"\x02" + zstd(payload, "--no-check")[:-5]))
add("zstd_long_window", b64(b"\x02" + zstd(payload, "--long=30", "--no-check")))
add("zstd_over_limit_with_size", b64(b"\x02" + zstd(b"\x00" * (2 * 1024 * 1024), "--no-check")))
add("zstd_over_limit_no_size", b64(b"\x02" + zstd(b"\x00" * (2 * 1024 * 1024), "--no-check", stdin=True)))
add("zstd_at_limit", b64(b"\x02" + zstd(b"\x01" * (1024 * 1024), "--no-check")))
add("zstd_one_over_limit", b64(b"\x02" + zstd(b"\x01" * (1024 * 1024 + 1), "--no-check")))
dict_header = bytearray(zstd(payload, "--no-check"))
dict_header[4] |= 0x01  # Frame_Header_Descriptor: Dictionary_ID_Flag = 1
add("zstd_dictionary_id_flag", b64(b"\x02" + bytes(dict_header)))
add("zstd_legacy_v07_magic", b64(b"\x02\x27\xb5\x2f\xfd" + b"\x00" * 16))

for name, value in cases:
    print(name, value)
