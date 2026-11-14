"""Extract the original repeat sections of Let's Go Pikachu's Wwise soundtrack."""
import argparse
import ctypes
import ctypes.util
import struct
import wave
from pathlib import Path

# Verified against BGM_Common's loop segments and the released soundtrack.
TRACKS = {
    "pallet": (304455763, 39.17679166666667),
    "route1": (271425788, 22.912104166666666),
    "lab": (227537927, 30.473166666666668),
    "center": (947976592, 28.915458333333298),
    "forest": (749930822, 101.20460416666667),
    "pewter": (753202552, 57.8275208333333),
    "trainer": (276060617, 83.04985416666666),
    "wild": (353946460, 56.30497916666667),
}
RATE = 48_000
OPUS_DELAY = 120
# Leave 6 dB of headroom for the game mix before converting to PCM16.
GAIN = 0.5


def load_opus(path):
    opus = ctypes.CDLL(path)
    opus.opus_decoder_create.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
    opus.opus_decoder_create.restype = ctypes.c_void_p
    opus.opus_decoder_destroy.argtypes = [ctypes.c_void_p]
    opus.opus_decode_float.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int,
                                 ctypes.c_void_p, ctypes.c_int, ctypes.c_int]
    return opus


def extract(opus, source, out, seconds):
    data = source.read_bytes()
    if data[:4] != b"RIFF" or data[8:16] != b"WAVEfmt " or struct.unpack_from("<IH", data, 16) != (40, 0x3039):
        raise ValueError(f"{source}: expected Wwise Switch Opus")
    channels, sample_rate = struct.unpack_from("<HI", data, 22)
    samples = round(seconds * RATE)
    source_samples, _, _, seek_size = struct.unpack_from("<4I", data, 44)
    if channels != 2 or sample_rate != RATE or source_samples < samples:
        raise ValueError(f"{source}: unsupported channels, sample rate or loop size")
    chunk = 12
    while chunk + 8 <= len(data) and data[chunk:chunk + 4] != b"data":
        length = struct.unpack_from("<I", data, chunk + 4)[0]
        chunk += 8 + length + (length & 1)
    if data[chunk:chunk + 4] != b"data":
        raise ValueError(f"{source}: missing data chunk")
    pos = chunk + 8 + seek_size
    decoder = opus.opus_decoder_create(RATE, channels, None)
    if not decoder:
        raise ValueError("Could not create Opus decoder")
    pcm = bytearray()
    frame = (ctypes.c_float * (5760 * channels))()
    scaled = (ctypes.c_int16 * (5760 * channels))()
    frame_bytes = channels * 2
    try:
        while pos + 8 <= len(data) and len(pcm) < (samples + OPUS_DELAY) * frame_bytes:
            size = struct.unpack_from(">I", data, pos)[0]
            packet = data[pos + 8:pos + 8 + size]
            if not size or len(packet) != size:
                raise ValueError(f"{source}: truncated Opus packet")
            pos += 8 + size
            count = opus.opus_decode_float(decoder, packet, size, frame, 5760, 0)
            if count < 1:
                raise ValueError(f"{source}: Opus decoder returned {count}")
            for i in range(count * channels):
                scaled[i] = max(-32768, min(32767, round(frame[i] * (32768 * GAIN))))
            pcm.extend(ctypes.string_at(scaled, count * frame_bytes))
    finally:
        opus.opus_decoder_destroy(decoder)
    loop = pcm[OPUS_DELAY * frame_bytes:(OPUS_DELAY + samples) * frame_bytes]
    if len(loop) != samples * frame_bytes:
        raise ValueError(f"{source}: truncated loop")
    tmp = out.with_suffix(".tmp")
    with wave.open(str(tmp), "wb") as wav:
        wav.setparams((channels, 2, RATE, samples, "NONE", "not compressed"))
        wav.writeframes(loop)
    tmp.replace(out)
    print(f"{out.name}: {source.name}, {samples} samples, {samples / RATE:.3f} s")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("romfs", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--opus-lib", default=ctypes.util.find_library("opus"))
    args = parser.parse_args()
    if not args.opus_lib:
        parser.error("Install libopus or pass --opus-lib")
    opus = load_opus(args.opus_lib)
    bank = args.romfs / "bin/sound/NX64/wwise/bank"
    args.output.mkdir(parents=True, exist_ok=True)
    for name, (id, seconds) in TRACKS.items():
        extract(opus, bank / f"{id}.wem", args.output / f"{name}.wav", seconds)


if __name__ == "__main__":
    main()
